use crate::color::Color;
use bvh::{
    aabb::{Bounded, AABB},
    bounding_hierarchy::BHShape,
    ray::Ray,
    Point3, Vector3,
};
use roots::Roots;
use serde::{Deserialize, Serialize};
use std::cmp::{max_by, min_by};

use super::Intersectable;

#[derive(Deserialize, Serialize)]
pub struct Triangle {
    a: Point3,
    b: Point3,
    c: Point3,
    node_index: usize,
    p_albedo_at: Color,
    p_roughness_at: f32,
    p_emission_at: f32,
}

impl Triangle {
    pub fn new(
        a: Point3,
        b: Point3,
        c: Point3,
        p_roughness_at: f32,
        p_albedo_at: Color,
        p_emission_at: f32,
    ) -> Self {
        Self {
            a,
            b,
            c,
            node_index: 0,
            p_albedo_at,
            p_roughness_at,
            p_emission_at,
        }
    }
}

impl Bounded for Triangle {
    fn aabb(&self) -> AABB {
        let min = Vector3::new(
            min_by(
                min_by(self.a.x, self.c.x, |a, b| {
                    a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                }),
                self.b.x,
                |a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            ),
            min_by(
                min_by(self.a.y, self.c.y, |a, b| {
                    a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                }),
                self.b.y,
                |a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            ),
            min_by(
                min_by(self.a.z, self.c.z, |a, b| {
                    a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                }),
                self.b.z,
                |a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            ),
        );
        let max = Vector3::new(
            max_by(
                max_by(self.a.x, self.c.x, |a, b| {
                    a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                }),
                self.b.x,
                |a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            ),
            max_by(
                max_by(self.a.y, self.c.y, |a, b| {
                    a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                }),
                self.b.y,
                |a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            ),
            max_by(
                max_by(self.a.z, self.c.z, |a, b| {
                    a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                }),
                self.b.z,
                |a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            ),
        );
        AABB::with_bounds(min, max)
    }
}

impl BHShape for Triangle {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}

impl Intersectable for Triangle {
    fn get_roots(&self, ray: &Ray) -> Roots<f32> {
        const EPSILON: f32 = 0.00001;
        let a_to_b = self.b - self.a;
        let a_to_c = self.c - self.a;

        // Begin calculating determinant - also used to calculate u parameter
        // u_vec lies in view plane
        // length of a_to_c in view_plane = |u_vec| = |a_to_c|*sin(a_to_c, dir)
        let u_vec = ray.direction.cross(a_to_c);

        // If determinant is near zero, ray lies in plane of triangle
        // The determinant corresponds to the parallelepiped volume:
        // det = 0 => [dir, a_to_b, a_to_c] not linearly independant
        let det = a_to_b.dot(u_vec);

        // Only testing positive bound, thus enabling backface culling
        // If backface culling is not desired write:
        // det < EPSILON && det > -EPSILON
        if det < EPSILON && det > -EPSILON {
            return Roots::No([]);
        }

        let inv_det = 1.0 / det;

        // Vector from point a to ray origin
        let a_to_origin = ray.origin - self.a;

        // Calculate u parameter
        let u = a_to_origin.dot(u_vec) * inv_det;

        // Test bounds: u < 0 || u > 1 => outside of triangle
        if !(0.0..=1.0).contains(&u) {
            return Roots::No([]);
        }

        // Prepare to test v parameter
        let v_vec = a_to_origin.cross(a_to_b);

        // Calculate v parameter and test bound
        let v = ray.direction.dot(v_vec) * inv_det;
        // The intersection lies outside of the triangle
        if v < 0.0 || u + v > 1.0 {
            return Roots::No([]);
        }

        let dist = a_to_c.dot(v_vec) * inv_det;

        if dist > EPSILON {
            Roots::One([dist])
        } else {
            Roots::No([])
        }
    }

    fn normal_at(&self, _: Point3) -> Vector3 {
        (self.a - self.b).cross(self.a - self.c).normalize_or_zero()
    }

    fn roughness_at(&self, point: Point3) -> f32 {
        self.p_roughness_at
    }

    fn albedo_at(&self, point: Point3) -> Color {
        self.p_albedo_at
    }
    fn emission_at(&self, point: Point3) -> f32 {
        self.p_emission_at
    }
}
