use itertools::Itertools;
use rapidhash::{RapidHashMap, RapidHashSet};
use strum::IntoEnumIterator;
use tileset_format::TilePixel;

use crate::game_logic::{Index, datastructures::direction::OrdinalDirection, flood_fill::flood_fill, game::{Game, Place, PlayerId, index_place, place_index}, tilepixel_ext::TilePixelExt};

#[derive(Debug)]
pub struct Structure {
    pub completed: bool,
    pub points: u32,
    pub followers_number: RapidHashMap<PlayerId, u32>,
    pub followers_idx: RapidHashSet<Index>,
    pub seed: Index,
}

impl Game {
    pub(crate) fn get_structures(&self, place: &Place, in_game: bool) -> Vec<Structure> {
        let index = place_index(place);

        let tile_indices = (0..5).cartesian_product(0..5)
            .map(|(x, y)| Index{x, y} + index);
        let mut not_visited: RapidHashSet<Index> = RapidHashSet::from_iter(tile_indices);

        let mut structures: Vec<Structure> = Vec::new();
        while let Some(seed) = not_visited.iter().next().copied() {
            not_visited.remove(&seed);
            let seed_pixel = self.map[seed];

            if TilePixel::scoring_tiles(&self.settings).contains(&seed_pixel) {
                let mut completed = true;
                let tiles = flood_fill(
                    seed, 
                    |idx| {
                        let current = self.map[idx];

                        // Flood fill algorithm queries for pixel that is not set
                        // which means the structure isnt fully connected
                        if current == TilePixel::Nothing {
                            completed = false;
                        }

                        current.connects(&seed_pixel)
                    }
                );
                not_visited.retain(|tile| !tiles.contains(tile));
                
                let mut followers_number = RapidHashMap::default();
                let mut followers_idx = RapidHashSet::default();
                let followers_iter = tiles.iter()
                    .filter_map(|idx| self.followers.get(&idx).map(|p| (p, idx)));    
                for (player, idx) in followers_iter {
                    *followers_number.entry(*player).or_default() += 1;
                    followers_idx.insert(*idx);
                }
                
                let points = if seed_pixel == TilePixel::Field {
                    // Obliczanie wyniku pola przy każdym ruchu jest trochę wolne.
                    // Jak już będziemy wiedzieli jak chcemy robić heurestyki
                    // to trzeba się będzie zastanowić jak to inteligentnie zrobić
                    if self.settings.calculate_move_score_field {
                        self.get_field_score(seed)
                    } else {
                        0
                    }
                } else {
                    self.get_structure_score(&tiles, in_game)
                };

                // Real cloister completenes test
                // TODO: Better flow - setting wrong completed and
                // then chnaging it is confusing
                if seed_pixel == TilePixel::Cloister {
                    completed = points == 9;
                }

                structures.push(Structure {
                    completed,
                    points,
                    followers_number,
                    followers_idx,
                    seed,
                });
            }
        }

        // Add neighbouring cloisters to check if placing this tile 
        // has completed it
        let cloisters = OrdinalDirection::iter()
            .map(|dir| place.neighbour(&dir))
            .filter_map(|place| {
                let upper_left = place_index(&place);
                // HARDCODED: Cloisters are always in the center
                // May not be true
                let idx = upper_left + Index{x: 2, y: 2};
                if self.map[idx] == TilePixel::Cloister {
                    if self.followers.contains_key(&idx) {
                        let score = self.get_structure_score(&RapidHashSet::from_iter([idx]), in_game);
                        if score == 9 {
                            return Some(Structure {
                                completed: true,
                                points: 9,
                                followers_number: RapidHashMap::from_iter([
                                    (self.followers[&idx], 1)
                                ]),
                                followers_idx: RapidHashSet::from_iter([idx]),
                                seed: idx,
                            })
                        }
                    }
                }

                None
            });
        structures.extend(cloisters);

        structures
    }

    fn get_field_score(&self, seed: Index) -> u32 {
        let mut cities = RapidHashSet::default();
        let mut followers_number: RapidHashMap<PlayerId, u32> = RapidHashMap::default();
        let mut followers_idx = RapidHashSet::default();
        flood_fill(
            seed, 
            |idx| {
                let current = self.map[idx];

                // Flood fill algorithm queries for pixel that is not set
                // which means the structure isnt fully connected
                if current == TilePixel::City || current == TilePixel::PennantCity {
                    cities.insert(idx);
                }

                if let Some(player) = self.followers.get(&idx) {
                    *followers_number.entry(*player).or_default() += 1;
                    followers_idx.insert(idx);
                }

                current.connects(&TilePixel::Field)
            },
        );

        let mut score = 0;

        while let Some(city_seed) = cities.iter().next() {
            let mut completed = true;
            flood_fill(
                *city_seed, 
                |idx| {
                    let current = self.map[idx];
                    cities.remove(&idx);

                    // Flood fill algorithm queries for pixel that is not set
                    // which means the structure isnt fully connected
                    if current == TilePixel::Nothing {
                        completed = false;
                    }

                    current.connects(&TilePixel::City)
                },
            );

            if completed {
                // TODO: Odmagicnumberować
                score += 3;
            }
        }

        score
    }

    fn get_structure_score(&self, tiles: &RapidHashSet<Index>, in_game: bool) -> u32 {
        let scoring_places = tiles.iter()
            .map(|idx| (self.map[*idx], index_place(idx)))
            .unique();

        scoring_places
            .map(|(pixel, place)| {
                match pixel {
                    TilePixel::Road | TilePixel::City | TilePixel::PennantCity => 
                        pixel.score(in_game),
                    TilePixel::Cloister => {
                        let score = self.neighbour_count(&place) + 1;
                        score
                    },
                    _ => 0,
                }
            })
            .sum()
    }
}