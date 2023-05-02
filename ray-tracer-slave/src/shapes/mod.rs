use crate::color::Color;
use auto_impl::auto_impl;
use bvh::{aabb::Bounded, bounding_hierarchy::BHShape, ray::Ray, Point3, Vector3};
use roots::Roots;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
pub mod mesh;
pub mod sphere;
use mesh::Triangle;
use sphere::Sphere;

const T_MIN: f32 = 0.001;
const T_MAX: f32 = 1000.0;

pub struct IntersectionTable {
    pub point: Point3,
    pub normal: Vector3,
    pub albedo: Color,
    pub roughness: f32,
    pub emission: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Object {
    Sphere(Sphere),
    Triangle(Triangle),
}

impl Intersectable for Object {
    fn get_intersection_point(&self, ray: &Ray) -> Option<Point3> {
        match self {
            Self::Triangle(o) => o.get_intersection_point(ray),
            Self::Sphere(o) => o.get_intersection_point(ray),
        }
    }

    fn get_roots(&self, ray: &Ray) -> Roots<f32> {
        match self {
            Self::Triangle(o) => o.get_roots(ray),
            Self::Sphere(o) => o.get_roots(ray),
        }
    }

    fn normal_at(&self, point: Point3) -> Vector3 {
        match self {
            Self::Triangle(o) => o.normal_at(point),
            Self::Sphere(o) => o.normal_at(point),
        }
    }

    fn albedo_at(&self, point: Point3) -> Color {
        match self {
            Self::Triangle(o) => o.albedo_at(point),
            Self::Sphere(o) => o.albedo_at(point),
        }
    }

    fn roughness_at(&self, point: Point3) -> f32 {
        match self {
            Self::Triangle(o) => o.roughness_at(point),
            Self::Sphere(o) => o.roughness_at(point),
        }
    }

    fn emission_at(&self, point: Point3) -> f32 {
        match self {
            Self::Triangle(o) => o.emission_at(point),
            Self::Sphere(o) => o.emission_at(point),
        }
    }
}

impl Bounded for Object {
    fn aabb(&self) -> bvh::aabb::AABB {
        match self {
            Self::Sphere(o) => o.aabb(),
            Self::Triangle(o) => o.aabb(),
        }
    }
}

impl BHShape for Object {
    fn bh_node_index(&self) -> usize {
        match self {
            Self::Sphere(o) => o.bh_node_index(),
            Self::Triangle(o) => o.bh_node_index(),
        }
    }

    fn set_bh_node_index(&mut self, index: usize) {
        match self {
            Self::Sphere(o) => o.set_bh_node_index(index),
            Self::Triangle(o) => o.set_bh_node_index(index),
        }
    }
}

#[auto_impl(&)]
pub trait Intersectable: Bounded {
    fn get_roots(&self, ray: &Ray) -> Roots<f32>;
    fn normal_at(&self, point: Point3) -> Vector3;
    fn albedo_at(&self, point: Point3) -> Color;
    fn roughness_at(&self, point: Point3) -> f32;
    fn emission_at(&self, point: Point3) -> f32;

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

#[derive(Serialize, Deserialize)]
pub struct WorldList(Vec<Object>);
pub struct WorldRefList<'a>(Vec<&'a Object>);

impl WorldList {
    pub fn get<'a>(&'a self) -> &'a Vec<Object> {
        &self.0
    }

    pub fn from_vec(vec: Vec<Object>) -> Self {
        Self(vec)
    }

    pub fn to_ref<'a>(&'a self) -> WorldRefList {
        let mut w = Vec::new();
        for o in self.get() {
            w.push(o)
        }
        WorldRefList::from_vec(w)
    }
}

impl<'a> WorldRefList<'a> {
    pub fn from_vec(vec: Vec<&'a Object>) -> Self {
        Self(vec)
    }
    pub fn intersect(&self, ray: &Ray) -> Option<IntersectionTable> {
        let intersects: Vec<_> = self
            .0
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
                    emission: e.0.emission_at(e.1),
                    point: e.1,
                    normal: e.0.normal_at(e.1),
                    albedo: e.0.albedo_at(e.1),
                    roughness: e.0.roughness_at(e.1),
                })
        }
    }
}

// #[derive(Debug, PartialEq, Deserialize, Serialize)]
// pub enum PropertyAt<T>
// where
//     T: Clone,
// {
//     Value(T),
//     // FromFunction(fn(Point3) -> T),
// }
