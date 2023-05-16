use std::io::BufReader;

use log::info;
use ray_tracer_interface::{
    color::Color,
    shapes::{mesh::Triangle, Object},
    Point3,
};

pub fn build_world(data: Vec<u8>, obj_size: usize) -> Vec<Object> {
    let mut world = vec![];
    let mut obj_br = BufReader::new(&data[..obj_size]);
    info!("Retrieving models and materials");
    if let Ok((models, Ok(materials))) =
        tobj::load_obj_buf(&mut obj_br, &tobj::LoadOptions::default(), |_| {
            tobj::load_mtl_buf(&mut BufReader::new(&data[obj_size..]))
        })
    {
        info!("starting world build");
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
