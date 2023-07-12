use crate::prelude::*;
use crate::errands::errand_queue::Errand;

#[derive(Component)]
pub struct SleepErrand {
    pub duration: f32,
}

impl Errand for SleepErrand {
    fn name(&self) -> String {
        format!("Sleep for {} seconds", self.duration)
    }
}

pub fn execute_sleep_errand(
    mut query: Query<(Entity, &mut SleepErrand)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut errand) in query.iter_mut() {
        errand.duration -= time.delta_seconds();
        if errand.duration <= 0.0 {
            commands.entity(entity).remove::<SleepErrand>();
        }
    }
}
