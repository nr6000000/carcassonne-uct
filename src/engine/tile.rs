use std::collections::HashMap;
use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::Hash;

use strum::{EnumCount, VariantArray};

use crate::engine::datastructures::tile_features::TileFeatures;
use crate::engine::fixed_tile::FixedTile;
use crate::engine::structures::RelStructureLinks;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tile {
    pub features: TileFeatures,
    pub cloister: bool,
    pub pennant: bool,
    pub cities_connected: bool,
    pub roads_connected: bool,
    pub city_road: bool,
}

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

impl Debug for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

#[repr(u8)]
#[derive(VariantArray, Copy, Clone, Debug)]
pub enum Rotation {
    Rot0 = 0,
    Rot1 = 1,
    Rot2 = 2,
    Rot3 = 3,
}

impl Tile {
    pub fn fix_rotation(
        &self, 
        rotation: &Rotation,
        structures: &HashMap<Tile, RelStructureLinks>,
    ) -> FixedTile {
        FixedTile { 
            edges: self.features.fix_rotation(rotation),
            structure_links: structures[self].to_abs(rotation),
            cloister: self.cloister,
            pennant: self.pennant,
            cities_connected: self.cities_connected,
            roads_connected: self.roads_connected,
            city_road: self.city_road,
        }
    }
}
