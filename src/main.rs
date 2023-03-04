use image::RgbImage;
use rand::rngs::ThreadRng;
use rand::Rng;
use rand_distr::{Distribution, UnitSphere};
mod camera;
mod color;
mod ray;
mod shapes;
pub mod vector3;
use color::Color;
use glam::f32::Vec3A;
use ray::Ray;
use std::thread::spawn;
use std::{f32::consts::PI, sync::mpsc};
mod gui;
use log::{info, warn};
use shapes::{IntersectableContainer, PropertyAt};

fn ray_color<T: IntersectableContainer>(
    ray: &Ray,
    world: &T,
    depth: u32,
    rng: &mut ThreadRng,
) -> Color {
    if depth == 0 {
        return color::BLACK;
    }
    match world.intersect(&ray) {
        Some(table) => {
            let diffuse_dir = Vec3A::from_slice(&UnitSphere.sample(rng)) + table.normal;
            let glossy_dir = ray.direction - 2f32 * ray.direction.dot(table.normal) * table.normal;
            let scatter_direction = diffuse_dir + table.roughness * (glossy_dir - diffuse_dir);
            table.albedo.blend(&ray_color(
                &Ray {
                    origin: table.point,
                    direction: scatter_direction.try_normalize().unwrap_or(table.normal),
                },
                world,
                depth - 1,
                rng,
            ))
        }
        None => {
            let t = ray.direction.normalize_or_zero().y;
            (1f32 - t) * color::WHITE
                + t * Color {
                    r: 0.5,
                    g: 0.7,
                    b: 1.0,
                }
        }
    }
}

pub enum MessageToGUI {
    Rendered(eframe::egui::ColorImage, u128),
}

fn main() {
    pretty_env_logger::init();
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
        Vec3A::ZERO,
        aspect_ratio,
        0.1f32,
        1f32,
        PI / 2f32,
        1f32,
        image_height,
    );
    let mut sample_count: u32 = 5;

    let mut img = RgbImage::new(image_width as u32, image_height as u32);
    let mut rng = rand::thread_rng();
    let mut world = vec![
        shapes::Sphere {
            radius: 0.5f32,
            center: Vec3A::new(0f32, -0.5f32, -1f32),
            p_roughness_at: PropertyAt::Value(1.0),
            p_albedo_at: PropertyAt::Value(color::RED),
        },
        shapes::Sphere {
            radius: 0.25f32,
            center: Vec3A::new(0.75f32, -0.25f32, -1.2),
            p_roughness_at: PropertyAt::Value(0.3),
            p_albedo_at: PropertyAt::Value(color::WHITE),
        },
        shapes::Sphere {
            radius: 0.3f32,
            center: Vec3A::new(-0.75f32, -0.7f32, -0.8),
            p_roughness_at: PropertyAt::Value(0.9),
            p_albedo_at: PropertyAt::Value(color::WHITE),
        },
        shapes::Sphere {
            radius: 0.05f32,
            center: Vec3A::new(0f32, -0.91f32, -0.02),
            p_roughness_at: PropertyAt::Value(0.9),
            p_albedo_at: PropertyAt::Value(color::BLACK),
        },
        shapes::Sphere {
            radius: 100f32,
            center: Vec3A::new(0f32, -101f32, -1f32),
            p_roughness_at: PropertyAt::Value(0.0),
            p_albedo_at: PropertyAt::Value(color::GREEN),
        },
    ];
    for _ in 0..10 {
        world.push(shapes::Sphere {
            radius: rng.gen_range(0.1f32..0.3f32),
            center: Vec3A::new(
                rng.gen_range(-3f32..3f32),
                rng.gen_range(-1f32..1f32),
                rng.gen_range(-8f32..0f32),
            ),
            p_roughness_at: PropertyAt::Value(rng.gen_range(0f32..1f32)),
            p_albedo_at: PropertyAt::Value(Color::random()),
        })
    }

    loop {
        match rx.recv() {
            Ok(gui::MessageToRender::Render) => {
                let now = std::time::Instant::now();
                for (x, y, p) in img.enumerate_pixels_mut() {
                    let y = image_height as u32 - y - 1;
                    let mut pix_color = color::BLACK;
                    for _ in 0..sample_count {
                        let r = camera.get_ray(x, y, &mut rng);
                        pix_color += ray_color(&r, &world, max_bounces, &mut rng);
                    }
                    pix_color.r = (pix_color.r / sample_count as f32).sqrt();
                    pix_color.g = (pix_color.g / sample_count as f32).sqrt();
                    pix_color.b = (pix_color.b / sample_count as f32).sqrt();
                    *p = pix_color.to_image_rgb();
                }
                let size = [img.width() as usize, img.height() as usize];
                let imgbuff = img.clone().into_raw();
                let egui_img = eframe::egui::ColorImage::from_rgb(size, &imgbuff);
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
            Ok(gui::MessageToRender::SaveImage) => {
                let hd = home::home_dir().map(|mut d| {
                    d.push("render.png");
                    d.into_os_string().to_str().unwrap().to_owned()
                });
                if hd.is_some() && img.save(hd.as_ref().unwrap()).is_err() {
                    warn!(
                        "Couldn't save render. Is the path {} accessible to the program?",
                        hd.unwrap()
                    )
                } else {
                    info!("Render saved")
                }
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
