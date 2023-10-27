/// # Battlesnake API Types:
///
/// This module contains the types for (de)serializing the battlesnake game
/// requests.
///
/// See: https://docs.battlesnake.com/api
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};
use std::mem::size_of;
use std::ops::{Add, Neg, Sub};

pub const API_VERSION: &str = "1";

pub const HAZARD_DAMAGE: u8 = 15;

/// Position in the a 2D grid.
#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Vec2D {
    pub x: i16,
    pub y: i16,
}

const _: () = assert!(size_of::<Vec2D>() == 4);

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
        self.x.unsigned_abs() as u64 + self.y.unsigned_abs() as u64
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
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

/// The Direction is returned as part of a `MoveResponse`.
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
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::Up, Self::Right, Self::Down, Self::Left]
            .iter()
            .copied()
    }

    /// Returns the invert direction (eg. Left for Right)
    pub fn invert(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Self::Up
    }
}

impl From<Vec2D> for Direction {
    fn from(p: Vec2D) -> Self {
        if p.x < 0 {
            Self::Left
        } else if p.x > 0 {
            Self::Right
        } else if p.y < 0 {
            Self::Down
        } else {
            Self::Up
        }
    }
}

impl From<u8> for Direction {
    fn from(v: u8) -> Self {
        debug_assert!(v < 4, "Invalid direction");
        match v {
            1 => Self::Right,
            2 => Self::Down,
            3 => Self::Left,
            _ => Self::Up,
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
    /// The source of this game. (tournament, league, arena, challenge, custom)
    #[serde(default)]
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Ruleset {
    pub name: String,
    #[serde(default)]
    pub version: String,
}

/// Object describing a snake.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Battlesnake {
    pub id: String,
    pub name: String,
    pub health: u8,
    /// head to tail
    pub body: Vec<Vec2D>,
    #[serde(default)]
    pub shout: String,
}

impl PartialEq for Battlesnake {
    fn eq(&self, rhs: &Self) -> bool {
        self.id == rhs.id
    }
}

/// The game board is represented by a standard 2D grid, oriented with (0,0) in the bottom left.
/// The Y-Axis is positive in the up direction, and X-Axis is positive to the right.
///
/// Thus a board with width `w` and hight `h` is represented as shown below.
/// ```txt
/// (  0,h-1)    (w-1,h-1)
///     ^       .
///     |   .
/// (  0,  0) -> (w-1,  0)
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Board {
    /// The number of rows in the y-axis of the game board.
    pub height: usize,
    /// The number of columns in the x-axis of the game board.
    pub width: usize,
    /// Array of coordinates representing food locations on the game board.
    pub food: Vec<Vec2D>,
    /// Array of coordinates representing hazardous locations on the game board.
    /// These will only appear in some game modes.
    pub hazards: Vec<Vec2D>,
    /// Array of [Battlesnake] Objects representing all Battlesnakes remaining on
    /// the game board (including yourself if you haven't been eliminated).
    pub snakes: Vec<Battlesnake>,
}

/// The game data that is send on the `start`, `move` and `end` requests.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameRequest {
    /// [Game](GameData) Object describing the game being played.
    pub game: GameData,
    /// Turn number for this move.
    pub turn: usize,
    /// [Board] Object describing the game board on this turn.
    pub board: Board,
    /// Battlesnake Object describing your Battlesnake.
    pub you: Battlesnake,
}

impl fmt::Display for GameRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}: {} ({}@{})",
            self.game.source, self.game.ruleset.name, self.you.id, self.game.id
        )
    }
}

/// This response configures the battlesnake and its appearance.
#[derive(Serialize, Debug)]
pub struct IndexResponse<'a> {
    pub apiversion: &'a str,
    pub author: &'a str,
    pub color: &'a str,
    pub head: &'a str,
    pub tail: &'a str,
    pub version: &'a str,
}

impl<'a> IndexResponse<'a> {
    pub fn new(
        apiversion: &'a str,
        author: &'a str,
        color: &'a str,
        head: &'a str,
        tail: &'a str,
        version: &'a str,
    ) -> Self {
        Self {
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
#[must_use]
pub struct MoveResponse {
    pub r#move: Direction,
    pub shout: String,
}

impl MoveResponse {
    pub fn new(r#move: Direction) -> Self {
        Self {
            r#move,
            shout: String::new(),
        }
    }
    pub fn shout(r#move: Direction, shout: String) -> Self {
        Self { r#move, shout }
    }
}
