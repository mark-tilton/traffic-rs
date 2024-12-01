use core::f32;
use std::collections::HashMap;

use rand::{self, seq::IteratorRandom};

use bevy::prelude::*;

use crate::{
    node_graph::{Node, NodeGraph},
    node_graph_renderer::NodeGraphRenderer,
    vehicle_id_generator::VehicleIdGenerator,
    vehicle_spawn_limiter::VehicleSpawnLimiter,
};

const MIN_SPEED: f32 = 4.;
const MAX_SPEED: f32 = 10.;

#[derive(Component)]
pub struct Vehicle {
    id: usize,
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
    fn new(id: usize, path: Vec<usize>) -> Self {
        Vehicle {
            id,
            path,
            path_index: 0,
            edge_position: 0.,
            speed: MIN_SPEED + (MAX_SPEED - MIN_SPEED) * rand::random::<f32>(),
        }
    }

    // These getter functions will panic if the vehicle is in a malformed state or
    // if the node graph is mutated
    fn get_current_node<'a>(&self, node_graph: &'a NodeGraph) -> &'a Node {
        &node_graph.nodes[self.get_current_node_index()]
    }

    fn get_current_node_index(&self) -> usize {
        self.path[self.path_index]
    }

    // A vehicle at the end of its path will not have a next node so the result is
    // optional. This call will still panic in any case where the the index is invalid
    // but we are not on the last node of the path.
    fn get_next_node<'a>(&self, node_graph: &'a NodeGraph) -> Option<&'a Node> {
        match self.get_next_node_index() {
            Some(node_index) => Some(&node_graph.nodes[node_index]),
            None => None,
        }
    }

    fn get_next_node_index(&self) -> Option<usize> {
        if self.path_index == self.path.len() - 1 {
            return None;
        }
        Some(self.path[self.path_index + 1])
    }

    fn get_edge(&self) -> Option<(usize, usize)> {
        let next_node = self.get_next_node_index()?;
        Some((self.get_current_node_index(), next_node))
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

    // Gets the distance in edge space to the next vehicle on the current edge.
    // Returns None if there are no vehicles in front of the vehicle.
    fn get_next_vehicle_edge_distance(
        &self,
        vehicle_map: &HashMap<(usize, usize), Vec<f32>>,
    ) -> Option<f32> {
        let edge = self.get_edge()?;
        let vehicles_on_edge = vehicle_map.get(&edge)?;
        let mut closest_vehicle = None;
        for edge_position in vehicles_on_edge {
            let vehicle_distance = edge_position - self.edge_position;
            // Ignore self and trailing vehicles
            if vehicle_distance <= 0. {
                continue;
            }
            if closest_vehicle.is_none() || vehicle_distance < closest_vehicle? {
                closest_vehicle = Some(vehicle_distance);
            }
        }
        closest_vehicle
    }

    // Attempts to drive along the current edge by a given world space distance.
    // If the vehicle hits the end of the edge, the path will be incremented and
    // the remaining distance will be returned.
    fn drive_edge(
        &mut self,
        distance: f32,
        node_graph: &mut NodeGraph,
        vehicle_map: &HashMap<(usize, usize), Vec<f32>>,
    ) -> f32 {
        // Calculate the parameterized speed of the vehicle along the edge
        // by querying the current and next nodes
        let current_node = self.get_current_node(node_graph);
        let Some(next_node) = self.get_next_node(node_graph) else {
            // If there is no next node, there is no remaining distance to drive
            return 0.;
        };
        let edge_vector = next_node.position - current_node.position;
        let edge_length = edge_vector.length();
        let mut edge_move_amount = distance / edge_length;

        // Clamp move amount to not pass the next vehicle
        if let Some(next_vehicle_distance) = self.get_next_vehicle_edge_distance(vehicle_map) {
            let follow_distance = 0.7;
            let edge_follow_distance = follow_distance / edge_length;
            let follow_point = next_vehicle_distance - edge_follow_distance;
            if follow_point < edge_move_amount {
                edge_move_amount = follow_point;
            }
        }

        let new_edge_position = self.edge_position + edge_move_amount;

        // The distance a vehicle should stay back from a node when waiting
        // Note: make sure this smaller than (min dist between connected nodes along a bidirectional edge / 2)
        let node_buffer = 0.9;
        let edge_buffer = node_buffer / edge_length;

        self.try_clear_node_reservation(edge_buffer, node_graph);
        if self.should_wait_at_node(edge_buffer, new_edge_position, node_graph) {
            // move vehicle as close to node as possible and wait for reservation
            self.edge_position = 1.0 - edge_buffer;
            return 0.;
        }

        // Move the vehicle along the edge. If we go past the end of the
        // edge, increment to the next edge.
        self.edge_position = new_edge_position;
        if self.edge_position > 1. {
            let overshoot = self.edge_position - 1.;
            self.path_index += 1;
            self.edge_position = 0.;
            return overshoot * edge_vector.length();
        }
        0.
    }

    fn should_wait_at_node(
        &self,
        edge_buffer: f32,
        new_edge_position: f32,
        node_graph: &mut NodeGraph,
    ) -> bool {
        // don't wait if there is no next node
        let Some(next_node_index) = self.get_next_node_index() else {
            return false;
        };

        // don't wait if the next node is our destination
        if self.path_index == self.path.len() - 2 {
            return false;
        }

        // don't wait if we are outside of the reservation range of the next node
        let distance_to_next_node = 1.0 - new_edge_position;
        if distance_to_next_node > edge_buffer {
            return false;
        }

        // get the vehicle id which reserved the node
        if let Some(vehicle_id_with_reservation) =
            node_graph.node_reservation_map.get(&next_node_index)
        {
            // TODO: update this to allow following cars through intersections
            // this can be accomplished by checking the direction of the car with
            // the reservation and if it is the same then overwrite the reservation
            // with this vehicle's id.

            // stop driving if this node is reserved by another vehicle
            self.id != *vehicle_id_with_reservation
        } else {
            // there's no reservation, reserve it
            node_graph
                .node_reservation_map
                .insert(next_node_index, self.id);
            false
        }
    }

    fn try_clear_node_reservation(&self, edge_buffer: f32, node_graph: &mut NodeGraph) {
        // check if we are outside the reservation range of the current node
        let current_node_index = self.get_current_node_index();
        if self.edge_position < edge_buffer {
            return;
        }

        // don't need to clear reservation if the current node has no reservation
        let Some(reserved_current_node) = node_graph.node_reservation_map.get(&current_node_index)
        else {
            return;
        };

        // check if we have the reservation
        if *reserved_current_node == self.id {
            // clear the reservation
            node_graph.node_reservation_map.remove(&current_node_index);
        }
    }

    // Drives along the vehicles node path by a specified world space distance
    fn drive(
        &mut self,
        distance: f32,
        node_graph: &mut NodeGraph,
        vehicle_map: &HashMap<(usize, usize), Vec<f32>>,
    ) {
        let mut remaining_distance = distance;
        while remaining_distance > 0. {
            remaining_distance = self.drive_edge(remaining_distance, node_graph, vehicle_map);
        }
    }
}

