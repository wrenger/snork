use std::collections::VecDeque;

use super::{Cell, Grid};
use crate::env::{Direction, SnakeData, Vec2D};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Outcome {
    None,
    Match,
    Winner(u8),
}

#[derive(Debug, Clone)]
pub struct Snake {
    pub id: u8,
    /// tail to head
    pub body: VecDeque<Vec2D>,
    pub health: u8,
}
impl Snake {
    pub fn new(id: u8, body: VecDeque<Vec2D>, health: u8) -> Snake {
        Snake { id, body, health }
    }

    pub fn from(snake: &SnakeData, id: u8) -> Snake {
        Snake::new(id, snake.body.iter().cloned().rev().collect(), snake.health)
    }

    pub fn head(&self) -> Vec2D {
        *self.body.back().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    pub snakes: Vec<Snake>,
    pub grid: Grid,
}

impl Game {
    pub fn new(width: usize, height: usize) -> Game {
        Game {
            snakes: Vec::with_capacity(4),
            grid: Grid::new(width, height),
        }
    }

    pub fn reset(&mut self, snakes: Vec<Snake>, food: &[Vec2D]) {
        self.grid.clear();
        self.grid.add_food(food);

        for snake in &snakes {
            self.grid.add_snake(snake.body.iter().cloned());
        }
        self.snakes = snakes
    }

    pub fn outcome(&self) -> Outcome {
        match self.snakes.len() {
            0 => Outcome::Match,
            1 => Outcome::Winner(self.snakes[0].id),
            _ => Outcome::None,
        }
    }

    pub fn snake_is_alive(&self, snake: u8) -> bool {
        self.snakes.iter().any(|s| s.id == snake)
    }

    pub fn valid_moves(&self, snake: u8) -> ValidMoves {
        if let Some(snake) = self.snakes.iter().find(|s| s.id == snake) {
            ValidMoves::new(self, snake)
        } else {
            ValidMoves::empty(self)
        }
    }

    /// Moves the given snake in the given direction
    pub fn step(&mut self, moves: [Direction; 4]) {
        // pop tail
        for snake in &mut self.snakes {
            let tail = snake.body.pop_front().unwrap();
            let new_tail = snake.body[0];
            if tail != new_tail {
                self.grid[tail] = Cell::Free;
            }
        }

        let mut survivors = [None; 4];

        // move head & eat
        for snake in &mut self.snakes {
            let dir = moves[snake.id as usize];
            let head = snake.head().apply(dir);
            if self.grid.has(head) && snake.health > 0 && self.grid[head] != Cell::Occupied {
                if self.grid[head] == Cell::Food {
                    snake.body.push_front(snake.body[0]);
                    snake.health = 100;
                } else {
                    snake.health -= 1;
                };
                snake.body.push_back(head);
                survivors[snake.id as usize] = Some((head, snake.body.len()));
            }
        }

        // check head to head
        for i in 0..3 {
            for j in i + 1..4 {
                if let Some(((head_i, len_i), (head_j, len_j))) = survivors[i].zip(survivors[j]) {
                    if head_i == head_j {
                        use std::cmp::Ordering;
                        match len_i.cmp(&len_j) {
                            Ordering::Less => survivors[i] = None,
                            Ordering::Greater => survivors[j] = None,
                            Ordering::Equal => {
                                survivors[i] = None;
                                survivors[j] = None;
                            }
                        }
                    }
                }
            }
        }

        // remove died snakes
        for (i, survivor) in survivors.iter().enumerate() {
            if let Some(survivor) = *survivor {
                self.grid[survivor.0] = Cell::Occupied;
            } else if let Some(pos) = self.snakes.iter().position(|s| s.id == i as u8) {
                for &p in &self.snakes[pos].body {
                    self.grid[p] = Cell::Free
                }
                self.snakes.remove(pos);
            }
        }
    }
}

pub struct ValidMoves<'a> {
    game: &'a Game,
    snake: Option<&'a Snake>,
    dir: u8,
}

impl<'a> ValidMoves<'a> {
    fn empty(game: &'a Game) -> ValidMoves {
        ValidMoves {
            game,
            snake: None,
            dir: 0,
        }
    }

    fn new(game: &'a Game, snake: &'a Snake) -> ValidMoves<'a> {
        ValidMoves {
            game,
            snake: Some(snake),
            dir: 0,
        }
    }
}

