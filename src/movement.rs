use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Velocity {
    pub velocity: Vec3,
}

pub fn velocity(mut velocity_query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut velocity_query {
        transform.translation += velocity.velocity * time.delta().as_secs_f32();
    }
}

#[derive(Component, Default)]
pub struct Acceleration {
    pub acceleration: Vec3,
}

pub fn acceleration(
    mut acceleration_query: Query<(&mut Velocity, &Acceleration)>,
    time: Res<Time>,
) {
    for (mut velocity, acceleration) in &mut acceleration_query {
        velocity.velocity += acceleration.acceleration * time.delta().as_secs_f32();
    }
}
