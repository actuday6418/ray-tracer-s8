use crate::MessageToGUI;
use bvh::Point3;
use eframe::egui;
use egui_extras::image::RetainedImage;
use std::{f32::consts::PI, sync::mpsc};

pub enum MessageToRender {
    Render,
    UpdateCameraOrigin(Point3),
    UpdateCameraFieldOfView(f32),
    UpdateCameraAperture(f32),
    UpdateCameraFocalLength(f32),
    UpdateCameraFocusDistance(f32),
    UpdateSampleCount(u32),
    SaveImage,
}

pub fn launch(tx: mpsc::Sender<MessageToRender>, rx: mpsc::Receiver<MessageToGUI>) {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(640.0, 475.0)),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(MyApp::new(tx, rx))),
    )
    .unwrap();
}

struct MyApp {
    tx: mpsc::Sender<MessageToRender>,
    rx: mpsc::Receiver<MessageToGUI>,
    image: Option<RetainedImage>,
    image_time: Option<u128>,
    origin_x: f32,
    origin_y: f32,
    origin_z: f32,
    focal_length: f32,
    field_of_view: f32,
    focus_distance: f32,
    aperture: f32,
    image_scale: f32,
    sample_count: u32,
}

impl MyApp {
    fn new(tx: mpsc::Sender<MessageToRender>, rx: mpsc::Receiver<MessageToGUI>) -> Self {
        tx.send(MessageToRender::Render).unwrap();
        Self {
            tx,
            rx,
            image: None,
            image_time: None,
            origin_x: 0f32,
            origin_y: 0f32,
            origin_z: 0f32,
            focal_length: 1f32,
            field_of_view: PI / 2f32,
            focus_distance: 1f32,
            aperture: 0.1f32,
            image_scale: 0.97,
            sample_count: 5,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if let Ok(MessageToGUI::Rendered(image, time)) = self.rx.try_recv() {
            self.image = Some(RetainedImage::from_color_image("rendered image", image));
            self.image_time = Some(time);
        }
        egui::TopBottomPanel::top("Controls")
            .frame(egui::Frame::default().outer_margin(5f32))
            .show(ctx, |ui| {
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
                            .send(MessageToRender::UpdateCameraOrigin(Point3::new(
                                self.origin_x,
                                self.origin_y,
                                self.origin_z,
                            )))
                            .unwrap();
                        self.tx.send(MessageToRender::Render).unwrap();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Camera field of view");
                    let f = ui.add(egui::DragValue::new(&mut self.field_of_view).speed(0.02));
                    if f.drag_released() || f.changed() && !f.dragged() {
                        self.tx
                            .send(MessageToRender::UpdateCameraFieldOfView(self.field_of_view))
                            .unwrap();
                        self.tx.send(MessageToRender::Render).unwrap()
                    }
                    ui.separator();
                    ui.label("Camera aperture");
                    let f = ui.add(egui::DragValue::new(&mut self.aperture).speed(0.02));
                    if f.drag_released() || f.changed() && !f.dragged() {
                        self.tx
                            .send(MessageToRender::UpdateCameraAperture(self.aperture))
                            .unwrap();
                        self.tx.send(MessageToRender::Render).unwrap()
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Camera focal length");
                    let f = ui.add(egui::DragValue::new(&mut self.focal_length).speed(0.02));
                    if f.drag_released() || f.changed() && !f.dragged() {
                        self.tx
                            .send(MessageToRender::UpdateCameraFocalLength(self.focal_length))
                            .unwrap();
                        self.tx.send(MessageToRender::Render).unwrap()
                    }
                    ui.separator();

                    ui.label("Camera focus distance");
                    let f = ui.add(egui::DragValue::new(&mut self.focus_distance).speed(0.02));
                    if f.drag_released() || f.changed() && !f.dragged() {
                        self.tx
                            .send(MessageToRender::UpdateCameraFocusDistance(
                                self.focus_distance,
                            ))
                            .unwrap();
                        self.tx.send(MessageToRender::Render).unwrap()
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Sample count");
                    let f = ui.add(
                        egui::Slider::new(&mut self.sample_count, 1..=100).drag_value_speed(0.02),
                    );
                    if f.drag_released() || f.changed() && !f.dragged() {
                        self.tx
                            .send(MessageToRender::UpdateSampleCount(self.sample_count))
                            .unwrap();
                        self.tx.send(MessageToRender::Render).unwrap()
                    }
                    if let Some(time) = self.image_time {
                        ui.label(format!("fps: {:.3}", 1000f32 / time as f32));
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Image viewer scale");
                    ui.add(
                        egui::Slider::new(&mut self.image_scale, 0.05f32..=3f32)
                            .drag_value_speed(0.5),
                    );
                    if ui.button("save").clicked() {
                        self.tx.send(MessageToRender::SaveImage).unwrap()
                    }
                });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(image) = &self.image {
                image.show_scaled(ui, self.image_scale);
            }
        });
        ctx.request_repaint();
    }
}
