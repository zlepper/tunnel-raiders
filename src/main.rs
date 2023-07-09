mod camera_control;
mod debug_text;
mod game_level;
mod mesh_generation;
mod path_finding;
mod prelude;
mod ray_hit_helpers;
mod selection;
mod tasks;

use crate::camera_control::CameraControlPlugin;
use crate::debug_text::DebugTextPlugin;
use crate::game_level::{GameLevel, GridPosition, HALF_TILE_SIZE, TILE_SIZE};
use crate::prelude::*;
use crate::selection::SelectionPlugin;
use crate::tasks::{Minable, Miner, PlayerMovable, Standable, TasksPlugin};
use bevy::window::ExitCondition;
use bevy::DefaultPlugins;
use bevy_ecs::query::ReadOnlyWorldQuery;
use bevy_editor_pls::prelude::*;
use bevy_editor_pls::EditorWindowPlacement;
use std::f32::consts::PI;

static ENABLE_EDITOR_PLUGIN: bool = true;

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
        // .add_plugin(RapierDebugRenderPlugin {
        //     always_on_top: true,
        //     mode: DebugRenderMode::default() | DebugRenderMode::CONTACTS,
        //     ..default()
        // })
        .add_state::<GameState>()
        .add_plugin(CameraControlPlugin)
        .add_plugin(SelectionPlugin)
        .add_plugin(TasksPlugin)
        .add_plugin(DebugTextPlugin)
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

    #[asset(path = "wall.gltf#Material0")]
    pub wall_material: Handle<StandardMaterial>,

    #[asset(path = "wall.gltf#Scene1")]
    pub floor: Handle<Scene>,

    #[asset(path = "wall.gltf#Scene2")]
    pub raider: Handle<Scene>,
}

fn spawn_world(
    mut commands: Commands,
    my_assets: Res<MyAssets>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
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

    for x in -1..=11 {
        for z in -1..=11 {
            if let Some(wall_mesh) = level.create_wall_mesh(x, z) {
                let pos = level.get_position_at(GridPosition::new(x, z));
                spawn_wall(&mut commands, wall_mesh, &my_assets, pos, &mut mesh_assets);
            }

            commands.spawn((
                SceneBundle {
                    scene: my_assets.floor.clone(),
                    transform: Transform::from_translation(
                        level.get_position_at(GridPosition::new(x, z)) - Vec3::Y,
                    ),
                    ..default()
                },
                Collider::cuboid(TILE_SIZE / 2.0, 1.0, TILE_SIZE / 2.0),
                RigidBody::Fixed,
                Selectable {
                    selection_ring_offset: Vec3::Y * 3.0,
                },
                Name::new(format!("Floor {} {}", x, z)),
                PlayerInteractable,
                Standable,
            ));
        }
    }

    commands.insert_resource(level);

    commands.spawn((
        SceneBundle {
            scene: my_assets.raider.clone(),
            transform: Transform::from_xyz(15.0, 3.2, 15.0),
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
            translation: Vec3::new(0.0, 20.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
    });
}

fn spawn_wall(
    commands: &mut Commands,
    mesh: Mesh,
    my_assets: &Res<MyAssets>,
    position: Vec3,
    mesh_assets: &mut ResMut<Assets<Mesh>>,
) {
    let collider =
        Collider::from_bevy_mesh(&mesh, &default()).expect("Failed to create collider from mesh");

    commands.spawn((
        PbrBundle {
            transform: Transform::from_xyz(position.x, HALF_TILE_SIZE, position.z),
            material: my_assets.wall_material.clone(),
            mesh: mesh_assets.add(mesh),
            ..default()
        },
        collider,
        RigidBody::Fixed,
        Selectable::default(),
        Name::new(format!("Wall {} {}", position.x, position.z)),
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
