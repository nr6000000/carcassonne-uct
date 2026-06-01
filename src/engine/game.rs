use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
use std::fmt::Display;
use std::iter;

use heapless::{Vec as ArrayVec};
use itertools::{Itertools, structs};
use strum::{EnumIter, IntoEnumIterator};
use thiserror::Error;
use tileset_format::TilePixel::Nothing;
use tileset_format::{TILE_SIZE, TileSet};

use crate::engine::TilePixel;
use crate::engine::datastructures::direction::{Direction, OrdinalDirection};
use crate::engine::datastructures::index::Index;
use crate::engine::datastructures::map::{Map};
use crate::engine::datastructures::multi_hashset::MultiHashSet;
use crate::engine::flood_fill::flood_fill;
use crate::engine::tile::{NOTHING_TILE, Tile, TileId};
use crate::engine::tilepixel_ext::TilePixelExt;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct PlayerId(u32);

pub struct Game {
    move_num: u32,
    map: Map<TilePixel>,
    tiles: HashMap<TileId, Tile>,
    tiles_left: MultiHashSet<TileId>,
    free_places: HashSet<Place>,
    followers: HashMap<Index, PlayerId>,
    followers_left: HashMap<PlayerId, u32>,
    score: HashMap<PlayerId, u32>,
}

#[derive(Debug, EnumIter, Clone, Copy)]
pub enum Rotation {
    Rot0 = 0,
    Rot1 = 1,
    Rot2 = 2,
    Rot3 = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Place {
    x: isize,
    y: isize,
}

impl Place {
    pub fn neighbour(&self, dir: &OrdinalDirection) -> Place {
        match dir {
            OrdinalDirection::North => Place { x: self.x, y: self.y - 1 },
            OrdinalDirection::NorthEast => Place { x: self.x + 1, y: self.y - 1 },
            OrdinalDirection::East => Place { x: self.x + 1, y: self.y },
            OrdinalDirection::SouthEast => Place { x: self.x + 1, y: self.y + 1 },
            OrdinalDirection::South => Place { x: self.x, y: self.y + 1 },
            OrdinalDirection::SouthWest => Place { x: self.x - 1, y: self.y + 1 },
            OrdinalDirection::West => Place { x: self.x - 1, y: self.y },
            OrdinalDirection::NorthWest => Place { x: self.x - 1, y: self.y - 1 },
        }
    }
}

#[derive(Error, Debug)]
pub enum TileError {
    #[error("Pozycja zajęta")]
    PlaceOccupied,
    #[error("Sasiędzi nie pasują")]
    DoesntFit,
    #[error("Kafelek odłączony")]
    Disconnected,
    #[error("Ruch dla starej mapy")]
    StaleMove,
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    move_num: u32,
    place: Place,
    tile: TileId,
    rotation: Rotation,
    follower: Option<Index>,
    player: PlayerId,
}

impl Move {
    pub fn get_move_num(&self) -> u32 {
        self.move_num
    }
}

#[derive(Debug)]
struct Structure {
    completed: bool,
    points: u32,
    followers_number: HashMap<PlayerId, u32>,
    followers_idx: HashSet<Index>,
    seed: Index,
}

impl Game {
    pub fn new(tileset: &TileSet, number_players: u32, starting_followers: u32) -> Game {
        // To support perfect circle with A tiles we need size of approx:
        // A = pi*r^2
        // r = sqrt(A/pi)
        // d = sqrt(A/pi)*2
        // 
        // Add generous margin to support most games without realloc:
        // d*4 = sqrt(A/pi)*8
        let tileset_size = tileset.tiles.iter().count() as f32;
        let init_map_size = (tileset_size/PI).sqrt().ceil() as usize * 8 * TILE_SIZE;

        let mut tiles: HashMap<TileId, Tile> = HashMap::new();
        let mut tiles_left: MultiHashSet<TileId> = MultiHashSet::new();
        let mut starting_tile_id: Option<TileId> = None;

        let mut tile_id_gen = 0..;
        for tile_name in tileset.tiles.keys() {
            let id = TileId(tile_id_gen.next().unwrap());

            tiles.insert(id, Tile::new(tileset.tiles[tile_name]));
            tiles_left.set(id, tileset.tile_numbers[tile_name]);
            if tile_name == &tileset.starting_tile {
                starting_tile_id = Some(id);
            }
        }
        
        let starting_place = Place{x: 0, y: 0};

        let mut game = Game{
            move_num: 0,
            map: Map::new(init_map_size),
            tiles,
            tiles_left,
            free_places: HashSet::new(),
            followers_left: HashMap::from_iter(
                (0..number_players).map(|i| (PlayerId(i), starting_followers)),
            ),
            followers: HashMap::new(),
            score: HashMap::from_iter(
                (0..number_players).map(|i| (PlayerId(i), 0)),
            ),
        };

        let starting_tile = game.tiles[&starting_tile_id.unwrap()];
        game.copy_tile(&starting_tile, Rotation::Rot0, starting_place);

        game.tiles_left.take(&starting_tile_id.unwrap());
        game.free_places.extend([
            starting_place.neighbour(&Direction::North.into()),
            starting_place.neighbour(&Direction::East.into()),
            starting_place.neighbour(&Direction::South.into()),
            starting_place.neighbour(&Direction::West.into()),
        ]);
        
        game
    }

