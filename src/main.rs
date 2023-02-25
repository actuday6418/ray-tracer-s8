use image::RgbImage;
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
    let height = 1080f32;
    let width = aspect_ratio * height;

    let vh = 2f32;
    let vw = aspect_ratio * vh;
    let mut focal_length = 1f32;

    let mut origin = Vec3::default();
    let horizontal = Vec3 {
        x: vw,
        y: 0f32,
        z: 0f32,
    };
    let vertical = Vec3 {
        x: 0f32,
        y: vh,
        z: 0f32,
    };
    let mut lower_left_corner = origin
        - horizontal / 2f32
        - vertical / 2f32
        - Vec3 {
            x: 0f32,
            y: 0f32,
            z: focal_length,
        };

    let mut img = RgbImage::new(width as u32, height as u32);
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
                    let y = height as u32 - y - 1;
                    let u = (x as f32) / (width - 1f32);
                    let v = (y as f32) / (height - 1f32);
                    let r = Ray {
                        origin,
                        direction: (lower_left_corner + u * horizontal + v * vertical - origin)
                            .unit_vector(),
                    };
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
            Ok(gui::MessageToRender::UpdateCameraOrigin(origin_new)) => {
                origin = origin_new;
                lower_left_corner = origin
                    - horizontal / 2f32
                    - vertical / 2f32
                    - Vec3 {
                        x: 0f32,
                        y: 0f32,
                        z: focal_length,
                    };
            }
            Ok(gui::MessageToRender::UpdateCameraFocalLength(focal_length_new)) => {
                focal_length = focal_length_new;
                lower_left_corner = origin
                    - horizontal / 2f32
                    - vertical / 2f32
                    - Vec3 {
                        x: 0f32,
                        y: 0f32,
                        z: focal_length,
                    };
            }
            Err(_) => return,
        }
    }
}
