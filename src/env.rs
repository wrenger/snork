use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::{Add, Neg, Sub};

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

impl From<u8> for Direction {
    fn from(v: u8) -> Direction {
        assert!(v < 4, "Invalid direction");
        unsafe { std::mem::transmute(v) }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GameData {
    pub id: String,
    #[serde(default)]
    pub ruleset: Ruleset,
    pub timeout: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Ruleset {
    pub name: String,
    #[serde(default)]
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnakeData {
    pub id: String,
    pub name: String,
    pub health: u8,
    /// head to tail
    pub body: Vec<Vec2D>,
    #[serde(default)]
    pub shout: String,
}

impl PartialEq for SnakeData {
    fn eq(&self, rhs: &SnakeData) -> bool {
        self.id == rhs.id
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Board {
    pub height: usize,
    pub width: usize,
    pub food: Vec<Vec2D>,
    pub hazards: Vec<Vec2D>,
    pub snakes: Vec<SnakeData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameRequest {
    pub game: GameData,
    pub turn: usize,
    pub board: Board,
    pub you: SnakeData,
}

#[derive(Serialize, Debug)]
pub struct IndexResponse {
    pub apiversion: &'static str,
    pub author: &'static str,
    pub color: String,
    pub head: String,
    pub tail: String,
}

impl IndexResponse {
    pub fn new(
        apiversion: &'static str,
        author: &'static str,
        color: String,
        head: String,
        tail: String,
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

#[derive(Serialize, Debug)]
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
