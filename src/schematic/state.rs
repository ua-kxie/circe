// different tools the schematic may have active

use crate::schematic::net_vertex::NetVertex;
use petgraph::graphmap::GraphMap;

#[derive(Default)]
pub struct State {
    nets: Nets,
    devices: Devices,
    comments: Comments,
    labels: Labels,
    ports: Ports,
}

struct Nets {
    graph: Box<GraphMap<NetVertex, (), petgraph::Undirected>>,
}
impl Default for Nets {
    fn default() -> Self {
        Nets {
            graph: Box::new(GraphMap::new()),
        }
    }
}

#[derive(Default)]
struct Devices;
#[derive(Default)]
struct Comments;
#[derive(Default)]
struct Labels;
#[derive(Default)]
struct Ports;
