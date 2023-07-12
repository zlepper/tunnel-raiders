use oxidized_navigation::{NavMesh, NavMeshSettings, query::find_path};
use crate::game_level::GameLevel;
use crate::has_any_query_matches;
use crate::prelude::*;
use crate::errands::Errand;
use crate::errands::errand_queue::ErrandQueue;

#[derive(Component)]
pub struct MoveToPosition {
    target: Vec3,
    path: Option<PathTracker>,
    search_radius: Option<f32>,
}

impl MoveToPosition {
    pub fn new(target: Vec3, search_radius: Option<f32>) -> Self {
        Self {
            target,
            path: None,
            search_radius,
        }
    }
}

impl Errand for MoveToPosition {
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
    if let Ok(nav_mesh) = nav_mesh.get().try_read() {
        for (entity, mut pos, mut controller, global_position, mut transform) in query.iter_mut() {
            if pos.path.is_none() {
                let start_pos = global_position.translation();
                let path = find_path(
                    &nav_mesh,
                    &*nav_mesh_settings,
                    start_pos,
                    pos.target,
                    pos.search_radius,
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
                            "Failed to find path from {:?} to {:?}. Skipping errand. Error: {:?}",
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
                    if direction.is_nan() {
                        continue;
                    }
                    let distance = direction.length_squared();
                    if distance < 0.01 {
                        path.advance();
                        continue;
                    }
                    let movement = direction.normalize();
                    transform.look_to(movement * (Vec3::X + Vec3::Z), Vec3::Y);
                    let speed = 5.0;
                    let velocity = movement * speed * time.delta_seconds();
                    controller.translation = Some(velocity);
                } else {
                    info!("Completed MoveToPosition errand for entity {:?}", entity);
                    commands.entity(entity).remove::<MoveToPosition>();
                }
            }
        }
    }
}

fn warn_about_invalid_move_to_position_target(
    query: Query<
        (Entity, Option<&Name>),
        (With<MoveToPosition>, Without<KinematicCharacterController>),
    >,
    mut commands: Commands,
) {
    for (entity, name) in query.iter() {
        let display_name = name
            .map(|n| n.to_string())
            .unwrap_or_else(|| format!("{:?}", entity));
        warn!("Entity {:?} has a MoveToPosition errand, but no KinematicCharacterController. Removing errand.", display_name);
        commands.entity(entity).remove::<MoveToPosition>();
    }
}

pub struct MoveToPositionErrandPlugin;

impl Plugin for MoveToPositionErrandPlugin {
    fn build(&self, app: &mut App) {
        app.register_errand::<MoveToPosition>()
            .add_system(execute_move_to_position.run_if(resource_exists::<GameLevel>()))
            .add_system(warn_about_invalid_move_to_position_target.run_if(
                has_any_query_matches::<(
                    With<MoveToPosition>,
                    Without<KinematicCharacterController>,
                )>,
            ))
            .add_system(move_selected_raider_to_target);
    }
}

#[derive(Component)]
pub struct PlayerMovable;

#[derive(Component)]
pub struct Standable;

fn move_selected_raider_to_target(
    mut movable: Query<&mut ErrandQueue, (With<PlayerMovable>, With<Selected>)>,
    interactable: Query<(), With<Standable>>,
    mut events: EventReader<InteractedWith>,
) {
    for event in events.iter() {
        if interactable.get(event.entity).is_ok() {
            for mut raider in movable.iter_mut() {
                event.add_interaction_to_queue(
                    &mut raider,
                    MoveToPosition::new(event.interaction.point, None),
                );
            }
        }
    }
}
