mod camera_control;
mod prelude;
mod selection;

use crate::camera_control::CameraControlPlugin;
use crate::prelude::*;
use crate::selection::SelectionPlugin;
use bevy::window::ExitCondition;
use bevy::DefaultPlugins;
use bevy_editor_pls::prelude::*;
use bevy_editor_pls::EditorWindowPlacement;

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
        .add_plugin(RapierDebugRenderPlugin {
            always_on_top: true,
            mode: DebugRenderMode::default() | DebugRenderMode::CONTACTS,
            ..default()
        })
        .add_plugin(HanabiPlugin)
        .add_state::<GameState>()
        .add_plugin(CameraControlPlugin)
        .add_plugin(SelectionPlugin)
        .add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Playing),
        )
        .add_collection_to_loading_state::<_, MyAssets>(GameState::Loading)
        .add_system(spawn_world.in_schedule(OnEnter(GameState::Playing)))
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "wall.glb#Scene0")]
    pub wall: Handle<Scene>,
}

const TILE_SIZE: f32 = 10.0;

fn spawn_world(mut commands: Commands, my_assets: Res<MyAssets>) {
    for i in 0..10 {
        for j in i..10 {
            commands.spawn((
                SceneBundle {
                    scene: my_assets.wall.clone(),
                    transform: Transform::from_xyz(i as f32 * TILE_SIZE, 0.0, j as f32 * TILE_SIZE),
                    ..default()
                },
                Collider::cuboid(TILE_SIZE / 2.0, TILE_SIZE / 2.0, TILE_SIZE / 2.0),
                Selectable,
                Name::new(format!("Wall {} {}", i, j)),
            ));
        }
    }

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            range: 100.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 20.0, 0.0),
        ..default()
    });
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Playing,
}
