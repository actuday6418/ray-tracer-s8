use crate::{ray::Ray, vector3::Vec3};

pub struct Camera {
    origin: Vec3,
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    focal_length: f32,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Self {
        let vh = 2.0;
        let focal_length = 1.0;
        let vw = aspect_ratio * vh;
        let origin = Vec3::default();
        let horizontal = Vec3 {
            x: vw,
            ..Default::default()
        };
        let vertical = Vec3 {
            y: vh,
            ..Default::default()
        };
        Self {
            origin,
            horizontal,
            vertical,
            focal_length,
            lower_left_corner: origin
                - horizontal / 2f32
                - vertical / 2f32
                - Vec3 {
                    z: focal_length,
                    ..Default::default()
                },
        }
    }

    pub fn set_focal_length(&mut self, focal_length: f32) {
        self.lower_left_corner = self.lower_left_corner
            + Vec3 {
                z: self.focal_length,
                ..Default::default()
            };
        self.focal_length = focal_length;
        self.lower_left_corner = self.lower_left_corner
            - Vec3 {
                z: self.focal_length,
                ..Default::default()
            };
    }

    pub fn set_origin(&mut self, origin: Vec3) {
        self.lower_left_corner = self.lower_left_corner - self.origin;
        self.origin = origin;
        self.lower_left_corner = self.lower_left_corner + origin;
    }

    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        Ray {
            origin: self.origin,
            direction: (self.lower_left_corner + u * self.horizontal + v * self.vertical
                - self.origin)
                .unit_vector(),
        }
    }
}
