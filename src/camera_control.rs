use crate::prelude::*;
use crate::ray_hit_helpers::get_hit;
use crate::selection::WantToSelect;
use crate::GameState;
use bevy::window::PrimaryWindow;
use leafwing_input_manager::action_state::ActionState;
use leafwing_input_manager::user_input::InputKind;
use leafwing_input_manager::user_input::Modifier;

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_camera.run_if(has_window_focus))
            .add_system(rotate_camera.run_if(has_window_focus))
            .add_plugin(InputManagerPlugin::<ControlAction>::default())
            .add_system(spawn_camera.in_schedule(OnEnter(GameState::Playing)))
            .insert_resource(MouseTargetedEntity { target: None })
            .add_system(mouse_over_things.run_if(has_window_focus))
            .add_system(
                (interact_with_things)
                    .run_if(has_window_focus)
                    .after(mouse_over_things),
            )
            .add_system(
                (select_things)
                    .run_if(has_window_focus)
                    .after(mouse_over_things),
            )
            .add_event::<InteractedWith>();
    }
}

pub fn has_window_focus(windows: Query<&Window, With<PrimaryWindow>>) -> bool {
    let window = windows.get_single();

    if let Ok(window) = window {
        window.focused && window.physical_cursor_position().is_some()
    } else {
        false
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(15.0, 20.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        InputManagerBundle::<ControlAction> {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(VirtualDPad::wasd(), ControlAction::Move)
                .insert(Modifier::Shift, ControlAction::MoveFast)
                .insert(SingleAxis::mouse_wheel_y(), ControlAction::Zoom)
                .insert_chord(
                    [
                        InputKind::Mouse(MouseButton::Middle),
                        DualAxis::mouse_motion().into(),
                    ],
                    ControlAction::Rotate,
                )
                .insert(MouseButton::Left, ControlAction::Select)
                .insert_modified(
                    Modifier::Shift,
                    MouseButton::Left,
                    ControlAction::SelectAdditional,
                )
                .insert(MouseButton::Right, ControlAction::Interact)
                .insert_modified(
                    Modifier::Shift,
                    MouseButton::Right,
                    ControlAction::InteractAdditional,
                )
                .build(),
        },
        CameraRotationTracker { angle_y: 0.0 },
        Selector,
    ));
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum ControlAction {
    Move,
    Zoom,
    Rotate,
    MoveFast,
    Select,
    SelectAdditional,
    Interact,
    InteractAdditional,
}

const CAMERA_MOVE_RATE: f32 = 20.0;

fn move_camera(time: Res<Time>, mut q: Query<(&mut Transform, &ActionState<ControlAction>)>) {
    for (mut transform, action_state) in q.iter_mut() {
        let plane_movement = action_state
            .clamped_axis_pair(ControlAction::Move)
            .unwrap_or_default();

        let mut forward_direction = transform.forward();
        forward_direction.y = 0.0;
        forward_direction = forward_direction.normalize_or_zero();

        let change =
            forward_direction * plane_movement.y() + transform.right() * plane_movement.x();

        let move_rate = if action_state.pressed(ControlAction::MoveFast) {
            CAMERA_MOVE_RATE * 10.0
        } else {
            CAMERA_MOVE_RATE
        };

        transform.translation += change * time.delta_seconds() * move_rate;

        let mut zoom = action_state.value(ControlAction::Zoom);

        if zoom != 0.0 {
            if action_state.pressed(ControlAction::MoveFast) {
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
        &ActionState<ControlAction>,
        &mut CameraRotationTracker,
    )>,
) {
    for (mut camera_transform, action_state, mut rotation_tracker) in q.iter_mut() {
        if !action_state.pressed(ControlAction::Rotate) {
            continue;
        }
        if let Some(camera_pan_vector) = action_state.axis_pair(ControlAction::Rotate) {
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
pub struct Selector;

fn select_things(
    q: Query<&ActionState<ControlAction>>,
    selectable: Query<(), With<Selectable>>,
    mut events: EventWriter<WantToSelect>,
    mouse_target: Res<MouseTargetedEntity>,
) {
    for action_state in q.iter() {
        if action_state.just_pressed(ControlAction::Select)
            || action_state.just_pressed(ControlAction::SelectAdditional)
        {
            info!("Selecting");
            if let Some(target) = &mouse_target.target {
                if selectable.get(target.entity).is_ok() {
                    info!("Hit {:?} at {:?}", target.entity, target.intersection);

                    let append = action_state.just_pressed(ControlAction::SelectAdditional);

                    if append {
                        events.send(WantToSelect::Additionally(target.entity));
                    } else {
                        events.send(WantToSelect::Exclusively(target.entity))
                    }
                }
            } else {
                info!("No hit")
            }
        }
    }
}

#[derive(Component)]
pub struct PlayerInteractable;

pub struct InteractedWith {
    pub entity: Entity,
    pub interaction: RayIntersection,
    pub append: bool,
}

impl InteractedWith {
    fn new(entity: Entity, interaction: RayIntersection, append: bool) -> Self {
        Self {
            entity,
            interaction,
            append,
        }
    }

    pub fn add_interaction_to_queue(&self, queue: &mut ErrandQueue, errand: impl Errand) {
        if !self.append {
            queue.clear();
        }

        queue.append_independent_errand(errand);
    }
}


fn interact_with_things(
    q: Query<&ActionState<ControlAction>>,
    interactable: Query<(), With<PlayerInteractable>>,
    mut events: EventWriter<InteractedWith>,
    mouse_target: Res<MouseTargetedEntity>,
) {
    for action_state in q.iter() {
        if action_state.just_pressed(ControlAction::Interact)
            || action_state.just_pressed(ControlAction::InteractAdditional)
        {
            if let Some(target) = &mouse_target.target {
                if interactable.get(target.entity).is_ok() {
                    info!("Hit {:?} at {:?}", target.entity, target.intersection);

                    let append = action_state.just_pressed(ControlAction::InteractAdditional);
                    events.send(InteractedWith::new(
                        target.entity,
                        target.intersection,
                        append,
                    ));
                }
            } else {
                info!("No hit")
            }
        }
    }
}

#[derive(Resource)]
pub struct MouseTargetedEntity {
    pub target: Option<MouseTargetedEntityTarget>,
}

pub struct MouseTargetedEntityTarget {
    pub entity: Entity,
    pub intersection: RayIntersection,
}

fn mouse_over_things(
    q: Query<(&GlobalTransform, &Camera), With<Selector>>,
    rapier_context: Res<RapierContext>,
    windows: Query<&Window>,
    targetable: Query<(), Or<(With<PlayerInteractable>, With<Selectable>)>>,
    mut mouse_target: ResMut<MouseTargetedEntity>,
    ui_buttons: Query<&Interaction>,
) {
    if ui_buttons.iter().any(|i| *i != Interaction::None) {
        mouse_target.target = None;
        return;
    }


    for (transform, camera) in q.iter() {
        let hit = get_hit(transform, camera, &rapier_context, &windows, |e| {
            targetable.contains(e)
        });

        if let Some((entity, ray_intersection)) = hit {
            mouse_target.target = Some(MouseTargetedEntityTarget {
                entity,
                intersection: ray_intersection,
            });
        } else {
            mouse_target.target = None;
        }
    }
}
