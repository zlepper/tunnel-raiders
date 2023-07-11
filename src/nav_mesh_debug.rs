use bevy_prototype_debug_lines::{DebugLines};
use oxidized_navigation::NavMesh;
use crate::prelude::*;


pub struct NavMeshDebugPlugin;

impl Plugin for NavMeshDebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(NavMeshDebugState {
                time_since_last_draw: 0.,
                draw_nav_mesh: false,
            })
            .add_system(draw_nav_mesh_system);
    }
}


#[derive(Resource)]
struct NavMeshDebugState {
    pub time_since_last_draw: f32,
    pub draw_nav_mesh: bool,
}

fn draw_nav_mesh_system(
    nav_mesh: Res<NavMesh>,
    mut lines: ResMut<DebugLines>,
    time: Res<Time>,
    mut state: ResMut<NavMeshDebugState>,
) {

    state.time_since_last_draw += time.delta_seconds();

    if state.time_since_last_draw < 2. || !state.draw_nav_mesh {
        return;
    }

    state.time_since_last_draw = 0.;

    if let Ok(nav_mesh) = nav_mesh.get().read() {
        for (tile_coord, tile) in nav_mesh.get_tiles().iter() {
            let tile_color = Color::Rgba {
                red: 0.0,
                green: (tile_coord.x % 10) as f32 / 10.0,
                blue: (tile_coord.y % 10) as f32 / 10.0,
                alpha: 1.0,
            };
            // Draw polygons.
            for poly in tile.polygons.iter() {
                let indices = &poly.indices;
                for i in 0..indices.len() {
                    let a = tile.vertices[indices[i] as usize];
                    let b = tile.vertices[indices[(i + 1) % indices.len()] as usize];
                    lines.line_colored(a, b, 2., tile_color);
                }
            }

            // Draw vertex points.
            for vertex in tile.vertices.iter() {
                lines.line_colored(*vertex, *vertex + Vec3::Y, 2., tile_color);
            }
        }
    }
}
