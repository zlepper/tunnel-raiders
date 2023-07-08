use crate::camera_control::{InteractedWith};
use crate::prelude::*;
use crate::tasks::move_to_position_task::MoveToPosition;


pub struct RaiderControlPlugin;

impl Plugin for RaiderControlPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(move_selected_raider_to_target);
    }
}

#[derive(Component)]
pub struct PlayerMovable;

#[derive(Component)]
pub struct Standable;

fn move_selected_raider_to_target(
    mut movable: Query<&mut TaskQueue, (With<PlayerMovable>, With<Selected>)>,
    interactable: Query<(), With<Standable>>,
    mut events: EventReader<InteractedWith>
) {
    for event in events.iter() {
        if interactable.get(event.entity).is_ok() {
            for mut raider in movable.iter_mut() {
                info!("Adding task");
                event.add_interaction_to_queue(&mut raider, MoveToPosition::new(event.interaction.point));
            }
        }
    }
}
