use std::cmp::Reverse;
use std::collections::VecDeque;
use std::f64;
use std::ops::Index;
use std::ops::IndexMut;
use std::usize;

use super::Cell;
use crate::env::{Direction, SnakeData, Vec2D};

#[derive(Debug, Clone)]
struct GridSnake {
    head: Vec2D,
    health: u8,
}
impl GridSnake {
    fn new(head: Vec2D, health: u8) -> GridSnake {
        GridSnake { head, health }
    }
}

#[derive(Debug)]
struct GridSnakeBody<'a> {
    p: Vec2D,
    grid: &'a Grid,
    tail_count: u8,
}

impl<'a> GridSnakeBody<'a> {
    fn new(grid: &'a Grid, head: Vec2D) -> GridSnakeBody {
        GridSnakeBody {
            p: head,
            grid,
            tail_count: 0,
        }
    }
}

impl<'a> Iterator for GridSnakeBody<'a> {
    type Item = Vec2D;
    fn next(&mut self) -> Option<Self::Item> {
        self.grid.snake_body_next(&mut self.p, &mut self.tail_count)
    }
}

#[derive(Debug)]
struct GridSnakeCellMut<'a> {
    p: Vec2D,
    grid: &'a mut Grid,
    tail_count: u8,
}

impl<'a> GridSnakeCellMut<'a> {
    fn new(grid: &'a mut Grid, head: Vec2D) -> GridSnakeCellMut {
        GridSnakeCellMut {
            p: head,
            grid,
            tail_count: 0,
        }
    }
}

impl<'a> Iterator for GridSnakeCellMut<'a> {
    type Item = &'a mut Cell;
    fn next(&mut self) -> Option<&'a mut Cell> {
        if let Some(p) = self.grid.snake_body_next(&mut self.p, &mut self.tail_count) {
            Some(unsafe { &mut *(self.grid.index_mut(p) as *mut _) })
        } else {
            None
        }
    }
}

