use std::cmp::Reverse;
use std::collections::VecDeque;
use std::ops::{Index, IndexMut};

use super::{Grid, Snake};
use crate::env::{Direction, Vec2D, HAZARD_DAMAGE};

use owo_colors::{OwoColorize, Style};

/// Floodfill Cell that stores the important data in a single Byte.
///
/// Bitfield:
/// - 15: you?
/// - 14: owned?
/// - 13: occupied?
/// - 11..0: num (head distance if occupied, health if owned)
#[derive(Clone, Copy)]
pub struct FCell(u16);

const FCELL_YOU: usize = 15;
const FCELL_OWNED: usize = 14;
const FCELL_OCCUPIED: usize = 13;
const FCELL_NUM: usize = 12;

impl FCell {
    #[inline(always)]
    pub const fn free() -> FCell {
        FCell(0)
    }

    #[inline(always)]
    pub const fn with_owner(you: bool, health: u8) -> FCell {
        FCell((you as u16) << FCELL_YOU | 1 << FCELL_OWNED | health as u16 & ((1 << FCELL_NUM) - 1))
    }

    #[inline(always)]
    pub const fn with_occupier(you: bool, num: u16) -> FCell {
        FCell((you as u16) << FCELL_YOU | 1 << FCELL_OCCUPIED | num & ((1 << FCELL_NUM) - 1))
    }

    #[inline(always)]
    pub const fn is_free(&self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub const fn is_you(&self) -> bool {
        self.0 & (1 << FCELL_YOU) != 0
    }

    #[inline(always)]
    pub const fn is_owned(&self) -> bool {
        self.0 & (1 << FCELL_OWNED) != 0
    }

    #[inline(always)]
    pub const fn get_num(&self) -> u16 {
        self.0 & ((1 << FCELL_NUM) - 1)
    }

    #[inline(always)]
    pub const fn is_occupied(&self) -> bool {
        self.0 & (1 << FCELL_OCCUPIED) != 0
    }
}

impl std::fmt::Debug for FCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_free() {
            write!(f, "___")
        } else {
            let style = if self.is_you() {
                Style::new().green()
            } else {
                Style::new().red()
            };
            let style = if self.is_owned() {
                style.on_bright_black()
            } else {
                style
            };
            if self.is_occupied() {
                write!(f, "{:0>3}", self.get_num().style(style))
            } else {
                write!(f, "{:0>3}", self.get_num().style(style))
            }
        }
    }
}

#[derive(Debug)]
struct SnakePos {
    p: Vec2D,
    you: bool,
    distance: u16,
    food: u16,
    health: u8,
}

impl SnakePos {
    fn new(p: Vec2D, you: bool, distance: u16, food: u16, health: u8) -> Self {
        Self {
            p,
            you,
            distance,
            food,
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
            cells: vec![FCell::free(); width * height],
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
    pub fn count_health(&self, you: bool) -> usize {
        self.cells
            .iter()
            .filter(|&c| c.is_owned() && c.is_you() == you)
            .map(|&c| c.get_num() as usize)
            .sum()
    }

    /// Counts the space of you or the enemies.
    pub fn count_space(&self, you: bool) -> usize {
        self.cells
            .iter()
            .filter(|&c| c.is_owned() && c.is_you() == you)
            .count()
    }

    /// Counts the space of you or the enemies weighted by the weight function.
    pub fn count_space_weighted<F: FnMut(Vec2D, FCell) -> f64>(
        &self,
        you: bool,
        mut weight: F,
    ) -> f64 {
        self.cells
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, c)| {
                if c.is_owned() && c.is_you() == you {
                    weight(
                        Vec2D::new((i % self.width) as i16, (i / self.width) as i16),
                        c,
                    )
                } else {
                    0.0
                }
            })
            .sum()
    }

