mod camera_control;
mod debug_text;
mod game_level;
mod prelude;
mod ray_hit_helpers;
mod selection;
mod errands;
mod nav_mesh_debug;
mod grid;
mod game_level_render;
mod gizmos;
mod depot_building;

use crate::camera_control::CameraControlPlugin;
use crate::debug_text::DebugTextPlugin;
use crate::game_level::{GameLevel};
use crate::prelude::*;
use crate::selection::SelectionPlugin;
use crate::errands::{Miner, PlayerMovable, ErrandsPlugin, WorkerPriorities};
use bevy::window::ExitCondition;
use bevy::DefaultPlugins;
use bevy_ecs::query::ReadOnlyWorldQuery;
use std::f32::consts::PI;
use std::time::Duration;
use bevy::asset::ChangeWatcher;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use oxidized_navigation::{NavMeshSettings, OxidizedNavigationPlugin};
use crate::game_level_render::GameLevelRenderPlugin;
use crate::gizmos::GizmosPlugin;
use crate::nav_mesh_debug::NavMeshDebugPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
                ..Default::default()
            })
            .set(WindowPlugin {
                exit_condition: ExitCondition::OnPrimaryClosed,
                ..default()
            }),
    );

    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugins(RapierDebugRenderPlugin {
        //     always_on_top: true,
        //     mode: DebugRenderMode::default() | DebugRenderMode::CONTACTS,
        //     ..default()
        // })
        .add_plugins(DebugLinesPlugin::default())
        .add_plugins(OxidizedNavigationPlugin {
            settings: NavMeshSettings {
                cell_width: 0.25,
                cell_height: 0.1,
                tile_width: 100,
                world_half_extents: 250.0,
                world_bottom_bound: -100.0,
                max_traversable_slope_radians: (1_f32).to_radians(),
                walkable_height: 20,
                walkable_radius: 5,
                step_height: 3,
                min_region_area: 100,
                merge_region_area: 500,
                max_contour_simplification_error: 1.1,
                max_edge_length: 80,
                max_tile_generation_tasks: Some(9),
            }
        } )
        .add_state::<GameState>()
        .add_state::<InteractionState>()
        .add_plugins((CameraControlPlugin, SelectionPlugin, ErrandsPlugin, DebugTextPlugin, NavMeshDebugPlugin, GameLevelRenderPlugin, GizmosPlugin))
        .add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Playing),
        )
        .add_collection_to_loading_state::<_, MyAssets>(GameState::Loading)
        .add_systems(OnEnter(GameState::Playing), spawn_world)
        .run();
}

#[derive(AssetCollection, Resource)]
pub struct MyAssets {
    #[asset(path = "wall.gltf#Mesh4/Primitive0")]
    pub full_wall_mesh: Handle<Mesh>,

    #[asset(path = "wall.gltf#Mesh5/Primitive0")]
    pub three_way_wall_mesh: Handle<Mesh>,

    #[asset(path = "wall.gltf#Mesh6/Primitive0")]
    pub outer_corner_wall_mesh: Handle<Mesh>,

    #[asset(path = "wall.gltf#Mesh7/Primitive0")]
    pub inner_corner_wall_mesh: Handle<Mesh>,

    #[asset(path = "wall.gltf#Mesh8/Primitive0")]
    pub inner_diagonal_wall_mesh: Handle<Mesh>,

    #[asset(path = "wall.gltf#Material0")]
    pub wall_material: Handle<StandardMaterial>,

    #[asset(path = "wall.gltf#Scene2")]
    pub floor: Handle<Scene>,

    #[asset(path = "wall.gltf#Scene3")]
    pub raider: Handle<Scene>,

    #[asset(path = "drill-icon.png")]
    pub mine_wall_icon: Handle<Image>
}

fn spawn_world(
    mut commands: Commands,
    my_assets: Res<MyAssets>,
) {
    let mut level = GameLevel::new(10, 10);

    for x in 1..=9 {
        level.remove_wall(x, 1);
        level.remove_wall(x, 9);
    }

    for z in 1..=9 {
        level.remove_wall(1, z);
        level.remove_wall(9, z);
    }

    commands.insert_resource(level);


    for i in 0..2 {
        commands.spawn((
            SceneBundle {
                scene: my_assets.raider.clone(),
                transform: Transform::from_xyz(15.0, 3.2, 15.0 + i as f32 * 10.),
                ..default()
            },
            Collider::cuboid(0.6, 3., 0.4),
            Name::new(format!("Raider{i}")),
            RigidBody::KinematicVelocityBased,
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            ErrandQueue::new(),
            KinematicCharacterController::default(),
            Selectable::default(),
            PlayerMovable,
            Miner,
            WorkerPriorities::default(),
        ));
    }

    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 20.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
    });
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Playing,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum InteractionState {
    #[default]
    Default,
    PlacingBuilding,
}

pub fn has_any_query_matches<F: ReadOnlyWorldQuery>(q: Query<(), F>) -> bool {
    !q.is_empty()
}

