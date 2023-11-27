use std::{f32::consts::PI, ops::*};

use ggez::mint::Point2;

/// Shorthand for making a 2d vector
#[macro_export]
macro_rules! vec2d {
    ($x:expr, $y:expr) => {
        Vector::new($x, $y)
    };
}

/// Represents a point in 2D space, using 2 f32's for the x and y
#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

#[allow(dead_code)]
impl Vector {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    /// Calculates the length of this vector, by pythagoras
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    /// Calculates the length of this vector, squared, by pythagoras
    pub fn sqr_length(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }
    /// Calculates the angle this vector makes
    /// with the positive x-axis, in radians
    /// in the range (-PI, PI]
    /// a positive angle is counter-clockwise
    pub fn angle(&self) -> f32 {
        f32::atan2(self.y, self.x)
    }
    /// The zero vector
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    /// The vector which goes one unit to the right
    pub const fn right() -> Self {
        Self { x: 1.0, y: 0.0 }
    }
    /// The vector which goes one unit up
    pub const fn up() -> Self {
        Self { x: 0.0, y: 1.0 }
    }
    /// Scales the vector up or down so that its
    /// length is 1
    /// Results in `Vector { x: NaN, y: NaN }` if the vector
    /// is `Vector { x: 0, y: 0 }`
    pub fn normalised(&self) -> Self {
        Self::new(self.x, self.y) / self.length()
    }
    /// Returns the result of rotating this vector
    /// counter-clockwise, by `delta` radians
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
    /// Calculates the dot product of this vector
    /// and another vector
    pub fn dot(&self, other: Vector) -> f32 {
        self.x * other.x + self.y * other.y
    }
    /// Projects this vector onto another
    pub fn project(&self, project_to: Vector) -> Vector {
        project_to * (self.dot(project_to) / project_to.dot(project_to))
    }
    /// Rotates this vector clockwise by 90 degrees
    pub fn clockwise_90deg(&self) -> Vector {
        Vector {
            x: self.y,
            y: -self.x,
        }
    }
    /// Rotates this vector anticlockwise by 90 degrees
    pub fn anticlockwise_90deg(&self) -> Vector {
        Vector {
            x: -self.y,
            y: self.x,
        }
    }
    /// Constructs a vector from polar coordinates
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

    /// Gets the minimum of the x-coordinates
    /// and the minimum of the y-coordinates
    pub fn min(&self, other: Vector) -> Vector {
        vec2d!(self.x.min(other.x), self.y.min(other.y))
    }

    /// Gets the maximum of the x-coordinates
    /// and the maximum of the y-coordinates
    pub fn max(&self, other: Vector) -> Vector {
        vec2d!(self.x.max(other.x), self.y.max(other.y))
    }

    /// Calculates the angle between two vectors (radians)
    pub fn angle_between(&self, other: Vector) -> f32 {
        (self.dot(other) / (self.length() * other.length())).acos()
    }
}

/// `+` operator for addition
impl Add for Vector {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

/// `+=` operator for addition
impl AddAssign for Vector {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

/// `-` operator for subtraction
impl Sub for Vector {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

/// `-=` operator for subtraction
impl SubAssign for Vector {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

/// `* f32` operator for scaling (up)
impl Mul<f32> for Vector {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

// `*= f32` operator for scaling (up)
impl MulAssign<f32> for Vector {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

/// `/ f32` for scaling (down)
impl Div<f32> for Vector {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

/// `/= f32` operator for scaling (down)
impl DivAssign<f32> for Vector {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

/// unary `-` operator for multplying by -1
impl Neg for Vector {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

/// Type conversion from vector to array
impl From<Vector> for [f32; 2] {
    fn from(val: Vector) -> Self {
        [val.x, val.y]
    }
}

/// Type conversion from array to vector
impl From<[f32; 2]> for Vector {
    fn from(value: [f32; 2]) -> Self {
        Self {
            x: value[0],
            y: value[1],
        }
    }
}

/// Type conversion from Point2 (for drawing)
/// to vector
impl From<Point2<f32>> for Vector {
    fn from(value: Point2<f32>) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

/// Type conversion into Point2 (for drawing)
/// to vector
impl Into<Point2<f32>> for Vector {
    fn into(self) -> Point2<f32> {
        Point2 {
            x: self.x,
            y: self.y,
        }
    }
}
