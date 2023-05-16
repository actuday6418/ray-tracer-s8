pub mod camera;
use uuid::Uuid;
pub mod color;
pub mod shapes;
pub use bvh::Point3;
use serde::{Deserialize, Serialize};
use shapes::Object;
use displaydoc::Display;

#[derive(Serialize, Deserialize, Display)]
pub struct RenderInfo {
    pub world: Vec<Object>,
    pub render_meta: RenderMeta,
    pub division_no: u32,
}

#[derive(Serialize, Deserialize)]
pub struct ImageSlice {
    pub division_no: u32,
    pub image: Vec<u8>,
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RenderMeta {
    pub height: u32,
    pub width: u32,
    pub divisions: u32,
    pub id: Uuid,
}
