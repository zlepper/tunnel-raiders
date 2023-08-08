use crate::prelude::*;

pub struct DepotBuildingPlugin;

impl Plugin for DepotBuildingPlugin {
    fn build(&self, app: &mut App) {
        app.load_assets::<DepotAssets>()
            .add_building::<DepotBuilding>()
            .add_building::<AnotherDepotBuilding>();
    }
}

#[derive(AssetCollection, Resource)]
struct DepotAssets {
    #[asset(path = "buildings/depot.gltf#Scene0")]
    depot: Handle<Scene>,

    #[asset(path = "buildings/depot.png")]
    depot_icon: Handle<Image>,
}

#[derive(Clone)]
struct DepotBuilding {
    model: Handle<Scene>,
    icon: Handle<Image>,
}

impl Building for DepotBuilding {
    type Assets = DepotAssets;

    fn get_model(&self) -> Handle<Scene> {
        self.model.clone()
    }

    fn get_name() -> String {
        "Depot".to_string()
    }

    fn get_order() -> i32 {
        0
    }

    fn get_icon(&self) -> Handle<Image> {
        self.icon.clone()
    }

    fn initialize(assets: &Self::Assets) -> Self {
        info!("Initializing DepotBuilding: {:?}", assets.depot);
        Self {
            icon: assets.depot_icon.clone(),
            model: assets.depot.clone(),
        }
    }
}

#[derive(Clone)]
struct AnotherDepotBuilding {
    model: Handle<Scene>,
    icon: Handle<Image>,
}

impl Building for AnotherDepotBuilding {
    type Assets = DepotAssets;

    fn get_model(&self) -> Handle<Scene> {
        self.model.clone()
    }

    fn get_name() -> String {
        "Another Depot".to_string()
    }

    fn get_order() -> i32 {
        1
    }

    fn get_icon(&self) -> Handle<Image> {
        self.icon.clone()
    }

    fn initialize(assets: &Self::Assets) -> Self {
        Self {
            icon: assets.depot_icon.clone(),
            model: assets.depot.clone(),
        }
    }
}