    /// Clears the board so that it can be reused for another floodfill computation.
    pub fn clear(&mut self) {
        for c in &mut self.cells {
            *c = FCell::free();
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
    pub fn flood(
        &mut self,
        grid: &Grid,
        heads: impl Iterator<Item = (Vec2D, bool, u8)>,
    ) -> Vec<u16> {
        assert_eq!(self.width, grid.width);
        assert_eq!(self.height, grid.height);

        #[inline]
        fn owns(cell: FCell, you: bool, num: u16, food: u16, health: u8) -> bool {
            cell.is_free()
                || (cell.is_occupied()
                    // follow your tail
                    && ((cell.is_you() == you && cell.get_num() <= num - food)
                        // follow enemy tail
                        // distance of 1 as buffer for eating
                        || (cell.is_you() != you && (cell.get_num() <= 1 || cell.get_num() <= num - you as u16))))
                || (cell.is_owned() && cell.is_you() == you && cell.get_num() < health as u16)
        }

        let mut food_distances = Vec::new();

        for (p, you, health) in heads {
            if self.has(p) {
                let num = 1;
                let cell = self[p];
                let g_cell = grid[p];

                let health = if g_cell.food() {
                    if you && !cell.is_owned() {
                        food_distances.push(1)
                    }
                    100
                } else {
                    health.saturating_sub(if g_cell.hazard() {
                        HAZARD_DAMAGE as u8
                    } else {
                        1
                    })
                };

                let food = g_cell.food() as u16;
                if owns(cell, you, num, food, health) {
                    self[p] = FCell::with_owner(you, health);
                    self.queue
                        .push_back(SnakePos::new(p, you, num + 1, food, health));
                }
            }
        }

        while let Some(SnakePos {
            p,
            you,
            distance,
            food,
            health,
        }) = self.queue.pop_front()
        {
            for p in Direction::iter().map(|d| p.apply(d)) {
                if self.has(p) {
                    let g_cell = grid[p];
                    let cell = self[p];

                    let health = if g_cell.food() {
                        if you && !cell.is_owned() {
                            food_distances.push(distance)
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

                    if health > 0 && owns(cell, you, distance, food, health) {
                        self[p] = FCell::with_owner(you, health);
                        self.queue
                            .push_back(SnakePos::new(p, you, distance + 1, food, health));
                    }
                }
            }
        }
        food_distances
    }

    /// Prepare the board and compute flood fill.
    /// It is assumed that the snake at position and id 0 is the evaluated
    /// agent and the other snakes are the enemies.
    pub fn flood_snakes(&mut self, grid: &Grid, snakes: &[Snake]) -> Vec<u16> {
        self.clear();
        let mut snakes: Vec<&Snake> = snakes.iter().filter(|s| s.alive()).collect();

        // Prepare board with snakes (tail = 1, ..., head = n)
        for snake in &snakes {
            for (i, p) in snake.body.iter().enumerate() {
                let num = (i + 1).min(1 << FCELL_NUM - 1) as _;
                self[*p] = FCell::with_occupier(snake.id == 0, num)
            }
        }

        // Longer or equally long snakes first
        snakes.sort_by_key(|s| Reverse(2 * s.body.len() - (s.id == 0) as usize));
        self.flood(
            grid,
            snakes.iter().flat_map(|s| {
                Direction::iter().map(move |d| (s.head().apply(d), s.id == 0, s.health))
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

    #[test]
    fn flood_head() {
        use super::*;
        let grid = Grid::new(11, 11);

        let mut floodfill = FloodFill::new(grid.width, grid.height);
        floodfill.flood(&grid, [(Vec2D::new(0, 0), true, 100)].iter().cloned());
        println!("Filled {} {:?}", floodfill.count_space(true), floodfill);
        assert_eq!(floodfill.count_space(true), 11 * 11);

        let grid = Grid::new(11, 11);
        floodfill.clear();
        floodfill.flood(
            &grid,
            [
                (Vec2D::new(0, 0), true, 100),
                (Vec2D::new(10, 10), false, 100),
            ]
            .iter()
            .cloned(),
        );
        println!("Filled {} {:?}", floodfill.count_space(true), floodfill);
        assert_eq!(floodfill.count_space(true), 66);
        let grid = Grid::new(11, 11);
        floodfill.clear();
        floodfill.flood(
            &grid,
            [
                (Vec2D::new(0, 0), false, 100),
                (Vec2D::new(10, 10), false, 100),
                (Vec2D::new(5, 5), true, 100),
            ]
            .iter()
            .cloned(),
        );
        println!("Filled {} {:?}", floodfill.count_space(true), floodfill);
        assert_eq!(floodfill.count_space(true), 61);
    }

    #[test]
    fn flood_snakes_follow_tail() {
        use super::*;
        use crate::game::Game;

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

        println!("Filled {} {:?}", floodfill.count_space(true), floodfill);
        assert_eq!(floodfill.count_space(true), 11 * 11);

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

        println!("Filled {} {:?}", floodfill.count_space(true), floodfill);
        assert_eq!(floodfill.count_space(true), 11 * 11);
    }

    #[test]
    fn flood_snakes_bite_tail() {
        use super::*;
        use crate::game::Game;

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

        println!("Filled {} {:?}", floodfill.count_space(true), floodfill);
        assert_eq!(floodfill.count_space(true), 4);
    }

    #[test]
    fn flood_snakes_bite_food() {
        use super::*;
        use crate::game::Game;

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

        println!("Filled {} {:?}", floodfill.count_space(true), floodfill);
        assert_eq!(floodfill.count_space(true), 11 * 11);

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
        println!("Filled {} {:?}", floodfill.count_space(true), floodfill);
        assert_eq!(floodfill.count_space(true), 1);
    }

    #[test]
    fn flood_snakes_enemy() {
        use super::*;
        use crate::game::Game;

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
        println!("Filled {} {:?}", floodfill.count_space(true), floodfill);
        assert_eq!(floodfill.count_space(true), 24);
    }

    #[test]
    fn flood_snakes_low_health() {
        use super::*;
        use crate::game::Game;

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
        println!("Filled {} {:?}", floodfill.count_space(true), floodfill);
        assert_eq!(floodfill.count_space(true), 59);
    }

    #[test]
    fn flood_snakes_hazard() {
        use super::*;
        use crate::game::Game;

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
        println!("Filled {} {:?}", floodfill.count_space(true), floodfill);
        assert_eq!(floodfill.count_space(true), 97);
    }
}
