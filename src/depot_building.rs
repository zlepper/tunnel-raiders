use crate::{GameState};
use crate::prelude::*;

struct DepotBuildingPlugin;

impl Plugin for DepotBuildingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_collection_to_loading_state::<_, DepotAssets>(GameState::Loading);
    }
}




#[derive(AssetCollection, Resource)]
struct DepotAssets {
    #[asset(path = "buildings/depot.glb#Scene0")]
    depot: Handle<Scene>,
}

















