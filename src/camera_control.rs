use crate::prelude::*;
use crate::GameState;
use bevy::render::camera::RenderTarget;
use bevy::window::{PrimaryWindow, WindowRef};
use leafwing_input_manager::action_state::ActionState;
use leafwing_input_manager::user_input::InputKind;
use leafwing_input_manager::user_input::Modifier;
use crate::selection::{WantToSelect};

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_camera.run_if(has_window_focus))
            .add_system(rotate_camera.run_if(has_window_focus))
            .add_plugin(InputManagerPlugin::<Action>::default())
            .add_system(spawn_camera.in_schedule(OnEnter(GameState::Playing)))
            .add_system(select_things.run_if(has_window_focus));
    }
}


pub fn has_window_focus(windows: Query<&Window, With<PrimaryWindow>>) -> bool {
    let window = windows.get_single();

    if let Ok(window) = window {
        window.focused
    } else {
        false
    }
}


fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        InputManagerBundle::<Action> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(VirtualDPad::wasd(), Action::Move)
                .insert(Modifier::Shift, Action::MoveFast)
                .insert(SingleAxis::mouse_wheel_y(), Action::Zoom)
                .insert_chord(
                    [
                        InputKind::Mouse(MouseButton::Middle),
                        DualAxis::mouse_motion().into(),
                    ],
                    Action::Rotate,
                )
                .insert(MouseButton::Left, Action::Select)
                .build(),
        },
        CameraRotationTracker { angle_y: 0.0 },
        Selector,
    ));
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Move,
    Zoom,
    Rotate,
    MoveFast,
    Select,
}

const CAMERA_MOVE_RATE: f32 = 20.0;

fn move_camera(time: Res<Time>, mut q: Query<(&mut Transform, &ActionState<Action>)>) {
    for (mut transform, action_state) in q.iter_mut() {
        let plane_movement = action_state
            .clamped_axis_pair(Action::Move)
            .unwrap_or_default();

        let mut forward_direction = transform.forward();
        forward_direction.y = 0.0;
        forward_direction = forward_direction.normalize_or_zero();

        let change =
            forward_direction * plane_movement.y() + transform.right() * plane_movement.x();

        let move_rate = if action_state.pressed(Action::MoveFast) {
            CAMERA_MOVE_RATE * 10.0
        } else {
            CAMERA_MOVE_RATE
        };

        transform.translation += change * time.delta_seconds() * move_rate;

        let mut zoom = action_state.value(Action::Zoom);

        if zoom != 0.0 {
            if action_state.pressed(Action::MoveFast) {
                zoom *= 5.0;
            }

            transform.translation.y = (transform.translation.y - zoom).clamp(1.0, 100.0);
        }
    }
}

const CAMERA_PAN_RATE: f32 = 0.5;

#[derive(Debug, Component)]
struct CameraRotationTracker {
    angle_y: f32,
}

fn rotate_camera(
    mut q: Query<(
        &mut Transform,
        &ActionState<Action>,
        &mut CameraRotationTracker,
    )>,
) {
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

#[derive(Component)]
struct Selector;

fn select_things(
    mut q: Query<(&GlobalTransform, &ActionState<Action>, &Camera), With<Selector>>,
    rapier_context: Res<RapierContext>,
    windows: Query<&Window>,
    selectable: Query<(), With<Selectable>>,
    mut event_writer: EventWriter<WantToSelect>,
) {
    for (transform, action_state, camera) in q.iter_mut() {
        if action_state.just_pressed(Action::Select) {
            let window = if let RenderTarget::Window(window_ref) = camera.target {
                if let WindowRef::Entity(id) = window_ref {
                    windows.get(id).ok()
                } else {
                    windows.iter().find(|_| true)
                }
            } else {
                None
            };

            const MAX_TOI: f32 = 400.0;
            const SOLID: bool = true;

            let hit = window
                .and_then(|window| window.cursor_position())
                .and_then(|cursor_position| camera.viewport_to_world(transform, cursor_position))
                .and_then(|ray_cast_source| {
                    rapier_context.cast_ray(
                        ray_cast_source.origin,
                        ray_cast_source.direction,
                        MAX_TOI,
                        SOLID,
                        QueryFilter::new().predicate(&|e| selectable.contains(e)),
                    )
                });

            if let Some((entity, toi)) = hit {
                debug!("Hit {:?} at {}", entity, toi);

                event_writer.send(WantToSelect::Exclusively(entity));
            } else {
                debug!("No hit")
            }
        }
    }
}
