use crate::prelude::*;

pub struct BuildingMenuPlugin;

impl Plugin for BuildingMenuPlugin {
    fn build(&self, app: &mut App) {
        app.load_assets::<BuildingMenuAssets>()
            .add_menu_gizmo::<BuildingListGizmo>();
    }
}

#[derive(AssetCollection, Resource)]
pub struct BuildingMenuAssets {
    #[asset(path = "buildings/building-menu.png")]
    main_menu: Handle<Image>,
}

#[derive(Resource)]
pub struct BuildingListGizmo(ButtonGizmo);

impl HasBaseGizmo for BuildingListGizmo {
    fn get_base_gizmo(&self) -> &ButtonGizmo {
        &self.0
    }
}

impl GizmoVisibility for BuildingListGizmo {
    type WorldQuery = ();
    type ReadOnlyWorldQuery = With<Selected>;

    fn is_visible(query: &Query<Self::WorldQuery, Self::ReadOnlyWorldQuery>) -> bool {
        query.is_empty()
    }
}

impl Gizmo for BuildingListGizmo {
    type Assets = BuildingMenuAssets;

    fn initialize(assets: &Self::Assets) -> Self {
        Self(ButtonGizmo::new(assets.main_menu.clone(), "Build", 0))
    }
}

impl MenuGizmo for BuildingListGizmo {}



