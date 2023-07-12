use crate::prelude::*;
use crate::errands::move_to_position_errand::MoveToPositionErrandPlugin;

pub mod mine_wall_errand;
pub mod move_to_position_errand;
mod sleep_errand;
mod errand_queue;

use mine_wall_errand::MineWallErrandPlugin;
use sleep_errand::{execute_sleep_errand, SleepErrand};
pub use mine_wall_errand::{Minable, MineWallErrand, Miner};
pub use move_to_position_errand::{MoveToPosition, Standable, PlayerMovable};
pub use errand_queue::{Errand, ErrandQueue, ErrandQueuePluginExtensions, ErrandQueueable};
use errand_queue::dequeue_next_errand;

pub struct ErrandsPlugin;

impl Plugin for ErrandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(dequeue_next_errand)
            .add_system(execute_sleep_errand.run_if(any_with_component::<SleepErrand>()))
            .register_errand::<SleepErrand>()
            .add_plugin(MoveToPositionErrandPlugin)
            .add_plugin(MineWallErrandPlugin);
    }
}
