use crate::errands::errand_queue::ErrandQueue;
use crate::errands::{Errand, MoveToPosition};
use crate::gizmos::{EntityGizmos, GizmoInfo, GizmoType};
use crate::prelude::*;
use crate::MyAssets;

pub struct MineWallErrandPlugin;

impl Plugin for MineWallErrandPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(execute_mine_wall)
            .add_system(start_mining_wall)
            .add_system(maintain_mine_gizmo.run_if(resource_exists::<MyAssets>()))
            .register_errand::<MineWallErrand>();
    }
}

#[derive(Component)]
pub struct MineWallErrand {
    target: Entity,
}

impl MineWallErrand {
    pub fn new(target: Entity) -> Self {
        Self { target }
    }
}

impl Errand for MineWallErrand {
    fn name(&self) -> String {
        format!("Mine wall: {:?}", self.target)
    }
}

#[derive(Component)]
pub struct Minable {
    pub remaining_health: f32,
    pub max_health: f32,
}

fn execute_mine_wall(
    miners: Query<(Entity, &MineWallErrand)>,
    mut walls: Query<&mut Minable>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, errand) in miners.iter() {
        if let Ok(mut wall) = walls.get_mut(errand.target) {
            wall.remaining_health -= time.delta_seconds();
            info!("Remaining wall health: {}", wall.remaining_health);
            if wall.remaining_health <= 0.0 {
                commands.entity(errand.target).despawn_recursive();
                commands.entity(entity).remove::<MineWallErrand>();
                info!("Completed mine wall errand");
            }
        } else {
            info!("Target wall to mine no longer exists. Removing errand.");
            commands.entity(entity).remove::<MineWallErrand>();
        }
    }
}

#[derive(Component)]
pub struct Miner;

fn start_mining_wall(
    mut miner: Query<&mut ErrandQueue, (With<Miner>, With<Selected>)>,
    minable: Query<(Entity, &GlobalTransform), With<Minable>>,
    mut events: EventReader<InteractedWith>,
) {
    for event in events.iter() {
        if let Ok((target, global_transform)) = minable.get(event.entity) {
            let target_position = global_transform.translation();
            info!("Starting to mine wall at {:?}", target_position);
            for mut raider in miner.iter_mut() {
                event.add_interaction_to_queue(
                    &mut raider,
                    (
                        MoveToPosition::new(target_position, Some(5.)),
                        MineWallErrand::new(target),
                    ),
                );
            }
        }
    }
}

fn maintain_mine_gizmo(
    mut q: Query<(&mut EntityGizmos), (With<Selected>, With<Minable>)>,
    my_assets: Res<MyAssets>,
) {
    for mut gizmos in q.iter_mut() {
        if gizmos.gizmos.len() == 0 {
            gizmos.gizmos.push(GizmoType::AddGlobalTask {
                gizmo: GizmoInfo {
                    id: "mine".to_string(),
                    order: 0,
                    name: "Mine".to_string(),
                    icon: my_assets.mine_wall_icon.clone(),
                },
            })
        }
    }
}
