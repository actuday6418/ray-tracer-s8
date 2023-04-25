use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use crossbeam_channel::unbounded;
use crossbeam_channel::{Receiver, Sender};
use futures::future::join_all;
use log::info;
use pollster::block_on;
use ray_tracer_interface::{
    color::Color,
    shapes::{mesh::Triangle, Object},
    Point3, RenderInfo, RenderMeta,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

enum MessageToWorker {
    NewJob(Job),
    Get(String),
}

struct Job {
    id: Uuid,
    obj: String,
}

enum MessageToServer {
    Output(String),
}

struct AppState {
    rx: Receiver<MessageToServer>,
    tx: Sender<MessageToWorker>,
    flag: Arc<RwLock<bool>>,
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
                    (material.shininess as f32) / 1000f32,
                    Color::from_slice(material.diffuse),
                    0f32,
                )));
            }
        }
    } else {
        panic!("Failed to load obj or mtl file")
    }
    world
}

async fn worker(
    rx: Receiver<MessageToWorker>,
    tx: Sender<MessageToServer>,
    flag: Arc<RwLock<bool>>,
) {
    let mut results: HashMap<String, String> = HashMap::new();
    loop {
        info!("worker booted");
        if let Ok(message) = rx.recv() {
            match message {
                MessageToWorker::NewJob(Job { id, obj: _ }) => {
                    {
                        let mut f = flag.write().unwrap();
                        *f = true;
                    }
                    let client = reqwest::Client::new();
                    let r = RenderInfo {
                        world: build_world(),
                        render_meta: RenderMeta::from_image_divisions(1920, 1080, 2),
                        division_no: 0,
                    };
                    let slaves = vec![
                        client
                            .post("http://127.0.0.1:8081/")
                            .header("Content-Type", "application/json")
                            .body(json!(r).to_string())
                            .send(),
                        client
                            .post("http://127.0.0.1:8081/")
                            .header("Content-Type", "application/json")
                            .body(
                                json!(RenderInfo {
                                    division_no: 1,
                                    ..r
                                })
                                .to_string(),
                            )
                            .send(),
                    ];
                    let slaves: Vec<_> = join_all(slaves)
                        .await
                        .into_iter()
                        .map(|x| x.unwrap().text())
                        .collect();
                    let img: String = join_all(slaves)
                        .await
                        .into_iter()
                        .map(|x| x.unwrap())
                        .fold(String::new(), |a, b| a + &b);
                    results.insert(id.to_string(), img);
                    {
                        let mut f = flag.write().unwrap();
                        *f = false;
                    }
                }
                MessageToWorker::Get(id) => {
                    let img = results.remove(&id).unwrap_or(String::new());
                    tx.send(MessageToServer::Output(img)).unwrap();
                }
            }
        }
    }
}

#[post("/")]
async fn index(req: web::Json<String>, state: web::Data<AppState>) -> impl Responder {
    info!("Got request");
    let req = req.into_inner();
    let id = Uuid::new_v4();
    {
        let f = state.flag.read().unwrap();
        if *f == true {
            return String::from("We're busy");
        }
    }

    state
        .tx
        .send(MessageToWorker::NewJob(Job { id, obj: req }))
        .unwrap();
    id.to_string()
}

#[get("/poll/{id}")]
async fn poll(id: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    {
        let f = state.flag.read().unwrap();
        if *f == true {
            return String::from("We're busy");
        }
    }
    state
        .tx
        .send(MessageToWorker::Get(id.into_inner()))
        .unwrap();
    if let Ok(message) = state.rx.recv() {
        match message {
            MessageToServer::Output(img) => json!(img).to_string(),
        }
    } else {
        String::new()
    }
}

#[actix_web::main]
async fn main() {
    pretty_env_logger::init();
    let (txw, rxw) = unbounded::<MessageToWorker>();
    let (txs, rxs) = unbounded::<MessageToServer>();
    let flag = Arc::new(RwLock::new(false));
    let flag2 = flag.clone();
    info!("Spawning worker!");
    std::thread::spawn(|| block_on(worker(rxw, txs, flag2)));
    let state = web::Data::new(AppState {
        rx: rxs,
        tx: txw,
        flag,
    });

    HttpServer::new(move || {
        App::new()
            .service(index)
            .service(poll)
            .app_data(state.clone())
    })
    .bind(("127.0.0.1", 8080))
    .unwrap()
    .run()
    .await
    .unwrap();
}
