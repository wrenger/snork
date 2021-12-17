use std::cmp::Reverse;
use std::collections::VecDeque;
use std::ops::{Index, IndexMut};

use super::{Grid, Snake};
use crate::env::{Direction, Vec2D, HAZARD_DAMAGE};

use owo_colors::{AnsiColors, OwoColorize};

/// Floodfill Cell that stores the important data in a single Byte.
///
/// Bitfield:
/// - 15: you?
/// - 14: owned?
/// - 13: occupied?
/// - 11..0: num (head distance if occupied, health if owned)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FCell {
    Free,
    /// id, tail-dist
    Occupied(u8, u16),
    /// id, health, len, distance
    Owned(u8, u8, u16, u16),
}

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
            FCell::Occupied(id, tail_dist) => {
                write!(f, "{:0>3}", tail_dist.color(id_color(*id)))
            }
            FCell::Owned(id, _health, _len, distance) => {
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
    queue: VecDeque<SnakePos>,
    pub width: usize,
    pub height: usize,
}

impl FloodFill {
    pub fn new(width: usize, height: usize) -> FloodFill {
        FloodFill {
            cells: vec![FCell::Free; width * height],
            queue: VecDeque::with_capacity(width * height),
            width,
            height,
        }
    }

    /// Returns if `p` is within the boundaries of the board.
    pub fn has(&self, p: Vec2D) -> bool {
        0 <= p.x && p.x < self.width as _ && 0 <= p.y && p.y < self.height as _
    }

    /// Counts the total health of you or the enemies.
    pub fn count_health(&self, id: u8) -> usize {
        self.cells
            .iter()
            .filter_map(|&c| match c {
                FCell::Owned(o_id, health, _, _) if o_id == id => Some(health as usize),
                _ => None,
            })
            .sum()
    }

    /// Counts the space of you or the enemies.
    pub fn count_space(&self, id: u8) -> usize {
        self.cells
            .iter()
            .filter(|&c| match c {
                FCell::Owned(o_id, _, _, _) if *o_id == id => true,
                _ => false,
            })
            .count()
    }

