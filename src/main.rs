use std::time::Duration;

use bevy::prelude::*;
use node_graph_renderer::HighlightedEdgeGizmos;

mod node_graph;
mod node_graph_renderer;
mod path_finding_data;
mod vehicle_id_generator;
mod vehicle_spawn_limiter;
mod vehicles;

fn main() {
    let graph = node_graph::NodeGraph::create();
    let path_finding_data = path_finding_data::PathFindingData::new(&graph);
    let graph_renderer = node_graph_renderer::NodeGraphRenderer::default();
    let spawn_interval = Duration::from_millis(500);
    let spawn_limiter = vehicle_spawn_limiter::VehicleSpawnLimiter::new(spawn_interval);
    App::new()
        .add_plugins(DefaultPlugins)
        .init_gizmo_group::<HighlightedEdgeGizmos>()
        .add_systems(Startup, setup)
        .add_systems(Startup, node_graph_renderer::configure_gizmos)
        .add_systems(Update, vehicles::spawn_vehicle)
        .add_systems(Update, vehicles::move_vehicles)
        .add_systems(Update, node_graph_renderer::show_node_graph)
        .insert_resource(graph)
        .insert_resource(graph_renderer)
        .insert_resource(path_finding_data)
        .insert_resource(spawn_limiter)
        .insert_resource(vehicle_id_generator::VehicleIdGenerator::default())
        .run();
}

#[derive(Component)]
struct Ground;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(25., 25.)),
            material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
            transform: Transform::from_xyz(0., -1., 0.),
            ..default()
        },
        Ground,
    ));

    // light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 25., 15.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
