#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RelDirection {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
}

impl RelDirection {
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
}