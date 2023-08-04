use crate::prelude::*;

#[derive(Component, Debug)]
pub struct Health {
    pub max: f32,
    pub current: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self {
            max,
            current: max,
        }
    }
}

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, remove_when_out_of_health);
    }
}

fn remove_when_out_of_health(q: Query<(Entity, &Health, &GlobalTransform)>, mut commands: Commands) {
    for (entity, health, transform) in q.iter() {
        if health.current <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub struct 
