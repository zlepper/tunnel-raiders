use crate::prelude::*;
use crate::tasks::{Minable, MineWallTask};


#[derive(Component)]
pub struct Miner;

pub fn start_mining_wall(
    mut miner: Query<&mut TaskQueue, (With<Miner>, With<Selected>)>,
    minable: Query<Entity, With<Minable>>,
    mut events: EventReader<InteractedWith>,
) {
    for event in events.iter() {
        info!("Checking interact event");
        if let Ok(target) = minable.get(event.entity) {
            info!("Found minable entity");
            for mut raider in miner.iter_mut() {
                info!("Queueing mine wall task");
                event.add_interaction_to_queue(
                    &mut raider,
                    MineWallTask::new(target),
                );
            }
        } else {
            info!("No minable entity found");
        }
    }
}
