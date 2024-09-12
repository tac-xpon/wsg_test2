#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub enum Direction {
    Up = 0,
    Right,
    Down,
    Left,
}

impl Default for Direction {
    fn default() -> Self {
        Self::Up
    }
}

#[allow(dead_code)]
impl Direction {
    pub fn turn_right(&self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        }
    }

    pub fn turn_left(&self) -> Self {
        match self {
            Self::Up => Self::Left,
            Self::Right => Self::Up,
            Self::Down => Self::Right,
            Self::Left => Self::Down,
        }
    }
}
