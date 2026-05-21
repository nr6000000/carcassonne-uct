use std::fmt::{Display, Formatter};

use crate::engine::{datastructures::tile_features::TileFeatures, structure_links::StructureLinks, tile::{Feature, Tile}};

#[derive(Clone, Debug)]
pub struct FixedTile {
    pub north: Feature,
    pub east: Feature,
    pub south: Feature,
    pub west: Feature,
    pub structure_links: StructureLinks,
    pub cloister: bool,
    pub pennant: bool,
    pub cities_connected: bool,
    pub roads_connected: bool,
    pub city_road: bool,
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
            city_road: value.city_road,
        }
    }
}

impl Display for FixedTile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let center = match (self.cloister, self.pennant) {
            (true, false) => "✝",
            (false, true) => "🛡",
            _ => " ",
        };

        let disconnected = if !self.cities_connected || !self.roads_connected {
            '◯'
        } else {
            ' '
        };

        writeln!(f, "╔ {} ╗", self.north)?;
        writeln!(f, "{}{}{}{}", self.west, center, disconnected, self.east)?;
        writeln!(f, "╚ {} ╝", self.south)?;

        Ok(())
    }
}
