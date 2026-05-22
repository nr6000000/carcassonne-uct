use std::fmt::{Display, Formatter};

use heapless::index_map::FnvIndexMap;

use crate::engine::{datastructures::{direction::Direction, tile_features::TileFeatures}, structure_links::StructureLinks, tile::{Feature, Tile}};

#[derive(Clone, Debug)]
pub struct FixedTile {
    pub edges: FnvIndexMap<Direction, Feature, 4>,
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
            features: TileFeatures::from_iter(value.edges.values().copied()),
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

        writeln!(f, "╔ {} ╗", self.edges[&Direction::North])?;
        writeln!(
            f, 
            "{}{}{}{}", 
            self.edges[&Direction::West], 
            center, 
            disconnected, 
            self.edges[&Direction::East]
        )?;
        writeln!(f, "╚ {} ╝", self.edges[&Direction::South])?;

        Ok(())
    }
}
