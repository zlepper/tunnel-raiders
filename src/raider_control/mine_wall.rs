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
        if let Ok(target) = minable.get(event.entity) {
            for mut raider in miner.iter_mut() {
                event.add_interaction_to_queue(
                    &mut raider,
                    MineWallTask::new(target),
                );
            }
        }
    }
}