    /// Counts the space of you or the enemies weighted by the weight function.
    pub fn count_space_0<F: FnMut(Vec2D, FCell) -> f64>(
        &self,
        id: u8,
        mut weight: F,
    ) -> f64 {
        self.cells
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, c)| match c {
                FCell::Owned(o_id, _, _, _) if o_id == id => weight(
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
        self.queue.clear();
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
    fn flood(&mut self, grid: &Grid, heads: impl Iterator<Item = SnakePos>) -> [u16; 4] {

        #[inline]
        fn owns(cell: FCell, id: u8, distance: u16, food: u16, len: u16, health: u8) -> bool {
            match cell {
                FCell::Free => true,
                FCell::Occupied(o_id, tail_dist) if o_id == id => tail_dist + food <= distance,
                FCell::Occupied(_, tail_dist) => tail_dist < distance, // <= enemy eats!
                FCell::Owned(o_id, _, o_len, o_distance) if o_id != id => {
                    o_distance == distance && o_len < len
                }
                FCell::Owned(_, o_health, _, o_distance) => {
                    o_distance == distance && o_health < health
                }
            }
        }

        let mut food_distances = [u16::MAX; 4];
        let mut food_distance_i = 0;

        for SnakePos {
            p,
            id,
            distance,
            food: _,
            mut len,
            health,
        } in heads
        {
            if self.has(p) {
                let num = 1;
                let cell = self[p];
                let g_cell = grid[p];

                let health = if g_cell.food() {
                    if id == 0 && cell == FCell::Free && food_distance_i < food_distances.len() {
                        food_distances[food_distance_i] = 1;
                        food_distance_i += 1;
                    }
                    len += 1;
                    100
                } else {
                    health.saturating_sub(if g_cell.hazard() {
                        HAZARD_DAMAGE as u8
                    } else {
                        1
                    })
                };

                let food = g_cell.food() as u16;
                if owns(cell, id, num, food, len, health) {
                    self[p] = FCell::Owned(id, health, len, distance);
                    self.queue
                        .push_back(SnakePos::new(p, id, num + 1, food, len, health));
                }
            }
        }

        while let Some(SnakePos {
            p,
            id,
            distance,
            food,
            mut len,
            health,
        }) = self.queue.pop_front()
        {
            for p in Direction::iter().map(|d| p.apply(d)) {
                if self.has(p) {
                    let g_cell = grid[p];
                    let cell = self[p];

                    let health = if g_cell.food() {
                        len += 1;
                        if id == 0 && cell == FCell::Free && food_distance_i < food_distances.len()
                        {
                            food_distances[food_distance_i] = distance;
                            food_distance_i += 1;
                        }
                        100
                    } else {
                        health.saturating_sub(if g_cell.hazard() {
                            HAZARD_DAMAGE as u8
                        } else {
                            1
                        })
                    };

                    let food = food + g_cell.food() as u16;

                    if health > 0 && owns(cell, id, distance, food, len, health) {
                        self[p] = FCell::Owned(id, health, len, distance);
                        self.queue
                            .push_back(SnakePos::new(p, id, distance + 1, food, len, health));
                    }
                }
            }
        }
        food_distances
    }

    /// Prepare the board and compute flood fill.
    /// It is assumed that the snake at position and id 0 is the evaluated
    /// agent and the other snakes are the enemies.
    pub fn flood_snakes(&mut self, grid: &Grid, snakes: &[Snake]) -> [u16; 4] {
        self.clear();
        let mut snakes: Vec<(u8, &Snake)> = snakes
            .iter()
            .enumerate()
            .map(|(i, s)| (i as u8, s))
            .filter(|&(_, s)| s.alive())
            .collect();

        // Prepare board with snakes (tail = 1, ..., head = n)
        for &(id, snake) in &snakes {
            for (i, p) in snake.body.iter().enumerate() {
                self[*p] = FCell::Occupied(id, i as u16 + 1)
            }
        }

        // Longer or equally long snakes first
        snakes.sort_by_key(|&(id, s)| Reverse(2 * s.body.len() - (id == 0) as usize));
        self.flood(
            grid,
            snakes.iter().flat_map(|&(id, s)| {
                Direction::iter().map(move |d| {
                    SnakePos::new(s.head().apply(d), id, 1, 0, s.body.len() as _, s.health)
                })
            }),
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
        floodfill.flood(&grid, [SnakePos::new(Vec2D::new(0, 0), 0, 0, 0, 3, 100)].into_iter());
        info!("Filled {} {:?}", floodfill.count_space(0), floodfill);
        assert_eq!(floodfill.count_space(0), 11 * 11);

        let grid = Grid::new(11, 11);
        floodfill.clear();
        floodfill.flood(
            &grid,
            [
                SnakePos::new(Vec2D::new(0, 0), 0, 0, 0, 3, 100),
                SnakePos::new(Vec2D::new(10, 10), 1, 0, 0, 3, 100),
            ]
            .iter()
            .cloned(),
        );
        info!("Filled {} {:?}", floodfill.count_space(0), floodfill);
        assert_eq!(floodfill.count_space(0), 66);
        let grid = Grid::new(11, 11);
        floodfill.clear();
        floodfill.flood(
            &grid,
            [
                SnakePos::new(Vec2D::new(0, 0), 2, 0, 0, 4, 100),
                SnakePos::new(Vec2D::new(10, 10), 1, 0, 0, 3, 100),
                SnakePos::new(Vec2D::new(5, 5), 0, 0, 0, 2, 100),
            ]
            .iter()
            .cloned(),
        );
        info!("Filled {} {:?}", floodfill.count_space(0), floodfill);
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

        info!("Filled {} {:?}", floodfill.count_space(0), floodfill);
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

        info!("Filled {} {:?}", floodfill.count_space(0), floodfill);
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

        info!("Filled {} {:?}", floodfill.count_space(0), floodfill);
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

        info!("Filled {} {:?}", floodfill.count_space(0), floodfill);
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
        info!("Filled {} {:?}", floodfill.count_space(0), floodfill);
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
        info!("Filled {} {:?}", floodfill.count_space(0), floodfill);
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
        info!("Filled {} {:?}", floodfill.count_space(0), floodfill);
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
                game.grid[Vec2D::new(x as _, y as _)].set_hazard(true);
            }
        }

        let mut floodfill = FloodFill::new(game.grid.width, game.grid.height);
        floodfill.flood_snakes(&game.grid, &game.snakes);
        info!("Filled {} {:?}", floodfill.count_space(0), floodfill);
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

        let mut floodfill = FloodFill::new(game.grid.width, game.grid.height);
        floodfill.flood_snakes(&game.grid, &game.snakes);
        info!("Filled {} {:?}", floodfill.count_space(0), floodfill);
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
        let mut floodfill = FloodFill::new(game.grid.width, game.grid.height);
        floodfill.flood_snakes(&game.grid, &game.snakes);
        info!("Filled {} {:?}", floodfill.count_space(0), floodfill);
        assert_eq!(35, floodfill.count_space(0));
    }
}