impl<'a> Iterator for ValidMoves<'a> {
    type Item = Direction;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(snake) = self.snake {
            while self.dir < 4 {
                let d = Direction::from(self.dir);
                let p = snake.head().apply(d);
                self.dir += 1;

                let grid = &self.game.grid;
                let snakes = &self.game.snakes;
                // Free or occupied by tail (free in the next turn)
                if grid.has(p)
                    && (grid[p] != Cell::Occupied
                        || snakes.iter().any(|s| p == s.body[0] && p != s.body[1]))
                {
                    return Some(d);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn game_step_test() {
        use super::*;
        use Direction::*;
        let snakes = vec![
            Snake::new(
                0,
                vec![Vec2D::new(4, 6), Vec2D::new(4, 7), Vec2D::new(4, 8)].into(),
                100,
            ),
            Snake::new(
                1,
                vec![Vec2D::new(6, 6), Vec2D::new(6, 7), Vec2D::new(6, 8)].into(),
                100,
            ),
        ];
        let mut game = Game::new(11, 11);
        game.reset(snakes.clone(), &[]);
        println!("{:?}", game.grid);
        game.step([Right, Right, Up, Up]);

        println!("{:?}", game.grid);
        assert!(game.snake_is_alive(0));
        assert!(game.snake_is_alive(1));
        assert_eq!(game.grid[Vec2D::new(4, 6)], Cell::Free);
        assert_eq!(game.grid[Vec2D::new(5, 8)], Cell::Occupied);
        assert_eq!(game.grid[Vec2D::new(6, 6)], Cell::Free);
        assert_eq!(game.grid[Vec2D::new(7, 8)], Cell::Occupied);

        game.step([Right, Right, Up, Up]);
        println!("{:?}", game.grid);
        assert!(!game.snake_is_alive(0));
        assert_eq!(game.grid[Vec2D::new(5, 8)], Cell::Free);
        assert!(game.snake_is_alive(1));
        assert_eq!(game.grid[Vec2D::new(8, 8)], Cell::Occupied);

        game.reset(snakes, &[]);
        game.step([Right, Left, Up, Up]);
        println!("{:?}", game.grid);
        assert!(!game.snake_is_alive(0));
        assert!(!game.snake_is_alive(1));
    }

    #[test]
    fn game_valid_moves() {
        use super::*;
        use Direction::*;

        let snakes = vec![
            Snake::new(
                0,
                vec![Vec2D::new(4, 1), Vec2D::new(4, 0), Vec2D::new(5, 0)].into(),
                100,
            ),
            Snake::new(
                1,
                vec![Vec2D::new(6, 0), Vec2D::new(6, 1), Vec2D::new(5, 1)].into(),
                100,
            ),
        ];
        let mut game = Game::new(11, 11);
        game.reset(snakes, &[]);

        println!("{:?}", game.grid);
        assert!([Right].iter().cloned().eq(game.valid_moves(0)));
    }

    #[test]
    #[ignore]
    fn game_step_circle() {
        use super::*;
        use std::time::Instant;
        let snakes = vec![Snake::new(
            0,
            vec![
                Vec2D::new(6, 6),
                Vec2D::new(6, 6),
                Vec2D::new(6, 6),
                Vec2D::new(5, 6),
                Vec2D::new(4, 6),
                Vec2D::new(4, 7),
                Vec2D::new(4, 8),
            ]
            .into(),
            100,
        )];
        let mut game = Game::new(11, 11);
        game.reset(snakes, &[]);
        println!("{:?}", game.grid);

        let start = Instant::now();
        loop {
            use Direction::*;
            game.step([Up, Up, Up, Up]);
            game.step([Up, Up, Up, Up]);
            game.step([Right, Up, Up, Up]);
            game.step([Right, Up, Up, Up]);
            game.step([Down, Up, Up, Up]);
            game.step([Down, Up, Up, Up]);
            game.step([Left, Up, Up, Up]);
            game.step([Left, Up, Up, Up]);
            if !game.snake_is_alive(0) {
                break;
            }
        }
        println!("Dead after {}us", (Instant::now() - start).as_nanos());
    }

    #[test]
    #[ignore]
    fn game_step_random() {
        use super::*;
        use rand::{
            distributions::{Distribution, Uniform},
            seq::IteratorRandom,
        };
        use std::time::{Duration, Instant};
        const SIMULATION_TIME: usize = 200;

        let snakes = vec![
            Snake::new(
                0,
                vec![Vec2D::new(6, 7), Vec2D::new(6, 7), Vec2D::new(6, 7)].into(),
                100,
            ),
            Snake::new(
                1,
                vec![Vec2D::new(3, 2), Vec2D::new(3, 2), Vec2D::new(3, 2)].into(),
                100,
            ),
            Snake::new(
                2,
                vec![Vec2D::new(7, 3), Vec2D::new(7, 3), Vec2D::new(7, 3)].into(),
                100,
            ),
            Snake::new(
                3,
                vec![Vec2D::new(3, 8), Vec2D::new(3, 8), Vec2D::new(3, 8)].into(),
                100,
            ),
        ];
        let mut rng = rand::thread_rng();
        let mut game = Game::new(11, 11);
        game.reset(snakes, &[]);

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
                let mut moves = [Direction::Up; 4];
                for i in 0..4 {
                    moves[i as usize] = game
                        .valid_moves(i)
                        .choose(&mut rng)
                        .unwrap_or(Direction::Up);
                }
                game.step(moves);

                // println!("{} {:?}", turn, game.grid);

                if game.outcome() != Outcome::None {
                    println!(
                        "game {}: {:?} after {} turns",
                        game_num,
                        game.outcome(),
                        turn
                    );
                    break;
                }
                turn += 1;
            }
            game_num += 1;

            if Instant::now() > start + Duration::from_millis(SIMULATION_TIME as _) {
                break;
            }
        }
        println!("Played {} games in {}ms", game_num, SIMULATION_TIME);
    }
}
