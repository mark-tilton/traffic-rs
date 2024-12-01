use std::collections::HashSet;

use bevy::{math::Vec3, prelude::Resource};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Node {
    pub position: Vec3,
}

#[derive(Resource, Serialize, Deserialize)]
pub struct NodeGraph {
    pub nodes: Vec<Node>,
    pub edges: HashSet<(usize, usize)>,
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
        NodeGraph { nodes, edges }
    }

    pub fn is_edge_in_path(source_node: usize, dest_node: usize, path: &Vec<usize>) -> bool {
        let Some(source_index) = path.iter().position(|x| x == &source_node) else {
            return false;
        };

        let Some(dest_index) = path.iter().position(|x| x == &dest_node) else {
            return false;
        };

        return dest_index == source_index + 1;
    }
}
