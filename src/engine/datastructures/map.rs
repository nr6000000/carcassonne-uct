use std::{collections::{HashMap, HashSet}, fmt::{Display, Error}, iter, ops::{Index as IndexTrait, IndexMut}};
use std::fmt::Write;

use itertools::Itertools;
use crate::engine::{datastructures::index::Index, game::PlayerId};

const GROWTH_FACTOR: usize = 2;

pub struct Map<T> {
    data: Vec<T>,
    size: usize,
}

impl<T: Default + Copy> Map<T> {
    pub fn new(size: usize) -> Self {
        let vec_size = size*size;
        let data = vec![T::default(); vec_size];

        Self { 
            data,
            size,
        }
    }

    pub fn insert(&mut self, row: isize, column: isize, el: T) {
        self[(row, column)] = el;
    }

    fn grow(&mut self) {
        let new_size = self.size*GROWTH_FACTOR;
        let to_add = new_size-self.size;

// Ensure center stays in the same place
// Like this:
// ....
// .###
// .###
// .###

// ......
// ......
// ..###.
// ..###.
// ..###.
// ......

// .....
// .##..
// .##..
// .....
// .....

        let left_pad = new_size/2 - self.size/2;
        let right_pad = to_add - left_pad;

        let new_data = iter::empty()
            .chain(vec![T::default(); new_size*left_pad + left_pad])
            .chain(
                self.data
                    .chunks(self.size)
                    .into_iter()
                    .intersperse(&vec![T::default(); to_add])
                    .flatten()
                    .copied()
                )
            .chain(vec![T::default(); new_size*right_pad + right_pad])
            .collect();

        self.data = new_data;
        self.size = new_size;
    }
}

impl<T: Default + Eq + Display> Map<T> {
    pub fn to_display_string(
        &self, 
        followers: Option<&HashMap<PlayerId, HashSet<Index>>>
    ) -> Result<String, Error> {
        let mut buf = String::new();

        let (min_x, min_y, max_x, max_y) = (self.min_idx()..self.max_idx()+1)
            .cartesian_product(self.min_idx()..self.max_idx()+1)
            .fold(
                (isize::MAX, isize::MAX, isize::MIN, isize::MIN), 
                |(min_x, min_y, max_x, max_y), (x, y)| {
                    if self[Index{x, y}] != T::default() {
                        (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y))
                    } else {
                        (min_x, min_y, max_x, max_y)
                    }
                }
            );

        if min_x == isize::MAX && min_y == isize::MAX
            && max_x == isize::MIN && max_y == isize::MIN 
        {
            writeln!(buf, "empty")?;
        }

        write!(buf, "  ")?;
        for column in min_x-1..max_x+2 {
            write!(buf, "{}", if column == 0 {"00"} else {"  "});
        }
        write!(buf, "\n")?;

        let flat_followers: Vec<&Index> = if let Some(followers) = followers {
            followers.values().flatten().collect()
        } else {
            Vec::new()
        };

        for row in min_y-1..max_y+2 {
            write!(buf, "{}", if row == 0 {"00"} else {"  "})?;
            for column in min_x-1..max_x+2 {
                write!(buf, "{}", self[(row, column)])?;
                if flat_followers.contains(&&Index{x: column, y: row}) {
                    buf.pop();
                    buf.push('🯅');
                }
            }

            write!(buf, "\n")?;
        }

        Ok(buf)
    }
}

impl<T> Map<T> {
    fn center(&self) -> usize {
        self.size / 2
    }

    fn min_idx(&self) -> isize {
        -(self.center() as isize)
    }

    fn max_idx(&self) -> isize {
        self.size as isize - self.center() as isize - 1
    }

    fn flat_index(&self, row: isize, column: isize) -> usize {
        let abs_row = (row + self.center() as isize) as usize;
        let abs_column = (column + self.center() as isize) as usize;

        abs_row*self.size + abs_column
    }
}

impl<T> IndexTrait<(isize, isize)> for Map<T> {
    type Output = T;

    fn index(&self, (row, column): (isize, isize)) -> &Self::Output {
        &self.data[self.flat_index(row, column)]
    }
}

impl<T: Default + Copy> IndexMut<(isize, isize)> for Map<T> {    
    fn index_mut(&mut self, (row, column): (isize, isize)) -> &mut Self::Output {
        while row <= self.min_idx() ||
            column <= self.min_idx() ||
            row >= self.max_idx() ||
            column >= self.max_idx()
        {
            self.grow();
        }

        let idx = self.flat_index(row, column);
        &mut self.data[idx]
    }
}

impl<T> IndexTrait<Index> for Map<T> {
    type Output = T;

    fn index(&self, place: Index) -> &Self::Output {
        &self[(place.y, place.x)]
    }
}

impl<T: Default + Copy> IndexMut<Index> for Map<T> {    
    fn index_mut(&mut self, place: Index) -> &mut Self::Output {
        &mut self[(place.y, place.x)]
    }
}

impl<T: Display + Default + Eq> Display for Map<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_display_string(None)?)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::TilePixel;

    use super::*;

    #[test]
    fn map_correct() {
        let mut map: Map<TilePixel> = Map::new(5);

        map[(0, 0)] = TilePixel::City;
        assert_eq!(map[(0, 0)], TilePixel::City);

        map[(1, 0)] = TilePixel::Cloister;
        assert_eq!(map[(1, 0)], TilePixel::Cloister);

        map[(-1, -1)] = TilePixel::Field;
        assert_eq!(map[(-1, -1)], TilePixel::Field);
    }

    #[test]
    fn map_correct_grow() {
        let mut map: Map<TilePixel> = Map::new(3);
        
        // Map always has sentinel edges
        map[(0, 0)] = TilePixel::City;
        assert_eq!(map[(0, 0)], TilePixel::City);

        map[(1, 0)] = TilePixel::Cloister;
        assert_eq!(map[(1, 0)], TilePixel::Cloister);
        
        map[(-1, -1)] = TilePixel::Field;
        assert_eq!(map[(-1, -1)], TilePixel::Field);
        
        // Ensure has grown
        assert!(map.size > 3);
        
        map[(-10, -10)] = TilePixel::PennantCity;
        assert_eq!(map[(-10, -10)], TilePixel::PennantCity);
        // Center stays at (0, 0)
        assert_eq!(map[(0, 0)], TilePixel::City);

        map[(99, 99)] = TilePixel::Road;
        assert_eq!(map[(99, 99)], TilePixel::Road);
        assert_eq!(map[(0, 0)], TilePixel::City);
    }
}
