use bvh::bvh::BVH;
use bvh::{Point3, Vector3};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand_distr::{Distribution, UnitSphere};
mod camera;
mod color;
mod shapes;
use bvh::ray::Ray;
use color::Color;
use rayon::prelude::{IndexedParallelIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use shapes::mesh::Triangle;
use std::{f32::consts::PI, sync::mpsc, thread::spawn};
mod gui;
use shapes::{Object, PropertyAt, WorldList, WorldRefList};

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

pub enum MessageToGUI {
    Rendered(eframe::egui::ColorImage, u128),
}

fn main() {
    let (txa, rxa) = mpsc::channel::<gui::MessageToRender>();
    let (txb, rxb) = mpsc::channel::<MessageToGUI>();
    spawn(move || render(rxa, txb));
    gui::launch(txa, rxb);
}

fn build_world() -> Vec<Object> {
    let mut world = vec![];
    if let Ok((models, Ok(materials))) = tobj::load_obj(
        "/home/actuday/temp/untitled.obj",
        &tobj::LoadOptions::default(),
    ) {
        for m in models.iter() {
            let mesh = &m.mesh;
            let material = &materials[mesh.material_id.unwrap()];
            for i in 0..mesh.indices.len() / 3 {
                let a = mesh.indices[3 * i];
                let b = mesh.indices[3 * i + 1];
                let c = mesh.indices[3 * i + 2];
                world.push(Object::Triangle(Triangle::new(
                    Point3::new(
                        mesh.positions[3 * a as usize],
                        mesh.positions[3 * a as usize + 1],
                        mesh.positions[3 * a as usize + 2],
                    ),
                    Point3::new(
                        mesh.positions[3 * b as usize],
                        mesh.positions[3 * b as usize + 1],
                        mesh.positions[3 * b as usize + 2],
                    ),
                    Point3::new(
                        mesh.positions[3 * c as usize],
                        mesh.positions[3 * c as usize + 1],
                        mesh.positions[3 * c as usize + 2],
                    ),
                    PropertyAt::Value((material.shininess as f32) / 1000f32),
                    PropertyAt::Value(Color::from_slice(material.diffuse)),
                    PropertyAt::Value(0f32),
                )))
            }
        }
    } else {
        panic!("Failed to load obj or mtl file")
    }
    world
}

fn render(rx: mpsc::Receiver<gui::MessageToRender>, tx: mpsc::Sender<MessageToGUI>) {
    let aspect_ratio = 16f32 / 9f32;
    let image_height = 360f32;
    let mut max_bounces = 10;
    let image_width = aspect_ratio * image_height;

    let mut camera = camera::Camera::new(
        Point3::ZERO,
        aspect_ratio,
        0.1f32,
        1f32,
        PI / 2f32,
        1f32,
        image_height,
    );
    let mut sample_count: u32 = 5;

    let mut img_buff = vec![0u8; image_width as usize * image_height as usize * 3];
    let mut world = build_world();
    let mut bvh = BVH::build(&mut world);
    let mut world = WorldList::from_vec(world);

    loop {
        match rx.recv() {
            Ok(gui::MessageToRender::UpdateDepth(depth)) => {
                max_bounces = depth;
            }
            Ok(gui::MessageToRender::ReloadWorld) => {
                let mut world_t = build_world();
                bvh = BVH::build(&mut world_t);
                world = WorldList::from_vec(world_t);
            }
            Ok(gui::MessageToRender::Render) => {
                let now = std::time::Instant::now();

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
                let egui_img = eframe::egui::ColorImage::from_rgb(
                    [image_width as usize, image_height as usize],
                    &img_buff,
                );
                tx.send(MessageToGUI::Rendered(egui_img, now.elapsed().as_millis()))
                    .unwrap();
            }
            Ok(gui::MessageToRender::UpdateCameraOrigin(origin)) => camera.set_origin(origin),
            Ok(gui::MessageToRender::UpdateCameraFieldOfView(field_of_view)) => {
                camera.set_field_of_view(field_of_view)
            }
            Ok(gui::MessageToRender::UpdateSampleCount(sample_count_new)) => {
                sample_count = sample_count_new
            }
            Ok(gui::MessageToRender::UpdateCameraAperture(aperture)) => {
                camera.set_aperture(aperture)
            }
            Ok(gui::MessageToRender::UpdateCameraFocalLength(focal_length)) => {
                camera.set_focal_length(focal_length)
            }
            Ok(gui::MessageToRender::UpdateCameraFocusDistance(focus_dstance)) => {
                camera.set_focus_distance(focus_dstance)
            }
            Err(_) => return,
        }
    }
}
