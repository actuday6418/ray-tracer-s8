use image::Rgb;
use std::ops;

#[derive(Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub fn to_image_rgb(&self) -> Rgb<u8> {
        Rgb::from([
            (self.r * 255.999) as u8,
            (self.g * 255.999) as u8,
            (self.b * 255.999) as u8,
        ])
    }
}

impl ops::Add for Color {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }
}

impl ops::Sub for Color {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            r: self.r - other.r,
            g: self.g - other.g,
            b: self.b - other.b,
        }
    }
}

impl ops::Mul<f32> for Color {
    type Output = Self;
    fn mul(self, other: f32) -> Self {
        Self {
            r: self.r * other,
            g: self.g * other,
            b: self.b * other,
        }
    }
}

impl ops::Mul<Color> for f32 {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        rhs * self
    }
}

pub const WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
};

pub const RED: Color = Color {
    r: 1.0,
    b: 0.2,
    g: 0.1,
};

pub const GREEN: Color = Color {
    r: 0.2,
    b: 0.2,
    g: 1.0,
};
