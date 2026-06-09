use std::f32::consts::PI;
use std::fmt::Display;

use heapless::{Vec as ArrayVec};
use itertools::Itertools;
use rand::rngs::ThreadRng;
use rapidhash::{RapidHashMap, RapidHashSet};
use strum::{EnumIter, IntoEnumIterator};
use thiserror::Error;
use tileset_format::{TILE_SIZE, TileSet};
use rand::seq::{IndexedRandom, IteratorRandom};

use crate::game_logic::TilePixel;
use crate::game_logic::datastructures::direction::{Direction, OrdinalDirection};
use crate::game_logic::datastructures::index::Index;
use crate::game_logic::datastructures::map::{Map};
use crate::game_logic::datastructures::multi_hashset::MultiHashSet;
use crate::game_logic::structures::Structure;
use crate::game_logic::tile::{NOTHING_TILE, Tile, TileId};
use crate::game_logic::tilepixel_ext::TilePixelExt;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct PlayerId(pub(crate) u32);

#[derive(Clone)]
pub struct Game {
    pub(crate) move_num: u32,
    pub(crate) map: Map<TilePixel>,
    pub(crate) tiles: RapidHashMap<TileId, Tile>,
    pub(crate) tiles_left: MultiHashSet<TileId>,
    pub(crate) free_places: RapidHashSet<Place>,
    pub(crate) followers: RapidHashMap<Index, PlayerId>,
    pub(crate) followers_left: RapidHashMap<PlayerId, u32>,
    pub(crate) score: RapidHashMap<PlayerId, u32>,
    pub(crate) settings: GameSettings,
    rng: ThreadRng,
}

#[derive(Debug, Clone, Copy)]
pub struct GameSettings {
    pub number_players: u32,
    pub number_followers: u32,
    pub calculate_move_score_field: bool,
    pub farmers_enabled: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self { 
            number_players: 2,
            number_followers: 8, 
            calculate_move_score_field: true,
            farmers_enabled: true,
        }
    }
}

#[derive(Debug, EnumIter, Clone, Copy)]
pub enum Rotation {
    Rot0 = 0,
    Rot1 = 1,
    Rot2 = 2,
    Rot3 = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Place {
    pub x: isize,
    pub y: isize,
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
    #[error("Nierozpoznany kafelek")]
    BadTile,
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub(crate) move_num: u32,
    pub(crate) place: Place,
    pub(crate) tile: TileId,
    pub(crate) rotation: Rotation,
    pub(crate) follower: Option<Index>,
    pub(crate) player: PlayerId,
}

impl Move {
    pub fn get_move_num(&self) -> u32 {
        self.move_num
    }

    pub fn get_place(&self) -> Place {
        self.place
    }

    pub fn get_follower(&self) -> Option<Index> {
        self.follower
    }

    pub fn get_tile(&self) -> TileId {
        self.tile
    }

    pub fn get_rotation(&self) -> Rotation {
        self.rotation
    }

    pub fn get_player(&self) -> PlayerId {
        self.player
    }
}

pub(crate) fn place_index(place: &Place) -> Index {
        Index {
            x: place.x*TILE_SIZE as isize,
            y: place.y*TILE_SIZE as isize,
        }
    }

pub(crate) fn index_place(index: &Index) -> Place {
    Place { 
        x: index.x.div_euclid(TILE_SIZE as isize), 
        y: index.y.div_euclid(TILE_SIZE as isize),
    }
}

impl Game {
    pub fn new(
        tileset: &TileSet, 
        settings: GameSettings,
    ) -> Game {
        // To support perfect circle with A tiles we need size of approx:
        // A = pi*r^2
        // r = sqrt(A/pi)
        // d = sqrt(A/pi)*2
        // 
        // Add generous margin to support most games without realloc:
        // d*4 = sqrt(A/pi)*8
        let tileset_size = tileset.tiles.iter().count() as f32;
        let init_map_size = (tileset_size/PI).sqrt().ceil() as usize * 8 * TILE_SIZE;

        let mut tiles: RapidHashMap<TileId, Tile> = RapidHashMap::default();
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
        let number_players = settings.number_players;
        let number_followers = settings.number_followers;

        let mut game = Game{
            move_num: 0,
            map: Map::new(init_map_size),
            tiles,
            tiles_left,
            free_places: RapidHashSet::default(),
            followers_left: RapidHashMap::from_iter(
                (0..number_players).map(|i| (PlayerId(i), number_followers)),
            ),
            followers: RapidHashMap::default(),
            score: RapidHashMap::from_iter(
                (0..number_players).map(|i| (PlayerId(i), 0)),
            ),
            settings,
            rng: rand::rng(),
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

    pub fn get_score(&self) -> RapidHashMap<PlayerId, u32> {
        self.score.clone()
    }

    pub fn get_followers(&self) -> RapidHashMap<PlayerId, u32> {
        self.followers_left.clone()
    }

    pub fn get_tiles_left(&self) -> usize {
        self.tiles_left.len()
    }

    fn place_occupied(&self, place: &Place) -> bool {
        self.map[place_index(place)] != TilePixel::Nothing
    }

    pub(crate) fn neighbour_count(&self, place: &Place) -> u32 {
        OrdinalDirection::iter()
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

    fn get_moves_structures(
        &mut self, 
        moves: Vec<Move>
    ) -> (Vec<Move>, RapidHashMap<Index, Structure>) {
        let (mut moves_with_followers, structures): (Vec<Move>, RapidHashMap<Index, Structure>) = moves
            .clone()
            .into_iter()
            .flat_map(|mov| {
                let tile = self.tiles[&mov.tile];

                self.copy_tile(&tile, mov.rotation, mov.place);
                let structures = self.get_structures(&mov.place, true);
                self.copy_tile(&NOTHING_TILE, Rotation::Rot0, mov.place);

                (self.followers_left[&mov.player] > 0).then(||
                    structures.into_iter()
                        .filter(|structure| structure.followers_number
                            .iter().all(|(_, &number)| number == 0)
                        )
                        .map(move |structure| (mov, structure))
                ).into_iter().flatten()
            }).map(|(mov, structure)| {
                let mut new_mov = mov.clone();
                new_mov.follower = Some(structure.seed);
                (new_mov, (structure.seed, structure))
            }).unzip();
        
        moves_with_followers.extend(moves);
        (
            moves_with_followers,
            structures
        )
    }

    pub fn get_moves(
        &mut self, 
        player: PlayerId
    ) -> (Vec<Move>, RapidHashMap<Index, Structure>) {
        let moves = self.get_moves_placement(player);
        self.get_moves_structures(moves)
    }

    pub fn get_random_move(&mut self, player: PlayerId) -> Option<Move> {
        let places = self.free_places.iter()
            .sample(&mut self.rng, self.free_places.len());

        let tile_ids = self.tiles_left.iter()
            .sample(&mut self.rng, self.tiles_left.len());

        let sampled_move = 'sample_move: {
            for place in places {
                for &tile_id in &tile_ids {
                    let tile = self.tiles[tile_id];
                    for rotation in Rotation::iter() {
                        if self.check_tile(&tile, place, &rotation).is_ok() {
                            break 'sample_move Some(Move {
                                move_num: self.move_num,
                                place: *place,
                                tile: *tile_id,
                                rotation,
                                follower: None,
                                player,
                            })
                        }
                    }
                }
            }
            None
        };

        if let Some(mov) = sampled_move {
            let (moves, _) = self.get_moves_structures(vec![mov]);
            Some(*moves.choose(&mut self.rng).unwrap())
        } else {
            sampled_move
        }
    }

