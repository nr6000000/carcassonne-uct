pub mod load;

use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

pub const TILE_SIZE: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TilePixel {
    Nothing,
    Blockade,
    Field,
    Road,
    Cloister,
    City,
    PennantCity,
}

impl Default for TilePixel {
    fn default() -> Self {
        Self::Nothing
    }
}

impl Display for TilePixel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TilePixel::Nothing => write!(f, "⦻⦻")?,
            TilePixel::Blockade => write!(f, "██")?,
            TilePixel::Field => write!(f, "··")?,
            TilePixel::Road => write!(f, "░░")?,
            TilePixel::Cloister => write!(f, "✝⌂")?,
            TilePixel::City => write!(f, "▒▒")?,
            TilePixel::PennantCity => write!(f, "▓▓")?,
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TileSet {
    pub name: String,
    pub starting_tile: String,
    pub tiles: HashMap<String, [[TilePixel; TILE_SIZE]; TILE_SIZE]>,
    pub tile_numbers: HashMap<String, u32>,
}