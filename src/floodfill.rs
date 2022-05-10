use std::collections::VecDeque;
use std::mem::size_of;
use std::ops::{Index, IndexMut};

use crate::env::{Direction, Vec2D, HAZARD_DAMAGE};
use crate::game::Snake;
use crate::grid::{CellT, Grid};
use crate::util::FixedVec;

use owo_colors::{AnsiColors, OwoColorize};

/// Floodfill Cell that stores the important data in a single Byte.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FCell {
    Free,
    Occupied {
        id: u8,
        /// Distance from the tail
        tail_dist: u16,
    },
    Owned {
        id: u8,
        health: u8,
        len: u16,
        distance: u16,
    },
}

const _: () = assert!(size_of::<FCell>() == 8);

impl std::fmt::Debug for FCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn id_color(id: u8) -> AnsiColors {
            match id {
                0 => AnsiColors::Green,
                1 => AnsiColors::Yellow,
                2 => AnsiColors::Blue,
                3 => AnsiColors::Magenta,
                _ => AnsiColors::Cyan,
            }
        }

        match self {
            FCell::Occupied { id, tail_dist } => {
                write!(f, "{:0>3}", tail_dist.color(id_color(*id)))
            }
            FCell::Owned { id, distance, .. } => {
                write!(f, "{:0>3}", distance.on_bright_black().color(id_color(*id)),)
            }
            FCell::Free => write!(f, "___"),
        }
    }
}

#[derive(Debug, Clone)]
struct SnakePos {
    p: Vec2D,
    id: u8,
    distance: u16,
    food: u16,
    len: u16,
    health: u8,
}

const _: () = assert!(size_of::<SnakePos>() == 4 + 8);

impl SnakePos {
    fn new(p: Vec2D, id: u8, distance: u16, food: u16, len: u16, health: u8) -> Self {
        Self {
            p,
            id,
            distance,
            food,
            len,
            health,
        }
    }
}

/// Grid that performs the floodfill algorithm asses area control.
///
/// This struct also contains all necessary buffers for the floodfill algorithm.
pub struct FloodFill {
    cells: Vec<FCell>,
    pub width: usize,
    pub height: usize,
}

impl FloodFill {
    #[must_use]
    pub fn new(width: usize, height: usize) -> FloodFill {
        FloodFill {
            cells: vec![FCell::Free; width * height],
            width,
            height,
        }
    }

    /// Returns if `p` is within the boundaries of the board.
    pub fn has(&self, p: Vec2D) -> bool {
        0 <= p.x && p.x < self.width as _ && 0 <= p.y && p.y < self.height as _
    }

    /// Counts the total health of you or the enemies.
    pub fn count_health(&self, i: u8) -> usize {
        self.cells
            .iter()
            .map(|&c| match c {
                FCell::Owned { id, health, .. } if id == i => health as usize,
                _ => 0,
            })
            .sum()
    }

    /// Counts the space of you or the enemies.
    pub fn count_space(&self, i: u8) -> usize {
        self.cells
            .iter()
            .filter(|&c| matches!(c, FCell::Owned { id, .. } if *id == i))
            .count()
    }

    /// Counts the space of you or the enemies weighted by the weight function.
    pub fn count_space_weighted<F: FnMut(Vec2D, FCell) -> f64>(
        &self,
        id: u8,
        mut weight: F,
    ) -> f64 {
        self.cells
            .iter()
            .copied()
            .enumerate()
            .map(|(i, c)| match c {
                FCell::Owned { id: o_id, .. } if o_id == id => weight(
                    Vec2D::new((i % self.width) as i16, (i / self.width) as i16),
                    c,
                ),
                _ => 0.0,
            })
            .sum()
    }

    /// Clears the board so that it can be reused for another floodfill computation.
    pub fn clear(&mut self) {
        for c in &mut self.cells {
            *c = FCell::Free;
        }
    }

