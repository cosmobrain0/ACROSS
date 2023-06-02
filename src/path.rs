use crate::vector::Vector;

pub struct RouteCreationError<'a> {
    invalid_connections: Vec<usize>,
    points: &'a Vec<&'a Point<'a>>,
}

pub enum PathCreationError {
    NoValidRoute,
    InvalidConnections {
        invalid_connections: Vec<(usize, usize)>,
        point_count: usize,
    },
}

pub struct Route<'a> {
    web: &'a Web<'a>,
    points: Vec<&'a Vector>,
}

impl<'a> Route<'a> {
    pub fn new<'b>(web: &Web, points: &'b Vec<&'a Point>) -> Result<Self, RouteCreationError<'b>>
    where
        'a: 'b,
    {
        let mut invalid_connections = vec![];
        for (i, &point) in points.iter().enumerate().skip(1) {
            if !point.is_neighbour(points[i - 1]) {
                invalid_connections.push(i - 1);
            }
        }
        if invalid_connections.len() > 0 {
            Err(RouteCreationError {
                invalid_connections,
                points,
            })
        } else {
            Ok(Self {
                web,
                points: points.iter().map(|&x| x.position()).collect(),
            })
        }
    }
}

pub struct Web<'a> {
    points: Vec<Point<'a>>,
    route: Route<'a>,
}
impl<'a> Web<'a> {
    pub fn new(
        points: Vec<Vector>,
        connections: Vec<(usize, usize)>,
    ) -> Result<Self, PathCreationError> {
        todo!()
    }
}

pub struct Point<'a> {
    web: &'a Web<'a>,
    position: Vector,
    connections: Vec<&'a Point<'a>>,
}
impl<'a> Point<'a> {
    pub fn new(web: &Web, position: Vector) -> Self {
        Self {
            web,
            position,
            connections: vec![],
        }
    }
    pub fn position(&self) -> &Vector {
        &self.position
    }

    /// Returns the number of connections which were actually added.
    /// Connections may be ignored if they refer to points which have already been added
    /// And I don't like how this doesn't check if `connections` itself doesn't have any duplicates
    /// TODO: fix that bug
    /// And a point can't be connected to itself
    pub fn add_connections(&mut self, connections: Vec<&'a Point>) -> usize {
        let original_length = self.connections.len();
        self.connections.extend(
            connections
                .iter()
                .filter(|&&x| {
                    !connections.iter().any(|&val| std::ptr::eq(val, x))
                        && !self.connections.iter().any(|&val| std::ptr::eq(val, x))
                })
                .copied(),
        );
        self.connections.len() - original_length
    }

    pub fn is_neighbour(&self, point: &Point) -> bool {
        self.connections.iter().any(|&x| std::ptr::eq(x, point))
    }

    /// TODO: optimise (maybe)
    pub fn are_all_neighbours(&self, points: &Vec<&Point>) -> bool {
        points.iter().all(|&point| self.is_neighbour(point))
    }
}
