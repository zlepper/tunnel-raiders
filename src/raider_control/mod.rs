use crate::prelude::*;
use crate::raider_control::walk_raider_to_target::move_selected_raider_to_target;

mod mine_wall;
mod walk_raider_to_target;

use crate::raider_control::mine_wall::start_mining_wall;
pub use walk_raider_to_target::{PlayerMovable, Standable};
pub use mine_wall::{Miner};

pub struct RaiderControlPlugin;

impl Plugin for RaiderControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_selected_raider_to_target)
            .add_system(start_mining_wall);
    }
}