    /// Flood fill combined with ignoring tails depending on distance to head.
    ///
    /// The board has to be cleared and prepared in beforehand.
    ///
    /// The general idea is to track the distance n from the heads and ignoring
    /// snake bodies that are vanishing after n moves.
    /// This allows the snake to follow its tail or enemy tails.
    ///
    /// Food on the way is been accounted for the own tail.
    fn flood(&mut self, grid: &Grid, heads: impl Iterator<Item = SnakePos>) -> FixedVec<u16, 4> {
        #[inline]
        fn owns(
            cell: FCell,
            s_id: u8,
            s_distance: u16,
            food: u16,
            s_len: u16,
            s_health: u8,
        ) -> bool {
            match cell {
                FCell::Free => true,
                // Follow own tail
                FCell::Occupied { id, tail_dist } if id == s_id => tail_dist + food <= s_distance,
                // Follow enemy tail
                FCell::Occupied { tail_dist, .. } => tail_dist <= s_distance, // <= enemy eats!
                // Reached in same step?
                FCell::Owned {
                    id,
                    health,
                    len,
                    distance,
                } => {
                    distance == s_distance
                        && if id != s_id {
                            // Longer snake wins (on draw we loose)
                            len < s_len || len == s_len && id < s_id
                        } else {
                            // We can reach this with more health
                            health < s_health
                        }
                }
            }
        }

        // Assuming there are at most n^2 elements in the queue
        let mut queue = VecDeque::with_capacity(self.width * self.height);
        queue.extend(heads);

        // Collect food on the way
        let mut food_distances = FixedVec::new();

        while let Some(SnakePos {
            p,
            id,
            distance,
            food,
            len,
            health,
        }) = queue.pop_front()
        {
            for p in Direction::iter()
                .map(|d| p.apply(d))
                .filter(|&p| grid.has(p))
            {
                let g_cell = grid[p];
                let cell = self[p];

                let is_food = g_cell.t == CellT::Food;

                let health = if is_food {
                    100
                } else {
                    health.saturating_sub(if g_cell.hazard { HAZARD_DAMAGE } else { 1 })
                };

                // Collect food
                if is_food && id == 0 && cell == FCell::Free {
                    food_distances.push(distance);
                }

                let food = food + is_food as u16;
                let len = len + is_food as u16;

                if health > 0 && owns(cell, id, distance, food, len, health) {
                    self[p] = FCell::Owned {
                        id,
                        health,
                        len,
                        distance,
                    };
                    queue.push_back(SnakePos::new(p, id, distance + 1, food, len, health));
                }
            }
        }
        food_distances
    }

    /// Prepare the board and compute flood fill.
    /// It is assumed that the snake at position and id 0 is the evaluated
    /// agent and the other snakes are the enemies.
    pub fn flood_snakes(&mut self, grid: &Grid, snakes: &[Snake]) -> FixedVec<u16, 4> {
        self.clear();

        // Prepare board with snakes (tail = 1, ..., head = n)
        for (id, snake) in snakes.iter().enumerate() {
            for (i, p) in snake.body.iter().enumerate() {
                self[*p] = FCell::Occupied {
                    id: id as _,
                    tail_dist: i as u16,
                }
            }
        }

        // Longer or equally long snakes first
        self.flood(
            grid,
            snakes
                .iter()
                .enumerate()
                .filter(|&(_, s)| s.alive())
                .map(|(id, s)| SnakePos::new(s.head(), id as _, 0, 0, s.body.len() as _, s.health)),
        )
    }
}

impl Index<Vec2D> for FloodFill {
    type Output = FCell;

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
        writeln!(f, "FloodFill {{")?;
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
    use log::info;

    use crate::logging;

    #[test]
    fn flood_head() {
        logging();
        use super::*;
        let grid = Grid::new(11, 11);

        let mut floodfill = FloodFill::new(grid.width, grid.height);
        floodfill.flood(
            &grid,
            [SnakePos::new(Vec2D::new(0, 0), 0, 0, 0, 3, 100)].into_iter(),
        );
        info!("Filled {} {floodfill:?}", floodfill.count_space(0));
        assert_eq!(floodfill.count_space(0), 11 * 11);

        let grid = Grid::new(11, 11);
        floodfill.clear();
        floodfill.flood(
            &grid,
            [
                SnakePos::new(Vec2D::new(0, 0), 0, 0, 0, 4, 100),
                SnakePos::new(Vec2D::new(10, 10), 1, 0, 0, 3, 100),
            ]
            .iter()
            .cloned(),
        );
        info!("Filled {} {floodfill:?}", floodfill.count_space(0));
        assert_eq!(floodfill.count_space(0), 66);
        let grid = Grid::new(11, 11);
        floodfill.clear();
        floodfill.flood(
            &grid,
            [
                SnakePos::new(Vec2D::new(5, 5), 0, 0, 0, 2, 100),
                SnakePos::new(Vec2D::new(10, 10), 1, 0, 0, 3, 100),
                SnakePos::new(Vec2D::new(0, 0), 2, 0, 0, 4, 100),
            ]
            .iter()
            .cloned(),
        );
        info!("Filled {} {floodfill:?}", floodfill.count_space(0));
        assert_eq!(floodfill.count_space(0), 61);
    }

    #[test]
    fn flood_snakes_follow_tail() {
        use super::*;
        use crate::game::Game;
        logging();

        let game = Game::parse(
            r#"
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            > > > v . . . . . . .
            ^ . . v . . . . . . .
            ^ 0 < < . . . . . . ."#,
        )
        .unwrap();

        let mut floodfill = FloodFill::new(game.grid.width, game.grid.height);
        floodfill.flood_snakes(&game.grid, &game.snakes);

        info!("Filled {} {floodfill:?}", floodfill.count_space(0));
        assert_eq!(floodfill.count_space(0), 11 * 11);

        let game = Game::parse(
            r#"
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            v . . . . . . . . . .
            v . . . . . . . . . .
            > > > v . . . . . . .
            . . . v . . . . . . .
            . 0 < < . . . . . . ."#,
        )
        .unwrap();

        let mut floodfill = FloodFill::new(game.grid.width, game.grid.height);
        floodfill.flood_snakes(&game.grid, &game.snakes);

        info!("Filled {} {floodfill:?}", floodfill.count_space(0));
        assert_eq!(floodfill.count_space(0), 11 * 11);
    }

