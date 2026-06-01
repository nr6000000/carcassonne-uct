use std::ops::{Add, Sub};

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug, Default)]
pub struct Index {
    pub x: isize,
    pub y: isize,
}

impl Add for Index {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {x: self.x + other.x, y: self.y + other.y}
    }
}

impl Sub for Index {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {x: self.x - other.x, y: self.y - other.y}
    }
}
