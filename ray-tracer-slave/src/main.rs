use bvh::bvh::BVH;
use bvh::ray::Ray;
use bvh::{Point3, Vector3};
use log::info;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand_distr::{Distribution, UnitSphere};
use ray_tracer_interface::{
    camera,
    color::{self, Color},
    shapes::{mesh::Triangle, Object, WorldList, WorldRefList},
    RenderInfo,
};
use rayon::prelude::{IndexedParallelIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::f32::consts::PI;
use tiny_http::{Response, Server};

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

fn main() {
    let aspect_ratio = 16f32 / 9f32;
    let image_height = 360f32;
    let max_bounces = 10;
    let image_width = aspect_ratio * image_height;
    pretty_env_logger::init();

    let camera = camera::Camera::new(
        Point3::ZERO,
        aspect_ratio,
        0.1f32,
        1f32,
        PI / 2f32,
        1f32,
        image_height,
    );
    let sample_count: u32 = 5;

    let mut img_buff = vec![0u8; image_width as usize * image_height as usize * 3];
    let server = Server::http("0.0.0.0:8000").unwrap();

    for mut request in server.incoming_requests() {
        let mut body = String::new();
        request.as_reader().read_to_string(&mut body).unwrap();
        let mut req: RenderInfo = serde_json::from_str(&body).unwrap();
        let bvh = BVH::build(&mut req.world);
        let world = WorldList::from_vec(req.world);
        info!("Got request");
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
                        pix_color += ray_color(&r, &world, max_bounces + 1, &mut rng, &bvh);
                    }
                    pix_color.r = (pix_color.r / sample_count as f32).sqrt();
                    pix_color.g = (pix_color.g / sample_count as f32).sqrt();
                    pix_color.b = (pix_color.b / sample_count as f32).sqrt();
                    [p[0], p[1], p[2]] = pix_color.as_slice();
                }
            });
        let response = Response::from_string(
            json!({
                "output": img_buff,
            })
            .to_string(),
        );
        request.respond(response).unwrap();
    }
}
