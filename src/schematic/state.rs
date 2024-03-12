// different tools the schematic may have active

use crate::types::SSPoint;
use petgraph::graphmap::GraphMap;

pub struct State {
    nets: Nets,
    devices: Devices,
    comments: Comments,
    labels: Labels,
    ports: Ports,
}

struct Nets {
    graph: Box<GraphMap<SSPoint, (), petgraph::Undirected>>,
}

struct Devices;
struct Comments;
struct Labels;
struct Ports;
