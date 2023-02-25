use std::{default, ops};

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl default::Default for Vec3 {
    fn default() -> Self {
        Self {
            x: 0f32,
            y: 0f32,
            z: 0f32,
        }
    }
}

impl Vec3 {
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> f32 {
        self.x.powi(2) + self.y.powi(2) + self.z.powi(2)
    }

    pub fn inverse(&self) -> Self {
        -1f32 * *self
    }

    pub fn unit_vector(&self) -> Self {
        *self / self.length()
    }

    pub fn from_slice(slice: [f32; 3]) -> Self {
        Self {
            x: slice[0],
            y: slice[1],
            z: slice[2],
        }
    }
}

/// implements CROSS PRODUCT
impl ops::Mul<Vec3> for Vec3 {
    type Output = Self;
    //A × B = (bz – cy)i – (az – cx)j + (ay – bx)k = (bz – cy)i + (cx – az)j + (ay – bx)k
    fn mul(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }
}

impl ops::Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::Div<f32> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl ops::Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}

/// implements DOT PRODUCT
impl ops::Rem for Vec3 {
    type Output = f32;
    fn rem(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl ops::Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl ops::Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

#[test]
fn overloaded_operators() {
    assert_eq!(
        Vec3 {
            x: 1f32,
            y: 1f32,
            z: 1f32
        } + Vec3 {
            x: 3f32,
            y: 4f32,
            z: 5f32
        },
        Vec3 {
            x: 4f32,
            y: 5f32,
            z: 6f32
        }
    );
    assert_eq!(
        Vec3 {
            x: 10f32,
            y: 10f32,
            z: 10f32
        } - Vec3 {
            x: 3f32,
            y: 4f32,
            z: 5f32
        },
        Vec3 {
            x: 7f32,
            y: 6f32,
            z: 5f32
        }
    );
}
