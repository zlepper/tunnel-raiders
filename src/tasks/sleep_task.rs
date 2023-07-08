use crate::prelude::*;
use crate::tasks::task_queue::Task;

#[derive(Component)]
pub struct SleepTask {
    pub duration: f32,
}

impl Task for SleepTask {
    fn name(&self) -> String {
        format!("Sleep for {} seconds", self.duration)
    }
}

pub fn execute_sleep_task(
    mut query: Query<(Entity, &mut SleepTask)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut task) in query.iter_mut() {
        task.duration -= time.delta_seconds();
        if task.duration <= 0.0 {
            commands.entity(entity).remove::<SleepTask>();
        }
    }
}
