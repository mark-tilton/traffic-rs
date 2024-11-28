use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct VehicleIdGenerator {
    id: usize,
}

impl VehicleIdGenerator {
    pub fn get_id(&mut self) -> usize {
        let id = self.id;
        self.id += 1;
        id
    }
}
