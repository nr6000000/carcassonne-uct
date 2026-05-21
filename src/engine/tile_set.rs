use std::collections::HashMap;

use crate::engine::{fixed_tile::FixedTile, structure_links::RelStructureLinks, tile::{Rotation, Tile}};

#[derive(Clone)]
pub struct TileSet {
    pub starting_tile: FixedTile,
    pub tiles: Vec<Tile>,
    pub structures: HashMap<Tile, RelStructureLinks>,
}

impl TileSet {
    pub fn new(
        starting_tile: Tile,
        tiles: Vec<Tile>,
    ) -> Self {
        let structures: HashMap<Tile, RelStructureLinks> = tiles
            .iter().copied()
            .map(|tile| (tile, RelStructureLinks::new(&tile)))
            .collect();

        Self { 
            starting_tile: starting_tile.fix_rotation(
                &Rotation::Rot0,
                &structures
            ),
            tiles, 
            structures,
        }
    }
}
