use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::mem::size_of;
use std::ops::{Index, IndexMut};
use std::{f64, usize};

use crate::env::{Direction, Vec2D, HAZARD_DAMAGE};
use crate::util::OrdPair;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CellT {
    Free,
    Food,
    Owned,
}

/// Represents a single tile of the board
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub t: CellT,
    pub hazard: bool,
}
const _: () = assert!(size_of::<Cell>() == 2);

impl Cell {
    pub const fn new(t: CellT, hazard: bool) -> Self {
        Self { t, hazard }
    }
}

impl std::fmt::Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use owo_colors::{OwoColorize, Style};

        let style = if self.hazard {
            Style::new().on_bright_black()
        } else {
            Style::new()
        };

        match self.t {
            CellT::Free => write!(f, "{}", "X".blue().style(style)),
            CellT::Food => write!(f, "{}", "o".red().style(style)),
            CellT::Owned => write!(f, "{}", ".".style(style)),
        }
    }
}

/// The board representation as grid of free and occupied cells.
///
/// This is allows fast access to specific positions on the grid and
/// if they are occupied by enemies or food.
#[derive(Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
}

const _: () = assert!(size_of::<Grid>() == 5 * 8);

impl Grid {
    /// Creates a new grid with the provided dimensions.
    pub fn new(width: usize, height: usize) -> Grid {
        Grid {
            width,
            height,
            cells: vec![Cell::new(CellT::Free, false); width * height],
        }
    }

    /// Creates a grid from a `cells` buffer.
    /// If the buffer is not dividable by `height` the buffer is truncated
    /// accordingly.
    pub fn from(mut cells: Vec<Cell>, height: usize) -> Grid {
        let width = cells.len() / height;
        cells.truncate(width * height);
        Grid {
            width,
            height,
            cells,
        }
    }

    /// Clears the grid.
    pub fn clear(&mut self) {
        for c in &mut self.cells {
            *c = Cell::new(CellT::Food, false);
        }
    }

    /// Adds the snakes as obstacles to the grid.
    pub fn add_snake(&mut self, body: impl Iterator<Item = Vec2D>) {
        for p in body {
            if self.has(p) {
                self[p].t = CellT::Owned;
            }
        }
    }

    /// Adds the provided food to the grid.
    pub fn add_food(&mut self, food: &[Vec2D]) {
        for &p in food {
            if self.has(p) {
                self[p].t = CellT::Food;
            }
        }
    }

    /// Adds the provided hazards to the grid.
    pub fn add_hazards(&mut self, hazards: &[Vec2D]) {
        for &p in hazards {
            if self.has(p) {
                self[p].hazard = true;
            }
        }
    }

    /// Returns if the cell is hazardous.
    pub fn is_hazardous(&self, p: Vec2D) -> bool {
        self.has(p) && self[p].hazard
    }

    /// Returns if `p` is within the boundaries of this grid.
    #[inline]
    pub fn has(&self, p: Vec2D) -> bool {
        p.within(self.width, self.height)
    }

    /// Performes an A* search that applies the `first_move_heuristic` as
    /// additional costs for the first move.
    pub fn a_star(
        &self,
        start: Vec2D,
        target: Vec2D,
        first_move_heuristic: &[f64; 4],
    ) -> Option<Vec<Vec2D>> {
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

        let mut queue = BinaryHeap::new();
        let mut data: HashMap<Vec2D, (Vec2D, f64)> = HashMap::new();
        data.insert(start, (Vec2D::new(-1, -1), 0.0));

        queue.push(OrdPair(Reverse(0), start));
        while let Some(OrdPair(_, front)) = queue.pop() {
            let cost = data.get(&front).unwrap().1;

            if front == target {
                return Some(make_path(&data, target));
            }

            for d in Direction::iter() {
                let neighbor = front.apply(d);
                let mut neighbor_cost = cost + 1.0;
                if self.is_hazardous(neighbor) {
                    neighbor_cost += HAZARD_DAMAGE as f64;
                }
                if front == start {
                    neighbor_cost += first_move_heuristic[d as usize]
                }

                if self.has(neighbor) && self[neighbor].t != CellT::Owned {
                    let cost_so_far = data.get(&neighbor).map(|(_, c)| *c).unwrap_or(f64::MAX);
                    if neighbor_cost < cost_so_far {
                        data.insert(neighbor, (front, neighbor_cost));
                        // queue does not accept float
                        let estimated_cost = neighbor_cost + (neighbor - start).manhattan() as f64;
                        queue.push(OrdPair(Reverse((estimated_cost * 10.0) as usize), neighbor));
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
        &self.cells[(p.x as usize + p.y as usize * self.width) as usize]
    }
}

impl IndexMut<Vec2D> for Grid {
    fn index_mut(&mut self, p: Vec2D) -> &mut Self::Output {
        assert!(0 <= p.x && p.x < self.width as _);
        assert!(0 <= p.y && p.y < self.height as _);
        &mut self.cells[(p.x as usize + p.y as usize * self.width) as usize]
    }
}

impl std::fmt::Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Grid {{")?;
        for y in (0..self.height as i16).rev() {
            write!(f, "  ")?;
            for x in 0..self.width as i16 {
                write!(f, "{:?} ", self[Vec2D::new(x, y)])?;
            }
            writeln!(f)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::logging;
    use log::info;

    #[test]
    fn grid_a_star() {
        use super::*;
        logging();
        let grid = Grid::new(11, 11);

        let path = grid
            .a_star(Vec2D::new(0, 0), Vec2D::new(1, 1), &[1.0, 0.0, 0.0, 0.0])
            .unwrap();
        info!("{:?}", path);
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], Vec2D::new(0, 0));
        assert_eq!(path[2], Vec2D::new(1, 1));
    }

    #[test]
    fn grid_a_star_hazards() {
        use super::*;
        logging();
        let mut grid = Grid::new(5, 5);
        grid.add_hazards(&[
            Vec2D::new(2, 0),
            Vec2D::new(2, 1),
            Vec2D::new(2, 2),
            Vec2D::new(2, 3),
        ]);
        let path = grid
            .a_star(Vec2D::new(0, 2), Vec2D::new(4, 2), &[1.0, 1.0, 1.0, 1.0])
            .unwrap();
        info!("{:?}", path);
        assert_eq!(path.len(), 9);
        assert_eq!(path[0], Vec2D::new(0, 2));
        assert_eq!(path[path.len() - 1], Vec2D::new(4, 2));
    }
}
