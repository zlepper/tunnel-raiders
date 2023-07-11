use std::fmt::Formatter;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Grid<T> {
    items: Vec<T>,
    height: i32,
    width: i32,
}

impl<T> Grid<T> {
    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn get(&self, x: i32, z: i32) -> Option<&T> {
        if !self.is_within(x, z) {
            return None;
        }

        self.items.get((x + z * self.width) as usize)
    }

    pub fn get_mut(&mut self, x: i32, z: i32) -> Option<&mut T> {
        if !self.is_within(x, z) {
            return None;
        }

        self.items.get_mut((x + z * self.width) as usize)
    }

    pub fn set(&mut self, x: i32, z: i32, value: T) {
        assert!(x < self.width);
        assert!(z < self.height);
        self.items[(x + z * self.width) as usize] = value;
    }

    pub fn map<S: Default>(&self, operate: impl Fn(&T) -> S) -> Grid<S> {
        let new_items = self.items.iter().map(operate).collect();

        Grid::new_from_list(self.width, self.height, new_items)
    }

    fn is_within(&self, x: i32, z: i32) -> bool {
        x >= 0 && x < self.width && z >= 0 && z < self.height
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
    pub fn new(width: i32, height: i32, initial_value: T) -> Self {
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

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub z: i32,
}

impl GridPosition {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

impl std::fmt::Display for GridPosition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.z)
    }
}

pub fn flood_fill_grid<T>(
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
