use rand::{self, seq::IteratorRandom};

use bevy::prelude::*;

use crate::{node_graph::NodeGraph, vehicle_spawn_limiter::VehicleSpawnLimiter};

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
        let start_node_index = *vehicle
            .path
            .get(vehicle.path_index)
            .expect("Vehicle path index past end of path");
        let next_node_index = *vehicle
            .path
            .get(vehicle.path_index + 1)
            .expect("Attempted to process node at the end of its path");
        let start_node = node_graph
            .nodes
            .get(start_node_index)
            .expect("Node doesn't exist in the graph");
        let next_node = node_graph
            .nodes
            .get(next_node_index)
            .expect("Node doesn't exist in the graph");
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
