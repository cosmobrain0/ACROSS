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

#[derive(Debug, Clone)]
pub struct Route {
    points: Vec<Vector>,
    length: f32,
}

impl Route {
    fn new(points: &Vec<Rc<RefCell<Point>>>) -> Result<Self, RouteCreationError> {
        let mut invalid_connections = vec![];
        for (i, point) in points.iter().enumerate().skip(1) {
            // this was checking if the next point was a neighbour of the previous point, which led to runtime errors
            if !points[i - 1].borrow().is_neighbour(point) {
                invalid_connections.push(i - 1);
            }
        }
        if invalid_connections.len() > 0 || points.len() < 2 {
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

    pub fn length(&self) -> f32 {
        self.length
    }

    pub fn get_position(&self, progress: f32) -> Option<Vector> {
        if progress < 0.0 || progress > 1.0 {
            None
        } else {
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
                    return Some(a + (b - a) * (progress - progress_made) / dist);
                }
                progress_made += dist;
            }
            Some(self.points[self.points.len() - 1])
        }
    }
}

#[derive(Debug)]
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
                points[a]
                    .borrow_mut()
                    .add_connections(&vec![Rc::clone(&points[b])])
            }
            let route: Vec<_> = route_indexes.iter().map(|&x| points[x].clone()).collect();
            let route = Route::new(&route);
            dbg!(&points); // correctly connected
            dbg!(&route);
            match route {
                Ok(x) => Ok(Self { points, route: x }),
                Err(_) => Err(WebCreationError::InvalidRoute),
            }
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

    pub fn route<'a>(&'a self) -> &'a Route {
        &self.route
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

    pub fn add_connection(&mut self, connection: &Rc<RefCell<Point>>) {
        if self.connections.iter().all(|x| !Rc::ptr_eq(x, connection)) {
            self.connections.push(connection.clone());
        }
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
        self.connections
            .iter()
            .any(|x| std::ptr::eq(x.as_ref().as_ptr(), point.as_ref().as_ptr()))
    }

    pub fn are_all_neighbours(&self, points: &Vec<Rc<RefCell<Point>>>) -> bool {
        points.iter().all(|point| self.is_neighbour(point))
    }
}
