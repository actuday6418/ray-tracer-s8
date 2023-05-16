use actix_web::web::Bytes;
use actix_web::{post, web, App, HttpServer, Responder};
use image::{ImageBuffer, Rgb};
use log::info;
mod obj;
use futures::future;
use ray_tracer_interface::{ImageSlice, RenderInfo, RenderMeta};
use reqwest::Client;
use serde_json::json;
use std::sync::RwLock;
use uuid::Uuid;

struct Job {
    render_meta: RenderMeta,
    result: Vec<ImageSlice>,
}

struct AppState {
    jobs: Vec<Job>,
}

#[post("/upload/{obj_size}/")]
async fn index(
    body: Bytes,
    path: web::Path<usize>,
    state: web::Data<RwLock<AppState>>,
) -> impl Responder {
    info!("Got request");
    let obj_size = path.into_inner();
    let body = body.to_vec();
    let id = Uuid::new_v4();
    let client = Client::new();
    let divisions = 20;
    let render_meta = RenderMeta {
        height: 1080,
        width: 1920,
        divisions,
        id,
    };
    let mut state = state.write().unwrap();
    state.jobs.push(Job {
        result: Vec::new(),
        render_meta: render_meta.clone(),
    });
    info!("metadata extraction complete");
    let world = obj::build_world(body, obj_size);
    future::join_all((0..divisions).map(|division_no| {
        let client = &client;
        let world = &world;
        let render_meta = &render_meta;
        async move {
            info!("Dispatch to slave {}", division_no + 1);
            info!(
                "Response from slave: {}",
                client
                    .post("http://slave:8081")
                    .body(
                        json!(RenderInfo {
                            division_no,
                            render_meta: render_meta.clone(),
                            world: world.clone(),
                        })
                        .to_string(),
                    )
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .send()
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap_or_default()
            );
        }
    }))
    .await;
    id.to_string()
}

#[post("/result")]
async fn result(req: web::Json<ImageSlice>, state: web::Data<RwLock<AppState>>) -> impl Responder {
    info!(
        "Got results from slave: id {} no {}",
        req.id, req.division_no
    );
    let id = req.id;
    let mut state = state.write().unwrap();
    if let Some(idx) = state.jobs.iter().position(|job| job.render_meta.id == id) {
        state.jobs[idx].result.push(req.into_inner());
    } else {
        info!("result not saved. ID wrong? : {}", id);
    }
    "slice saved. thank you slave."
}

#[post("/poll")]
async fn poll(req: String, state: web::Data<RwLock<AppState>>) -> Bytes {
    info!("poll {}", req);
    let mut state = state.write().unwrap();
    Uuid::parse_str(&req)
        .map(|id| {
            if let Some(idx) = state.jobs.iter().position(|job| {
                info!("{}", job.render_meta.id);
                job.render_meta.id == id
            }) {
                let job = &mut state.jobs[idx];
                if (0..job.render_meta.divisions)
                    .all(|i| job.result.iter().find(|res| res.division_no == i).is_some())
                {
                    job.result.sort_by(|a, b| a.division_no.cmp(&b.division_no));
                    let res: Vec<u8> = job
                        .result
                        .iter()
                        .map(|a| a.image.clone())
                        .flatten()
                        .collect();
                    let mut c = std::io::Cursor::new(Vec::new());
                    let img: ImageBuffer<Rgb<u8>, _> =
                        ImageBuffer::from_vec(job.render_meta.width, job.render_meta.height, res)
                            .unwrap();
                    img.write_to(&mut c, image::ImageOutputFormat::Jpeg(90))
                        .unwrap();
                    state.jobs.remove(idx);
                    Bytes::from(c.into_inner())
                } else {
                    Bytes::from(format!(
                        "Job not finished yet {}/{}",
                        (0..job.render_meta.divisions)
                            .filter(|i| job
                                .result
                                .iter()
                                .find(|res| res.division_no == *i)
                                .is_some())
                            .count(),
                        job.render_meta.divisions
                    ))
                }
            } else {
                Bytes::from_static(b"No such job")
            }
        })
        .unwrap_or(Bytes::from_static(b"Invalid Uuid"))
}

#[actix_web::main]
async fn main() {
    pretty_env_logger::init();
    let state = web::Data::new(RwLock::new(AppState { jobs: Vec::new() }));

    HttpServer::new(move || {
        App::new()
            .service(index)
            .service(poll)
            .service(result)
            .app_data(state.clone())
            .app_data(web::PayloadConfig::new(500_000_000))
            .app_data(web::JsonConfig::default().limit(500_000_000))
    })
    .bind(("0.0.0.0", 8080))
    .unwrap()
    .run()
    .await
    .unwrap();
}
