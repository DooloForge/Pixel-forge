use super::*;

#[derive (Copy, PartialEq, Default)]
#[turbo::serialize]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    
    pub fn add(&self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
    
    pub fn sub(&self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
    
    pub fn mul(&self, scalar: f32) -> Vec2 {
        Vec2::new(self.x * scalar, self.y * scalar)
    }
    
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    
    pub fn distance_to(&self, other: &Vec2) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
    
    pub fn normalize(&self) -> Vec2 {
        let len = self.length();
        if len > 0.0 {
            self.mul(1.0 / len)
        } else {
            Vec2::zero()
        }
    }
    
    pub fn scale(&self, scalar: f32) -> Vec2 {
        self.mul(scalar)
    }
}

#[derive (Copy, PartialEq, Default)]
#[turbo::serialize]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
    pub fn zero() -> Self { Self { x: 0.0, y: 0.0, z: 0.0 } }
    pub fn add(&self, other: Vec3) -> Vec3 { Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z) }
    pub fn sub(&self, other: Vec3) -> Vec3 { Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z) }
    pub fn scale(&self, s: f32) -> Vec3 { Vec3::new(self.x * s, self.y * s, self.z * s) }
}

pub fn project_topdown(p: Vec3) -> Vec2 { Vec2::new(p.x, p.y) }
pub fn project_dive(p: Vec3) -> Vec2 { Vec2::new(p.x, p.z) }
