use actix_web::web::Bytes;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use crossbeam_channel::unbounded;
use crossbeam_channel::{Receiver, Sender};
use futures::future::join_all;
use image::{ImageBuffer, Rgb};
use log::info;
mod obj;
use pollster::block_on;
use ray_tracer_interface::{ImageSlice, RenderInfo, RenderMeta};
use reqwest::blocking::Client;
use serde_json::json;
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex, RwLock};
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
    let world = obj::build_world(body, obj_size);
    for division_no in 0..divisions {
        info!(
            "Response from slave: {}",
            client
                .post("http://0.0.0.0:8081")
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
                .unwrap()
                .text()
                .unwrap_or_default()
        );
    }
    let mut state = state.write().unwrap();
    state.jobs.push(Job {
        result: Vec::new(),
        render_meta,
    });
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
                println!("{}", job.render_meta.id);
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
                    Bytes::from_static(b"Job not finished yet")
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
    })
    .bind(("0.0.0.0", 8080))
    .unwrap()
    .run()
    .await
    .unwrap();
}
