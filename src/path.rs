use std::{cell::RefCell, rc::Rc};

use ggez::{graphics::Color, Context};

use crate::{
    pathfind::{Connection, NodeIndex, Pathfinder},
    renderer::{draw_circle, draw_line},
    tower::Tower,
    vector::Vector,
};

/// Represents an error when creating a `Route`
/// due to connections between points which
/// don't exist
#[derive(Debug)]
pub struct RouteCreationError {
    invalid_connections: Vec<usize>,
    point_count: usize,
}

/// Represents an error when creating a `Web`
#[derive(Debug)]
pub enum WebCreationError {
    /// Invalid connections were supplied, or there is no route from the start
    /// to the end
    InvalidRoute,
    /// The start index or the end index is invalid.
    InvalidEndpoints,
}

/// Represents a path which an `Enemy` should follow
#[derive(Debug, Clone)]
pub struct Route {
    points: Vec<Vector>,
    length: f32,
}

impl Route {
    /// Constructs a new route, or returns an error message
    /// if the point list is invalid
    fn new(points: &Vec<Rc<RefCell<Point>>>) -> Result<Self, RouteCreationError> {
        let mut invalid_connections = vec![];
        for (i, point) in points.iter().enumerate().skip(1) {
            // this was checking if the next point was a neighbour of the previous point, which led to runtime errors
            if !points[i - 1].borrow().is_neighbour(point) {
                invalid_connections.push(i - 1);
            }
        }
        if !invalid_connections.is_empty() || points.len() < 2 {
            Err(RouteCreationError {
                invalid_connections,
                point_count: points.len(),
            })
        } else {
            let positions: Vec<_> = points.iter().map(|x| *x.borrow().position()).collect();
            Ok(Self {
                length: positions
                    .iter()
                    .enumerate()
                    .skip(1)
                    .fold(0.0, |acc, (i, &x)| acc + (x - positions[i - 1]).length()),
                points: positions,
            })
        }
    }

    /// Panics if there are less than 2 points
    /// Constructs a new route, ignoring whether or not
    /// the route is actually valid
    pub fn from_positions_unchecked(positions: Vec<Vector>) -> Self {
        Self {
            length: positions
                .iter()
                .enumerate()
                .skip(1)
                .fold(0.0, |acc, (i, &x)| acc + (x - positions[i - 1]).length()),
            points: positions,
        }
    }

    /// Gets the position of an enemy on this route
    /// based on a normalised `progress` value [0-1]
    pub fn get_position(&self, progress: f32) -> Vector {
        let progress = progress.clamp(0.0, 1.0);
        let mut progress_made = 0.0;
        for (a, b, dist) in self
            .points
            .iter()
            .skip(1)
            .enumerate()
            .map(|(i, &x)| (self.points[i], x, (x - self.points[i]).length()))
            .map(|(a, b, dist)| (a, b, dist / self.length))
        {
            if progress_made + dist >= progress {
                return a + (b - a) * (progress - progress_made) / dist;
            }
            progress_made += dist;
        }
        self.points[self.points.len() - 1]
    }
}

/// Represents a web of points which
/// enemies move along, with a start point
/// for enemies to spawn at, and an end point
/// for enemies to reach
#[derive(Debug)]
pub struct Web {
    points: Vec<Rc<RefCell<Point>>>,
    pathfinder: Pathfinder,
    start: NodeIndex,
    end: NodeIndex,
}
impl Web {
    /// Validates the input (checking if it represents
    /// a real web with no invalid connections etc)
    /// and returns a Web if the input is valid
    /// or an error if the input is invalid
    pub fn new(
        positions: Vec<Vector>,
        connections: Vec<(usize, usize)>,
        start: usize,
        end: usize,
        weight_calculation: impl Fn(Vector, Vector) -> f32,
    ) -> Result<Self, WebCreationError> {
        let points: Vec<Rc<RefCell<Point>>> = positions
            .iter()
            .map(|&x| Rc::new(RefCell::new(Point::new(x))))
            .collect();
        for &(a, b) in connections.iter() {
            points[a]
                .borrow_mut()
                .add_connections(&vec![Rc::clone(&points[b])])
        }

        let pathfinder = Pathfinder::new(&positions, &connections, weight_calculation);

        if start >= positions.len() || end >= positions.len() {
            return Err(WebCreationError::InvalidEndpoints);
        }
        let start = NodeIndex(start);
        let end = NodeIndex(end);

        match pathfinder {
            Some(mut x) => {
                x.pathfind(start, end);
                Ok(Self {
                    points,
                    pathfinder: x,
                    start,
                    end,
                })
            }
            None => Err(WebCreationError::InvalidRoute),
        }
    }

    /// Uses the A* algorithm to calculate the best
    /// route from `start` to `end`
    pub fn pathfind(&mut self) {
        self.pathfinder.pathfind(self.start, self.end);
    }

    /// Draws the web to the screen, highlighting the current
    /// best route
    pub fn draw(&self, ctx: &mut Context) {
        self.points.iter().for_each(|x| {
            x.borrow().connections.iter().for_each(|y| {
                draw_line(
                    ctx,
                    x.borrow().position,
                    y.borrow().position,
                    2.0,
                    Color::new(0.5, 0.5, 0.5, 1.0),
                )
            });
        });
        let path = self.route();
        path.iter()
            .skip(1)
            .enumerate()
            .for_each(|(i, &x)| draw_line(ctx, path[i], x, 3.5, Color::WHITE));
        self.points
            .iter()
            .for_each(|x| draw_circle(ctx, x.borrow().position, 20.0, Color::WHITE));
    }

    /// Gets the best route, assuming that it has
    /// already been calculated
    /// # Panics
    /// Panics if the best route has not been calculated
    pub fn route(&self) -> Vec<Vector> {
        self.pathfinder.best_route().unwrap()
    }

    /// Recalculates the weights of connections between nodes
    /// on this web, given a function to specify how the weights
    /// are calculated
    pub fn recalculate_weights(&mut self, weight_calculation: impl Fn(Vector, Vector) -> f32) {
        self.pathfinder.recalculate_weights(weight_calculation);
    }
}

/// A node which enemies can move to/from on a web
#[derive(Debug, Clone)]
struct Point {
    /// The screen-space position of the node
    position: Vector,
    /// References to the nodes which it is connected to
    connections: Vec<Rc<RefCell<Point>>>,
}
impl Point {
    /// Constructs a new node
    pub fn new(position: Vector) -> Self {
        Self {
            position,
            connections: vec![],
        }
    }

    /// Gets the position of this node
    pub fn position(&self) -> &Vector {
        &self.position
    }

    /// Connects this node to the other given nodes
    /// This is a one-way connection
    pub fn add_connections(&mut self, connections: &Vec<Rc<RefCell<Point>>>) {
        self.connections.reserve(connections.len());
        for p in connections.iter() {
            if self.connections.iter().all(|x| !Rc::ptr_eq(x, p)) {
                self.connections.push(p.clone());
            }
        }
    }

    /// Checks if a given node is a neighbour of this node
    pub fn is_neighbour(&self, point: &Rc<RefCell<Point>>) -> bool {
        self.connections
            .iter()
            .any(|x| std::ptr::eq(x.as_ref().as_ptr(), point.as_ref().as_ptr()))
    }
}
