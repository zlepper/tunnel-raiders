use leafwing_input_manager::action_state::ActionState;
use leafwing_input_manager::user_input::InputKind;
use crate::GameState;
use crate::prelude::*;

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_camera)
            .add_system(rotate_camera)
            .add_plugin(InputManagerPlugin::<Action>::default())
            .add_system(spawn_camera.in_schedule(OnEnter(GameState::Playing)));
    }
}

fn spawn_camera(mut commands: Commands) {
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
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Move,
    Zoom,
    Rotate,
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
