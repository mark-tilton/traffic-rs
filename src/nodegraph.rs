use rand::{self, seq::IteratorRandom};
use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

#[derive(Component)]
pub struct Vehicle {
    edge_position: f32,
    node_path: Vec<usize>,
    path_position: usize,
}

#[derive(Clone)]
pub struct Node {
    position: Vec3,
}

#[derive(Resource)]
pub struct NodeGraph {
    nodes: Vec<Node>,
    edges: HashSet<(usize, usize)>,
    source_nodes: HashSet<usize>,
    dest_nodes: HashSet<usize>,
    node_map: HashMap<usize, HashSet<usize>>,
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
        // Source nodes are nodes that have no other nodes pointing to them
        let mut source_nodes = HashSet::from_iter(0..nodes.len());
        // Destination nodes are nodes that don't have any nodes leading from them
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

pub fn spawn_vehicle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    node_graph: ResMut<NodeGraph>,
) {
    let mut rng = rand::thread_rng();
    if rand::random::<f32>() > 0.05 {
        return;
    }
    let Some(start_node) = node_graph.source_nodes.iter().choose(&mut rng) else {
        return;
    };
    let mut node_path = vec![*start_node];
    loop {
        let latest_node = node_path.last().unwrap();
        if node_graph.dest_nodes.contains(latest_node) {
            break;
        }
        let next_node = node_graph
            .node_map
            .get(latest_node)
            .expect("Invalid node path")
            .iter()
            .choose(&mut rng)
            .expect("Invalid edge map");
        node_path.push(*next_node);
    }
    let vehicle = Vehicle {
        edge_position: 0.,
        node_path,
        path_position: 0,
    };

    let start_node_position = node_graph.nodes.get(*start_node).unwrap().position;
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.3, 0.2, 0.5).mesh()),
            material: materials.add(Color::srgb(0.3, 0.3, 0.5)),
            transform: Transform::from_translation(start_node_position),
            ..default()
        },
        vehicle,
    ));
}

pub fn move_vehicles(
    mut commands: Commands,
    mut vehicle_query: Query<(Entity, &mut Transform, &mut Vehicle)>,
    node_graph: Res<NodeGraph>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut vehicle) in &mut vehicle_query {
        let speed = 3.;

        let start_node = node_graph
            .nodes
            .get(*vehicle.node_path.get(vehicle.path_position).unwrap())
            .unwrap();
        let next_node = node_graph
            .nodes
            .get(*vehicle.node_path.get(vehicle.path_position + 1).unwrap())
            .unwrap();
        let start_to_next = next_node.position - start_node.position;
        let edge_speed = speed / start_to_next.length();

        vehicle.edge_position += edge_speed * time.delta_seconds();
        if vehicle.edge_position > 1. {
            vehicle.path_position += 1;
            vehicle.edge_position = 0.;
            if vehicle.path_position >= vehicle.node_path.len() - 1 {
                commands.entity(entity).despawn();
            }
            continue;
        }

        transform.translation = start_node.position + start_to_next * vehicle.edge_position;
        transform.look_at(next_node.position, Dir3::Y);
    }
}

pub fn show_node_graph(node_graph: Res<NodeGraph>, mut gizmos: Gizmos) {
    for (i, node) in node_graph.nodes.iter().enumerate() {
        let color = if node_graph.source_nodes.contains(&i) {
            Color::srgb(0.1, 0.9, 0.1)
        } else if node_graph.dest_nodes.contains(&i) {
            Color::srgb(0.9, 0.1, 0.1)
        } else {
            Color::srgb(0.1, 0.1, 0.9)
        };
        gizmos.sphere(node.position, Quat::IDENTITY, 0.5, color);
    }

    for (source, dest) in node_graph.edges.iter() {
        let source_pos = node_graph.nodes[*source].position;
        let dest_pos = node_graph.nodes[*dest].position;
        let dest_to_src = dest_pos - source_pos;
        let arrow_start = source_pos + dest_to_src.normalize() * 0.5;
        let arrow_end = source_pos + dest_to_src.normalize() * (dest_to_src.length() - 0.5);
        gizmos.arrow(arrow_start, arrow_end, Color::srgb(1., 1., 1.));
    }
}
