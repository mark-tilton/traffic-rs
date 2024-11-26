use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

#[derive(Clone)]
pub struct Node {
    pub position: Vec3,
}

#[derive(Resource)]
pub struct NodeGraph {
    pub nodes: Vec<Node>,
    pub edges: HashSet<(usize, usize)>,
    // Source nodes are nodes that have no other nodes pointing to them
    pub source_nodes: HashSet<usize>,
    // Destination nodes are nodes that don't have any nodes leading from them
    pub dest_nodes: HashSet<usize>,
    // A convenient data structure for navigating forward
    // through the graph.
    pub node_map: HashMap<usize, HashSet<usize>>,
}

impl NodeGraph {
    // Creates a four way intersection with the following structure
    //          2     3
    //          |     ^
    //          V     |
    //    4<---10<----11<----6
    //          | \ / ^
    //          |  X  |
    //          V / \ |
    //    5---->8---->9----->7
    //          |     ^
    //          V     |
    //          0     1
    pub fn create() -> Self {
        // Bevy uses a right handed y-up coordinate system
        // This means that the forward vector is -z
        let node_positions = [
            // Bottom
            Vec3::new(-1., 0., 10.),
            Vec3::new(1., 0., 10.),
            // Top
            Vec3::new(-1., 0., -10.),
            Vec3::new(1., 0., -10.),
            // Left
            Vec3::new(-10., 0., -1.),
            Vec3::new(-10., 0., 1.),
            // Right
            Vec3::new(10., 0., -1.),
            Vec3::new(10., 0., 1.),
            // Intersection
            Vec3::new(-1., 0., 1.),
            Vec3::new(1., 0., 1.),
            Vec3::new(-1., 0., -1.),
            Vec3::new(1., 0., -1.),
        ];
        let nodes = node_positions.map(|position| Node { position }).to_vec();
        let edges = HashSet::from([
            // Sources to the intersection
            (1, 9),
            (2, 10),
            (6, 11),
            (5, 8),
            // Intersection out to destinations
            (9, 7),
            (11, 3),
            (10, 4),
            (8, 0),
            // Intersection to intersection
            (9, 11),
            (9, 10),
            (11, 10),
            (11, 8),
            (10, 8),
            (10, 9),
            (8, 9),
            (8, 11),
        ]);
        Self::new(nodes, edges)
    }

    pub fn new(nodes: Vec<Node>, edges: HashSet<(usize, usize)>) -> Self {
        // Automatically classify nodes as source, or destination nodes based
        // on edge directions.
        let mut source_nodes = HashSet::from_iter(0..nodes.len());
        let mut dest_nodes = HashSet::from_iter(0..nodes.len());
        let mut node_map: HashMap<usize, HashSet<usize>> = HashMap::new();
        for (source, dest) in edges.iter() {
            dest_nodes.remove(source);
            source_nodes.remove(dest);
            node_map.entry(*source).or_default().insert(*dest);
        }
        NodeGraph {
            nodes,
            edges,
            source_nodes,
            dest_nodes,
            node_map,
        }
    }
}

pub fn show_node_graph(node_graph: Res<NodeGraph>, mut gizmos: Gizmos) {
    let node_radius = 0.5;
    // Draw nodes different colors based on their types
    for (i, node) in node_graph.nodes.iter().enumerate() {
        let color = if node_graph.source_nodes.contains(&i) {
            Color::srgb(0.1, 0.9, 0.1)
        } else if node_graph.dest_nodes.contains(&i) {
            Color::srgb(0.9, 0.1, 0.1)
        } else {
            Color::srgb(0.1, 0.1, 0.9)
        };
        gizmos.sphere(node.position, Quat::IDENTITY, node_radius, color);
    }

    // Draw edges as arrows while leaving space for the node.
    for (source, dest) in node_graph.edges.iter() {
        let source_pos = node_graph.nodes[*source].position;
        let dest_pos = node_graph.nodes[*dest].position;
        let dest_to_src = dest_pos - source_pos;
        let arrow_start = source_pos + dest_to_src.normalize() * node_radius;
        let arrow_end = source_pos + dest_to_src.normalize() * (dest_to_src.length() - node_radius);
        gizmos.arrow(arrow_start, arrow_end, Color::srgb(1., 1., 1.));
    }
}
