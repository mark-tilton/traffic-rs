use rand::{self, seq::IteratorRandom};

use bevy::prelude::*;

use crate::{
    node_graph::{Node, NodeGraph},
    vehicle_spawn_limiter::VehicleSpawnLimiter,
};

const MIN_SPEED: f32 = 1.;
const MAX_SPEED: f32 = 10.;

#[derive(Component)]
pub struct Vehicle {
    // A pre-calculated node path through the network
    path: Vec<usize>,
    // The position of the vehicle along the node path
    path_index: usize,
    // A parameterized value along the edge described by
    // (path[path_index], path[path_index+1])
    edge_position: f32,
    // The speed of the vehicle
    speed: f32,
}

impl Vehicle {
    fn new(path: Vec<usize>) -> Self {
        Vehicle {
            path,
            path_index: 0,
            edge_position: 0.,
            speed: MIN_SPEED + (MAX_SPEED - MIN_SPEED) * rand::random::<f32>(),
        }
    }

    // These getter functions will panic if the vehicle is in a malformed state or
    // if the node graph is mutated
    fn get_current_node<'a>(&self, node_graph: &'a NodeGraph) -> &'a Node {
        let node_index = self.path[self.path_index];
        &node_graph.nodes[node_index]
    }

    // A vehicle at the end of its path will not have a next node so the result is
    // optional. This call will still panic in any case where the the index is invalid
    // but we are not on the last node of the path.
    fn get_next_node<'a>(&self, node_graph: &'a NodeGraph) -> Option<&'a Node> {
        if self.path_index == self.path.len() - 1 {
            return None;
        }
        let node_index = self.path[self.path_index + 1];
        Some(&node_graph.nodes[node_index])
    }

    // Gets the world position of the vehicle by interpolating between the
    // positions of the current and next nodes
    fn get_world_position(&self, node_graph: &NodeGraph) -> Vec3 {
        let current_node_pos = self.get_current_node(node_graph).position;
        let Some(next_node) = self.get_next_node(node_graph) else {
            // If there is no next node, the position will just be the current(last) node.
            return current_node_pos;
        };
        current_node_pos + (next_node.position - current_node_pos) * self.edge_position
    }

    // Attempts to drive along the current edge by a given world space distance.
    // If the vehicle hits the end of the edge, the path will be incremented and
    // the remaining distance will be returned.
    fn drive_edge(&mut self, distance: f32, node_graph: &NodeGraph) -> f32 {
        // Calculate the parameterized speed of the vehicle along the edge
        // by querying the current and next nodes
        let current_node = self.get_current_node(node_graph);
        let Some(next_node) = self.get_next_node(node_graph) else {
            // If there is no next node, there is no remaining distance to drive
            return 0.;
        };
        let edge_vector = next_node.position - current_node.position;
        let edge_speed = distance / edge_vector.length();

        // Move the vehicle along the edge. If we go past the end of the
        // edge, increment to the next edge.
        self.edge_position += edge_speed;
        if self.edge_position > 1. {
            let overshoot = self.edge_position - 1.;
            self.path_index += 1;
            self.edge_position = 0.;
            return overshoot * edge_vector.length();
        }
        0.
    }

    // Drives along the vehicles node path by a specified world space distance
    fn drive(&mut self, distance: f32, node_graph: &NodeGraph) {
        let mut remaining_distance = distance;
        while remaining_distance > 0. {
            remaining_distance = self.drive_edge(remaining_distance, node_graph);
        }
    }
}

pub fn spawn_vehicle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    node_graph: Res<NodeGraph>,
    mut spawn_limiter: ResMut<VehicleSpawnLimiter>,
) {
    // Only allow vehicle spawning at certain intervals
    if !spawn_limiter.try_spawn() {
        return;
    }

    // Choose random source and destination nodes
    let mut rng = rand::thread_rng();
    let ((start_node, _), node_path) = node_graph
        .shortest_path_map
        .iter()
        .choose(&mut rng)
        .expect("No path found");

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
        Vehicle::new(node_path.clone()),
    ));
}

pub fn move_vehicles(
    mut commands: Commands,
    mut vehicle_query: Query<(Entity, &mut Transform, &mut Vehicle)>,
    node_graph: Res<NodeGraph>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut vehicle) in &mut vehicle_query {
        let speed = vehicle.speed;

        // Drive the given distance and update the position of the transform
        vehicle.drive(speed * time.delta_seconds(), &node_graph);
        transform.translation = vehicle.get_world_position(&node_graph);

        // Despawn the vehicle if it's on the final node.
        let Some(next_node) = vehicle.get_next_node(&node_graph) else {
            commands.entity(entity).despawn();
            continue;
        };
        transform.look_at(next_node.position, Dir3::Y);
    }
}
