use bvh::{ray::Ray, Point3, Vector3};
use rand::{rngs::SmallRng, Rng};
use rand_distr::{Distribution, UnitDisc};

pub struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vector3,
    aspect_ratio: f32,
    image_height: f32,
    vertical: Vector3,
    aperture: f32,
    focal_length: f32,
    field_of_view: f32,
    focus_distance: f32,
}

impl Camera {
    pub fn new(
        origin: Point3,
        aspect_ratio: f32,
        aperture: f32,
        focus_distance: f32,
        field_of_view: f32,
        focal_length: f32,
        image_height: f32,
    ) -> Self {
        let vh = 2.0 * (field_of_view / 2f32).tan();
        let vw = aspect_ratio * vh;
        let horizontal = Vector3::new(vw, 0f32, 0f32);
        let vertical = Vector3::new(0f32, vh, 0f32);
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
                - Vector3::new(0f32, 0f32, focal_length),
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

    pub fn set_origin(&mut self, origin: Point3) {
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

    pub fn get_ray(&self, x: u32, y: u32, rng: &mut SmallRng) -> Ray {
        let lens_radius = self.aperture / 2f32;
        let offset = Vector3::from_slice({
            let [a, b]: [f32; 2] = UnitDisc.sample(rng);
            &[a * lens_radius, b * lens_radius, 0f32]
        });
        let u =
            (x as f32 + rng.gen_range(0f32..1f32)) / (self.aspect_ratio * self.image_height - 1f32);
        let v = (y as f32 + rng.gen_range(0f32..1f32)) / (self.image_height - 1f32);
        let focal_point = Ray::new(
            self.origin,
            (self.lower_left_corner + u * self.horizontal + v * self.vertical - self.origin)
                .normalize_or_zero(),
        )
        .at(self.focus_distance);
        let final_ray_origin = self.origin + offset;
        Ray::new(
            final_ray_origin,
            (focal_point - final_ray_origin).normalize_or_zero(),
        )
    }
}
