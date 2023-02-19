use std::ops;

#[derive(PartialEq, Debug)]
struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

///TODO: implement methods
impl Vec3 {
pub fn length(&self)-> f32{

(self.x.pow(2)+self.y.pow(2)+self.z.pow(2)).sqrt()
}

pub fn length_squared(&self)-> f32 {
    self.x.pow(2)+self.y.pow(2)+self.z.pow(2)
}

pub fn inverse(&self) -> Self {
    Self{
        x:-self.x,
        y:-self.y,
        z:-self.z,
    }
}

pub fun unit_vector() -> Self {
    Self{
        x:  -self.x/ self.length,
        y:  -self.y/ self.length,
        z:  -self.z/ self.length,
    }
}
}




impl ops:Mul for Vec3 {
    type Output = Self;
    //A × B = (bz – cy)i – (az – cx)j + (ay – bx)k = (bz – cy)i + (cx – az)j + (ay – bx)k
    fn mul(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }  
}

imp ops:Rem for Vec3 {
 type Output=f32;
 fn rem(self,other: Self) -> f32{
    self.x*other.x + self.y*other.y +self.z*other.z
 }
}

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
