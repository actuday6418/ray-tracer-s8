use glam::f32::Vec3A;

#[derive(Debug, PartialEq, Default)]
pub struct Ray {
    pub origin: Vec3A,
    pub direction: Vec3A,
}

impl Ray {
    pub fn at(&self, t: f32) -> Vec3A {
        self.origin + t * self.direction
    }
}
