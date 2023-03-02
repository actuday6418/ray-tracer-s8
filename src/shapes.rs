use crate::color::Color;
use crate::ray::Ray;
use glam::f32::Vec3A;
use roots::Roots;
use std::cmp::Ordering;

const T_MIN: f32 = 0.001;
const T_MAX: f32 = 1000.0;

pub struct IntersectionTable {
    pub point: Vec3A,
    pub normal: Vec3A,
    pub albedo: Color,
    pub roughness: f32,
}

trait Intersectable {
    fn get_roots(&self, ray: &Ray) -> Roots<f32>;
    fn normal_at(&self, point: Vec3A) -> Vec3A;
    fn albedo_at(&self, point: Vec3A) -> Color;
    fn roughness_at(&self, point: Vec3A) -> f32;

    fn get_intersection_point(&self, ray: &Ray) -> Option<Vec3A> {
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

pub trait IntersectableContainer {
    fn intersect(&self, ray: &Ray) -> Option<IntersectionTable>;
}

impl<T> IntersectableContainer for T
where
    T: Intersectable,
{
    fn intersect(&self, ray: &Ray) -> Option<IntersectionTable> {
        let p = self.get_intersection_point(ray);
        p.map(|p| IntersectionTable {
            point: p,
            normal: self.normal_at(p),
            albedo: self.albedo_at(p),
            roughness: self.roughness_at(p),
        })
    }
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
pub struct Sphere {
    pub radius: f32,
    pub center: Vec3A,
    pub f_albedo_at: fn(Vec3A) -> Color,
    pub f_roughness_at: fn(Vec3A) -> f32,
}

impl Intersectable for Sphere {
    fn get_roots(&self, ray: &Ray) -> Roots<f32> {
        let a = 1f32;
        let b = (2f32 * ray.direction).dot(ray.origin - self.center);
        let c = (ray.origin - self.center).length().powi(2) - self.radius.powi(2);
        roots::find_roots_quadratic(a, b, c)
    }

    fn normal_at(&self, point: Vec3A) -> Vec3A {
        (point - self.center).normalize_or_zero()
    }

    fn roughness_at(&self, point: Vec3A) -> f32 {
        let f = self.f_roughness_at;
        f(point)
    }
    fn albedo_at(&self, point: Vec3A) -> Color {
        let f = self.f_albedo_at;
        f(point)
    }
}
