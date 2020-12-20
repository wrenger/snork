use std::cmp::Reverse;
use std::collections::VecDeque;
use std::ops::{Index, IndexMut};

use super::{Cell, Grid, Snake};
use crate::env::{Direction, Vec2D};

const FLOODFILL_FREE: i8 = -1;

pub struct FloodFill {
    cells: Vec<i8>,
    queue: VecDeque<(u8, Vec2D)>,
    pub width: usize,
    pub height: usize,
}

impl FloodFill {
    pub fn new(width: usize, height: usize) -> FloodFill {
        FloodFill {
            cells: vec![FLOODFILL_FREE; width * height],
            queue: VecDeque::with_capacity(width * height),
            width,
            height,
        }
    }

    pub fn has(&self, p: Vec2D) -> bool {
        0 <= p.x && p.x < self.width as _ && 0 <= p.y && p.y < self.height as _
    }

    pub fn count_space_of(&self, snake: u8) -> usize {
        self.cells.iter().filter(|&c| *c == snake as i8).count()
    }

    pub fn flood(&mut self, grid: &Grid, heads: impl Iterator<Item = (u8, Vec2D)>) {
        assert_eq!(self.width, grid.width);
        assert_eq!(self.height, grid.height);

        for c in &mut self.cells {
            *c = FLOODFILL_FREE;
        }
        self.queue.clear();

        for (i, p) in heads {
            if grid.has(p) && grid[p] != Cell::Occupied && self[p] == FLOODFILL_FREE {
                self[p] = i as i8;
                self.queue.push_back((i, p));
            }
        }

        while let Some((i, p)) = self.queue.pop_front() {
            for dir in Direction::iter() {
                let p = p.apply(dir);
                if grid.has(p) && grid[p] != Cell::Occupied && self[p] == FLOODFILL_FREE {
                    self[p] = i as i8;
                    self.queue.push_back((i, p));
                }
            }
        }
    }

    pub fn flood_snakes(&mut self, grid: &Grid, snakes: &[Snake], you_i: u8) {
        let mut snakes: Vec<&Snake> = snakes.iter().collect();
        // Longer or equally long snakes first
        snakes.sort_by_key(|s| Reverse(2 * s.body.len() - (s.id == you_i) as usize));
        self.flood(
            grid,
            snakes
                .iter()
                .flat_map(|s| Direction::iter().map(move |d| (s.id, s.head().apply(d)))),
        );
    }
}

impl Index<Vec2D> for FloodFill {
    type Output = i8;

    fn index(&self, p: Vec2D) -> &Self::Output {
        assert!(0 <= p.x && p.x < self.width as _);
        assert!(0 <= p.y && p.y < self.height as _);
        &self.cells[(p.x as usize % self.width + p.y as usize * self.width) as usize]
    }
}

impl IndexMut<Vec2D> for FloodFill {
    fn index_mut(&mut self, p: Vec2D) -> &mut Self::Output {
        assert!(0 <= p.x && p.x < self.width as _);
        assert!(0 <= p.y && p.y < self.height as _);
        &mut self.cells[(p.x as usize % self.width + p.y as usize * self.width) as usize]
    }
}

impl std::fmt::Debug for FloodFill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Grid {{")?;
        for y in 0..self.height as i16 {
            write!(f, "  ")?;
            for x in 0..self.width as i16 {
                write!(f, "{:>2} ", self[Vec2D::new(x, self.height as i16 - y - 1)])?;
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
    fn grid_flood_fill() {
        use super::*;
        let grid = Grid::new(11, 11);

        let mut floodfill = FloodFill::new(grid.width, grid.height);
        floodfill.flood(&grid, [(0, Vec2D::new(0, 0))].iter().cloned());
        println!("Filled {:?}", floodfill);

        let grid = Grid::new(11, 11);
        floodfill.flood(
            &grid,
            [(0, Vec2D::new(0, 0)), (1, Vec2D::new(10, 10))]
                .iter()
                .cloned(),
        );
        println!("Filled {:?}", floodfill);

        let grid = Grid::new(11, 11);
        floodfill.flood(
            &grid,
            [
                (0, Vec2D::new(0, 0)),
                (1, Vec2D::new(10, 10)),
                (2, Vec2D::new(5, 5)),
            ]
            .iter()
            .cloned(),
        );
        println!("Filled {:?}", floodfill);
    }
}