/// The game state including up to four snakes.
#[derive(Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    snakes: [Option<GridSnake>; 4],
    cells: Vec<Cell>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Grid {
        Grid {
            width,
            height,
            snakes: [None, None, None, None],
            cells: vec![Cell::default(); width * height],
        }
    }

    pub fn add_snake(&mut self, id: u8, snake: &SnakeData) {
        for i in 0..snake.body.len() {
            let p = snake.body[i];
            match self[p] {
                Cell::Free | Cell::Food | Cell::Owned(_) => {
                    self[p] = Cell::snake_body(id, p, snake.body.get(i + 1).cloned())
                }
                Cell::Tail(i) if i == id => self[p] = Cell::TailDouble(id),
                Cell::TailDouble(i) if i == id => self[p] = Cell::TailTriple(id),
                _ => unreachable!("Overlapping snakes!"),
            }
        }
        self.snakes[id as usize] = Some(GridSnake::new(snake.body[0], snake.health))
    }

    pub fn snake_health(&self, snake: u8) -> Option<u8> {
        self.snakes[snake as usize].as_ref().map(|s| s.health)
    }

    pub fn snake_body(&self, snake: u8) -> Option<GridSnakeBody> {
        self.snakes[snake as usize]
            .as_ref()
            .map(|s| GridSnakeBody::new(self, s.head))
    }

    fn snake_body_next(&self, p: &mut Vec2D, tail_count: &mut u8) -> Option<Vec2D> {
        let result = *p;
        match self[*p] {
            Cell::Up(_) => *p = p.apply(Direction::Up),
            Cell::Right(_) => *p = p.apply(Direction::Right),
            Cell::Down(_) => *p = p.apply(Direction::Down),
            Cell::Left(_) => *p = p.apply(Direction::Left),
            Cell::Tail(_) => {
                if *tail_count < 1 {
                    *tail_count += 1;
                } else {
                    return None;
                }
            }
            Cell::TailDouble(_) => {
                if *tail_count < 2 {
                    *tail_count += 1;
                } else {
                    return None;
                }
            }
            Cell::TailTriple(_) => {
                if *tail_count < 3 {
                    *tail_count += 1;
                } else {
                    return None;
                }
            }
            Cell::Free => return None,
            _ => unreachable!("Invalid Grid Data!"),
        }
        Some(result)
    }

    pub fn snake_is_alive(&self, snake: u8) -> bool {
        self.snakes[snake as usize].is_some()
    }

    pub fn add_food(&mut self, food: &[Vec2D]) {
        for &p in food {
            if self.has(p) && !self[p].is_occupied() {
                self[p] = Cell::Food;
            }
        }
    }

    pub fn has(&self, p: Vec2D) -> bool {
        0 <= p.x && p.x < self.width as _ && 0 <= p.y && p.y < self.height as _
    }

    pub fn valid_moves(&self, snake: u8) -> [bool; 4] {
        let mut moves = [false; 4];
        if let Some(snake) = &self.snakes[snake as usize] {
            for (i, d) in Direction::iter().enumerate() {
                let p = snake.head.apply(d);
                moves[i] = self.has(p) && !self[p].is_occupied();
            }
        }
        moves
    }

    /// Moves the given snake in the given direction
    pub fn step(&mut self, snake: u8, direction: Direction) {
        let id = snake;
        if let Some(snake) = self.snakes[id as usize].clone() {
            // pop tail
            let mut tail = snake.head;
            let mut new_tail = snake.head;
            for p in GridSnakeBody::new(self, snake.head) {
                new_tail = tail;
                tail = p;
            }

            match self[tail] {
                Cell::Tail(_) => {
                    self[tail] = Cell::Free;
                    self[new_tail] = Cell::Tail(id)
                }
                Cell::TailDouble(_) => self[tail] = Cell::Tail(id),
                Cell::TailTriple(_) => self[tail] = Cell::TailDouble(id),
                _ => unreachable!("Invalid Grid Data"),
            }

            // move head

            let head = snake.head.apply(direction);
            if snake.health > 0 && self.has(head) && !self[head].is_occupied() {
                let health = if self[head] == Cell::Food {
                    match self[new_tail] {
                        Cell::Tail(_) => self[new_tail] = Cell::TailDouble(id),
                        Cell::TailDouble(_) => self[new_tail] = Cell::TailTriple(id),
                        _ => unreachable!("Invalid Grid Data"),
                    }
                    100
                } else {
                    snake.health - 1
                };
                self[head] = Cell::snake_body(id, head, Some(snake.head));
                self.snakes[id as usize] = Some(GridSnake::new(head, health));
            } else {
                println!("dead {} hp={} d={:?}", id, snake.health, direction);
                // die
                for cell in GridSnakeCellMut::new(self, snake.head) {
                    *cell = Cell::Free;
                }
                self.snakes[id as usize] = None
            }
        }
    }

    pub fn count_space_of(&self, snake: u8) -> usize {
        self.cells.iter().filter(|&c| c.is_owned_by(snake)).count()
    }

    pub fn flood_fill(&mut self, heads: impl Iterator<Item = (u8, Vec2D)>) {
        let mut queue = VecDeque::with_capacity(self.width * self.height);
        for (i, p) in heads {
            if self.has(p) && !self[p].is_occupied() {
                queue.push_back((i, p));
            }
        }
        while let Some((i, p)) = queue.pop_front() {
            if self.has(p) && !self[p].is_owned_or_occupied() {
                self[p] = Cell::Owned(i);

                for dir in Direction::iter() {
                    let p = p.apply(dir);
                    if self.has(p) && !self[p].is_owned_or_occupied() {
                        queue.push_back((i, p));
                    }
                }
            }
        }
    }

    pub fn flood_fill_snakes(&mut self, snakes: &[SnakeData], you_i: u8) {
        let mut snakes: Vec<(u8, &SnakeData)> = snakes
            .iter()
            .enumerate()
            .map(|(i, s)| (i as u8, s))
            .collect();
        // Longer or equally long snakes first
        snakes.sort_by_key(|&(i, s)| Reverse(2 * s.body.len() - (i == you_i) as usize));
        self.flood_fill(
            snakes
                .iter()
                .flat_map(|&(i, s)| Direction::iter().map(move |d| (i, s.body[0].apply(d)))),
        );
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

                if self.has(neighbor) && !self[neighbor].is_occupied() {
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

    pub fn space_after_move(&self, you_i: u8, snakes: &[SnakeData]) -> [usize; 4] {
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
            let mut next_grid = self.clone();
            // free tail
            for (_, snake) in &snakes {
                if snake.body[snake.body.len() - 1] != snake.body[snake.body.len() - 2] {
                    next_grid[snake.body[snake.body.len() - 1]] = Cell::Free;
                }
            }
            // longer heads
            let mut next_heads: Vec<(u8, Vec2D)> = Vec::new();
            for &(i, p) in &longer_enemies {
                if self.has(p) && !self[p].is_occupied() {
                    next_heads.extend(Direction::iter().map(move |d| (i, p.apply(d))));
                    next_grid[p] = Cell::Tail(i);
                }
            }
            if self.has(p) && !self[p].is_occupied() {
                next_heads.extend(Direction::iter().map(move |d| (you_i, p.apply(d))));
                next_grid[p] = Cell::snake_body(you_i, p, Some(you.body[0]));
                // shorter heads
                for &(i, p) in &shorter_enemies {
                    if next_grid.has(p) {
                        next_heads.extend(Direction::iter().map(move |d| (i, p.apply(d))));
                        next_grid[p] = Cell::Tail(i);
                    }
                }

                next_grid.flood_fill(next_heads.iter().cloned());
                space_after_move[dir_i] = next_grid.count_space_of(you_i);
            }
        }
        space_after_move
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
    fn grid_snake() {
        use super::*;
        let mut grid = Grid::new(11, 11);
        let snake = SnakeData::new(
            100,
            vec![Vec2D::new(5, 5), Vec2D::new(5, 5), Vec2D::new(5, 5)],
        );
        grid.add_snake(0, &snake);
        println!("{:?}", grid);
        assert_eq!(grid[Vec2D::new(5, 5)], Cell::TailTriple(0));
        assert_eq!(grid.snake_body(0).unwrap().collect::<Vec<_>>(), snake.body);

        let mut grid = Grid::new(11, 11);
        let snake = SnakeData::new(
            100,
            vec![Vec2D::new(6, 5), Vec2D::new(5, 5), Vec2D::new(5, 5)],
        );
        grid.add_snake(0, &snake);
        println!("{:?}", grid);
        assert_eq!(grid[Vec2D::new(6, 5)], Cell::Left(0));
        assert_eq!(grid[Vec2D::new(5, 5)], Cell::TailDouble(0));
        assert_eq!(grid.snake_body(0).unwrap().collect::<Vec<_>>(), snake.body);

        let mut grid = Grid::new(11, 11);
        let snake = SnakeData::new(
            100,
            vec![Vec2D::new(6, 6), Vec2D::new(6, 5), Vec2D::new(5, 5)],
        );
        grid.add_snake(0, &snake);
        println!("{:?}", grid);
        assert_eq!(grid[Vec2D::new(6, 6)], Cell::Down(0));
        assert_eq!(grid[Vec2D::new(6, 5)], Cell::Left(0));
        assert_eq!(grid[Vec2D::new(5, 5)], Cell::Tail(0));
        assert_eq!(grid.snake_body(0).unwrap().collect::<Vec<_>>(), snake.body);
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

    #[test]
    fn grid_flood_fill() {
        use super::*;
        let mut grid = Grid::new(11, 11);
        grid.flood_fill([(0, Vec2D::new(0, 0))].iter().cloned());
        println!("Filled {:?}", grid);

        let mut grid = Grid::new(11, 11);
        grid.flood_fill(
            [(0, Vec2D::new(0, 0)), (1, Vec2D::new(10, 10))]
                .iter()
                .cloned(),
        );
        println!("Filled {:?}", grid);

        let mut grid = Grid::new(11, 11);
        grid.flood_fill(
            [
                (0, Vec2D::new(0, 0)),
                (1, Vec2D::new(10, 10)),
                (2, Vec2D::new(5, 5)),
            ]
            .iter()
            .cloned(),
        );
        println!("Filled {:?}", grid);
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
            grid.add_snake(i as u8, snake);
        }
        let space = grid.space_after_move(0, &snakes);
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
            grid.add_snake(i as u8, snake);
        }
        let space = grid.space_after_move(0, &snakes);
        println!("space after move {:?}", space);
    }

    #[test]
    fn grid_step() {
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
        let mut grid = Grid::new(11, 11);
        for (i, snake) in snakes.iter().enumerate() {
            grid.add_snake(i as u8, snake);
        }
        println!("{:?}", grid);

        let start = Instant::now();
        loop {
            grid.step(0, Direction::Up);
            grid.step(0, Direction::Up);
            grid.step(0, Direction::Right);
            grid.step(0, Direction::Right);
            grid.step(0, Direction::Down);
            grid.step(0, Direction::Down);
            grid.step(0, Direction::Left);
            grid.step(0, Direction::Left);
            if !grid.snake_is_alive(0) {
                break;
            }
        }
        println!("Dead after {}us", (Instant::now() - start).as_nanos());
    }

    #[test]
    fn grid_step_random() {
        use super::*;
        use rand::seq::IteratorRandom;
        use std::time::{Instant, Duration};
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
        let mut grid = Grid::new(11, 11);

        for (i, snake) in snakes.iter().enumerate() {
            grid.add_snake(i as u8, snake);
        }

        for cell in grid.cells.iter_mut().choose_multiple(&mut rng, 20) {
            if !cell.is_occupied() {
                *cell = Cell::Food;
            }
        }

        println!("{:?}", grid);

        let start = Instant::now();
        let mut game = 0_usize;
        loop {
            let mut turn = 0;
            let mut grid = grid.clone();
            loop {
                for i in 0..4 {
                    let d = grid
                        .valid_moves(i)
                        .iter()
                        .enumerate()
                        .filter(|&(_, valid)| *valid)
                        .map(|v| Direction::from(v.0 as u8))
                        .choose(&mut rng)
                        .unwrap_or(Direction::Up);
                    grid.step(i, d);
                }

                // println!("{} {:?}", turn, grid);

                if !grid.snake_is_alive(0)
                    && !grid.snake_is_alive(1)
                    && !grid.snake_is_alive(1)
                    && !grid.snake_is_alive(1)
                {
                    break;
                }
                turn += 1;
            }
            println!("game {}: dead after {} turns", game, turn,);
            game += 1;

            if Instant::now() > start + Duration::from_millis(300) {
                break
            }
        }
        println!("Played {} games", game);
    }
}
