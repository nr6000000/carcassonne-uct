use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
use std::fmt::Display;

use heapless::Vec as ArrayVec;
use strum::{EnumIter, IntoEnumIterator};
use thiserror::Error;
use tileset_format::{TILE_SIZE, TileSet};

use crate::engine::TilePixel;
use crate::engine::datastructures::direction::{Direction, OrdinalDirection};
use crate::engine::datastructures::index::Index;
use crate::engine::datastructures::map::{Map};
use crate::engine::datastructures::multi_hashset::MultiHashSet;
use crate::engine::tile::{Tile, TileId};
use crate::engine::tilepixel_ext::TilePixelFits;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct PlayerId(u32);

pub struct Game {
    move_num: u32,
    map: Map<TilePixel>,
    tiles: HashMap<TileId, Tile>,
    tiles_left: MultiHashSet<TileId>,
    free_places: HashSet<Place>,
    followers_left: MultiHashSet<PlayerId>,
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

#[derive(Debug)]
pub struct Move {
    move_num: u32,
    place: Place,
    tile: TileId,
    rotation: Rotation,
    follower: Option<usize>,
}

impl Move {
    pub fn get_move_num(&self) -> u32 {
        self.move_num
    }
}

impl Game {
    pub fn new(tileset: &TileSet, number_players: u32) -> Game {
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
        let starting_followers = 8;

        let mut game = Game{
            move_num: 0,
            map: Map::new(init_map_size),
            tiles,
            tiles_left,
            free_places: HashSet::new(),
            followers_left: MultiHashSet::from_iter(
                (0..number_players).map(|id| (PlayerId(id), starting_followers))
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

    fn place_index(&self, place: &Place) -> Index {
        Index {
            x: place.x*TILE_SIZE as isize,
            y: place.y*TILE_SIZE as isize,
        }
    }

    fn place_occupied(&self, place: &Place) -> bool {
        self.map[self.place_index(place)] != TilePixel::Nothing
    }

    fn neighbour_count(&self, place: &Place) -> usize {
        Direction::iter()
            .filter(|dir| self.place_occupied(&place.neighbour(dir.into())))
            .count()
    }

    fn empty_neighbour_places(&self, place: &Place) -> ArrayVec<Place, 4> {
        Direction::iter()
            .map(|dir| place.neighbour(&dir.into()))
            .filter(|place| !self.place_occupied(place))
            .collect()
    }

    pub fn get_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new(); 
        for tile_id in self.tiles_left.elements() {
            let tile = &self.tiles[tile_id];

            for place in self.free_places.iter() {
                for rotation in Rotation::iter() {
                    if self.check_tile(&tile, place, &rotation).is_ok() {
                        moves.push(Move {
                            move_num: self.move_num,
                            place: *place,
                            tile: *tile_id,
                            rotation,
                            follower: None,
                        });
                    }
                }
            }
        }

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
                let compared = self.place_index(&place.neighbour(&dir.into()));

                let map_idx = match dir {
                    Direction::North => compared+Index{x: i as isize, y: TILE_SIZE as isize-1},
                    Direction::East => compared+Index{x: 0, y: i as isize},
                    Direction::South => compared+Index{x: i as isize, y: 0},
                    Direction::West => compared+Index{x: TILE_SIZE as isize-1, y: i as isize},
                };

                let edge_fits = match dir {
                    Direction::North => self.map[map_idx].fits(&tile[(0, i, *rotation)]),
                    Direction::East => self.map[map_idx].fits(&tile[(i, TILE_SIZE-1, *rotation)]),
                    Direction::South => self.map[map_idx].fits(&tile[(TILE_SIZE-1, i, *rotation)]),
                    Direction::West => self.map[map_idx].fits(&tile[(i, 0, *rotation)]),
                };

                if !edge_fits {
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

        Ok(())
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.map)?;
        writeln!(f, "Free places: {:?}", self.free_places)?;

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

        let mut game = Game::new(&STANDARD_TILESET.clone(), 2);
        game.copy_tile(tile, Rotation::Rot0, Place { x: 0, y: 0 });

        assert!(game.check_tile(tile2, &Place { x: -1, y: 0 }, &Rotation::Rot0).is_ok());
    }

    #[test]
    fn check_tile_doesnt_fit() {
        let tile = &Tile::new(*STANDARD_TILESET.tiles.get("CRFR").unwrap());
        let tile2 = &Tile::new(*STANDARD_TILESET.tiles.get("FFFF_CLOISTER").unwrap());

        let mut game = Game::new(&STANDARD_TILESET.clone(), 2);
        game.copy_tile(tile, Rotation::Rot0, Place { x: 0, y: 0 });

        assert!(game.check_tile(tile2, &Place { x: 0, y: -1 }, &Rotation::Rot0).is_err());
    }

    #[test]
    fn check_tile_doesnt_fit_rot() {
        let tile = &Tile::new(*STANDARD_TILESET.tiles.get("FRFR").unwrap());
        let tile2 = &Tile::new(*STANDARD_TILESET.tiles.get("FFRR").unwrap());

        let mut game = Game::new(&STANDARD_TILESET.clone(), 2);
        game.copy_tile(tile, Rotation::Rot1, Place { x: 0, y: 0 });

        assert!(game.check_tile(tile2, &Place { x: 0, y: -1 }, &Rotation::Rot2).is_err());
    }
}