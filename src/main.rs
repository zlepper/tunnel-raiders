mod camera_control;
mod prelude;
mod selection;
mod nav_mesh_debug;
mod tasks;
mod ray_hit_helpers;

use std::f32::consts::PI;
use crate::camera_control::CameraControlPlugin;
use crate::prelude::*;
use crate::selection::SelectionPlugin;
use bevy::window::ExitCondition;
use bevy::DefaultPlugins;
use bevy_ecs::query::ReadOnlyWorldQuery;
use bevy_editor_pls::prelude::*;
use bevy_editor_pls::EditorWindowPlacement;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use oxidized_navigation::{NavMeshAffector, NavMeshSettings, OxidizedNavigationPlugin};
use crate::nav_mesh_debug::NavMeshDebugPlugin;
use crate::tasks::{Minable, Miner, TasksPlugin, PlayerMovable, Standable};

static ENABLE_EDITOR_PLUGIN: bool = false;

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                watch_for_changes: true,
                ..Default::default()
            })
            .set(WindowPlugin {
                exit_condition: ExitCondition::OnPrimaryClosed,
                ..default()
            }),
    );

    if ENABLE_EDITOR_PLUGIN {
        app.add_plugin(EditorPlugin {
            window: EditorWindowPlacement::New(Window {
                title: "Editor".to_string(),
                ..Default::default()
            }),
        });
    }

    app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin {
            always_on_top: true,
            mode: DebugRenderMode::default() | DebugRenderMode::CONTACTS,
            ..default()
        })
        .add_plugin(DebugLinesPlugin::default())
        .add_plugin(OxidizedNavigationPlugin {
            settings: NavMeshSettings {
                cell_width: 0.25,
                cell_height: 0.1,
                tile_width: 100,
                world_half_extents: 250.0,
                world_bottom_bound: -100.0,
                max_traversable_slope_radians: (40.0_f32 - 0.1).to_radians(),
                walkable_height: 20,
                walkable_radius: 5,
                step_height: 3,
                min_region_area: 100,
                merge_region_area: 500,
                max_contour_simplification_error: 1.1,
                max_edge_length: 80,
                max_tile_generation_tasks: Some(9)
            }
        } )
        .add_state::<GameState>()
        .add_plugin(CameraControlPlugin)
        .add_plugin(SelectionPlugin)
        .add_plugin(TasksPlugin)
        .add_plugin(NavMeshDebugPlugin)
        .add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Playing),
        )
        .add_collection_to_loading_state::<_, MyAssets>(GameState::Loading)
        .add_system(spawn_world.in_schedule(OnEnter(GameState::Playing)))
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "wall.gltf#Scene0")]
    pub wall: Handle<Scene>,

    #[asset(path = "wall.gltf#Scene1")]
    pub floor: Handle<Scene>,

    #[asset(path = "wall.gltf#Scene2")]
    pub raider: Handle<Scene>,
}

const TILE_SIZE: f32 = 10.0;

fn spawn_world(mut commands: Commands, my_assets: Res<MyAssets>) {
    for i in -20..=20 {
        spawn_wall(&mut commands, &my_assets, i, -20);
        spawn_wall(&mut commands, &my_assets, i, 20);
    }

    for j in -20..=20 {
        spawn_wall(&mut commands, &my_assets, -20, j);
        spawn_wall(&mut commands, &my_assets, 20, j);
    }

    spawn_wall(&mut commands, &my_assets, -1, -1);

    for i in -20..=20 {
        for j in -20..=20 {
            commands.spawn((
                SceneBundle {
                    scene: my_assets.floor.clone(),
                    transform: Transform::from_xyz(i as f32 * TILE_SIZE, 0.0, j as f32 * TILE_SIZE),
                    ..default()
                },
                Collider::cuboid(TILE_SIZE / 2.0, 1.0, TILE_SIZE / 2.0),
                RigidBody::Fixed,
                Selectable {
                    selection_ring_offset: Vec3::Y * 3.0,
                },
                Name::new(format!("Floor {} {}", i, j)),
                NavMeshAffector,
                PlayerInteractable,
                Standable,
            ));
        }
    }

    commands.spawn((
        SceneBundle {
            scene: my_assets.raider.clone(),
            transform: Transform::from_xyz(0.0, 3.2, 0.0),
            ..default()
        },
        Collider::cuboid(0.6, 3., 0.4),
        Name::new(format!("Raider")),
        RigidBody::KinematicVelocityBased,
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        TaskQueue::new(),
        KinematicCharacterController::default(),
        Selectable::default(),
        PlayerMovable,
        Miner,
    ));


    // light
    commands.spawn(DirectionalLightBundle {
       directional_light: DirectionalLight {
           shadows_enabled: true,
           illuminance: 10000.0,
           ..default()
       },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
    });
}

fn spawn_wall(commands: &mut Commands, my_assets: &Res<MyAssets>, i: i32, j: i32) {
    commands.spawn((
        SceneBundle {
            scene: my_assets.wall.clone(),
            transform: Transform::from_xyz(
                i as f32 * TILE_SIZE,
                TILE_SIZE / 2.0 - 0.5,
                j as f32 * TILE_SIZE,
            ),
            ..default()
        },
        Collider::cuboid(TILE_SIZE / 2.0, TILE_SIZE / 2.0, TILE_SIZE / 2.0),
        RigidBody::Fixed,
        Selectable::default(),
        Name::new(format!("Wall {} {}", i, j)),
        NavMeshAffector,
        PlayerInteractable,
        Minable {
            max_health: 5.,
            remaining_health: 5.,
        },
    ));
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Playing,
}

pub fn has_any_query_matches<F: ReadOnlyWorldQuery>(q: Query<(), F>) -> bool {
    !q.is_empty()
}
