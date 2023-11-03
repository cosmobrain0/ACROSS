use crate::vec2d;
use crate::Vector;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineCircleCollision {
    None,
    One(Vector),
    Two(Vector, Vector),
}
impl LineCircleCollision {
    pub fn from_vec(v: Vec<Vector>) -> Option<LineCircleCollision> {
        match v.len() {
            0 => Some(LineCircleCollision::None),
            1 => Some(LineCircleCollision::One(v[0])),
            2 => Some(LineCircleCollision::Two(v[0], v[1])),
            _ => None,
        }
    }
}

pub fn point_circle_collision(centre: Vector, radius: f32, p: Vector) -> Option<Vector> {
    if (p - centre).sqr_length() <= radius * radius {
        Some(p)
    } else {
        None
    }
}

pub fn line_circle_collision(
    centre: Vector,
    radius: f32,
    a: Vector,
    b: Vector,
) -> LineCircleCollision {
    let m = (b.y - a.y) / (b.x - a.x);
    if m.is_finite() {
        let c = b.y - m * b.x;
        // (x - a)^2 + (mx + c - b)^2 = r^2
        // x^2 - 2ax + a^2 + (mx)^2 + 2m(c-b)x + (c-b)^2 = r^2
        // (1+m^2)x^2 + 2(m(c-b) - a)x + (a^2 + (c-b)^2) = r^2
        let x_coords = quadratic(
            1.0 + m * m,
            2.0 * (m * (c - centre.y) - centre.x),
            (centre.x * centre.x + (c - centre.y) * (c - centre.y)),
        );
        LineCircleCollision::from_vec(
            x_coords
                .into_iter()
                .filter(|x| *x >= a.x.min(b.x) && *x <= a.x.max(b.x))
                .map(|x| vec2d![x, m * x + c])
                .collect(),
        )
        .unwrap()
    } else {
        let x = a.x;
        let dy = {
            let dy_squared = radius * radius - (x - centre.x) * (x - centre.x);
            if dy_squared >= 0.0 {
                Some(dy_squared.sqrt())
            } else {
                None
            }
        };
        LineCircleCollision::from_vec(
            dy.map(|dy| vec![vec2d![x, centre.y + dy], vec2d![x, centre.y - dy]])
                .unwrap_or_else(|| vec![]),
        )
        .unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum QuadraticSolution {
    NoRoots,
    RepeatedRoot(f32),
    Roots(f32, f32),
}
impl IntoIterator for QuadraticSolution {
    type Item = f32;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        use QuadraticSolution::*;
        match self {
            NoRoots => vec![],
            RepeatedRoot(x) => vec![x],
            Roots(a, b) => vec![a, b],
        }
        .into_iter()
    }
}

fn quadratic(a: f32, b: f32, c: f32) -> QuadraticSolution {
    let discriminant = b * b - 4.0 * a * c;
    if discriminant > 0.0 {
        let root1 = (-b + discriminant.sqrt()) / (2.0 * a);
        let root2 = (-b - discriminant.sqrt()) / (2.0 * a);
        QuadraticSolution::Roots(root1, root2)
    } else if discriminant == 0.0 {
        QuadraticSolution::RepeatedRoot(-b / (2.0 * a))
    } else {
        QuadraticSolution::NoRoots
    }
}
