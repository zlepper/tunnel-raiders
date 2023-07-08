use crate::prelude::*;
use crate::tasks::Task;

pub struct MineWallTaskPlugin;

impl Plugin for MineWallTaskPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(execute_mine_wall).add_system(start_mining_wall);
    }
}

#[derive(Component)]
pub struct MineWallTask {
    target: Entity,
}

impl MineWallTask {
    pub fn new(target: Entity) -> Self {
        Self { target }
    }
}

impl Task for MineWallTask {
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
    miners: Query<(Entity, &MineWallTask)>,
    mut walls: Query<&mut Minable>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, task) in miners.iter() {
        if let Ok(mut wall) = walls.get_mut(task.target) {
            wall.remaining_health -= time.delta_seconds();
            info!("Remaining wall health: {}", wall.remaining_health);
            if wall.remaining_health <= 0.0 {
                commands.entity(task.target).despawn_recursive();
                commands.entity(entity).remove::<MineWallTask>();
            }
        }
    }
}


#[derive(Component)]
pub struct Miner;

fn start_mining_wall(
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
