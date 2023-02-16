use std::ops;

#[derive(PartialEq, Debug)]
struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

///TODO: implement methods
// impl Vec3 {
//     pub fn length() -> f32 {}
//     pub fn length_squared() -> f32 {}
//     pub fn inverse() -> Self {}
//     pub fn unit_vector() -> Self {}
// }

///TODO: represents cross product 
// impl ops::Mul for Vec3 {}

///TODO: represents dot product
// impl ops::Rem for Vec3 {}

impl ops::Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {x: self.x + other.x, y: self.y + other.y, z: self.z + other.z}
    }
}

impl ops::Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {x: self.x - other.x, y: self.y - other.y, z: self.z - other.z}
    }
}

#[test]
fn overloaded_operators() {
    assert_eq!(Vec3 {x: 1f32, y: 1f32, z: 1f32} + Vec3 {x: 3f32, y: 4f32, z: 5f32}, Vec3 { x: 4f32, y: 5f32, z: 6f32});
    assert_eq!(Vec3 {x: 10f32, y: 10f32, z: 10f32} - Vec3 {x: 3f32, y: 4f32, z: 5f32}, Vec3 { x: 7f32, y: 6f32, z: 5f32});
}
