use crate::vector::Vector;

/// Represents an index for a node
/// Prevents unsigned integers which were
/// intended for other purposes to be used
/// as node indexes by accident
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeIndex(pub usize);
impl From<usize> for NodeIndex {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

/// Represents a connection between two nodes
/// with a calculated weight
#[derive(Debug, Clone, PartialEq)]
pub struct Connection {
    start: NodeIndex,
    end: NodeIndex, // I don't like this
    weight: f32, // is this just due to towers or does it consider distances? is it [0-1] or something else?
}
impl From<(usize, usize, f32)> for Connection {
    fn from(value: (usize, usize, f32)) -> Self {
        Self {
            start: NodeIndex(value.0),
            end: NodeIndex(value.1),
            weight: value.2,
        }
    }
}
/// Represents a node for the A* algorithm
/// with a position, g_cost, h_cost, f_cost and parent
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    position: Vector,
    g_cost: Option<f32>,
    h_cost: Option<f32>, // this should only be evaluated once
    parent: Option<NodeIndex>,
}
impl Node {
    /// Gets the h_cost, without reevaluating it
    /// if it's already been evaluated once.
    fn h_cost_calculate(&mut self, target: &Node) -> f32 {
        if let Some(h_cost) = self.h_cost {
            h_cost
        } else {
            let cost = (target.position - self.position).length();
            self.h_cost = Some(cost);
            cost
        }
    }

    /// Gets the h_cost of this node,
    /// assuming that it has already been calculated
    /// # Panics
    /// Panics if the h_cost has not been calculated
    fn h_cost(&self) -> f32 {
        if let Some(h_cost) = self.h_cost {
            h_cost
        } else {
            panic!("The h_cost of this node hasn't been calculated!");
        }
    }

    /// Sets the g_cost of this node, based on
    /// the fact that the shortest route to it
    /// is from a given parent node.
    /// # Panics
    /// Panics if the parent node does not have a g_cost
    fn set_g_cost(
        &mut self,
        parent: &Node,
        parent_index: NodeIndex,
        this_index: NodeIndex,
        connections: &Vec<Connection>,
    ) {
        self.parent = Some(parent_index);
        let weight = connections
            .iter()
            .find(|x| x.start == parent_index && x.end == this_index)
            .unwrap()
            .weight;
        self.g_cost = Some(parent.g_cost.unwrap() + weight);
    }

    /// Gets the position of this node
    pub fn position(&self) -> Vector {
        self.position
    }
}
impl Node {
    /// Constructs a node
    pub fn new(position: Vector) -> Self {
        Self {
            position,
            g_cost: 0.0.into(),
            h_cost: None,
            parent: None,
        }
    }

    /// Calculates the h_cost and f_cost, if the g_cost has been calculated
    #[inline(always)]
    pub fn f_cost_calculate(&mut self, target: &Node) -> Option<f32> {
        self.g_cost
            .map(|g_cost| g_cost + self.h_cost_calculate(target))
    }

    /// Gets the f_cost, if the g_cost has been calculated
    /// # Panics
    /// Panics if the h_cost has not been calculated
    #[inline(always)]
    pub fn f_cost(&self) -> Option<f32> {
        self.g_cost.map(|g_cost| g_cost + self.h_cost())
    }
}

/// Represents all of the data required for the A* algorithm
#[derive(Debug, Clone)]
pub struct Pathfinder {
    nodes: Vec<Node>,
    connections: Vec<Connection>,
    best_route: Option<Vec<Node>>,
}
impl Pathfinder {
    /// Constructs a new pathfinder,
    /// if the given inputs are valid
    pub fn new(
        nodes: &Vec<Vector>,
        connections: &Vec<(usize, usize)>,
        weight_calculation: impl Fn(Vector, Vector) -> f32,
    ) -> Option<Self> {
        let mut final_connections = Vec::with_capacity(connections.len());
        if nodes.len() == 0
            || connections
                .iter()
                .any(|(start, end)| *start >= nodes.len() || *end >= nodes.len())
        {
            None
        } else {
            for potential_connection in connections.into_iter() {
                if !final_connections
                    .iter()
                    .any(|connection: &(usize, usize, f32)| {
                        (connection.0, connection.1) == *potential_connection
                    })
                {
                    let (a, b) = *potential_connection;
                    final_connections.push((a, b, weight_calculation(nodes[a], nodes[b])));
                }
            }
            Some(Self {
                nodes: nodes
                    .into_iter()
                    .map(|position| Node::new(*position))
                    .collect(),
                connections: final_connections.into_iter().map(Into::into).collect(),
                best_route: None,
            })
        }
    }

