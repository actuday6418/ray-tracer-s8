use crate::ray::Ray;
use crate::vector3::Vec3;
use roots::Roots;
use std::cmp::Ordering;

const T_MIN: f32 = 0.05;
const T_MAX: f32 = 1000.0;

trait Intersectable {
    fn intersects(&self, ray: &Ray) -> Roots<f32>;
    fn normal_at(&self, point: Vec3) -> Vec3;
}

pub trait Seeable {
    fn seen(&self, ray: &Ray) -> Option<(Vec3, Vec3)>;
}

impl<T> Seeable for Vec<T>
where
    T: Seeable + std::fmt::Debug,
{
    fn seen(&self, ray: &Ray) -> Option<(Vec3, Vec3)> {
        let intersects: Vec<_> = self.iter().map(|e| e.seen(ray)).collect();
        if intersects.iter().all(|e| e.is_none()) {
            None
        } else {
            intersects.iter().filter_map(|e| *e).min_by(|a, b| {
                (a.0 - ray.origin)
                    .length()
                    .partial_cmp(&(b.0 - ray.origin).length())
                    .unwrap_or(Ordering::Less)
            })
        }
    }
}

impl<T> Seeable for T
where
    T: Intersectable,
{
    fn seen(&self, ray: &Ray) -> Option<(Vec3, Vec3)> {
        match self.intersects(ray) {
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
        .map(|x| {
            let point = ray.origin + x * ray.direction;
            (point, self.normal_at(point))
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Sphere {
    pub radius: f32,
    pub center: Vec3,
}

impl Intersectable for Sphere {
    fn intersects(&self, ray: &Ray) -> Roots<f32> {
        let a = 1f32;
        let b = (2f32 * ray.direction) % (ray.origin - self.center);
        let c = (ray.origin - self.center).length().powi(2) - self.radius.powi(2);
        roots::find_roots_quadratic(a, b, c)
    }

    fn normal_at(&self, point: Vec3) -> Vec3 {
        (point - self.center).unit_vector()
    }
}
