use rand::{self, seq::IteratorRandom};
use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

#[derive(Component)]
pub struct Vehicle {
    edge_id: isize,
    edge_position: f32,
    node_path: Vec<isize>,
}

#[derive(Clone)]
pub struct Node {
    position: Vec3,
}

#[derive(Resource)]
pub struct Simulation {
    vehicles: Vec<Vehicle>,
    nodes: Vec<Node>,
    edges: HashSet<(usize, usize)>,
    source_nodes: HashSet<usize>,
    dest_nodes: HashSet<usize>,
    dest_map: HashMap<usize, HashSet<usize>>,
}

impl Simulation {
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
        // Source nodes are nodes that have no other nodes pointing to them
        let mut source_nodes = HashSet::from_iter(0..nodes.len());
        // Destination nodes are nodes that don't have any nodes leading from them
        let mut dest_nodes = HashSet::from_iter(0..nodes.len());
        let mut dest_map: HashMap<usize, HashSet<usize>> = HashMap::new();
        for (source, dest) in edges.iter() {
            dest_nodes.remove(source);
            source_nodes.remove(dest);
            dest_map.entry(*source).or_default().insert(*dest);
        }
        Simulation {
            vehicles: Vec::new(),
            nodes,
            edges,
            source_nodes,
            dest_nodes,
            dest_map,
        }
    }
}

pub fn spawn_vehicle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut simulation: ResMut<Simulation>,
) {
    let mut rng = rand::thread_rng();
    let start_node = simulation.source_nodes.iter().choose(&mut rng);

    // let entity = commands.spawn((PbrBundle {
    //     mesh: meshes.add(Cuboid::new(5., 2., 3.).mesh()),
    //     material: materials.add(Color::srgb(0.3, 0.3, 0.5)),
    //     ..default()
    // },));
}

pub fn set_vehicle_position(
    mut vehicles: Query<(&mut Transform, &Vehicle)>,
    simulation: Res<Simulation>,
) {
}

pub fn show_node_graph(simulation: Res<Simulation>, mut gizmos: Gizmos) {
    for (i, node) in simulation.nodes.iter().enumerate() {
        let color = if simulation.source_nodes.contains(&i) {
            Color::srgb(0.1, 0.9, 0.1)
        } else if simulation.dest_nodes.contains(&i) {
            Color::srgb(0.9, 0.1, 0.1)
        } else {
            Color::srgb(0.1, 0.1, 0.9)
        };
        gizmos.sphere(node.position, Quat::IDENTITY, 0.5, color);
    }

    for (source, dest) in simulation.edges.iter() {
        let source_pos = simulation.nodes[*source].position;
        let dest_pos = simulation.nodes[*dest].position;
        let dest_to_src = dest_pos - source_pos;
        let arrow_start = source_pos + dest_to_src.normalize() * 0.5;
        let arrow_end = source_pos + dest_to_src.normalize() * (dest_to_src.length() - 0.5);
        gizmos.arrow(arrow_start, arrow_end, Color::srgb(1., 1., 1.));
    }
}
