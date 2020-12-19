use std::cmp::Reverse;
use std::collections::VecDeque;
use std::ops::{Index, IndexMut};

use super::{Cell, Grid};
use crate::env::{Direction, SnakeData, Vec2D};

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
                self.queue.push_back((i, p));
            }
        }

        while let Some((i, p)) = self.queue.pop_front() {
            self[p] = i as i8;

            for dir in Direction::iter() {
                let p = p.apply(dir);
                if grid.has(p) && grid[p] != Cell::Occupied && self[p] == FLOODFILL_FREE {
                    self.queue.push_back((i, p));
                }
            }
        }
    }

    pub fn flood_snakes(&mut self, grid: &Grid, snakes: &[SnakeData], you_i: u8) {
        let mut snakes: Vec<(u8, &SnakeData)> = snakes
            .iter()
            .enumerate()
            .map(|(i, s)| (i as u8, s))
            .collect();
        // Longer or equally long snakes first
        snakes.sort_by_key(|&(i, s)| Reverse(2 * s.body.len() - (i == you_i) as usize));
        self.flood(
            grid,
            snakes
                .iter()
                .flat_map(|&(i, s)| Direction::iter().map(move |d| (i, s.body[0].apply(d)))),
        );
    }

    pub fn space_after_move(&mut self, grid: &Grid, you_i: u8, snakes: &[SnakeData]) -> [usize; 4] {
        assert_eq!(self.width, grid.width);
        assert_eq!(self.height, grid.height);

        let you = &snakes[you_i as usize];
        let snakes: Vec<(u8, &SnakeData)> = snakes
            .iter()
            .enumerate()
            .map(|(i, s)| (i as u8, s))
            .collect();

        // longer snakes are expanded in all directions
        let longer_enemies: Vec<(u8, Vec2D)> = snakes
            .iter()
            .filter(|&(i, s)| *i != you_i && s.body.len() >= you.body.len())
            .map(|(i, s)| (*i, s.body[0]))
            .flat_map(|(i, s)| Direction::iter().map(move |d| (i, s.apply(d))))
            .collect();
        let shorter_enemies: Vec<(u8, Vec2D)> = snakes
            .iter()
            .filter(|&(i, s)| *i != you_i && s.body.len() < you.body.len())
            .map(|(i, s)| (*i, s.body[0]))
            .collect();

        let mut space_after_move = [0; 4];
        for (dir_i, dir) in Direction::iter().enumerate() {
            let p = you.body[0].apply(dir);
            let mut next_grid = grid.clone();
            // free tail
            for (_, snake) in &snakes {
                if snake.body[snake.body.len() - 1] != snake.body[snake.body.len() - 2] {
                    next_grid[snake.body[snake.body.len() - 1]] = Cell::Free;
                }
            }
            // longer heads
            let mut next_heads: Vec<(u8, Vec2D)> = Vec::new();
            for &(i, p) in &longer_enemies {
                if grid.has(p) && grid[p] != Cell::Occupied {
                    next_heads.extend(Direction::iter().map(move |d| (i, p.apply(d))));
                    next_grid[p] = Cell::Occupied;
                }
            }
            if grid.has(p) && grid[p] != Cell::Occupied {
                next_heads.extend(Direction::iter().map(move |d| (you_i, p.apply(d))));
                next_grid[p] = Cell::Occupied;
                // shorter heads
                for &(i, p) in &shorter_enemies {
                    if next_grid.has(p) {
                        next_heads.extend(Direction::iter().map(move |d| (i, p.apply(d))));
                        next_grid[p] = Cell::Occupied;
                    }
                }

                self.flood(&next_grid, next_heads.iter().cloned());
                space_after_move[dir_i] = self.count_space_of(you_i);
            }
        }
        space_after_move
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

    #[test]
    fn grid_space_after_move() {
        use super::*;
        let snakes = [SnakeData::new(
            100,
            vec![Vec2D::new(0, 0), Vec2D::new(1, 0)],
        )];
        let mut grid = Grid::new(11, 11);
        for (i, snake) in snakes.iter().enumerate() {
            grid.add_snake(snake.body.iter().cloned());
        }
        let mut floodfill = FloodFill::new(grid.width, grid.height);

        let space = floodfill.space_after_move(&grid, 0, &snakes);
        println!("space after move {:?}", space);
        assert_eq!(space, [11 * 11 - 2, 0, 0, 0]);

        let snakes = [
            SnakeData::new(100, vec![Vec2D::new(0, 0), Vec2D::new(1, 0)]),
            SnakeData::new(
                100,
                vec![Vec2D::new(6, 10), Vec2D::new(5, 10), Vec2D::new(5, 10)],
            ),
        ];
        let mut grid = Grid::new(11, 11);
        for (i, snake) in snakes.iter().enumerate() {
            grid.add_snake(snake.body.iter().cloned());
        }
        let space = floodfill.space_after_move(&grid, 0, &snakes);
        println!("space after move {:?}", space);
    }
}