    pub fn get_players(&self) -> impl Iterator<Item = PlayerId> {
        self.followers_left.keys().copied()
    }

    fn place_index(&self, place: &Place) -> Index {
        Index {
            x: place.x*TILE_SIZE as isize,
            y: place.y*TILE_SIZE as isize,
        }
    }

    fn index_place(&self, index: &Index) -> Place {
        Place { 
            x: index.x.div_euclid(TILE_SIZE as isize), 
            y: index.y.div_euclid(TILE_SIZE as isize),
        }
    }

    fn place_occupied(&self, place: &Place) -> bool {
        self.map[self.place_index(place)] != TilePixel::Nothing
    }

    fn neighbour_count(&self, place: &Place) -> u32 {
        Direction::iter()
            .filter(|dir| self.place_occupied(&place.neighbour(dir.into())))
            .count() as u32
    }

    fn empty_neighbour_places(&self, place: &Place) -> ArrayVec<Place, 4> {
        Direction::iter()
            .map(|dir| place.neighbour(&dir.into()))
            .filter(|place| !self.place_occupied(place))
            .collect()
    }

    fn get_moves_placement(&self, player: PlayerId) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new(); 
        for tile_id in self.tiles_left.elements() {
            let tile = self.tiles[&tile_id];

            for place in self.free_places.iter() {
                for rotation in Rotation::iter() {
                    if self.check_tile(&tile, place, &rotation).is_ok() {
                        moves.push(Move {
                            move_num: self.move_num,
                            place: *place,
                            tile: *tile_id,
                            rotation,
                            follower: None,
                            player,
                        });
                    }
                }
            }
        }

        moves
    }

    fn get_moves_structures(&mut self, moves: Vec<Move>) -> Vec<Move> {
        moves.into_iter().flat_map(|mov| {
            let tile = self.tiles[&mov.tile];
            self.copy_tile(&tile, mov.rotation, mov.place);

            let structures = self.get_structures(&mov.place, true);
            let mut moves: ArrayVec<Move, 9> = ArrayVec::from_array([mov]);
            if self.followers_left[&mov.player] > 0 {
                moves.extend(
                    structures.iter()
                    .filter(|structure| structure.followers_number
                        .iter().all(|(_, &number)| number == 0)
                    )
                    .map(|structure| {
                        let mut new_mov = mov.clone();
                        new_mov.follower = Some(structure.seed);
                        new_mov
                    })
                );
            }
            
            self.copy_tile(&NOTHING_TILE, Rotation::Rot0, mov.place);
            moves
        })
        .collect()
    }

    pub fn get_moves(&mut self, player: PlayerId) -> Vec<Move> {
        let mut moves = self.get_moves_placement(player);
        moves = self.get_moves_structures(moves);
        moves
    }

