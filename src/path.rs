use std::{cell::RefCell, rc::Rc};

use ggez::{graphics::Color, Context};

use crate::{
    renderer::{draw_circle, draw_line},
    vector::Vector,
};

#[derive(Debug)]
pub struct RouteCreationError {
    invalid_connections: Vec<usize>,
    point_count: usize,
}

#[derive(Debug)]
pub enum WebCreationError {
    InvalidRoute,
    InvalidConnections {
        invalid_connections: Vec<(usize, usize)>,
        point_count: usize,
    },
}

pub struct Route {
    points: Vec<Vector>,
}

impl Route {
    fn new(points: &Vec<Rc<RefCell<Point>>>) -> Result<Self, RouteCreationError> {
        let mut invalid_connections = vec![];
        for (i, point) in points.iter().enumerate().skip(1) {
            if !point.borrow().is_neighbour(&points[i - 1]) {
                invalid_connections.push(i - 1);
            }
        }
        if invalid_connections.len() > 0 {
            Err(RouteCreationError {
                invalid_connections,
                point_count: points.len(),
            })
        } else {
            Ok(Self {
                points: points.iter().map(|x| *x.borrow().position()).collect(),
            })
        }
    }
}

pub struct Web {
    points: Vec<Rc<RefCell<Point>>>,
    route: Route,
}
impl Web {
    pub fn new(
        positions: Vec<Vector>,
        connections: Vec<(usize, usize)>,
        route_indexes: Vec<usize>,
    ) -> Result<Self, WebCreationError> {
        if route_indexes.iter().any(|&x| x >= positions.len())
            || connections
                .iter()
                .any(|&(a, b)| a >= positions.len() || b >= positions.len())
        {
            Err(WebCreationError::InvalidRoute)
        } else {
            let points: Vec<Rc<RefCell<Point>>> = positions
                .iter()
                .map(|&x| Rc::new(RefCell::new(Point::new(x))))
                .collect();
            for &(a, b) in connections.iter() {
                // TODO: add an `add_connection` method which just adds one connection?
                points[a]
                    .borrow_mut()
                    .add_connections(&vec![Rc::clone(&points[b])])
            }
            let route: Vec<_> = route_indexes
                .iter()
                .map(|&x| points[x].borrow().position)
                .collect();
            Ok(Self {
                points,
                route: Route { points: route },
            })
        }
    }

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
        self.route
            .points
            .iter()
            .skip(1)
            .enumerate()
            .for_each(|(i, &x)| draw_line(ctx, self.route.points[i].clone(), x, 3.5, Color::WHITE));
        self.points
            .iter()
            .for_each(|x| draw_circle(ctx, x.borrow().position, 20.0, Color::WHITE));
    }
}

#[derive(Debug, Clone)]
struct Point {
    position: Vector,
    connections: Vec<Rc<RefCell<Point>>>,
}
impl Point {
    pub fn new(position: Vector) -> Self {
        Self {
            position,
            connections: vec![],
        }
    }
    pub fn position(&self) -> &Vector {
        &self.position
    }

    pub fn add_connections(&mut self, connections: &Vec<Rc<RefCell<Point>>>) {
        self.connections.reserve(connections.len());
        for p in connections.iter() {
            if self.connections.iter().all(|x| !Rc::ptr_eq(x, p)) {
                self.connections.push(p.clone());
            }
        }
    }

    pub fn is_neighbour(&self, point: &Rc<RefCell<Point>>) -> bool {
        self.connections.iter().any(|x| Rc::ptr_eq(x, point))
    }

    /// TODO: optimise (maybe)
    pub fn are_all_neighbours(&self, points: &Vec<Rc<RefCell<Point>>>) -> bool {
        points.iter().all(|point| self.is_neighbour(point))
    }
}
