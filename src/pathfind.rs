pub trait Node {
    fn is_neighbour(&self, other: &Box<dyn Node>) -> bool;
    fn cost_to_travel(&self, other: &Box<dyn Node>) -> f32;
}

struct NodeData {
    node: Box<dyn Node>,
    g_cost: f32,
    h_cost: f32,
}
impl NodeData {
    pub fn f_cost(&self) -> f32 {
        self.g_cost + self.h_cost
    }
}

pub struct Map<'a> {
    nodes: Vec<NodeData>,
    start: &'a NodeData,
    end: &'a NodeData,
}

impl<'a> Map<'a> {
    pub fn new(nodes: Vec<Box<dyn Node>>, start: usize, end: usize) -> Result<Self, ()> {
        let node_data: Vec<_> = nodes
            .iter()
            .map(|&x| NodeData {
                node: x,
                g_cost: 0.0,
                h_cost: 0.0,
            })
            .collect();
        if start < node_data.len() && end < node_data.len() {
            Ok(Self {
                nodes: node_data,
                start: &node_data[start],
                end: &node_data[end],
            })
        } else {
            Err(())
        }
    }
}
