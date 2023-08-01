use crate::errands::{
    Designation, ErrandsV2AppExtensions, MoveToPosition, QueuedErrand, QueuedErrandFailureBuilder,
    QueuedErrandImpl, WorkingOnErrand,
};
use crate::game_level::TILE_SIZE;
use crate::gizmos::{Gizmo, GizmoAppExtension, HasBaseGizmo, ButtonGizmo, DesignationGizmo};
use crate::prelude::*;
use crate::MyAssets;

pub struct MineWallErrandPlugin;

impl Plugin for MineWallErrandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (execute_mine_wall, start_mining_wall))
            .add_errand::<MineWallErrand>()
            .add_gizmo::<MineWallGizmo>();
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


#[derive(Resource)]
pub struct MineWallGizmo(ButtonGizmo);

impl HasBaseGizmo for MineWallGizmo {
    fn get_base_gizmo(&self) -> &ButtonGizmo {
        &self.0
    }
}

impl DesignationGizmo for MineWallGizmo {
    type WorldQuery = Option<&'static Designation>;
    type ReadOnlyWorldQuery = (With<Selected>, With<Minable>);
    type Errand = MineWallErrand;

    fn create_errand(entity: Entity) -> Self::Errand {
        MineWallErrand::new(entity)
    }
}

impl Gizmo for MineWallGizmo {
    type Assets = MyAssets;

    fn is_visible(query: &Query<Self::WorldQuery, Self::ReadOnlyWorldQuery>) -> bool {
        query
            .iter()
            .any(|d| !d.is_some_and(|d| d.is_errand::<MineWallErrand>()))
    }

    fn initialize(assets: &Self::Assets) -> Self {
        MineWallGizmo(ButtonGizmo::new(assets.mine_wall_icon.clone(), "Mine", 0))
    }

}
