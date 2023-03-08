use bvh::bvh::BVH;
use bvh::{Point3, Vector3};
use rand::rngs::SmallRng;
use rand::Rng;
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
use shapes::sphere::Sphere;
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
        // match world.to_ref().intersect(&ray) {
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
        None =>
        // color::BLACK,
        {
            let t = ray.direction.normalize_or_zero().y;
            (1f32 - t)
                * Color {
                    r: 0.01,
                    g: 0.02,
                    b: 0.07,
                }
                + t * Color {
                    r: 0.02,
                    g: 0.04,
                    b: 0.1,
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

fn render(rx: mpsc::Receiver<gui::MessageToRender>, tx: mpsc::Sender<MessageToGUI>) {
    let aspect_ratio = 16f32 / 9f32;
    let image_height = 360f32;
    let max_bounces = 10;
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
    let mut rng = SmallRng::from_seed([0; 32]);
    let mut world = vec![
        Object::Triangle(Triangle::new(
            Point3::new(-1.93f32, 0.1f32, -0.5f32),
            Point3::new(-2.1f32, 1f32, -0.14f32),
            Point3::new(-2.6f32, -0.82f32, -0.87f32),
            PropertyAt::Value(0.0),
            PropertyAt::Value(color::RED),
            PropertyAt::Value(20f32),
        )),
        Object::Triangle(Triangle::new(
            Point3::new(-2.8f32, -0.83f32, -0.43f32),
            Point3::new(-2.1f32, 1f32, -0.14f32),
            Point3::new(-2.6f32, -0.82f32, -0.87f32),
            PropertyAt::Value(0.0),
            PropertyAt::Value(color::RED),
            PropertyAt::Value(20f32),
        )),
        Object::Sphere(Sphere::new(
            0.5f32,
            Point3::new(0f32, -0.5f32, -1f32),
            PropertyAt::Value(1.0),
            PropertyAt::Value(color::RED),
            PropertyAt::Value(0f32),
        )),
        Object::Sphere(Sphere::new(
            0.25f32,
            Point3::new(1.75f32, -0.25f32, -1.2),
            PropertyAt::Value(0.3),
            PropertyAt::Value(color::WHITE),
            PropertyAt::Value(40f32),
        )),
        Object::Sphere(Sphere::new(
            0.3f32,
            Point3::new(-0.75f32, -0.7f32, -0.8),
            PropertyAt::Value(0.9),
            PropertyAt::Value(color::WHITE),
            PropertyAt::Value(0f32),
        )),
        Object::Sphere(Sphere::new(
            0.05f32,
            Point3::new(0f32, -0.91f32, -0.02),
            PropertyAt::Value(0.9),
            PropertyAt::Value(color::BLACK),
            PropertyAt::Value(0f32),
        )),
        Object::Sphere(Sphere::new(
            100f32,
            Point3::new(0f32, -101f32, -1f32),
            PropertyAt::Value(0.0),
            PropertyAt::Value(color::GREEN),
            PropertyAt::Value(0f32),
        )),
    ];
    for _ in 0..300 {
        world.push(Object::Sphere(Sphere::new(
            rng.gen_range(0.1f32..0.3f32),
            Point3::new(
                rng.gen_range(-3f32..3f32),
                rng.gen_range(-1f32..1f32),
                rng.gen_range(-30f32..0f32),
            ),
            PropertyAt::Value(rng.gen_range(0f32..1f32)),
            PropertyAt::Value(Color::random()),
            PropertyAt::Value(0f32),
        )))
    }
    let bvh = BVH::build(&mut world);
    let world = WorldList::from_vec(world);

    loop {
        match rx.recv() {
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
                                pix_color += ray_color(&r, &world, max_bounces, &mut rng, &bvh);
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
