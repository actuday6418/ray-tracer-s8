use image::RgbImage;
mod camera;
mod color;
mod ray;
mod shapes;
pub mod vector3;
use color::Color;
use ray::Ray;
use shapes::Seeable;
use std::sync::mpsc;
use std::thread::spawn;
use vector3::Vec3;
mod gui;

fn ray_color(ray: &Ray) -> Color {
    let t = ray.direction.unit_vector().y;
    (1f32 - t) * color::WHITE
        + t * Color {
            r: 0.5,
            g: 0.7,
            b: 1.0,
        }
}

pub enum MessageToGUI {
    ImageRendered(eframe::egui::ColorImage),
}

fn main() {
    let (txa, rxa) = mpsc::channel::<gui::MessageToRender>();
    let (txb, rxb) = mpsc::channel::<MessageToGUI>();
    spawn(move || render(rxa, txb));
    gui::launch(txa, rxb);
}

fn render(rx: mpsc::Receiver<gui::MessageToRender>, tx: mpsc::Sender<MessageToGUI>) {
    let aspect_ratio = 16f32 / 9f32;
    let image_height = 1080f32;
    let image_width = aspect_ratio * image_height;

    let mut camera = camera::Camera::new(aspect_ratio);

    let mut img = RgbImage::new(image_width as u32, image_height as u32);
    let w = vec![
        shapes::Sphere {
            radius: 0.5f32,
            center: Vec3 {
                x: 0f32,
                y: 0f32,
                z: -1f32,
            },
        },
        shapes::Sphere {
            radius: 100f32,
            center: Vec3 {
                x: 0f32,
                y: -101f32,
                z: -1f32,
            },
        },
    ];
    loop {
        match rx.recv() {
            Ok(gui::MessageToRender::Render) => {
                for (x, y, p) in img.enumerate_pixels_mut() {
                    let y = image_height as u32 - y - 1;
                    let u = (x as f32) / (image_width - 1f32);
                    let v = (y as f32) / (image_height - 1f32);
                    let r = camera.get_ray(u, v);
                    *p = match w.seen(&r) {
                        Some((_, normal)) => Color {
                            r: normal.x,
                            g: normal.y,
                            b: normal.z,
                        }
                        .to_image_rgb(),
                        None => ray_color(&r).to_image_rgb(),
                    };
                }
                let size = [img.width() as usize, img.height() as usize];
                let imgbuff = img.clone().into_raw();
                let egui_img = eframe::egui::ColorImage::from_rgb(size, &imgbuff);
                tx.send(MessageToGUI::ImageRendered(egui_img)).unwrap();
            }
            Ok(gui::MessageToRender::UpdateCameraOrigin(origin)) => camera.set_origin(origin),
            Ok(gui::MessageToRender::UpdateCameraFocalLength(focal_length)) => {
                camera.set_focal_length(focal_length)
            }
            Err(_) => return,
        }
    }
}
