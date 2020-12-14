use std::cmp::Reverse;
use std::f64;
use std::ops::Index;
use std::ops::IndexMut;

use crate::env::*;

pub const BOARD_FREE: i8 = -1;
pub const BOARD_OBSTACLE: i8 = -2;
pub const BOARD_TMP: i8 = -3;
pub const BOARD_FOOD: i8 = -4;

const BOARD_SIZE: usize = 11;

pub struct Grid {
    cells: [[i8; BOARD_SIZE]; BOARD_SIZE],
}

impl Grid {
    pub fn new() -> Grid {
        Grid {
            cells: [[-1; BOARD_SIZE]; BOARD_SIZE],
        }
    }

    pub fn filled(food: &[Vec2D], snakes: &[CSnake]) -> Grid {
        let mut board = Grid::new();

        for &p in food {
            board[p] = BOARD_FOOD;
        }

        for snake in snakes {
            for &p in &snake.body {
                board[p] = BOARD_OBSTACLE;
            }
        }

        board
    }

    pub fn avaliable(&self, p: Vec2D) -> bool {
        self.has(p) && self[p] != BOARD_OBSTACLE
    }

    pub fn has(&self, p: Vec2D) -> bool {
        0 <= p.x && p.x < BOARD_SIZE as _ && 0 <= p.y && p.y < BOARD_SIZE as _
    }

    pub fn flood_fill(&mut self, snakes: &[CSnake]) {}

    pub fn a_star(&self, start: Vec2D, target: Vec2D) -> Option<Vec<Vec2D>> {
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
                let neighbor_cost = cost + 1.0;

                if self.avaliable(neighbor) {
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
}

impl Index<Vec2D> for Grid {
    type Output = i8;

    fn index(&self, p: Vec2D) -> &Self::Output {
        &self.cells[p.y as usize][p.x as usize]
    }
}

impl IndexMut<Vec2D> for Grid {
    fn index_mut(&mut self, p: Vec2D) -> &mut Self::Output {
        &mut self.cells[p.y as usize][p.x as usize]
    }
}

pub struct CSnake {
    pub id: usize,
    pub body: Vec<Vec2D>,
}

impl CSnake {
    pub fn new(id: usize, body: Vec<Vec2D>) -> CSnake {
        CSnake { id, body }
    }

    pub fn head(&self) -> Vec2D {
        self.body[0]
    }
}

impl PartialEq for CSnake {
    fn eq(&self, s: &CSnake) -> bool {
        self.id == s.id
    }
}
