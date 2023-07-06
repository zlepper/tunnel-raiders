use crate::prelude::*;
use bevy_ecs::system::EntityCommands;
use std::collections::VecDeque;
use crate::tasks::move_to_position_task::MoveToPositionTaskPlugin;

pub mod move_to_position_task;

#[derive(Component)]
pub struct TaskQueue {
    tasks: VecDeque<Box<dyn QueuedTask>>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
        }
    }

    pub fn add_task(&mut self, task: impl Component + Task) {
        self.tasks.push_back(Box::new(QueuedComponentTask::new(task)));
    }

    pub fn override_task(&mut self, task: impl Component + Task) {
        self.tasks.clear();
        self.tasks.push_back(Box::new(QueuedComponentTask::new(task)));
    }
}

pub trait QueuedTask: Sync + Send + 'static {
    fn insert_component(&mut self, commands: &mut EntityCommands);
}

pub struct QueuedComponentTask<T: Component + Task>(Option<T>);

impl<T : Component + Task> QueuedComponentTask<T> {
    pub fn new(comp: T) -> Self {
        Self(Some(comp))
    }
}

impl<T: Component + Task> QueuedTask for QueuedComponentTask<T>
{
    fn insert_component(&mut self, commands: &mut EntityCommands) {
        if let Some(comp) = self.0.take() {
            commands.insert(comp);
        }
    }
}

fn on_task_finished_system<T: Component>(
    mut finished: RemovedComponents<T>,
    mut commands: Commands,
    mut task_queues: Query<&mut TaskQueue>,
) {
    for entity in finished.iter() {
        if let Ok(mut task_queue) = task_queues.get_mut(entity) {
            let mut entity_commands = commands.entity(entity);
            if let Some(mut task) = task_queue.tasks.pop_front() {
                task.insert_component(&mut entity_commands);
            } else {
                entity_commands.remove::<WorkingOnTask>();
            }
        } else {
            warn!("Entity {:?} finished task, but has no task queue", entity);
        }
    }
}

fn has_finished_task<T: Component>(finished: RemovedComponents<T>) -> bool {
    finished.len() > 0
}

pub trait TaskQueuePluginExtensions {
    fn register_task<T: Component + Task>(&mut self) -> &mut Self;
}

impl TaskQueuePluginExtensions for App {
    fn register_task<T: Component + Task>(&mut self) -> &mut Self {
        self.add_system(on_task_finished_system::<T>.run_if(has_finished_task::<T>))
    }
}

pub struct TaskQueuePlugin;

impl Plugin for TaskQueuePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(dequeue_next_task)
            .add_system(execute_sleep_task.run_if(any_with_component::<SleepTask>()))
            .register_task::<SleepTask>()
            .add_plugin(MoveToPositionTaskPlugin);
    }
}


#[derive(Component)]
struct WorkingOnTask;

fn dequeue_next_task(
    mut workers: Query<(Entity, &mut TaskQueue), Without<WorkingOnTask>>,
    mut commands: Commands,
) {
    for (entity, mut queue) in workers.iter_mut() {
        if let Some(mut next) = queue.tasks.pop_front() {
            let mut entity_commands = commands.entity(entity);
            next.insert_component(&mut entity_commands);
            entity_commands.insert(WorkingOnTask);
        }
    }
}

pub trait Task {
    fn name(&self) -> String;
}

#[derive(Component)]
pub struct SleepTask {
    pub duration: f32,
}

impl Task for SleepTask {
    fn name(&self) -> String {
        format!("Sleep for {} seconds", self.duration)
    }
}

fn execute_sleep_task(
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