    pub(crate) fn check_tile(
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
                let other = place_index(&place.neighbour(&dir.into()));
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

    pub(crate) fn copy_tile(
        &mut self, 
        tile: &Tile, 
        rotation: Rotation,
        place: Place,
    ) {
        let upper_left = place_index(&place);

        for y in 0..TILE_SIZE {
            for x in 0..TILE_SIZE {
                let map_idx = upper_left + Index{x: x as isize, y: y as isize};
                self.map[map_idx] = tile[(y, x, rotation)];
            }
        }
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

    pub fn play_custom_move(
        &mut self, 
        place: &Place, 
        tile: Tile,
        follower: Option<Index>,
        player: PlayerId,
    ) -> Result<(), TileError> {
        let (tile_id, rot) = 'find_tile: {
            for (tile_id, tile_ref) in self.tiles.iter() {
                for rot in Rotation::iter() {
                    let mut matches = true;
                    for (y, x) in (0..TILE_SIZE).cartesian_product(0..TILE_SIZE) {
                        if tile[(y, x, Rotation::Rot0)] != tile_ref[(y, x, rot)] {
                            matches = false;
                            break;
                        }
                    }

                    if matches {
                        break 'find_tile Ok((tile_id, rot));
                    }
                }
            }

            Err(TileError::BadTile)
        }?;

        self.check_tile(&self.tiles[tile_id], place, &rot)?;

        let mov = Move {
            move_num: self.move_num,
            place: *place,
            tile: *tile_id,
            rotation: rot,
            follower: follower.map(|f| f + place_index(place)),
            player,
        };
        self.play_move(mov)?;
        
        Ok(())
    }

    pub fn end_game(&mut self) {
        let mut left_to_score: RapidHashSet<Index> = RapidHashSet::from_iter(self.followers.keys().copied());
        while !left_to_score.is_empty() {
            let idx = left_to_score.iter().copied().next().unwrap();
            let structure = self.get_structures(&index_place(&idx), false)
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
    use crate::game_logic::standard_tileset::STANDARD_TILESET;

    use super::*;

    #[test]
    fn check_tile_fits() {
        let tile = &Tile::new(*STANDARD_TILESET.tiles.get("CRFR").unwrap());
        let tile2 = &Tile::new(*STANDARD_TILESET.tiles.get("RRRR").unwrap());

        let mut game = Game::new(&STANDARD_TILESET.clone(), GameSettings::default());
        game.copy_tile(tile, Rotation::Rot0, Place { x: 0, y: 0 });

        assert!(game.check_tile(tile2, &Place { x: -1, y: 0 }, &Rotation::Rot0).is_ok());
    }

    #[test]
    fn check_tile_doesnt_fit() {
        let tile = &Tile::new(*STANDARD_TILESET.tiles.get("CRFR").unwrap());
        let tile2 = &Tile::new(*STANDARD_TILESET.tiles.get("FFFF_CLOISTER").unwrap());

        let mut game = Game::new(&STANDARD_TILESET.clone(), GameSettings::default());
        game.copy_tile(tile, Rotation::Rot0, Place { x: 0, y: 0 });

        assert!(game.check_tile(tile2, &Place { x: 0, y: -1 }, &Rotation::Rot0).is_err());
    }

    #[test]
    fn check_tile_doesnt_fit_rot() {
        let tile = &Tile::new(*STANDARD_TILESET.tiles.get("FRFR").unwrap());
        let tile2 = &Tile::new(*STANDARD_TILESET.tiles.get("FFRR").unwrap());

        let mut game = Game::new(&STANDARD_TILESET.clone(), GameSettings::default());
        game.copy_tile(tile, Rotation::Rot1, Place { x: 0, y: 0 });

        assert!(game.check_tile(tile2, &Place { x: 0, y: -1 }, &Rotation::Rot2).is_err());
    }
}