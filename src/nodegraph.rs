use rand::{self, seq::IteratorRandom};
use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::vehicle_spawn_limiter::VehicleSpawnLimiter;

#[derive(Component)]
pub struct Vehicle {
    // A pre-calculated node path through the network
    path: Vec<usize>,
    // The position of the vehicle along the node path
    path_index: usize,
    // A parameterized value along the edge described by
    // (path[path_index], path[path_index+1])
    edge_position: f32,
}

#[derive(Clone)]
pub struct Node {
    position: Vec3,
}

#[derive(Resource)]
pub struct NodeGraph {
    nodes: Vec<Node>,
    edges: HashSet<(usize, usize)>,
    // Source nodes are nodes that have no other nodes pointing to them
    source_nodes: HashSet<usize>,
    // Destination nodes are nodes that don't have any nodes leading from them
    dest_nodes: HashSet<usize>,
    // A convenient data structure for navigating forward
    // through the graph.
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

pub fn spawn_vehicle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    node_graph: ResMut<NodeGraph>,
    mut spawn_limiter: ResMut<VehicleSpawnLimiter>,
) {
    // Only allow vehicle spawning at certain intervals
    if !spawn_limiter.try_spawn() {
        return;
    }

    // Choose a random starting node for the path.
    let mut rng = rand::thread_rng();
    let Some(start_node) = node_graph.source_nodes.iter().choose(&mut rng) else {
        return;
    };

    // Choose random connecting nodes until we hit a terminal node.
    // This loop will run forever if there are any loops that
    // can't eventually hit a destination node.
    let mut node_path = vec![*start_node];
    loop {
        let latest_node = node_path.last().unwrap();

        let next_node = node_graph
            .node_map
            .get(latest_node)
            .expect("Invalid node path")
            .iter()
            .choose(&mut rng)
            .expect("Invalid edge map");
        node_path.push(*next_node);

        // If the new node is a destination node, we're done!
        if node_graph.dest_nodes.contains(next_node) {
            break;
        }
    }

    // Spawn the vehicle entity at the correct position.
    // If we don't get the position here, the entity will be displayed
    // at the center of the scene for a frame.
    let start_node_position = node_graph.nodes.get(*start_node).unwrap().position;
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.3, 0.2, 0.5).mesh()),
            material: materials.add(Color::srgb(0.3, 0.3, 0.5)),
            transform: Transform::from_translation(start_node_position),
            ..default()
        },
        Vehicle {
            path: node_path,
            path_index: 0,
            edge_position: 0.,
        },
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

        // Calculate the parameterized speed of the vehicle along the edge
        // by querying the start and end nodes.
        let start_node = node_graph
            .nodes
            .get(*vehicle.path.get(vehicle.path_index).unwrap())
            .unwrap();
        let next_node = node_graph
            .nodes
            .get(*vehicle.path.get(vehicle.path_index + 1).unwrap())
            .unwrap();
        let start_to_next = next_node.position - start_node.position;
        let edge_speed = speed / start_to_next.length();

        // Move the vehicle along the edge. If we go past the end of the
        // edge, increment to the next edge.
        vehicle.edge_position += edge_speed * time.delta_seconds();
        if vehicle.edge_position > 1. {
            vehicle.path_index += 1;
            vehicle.edge_position = 0.;

            // If we are at the end of the node path, despawn the vehicle.
            if vehicle.path_index >= vehicle.path.len() - 1 {
                commands.entity(entity).despawn();
            }

            // If we go to the next edge, skip updating the transform.
            // Next frame the vehicle will be snapped to the next edge.
            // We could / should recalculate the edge speed here and
            // continue along the next edge.
            continue;
        }

        // Update the postiion of the vehicle entity.
        transform.translation = start_node.position + start_to_next * vehicle.edge_position;
        transform.look_at(next_node.position, Dir3::Y);
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
