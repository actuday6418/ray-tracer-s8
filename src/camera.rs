use crate::ray::Ray;
use glam::f32::Vec3A;
use rand::{rngs::ThreadRng, Rng};
use rand_distr::{Distribution, UnitDisc};

pub struct Camera {
    origin: Vec3A,
    lower_left_corner: Vec3A,
    horizontal: Vec3A,
    aspect_ratio: f32,
    image_height: f32,
    vertical: Vec3A,
    aperture: f32,
    focal_length: f32,
    field_of_view: f32,
    focus_distance: f32,
}

impl Camera {
    pub fn new(
        origin: Vec3A,
        aspect_ratio: f32,
        aperture: f32,
        focus_distance: f32,
        field_of_view: f32,
        focal_length: f32,
        image_height: f32,
    ) -> Self {
        let vh = 2.0 * (field_of_view / 2f32).tan();
        let vw = aspect_ratio * vh;
        let horizontal = Vec3A::new(vw, 0f32, 0f32);
        let vertical = Vec3A::new(0f32, vh, 0f32);
        Self {
            origin,
            horizontal,
            focus_distance,
            image_height,
            field_of_view,
            focal_length,
            aspect_ratio,
            vertical,
            aperture,
            lower_left_corner: origin
                - horizontal / 2f32
                - vertical / 2f32
                - Vec3A::new(0f32, 0f32, focal_length),
        }
    }

    pub fn set_field_of_view(&mut self, field_of_view: f32) {
        *self = Self::new(
            self.origin,
            self.aspect_ratio,
            self.aperture,
            self.focus_distance,
            field_of_view,
            self.focal_length,
            self.image_height,
        )
    }

    pub fn set_focal_length(&mut self, focal_length: f32) {
        *self = Self::new(
            self.origin,
            self.aspect_ratio,
            self.aperture,
            self.focus_distance,
            self.field_of_view,
            focal_length,
            self.image_height,
        )
    }

    pub fn set_origin(&mut self, origin: Vec3A) {
        *self = Self::new(
            origin,
            self.aspect_ratio,
            self.aperture,
            self.focus_distance,
            self.field_of_view,
            self.focal_length,
            self.image_height,
        )
    }

    pub fn set_focus_distance(&mut self, focus_distance: f32) {
        *self = Self::new(
            self.origin,
            self.aspect_ratio,
            self.aperture,
            focus_distance,
            self.field_of_view,
            self.focal_length,
            self.image_height,
        )
    }

    pub fn set_aperture(&mut self, aperture: f32) {
        *self = Self::new(
            self.origin,
            self.aspect_ratio,
            aperture,
            self.focus_distance,
            self.field_of_view,
            self.focal_length,
            self.image_height,
        )
    }

    pub fn get_ray(&self, x: u32, y: u32, rng: &mut ThreadRng) -> Ray {
        let lens_radius = self.aperture / 2f32;
        // sample a point from the disk with radius "lens_radius"
        let offset = Vec3A::from_slice({
            let [a, b]: [f32; 2] = UnitDisc.sample(rng);
            &[a * lens_radius, b * lens_radius, 0f32]
        });
        // get viewport coordinates for pixel (randomised subpixel coordinates)
        let u =
            (x as f32 + rng.gen_range(0f32..1f32)) / (self.aspect_ratio * self.image_height - 1f32);
        let v = (y as f32 + rng.gen_range(0f32..1f32)) / (self.image_height - 1f32);
        // get focal point (point at length "focus_distance" on the vector from origin to viewport
        // coordinate)
        let focal_point = Ray {
            origin: self.origin,
            direction: (self.lower_left_corner + u * self.horizontal + v * self.vertical
                - self.origin)
                .normalize_or_zero(),
        }
        .at(self.focus_distance);
        // return ray, from random point in the disk, to the focal point
        Ray {
            origin: self.origin,
            direction: (focal_point - self.origin - offset).normalize_or_zero(),
        }
    }
}
