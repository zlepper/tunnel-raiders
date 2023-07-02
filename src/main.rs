mod prelude;

use std::f32::consts::PI;
use bevy::DefaultPlugins;
use bevy::math::Vec3Swizzles;
use bevy::render::camera;
use bevy_editor_pls::EditorWindowPlacement;
use bevy_editor_pls::prelude::*;
use leafwing_input_manager::user_input::InputKind;
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
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Playing)
        )
        .add_collection_to_loading_state::<_, MyAssets>(GameState::Loading)
        .add_system(spawn_world.in_schedule(OnEnter(GameState::Playing)))
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_systems((move_camera, rotate_camera))
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Move,
    Zoom,
    Rotate,
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "wall.glb#Scene0")]
    pub wall: Handle<Scene>,
}

fn spawn_world(mut commands: Commands, my_assets: Res<MyAssets>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    }).insert(InputManagerBundle::<Action> {
        action_state: ActionState::default(),
        input_map: InputMap::default()
            .insert(VirtualDPad::wasd(), Action::Move)
            .insert(SingleAxis::mouse_wheel_y(), Action::Zoom)
            .insert_chord([InputKind::Mouse(MouseButton::Middle), DualAxis::mouse_motion().into()], Action::Rotate)
            .build(),
    }).insert(CameraRotationTracker {
        angle_y: 0.0,
    });

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

fn move_camera(time: Res<Time>, mut q: Query<(&mut Transform, &ActionState<Action>)>) {
    for (mut transform, action_state) in q.iter_mut() {
        let plane_movement = action_state.clamped_axis_pair(Action::Move).unwrap_or_default();

        let mut forward_direction = transform.forward();
        forward_direction.y = 0.0;
        forward_direction = forward_direction.normalize_or_zero();

        let change = forward_direction * plane_movement.y() + transform.right() * plane_movement.x();

        transform.translation += change * time.delta_seconds() * 10.0;

        let zoom = action_state.value(Action::Zoom);
        transform.translation.y = (transform.translation.y - zoom).clamp(1.0, 100.0);
    }
}


const CAMERA_PAN_RATE: f32 = 0.5;

#[derive(Debug, Component)]
struct CameraRotationTracker {
    angle_y: f32,
}

fn rotate_camera( mut q: Query<(&mut Transform, &ActionState<Action>, &mut CameraRotationTracker)>) {
    for (mut camera_transform, action_state, mut rotation_tracker) in q.iter_mut() {
        if !action_state.pressed(Action::Rotate) {
            continue;
        }
        if let Some(camera_pan_vector) = action_state.axis_pair(Action::Rotate) {

            if camera_pan_vector.x() != 0.0 {
                camera_transform.rotate_y((-camera_pan_vector.x() * CAMERA_PAN_RATE).to_radians());
            }

            if camera_pan_vector.y() != 0.0 {

                let previous_angle = rotation_tracker.angle_y;
                rotation_tracker.angle_y += camera_pan_vector.y() * CAMERA_PAN_RATE;

                rotation_tracker.angle_y = rotation_tracker.angle_y.clamp(-45.0, 45.0);


                let diff = previous_angle - rotation_tracker.angle_y;

                if diff != 0.0 {
                    let right_axis = camera_transform.right();
                    camera_transform.rotate_axis(right_axis, diff.to_radians());
                }
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Playing,
}
