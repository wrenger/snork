use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

use super::{Cell, Grid};
use crate::env::{Direction, GameRequest, SnakeData, Vec2D};

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

    pub fn alive(&self) -> bool {
        self.health > 0
    }

    pub fn head(&self) -> Vec2D {
        *self.body.back().unwrap()
    }
}
impl Hash for Snake {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}
impl PartialEq for Snake {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Snake {}

#[derive(Debug, Clone)]
pub struct Game {
    /// All snakes. Dead ones have health = 0 and no body.
    /// The ids are have to match the index!
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

    pub fn reset_from_request(&mut self, request: &GameRequest) {
        let mut snakes = Vec::with_capacity(0);
        snakes.push(Snake::from(&request.you, 0));
        if request.board.snakes.len() > 4 {
            use priority_queue::PriorityQueue;
            let mut queue = PriorityQueue::new();
            for snake in request
                .board
                .snakes
                .iter()
                .filter(|s| s.id != request.you.id)
                .enumerate()
                .map(|(i, s)| Snake::from(s, i as u8 + 1))
            {
                let body_dist = snake
                    .body
                    .iter()
                    .map(|&p| (p - snakes[0].head()).manhattan())
                    .min()
                    .unwrap_or_default();
                let head_dist = (snake.body[0] - snakes[0].head()).manhattan() / 2;
                queue.push(snake, -(head_dist.max(body_dist) as i32));
            }
            snakes.extend(queue.into_iter().map(|(s, _)| s).take(3));
        } else {
            snakes.extend(
                request
                    .board
                    .snakes
                    .iter()
                    .filter(|s| s.id != request.you.id)
                    .enumerate()
                    .map(|(i, s)| Snake::from(s, i as u8 + 1)),
            );
        }
        self.reset(snakes, &request.board.food);
    }

    pub fn reset(&mut self, snakes: Vec<Snake>, food: &[Vec2D]) {
        self.grid.clear();
        self.grid.add_food(food);

        for (i, snake) in snakes.iter().enumerate() {
            assert_eq!(snake.id, i as u8);
            self.grid.add_snake(snake.body.iter().cloned());
        }
        self.snakes = snakes
    }

    pub fn outcome(&self) -> Outcome {
        let mut living_snakes = 0;
        let mut survivor = 0;
        for snake in &self.snakes {
            if snake.alive() {
                living_snakes += 1;
                survivor = snake.id;
            }
        }
        match living_snakes {
            0 => Outcome::Match,
            1 => Outcome::Winner(survivor),
            _ => Outcome::None,
        }
    }

    pub fn snake_is_alive(&self, snake: u8) -> bool {
        snake < self.snakes.len() as u8 && self.snakes[snake as usize].alive()
    }

    pub fn valid_moves(&self, snake: u8) -> ValidMoves {
        if self.snake_is_alive(snake) {
            ValidMoves::new(self, &self.snakes[snake as usize])
        } else {
            ValidMoves::empty(self)
        }
    }

    pub fn move_is_valid(&self, snake: u8, dir: Direction) -> bool {
        if self.snake_is_alive(snake) {
            let snake = &self.snakes[snake as usize];
            self.snake_move_is_valid(snake, dir)
        } else {
            false
        }
    }

    fn snake_move_is_valid(&self, snake: &Snake, dir: Direction) -> bool {
        let p = snake.head().apply(dir);
        // Free or occupied by tail (free in the next turn)
        self.grid.has(p)
            && (self.grid[p] != Cell::Occupied
                || self
                    .snakes
                    .iter()
                    .filter(|s| s.alive())
                    .any(|s| p == s.body[0] && p != s.body[1]))
    }

    /// Moves the given snake in the given direction
    pub fn step(&mut self, moves: &[Direction]) {
        assert!(moves.len() >= self.snakes.len());

        // Pop tail
        for snake in &mut self.snakes {
            if snake.alive() {
                let tail = snake.body.pop_front().unwrap();
                let new_tail = snake.body[0];
                if tail != new_tail {
                    self.grid[tail] = Cell::Free;
                }
            }
        }

        // Move head & eat
        for snake in &mut self.snakes {
            if snake.alive() {
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
                } else {
                    snake.health = 0;
                }
            }
        }

        // Check head to head
        // This is only accurate for head to head on two snakes but not more
        for i in 0..self.snakes.len() - 1 {
            if self.snakes[i].alive() {
                for j in i + 1..self.snakes.len() {
                    if self.snakes[j].alive() && self.snakes[i].head() == self.snakes[j].head() {
                        use std::cmp::Ordering;
                        match self.snakes[i].body.len().cmp(&self.snakes[j].body.len()) {
                            Ordering::Less => self.snakes[i].health = 0,
                            Ordering::Greater => self.snakes[j].health = 0,
                            Ordering::Equal => {
                                self.snakes[i].health = 0;
                                self.snakes[j].health = 0;
                            }
                        }
                    }
                }
            }
        }

        // Clear died snakes
        let grid = &mut self.grid;
        for snake in &mut self.snakes {
            if snake.alive() {
                grid[snake.head()] = Cell::Occupied;
            } else if !snake.body.is_empty() {
                for &p in &snake.body {
                    grid[p] = Cell::Free
                }
                snake.body.clear();
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
                self.dir += 1;
                if self.game.snake_move_is_valid(snake, d) {
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
    fn game_step() {
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
        game.step(&[Right, Right]);

        println!("{:?}", game.grid);
        assert!(game.snake_is_alive(0));
        assert!(game.snake_is_alive(1));
        assert_eq!(game.grid[Vec2D::new(4, 6)], Cell::Free);
        assert_eq!(game.grid[Vec2D::new(5, 8)], Cell::Occupied);
        assert_eq!(game.grid[Vec2D::new(6, 6)], Cell::Free);
        assert_eq!(game.grid[Vec2D::new(7, 8)], Cell::Occupied);

        game.step(&[Right, Right]);
        println!("{:?}", game.grid);
        assert!(!game.snake_is_alive(0));
        assert_eq!(game.grid[Vec2D::new(5, 8)], Cell::Free);
        assert!(game.snake_is_alive(1));
        assert_eq!(game.grid[Vec2D::new(8, 8)], Cell::Occupied);

        game.reset(snakes, &[]);
        game.step(&[Right, Left]);
        println!("{:?}", game.grid);
        assert!(!game.snake_is_alive(0));
        assert!(!game.snake_is_alive(1));
    }

    #[test]
    fn test_valid_moves() {
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
    fn bench_step_circle() {
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
            game.step(&[Up]);
            game.step(&[Up]);
            game.step(&[Right]);
            game.step(&[Right]);
            game.step(&[Down]);
            game.step(&[Down]);
            game.step(&[Left]);
            game.step(&[Left]);
            if !game.snake_is_alive(0) {
                break;
            }
        }
        println!("Dead after {}us", (Instant::now() - start).as_nanos());
    }

    #[test]
    #[ignore]
    fn bench_step_random() {
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
                game.step(&moves);

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
