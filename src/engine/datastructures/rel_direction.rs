use crate::engine::{datastructures::direction::Direction, tile::Rotation};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RelDirection {
    Up = 0,
    UpRight = 1,
    Right = 2,
    DownRight = 3,
    Down = 4,
    DownLeft = 5,
    Left = 6,
    UpLeft = 7,
}

impl RelDirection {
    pub fn from_index(i: u8) -> Self {
        match i % 8 {
            0 => Self::Up,
            1 => Self::UpRight,
            2 => Self::Right,
            3 => Self::DownRight,
            4 => Self::Down,
            5 => Self::DownLeft,
            6 => Self::Left,
            _ => Self::UpLeft,
        }
    }

    pub fn index(self) -> u8 {
        self as u8
    }

    pub fn edges() -> [RelDirection; 4] {
        [
            RelDirection::Up,
            RelDirection::Right,
            RelDirection::Down,
            RelDirection::Left,
        ]
    }

    pub fn corners() -> [RelDirection; 4] {
        [
            RelDirection::UpRight,
            RelDirection::DownRight,
            RelDirection::DownLeft,
            RelDirection::UpLeft,
        ]
    }

    pub fn cw_neighbour_edge(&self) -> RelDirection {
        match self {
            RelDirection::Up => RelDirection::Right,
            RelDirection::UpRight => RelDirection::Right,
            RelDirection::Right => RelDirection::Down,
            RelDirection::DownRight => RelDirection::Down,
            RelDirection::Down => RelDirection::Left,
            RelDirection::DownLeft => RelDirection::Left,
            RelDirection::Left => RelDirection::Up,
            RelDirection::UpLeft => RelDirection::Up,
        }
    }

    pub fn ccw_neighbour_edge(&self) -> RelDirection {
        match self {
            RelDirection::Up => RelDirection::Left,
            RelDirection::UpRight => RelDirection::Up,
            RelDirection::Right => RelDirection::Up,
            RelDirection::DownRight => RelDirection::Right,
            RelDirection::Down => RelDirection::Right,
            RelDirection::DownLeft => RelDirection::Down,
            RelDirection::Left => RelDirection::Down,
            RelDirection::UpLeft => RelDirection::Left,
        }
    }

    pub fn cw_neighbour(&self) -> RelDirection {
        match self {
            RelDirection::Up => RelDirection::Right,
            RelDirection::UpRight => RelDirection::DownRight,
            RelDirection::Right => RelDirection::Down,
            RelDirection::DownRight => RelDirection::DownLeft,
            RelDirection::Down => RelDirection::Left,
            RelDirection::DownLeft => RelDirection::UpLeft,
            RelDirection::Left => RelDirection::Up,
            RelDirection::UpLeft => RelDirection::UpRight,
        }
    }

    pub fn to_abs(&self, rot: &Rotation) -> Direction {
        let rot_amount = *rot as u8;
        Direction::from_index(self.index() + rot_amount*2)
    }
}