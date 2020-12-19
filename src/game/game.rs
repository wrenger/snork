use std::collections::VecDeque;

use super::Grid;
use crate::env::{Direction, SnakeData, Vec2D};

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

#[derive(Debug, Clone)]
struct Snake {
    /// tail to head
    body: VecDeque<Vec2D>,
    health: u8,
}
impl Snake {
    fn new(body: VecDeque<Vec2D>, health: u8) -> Snake {
        Snake { body, health }
    }

    fn head(&self) -> Vec2D {
        *self.body.back().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct StandardGame {
    snakes: [Option<Snake>; 4],
    grid: Grid,
}

impl StandardGame {
    pub fn new(width: usize, height: usize, snakes: &[SnakeData], food: &[Vec2D]) -> StandardGame {
        let mut grid = Grid::new(width, height);
        grid.add_food(food);

        let mut game_snakes = [None, None, None, None];
        for (i, snake) in snakes.iter().enumerate() {
            grid.add_snake(snake.body.iter().cloned());
            game_snakes[i] = Some(Snake::new(
                snake.body.iter().cloned().rev().collect(),
                snake.health,
            ))
        }

        StandardGame {
            snakes: game_snakes,
            grid,
        }
    }

    pub fn snake_is_alive(&self, snake: u8) -> bool {
        self.snakes[snake as usize].is_some()
    }

    pub fn valid_moves(&self, snake: u8) -> [bool; 4] {
        let mut moves = [false; 4];
        if let Some(snake) = &self.snakes[snake as usize] {
            for (i, d) in Direction::iter().enumerate() {
                let p = snake.head().apply(d);
                moves[i] = self.grid.has(p) && self.grid[p] != Cell::Occupied;
            }
        }
        moves
    }

    /// Moves the given snake in the given direction
    pub fn step(&mut self, snake: u8, direction: Direction) {
        let id = snake;

        // eat enemy
        if let Some((health, head, len)) = self.snakes[id as usize]
            .as_ref()
            .map(|s| (s.health, s.head(), s.body.len()))
        {
            if health > 0 {
                let killed_enemy: Option<u8> = self
                    .snakes
                    .iter()
                    .enumerate()
                    .find(|&(i, s)| {
                        i as u8 != id
                            && s.as_ref()
                                .map(|s| s.head() == head && s.body.len() < len)
                                .unwrap_or(false)
                    })
                    .map(&|(i, _)| i as u8);

                if let Some(enemy) = killed_enemy {
                    for &p in &self.snakes[enemy as usize].as_ref().unwrap().body {
                        self.grid[p] = Cell::Free;
                    }
                    self.snakes[enemy as usize] = None;
                    println!("{} killed {}", snake, enemy);
                }
            }
        }

        if let Some(snake) = &mut self.snakes[id as usize] {
            // pop tail
            let tail = snake.body.pop_front().unwrap();
            let new_tail = snake.body[0];
            if tail != new_tail {
                self.grid[tail] = Cell::Free;
            }

            // move head
            let head = snake.head().apply(direction);
            if self.grid.has(head) && snake.health > 0 && self.grid[head] != Cell::Occupied {
                if self.grid[head] == Cell::Food {
                    snake.body.push_front(new_tail);
                    snake.health = 100;
                } else {
                    snake.health -= 1;
                };
                self.grid[head] = Cell::Occupied;
                snake.body.push_back(head);
                return;
            }

            // die
            for &p in &snake.body {
                self.grid[p] = Cell::Free;
            }
            self.snakes[id as usize] = None
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn game_step_circle() {
        use super::*;
        use std::time::Instant;
        let snakes = [SnakeData::new(
            100,
            vec![
                Vec2D::new(4, 8),
                Vec2D::new(4, 7),
                Vec2D::new(4, 6),
                Vec2D::new(5, 6),
                Vec2D::new(6, 6),
                Vec2D::new(6, 6),
                Vec2D::new(6, 6),
            ],
        )];
        let mut game = StandardGame::new(11, 11, &snakes, &[]);
        println!("{:?}", game.grid);

        let start = Instant::now();
        loop {
            game.step(0, Direction::Up);
            game.step(0, Direction::Up);
            game.step(0, Direction::Right);
            game.step(0, Direction::Right);
            game.step(0, Direction::Down);
            game.step(0, Direction::Down);
            game.step(0, Direction::Left);
            game.step(0, Direction::Left);
            if !game.snake_is_alive(0) {
                break;
            }
        }
        println!("Dead after {}us", (Instant::now() - start).as_nanos());
    }

    #[test]
    fn game_step_random() {
        use super::*;
        use rand::{
            distributions::{Distribution, Uniform},
            seq::IteratorRandom,
        };
        use std::time::{Duration, Instant};
        const SIMULATION_TIME: usize = 200;

        let snakes = [
            SnakeData::new(
                100,
                vec![Vec2D::new(6, 7), Vec2D::new(6, 7), Vec2D::new(6, 7)],
            ),
            SnakeData::new(
                100,
                vec![Vec2D::new(3, 2), Vec2D::new(3, 2), Vec2D::new(3, 2)],
            ),
            SnakeData::new(
                100,
                vec![Vec2D::new(7, 3), Vec2D::new(7, 3), Vec2D::new(7, 3)],
            ),
            SnakeData::new(
                100,
                vec![Vec2D::new(3, 8), Vec2D::new(3, 8), Vec2D::new(3, 8)],
            ),
        ];
        let mut rng = rand::thread_rng();
        let mut game = StandardGame::new(11, 11, &snakes, &[]);

        let dist = Uniform::from(0..11);
        for _ in 0..20 {
            let p = Vec2D::new(dist.sample(&mut rng), dist.sample(&mut rng));
            if game.grid[p] == Cell::Free {
                game.grid[p] = Cell::Food;
            }
        }

        println!("{:?}", game.grid);

        let start = Instant::now();
        let mut game_num = 0_usize;
        loop {
            let mut turn = 0;
            let mut game = game.clone();
            loop {
                for i in 0..4 {
                    let d = game
                        .valid_moves(i)
                        .iter()
                        .enumerate()
                        .filter(|&(_, valid)| *valid)
                        .map(|v| Direction::from(v.0 as u8))
                        .choose(&mut rng)
                        .unwrap_or(Direction::Up);
                    game.step(i, d);
                }

                // println!("{} {:?}", turn, game.grid);

                if !game.snake_is_alive(0)
                    && !game.snake_is_alive(1)
                    && !game.snake_is_alive(2)
                    && !game.snake_is_alive(3)
                {
                    break;
                }
                turn += 1;
            }
            println!("game {}: dead after {} turns", game_num, turn);
            game_num += 1;

            if Instant::now() > start + Duration::from_millis(SIMULATION_TIME as _) {
                break;
            }
        }
        println!("Played {} games in {}ms", game_num, SIMULATION_TIME);
    }
}
