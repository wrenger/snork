/// # Battlesnake API Types:
///
/// This module contains the types for (de)serializing the battlesnake game
/// requests.
///
/// See: https://docs.battlesnake.com/references/api
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::Debug;
use std::ops::{Add, Neg, Sub};

pub const API_VERSION: &str = "1";

pub const HAZARD_DAMAGE: u8 = 15;

/// Position in the a 2D grid.
#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Vec2D {
    pub x: i16,
    pub y: i16,
}

#[inline(always)]
pub fn v2(x: i16, y: i16) -> Vec2D {
    Vec2D::new(x, y)
}

impl Vec2D {
    pub fn new(x: i16, y: i16) -> Vec2D {
        Vec2D { x, y }
    }

    pub fn apply(self, d: Direction) -> Vec2D {
        self + d.into()
    }

    /// Returns the manhattan distance to (0,0)
    pub fn manhattan(self) -> u64 {
        self.x.abs() as u64 + self.y.abs() as u64
    }

    /// Returns whether the vector is inside a rectangle from (0,0) to (width-1,height-1)
    pub fn within(self, width: usize, height: usize) -> bool {
        self.x >= 0 && self.x < width as _ && self.y >= 0 && self.y < height as _
    }
}

impl From<(i16, i16)> for Vec2D {
    fn from(val: (i16, i16)) -> Self {
        Vec2D::new(val.0, val.1)
    }
}

impl From<(usize, usize)> for Vec2D {
    fn from(val: (usize, usize)) -> Self {
        Vec2D::new(val.0 as _, val.1 as _)
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

/// The Direction is returned as part of a MoveResponse.
///
/// The Y-Axis is positive in the up direction, and X-Axis is positive to the right.
#[derive(Serialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum Direction {
    /// Positive Y
    Up,
    /// Positive X
    Right,
    /// Negative Y
    Down,
    /// Negative X
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

    /// Returns the invert direction (eg. Left for Right)
    pub fn invert(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Up
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
        debug_assert!(v < 4, "Invalid direction");
        match 0 {
            1 => Direction::Right,
            2 => Direction::Down,
            3 => Direction::Left,
            _ => Direction::Up,
        }
    }
}

/// Game Object describing the game being played.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GameData {
    /// A unique identifier for this Game.
    pub id: String,
    /// Information about the ruleset being used to run this game.
    #[serde(default)]
    pub ruleset: Ruleset,
    /// How much time your snake has to respond to requests for this Game in milliseconds.
    pub timeout: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Ruleset {
    pub name: String,
    #[serde(default)]
    pub version: String,
}

/// Object describing a snake.
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

/// The game board is represented by a standard 2D grid, oriented with (0,0) in the bottom left.
/// The Y-Axis is positive in the up direction, and X-Axis is positive to the right.
///
/// Thus a board with width `w` and hight `m` is represented as shown below.
/// ```txt
/// (  0,m-1)    (w-1,h-1)
///     ^       .
///     |   .
/// (  0,  0) -> (w-1,  0)
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Board {
    pub height: usize,
    pub width: usize,
    pub food: Vec<Vec2D>,
    pub hazards: Vec<Vec2D>,
    pub snakes: Vec<SnakeData>,
}

/// The game data that is send on the `start`, `move` and `end` requests.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameRequest {
    /// Game Object describing the game being played.
    pub game: GameData,
    /// Turn number for this move.
    pub turn: usize,
    /// Board Object describing the game board on this turn.
    pub board: Board,
    /// Battlesnake Object describing your Battlesnake.
    pub you: SnakeData,
}

/// This response configures the battlesnake and its appearance.
#[derive(Serialize, Debug)]
pub struct IndexResponse {
    pub apiversion: Cow<'static, str>,
    pub author: Cow<'static, str>,
    pub color: Cow<'static, str>,
    pub head: Cow<'static, str>,
    pub tail: Cow<'static, str>,
    pub version: Cow<'static, str>,
}

impl IndexResponse {
    pub fn new(
        apiversion: Cow<'static, str>,
        author: Cow<'static, str>,
        color: Cow<'static, str>,
        head: Cow<'static, str>,
        tail: Cow<'static, str>,
        version: Cow<'static, str>,
    ) -> IndexResponse {
        IndexResponse {
            apiversion,
            author,
            color,
            head,
            tail,
            version,
        }
    }
}

/// Game response with the direction in which a snake has decided to move.
#[derive(Serialize, Debug, Default)]
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
