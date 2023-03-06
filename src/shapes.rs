use crate::color::Color;
use auto_impl::auto_impl;
use bvh::{
    aabb::{Bounded, AABB},
    bounding_hierarchy::BHShape,
    ray::Ray,
    Point3, Vector3,
};
use roots::Roots;
use std::cmp::Ordering;

const T_MIN: f32 = 0.001;
const T_MAX: f32 = 1000.0;

pub struct IntersectionTable {
    pub point: Point3,
    pub normal: Vector3,
    pub albedo: Color,
    pub roughness: f32,
}

#[auto_impl(&)]
pub trait Intersectable: Bounded {
    fn get_roots(&self, ray: &Ray) -> Roots<f32>;
    fn normal_at(&self, point: Point3) -> Vector3;
    fn albedo_at(&self, point: Point3) -> Color;
    fn roughness_at(&self, point: Point3) -> f32;

    fn get_intersection_point(&self, ray: &Ray) -> Option<Point3> {
        match self.get_roots(ray) {
            Roots::No(_) => None,
            Roots::One([x]) => {
                if (T_MIN..T_MAX).contains(&x) {
                    Some(x)
                } else {
                    None
                }
            }
            Roots::Two([x, y]) => {
                let x_in_range = (T_MIN..T_MAX).contains(&x);
                let y_in_range = (T_MIN..T_MAX).contains(&y);
                match (x_in_range, y_in_range) {
                    (true, true) => Some(if x < y { x } else { y }),
                    (true, false) => Some(x),
                    (false, true) => Some(y),
                    (false, false) => None,
                }
            }
            _ => unreachable!(),
        }
        .map(|x| ray.at(x))
    }
}

// pub trait IntersectableBHShape: Intersectable + BHShape {}

pub trait IntersectableContainer {
    fn intersect(&self, ray: &Ray) -> Option<IntersectionTable>;
}

impl<T> IntersectableContainer for Vec<T>
where
    T: Intersectable,
{
    fn intersect(&self, ray: &Ray) -> Option<IntersectionTable> {
        let intersects: Vec<_> = self
            .iter()
            .map(|e| (e, e.get_intersection_point(ray)))
            .map(|e| {
                if let Some(point) = e.1 {
                    Some((e.0, point))
                } else {
                    None
                }
            })
            .collect();
        if intersects.iter().all(|e| e.is_none()) {
            None
        } else {
            intersects
                .iter()
                .filter_map(|e| *e)
                .min_by(|a, b| {
                    (a.1 - ray.origin)
                        .length()
                        .partial_cmp(&(b.1 - ray.origin).length())
                        .unwrap_or(Ordering::Less)
                })
                .map(|e| IntersectionTable {
                    point: e.1,
                    normal: e.0.normal_at(e.1),
                    albedo: e.0.albedo_at(e.1),
                    roughness: e.0.roughness_at(e.1),
                })
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PropertyAt<T>
where
    T: Clone,
{
    Value(T),
    FromFunction(fn(Point3) -> T),
}

#[derive(Debug, PartialEq)]
pub struct Sphere {
    radius: f32,
    center: Point3,
    node_index: usize,
    p_albedo_at: PropertyAt<Color>,
    p_roughness_at: PropertyAt<f32>,
}

impl Sphere {
    pub fn new(
        radius: f32,
        center: Point3,
        p_roughness_at: PropertyAt<f32>,
        p_albedo_at: PropertyAt<Color>,
    ) -> Self {
        Self {
            radius,
            center,
            node_index: 0,
            p_albedo_at,
            p_roughness_at,
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

// impl IntersectableBHShape for Sphere {}
