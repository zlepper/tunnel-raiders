use crate::prelude::*;
use crate::tasks::move_to_position_task::MoveToPositionTaskPlugin;

pub mod mine_wall_task;
pub mod move_to_position_task;
mod sleep_task;
mod task_queue;

use mine_wall_task::MineWallTaskPlugin;
use sleep_task::{execute_sleep_task, SleepTask};
pub use mine_wall_task::{Minable, MineWallTask, Miner};
pub use move_to_position_task::{MoveToPosition, Standable, PlayerMovable};
pub use task_queue::{Task, TaskQueue, TaskQueuePluginExtensions};
use task_queue::dequeue_next_task;

pub struct TasksPlugin;

impl Plugin for TasksPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(dequeue_next_task)
            .add_system(execute_sleep_task.run_if(any_with_component::<SleepTask>()))
            .register_task::<SleepTask>()
            .add_plugin(MoveToPositionTaskPlugin)
            .add_plugin(MineWallTaskPlugin);
    }
}
