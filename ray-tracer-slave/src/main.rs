use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use bvh::bvh::BVH;
use bvh::ray::Ray;
use bvh::{Point3, Vector3};
use crossbeam_channel::unbounded;
use crossbeam_channel::{Receiver, Sender};
use log::info;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand_distr::{Distribution, UnitSphere};
use ray_tracer_interface::ImageSlice;
use ray_tracer_interface::{
    camera,
    color::{self, Color},
    shapes::{mesh::Triangle, Object, WorldList, WorldRefList},
    RenderInfo,
};
use rayon::prelude::{IndexedParallelIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use reqwest::blocking::Client;
use serde_json::json;
use std::f32::consts::PI;

enum MessageToWorker {
    NewJob(RenderInfo),
}

enum MessageToServer {
    Output(Option<Vec<u8>>),
}

struct AppState {
    rx: Receiver<MessageToServer>,
    tx: Sender<MessageToWorker>,
}

fn worker(rx: Receiver<MessageToWorker>, tx: Sender<MessageToServer>) {
    let client = Client::new();
    loop {
        if let Ok(message) = rx.recv() {
            match message {
                MessageToWorker::NewJob(mut req) => {
                    info!("Got job");
                    let max_bounces = 10;
                    let image_height = req.render_meta.height;
                    let image_width = req.render_meta.width;
                    let camera = camera::Camera::new(
                        Point3::ZERO,
                        image_width as f32 / image_height as f32,
                        0.1f32,
                        1f32,
                        PI / 2f32,
                        1f32,
                        image_height as f32,
                    );
                    let sample_count: u32 = 100;

                    let mut img_buff = vec![
                        0u8;
                        (image_height as usize
                            / req.render_meta.divisions as usize)
                            * image_width as usize
                            * 3
                    ];
                    let bvh = BVH::build(&mut req.world);
                    let world = WorldList::from_vec(req.world);
                    img_buff
                        .par_chunks_exact_mut(image_width as usize * 3)
                        .enumerate()
                        .for_each(|(y, row)| {
                            let mut rng = SmallRng::from_entropy();
                            for (x, p) in row.chunks_exact_mut(3).enumerate() {
                                let y = image_height as usize - y - 1;
                                let mut pix_color = color::BLACK;
                                for _ in 0..sample_count {
                                    let r = camera.get_ray(x as u32, y as u32, &mut rng);
                                    pix_color +=
                                        ray_color(&r, &world, max_bounces + 1, &mut rng, &bvh);
                                }
                                pix_color.r = (pix_color.r / sample_count as f32).sqrt();
                                pix_color.g = (pix_color.g / sample_count as f32).sqrt();
                                pix_color.b = (pix_color.b / sample_count as f32).sqrt();
                                [p[0], p[1], p[2]] = pix_color.as_slice();
                            }
                        });
                    info!("render finished");
                    let p = json!(ImageSlice {
                        id: req.render_meta.id,
                        image: img_buff,
                        division_no: req.division_no
                    })
                    .to_string();
                    let mut f = std::fs::File::create("t.json").unwrap();
                    use std::io::Write;
                    f.write_all(p.as_bytes()).unwrap();
                    info!(
                        "master responded to result:  {}",
                        client
                            .post("http://master:8080/result")
                            .body(p,)
                            .header(reqwest::header::CONTENT_TYPE, "application/json")
                            .send()
                            .unwrap()
                            .text()
                            .unwrap_or_default()
                    );
                }
            }
        }
    }
}

fn ray_color(ray: &Ray, world: &WorldList, depth: u32, rng: &mut SmallRng, bvh: &BVH) -> Color {
    if depth == 0 {
        return color::BLACK;
    }
    let v = world.get();
    let world_sub = bvh.traverse(ray, &v);
    match WorldRefList::from_vec(world_sub).intersect(&ray) {
        Some(table) => {
            if table.emission > 0f32 {
                table.emission * table.albedo
            } else {
                let diffuse_dir = Vector3::from_slice(&UnitSphere.sample(rng)) + table.normal;
                let glossy_dir =
                    ray.direction - 2f32 * ray.direction.dot(table.normal) * table.normal;
                let scatter_direction = diffuse_dir + table.roughness * (glossy_dir - diffuse_dir);
                table.albedo.blend(&ray_color(
                    &Ray::new(
                        table.point,
                        scatter_direction.try_normalize().unwrap_or(table.normal),
                    ),
                    world,
                    depth - 1,
                    rng,
                    bvh,
                ))
            }
        }
        None => {
            let t = ray.direction.normalize_or_zero().y * 0.5 + 1f32;
            t * color::WHITE
                + (1f32 - t)
                    * Color {
                        r: 0.3,
                        g: 0.3,
                        b: 0.8f32,
                    }
        }
    }
}

#[post("/")]
async fn index(req: web::Json<RenderInfo>, state: web::Data<AppState>) -> impl Responder {
    info!("Got request");
    let req = req.into_inner();
    state.tx.send(MessageToWorker::NewJob(req)).unwrap();
    "i'll get you a slice at once"
}

#[actix_web::main]
async fn main() {
    pretty_env_logger::init();
    let (txw, rxw) = unbounded::<MessageToWorker>();
    let (txs, rxs) = unbounded::<MessageToServer>();
    std::thread::spawn(|| worker(rxw, txs));
    let state = web::Data::new(AppState { rx: rxs, tx: txw });

    HttpServer::new(move || App::new().service(index).app_data(state.clone()))
        .bind(("0.0.0.0", 8081))
        .unwrap()
        .run()
        .await
        .unwrap();
}