    #[test]
    fn flood_snakes_bite_tail() {
        use super::*;
        use crate::game::Game;
        logging();

        let game = Game::parse(
            r#"
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            v . . . . . . . . . .
            v . . . . . . . . . .
            v . . . . . . . . . .
            > > > v . . . . . . .
            . . . v . . . . . . .
            . 0 < < . . . . . . ."#,
        )
        .unwrap();

        let mut floodfill = FloodFill::new(game.grid.width, game.grid.height);
        floodfill.flood_snakes(&game.grid, &game.snakes);

        info!("Filled {} {floodfill:?}", floodfill.count_space(0));
        assert_eq!(floodfill.count_space(0), 4);
    }

    #[test]
    fn flood_snakes_bite_food() {
        use super::*;
        use crate::game::Game;
        logging();

        let game = Game::parse(
            r#"
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            v . . . . . . . . . .
            > > v . . . . . . . .
            . 0 < . . . . . . . ."#,
        )
        .unwrap();

        let mut floodfill = FloodFill::new(game.grid.width, game.grid.height);
        floodfill.flood_snakes(&game.grid, &game.snakes);

        info!("Filled {} {floodfill:?}", floodfill.count_space(0));
        assert_eq!(floodfill.count_space(0), 11 * 11);

        let game = Game::parse(
            r#"
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            v . . . . . . . . . .
            > > v . . . . . . . .
            o 0 < . . . . . . . ."#,
        )
        .unwrap();

        floodfill.clear();
        floodfill.flood_snakes(&game.grid, &game.snakes);
        info!("Filled {} {floodfill:?}", floodfill.count_space(0));
        assert_eq!(floodfill.count_space(0), 1);
    }

    #[test]
    fn flood_snakes_enemy() {
        use super::*;
        use crate::game::Game;
        logging();

        let game = Game::parse(
            r#"
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            1 . . . . . . . . . .
            ^ v . . . . . . . . .
            ^ > v . . . . . . . .
            . 0 < . . . . . . . ."#,
        )
        .unwrap();

        let mut floodfill = FloodFill::new(game.grid.width, game.grid.height);
        floodfill.flood_snakes(&game.grid, &game.snakes);
        info!("Filled {} {floodfill:?}", floodfill.count_space(0));
        assert_eq!(floodfill.count_space(0), 24);
    }

    #[test]
    fn flood_snakes_low_health() {
        use super::*;
        use crate::game::Game;
        logging();

        let mut game = Game::parse(
            r#"
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . 0 . . . . .
            . . . . . ^ . . . . .
            . . . . . ^ . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . ."#,
        )
        .unwrap();
        game.snakes[0].health = 6;

        let mut floodfill = FloodFill::new(game.grid.width, game.grid.height);
        floodfill.flood_snakes(&game.grid, &game.snakes);
        info!("Filled {} {floodfill:?}", floodfill.count_space(0));
        assert_eq!(floodfill.count_space(0), 59);
    }

    #[test]
    fn flood_snakes_hazard() {
        use super::*;
        use crate::game::Game;
        logging();

        let mut game = Game::parse(
            r#"
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . 0 . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . ."#,
        )
        .unwrap();
        game.snakes[0].health = 50;
        for y in 0..game.grid.height {
            for x in game.grid.width / 2 + 1..game.grid.width {
                game.grid[Vec2D::new(x as _, y as _)].hazard = true;
            }
        }

        let mut floodfill = FloodFill::new(game.grid.width, game.grid.height);
        floodfill.flood_snakes(&game.grid, &game.snakes);
        info!("Filled {} {floodfill:?}", floodfill.count_space(0));
        assert_eq!(floodfill.count_space(0), 96);
    }

    #[test]
    fn flood_escape() {
        use super::*;
        use crate::game::Game;
        logging();

        let game = Game::parse(
            r#"
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . 1 < < < v . . . .
            o 0 < < < ^ v . . . .
            > > > > ^ ^ < . . . ."#,
        )
        .unwrap();
        info!("{:?}", game);

        let mut floodfill = FloodFill::new(game.grid.width, game.grid.height);
        floodfill.flood_snakes(&game.grid, &game.snakes);
        info!("Filled {} {floodfill:?}", floodfill.count_space(0));
        assert_eq!(29, floodfill.count_space(0));

        let game = Game::parse(
            r#"
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . . . . . . . . . . .
            . 1 < < < < . . . . .
            0 < < < < ^ v . . . .
            > > > > ^ ^ < . . . ."#,
        )
        .unwrap();
        info!("{:?}", game);

        let mut floodfill = FloodFill::new(game.grid.width, game.grid.height);
        floodfill.flood_snakes(&game.grid, &game.snakes);
        info!("Filled {} {floodfill:?}", floodfill.count_space(0));
        assert_eq!(35, floodfill.count_space(0));
    }
}
