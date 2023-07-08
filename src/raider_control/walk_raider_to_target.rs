use crate::camera_control::InteractedWith;
use crate::prelude::*;
use crate::tasks::MoveToPosition;

#[derive(Component)]
pub struct PlayerMovable;

#[derive(Component)]
pub struct Standable;

pub fn move_selected_raider_to_target(
    mut movable: Query<&mut TaskQueue, (With<PlayerMovable>, With<Selected>)>,
    interactable: Query<(), With<Standable>>,
    mut events: EventReader<InteractedWith>,
) {
    for event in events.iter() {
        if interactable.get(event.entity).is_ok() {
            for mut raider in movable.iter_mut() {
                event.add_interaction_to_queue(
                    &mut raider,
                    MoveToPosition::new(event.interaction.point),
                );
            }
        }
    }
}
