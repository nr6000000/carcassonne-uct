use std::ops::Index as Index;

use tileset_format::TILE_SIZE;

use crate::game_logic::{TilePixel, game::Rotation};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct TileId(pub u32);

#[derive(Clone, Copy, Debug)]
pub struct Tile(pub [[TilePixel; TILE_SIZE]; TILE_SIZE]);

impl Tile {
    pub const fn new(content: [[TilePixel; TILE_SIZE]; TILE_SIZE]) -> Self {
        Self(content)
    }
}

pub const NOTHING_TILE: Tile = Tile::new([[TilePixel::Nothing; TILE_SIZE]; TILE_SIZE]);

impl Index<(usize, usize, Rotation)> for Tile {
    type Output = TilePixel;

    fn index(&self, (y, x, rotation): (usize, usize, Rotation)) -> &Self::Output {
        match rotation {
            Rotation::Rot0 => &self.0[y][x],
            Rotation::Rot1 => &self.0[x][TILE_SIZE-1-y],
            Rotation::Rot2 => &self.0[TILE_SIZE-1-y][TILE_SIZE-1-x],
            Rotation::Rot3 => &self.0[TILE_SIZE-1-x][y],
        }
    }
}