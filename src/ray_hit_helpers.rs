use crate::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::window::WindowRef;

pub fn get_hit(
    camera_transform: &GlobalTransform,
    camera: &Camera,
    rapier_context: &Res<RapierContext>,
    windows: &Query<&Window>,
    can_match: impl Fn(Entity) -> bool,
) -> Option<(Entity, RayIntersection)> {
    let window = if let RenderTarget::Window(window_ref) = camera.target {
        if let WindowRef::Entity(id) = window_ref {
            windows.get(id).ok()
        } else {
            windows.iter().find(|_| true)
        }
    } else {
        None
    };

    const MAX_TOI: f32 = 400.0;
    const SOLID: bool = true;

    let hit = window
        .and_then(|window| window.cursor_position())
        .and_then(|cursor_position| camera.viewport_to_world(camera_transform, cursor_position))
        .and_then(|ray_cast_source| {
            rapier_context.cast_ray_and_get_normal(
                ray_cast_source.origin,
                ray_cast_source.direction,
                MAX_TOI,
                SOLID,
                QueryFilter::new().predicate(&can_match),
            )
        });

    hit
}