    /// Recalculates the weights of connections between nodes
    /// given a function to calculate those weights
    pub fn recalculate_weights(&mut self, weight_calculation: impl Fn(Vector, Vector) -> f32) {
        for connection in &mut self.connections {
            connection.weight = weight_calculation(
                self.nodes[connection.start.0].position,
                self.nodes[connection.end.0].position,
            );
        }
        self.best_route = None;
    }

    /// Calculates the best route between given `start` and `end` nodes
    pub fn pathfind(&mut self, start: NodeIndex, end: NodeIndex) -> &Vec<Node> {
        // everything added to `open` must have a calculated g_cost
        let mut open = vec![];
        let mut closed = vec![];
        self.nodes[start.0].g_cost = Some(0.0);
        let target = self.nodes[end.0].clone();
        self.nodes[start.0].h_cost_calculate(&target);
        open.push(start);

        loop {
            let current_index = open
                .iter()
                .enumerate()
                .fold(None, |acc, (i, el)| {
                    if acc.is_some_and(|(_, acc): (usize, NodeIndex)| {
                        self.nodes[acc.0].f_cost() < self.nodes[el.0].f_cost()
                    }) {
                        acc
                    } else {
                        Some((i, *el))
                    }
                })
                .unwrap();
            let current = open.remove(current_index.0);
            closed.push(current);

            if current == end {
                let mut path = vec![self.nodes[current.0].clone()];
                let mut current = current;
                while let Some(parent) = &self.nodes[current.0].parent {
                    path.push(self.nodes[parent.0].clone());
                    current = *parent;
                }
                assert_eq!(current, start);
                path = path.into_iter().rev().collect();
                self.best_route = Some(path);
                return self.best_route.as_ref().unwrap();
            }

            for neighbour in self.neighbours(current_index.1) {
                if !closed.contains(&neighbour) {
                    let current_node = &self.nodes[current.0];
                    let neighbour_node = &self.nodes[neighbour.0];
                    let new_path_g_cost = current_node.g_cost.unwrap()
                        + (current_node.position - neighbour_node.position).length();
                    let current_g_cost = neighbour_node.g_cost;

                    let neighbour_in_open = open.contains(&neighbour);
                    if current_g_cost.is_none()
                        || current_g_cost.unwrap() > new_path_g_cost
                        || !neighbour_in_open
                    {
                        let parent = self.nodes[current.0].clone();
                        self.nodes[neighbour.0].set_g_cost(
                            &parent,
                            current_index.1,
                            neighbour,
                            &self.connections,
                        );
                        if !neighbour_in_open {
                            self.nodes[neighbour.0].h_cost_calculate(&target);
                            open.push(neighbour);
                        }
                    }
                }
            }
        }
    }

    /// Gets the neighbours of a node specified by a `NodeIndex`
    fn neighbours<'a>(&'a mut self, node_index: NodeIndex) -> Vec<NodeIndex> {
        self.connections
            .iter()
            .filter(|x| x.start.0 == node_index.0)
            .map(|x| x.end)
            .collect()
    }

    /// Gets the best route, if it has been calculated
    pub fn best_route(&self) -> Option<Vec<Vector>> {
        self.best_route
            .as_ref()
            .map(|nodes| nodes.iter().map(|node| node.position).collect())
    }
}
