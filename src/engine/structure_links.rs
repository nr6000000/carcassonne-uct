use heapless::{Vec as ArrayVec, index_map::FnvIndexMap, index_set::FnvIndexSet};

use crate::engine::{datastructures::{direction::Direction, rel_direction::RelDirection}, tile::{Feature, Rotation, Tile}};

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
pub struct RelStructureLinks {
    structures: FnvIndexMap<usize, Structure, 16>,
    internal_connections: FnvIndexMap<RelDirection, usize, 8>,
    external_connections: FnvIndexMap<usize, FnvIndexSet<RelDirection, 4>, 8>,
    cloister: Option<usize>,
}

impl RelStructureLinks {
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

                if features.get_feature(&dir.cw_neighbour_edge()) == &Feature::Road ||
                    (
                        features.get_feature(&dir.cw_neighbour_edge()) == &Feature::City &&
                        tile.city_road
                    )
                {
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

        RelStructureLinks { 
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    
    use crate::engine::{datastructures::{rel_direction::RelDirection, tile_features::TileFeatures}, tile::{Feature, Tile}};

    use super::*;
    
    #[test]
    fn structures_gen_crfr() {
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
            city_road: false,
        };

        let structure_links = RelStructureLinks::new(&tile);
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
            city_road: false,
        };

        let structure_links = RelStructureLinks::new(&tile);
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

    #[test]
    fn structures_gen_cccr() {
        // CCCR
        let tile = Tile { 
            features: TileFeatures::new(
                Feature::City,
                Feature::City,
                Feature::City,
                Feature::Road,
            ), 
            cloister: false, 
            pennant: true, 
            cities_connected: true, 
            roads_connected: false, 
            city_road: true,
        };

        let structure_links = RelStructureLinks::new(&tile);
        assert_eq!(structure_links.structures.len(), 4);
        assert_eq!(
            structure_links.get_structures()
                .filter(|s| s.structure_type == StructureType::Feature(Feature::City))
                .count(), 
            1
        );
        assert_eq!(
            structure_links.get_structures()
                .filter(|s| s.structure_type == StructureType::Feature(Feature::Road))
                .count(), 
            1
        );
        assert_eq!(
            structure_links.get_structures()
                .filter(|s| s.structure_type == StructureType::Feature(Feature::Field))
                .count(), 
            2
        );
    }

        #[test]
    fn structures_gen_ccrr() {
        // CCCR
        let tile = Tile { 
            features: TileFeatures::new(
                Feature::City,
                Feature::City,
                Feature::Road,
                Feature::Road,
            ), 
            cloister: false, 
            pennant: false, 
            cities_connected: true, 
            roads_connected: true, 
            city_road: false,
        };

        let structure_links = RelStructureLinks::new(&tile);
        assert_eq!(structure_links.structures.len(), 4);
        assert_eq!(
            structure_links.get_structures()
                .filter(|s| s.structure_type == StructureType::Feature(Feature::City))
                .count(), 
            1
        );
        assert_eq!(
            structure_links.get_structures()
                .filter(|s| s.structure_type == StructureType::Feature(Feature::Road))
                .count(), 
            1
        );
        assert_eq!(
            structure_links.get_structures()
                .filter(|s| s.structure_type == StructureType::Feature(Feature::Field))
                .count(), 
            2
        );
    }
}

#[derive(Debug, Clone)]
pub struct StructureLinks {
    structures: FnvIndexMap<usize, Structure, 16>,
    internal_connections: FnvIndexMap<Direction, usize, 8>,
    external_connections: FnvIndexMap<usize, FnvIndexSet<Direction, 4>, 8>,
    cloister: Option<usize>,
}

impl RelStructureLinks {
    pub fn to_abs(&self, rot: &Rotation) -> StructureLinks {
        let abs_internal: FnvIndexMap<Direction, usize, 8> = self.internal_connections
            .iter()
            .map(|(&dir, &id)| (dir.to_abs(rot), id))
            .collect();

        let abs_external: FnvIndexMap<usize, FnvIndexSet<Direction, 4>, 8> = self.external_connections
            .iter()
            .map(|(&id, dirs)| 
                (
                    id, 
                    dirs.iter()
                        .map(|&dir| dir.to_abs(rot))
                        .collect()
                )
            )
            .collect();

        StructureLinks { 
            structures: self.structures.clone(), 
            internal_connections: abs_internal, 
            external_connections: abs_external, 
            cloister: self.cloister,
        }
    }
}