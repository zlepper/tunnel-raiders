use crate::game_level::{GameLevel, GridPosition};
use crate::prelude::*;
use pathfinding::directed::astar::astar;

pub fn find_path(start: Vec3, end: Vec3, level: &GameLevel, collision_radius: Option<f32>, search_radius: Option<f32>) -> Vec<Vec3> {

    let start_tile = level.get_tile_at(start);
    let end_tile = level.get_tile_at(end);

    let mut end = end;

    // Adjust the endpoint to avoid having to stand inside a wall
    let search_radius = search_radius.unwrap_or(2.);
    if let Some(collision_radius) = collision_radius {
        for adjustment in [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z].map(|v| v * collision_radius) {

            if level.get_tile_at(end + adjustment) == end_tile {
                continue;
            }

            end = end - adjustment;

        }
    }





    // If the start and end are on the same tile, just return the start and end
    // as we can path directly between them always.
    if start_tile == end_tile {
        return vec![start, end];
    }

    find_path_on_tiles(start_tile, end_tile, level)
        .into_iter()
        .map(|t| {
            if t == start_tile {
                start
            } else if t == end_tile {
                end
            } else {
                level.get_position_at(t)
            }
        })
        .map(|t| Vec3::new(t.x, start.y, t.z))
        .collect()
}

fn find_path_on_tiles(
    start_tile: GridPosition,
    end_tile: GridPosition,
    level: &GameLevel,
) -> Vec<GridPosition> {
    if start_tile == end_tile {
        return vec![end_tile];
    }

    if !level.is_open(end_tile.x, end_tile.z) || !level.is_open(start_tile.x, start_tile.z) {
        return Vec::new();
    }

    let path = astar(
        &start_tile,
        |t| {
            let mut result = Vec::with_capacity(4);
            if level.is_open(t.x + 1, t.z) {
                result.push(GridPosition { x: t.x + 1, z: t.z });
            }
            if level.is_open(t.x - 1, t.z) {
                result.push(GridPosition { x: t.x - 1, z: t.z });
            }
            if level.is_open(t.x, t.z + 1) {
                result.push(GridPosition { x: t.x, z: t.z + 1 });
            }
            if level.is_open(t.x, t.z - 1) {
                result.push(GridPosition { x: t.x, z: t.z - 1 });
            }
            result.into_iter().map(|p| (p, 1))
        },
        |t| t.distance(&end_tile) as i32,
        |t| t == &end_tile,
    );

    if let Some((path, _)) = path {
        path
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_straight_path() {
        let mut level = GameLevel::new(3, 3);
        level.remove_wall(1, 0);
        level.remove_wall(1, 1);
        level.remove_wall(1, 2);
        let path = find_path_on_tiles(GridPosition::new(1, 0), GridPosition::new(1, 2), &level);
        assert_eq!(
            path,
            vec![
                GridPosition::new(1, 0),
                GridPosition::new(1, 1),
                GridPosition::new(1, 2)
            ]
        );
    }

    #[test]
    fn finds_path_around_corner() {
        let mut level = GameLevel::new(3, 3);
        level.remove_wall(0, 0);
        level.remove_wall(0, 1);
        level.remove_wall(0, 2);
        level.remove_wall(1, 2);
        level.remove_wall(2, 2);
        let path = find_path_on_tiles(GridPosition::new(0, 0), GridPosition::new(2, 2), &level);
        assert_eq!(
            path,
            vec![
                GridPosition::new(0, 0),
                GridPosition::new(0, 1),
                GridPosition::new(0, 2),
                GridPosition::new(1, 2),
                GridPosition::new(2, 2)
            ]
        );
    }

    #[test]
    fn skips_center_travel_on_start_and_end() {
        let mut level = GameLevel::new(3, 3);
        level.remove_wall(0, 0);
        level.remove_wall(0, 1);
        level.remove_wall(0, 2);

        let path = find_path(Vec3::new(1.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 29.0), &level, None, None);
        assert_eq!(
            path,
            vec![
                Vec3::new(1.0, 0.0, 1.0),
                Vec3::new(5.0, 0.0, 15.0),
                Vec3::new(1.0, 0.0, 29.0),
            ]
        );
    }

    #[test]
    fn skips_center_travel_within_same_tile() {
        let mut level = GameLevel::new(3, 3);
        level.remove_wall(0, 0);
        level.remove_wall(0, 1);
        level.remove_wall(0, 2);

        let path = find_path(Vec3::new(1.0, 0.0, 1.0), Vec3::new(9.0, 0.0, 9.0), &level, None, None);
        assert_eq!(
            path,
            vec![Vec3::new(1.0, 0.0, 1.0), Vec3::new(9.0, 0.0, 9.0),]
        );
    }

    #[test]
    fn skips_center_travel_within_direct_neighbors() {
        let mut level = GameLevel::new(3, 3);
        level.remove_wall(0, 0);
        level.remove_wall(0, 1);
        level.remove_wall(0, 2);

        let path = find_path(Vec3::new(1.0, 0.0, 1.0), Vec3::new(9.0, 0.0, 19.0), &level, None, None);
        assert_eq!(
            path,
            vec![Vec3::new(1.0, 0.0, 1.0), Vec3::new(9.0, 0.0, 19.0),]
        );
    }
}
