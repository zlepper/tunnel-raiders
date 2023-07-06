use crate::camera_control::Selector;
use crate::prelude::*;
use crate::ray_hit_helpers::get_hit;
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
pub struct PlayerInteractable;

#[derive(Component)]
pub struct Standable;

fn move_selected_raider_to_target(
    mut q: Query<(&GlobalTransform, &ActionState<ControlAction>, &Camera), With<Selector>>,
    rapier_context: Res<RapierContext>,
    windows: Query<&Window>,
    mut movable: Query<&mut TaskQueue, (With<PlayerMovable>, With<Selected>)>,
    interactable: Query<Option<&Standable>, With<PlayerInteractable>>,
) {
    for (transform, action_state, camera) in q.iter_mut() {
        if action_state.just_pressed(ControlAction::Interact) {

            let hit = get_hit(&transform, &camera, &rapier_context, &windows, |e| interactable.contains(e));

            if let Some((entity, ray_intersection)) = hit {
                info!("Hit {:?} at {:?}", entity, ray_intersection);

                if let Ok(interaction_target) = interactable.get(entity) {
                    if interaction_target.is_none() {
                        info!("Entity {:?} is not standable", entity);
                        continue;
                    }

                    for mut raider in movable.iter_mut() {
                        info!("Adding task");
                        raider.override_task(MoveToPosition::new(ray_intersection.point));
                    }
                }


            } else {
                info!("No hit")
            }
        }
    }
}
