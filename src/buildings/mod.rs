mod building;
mod building_menu;
mod depot_building;

use crate::buildings::building::{cancel_building, place_building, update_placeholder_render};
use crate::prelude::*;
pub use building::{is_placing_building, Building, BuildingAppExtensions, BuildingInfo};

pub struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            building_menu::BuildingMenuPlugin,
            depot_building::DepotBuildingPlugin,
        ))
        .add_systems(
            Update,
            (place_building, cancel_building, update_placeholder_render)
                .run_if(is_placing_building),
        );
    }
}

#[derive(Component)]
pub struct OpenForBuilding;
