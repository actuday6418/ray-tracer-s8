use rand::{thread_rng, Rng};
use std::ops;

#[derive(Debug, PartialEq, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub fn as_slice(&self) -> [u8; 3] {
        [
            (self.r * 255.999) as u8,
            (self.g * 255.999) as u8,
            (self.b * 255.999) as u8,
        ]
    }

    pub fn from_slice(s: [f32; 3]) -> Self {
        Self {
            r: s[0],
            g: s[1],
            b: s[2],
        }
    }

    pub fn blend(&self, other: &Self) -> Self {
        Self {
            r: self.r * other.r,
            g: self.g * other.g,
            b: self.b * other.b,
        }
    }

    pub fn random() -> Self {
        let mut rng = thread_rng();
        Self {
            r: rng.gen_range(0f32..1f32),
            g: rng.gen_range(0f32..1f32),
            b: rng.gen_range(0f32..1f32),
        }
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

impl ops::AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
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

impl ops::Div<f32> for Color {
    type Output = Color;
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
        }
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

pub const BLACK: Color = Color {
    r: 0.0,
    b: 0.0,
    g: 0.0,
};