pub fn spawn_vehicle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    node_graph: Res<NodeGraph>,
    mut node_graph_renderer: ResMut<NodeGraphRenderer>,
    mut spawn_limiter: ResMut<VehicleSpawnLimiter>,
    mut vehicle_id_generator: ResMut<VehicleIdGenerator>,
) {
    // Only allow vehicle spawning at certain intervals
    if !spawn_limiter.try_spawn() {
        return;
    }

    // Choose random source and destination nodes
    let mut rng = rand::thread_rng();
    let ((source_node, dest_node), node_path) = node_graph
        .shortest_path_map
        .iter()
        .choose(&mut rng)
        .expect("No path found");

    let vehicle_id = vehicle_id_generator.get_id();

    // Highlight this vehicle if there is no current highlight
    let highlight_vehicle = node_graph_renderer.highlighted_vehicle_id.is_none();
    let vehicle_color: Color;
    if highlight_vehicle {
        node_graph_renderer.highlighted_vehicle_id = Some(vehicle_id);
        node_graph_renderer.highlighted_path_index = Some((*source_node, *dest_node));
        vehicle_color = Color::srgb(1., 1., 0.);
    } else {
        vehicle_color = Color::srgb(0.3, 0.3, 0.5);
    }

    // Spawn the vehicle entity at the correct position.
    // If we don't get the position here, the entity will be displayed
    // at the center of the scene for a frame.
    let start_node_position = node_graph.nodes.get(*source_node).unwrap().position;
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.3, 0.2, 0.5).mesh()),
            material: materials.add(vehicle_color),
            transform: Transform::from_translation(start_node_position),
            ..default()
        },
        Vehicle::new(vehicle_id, node_path.clone()),
    ));
}

pub fn move_vehicles(
    mut commands: Commands,
    mut vehicle_query: Query<(Entity, &mut Transform, &mut Vehicle)>,
    mut node_graph: ResMut<NodeGraph>,
    mut node_graph_renderer: ResMut<NodeGraphRenderer>,
    time: Res<Time>,
) {
    // Build a map to communicate vehicle positions between vehicles
    let mut vehicle_map: HashMap<(usize, usize), Vec<f32>> = HashMap::new();
    for (_, _, vehicle) in &mut vehicle_query {
        let Some(edge) = vehicle.get_edge() else {
            continue;
        };
        vehicle_map
            .entry(edge)
            .or_default()
            .push(vehicle.edge_position);
    }

    for (entity, mut transform, mut vehicle) in &mut vehicle_query {
        let speed = vehicle.speed;

        // Drive the given distance and update the position of the transform
        vehicle.drive(
            speed * time.delta_seconds(),
            node_graph.as_mut(),
            &vehicle_map,
        );
        transform.translation = vehicle.get_world_position(&node_graph);

        // Despawn the vehicle if it's on the final node.
        let Some(next_node) = vehicle.get_next_node(&node_graph) else {
            // Clear the highlight if this vehicle was being highlighted
            if let Some(highlighted_vehicle_id) = node_graph_renderer.highlighted_vehicle_id {
                if highlighted_vehicle_id == vehicle.id {
                    node_graph_renderer.highlighted_vehicle_id = None;
                    node_graph_renderer.highlighted_path_index = None;
                }
            }

            commands.entity(entity).despawn();
            continue;
        };

        transform.look_at(next_node.position, Dir3::Y);
    }
}
