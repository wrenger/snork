use std::cmp::Reverse;
use std::f64;
use std::ops::Index;
use std::ops::IndexMut;
use std::usize;

use crate::env::{Direction, Vec2D};

/// Represents a single tile of the board
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Cell {
    Free,
    Food,
    Occupied,
}

impl Default for Cell {
    fn default() -> Cell {
        Cell::Free
    }
}

impl std::fmt::Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Cell::Free => write!(f, "__"),
            Cell::Food => write!(f, "()"),
            Cell::Occupied => write!(f, "[]"),
        }
    }
}

/// The game state including up to four snakes.
#[derive(Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    cells: Vec<Cell>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Grid {
        Grid {
            width,
            height,
            cells: vec![Cell::default(); width * height],
        }
    }

    pub fn clear(&mut self) {
        for c in &mut self.cells {
            *c = Cell::Free;
        }
    }

    pub fn add_snake(&mut self, body: impl Iterator<Item = Vec2D>) {
        for p in body {
            if self.has(p) {
                self[p] = Cell::Occupied;
            }
        }
    }

    pub fn add_food(&mut self, food: &[Vec2D]) {
        for &p in food {
            if self.has(p) && self[p] != Cell::Occupied {
                self[p] = Cell::Food;
            }
        }
    }

    pub fn has(&self, p: Vec2D) -> bool {
        0 <= p.x && p.x < self.width as _ && 0 <= p.y && p.y < self.height as _
    }

    pub fn a_star(
        &self,
        start: Vec2D,
        target: Vec2D,
        first_move_heuristic: [f64; 4],
    ) -> Option<Vec<Vec2D>> {
        use priority_queue::PriorityQueue;
        use std::collections::HashMap;

        fn make_path(data: &HashMap<Vec2D, (Vec2D, f64)>, target: Vec2D) -> Vec<Vec2D> {
            let mut path = Vec::new();
            let mut p = target;
            while p.x >= 0 {
                path.push(p);
                p = data.get(&p).unwrap().0;
            }
            path.reverse();
            path
        }

        let mut queue = PriorityQueue::new();
        let mut data: HashMap<Vec2D, (Vec2D, f64)> = HashMap::new();
        data.insert(start, (Vec2D::new(-1, -1), 0.0));

        queue.push(start, Reverse(0));
        while let Some((front, _)) = queue.pop() {
            let cost = data.get(&front).unwrap().1;

            if front == target {
                return Some(make_path(&data, target));
            }

            for d in Direction::iter() {
                let neighbor = front.apply(d);
                let neighbor_cost = if front == start {
                    cost + 1.0 + first_move_heuristic[d as usize]
                } else {
                    cost + 1.0
                };

                if self.has(neighbor) && self[neighbor] != Cell::Occupied {
                    let cost_so_far = data.get(&neighbor).map(|(_, c)| *c).unwrap_or(f64::MAX);
                    if neighbor_cost < cost_so_far {
                        data.insert(neighbor, (front, neighbor_cost));
                        // queue does not accept float
                        let estimated_cost = neighbor_cost + (neighbor - start).manhattan() as f64;
                        queue.push(neighbor, Reverse((estimated_cost * 10.0) as usize));
                    }
                }
            }
        }

        None
    }
}

impl Index<Vec2D> for Grid {
    type Output = Cell;

    fn index(&self, p: Vec2D) -> &Self::Output {
        assert!(0 <= p.x && p.x < self.width as _);
        assert!(0 <= p.y && p.y < self.height as _);
        &self.cells[(p.x as usize % self.width + p.y as usize * self.width) as usize]
    }
}

impl IndexMut<Vec2D> for Grid {
    fn index_mut(&mut self, p: Vec2D) -> &mut Self::Output {
        assert!(0 <= p.x && p.x < self.width as _);
        assert!(0 <= p.y && p.y < self.height as _);
        &mut self.cells[(p.x as usize % self.width + p.y as usize * self.width) as usize]
    }
}

impl std::fmt::Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Grid {{")?;
        for y in 0..self.height as i16 {
            write!(f, "  ")?;
            for x in 0..self.width as i16 {
                write!(f, "{:?} ", self[Vec2D::new(x, self.height as i16 - y - 1)])?;
            }
            writeln!(f)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn grid_size() {
        use super::*;
        use std::mem;
        println!("Cell: {}", mem::size_of::<Cell>());
        println!("Grid: {}", mem::size_of::<Grid>());
        println!("[Cell; 11 * 11]: {}", mem::size_of::<[Cell; 11 * 11]>());
    }

    #[test]
    fn grid_snake() {
        use super::*;
        use crate::env::SnakeData;

        let mut grid = Grid::new(11, 11);
        let snake = SnakeData::new(
            100,
            vec![Vec2D::new(5, 5), Vec2D::new(5, 5), Vec2D::new(5, 5)],
        );
        grid.add_snake(snake.body.iter().cloned());
        println!("{:?}", grid);
        assert_eq!(grid[Vec2D::new(5, 5)], Cell::Occupied);

        let mut grid = Grid::new(11, 11);
        let snake = SnakeData::new(
            100,
            vec![Vec2D::new(6, 5), Vec2D::new(5, 5), Vec2D::new(5, 5)],
        );
        grid.add_snake(snake.body.iter().cloned());
        println!("{:?}", grid);
        assert_eq!(grid[Vec2D::new(6, 5)], Cell::Occupied);
        assert_eq!(grid[Vec2D::new(5, 5)], Cell::Occupied);

        let mut grid = Grid::new(11, 11);
        let snake = SnakeData::new(
            100,
            vec![Vec2D::new(6, 6), Vec2D::new(6, 5), Vec2D::new(5, 5)],
        );
        grid.add_snake(snake.body.iter().cloned());
        println!("{:?}", grid);
        assert_eq!(grid[Vec2D::new(6, 6)], Cell::Occupied);
        assert_eq!(grid[Vec2D::new(6, 5)], Cell::Occupied);
        assert_eq!(grid[Vec2D::new(5, 5)], Cell::Occupied);
    }

    #[test]
    fn grid_a_star() {
        use super::*;
        let grid = Grid::new(11, 11);

        let path = grid
            .a_star(Vec2D::new(0, 0), Vec2D::new(1, 1), [1.0, 0.0, 0.0, 0.0])
            .unwrap();
        println!("{:?}", path);
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], Vec2D::new(0, 0));
        assert_eq!(path[2], Vec2D::new(1, 1));
    }
}
