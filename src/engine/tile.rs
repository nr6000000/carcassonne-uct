use std::fmt::{Debug, Display, Error, Formatter};

use strum::{EnumCount, VariantArray};

use crate::engine::tile_features::TileFeatures;

#[derive(PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord, EnumCount, Debug)]
pub enum Feature {
    City,
    Road,
    Field,
}

impl Display for Feature {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", match self {
            Feature::City => "🏰",//"C",
            Feature::Road => "〰️",//"R",
            Feature::Field => "🌾",//"F",
        })?;

        Ok(())
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Tile {
    pub features: TileFeatures,
    pub cloister: bool,
    pub pennant: bool,
    pub cities_connected: bool,
    pub roads_connected: bool,
}

impl Debug for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // f.debug_struct("Tile").field("features", &self.features).field("cloister", &self.cloister).field("pennant", &self.pennant).field("cities_connected", &self.cities_connected).field("roads_connected", &self.roads_connected).finish()
        write!(f, "{:?}", self.features)?;

        if self.cloister {
            write!(f, "_CLOISTER")?;
        }

        if self.pennant {
            write!(f, "_PENNANT")?;
        }

        if !self.cities_connected || !self.roads_connected {
            write!(f, "_DISCONNECTED")?;
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FixedTile {
    pub north: Feature,
    pub east: Feature,
    pub south: Feature,
    pub west: Feature,
    pub cloister: bool,
    pub pennant: bool,
    pub cities_connected: bool,
    pub roads_connected: bool,
}

pub trait TileSet {
    fn starting_tile(&self) -> FixedTile;
    fn tiles(&self) -> &[Tile];
}

impl From<FixedTile> for Tile {
    fn from(value: FixedTile) -> Self {
        Tile {
            features: TileFeatures::new(
                value.north, 
                value.east, 
                value.south, 
                value.west,
            ), 
            cloister: value.cloister,
            pennant: value.pennant,
            cities_connected: value.cities_connected,
            roads_connected: value.roads_connected,
        }
    }
}

#[derive(VariantArray, Copy, Clone)]
pub enum Rotation {
    Rot0 = 0,
    Rot1 = 1,
    Rot2 = 2,
    Rot3 = 3,
}

impl Tile {
    pub fn fix_rotation(&self, rotation: &Rotation) -> FixedTile {
        let unrotated = self.features.get();
        let rot = *rotation as usize;
        let rotated = [
            unrotated[rot % 4],
            unrotated[(rot+1) % 4],
            unrotated[(rot+2) % 4],
            unrotated[(rot+3) % 4],
        ];

        FixedTile { 
            north: rotated[0],
            east: rotated[1],
            south: rotated[2],
            west: rotated[3],
            cloister: self.cloister,
            pennant: self.pennant,
            cities_connected: self.cities_connected,
            roads_connected: self.roads_connected,
        }
    }
//     pub fn from_code(code: &str) -> Result<Tile, Box<dyn std::error::Error>> {
//         fn letter_to_feature(letter: u8) -> Result<Feature, Box<dyn std::error::Error>> {
//             match letter {
//                 b'C' => Ok(Feature::City),
//                 b'R' => Ok(Feature::Road),
//                 b'F' => Ok(Feature::Field),
//                 _ => Err("Błędny kod kafelka".into())
//             }
//         }

//         let mut parts = code.split("_");
//         let features = parts.next().unwrap().as_bytes();
//         if features.len() != 4 {
//             return Err("Błędna liczba krawędzi".into());
//         }
//         let north = letter_to_feature(features[0])?;
//         let east = letter_to_feature(features[1])?;
//         let south = letter_to_feature(features[2])?;
//         let west = letter_to_feature(features[3])?;
        
//         let mut cloister = false;
//         let mut pennant = false;
//         if let Some(modifier) = parts.next() {  
//             match modifier {
//                 "CLOISTER" => cloister = true,
//                 "PENNANT" => pennant = true,
//                 _ => (),
//             }
//         }

//         Ok(Tile{
//             north: north,
//             east: east,
//             south: south,
//             west: west,
//             cloister,
//             pennant,
//         })
//     }
}

impl Display for FixedTile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let center = match (self.cloister, self.pennant) {
            (true, false) => "✝",
            (false, true) => "🛡",
            _ => " ",
        };

        writeln!(f, "╔ {} ╗", self.north)?;
        writeln!(f, "{}{} {}", self.west, center, self.east)?;
        writeln!(f, "╚ {} ╝", self.south)?;

        Ok(())
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct Place {
    pub x: i32,
    pub y: i32,
}

impl Place {
    pub fn north(&self) -> Place {
        Place{x: self.x, y: self.y - 1}
    }

    pub fn northeast(&self) -> Place {
        Place{x: self.x + 1, y: self.y - 1}
    }

    pub fn east(&self) -> Place {
        Place{x: self.x + 1, y: self.y}
    }
   
    pub fn southeast(&self) -> Place {
        Place{x: self.x + 1, y: self.y + 1}
    }

    pub fn south(&self) -> Place {
        Place{x: self.x, y: self.y + 1}
    }
    
    pub fn southwest(&self) -> Place {
        Place{x: self.x - 1, y: self.y + 1}
    }

    pub fn west(&self) -> Place {
        Place{x: self.x - 1, y: self.y}
    }
    
    pub fn northwest(&self) -> Place {
        Place{x: self.x - 1, y: self.y - 1}
    }
}