    fn check_tile(
        &self, 
        tile: &Tile, 
        place: &Place,
        rotation: &Rotation,
    ) -> Result<(), TileError> {
        if !self.free_places.contains(place) {
            if self.place_occupied(place) {
                return Err(TileError::PlaceOccupied);
            }

            return Err(TileError::Disconnected);
        }

        for dir in Direction::iter() {
            for i in 0..TILE_SIZE {
                let other = self.place_index(&place.neighbour(&dir.into()));
                let stride = match dir {
                    Direction::North => Index{x: i as isize, y: TILE_SIZE as isize-1},
                    Direction::East => Index{x: 0, y: i as isize},
                    Direction::South => Index{x: i as isize, y: 0},
                    Direction::West => Index{x: TILE_SIZE as isize-1, y: i as isize},
                };
                let other_pixel = self.map[other+stride];

                let our_pixel = match dir {
                    Direction::North => tile[(0, i, *rotation)],
                    Direction::East => tile[(i, TILE_SIZE-1, *rotation)],
                    Direction::South => tile[(TILE_SIZE-1, i, *rotation)],
                    Direction::West => tile[(i, 0, *rotation)],
                };

                if !our_pixel.fits(&other_pixel) {
                    return Err(TileError::DoesntFit);
                }
            }
        }

        Ok(())
    }

    fn copy_tile(
        &mut self, 
        tile: &Tile, 
        rotation: Rotation,
        place: Place,
    ) {
        let upper_left = self.place_index(&place);

        for y in 0..TILE_SIZE {
            for x in 0..TILE_SIZE {
                let map_idx = upper_left + Index{x: x as isize, y: y as isize};
                self.map[map_idx] = tile[(y, x, rotation)];
            }
        }
    }

    fn get_structures(&self, place: &Place, in_game: bool) -> ArrayVec<Structure, 9> {
        let index = self.place_index(place);

        let tile_indices = (0..5).cartesian_product(0..5)
            .map(|(x, y)| Index{x, y} + index);
        let mut not_visited: HashSet<Index> = HashSet::from_iter(tile_indices);

        let mut structures: ArrayVec<Structure, 9> = ArrayVec::new();
        while let Some(seed) = not_visited.iter().next().copied() {
            not_visited.remove(&seed);
            let seed_pixel = self.map[seed];

            if TilePixel::scoring_tiles().contains(&seed_pixel) {
                let mut completed = true;
                let mut followers_number = HashMap::new();
                let mut followers_idx = HashSet::new();
                let tiles = flood_fill(
                    seed, 
                    |idx| {
                        let current = self.map[idx];
                        not_visited.remove(&idx);

                        // Flood fill algorithm queries for pixel that is not set
                        // which means the structure isnt fully connected
                        if current == TilePixel::Nothing {
                            completed = false;
                        }

                        if let Some(player) = self.followers.get(&idx) {
                            *followers_number.entry(*player).or_default() += 1;
                            followers_idx.insert(idx);
                        }

                        current.connects(&seed_pixel)
                    },
                );
                
                let points = if seed_pixel == TilePixel::Field {
                    if in_game {
                        0
                    } else {
                        self.get_field_score(seed)
                    }
                } else {
                    self.get_structure_score(&tiles, in_game)
                };

                structures.push(Structure {
                    completed,
                    points,
                    followers_number,
                    followers_idx,
                    seed,
                }).unwrap();
            }
        }

        structures
    }

