mod camera_control;
mod prelude;
mod selection;
mod tasks;
mod move_to_position_task;

use std::f32::consts::PI;
use crate::camera_control::CameraControlPlugin;
use crate::prelude::*;
use crate::selection::SelectionPlugin;
use bevy::window::ExitCondition;
use bevy::DefaultPlugins;
use bevy_editor_pls::prelude::*;
use bevy_editor_pls::EditorWindowPlacement;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use oxidized_navigation::{NavMeshAffector, NavMeshSettings, OxidizedNavigationPlugin};
use crate::move_to_position_task::{LogMessageTask, MoveToPositionTaskPlugin, SleepTask};
use crate::tasks::TaskQueuePlugin;

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
                cell_width: 0.2,
                cell_height: 0.1,
                tile_width: 10,
                world_half_extents: 300.0,
                world_bottom_bound: -1.0,
                max_traversable_slope_radians: 45.0f32.to_radians(),
                walkable_height: 1,
                walkable_radius: 3,
                step_height: 5,
                min_region_area: 100,
                merge_region_area: 500,
                max_edge_length: 80,
                max_contour_simplification_error: 1.1,
                max_tile_generation_tasks: None,
            }
        } )
        .add_state::<GameState>()
        .add_plugin(CameraControlPlugin)
        .add_plugin(SelectionPlugin)
        .add_plugin(TaskQueuePlugin)
        .add_plugin(MoveToPositionTaskPlugin)
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
                NavMeshAffector
            ));
        }
    }

    let mut raider_task_queue = TaskQueue::new();

    raider_task_queue.add_task(LogMessageTask("Hello".to_string()));
    raider_task_queue.add_task(SleepTask(2.));
    raider_task_queue.add_task(LogMessageTask("Finished sleep 2 seconds".to_string()));
    raider_task_queue.add_task(SleepTask(5.));
    raider_task_queue.add_task(LogMessageTask("Finished sleep 5 seconds".to_string()));

    commands.spawn((
        SceneBundle {
            scene: my_assets.raider.clone(),
            transform: Transform::from_xyz(0.0, 10.0, 0.0),
            ..default()
        },
        Collider::cuboid(0.6, 2.2, 0.4),
        Name::new(format!("Raider")),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
        raider_task_queue,
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
                TILE_SIZE / 2.0,
                j as f32 * TILE_SIZE,
            ),
            ..default()
        },
        Collider::cuboid(TILE_SIZE / 2.0, TILE_SIZE / 2.0, TILE_SIZE / 2.0),
        RigidBody::Fixed,
        Selectable::default(),
        Name::new(format!("Wall {} {}", i, j)),
        NavMeshAffector,
    ));
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Playing,
}
