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

fn remove_when_out_of_health(q: Query<(Entity, &Health, &GlobalTransform, Option<&OnDeathAction>)>, mut commands: Commands) {
    for (entity, health, transform, on_death_action) in q.iter() {
        if health.current <= 0.0 {
            if let Some(on_death_action) = on_death_action {
                on_death_action.action.on_death(entity, &mut commands, transform);
            }

            commands.entity(entity).despawn_recursive();
        }
    }
}


pub trait DeathAction: Send + Sync + 'static {
    fn on_death(&self, entity: Entity, commands: &mut Commands, transform: &GlobalTransform);
}

#[derive(Component)]
pub struct OnDeathAction {
    action: Box<dyn DeathAction>,
}

impl OnDeathAction {
    pub fn new(action: impl DeathAction) -> Self {
        Self {
            action: Box::new(action),
        }
    }
}
