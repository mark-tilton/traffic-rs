use bevy::prelude::*;

mod nodegraph;

fn main() {
    let graph = nodegraph::NodeGraph::create();
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, nodegraph::spawn_vehicle)
        .add_systems(Update, nodegraph::move_vehicles)
        .add_systems(Update, nodegraph::show_node_graph)
        .insert_resource(graph)
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
