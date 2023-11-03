use crate::tower::shortest_angle_distance;
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
impl IntoIterator for LineCircleCollision {
    type Item = Vector;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            LineCircleCollision::None => vec![],
            LineCircleCollision::One(x) => vec![x],
            LineCircleCollision::Two(a, b) => vec![a, b],
        }
        .into_iter()
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
            centre.x * centre.x + (c - centre.y) * (c - centre.y),
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

/// This probably breaks horribly if a line has a length of 0
pub fn line_line_collision(a1: Vector, b1: Vector, a2: Vector, b2: Vector) -> Option<Vector> {
    // finding the vector equation of the lines
    // so I don't have to deal with vertical lines having undefined gradients
    let (point1, direction1) = (a1, b1 - a1);
    let (point2, direction2) = (a2, b2 - a2);
    let (param1, param2) = simultaneous_equations(
        direction1.x,
        -direction2.x,
        point1.x - point2.x,
        direction1.y,
        -direction2.y,
        point1.y - point2.y,
    );
    let collision = {
        let p1 = point1 + direction1 * param1;
        let p2 = point2 + direction2 * param2;
        (p1 + p2) / 2.0
    };
    if param1 >= 0.0
        && param1 <= (a1 - b1).length() / direction1.length()
        && param2 >= 0.0
        && param2 <= (a2 - b2).length() / direction2.length()
    {
        Some(collision)
    } else {
        None
    }
}

/// Solves
/// ax + by = c
/// dx + ey = f
/// simultaneously. Probably panics if you give it something unsolveable?
/// TODO: figure out when this panics/returns NaN and fix that
fn simultaneous_equations(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> (f32, f32) {
    let y = (c / a - f / d) / (b / a - e / d);
    let x = (c - b * y) / a;
    (x, y)
}

pub fn point_sector_collision(
    centre: Vector,
    radius: f32,
    direction: f32,
    fov: f32,
    point: Vector,
) -> Option<Vector> {
    point_circle_collision(centre, radius, point)
        .map(|p| {
            if shortest_angle_distance((p - centre).angle(), direction).abs() <= fov / 2.0 {
                Some(p)
            } else {
                None
            }
        })
        .flatten()
}

pub fn sector_line_collision(
    centre: Vector,
    radius: f32,
    direction: f32,
    fov: f32,
    a: Vector,
    b: Vector,
) -> Vec<Vector> {
    let arc = line_circle_collision(centre, radius, a, b)
        .into_iter()
        .filter(|p| shortest_angle_distance((*p - centre).angle(), direction).abs() <= fov / 2.0);
    let line1 = line_line_collision(
        centre,
        centre + Vector::from_polar(direction - fov / 2.0, radius),
        a,
        b,
    )
    .map(|x| vec![x])
    .unwrap_or_else(|| vec![]);
    let line2 = line_line_collision(
        centre,
        centre + Vector::from_polar(direction + fov / 2.0, radius),
        a,
        b,
    )
    .map(|x| vec![x])
    .unwrap_or_else(|| vec![]);
    arc.chain(line1).chain(line2).collect()
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
