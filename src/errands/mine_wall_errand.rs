use bevy::utils::petgraph::visit::Walker;
use crate::errands::{Designation, ErrandsV2AppExtensions, MoveToPosition, QueuedErrand, QueuedErrandFailureBuilder, QueuedErrandImpl, WorkingOnErrand};
use crate::game_level::TILE_SIZE;
use crate::gizmos::{EntityGizmos, GizmoContainer, GizmoSelectedAction};
use crate::prelude::*;
use crate::MyAssets;

pub struct MineWallErrandPlugin;

impl Plugin for MineWallErrandPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(execute_mine_wall)
            .add_system(start_mining_wall)
            .add_system(maintain_mine_gizmo.run_if(resource_exists::<MyAssets>()))
            .add_errand::<MineWallErrand>();
    }
}

#[derive(Clone, Debug)]
pub struct MineWallErrand {
    target: Entity,
}

impl MineWallErrand {
    pub fn new(target: Entity) -> Self {
        Self { target }
    }
}

impl Errand for MineWallErrand {
    type WorkerComponent = Miner;

    fn on_enqueued<TEnqueued: QueuedErrand>(&self, queued: &mut TEnqueued) {
        queued.fail_if_entity_missing(self.target);
    }

    fn get_errand_type_order() -> i32 {
        5000
    }
}

#[derive(Component)]
pub struct Minable {
    pub remaining_health: f32,
    pub max_health: f32,
}

fn execute_mine_wall(
    mut miners: Query<(
        &mut WorkingOnErrand<MineWallErrand>,
        &GlobalTransform,
        &mut ErrandQueue,
    )>,
    mut walls: Query<(&mut Minable, &GlobalTransform)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (mut errand, miner_position, mut queue) in miners.iter_mut() {
        if let Ok((mut wall, wall_position)) = walls.get_mut(errand.target) {
            if wall_position
                .translation_vec3a()
                .distance(miner_position.translation_vec3a())
                > TILE_SIZE
            {
                queue.prepend_errand(|id| {
                    let mut e = QueuedErrandImpl::new(
                        id,
                        MoveToPosition::new(wall_position.translation(), None),
                    );
                    e.fail_if_entity_missing(errand.target);

                    e
                });
                continue;
            }

            wall.remaining_health -= time.delta_seconds();
            info!("Remaining wall health: {}", wall.remaining_health);
            if wall.remaining_health <= 0.0 {
                commands.entity(errand.target).despawn_recursive();
                errand.done();
                info!("Completed mine wall errand");
            }
        } else {
            info!("Target wall to mine no longer exists. Removing errand.");
            errand.done();
        }
    }
}

#[derive(Component)]
pub struct Miner;

fn start_mining_wall(
    mut miners: Query<&mut ErrandQueue, (With<Miner>, With<Selected>)>,
    minable: Query<(Entity, &GlobalTransform), With<Minable>>,
    mut events: EventReader<InteractedWith>,
) {
    for event in events.iter() {
        if let Ok((target, global_transform)) = minable.get(event.entity) {
            let target_position = global_transform.translation();
            info!("Starting to mine wall at {:?}", target_position);
            for mut miner in miners.iter_mut() {
                event.add_interaction_to_queue(&mut miner, MineWallErrand::new(target));
            }
        }
    }
}

fn maintain_mine_gizmo(
    mut q: Query<(Entity, &mut EntityGizmos, Option<&Designation>), (With<Selected>, With<Minable>)>,
    my_assets: Res<MyAssets>,
) {
    for (entity, mut gizmos, designation) in q.iter_mut() {

        let has_mining_designation = designation.is_some_and(|d| d.is_errand::<MineWallErrand>());

        let existing_mine_gizmo = gizmos.gizmos.iter_mut().position(|g| g.id == "mine");

        match (existing_mine_gizmo, has_mining_designation) {
            (Some(existing_index), true) => {
                gizmos.gizmos.remove(existing_index);
            },
            (Some(existing_index), false) => {
                let gizmo = &mut gizmos.gizmos[existing_index];
                match gizmo.on_selected {
                    GizmoSelectedAction::NoAction => {
                        gizmo.on_selected = GizmoSelectedAction::SetDesignation(Designation::new(entity, MineWallErrand::new(entity)));
                    }
                    GizmoSelectedAction::SetDesignation(_) => {}
                }
            },
            (None, false) => {
                gizmos.gizmos.push(GizmoContainer::new(
                    "mine",
                    "Mine",
                    0,
                    my_assets.mine_wall_icon.clone(),
                    GizmoSelectedAction::SetDesignation(Designation::new(entity, MineWallErrand::new(entity))),
                ));
            },
            _ => {}
        }
    }
}
