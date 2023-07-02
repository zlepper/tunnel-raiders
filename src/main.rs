mod prelude;
mod camera_control;

use std::f32::consts::PI;
use bevy::DefaultPlugins;
use bevy::math::Vec3Swizzles;
use bevy::render::camera;
use bevy_editor_pls::EditorWindowPlacement;
use bevy_editor_pls::prelude::*;
use leafwing_input_manager::user_input::InputKind;
use crate::camera_control::CameraControlPlugin;
use crate::prelude::*;

static ENABLE_EDITOR_PLUGIN: bool = false;


fn main() {
    let mut app = App::new();

    if ENABLE_EDITOR_PLUGIN {
        app.add_plugin(EditorPlugin {
            window: EditorWindowPlacement::New(Window {
                title: "Editor".to_string(),
                ..Default::default()
            }),
        });
    }

    app.add_plugins(DefaultPlugins.set(AssetPlugin {
        watch_for_changes: true,
        ..Default::default()
    }))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin {
            always_on_top: true,
            mode: DebugRenderMode::default() | DebugRenderMode::CONTACTS,
            ..default()
        })
        .add_state::<GameState>()
        .add_plugin(CameraControlPlugin)
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Playing)
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

fn spawn_world(mut commands: Commands, my_assets: Res<MyAssets>) {
    commands.spawn(SceneBundle {
        scene: my_assets.wall.clone(),
        ..default()
    });


    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-20.0, 20.0, -20.0),
        ..default()
    });
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Playing,
}
