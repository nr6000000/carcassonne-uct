use std::collections::HashMap;
use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::Hash;
use std::vec::Vec;

use heapless::Vec as ArrayVec;
use heapless::index_map::FnvIndexMap;
use heapless::index_set::FnvIndexSet;
use strum::{EnumCount, VariantArray};

use crate::engine::rel_direction::RelDirection;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StructureType {
    Feature(Feature),
    Cloister,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Structure {
    id: usize,
    structure_type: StructureType,
}

#[derive(Clone, Debug)]
pub struct StructureLinks {
    structures: FnvIndexMap<usize, Structure, 16>,
    internal_connections: FnvIndexMap<RelDirection, usize, 8>,
    external_connections: FnvIndexMap<usize, FnvIndexSet<RelDirection, 4>, 8>,
    cloister: Option<usize>,
}

impl StructureLinks {
    pub fn new(tile: &Tile) -> Self {
        let mut structures: FnvIndexMap<usize, Structure, 16> = FnvIndexMap::new();
        let mut internal_connections: FnvIndexMap<RelDirection, usize, 8> = FnvIndexMap::new();
        let mut external_connections: FnvIndexMap<usize, FnvIndexSet<RelDirection, 4>, 8> = FnvIndexMap::new();
        let mut cloister_ref: Option<usize> = None;
        let mut id_gen = 0..;

        let features = tile.features;

        // Roads and Cities
        for (feature, connected) in [
            (Feature::Road, tile.roads_connected), 
            (Feature::City, tile.cities_connected),
        ] {
            if connected {
                let mut current_id = 0;
                let mut created = false;

                for dir in RelDirection::edges() {
                    if features.get_feature(&dir) == &feature {
                        if !created {
                            current_id = id_gen.next().unwrap();
                            structures.insert(
                                current_id,
                                Structure {
                                    id: current_id,
                                    structure_type: StructureType::Feature(feature)
                                }
                            ).unwrap();

                            created = true;
                        }

                        internal_connections.insert(dir, current_id).unwrap();
                        external_connections.insert(
                            current_id, 
                            FnvIndexSet::from_iter([dir])
                        ).unwrap();
                    }
                }
            } else {
                for dir in RelDirection::edges() {
                    if features.get_feature(&dir) == &feature {
                        let current_id = id_gen.next().unwrap();
                        structures.insert(
                            current_id,
                            Structure {
                                id: current_id,
                                structure_type: StructureType::Feature(feature)
                            }
                        ).unwrap();

                        internal_connections.insert(dir, current_id).unwrap();
                        external_connections.insert(
                            current_id, 
                            FnvIndexSet::from_iter([dir])
                        ).unwrap();
                    }                    
                }
            }
        }

        // Corner Fields
        {
            let starting_dir = RelDirection::corners()
                .into_iter()
                .find(|dir| {
                    features.get_feature(&dir.ccw_neighbour_edge()) == &Feature::Road
                })
                .unwrap_or(RelDirection::UpRight);

            let mut filtered_dirs: ArrayVec<RelDirection, 4> = ArrayVec::new();
            let mut current = starting_dir;
            loop {
                if !(
                    features.get_feature(&current.cw_neighbour_edge()) == &Feature::City &&
                    features.get_feature(&current.ccw_neighbour_edge()) == &Feature::City
                ) {
                    filtered_dirs.push(current).unwrap();
                }

                current = current.cw_neighbour();
                if current == starting_dir {
                    break;
                }
            }
            
            let unmerged = filtered_dirs
                .iter()
                .map(|dir| {
                    let mut connections: FnvIndexSet<RelDirection, 4> = FnvIndexSet::new();
                    if features.get_feature(&dir.ccw_neighbour_edge()) != &Feature::City {
                        connections.insert(dir.ccw_neighbour_edge()).unwrap();
                    }

                    if features.get_feature(&dir.cw_neighbour_edge()) != &Feature::City {
                        connections.insert(dir.cw_neighbour_edge()).unwrap();
                    }

                    (dir, connections)
                });
            
            let mut new_slot = true;
            let mut current_id = 0;
            for (dir, connections) in unmerged {
                if new_slot {
                    current_id = id_gen.next().unwrap();
                    structures.insert(
                    current_id,
                    Structure { 
                            id: current_id, 
                            structure_type: StructureType::Feature(Feature::Field) 
                        }
                    ).unwrap();
                    external_connections.insert(current_id, FnvIndexSet::new()).unwrap();
                    new_slot = false;
                }

                internal_connections.insert(*dir, current_id).unwrap();
                external_connections[&current_id] = external_connections[&current_id]
                    .union(&connections)
                    .copied()
                    .collect();

                if features.get_feature(&dir.cw_neighbour_edge()) == &Feature::Road {
                    new_slot = true;
                }
            }
        }

        // Cloister
        if tile.cloister {
            let id = id_gen.next().unwrap();
            structures.insert(
                id,
                Structure { 
                    id, 
                    structure_type: StructureType::Cloister,
                }
            ).unwrap();
            cloister_ref = Some(id);
        }

        StructureLinks { 
            structures, 
            internal_connections, 
            external_connections, 
            cloister: cloister_ref 
        }
    }

    pub fn get_structures(&self) -> impl Iterator<Item = &Structure> {
        self.structures.values()
    }

    pub fn get_structure(&self, dir: &RelDirection) -> &Structure {
        let structure_id = self.internal_connections[dir];
        &self.structures[&structure_id]
    }

    pub fn get_cloister(&self) -> Option<&Structure> {
        self.cloister.map(|c| &self.structures[&c])
    }

    pub fn connects_to(&self, structure: &Structure) -> impl Iterator<Item = &RelDirection> {
        self.external_connections[&structure.id].iter()
    }

}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tile {
    pub features: TileFeatures,
    pub cloister: bool,
    pub pennant: bool,
    pub cities_connected: bool,
    pub roads_connected: bool,
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

#[derive(Clone, Debug)]
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

#[derive(Clone)]
pub struct TileSet {
    pub starting_tile: FixedTile,
    pub tiles: Vec<Tile>,
    pub structures: HashMap<Tile, StructureLinks>,
}

impl TileSet {
    pub fn new(
        starting_tile: FixedTile,
        tiles: Vec<Tile>,
    ) -> Self {
        let structures: HashMap<Tile, StructureLinks> = tiles
            .iter().copied()
            .map(|tile| (tile, StructureLinks::new(&tile)))
            .collect();

        Self { 
            starting_tile, 
            tiles, 
            structures,
        }
    }
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

#[derive(VariantArray, Copy, Clone, Debug)]
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    
    use super::*;
    
    #[test]
    fn structures_gen() {
        // CRFR
        let tile = Tile { 
            features: TileFeatures::new(
                Feature::City,
                Feature::Road,
                Feature::Field,
                Feature::Road,
            ), 
            cloister: false, 
            pennant: false, 
            cities_connected: true, 
            roads_connected: true, 
        };

        let structure_links = StructureLinks::new(&tile);
        let structures = structure_links.get_structures();
        assert_eq!(structures.count(), 4);

        // Correct edges
        assert_eq!(
            structure_links.get_structure(&RelDirection::Up).structure_type,
            StructureType::Feature(Feature::City),
        );
        assert_eq!(
            structure_links.get_structure(&RelDirection::Right).structure_type,
            StructureType::Feature(Feature::Road),
        );
        // assert_eq!(
        //     structure_links.get_structure(&RelDirection::Down).structure_type,
        //     StructureType::Nothing,
        // );
        assert_eq!(
            structure_links.get_structure(&RelDirection::Left).structure_type,
            StructureType::Feature(Feature::Road),
        );

        // Correct corners
        assert_eq!(
            structure_links.get_structure(&RelDirection::UpRight).structure_type,
            StructureType::Feature(Feature::Field),
        );
        assert_eq!(
            structure_links.get_structure(&RelDirection::DownRight).structure_type,
            StructureType::Feature(Feature::Field),
        );
        assert_eq!(
            structure_links.get_structure(&RelDirection::DownLeft).structure_type,
            StructureType::Feature(Feature::Field),
        );
        assert_eq!(
            structure_links.get_structure(&RelDirection::UpLeft).structure_type,
            StructureType::Feature(Feature::Field),
        );

        // Correct connections
        assert_eq!(
            structure_links.get_structure(&RelDirection::Left).id,
            structure_links.get_structure(&RelDirection::Right).id
        );
        assert_eq!(
            structure_links.get_structure(&RelDirection::DownLeft).id,
            structure_links.get_structure(&RelDirection::DownRight).id
        );
        assert_eq!(
            structure_links.get_structure(&RelDirection::UpLeft).id,
            structure_links.get_structure(&RelDirection::UpRight).id
        );
        assert_ne!(
            structure_links.get_structure(&RelDirection::UpLeft).id,
            structure_links.get_structure(&RelDirection::DownLeft).id
        );
        assert_ne!(
            structure_links.get_structure(&RelDirection::Up).id,
            structure_links.get_structure(&RelDirection::Left).id
        );

        // External connections
        let up_connections: HashSet<&RelDirection> = structure_links
            .connects_to(structure_links.get_structure(&RelDirection::Up))
            .collect();
        assert_eq!(up_connections, HashSet::from([&RelDirection::Up]));

        let upright_connections: HashSet<&RelDirection> = structure_links
            .connects_to(structure_links.get_structure(&RelDirection::UpRight))
            .collect();
        assert_eq!(upright_connections, HashSet::from([&RelDirection::Left, &RelDirection::Right]));

        let downleft_connections: HashSet<&RelDirection> = structure_links
            .connects_to(structure_links.get_structure(&RelDirection::DownLeft))
            .collect();
        assert_eq!(downleft_connections, HashSet::from([
            &RelDirection::Down, 
            &RelDirection::Left,
            &RelDirection::Right,
        ]));
    }

    #[test]
    fn structures_gen_cloister() {
        // FFFR_CLOISTER
        let tile = Tile { 
            features: TileFeatures::new(
                Feature::Field,
                Feature::Field,
                Feature::Field,
                Feature::Road,
            ), 
            cloister: true, 
            pennant: false, 
            cities_connected: true, 
            roads_connected: false, 
        };

        let structure_links = StructureLinks::new(&tile);
        let structures = structure_links.get_structures();
        assert_eq!(structures.count(), 3);

        assert_eq!(
            structure_links.get_structure(&RelDirection::Up).structure_type,
            StructureType::Feature(Feature::Road),
        );
        assert_eq!(
            structure_links.get_cloister().unwrap().structure_type,
            StructureType::Cloister,
        );
    }
}