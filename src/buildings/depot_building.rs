use crate::buildings::building_menu::BuildingListGizmo;
use crate::buildings::OpenForBuilding;
use crate::camera_control::MouseTargetedEntity;
use crate::prelude::*;
use std::ops::Deref;

pub struct DepotBuildingPlugin;

impl Plugin for DepotBuildingPlugin {
    fn build(&self, app: &mut App) {
        app.load_assets::<DepotAssets>()
            .add_building::<DepotBuilding>()
            .add_building::<AnotherDepotBuilding>()
            .add_systems(
                Update,
                (place_building, cancel_building, update_placeholder_render)
                    .run_if(is_placing_building),
            );
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

// Building.rs later on

pub trait Building: Clone + Send + Sync + 'static {
    type Assets: Resource + 'static;

    fn get_model(&self) -> Handle<Scene>;
    fn get_name() -> String;
    fn get_order() -> i32;
    fn get_icon(&self) -> Handle<Image>;
    fn initialize(assets: &Self::Assets) -> Self;
}

pub trait BuildingInfo: Send + Sync + 'static {
    fn get_model(&self) -> Handle<Scene>;
}

struct BuildingInfoWrapper<B: Building>(B);

impl<B: Building> BuildingInfo for BuildingInfoWrapper<B> {
    fn get_model(&self) -> Handle<Scene> {
        self.0.get_model()
    }
}

#[derive(Resource)]
struct BuildingGizmo<B: Building> {
    building: B,
}

impl<B: Building> GizmoChildOf for BuildingGizmo<B> {
    type Parent = BuildingListGizmo;
}

impl<B: Building> BaseGizmo for BuildingGizmo<B> {
    fn get_icon(&self) -> Handle<Image> {
        self.building.get_icon()
    }

    fn get_name(&self) -> String {
        B::get_name()
    }

    fn get_order(&self) -> i32 {
        B::get_order()
    }
}

impl<B: Building> Gizmo for BuildingGizmo<B> {
    type Assets = B::Assets;

    fn initialize(assets: &Self::Assets) -> Self {
        Self {
            building: B::initialize(assets),
        }
    }
}

#[derive(Resource)]
pub struct PlacingBuilding {
    info: Box<dyn BuildingInfo>,
}

impl Deref for PlacingBuilding {
    type Target = Box<dyn BuildingInfo>;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

fn start_building_when_gizmo_clicked<B: Building>(
    q: Query<&Interaction, (Changed<Interaction>, With<GizmoTag<BuildingGizmo<B>>>)>,
    mut commands: Commands,
    gizmo: Res<BuildingGizmo<B>>,
) {
    for interaction in q.iter() {
        if *interaction == Interaction::Pressed {
            commands.insert_resource(PlacingBuilding {
                info: Box::new(BuildingInfoWrapper::<B>(gizmo.building.clone())),
            });

            println!("Start building {}", B::get_name());
        }
    }
}

pub trait BuildingAppExtensions {
    fn add_building<B: Building>(&mut self) -> &mut Self;
}

impl BuildingAppExtensions for App {
    fn add_building<B: Building>(&mut self) -> &mut Self {
        add_base_gizmo_systems::<BuildingGizmo<B>>(self);

        self.add_systems(
            Update,
            start_building_when_gizmo_clicked::<B>.run_if(resource_exists::<BuildingGizmo<B>>()),
        )
    }
}

pub fn is_placing_building(b: Option<Res<PlacingBuilding>>) -> bool {
    b.is_some()
}

#[derive(Component)]
struct BuildingPlaceholder;

fn place_building(
    mut commands: Commands,
    mut placeholder: Query<(Entity, &mut Transform), With<BuildingPlaceholder>>,
    info: Res<PlacingBuilding>,
    mouse_target: Res<MouseTargetedEntity>,
    floor_query: Query<&GlobalTransform, With<OpenForBuilding>>,
) {
    let placeholder_translation = if let Some(mouse_target) = &mouse_target.target {
        if let Ok(floor_transform) = floor_query.get(mouse_target.entity) {
            Some(floor_transform.translation() + Vec3::Y)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(placeholder_translation) = placeholder_translation {
        if placeholder.is_empty() {
            commands.spawn((
                BuildingPlaceholder,
                SceneBundle {
                    scene: info.get_model(),
                    transform: Transform::from_translation(placeholder_translation),
                    ..default()
                },
            ));
        } else {
            for (_, mut transform) in placeholder.iter_mut() {
                transform.translation = placeholder_translation;
            }
        }
    } else {
        for (entity, _) in placeholder.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Component)]
struct PlaceholderRenderingFixed;

fn update_placeholder_render(
    placeholder: Query<Entity, (With<BuildingPlaceholder>, Without<PlaceholderRenderingFixed>)>,
    children: Query<&Children>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for scene in placeholder.iter() {
        let color_mat = materials.add(Color::rgba(0., 1., 0., 0.5).into());
        let mut found_children = false;
        for child in children.iter_descendants(scene) {
            if let Some(mut commands) = commands.get_entity(child) {
                commands.insert(color_mat.clone());
            }
            found_children = true;
        }
        if found_children {
            commands.entity(scene).insert(PlaceholderRenderingFixed);
        }
    }
}

fn cancel_building(
    mut commands: Commands,
    control: Query<&ActionState<ControlAction>>,
    placeholder: Query<Entity, With<BuildingPlaceholder>>,
) {
    for action_state in control.iter() {
        if action_state.just_pressed(ControlAction::Deselect) {
            commands.remove_resource::<PlacingBuilding>();

            for entity in placeholder.iter() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
