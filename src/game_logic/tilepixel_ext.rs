use tileset_format::{TilePixel::{self, Blockade, City, Cloister, Field, Nothing, PennantCity, Road}};

pub trait TilePixelExt {
    /// Whether two tiles can be placed next to each other
    /// if they have edges with these pixels.
    fn fits(&self, other: &TilePixel) -> bool;

    /// Whether two pixels connect together to form a tructure.
    fn connects(&self, other: &TilePixel) -> bool;

    // TilePixel value in scoring
    fn score(&self, in_game: bool) -> u32;

    /// TilePixels that form scoring structures 
    fn scoring_tiles() -> &'static[TilePixel];
}

const SCORING_TILES: [TilePixel; 5] = [
    Field,
    Road,
    Cloister,
    City,
    PennantCity,
];

impl TilePixelExt for TilePixel {
    fn fits(&self, other: &TilePixel) -> bool {
        match (self, other) {
            (Nothing, _) => true,
            (_, Nothing) => true,
            (pixel1, pixel2) if pixel1 == pixel2 => true,
            (Blockade, _) => true,
            (_, Blockade) => true,
            (City, PennantCity) => true,
            (PennantCity, City) => true,
            _ => false,
        }
    }

    fn connects(&self, other: &TilePixel) -> bool {
        match (self, other) {
            (pixel1, pixel2) if pixel1 == pixel2 => true,
            (City, PennantCity) => true,
            (PennantCity, City) => true,
            _ => false,
        }
    }

    fn score(&self, in_game: bool) -> u32 {
        match (self, in_game) {
            (Road, true) => 1,
            (Road, false) => 1,
            (City, true) => 2,
            (City, false) => 1,
            (PennantCity, true) => 4,
            (PennantCity, false) => 2,
            _ => 0
        }
    }
    
    fn scoring_tiles() -> &'static[TilePixel] {
        &SCORING_TILES
    }
}