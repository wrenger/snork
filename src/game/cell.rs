use crate::env::{Direction, Vec2D};

/// Represents a single tile of the board
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Cell {
    Free,
    Food,
    /// Not occupied but owned
    Owned(u8),
    /// Next snake body part is at y+1
    Up(u8),
    /// Next snake body part is at x+1
    Right(u8),
    /// Next snake body part is at y-1
    Down(u8),
    /// Next snake body part is at x-1
    Left(u8),
    /// Last snake body part
    Tail(u8),
    /// When eating food
    TailDouble(u8),
    /// Start configuration
    TailTriple(u8),
}

impl Cell {
    pub fn snake_body(snake: u8, p: Vec2D, next: Option<Vec2D>) -> Cell {
        if let Some(next) = next {
            if p != next {
                return match Direction::from(next - p) {
                    Direction::Up => Cell::Up(snake),
                    Direction::Right => Cell::Right(snake),
                    Direction::Down => Cell::Down(snake),
                    Direction::Left => Cell::Left(snake),
                };
            }
        }
        Cell::Tail(snake)
    }

    pub fn is_occupied(&self) -> bool {
        !matches!(self, Cell::Free | Cell::Food | Cell::Owned(_))
    }
    pub fn is_owned_or_occupied(&self) -> bool {
        !matches!(self, Cell::Free | Cell::Food)
    }
    pub fn is_owned_by(&self, snake: u8) -> bool {
        match self {
            Cell::Owned(i) => snake == *i,
            _ => false,
        }
    }

    pub fn get_snake(&self) -> Option<u8> {
        match self {
            Cell::Owned(i)
            | Cell::Up(i)
            | Cell::Right(i)
            | Cell::Down(i)
            | Cell::Left(i)
            | Cell::Tail(i)
            | Cell::TailDouble(i)
            | Cell::TailTriple(i) => Some(*i),
            _ => None,
        }
    }
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
            Cell::Food => write!(f, "*_"),
            Cell::Owned(i) => write!(f, "_{}", i),
            Cell::Up(i) => write!(f, "^{}", i),
            Cell::Right(i) => write!(f, ">{}", i),
            Cell::Down(i) => write!(f, "v{}", i),
            Cell::Left(i) => write!(f, "<{}", i),
            Cell::Tail(i) => write!(f, ".{}", i),
            Cell::TailDouble(i) => write!(f, "-{}", i),
            Cell::TailTriple(i) => write!(f, "={}", i),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_snake_body() {
        use super::*;

        println!("sizeof Cell: {}", std::mem::size_of::<Cell>());

        assert_eq!(Cell::snake_body(0, Vec2D::new(0, 0), None), Cell::Tail(0));
        assert_eq!(Cell::snake_body(0, Vec2D::new(0, 0), Some(Vec2D::new(1, 0))), Cell::Right(0));
        assert_eq!(Cell::snake_body(0, Vec2D::new(1, 0), Some(Vec2D::new(0, 0))), Cell::Left(0));
        assert_eq!(Cell::snake_body(0, Vec2D::new(0, 0), Some(Vec2D::new(0, 1))), Cell::Up(0));
        assert_eq!(Cell::snake_body(0, Vec2D::new(0, 1), Some(Vec2D::new(0, 0))), Cell::Down(0));
    }
}
