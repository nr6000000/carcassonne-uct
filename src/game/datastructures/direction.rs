use strum::EnumIter;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter)]
pub enum Direction {
    North = 0,
    East = 2,
    South = 4,
    West = 6,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter)]
pub enum OrdinalDirection {
    North = 0,
    NorthEast = 1,
    East = 2,
    SouthEast = 3,
    South = 4,
    SouthWest = 5,
    West = 6,
    NorthWest = 7,
}

impl OrdinalDirection {
    pub fn opposite(&self) -> Self {
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

impl From<Direction> for OrdinalDirection {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::North => OrdinalDirection::North,
            Direction::East => OrdinalDirection::East,
            Direction::South => OrdinalDirection::South,
            Direction::West => OrdinalDirection::West,
        }
    }
}

impl From<&Direction> for &OrdinalDirection {
    fn from(dir: &Direction) -> Self {
        match dir {
            Direction::North => &OrdinalDirection::North,
            Direction::East => &OrdinalDirection::East,
            Direction::South => &OrdinalDirection::South,
            Direction::West => &OrdinalDirection::West,
        }
    }
}