use eframe::egui;
use std::sync::mpsc;

use crate::vector3::Vec3;

pub enum Message {
    Render,
    UpdateCameraOrigin(Vec3),
    UpdateCameraFocalLength(f32),
}

pub fn launch(tx: mpsc::Sender<Message>) {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(MyApp::new(tx))),
    )
    .unwrap();
}

struct MyApp {
    tx: mpsc::Sender<Message>,
    origin_x: f32,
    origin_y: f32,
    origin_z: f32,
    focal_length: f32,
}

impl MyApp {
    fn new(tx: mpsc::Sender<Message>) -> Self {
        Self {
            tx,
            origin_x: 0f32,
            origin_y: 0f32,
            origin_z: 0f32,
            focal_length: 1f32,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Render").clicked() {
                self.tx
                    .send(Message::Render)
                    .expect("Rendering worked shut down")
            }
            ui.horizontal(|ui| {
                ui.label("Camera origin");
                let x = ui.add(
                    egui::DragValue::new(&mut self.origin_x)
                        .prefix("x: ")
                        .speed(0.02),
                );
                let y = ui.add(
                    egui::DragValue::new(&mut self.origin_y)
                        .prefix("y: ")
                        .speed(0.02),
                );
                let z = ui.add(
                    egui::DragValue::new(&mut self.origin_z)
                        .prefix("z: ")
                        .speed(0.02),
                );
                if x.drag_released()
                    || y.drag_released()
                    || z.drag_released()
                    || x.changed() && !x.dragged()
                    || y.changed() && !y.dragged()
                    || z.changed() && !z.dragged()
                {
                    self.tx
                        .send(Message::UpdateCameraOrigin(Vec3 {
                            x: self.origin_x,
                            y: self.origin_y,
                            z: self.origin_z,
                        }))
                        .unwrap();
                    self.tx.send(Message::Render).unwrap()
                }
            });
            ui.horizontal(|ui| {
                ui.label("Camera focal length");
                let f = ui.add(egui::DragValue::new(&mut self.focal_length).speed(0.02));
                if f.drag_released() || f.changed() && !f.dragged() {
                    self.tx
                        .send(Message::UpdateCameraFocalLength(self.focal_length))
                        .unwrap();
                    self.tx.send(Message::Render).unwrap()
                }
            });
        });
    }
}
