use ggez::{graphics::Color, Context};

use crate::{
    renderer::{draw_circle, draw_line},
    vector::Vector,
};

#[derive(Debug)]
pub struct ConnectionAddError {
    point_count: usize,
    start_index: usize,
    end_index: usize,
}

pub struct Path {
    points: Vec<Vector>,
    connections: Vec<(usize, usize)>, // (a, b) means you can go from points[a] to points[b]
}

impl Path {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            connections: Vec::new(),
        }
    }
    pub fn add_points(&mut self, points: Vec<Vector>) -> &mut Self {
        self.points.reserve(points.len());
        for point in points {
            self.points.push(point);
        }
        self
    }
    pub fn add_connections(
        &mut self,
        connections: &Vec<(usize, usize)>,
    ) -> Result<&mut Self, ConnectionAddError> {
        for &connection in connections {
            match connection {
                (a, b) if a >= self.points.len() || b >= self.points.len() => {
                    return Err(ConnectionAddError {
                        point_count: self.points.len(),
                        start_index: a,
                        end_index: b,
                    });
                }
                (a, b) => self.connections.push((a, b)),
            }
        }
        Ok(self)
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        for connection in &self.connections {
            draw_line(
                ctx,
                self.points[connection.0],
                self.points[connection.1],
                3.0,
                Color::WHITE,
            );
        }
        self.points
            .iter()
            .for_each(|&point| draw_circle(ctx, point, 20.0, Color::WHITE));
    }
}
