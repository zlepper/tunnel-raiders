use crate::mesh_generation::{generate_wall_mesh, WallGenerationArgs};
use crate::prelude::*;

#[derive(Resource, Eq, PartialEq, Debug)]
pub struct GameLevel {
    open_tiles: Grid<bool>,
    walled_tiles: Grid<bool>,
}

pub const TILE_SIZE: f32 = 10.0;

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

    pub fn create_wall_mesh(&self, x: i32, z: i32) -> Option<Mesh> {
        // Can't make a mesh if the wall is not there.
        if self.is_open(x, z) {
            return None;
        }

        let mesh = generate_wall_mesh(WallGenerationArgs {
            has_south_wall: !self.is_open(x, z - 1),
            has_north_wall: !self.is_open(x, z + 1),
            has_west_wall: !self.is_open(x - 1, z),
            has_east_wall: !self.is_open(x + 1, z),
            width: TILE_SIZE,
            height: TILE_SIZE,
        });

        Some(mesh)
    }

    fn is_open(&self, x: i32, z: i32) -> bool {
        *self.open_tiles.get(x, z).unwrap_or(&false)
    }

    pub fn iter_tiles(&self) -> impl Iterator<Item = GridPosition> {
        let height = self.height();
        (0..self.width()).flat_map(move |x| (0..height).map(move |y| GridPosition { x, z: y }))
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Grid<T> {
    items: Vec<T>,
    height: i32,
    width: i32,
}

impl<T> Grid<T> {
    fn width(&self) -> i32 {
        self.width
    }

    fn height(&self) -> i32 {
        self.height
    }

    fn get(&self, x: i32, z: i32) -> Option<&T> {
        if x < 0 || z < 0 {
            return None;
        }

        if x >= self.width || z >= self.height {
            return None;
        }

        self.items.get((x + z * self.width) as usize)
    }

    fn get_mut(&mut self, x: i32, z: i32) -> Option<&mut T> {
        self.items.get_mut((x + z * self.width) as usize)
    }

    fn set(&mut self, x: i32, z: i32, value: T) {
        assert!(x < self.width);
        assert!(z < self.height);
        self.items[(x + z * self.width) as usize] = value;
    }

    fn map<S: Default>(&self, operate: impl Fn(&T) -> S) -> Grid<S> {
        let new_items = self.items.iter().map(operate).collect();

        Grid::new_from_list(self.width, self.height, new_items)
    }

    fn new_from_list(width: i32, height: i32, items: Vec<T>) -> Self {
        assert_eq!(width * height, items.len() as i32);
        Self {
            items,
            height,
            width,
        }
    }
}

impl<T> Grid<T>
where
    T: Clone,
{
    fn new(width: i32, height: i32, initial_value: T) -> Self {
        let count = width * height;
        let mut items = Vec::with_capacity(count as usize);
        for _ in 0..count {
            items.push(initial_value.clone());
        }
        Self {
            items,
            height,
            width,
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct GridPosition {
    pub x: i32,
    pub z: i32,
}

fn flood_fill_grid<T>(
    grid: &Grid<T>,
    start_x: i32,
    start_z: i32,
    expand_to: impl Fn(i32, i32) -> bool,
) -> Vec<GridPosition> {
    if start_x > grid.width || start_z > grid.height {
        return Vec::new();
    }

    let mut visited = Grid::new(grid.width(), grid.height(), false);

    let mut queue = Vec::new();

    queue.push((start_x, start_z));
    let mut result = Vec::new();
    result.push(GridPosition {
        x: start_x,
        z: start_z,
    });

    while let Some((x, z)) = queue.pop() {
        if *visited.get(x, z).unwrap_or(&true) {
            continue;
        }
        visited.set(x, z, true);
        if expand_to(x, z) {
            result.push(GridPosition { x, z });
            if x + 1 < grid.width() {
                queue.push((x + 1, z));
            }
            if x > 0 {
                queue.push((x - 1, z));
            }
            if z + 1 < grid.height() {
                queue.push((x, z + 1));
            }
            if z > 0 {
                queue.push((x, z - 1));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
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
}
