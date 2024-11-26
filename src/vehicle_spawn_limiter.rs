use std::time::{Duration, Instant};

use bevy::prelude::Resource;

#[derive(Resource, Default)]
pub struct VehicleSpawnLimiter {
    interval: Duration,
    last_spawned: Option<Instant>
}

impl VehicleSpawnLimiter {
    pub fn new(interval: Duration) -> Self {
        VehicleSpawnLimiter {
            interval,
            last_spawned: Option::None,
        }
    }

    pub fn try_spawn(&mut self) -> bool {
        if let Some(last_spawned) = self.last_spawned {
            if last_spawned.elapsed() < self.interval {
                return false;
            }
        }
        self.last_spawned = Some(Instant::now());
        return true;
    }
}
