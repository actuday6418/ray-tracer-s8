use crate::ray::Ray;
use crate::vector3::Vec3;

#[derive(Debug, PartialEq)]
pub struct Sphere {
    pub radius: f32,
    pub center: Vec3,
}

impl Sphere {
    pub fn collides(&self, ray: &Ray) -> roots::Roots<f32> {
        let a = 1f32;
        let b = (2f32 * ray.direction) % (ray.origin - self.center);
        let c = (ray.origin - self.center).length().powi(2) - self.radius.powi(2);
        roots::find_roots_quadratic(a, b, c)
    }
}
