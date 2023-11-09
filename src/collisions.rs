use crate::tower::shortest_angle_distance;
use crate::vec2d;
use crate::Vector;

/// a line fully enclosed by a circle is represented with `LineCircleCollision::None`
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
    let mut collisions = Vec::with_capacity(2);
    for endpoint in [a, b] {
        if point_circle_collision(centre, radius, endpoint).is_some() {
            collisions.push(endpoint);
        }
    }

    let m = (b.y - a.y) / (b.x - a.x);
    if m.is_finite() {
        let c = b.y - m * b.x;
        let x_coords = quadratic(
            1.0 + m * m,
            2.0 * (m * (c - centre.y) - centre.x),
            centre.x * centre.x + (c - centre.y) * (c - centre.y) - radius * radius,
        );
        collisions.extend(
            x_coords
                .into_iter()
                .filter(|x| *x >= a.x.min(b.x) && *x <= a.x.max(b.x))
                .map(|x| vec2d![x, m * x + c]),
        );
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
        collisions.extend(
            dy.map(|dy| vec![vec2d![x, centre.y + dy], vec2d![x, centre.y - dy]])
                .unwrap_or_else(|| vec![])
                .into_iter(),
        );
    }

    assert!(collisions.len() <= 2, "Oh no! {:#?}", collisions);
    LineCircleCollision::from_vec(collisions).unwrap()
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
        point2.x - point1.x,
        direction1.y,
        -direction2.y,
        point2.y - point1.y,
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
/// simultaneously.
/// # Panics
/// Panics if lines are parallel
fn simultaneous_equations(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> (f32, f32) {
    let x = (b * f - e * c) / (b * d - e * a);
    let y = (c - a * x) / b;
    return (x, y);
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

/// This function never returns more than two collisions
pub fn line_sector_collision(
    centre: Vector,
    radius: f32,
    direction: f32,
    fov: f32,
    a: Vector,
    b: Vector,
) -> Vec<Vector> {
    // this handles endpoints inside the arc
    let arc = line_circle_collision(centre, radius, a, b)
        .into_iter()
        .filter(|p| shortest_angle_distance((*p - centre).angle(), direction).abs() <= fov / 2.0);

    let line1 = line_line_collision(
        centre,
        centre + Vector::from_polar(direction - fov / 2.0, radius),
        a,
        b,
    );

    let line2 = line_line_collision(
        centre,
        centre + Vector::from_polar(direction + fov / 2.0, radius),
        a,
        b,
    );

    if line1.is_some() && line2.is_some() {
        vec![line1.unwrap(), line2.unwrap()]
    } else {
        arc.chain(line1).chain(line2).collect()
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
