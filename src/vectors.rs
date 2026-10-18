use std::ops::{Sub, Add, Mul, Div, AddAssign};

#[derive(Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn bearing(&self) -> f32 {
        self.x.atan2(self.y)
    }
    pub fn from_bearing(bearing: f32) -> Self {
        Vec2 {
            x: bearing.sin(),
            y: bearing.cos(),
        }
    }
    pub fn lengthsq(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }
    pub fn length(&self) -> f32 { 
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn interpolate(self, other: Self, t: f32) -> Self{
        self + (other - self) * t
    }
    pub fn normalize(self) -> Self {
        if self.x == 0.0 && self.y == 0.0 {
            self
        }
        else {
            self / self.length()
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Vec2 { 
            x: self.x - rhs.x, 
            y: self.y - rhs.y, 
        }
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Vec2 { 
            x: self.x + rhs.x, 
            y: self.y + rhs.y, 
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Vec2 {
            x: self.x * rhs, 
            y: self.y * rhs, 
        }
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Vec2 { 
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}