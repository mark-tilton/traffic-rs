use bevy::prelude::*;

use crate::movement::{Acceleration, Velocity};

pub fn spawn_vehicles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(5., 2., 3.).mesh()),
            material: materials.add(Color::srgb(0.3, 0.3, 0.5)),
            ..default()
        },
        Velocity::default(),
        Acceleration {
            acceleration: Vec3::new(1., 0., 0.),
        },
    ));
}
