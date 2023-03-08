use super::{Intersectable, PropertyAt};
use crate::color::Color;
use bvh::{
    aabb::{Bounded, AABB},
    bounding_hierarchy::BHShape,
    ray::Ray,
    Point3, Vector3,
};
use roots::Roots;

#[derive(Debug, PartialEq)]
pub struct Sphere {
    radius: f32,
    center: Point3,
    node_index: usize,
    p_albedo_at: PropertyAt<Color>,
    p_roughness_at: PropertyAt<f32>,
    p_emission_at: PropertyAt<f32>,
}

impl Sphere {
    pub fn new(
        radius: f32,
        center: Point3,
        p_roughness_at: PropertyAt<f32>,
        p_albedo_at: PropertyAt<Color>,
        p_emission_at: PropertyAt<f32>,
    ) -> Self {
        Self {
            radius,
            center,
            node_index: 0,
            p_albedo_at,
            p_roughness_at,
            p_emission_at,
        }
    }
}

impl Intersectable for Sphere {
    fn get_roots(&self, ray: &Ray) -> Roots<f32> {
        let a = 1f32;
        let b = (2f32 * ray.direction).dot(ray.origin - self.center);
        let c = (ray.origin - self.center).length().powi(2) - self.radius.powi(2);
        roots::find_roots_quadratic(a, b, c)
    }

    fn normal_at(&self, point: Point3) -> Vector3 {
        (point - self.center).normalize_or_zero()
    }

    fn roughness_at(&self, point: Point3) -> f32 {
        match self.p_roughness_at {
            PropertyAt::Value(value) => value,
            PropertyAt::FromFunction(f) => f(point),
        }
    }

    fn albedo_at(&self, point: Point3) -> Color {
        match &self.p_albedo_at {
            PropertyAt::Value(value) => value.clone(),
            PropertyAt::FromFunction(f) => f(point),
        }
    }
    fn emission_at(&self, point: Point3) -> f32 {
        match &self.p_emission_at {
            PropertyAt::Value(value) => value.clone(),
            PropertyAt::FromFunction(f) => f(point),
        }
    }
}

impl Bounded for Sphere {
    fn aabb(&self) -> AABB {
        let half_size = Vector3::new(self.radius, self.radius, self.radius);
        let min = self.center - half_size;
        let max = self.center + half_size;
        AABB::with_bounds(min, max)
    }
}

impl BHShape for Sphere {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}
