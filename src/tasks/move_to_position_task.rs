use bevy::math::Vec3Swizzles;
use crate::has_any_query_matches;
use crate::prelude::*;
use crate::tasks::Task;
use oxidized_navigation::query::find_path;
use oxidized_navigation::{NavMesh, NavMeshSettings};

#[derive(Component)]
pub struct MoveToPosition {
    target: Vec3,
    path: Option<PathTracker>,
}

impl MoveToPosition {
    pub fn new(target: Vec3) -> Self {
        Self { target, path: None }
    }
}

impl Task for MoveToPosition {
    fn name(&self) -> String {
        format!("Move to position {}, {}", self.target.x, self.target.z)
    }
}

struct PathTracker {
    path: Vec<Vec3>,
    next: usize,
}

impl PathTracker {
    fn new(path: Vec<Vec3>) -> Self {
        Self { path, next: 0 }
    }

    fn next(&self) -> Option<Vec3> {
        self.path.get(self.next).copied()
    }

    fn advance(&mut self) {
        self.next += 1;
    }
}

fn execute_move_to_position(
    mut query: Query<(
        Entity,
        &mut MoveToPosition,
        &mut KinematicCharacterController,
        &GlobalTransform,
        &mut Transform,
    )>,
    mut commands: Commands,
    nav_mesh_settings: Res<NavMeshSettings>,
    nav_mesh: Res<NavMesh>,
    time: Res<Time>,
) {
    if let Ok(nav_mesh) = nav_mesh.get().read() {
        for (entity, mut pos, mut controller, global_position, mut transform) in query.iter_mut() {
            if pos.path.is_none() {
                let start_pos = global_position.translation();
                let path = find_path(
                    &nav_mesh,
                    &*nav_mesh_settings,
                    start_pos,
                    pos.target,
                    None,
                    None,
                );
                match path {
                    Ok(p) => {
                        info!("Calculated path: {:?}", p);

                        let initial_target = p.first().expect("Got empty path finding path");

                        let path_offset = start_pos - *initial_target;

                        let path = p.iter().map(|p| *p + path_offset).collect();

                        pos.path = Some(PathTracker::new(path));
                    }
                    Err(e) => {
                        warn!(
                            "Failed to find path from {:?} to {:?}. Skipping task. Error: {:?}",
                            start_pos, pos.target, e
                        );
                        // commands.entity(entity).remove::<MoveToPosition>();
                        continue;
                    }
                }
            }

            if let Some(ref mut path) = &mut pos.path {
                if let Some(next) = path.next() {
                    let direction = next - global_position.translation();
                    let distance = direction.length();
                    if distance < 1.0 {
                        path.advance();
                    }
                    let direction = direction.normalize();
                    let speed = 5.0;
                    let velocity = direction * speed * time.delta_seconds();
                    controller.translation = Some(velocity);
                } else {

                    let total_remaining_distance =
                        (pos.target.xz()  - global_position.translation().xz()).length();
                    if total_remaining_distance > 1. {
                        warn!(
                            "Still some pathing left to go apparently: {}",
                            total_remaining_distance
                        );
                        pos.path = None;
                    } else {
                        info!("Completed MoveToPosition task for entity {:?}", entity);
                        transform.look_at(pos.target, Vec3::Y);
                        commands.entity(entity).remove::<MoveToPosition>();
                    }
                }
            }
        }
    }
}

fn warn_about_invalid_move_to_position_target(
    query: Query<Entity, (With<MoveToPosition>, Without<KinematicCharacterController>)>,
    mut commands: Commands,
) {
    for entity in query.iter() {
        warn!("Entity {:?} has a MoveToPosition task, but no KinematicCharacterController. Removing task.", entity);
        commands.entity(entity).remove::<MoveToPosition>();
    }
}

pub struct MoveToPositionTaskPlugin;

impl Plugin for MoveToPositionTaskPlugin {
    fn build(&self, app: &mut App) {
        app.register_task::<MoveToPosition>()
            .add_system(execute_move_to_position)
            .add_system(warn_about_invalid_move_to_position_target.run_if(
                has_any_query_matches::<(
                    With<MoveToPosition>,
                    Without<KinematicCharacterController>,
                )>,
            ));
    }
}