    fn get_field_score(&self, seed: Index) -> u32 {
        let mut cities = HashSet::new();
        let mut followers_number: HashMap<PlayerId, u32> = HashMap::new();
        let mut followers_idx = HashSet::new();
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

    fn get_structure_score(&self, tiles: &HashSet<Index>, in_game: bool) -> u32 {
        let scoring_places = tiles.iter()
            .map(|idx| (self.map[*idx], self.index_place(idx)))
            .unique();

        scoring_places
            .map(|(pixel, place)| {
                match pixel {
                    TilePixel::Road | TilePixel::City | TilePixel::PennantCity => 
                        pixel.score(in_game),
                    TilePixel::Cloister => self.neighbour_count(&place) + 1,
                    _ => 0,
                }
            })
            .sum()
    }

    fn score_structure(&mut self, structure: &Structure) {
        let scoring_players = structure.followers_number.iter()
            .max_set_by_key(|(_, number)| *number)
            .into_iter()
            .map(|(player, _)| player);

        for player in scoring_players {
            self.score.entry(*player).and_modify(|score| *score += structure.points);
        }

        for (player, number) in structure.followers_number.iter() {
            self.followers_left.entry(*player).and_modify(|el| *el += number);
        }

        for idx in structure.followers_idx.iter() {
            self.followers.remove(idx);
        }
    }

    pub fn play_move(&mut self, mov: Move) -> Result<(), TileError>{
        if self.move_num != mov.move_num {
            return Err(TileError::StaleMove)
        }
        self.move_num += 1;

        let tile = self.tiles[&mov.tile];
        self.tiles_left.take(&mov.tile);
        self.copy_tile(&tile, mov.rotation, mov.place);

        self.free_places.remove(&mov.place);
        self.free_places.extend(self.empty_neighbour_places(&mov.place));

        if let Some(follower) = mov.follower {
            self.followers.insert(follower, mov.player);
            self.followers_left.entry(mov.player).and_modify(|count| *count -= 1);
        }

        let structures = self.get_structures(&mov.place, true).into_iter()
            .filter(|structure| structure.completed);
        for structure in structures {
            self.score_structure(&structure);
        }

        Ok(())
    }

    pub fn end_game(&mut self) {
        let mut left_to_score: HashSet<Index> = HashSet::from_iter(self.followers.keys().copied());
        while !left_to_score.is_empty() {
            let idx = left_to_score.iter().copied().next().unwrap();
            let structure = self.get_structures(&self.index_place(&idx), false)
                .into_iter()
                .filter(|structure| structure.followers_idx.contains(&idx))
                .next()
                .unwrap();

            self.score_structure(&structure);
            left_to_score = &left_to_score - &structure.followers_idx;
        }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.map.to_display_string(Some(&self.followers))?)?;
        writeln!(f, "Score: {:?}", self.score)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::standard_tileset::STANDARD_TILESET;

    use super::*;

    #[test]
    fn check_tile_fits() {
        let tile = &Tile::new(*STANDARD_TILESET.tiles.get("CRFR").unwrap());
        let tile2 = &Tile::new(*STANDARD_TILESET.tiles.get("RRRR").unwrap());

        let mut game = Game::new(&STANDARD_TILESET.clone(), 2, 8);
        game.copy_tile(tile, Rotation::Rot0, Place { x: 0, y: 0 });

        assert!(game.check_tile(tile2, &Place { x: -1, y: 0 }, &Rotation::Rot0).is_ok());
    }

    #[test]
    fn check_tile_doesnt_fit() {
        let tile = &Tile::new(*STANDARD_TILESET.tiles.get("CRFR").unwrap());
        let tile2 = &Tile::new(*STANDARD_TILESET.tiles.get("FFFF_CLOISTER").unwrap());

        let mut game = Game::new(&STANDARD_TILESET.clone(), 2, 8);
        game.copy_tile(tile, Rotation::Rot0, Place { x: 0, y: 0 });

        assert!(game.check_tile(tile2, &Place { x: 0, y: -1 }, &Rotation::Rot0).is_err());
    }

    #[test]
    fn check_tile_doesnt_fit_rot() {
        let tile = &Tile::new(*STANDARD_TILESET.tiles.get("FRFR").unwrap());
        let tile2 = &Tile::new(*STANDARD_TILESET.tiles.get("FFRR").unwrap());

        let mut game = Game::new(&STANDARD_TILESET.clone(), 2, 8);
        game.copy_tile(tile, Rotation::Rot1, Place { x: 0, y: 0 });

        assert!(game.check_tile(tile2, &Place { x: 0, y: -1 }, &Rotation::Rot2).is_err());
    }
}