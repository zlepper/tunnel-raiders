use crate::prelude::*;

#[derive(Component)]
pub struct SleepTask(pub f32);

#[derive(Component)]
pub struct LogMessageTask(pub String);


fn execute_sleep_task(mut commands: Commands, mut sleep_tasks: Query<(Entity, &mut SleepTask)>, time: Res<Time>) {
    for (entity, mut task) in sleep_tasks.iter_mut() {
        task.0 -= time.delta_seconds();
        if task.0 <= 0.0 {
            commands.entity(entity).remove::<SleepTask>();
        }
    }
}

fn execute_log_task(mut commands: Commands, log_tasks: Query<(Entity, &LogMessageTask)>) {
    for (entity, task) in log_tasks.iter() {
        info!("{}", task.0);
        commands.entity(entity).remove::<LogMessageTask>();
    }
}

pub struct MoveToPositionTaskPlugin;

impl Plugin for MoveToPositionTaskPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((execute_sleep_task, execute_log_task))
            .register_task::<SleepTask>()
            .register_task::<LogMessageTask>();
    }
}
