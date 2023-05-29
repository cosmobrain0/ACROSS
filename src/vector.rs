use std::{f32::consts::PI, ops::*};

use ggez::mint::Point2;

#[macro_export]
macro_rules! vec2d {
    ($x:expr, $y:expr) => {
        Vector::new($x, $y)
    };
}

/// Represents a point in 2D space, using 2 f32's for the x and y
#[derive(Debug, Clone, Default, Copy)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

#[allow(dead_code)]
impl Vector {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn sqr_length(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }
    pub fn angle(&self) -> f32 {
        f32::atan2(self.y, self.x)
    }
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    pub const fn right() -> Self {
        Self { x: 1.0, y: 0.0 }
    }
    pub const fn up() -> Self {
        Self { x: 0.0, y: 1.0 }
    }
    pub fn normalised(&self) -> Self {
        Self::new(self.x, self.y) / self.length()
    }
    pub fn rotate(&self, delta: f32) -> Self {
        // matrix is
        // cos -sin
        // sin cos
        let cos = delta.cos();
        let sin = delta.sin();
        Self {
            x: cos * self.x - sin * self.y,
            y: sin * self.x + cos * self.y,
        }
    }
    pub fn dot(&self, other: Vector) -> f32 {
        self.x * other.x + self.y * other.y
    }
    pub fn project(&self, project_to: Vector) -> Vector {
        project_to * (self.dot(project_to) / project_to.dot(project_to))
    }
    pub fn clockwise_90deg(&self) -> Vector {
        Vector {
            x: self.y,
            y: -self.x,
        }
    }
    pub fn anticlockwise_90deg(&self) -> Vector {
        Vector {
            x: -self.y,
            y: self.x,
        }
    }
    pub fn from_polar(angle: f32, radius: f32) -> Self {
        Self {
            x: radius * angle.cos(),
            y: radius * angle.sin(),
        }
    }

    /// returns the signed angle distance between two angles
    pub fn angle_distnace(angle1: f32, angle2: f32) -> f32 {
        let diff = (angle2 - angle1 + PI) % (2.0 * PI) - PI;
        if diff < -PI {
            diff + 2.0 * PI
        } else {
            diff
        }
    }
}

impl Add for Vector {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Vector {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vector {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for Vector {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<f32> for Vector {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl MulAssign<f32> for Vector {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div<f32> for Vector {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl DivAssign<f32> for Vector {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Neg for Vector {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Into<[f32; 2]> for Vector {
    fn into(self) -> [f32; 2] {
        [self.x, self.y]
    }
}

impl From<[f32; 2]> for Vector {
    fn from(value: [f32; 2]) -> Self {
        Self {
            x: value[0],
            y: value[1],
        }
    }
}

impl From<Point2<f32>> for Vector {
    fn from(value: Point2<f32>) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}
