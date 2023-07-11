use crate::grid::{flood_fill_grid, Grid, GridPosition};
use crate::prelude::*;

#[derive(Resource, Eq, PartialEq, Debug)]
pub struct GameLevel {
    open_tiles: Grid<bool>,
    walled_tiles: Grid<bool>,
}

pub const TILE_SIZE: f32 = 10.0;
pub const HALF_TILE_SIZE: f32 = TILE_SIZE / 2.0;

impl GameLevel {
    pub fn new(height: i32, width: i32) -> Self {
        Self {
            open_tiles: Grid::new(width, height, false),
            walled_tiles: Grid::new(width, height, true),
        }
    }

    pub fn new_from_open_tiles(grid: Grid<bool>) -> Self {
        Self {
            open_tiles: grid.clone(),
            walled_tiles: grid.map(|b| !*b),
        }
    }

    pub fn width(&self) -> i32 {
        self.open_tiles.width()
    }

    pub fn height(&self) -> i32 {
        self.open_tiles.height()
    }

    pub fn remove_wall(&mut self, x: i32, z: i32) {
        self.walled_tiles.set(x, z, false);
        self.open_tiles.set(x, z, true);
        self.expand_open_tiles(x, z);
    }

    fn expand_open_tiles(&mut self, x: i32, z: i32) {
        if !self.is_open(x, z) {
            return;
        }

        let walled_tiles = &self.walled_tiles;
        let matched = flood_fill_grid(&walled_tiles, x, z, |x, y| {
            !*walled_tiles.get(x, y).unwrap_or(&true)
        });

        for pos in matched {
            self.open_tiles.set(pos.x, pos.z, true);
        }
    }

    pub fn is_open(&self, x: i32, z: i32) -> bool {
        *self.open_tiles.get(x, z).unwrap_or(&false)
    }

    pub fn iter_tiles(&self) -> impl Iterator<Item = GridPosition> {
        let height = self.height();
        (0..self.width()).flat_map(move |x| (0..height).map(move |y| GridPosition { x, z: y }))
    }

    pub fn get_tile_at(&self, pos: Vec3) -> GridPosition {
        let x = (pos.x / TILE_SIZE).floor() as i32;
        let z = (pos.z / TILE_SIZE).floor() as i32;
        GridPosition { x, z }
    }


    pub fn get_position_at(&self, pos: GridPosition) -> Vec3 {
        let x = pos.x as f32 * TILE_SIZE + HALF_TILE_SIZE;
        let z = pos.z as f32 * TILE_SIZE + HALF_TILE_SIZE;
        Vec3::new(x, 0.0, z)
    }

    pub fn within(&self, x: i32, z: i32) -> bool {
        x >= 0 && x < self.width() && z >= 0 && z < self.height()
    }
}

#[cfg(test)]
mod tests {
    use crate::grid::GridPosition;
    use super::*;

    #[test]
    fn generates_wall_with_hole_in_middle() {
        let mut level = GameLevel::new_from_open_tiles(Grid::new(3, 3, false));
        level.remove_wall(1, 1);

        let expected = GameLevel::new_from_open_tiles(Grid::new_from_list(
            3,
            3,
            vec![
                false, false, false,
                false, true, false,
                false, false, false
            ],
        ));

        assert_eq!(level, expected);
    }

    #[test]
    fn generates_wall_with_hole_in_zero_zero() {
        let mut level = GameLevel::new_from_open_tiles(Grid::new(3, 3, false));
        level.remove_wall(0, 0);

        let expected = GameLevel::new_from_open_tiles(Grid::new_from_list(
            3,
            3,
            vec![
                true, false, false,
                false, false, false,
                false, false, false
            ],
        ));

        assert_eq!(level, expected);
    }

    #[test]
    fn generates_wall_with_hole_in_zero_max() {
        let mut level = GameLevel::new_from_open_tiles(Grid::new(3, 3, false));
        level.remove_wall(0, 2);

        let expected = GameLevel::new_from_open_tiles(Grid::new_from_list(
            3,
            3,
            vec![
                false, false, false,
                false, false, false,
                true, false, false
            ],
        ));

        assert_eq!(level, expected);
    }

    #[test]
    fn generates_wall_with_hole_in_max_max() {
        let mut level = GameLevel::new_from_open_tiles(Grid::new(3, 3, false));
        level.remove_wall(2, 2);

        let expected = GameLevel::new_from_open_tiles(Grid::new_from_list(
            3,
            3,
            vec![
                false, false, false,
                false, false, false,
                false, false, true
            ],
        ));

        assert_eq!(level, expected);
    }

    #[test]
    fn generates_wall_with_hole_in_max_zero() {
        let mut level = GameLevel::new_from_open_tiles(Grid::new(3, 3, false));
        level.remove_wall(2, 0);

        let expected = GameLevel::new_from_open_tiles(Grid::new_from_list(
            3,
            3,
            vec![
                false, false, true,
                false, false, false,
                false, false, false
            ],
        ));

        assert_eq!(level, expected);
    }

    #[test]
    fn expands_open_tiles() {
        let mut level = GameLevel {
            open_tiles: Grid {
                items: vec![false, false, false, true, false, false, false, false, false],
                height: 3,
                width: 3,
            },
            walled_tiles: Grid {
                items: vec![true, true, false, false, true, false, true, true, false],
                height: 3,
                width: 3,
            },
        };
        level.remove_wall(1, 1);

        let expected = GameLevel {
            open_tiles: Grid {
                items: vec![false, false, true, true, true, true, false, false, true],
                height: 3,
                width: 3,
            },
            walled_tiles: Grid {
                items: vec![true, true, false, false, false, false, true, true, false],
                height: 3,
                width: 3,
            },
        };

        assert_eq!(level, expected);
    }

    #[test]
    fn test_tile_positions() {
        let level = GameLevel::new(10, 10);


        assert_eq!(level.get_position_at(GridPosition::new(0, 0)), Vec3::new(5.0, 0.0, 5.0));
        assert_eq!(level.get_position_at(GridPosition::new(1, 1)), Vec3::new(15.0, 0.0, 15.0));


        assert_eq!(level.get_tile_at(Vec3::new(5.0, 0.0, 5.0)), GridPosition::new(0, 0));
        assert_eq!(level.get_tile_at(Vec3::new(1.0, 0.0, 1.0)), GridPosition::new(0, 0));
        assert_eq!(level.get_tile_at(Vec3::new(9.0, 0.0, 9.0)), GridPosition::new(0, 0));
        assert_eq!(level.get_tile_at(Vec3::new(9.0, 0.0, 1.0)), GridPosition::new(0, 0));
        assert_eq!(level.get_tile_at(Vec3::new(1.0, 0.0, 9.0)), GridPosition::new(0, 0));
        assert_eq!(level.get_tile_at(Vec3::new(11.0, 0.0, 11.0)), GridPosition::new(1, 1));
        assert_eq!(level.get_tile_at(Vec3::new(19.0, 0.0, 19.0)), GridPosition::new(1, 1));
    }
}
