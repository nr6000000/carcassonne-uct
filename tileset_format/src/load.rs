use std::{collections::HashMap, fmt, fs, path::Path};

use serde::Deserialize;
use toml::Table;

use crate::{TILE_SIZE, TilePixel, TileSet};

#[derive(Debug, Deserialize)]
struct TileSetConfig {
    name: String,
    starting_tile: String,

    #[serde(rename = "tile-filenames")]
    tile_filenames: Table,

    #[serde(rename = "tile-numbers")]
    tile_numbers: Table,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    path: String,
}

impl ParseError {
    fn new(path: &Path) -> Self {
        Self {
            path: path.to_string_lossy().into()
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad format: {:?}", self.path)
    }
}

fn parse_tile(path: &Path) -> Result<[[TilePixel; TILE_SIZE]; TILE_SIZE], ParseError> {
    let data = fs::read_to_string(path).unwrap();
    let mut data_iter = data.chars();

    let mut pixels = [[TilePixel::Nothing; TILE_SIZE]; TILE_SIZE];

    for row in 0..TILE_SIZE {
        for column in 0..TILE_SIZE {
            match (
                data_iter.next().ok_or(ParseError::new(&path))?, 
                data_iter.next().ok_or(ParseError::new(&path))?, 
            ) {
                ('·', '·') => pixels[row][column] = TilePixel::Field,
                ('░', '░') => pixels[row][column] = TilePixel::Road,
                ('▒', '▒') => pixels[row][column] = TilePixel::City,
                ('▓', '▓') => pixels[row][column] = TilePixel::PennantCity,
                ('█', '█') => pixels[row][column] = TilePixel::Blockade,
                ('✝', '⌂') => pixels[row][column] = TilePixel::Cloister,
                _ => Err(ParseError::new(&path))?,
            }
        }

        data_iter.next()
            .filter(|c| *c == '\n')
            .ok_or(ParseError::new(&path))?;
    }

    Ok(pixels)
}

pub fn parse_tilesets(path: &Path) -> Result<TileSet, ParseError> {
    let config_str = fs::read_to_string(path.join("tileset.toml")).unwrap();
    let config: TileSetConfig = toml::from_str(&config_str).unwrap();

    let starting_tile = config.starting_tile;
    let mut tiles: HashMap<String, [[TilePixel; TILE_SIZE]; TILE_SIZE]> = HashMap::new();
    let mut tile_numbers: HashMap<String, u32> = HashMap::new();

    for (tile_name, filename) in config.tile_filenames {
        let file_path = path.join(filename.as_str().unwrap());
        let tile = parse_tile(file_path.as_path())?;
        tiles.insert(tile_name.clone(), tile);
    }

    for (tile_name, tile_number) in config.tile_numbers {
        let tile_number = tile_number
            .as_integer().unwrap()
            .try_into().unwrap();
        tile_numbers.insert(tile_name.clone(), tile_number);
    }

    Ok(TileSet {
        name: config.name,
        starting_tile,
        tiles,
        tile_numbers,
    })
}