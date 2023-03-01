use crate::ray::Ray;
use glam::f32::Vec3A;

pub struct Camera {
    origin: Vec3A,
    lower_left_corner: Vec3A,
    horizontal: Vec3A,
    vertical: Vec3A,
    focal_length: f32,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Self {
        let vh = 2.0;
        let focal_length = 1.0;
        let vw = aspect_ratio * vh;
        let origin = Vec3A::ZERO;
        let horizontal = Vec3A::new(vw, 0f32, 0f32);
        let vertical = Vec3A::new(0f32, vh, 0f32);
        Self {
            origin,
            horizontal,
            vertical,
            focal_length,
            lower_left_corner: origin
                - horizontal / 2f32
                - vertical / 2f32
                - Vec3A::new(0f32, 0f32, focal_length),
        }
    }

    pub fn set_focal_length(&mut self, focal_length: f32) {
        self.lower_left_corner = self.lower_left_corner + Vec3A::new(0f32, 0f32, self.focal_length);
        self.focal_length = focal_length;
        self.lower_left_corner = self.lower_left_corner - Vec3A::new(0f32, 0f32, self.focal_length);
    }

    pub fn set_origin(&mut self, origin: Vec3A) {
        self.lower_left_corner = self.lower_left_corner - self.origin;
        self.origin = origin;
        self.lower_left_corner = self.lower_left_corner + origin;
    }

    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        Ray {
            origin: self.origin,
            direction: (self.lower_left_corner + u * self.horizontal + v * self.vertical
                - self.origin)
                .normalize_or_zero(),
        }
    }
}
