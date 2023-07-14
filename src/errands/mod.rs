use crate::prelude::*;
use crate::errands::move_to_position_errand::MoveToPositionErrandPlugin;

pub mod mine_wall_errand;
pub mod move_to_position_errand;
mod sleep_errand;
mod errands_v2;

use mine_wall_errand::MineWallErrandPlugin;
use sleep_errand::{execute_sleep_errand, SleepErrand};
pub use mine_wall_errand::{Minable, MineWallErrand, Miner};
pub use move_to_position_errand::{MoveToPosition, Standable, PlayerMovable};
pub use errands_v2::*;

pub struct ErrandsPlugin;

impl Plugin for ErrandsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(execute_sleep_errand)
            .add_errand::<SleepErrand>()
            .add_plugin(MoveToPositionErrandPlugin)
            .add_plugin(MineWallErrandPlugin)
            .add_plugin(ErrandsV2Plugin);
    }
}
