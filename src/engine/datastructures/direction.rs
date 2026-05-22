use strum::EnumIter;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter)]
pub enum Direction {
    North = 0,
    NorthEast = 1,
    East = 2,
    SouthEast = 3,
    South = 4,
    SouthWest = 5,
    West = 6,
    NorthWest = 7,
}

impl Direction {
    pub fn from_index(i: u8) -> Self {
        match i % 8 {
            0 => Self::North,
            1 => Self::NorthEast,
            2 => Self::East,
            3 => Self::SouthEast,
            4 => Self::South,
            5 => Self::SouthWest,
            6 => Self::West,
            _ => Self::NorthWest,
        }
    }

    pub fn index(self) -> u8 {
        self as u8
    }

    pub fn edges() -> [Direction; 4] {
        [
            Self::North,
            Self::East,
            Self::South,
            Self::West,
        ]
    }

    pub fn corners() -> [Direction; 4] {
        [
            Self::NorthEast,
            Self::SouthEast,
            Self::SouthWest,
            Self::NorthWest,
        ]
    }

    pub fn opposite(&self) -> Direction {
        match self {
            Self::North => Self::South,
            Self::NorthEast => Self::SouthWest,
            Self::East => Self::West,
            Self::SouthEast => Self::NorthWest,
            Self::South => Self::North,
            Self::SouthWest => Self::NorthEast,
            Self::West => Self::East,
            Self::NorthWest => Self::SouthEast,
        }
    }
}