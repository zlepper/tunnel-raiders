mod building_menu;
mod depot_building;
use crate::prelude::*;
pub use depot_building::{BuildingInfo, is_placing_building};

pub struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            building_menu::BuildingMenuPlugin,
            depot_building::DepotBuildingPlugin,
        ));
    }
}

#[derive(Component)]
pub struct OpenForBuilding;
