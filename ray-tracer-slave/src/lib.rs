pub mod camera;
pub mod color;
pub mod shapes;
pub use bvh::Point3;
use serde::{Deserialize, Serialize};
use shapes::Object;

#[derive(Serialize, Deserialize)]
pub struct RenderInfo {
    pub world: Vec<Object>,
}
