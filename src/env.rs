use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::{Add, Neg, Sub};

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Vec2D {
    pub x: i16,
    pub y: i16,
}

impl Vec2D {
    pub fn new(x: i16, y: i16) -> Vec2D {
        Vec2D { x, y }
    }

    pub fn apply(self, d: Direction) -> Vec2D {
        self + d.into()
    }

    pub fn manhattan(&self) -> u64 {
        self.x.abs() as u64 + self.y.abs() as u64
    }
}

impl From<(i16, i16)> for Vec2D {
    fn from(val: (i16, i16)) -> Self {
        Vec2D::new(val.0, val.1)
    }
}

impl From<Direction> for Vec2D {
    fn from(d: Direction) -> Self {
        match d {
            Direction::Up => Vec2D::new(0, 1),
            Direction::Right => Vec2D::new(1, 0),
            Direction::Down => Vec2D::new(0, -1),
            Direction::Left => Vec2D::new(-1, 0),
        }
    }
}

impl Add for Vec2D {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vec2D {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Neg for Vec2D {
    type Output = Vec2D;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Game {
    pub id: String,
    pub ruleset: Ruleset,
    pub timeout: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ruleset {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Snake {
    pub id: String,
    pub name: String,
    pub health: i64,
    pub body: Vec<Vec2D>,
    #[serde(default)]
    pub latency: Option<f64>,
    // #[serde(default)]
    // pub head: Vec2D,
    // #[serde(default)]
    // pub length: i64,
    #[serde(default)]
    pub shout: String,
}

impl PartialEq for Snake {
    fn eq(&self, rhs: &Snake) -> bool {
        self.id == rhs.id
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Board {
    pub height: usize,
    pub width: usize,
    pub food: Vec<Vec2D>,
    pub hazards: Vec<Vec2D>,
    pub snakes: Vec<Snake>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameRequest {
    pub game: Game,
    pub turn: i64,
    pub board: Board,
    pub you: Snake,
}

#[derive(Serialize)]
pub struct IndexResponse {
    pub apiversion: &'static str,
    pub author: &'static str,
    pub color: &'static str,
    pub head: &'static str,
    pub tail: &'static str,
}

impl IndexResponse {
    pub fn new(
        apiversion: &'static str,
        author: &'static str,
        color: &'static str,
        head: &'static str,
        tail: &'static str,
    ) -> IndexResponse {
        IndexResponse {
            apiversion,
            author,
            color,
            head,
            tail,
        }
    }
}

#[derive(Serialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    pub fn iter() -> impl Iterator<Item = Direction> {
        [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ]
        .iter()
        .copied()
    }
}

impl From<Vec2D> for Direction {
    fn from(p: Vec2D) -> Direction {
        if p.x < 0 {
            Direction::Left
        } else if p.x > 0 {
            Direction::Right
        } else if p.y < 0 {
            Direction::Down
        } else {
            Direction::Up
        }
    }
}

#[derive(Serialize)]
pub struct MoveResponse {
    pub r#move: Direction,
    pub shout: String,
}

impl MoveResponse {
    pub fn new(r#move: Direction) -> MoveResponse {
        MoveResponse {
            r#move,
            shout: String::new(),
        }
    }
    pub fn shout(r#move: Direction, shout: String) -> MoveResponse {
        MoveResponse { r#move, shout }
    }
}

impl Default for MoveResponse {
    fn default() -> MoveResponse {
        MoveResponse::new(Direction::Up)
    }
}
