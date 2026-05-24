use std::{collections::{HashMap, HashSet}, fmt::{self, Display}};

use heapless::{Vec as ArrayVec, index_map::FnvIndexMap, index_set::FnvIndexSet};
use strum::{IntoEnumIterator, VariantArray};
use thiserror::Error;

use crate::engine::{datastructures::{direction::Direction, multi_hashset::MultiHashSet}, fixed_tile::FixedTile, structures::{self, RelStructureLinks, StructureLinks, StructureType, TileStructure}, tile::{Feature, Rotation, Tile}, tile_set::TileSet};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Player(u32);

#[derive(Debug, Clone)]
pub struct Structure {
    id: usize,
    followers: MultiHashSet<Player>,
    complete: bool,
    structure_type: StructureType,
}

pub struct Game {
    move_num: u32,
    map: HashMap<Place, FixedTile>,
    tiles_left: MultiHashSet<Tile>,
    places_available: HashSet<Place>,
    tileset: TileSet,
    followers_left: MultiHashSet<Player>,
    structures: HashMap<usize, Structure>,
    structure_map: HashMap<Place, FnvIndexSet<usize, 16>>,
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

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct Place {
    pub x: i32,
    pub y: i32,
}

impl Place {
    pub fn neighbour(&self, dir: &Direction) -> Place {
        match dir {
            Direction::North => Place { x: self.x, y: self.y - 1 },
            Direction::NorthEast => Place { x: self.x + 1, y: self.y - 1 },
            Direction::East => Place { x: self.x + 1, y: self.y },
            Direction::SouthEast => Place { x: self.x + 1, y: self.y + 1 },
            Direction::South => Place { x: self.x, y: self.y + 1 },
            Direction::SouthWest => Place { x: self.x - 1, y: self.y + 1 },
            Direction::West => Place { x: self.x - 1, y: self.y },
            Direction::NorthWest => Place { x: self.x - 1, y: self.y - 1 },
        }
    }
}

#[derive(Debug)]
pub struct Move {
    move_num: u32,
    place: Place,
    tile: FixedTile,
    follower: Option<usize>,
}

impl Move {
    pub fn get_move_num(&self) -> u32 {
        self.move_num
    }
}

impl Game {
    pub fn new(tileset: TileSet, number_players: u32) -> Game {        
        let mut game = Game{
            move_num: 0,
            map: HashMap::new(),
            tiles_left: tileset.tiles.iter().cloned().collect(),
            places_available: HashSet::new(),
            tileset,
            followers_left: MultiHashSet::from_iter(
                (0..number_players).map(|id| (Player(id), 8))
            ),
            structures: HashMap::new(),
            structure_map: HashMap::new(),
        };

        let starting_place = Place{x: 0, y: 0};
        let starting_tile = game.tileset.starting_tile.clone();
        game.map.insert(
            starting_place,
            starting_tile.clone(),
        );
        game.tiles_left.take(&starting_tile.clone().into());
        game.places_available.extend([
            starting_place.neighbour(&Direction::North),
            starting_place.neighbour(&Direction::East),
            starting_place.neighbour(&Direction::South),
            starting_place.neighbour(&Direction::West),
        ]);

        let mut ids_set: FnvIndexSet<usize, 16> = FnvIndexSet::new();
        starting_tile.structure_links
            .get_structures()
            .for_each(|s| {
                let structure_type = *s.get_structure_type();
                let id = game.add_structure(Structure {
                    id: 0,
                    followers: HashSet::new(),
                    complete: false,
                    structure_type: structure_type,
                });

                let _ = ids_set.insert(id);
            });
        game.structure_map.insert(starting_place, ids_set);
        
        game
    }

    fn neighbour_edges(&self, place: &Place) -> FnvIndexMap<Direction, &FixedTile, 8> {
        Direction::edges().into_iter()
            .filter_map(|dir| {
                self.map.get(&place.neighbour(&dir))
                    .map(|tile| (dir, tile))
            })
            .collect()
    }

    fn neighbour_edges_empty(&self, place: &Place) -> ArrayVec<Place, 8> {
        Direction::iter()
            .map(|dir| place.neighbour(&dir))
            .filter(|place| self.map.get(&place).is_none())
            .collect()
    }

    pub fn get_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new(); 
        for tile in self.tiles_left.elements() {
            for place in self.places_available.iter() {
                for rotation in Rotation::VARIANTS {
                    let fixed_tile = tile.fix_rotation(rotation, &self.tileset.structures);

                    if self.check_tile(&fixed_tile, place).is_ok() {
                        moves.push(Move { 
                            move_num: self.move_num,
                            place: *place, 
                            tile: fixed_tile, 
                        });
                    }
                }
            }
        }

        moves
    }

    fn check_tile(&self, tile: &FixedTile, place: &Place) -> Result<(), TileError> {
        if self.map.contains_key(place) {
            return Err(TileError::PlaceOccupied);
        }

        let neighbours = self.neighbour_edges(place);
        if neighbours.iter().count() == 0
        {
            return Err(TileError::Disconnected);
        }

        for (dir, neighbour) in neighbours.into_iter() {
            if tile.edges[&dir] != neighbour.edges[&dir.opposite()] {
                return Err(TileError::DoesntFit);
            }
        }

        Ok(())
    }

    pub fn place_tile(
        &mut self, 
        tile: Tile, 
        rotation: Rotation,
        place: Place
    ) -> Result<(), TileError> {
        let fixed_tile = tile.fix_rotation(&rotation, &self.tileset.structures);
        self.check_tile(&fixed_tile, &place)?;
        self.play_move(Move { 
            move_num: self.move_num,
            place, 
            tile: fixed_tile 
        })?;

        Ok(())
    }

    pub fn play_move(&mut self, mov: Move) -> Result<(), TileError>{
        // println!("Tiles left: {:#?}", self.tiles_left);
        // println!("Places available: {:#?}", self.places_available);
        if self.move_num != mov.move_num {
            return Err(TileError::StaleMove)
        }

        self.move_num += 1;

        self.tiles_left.take(&mov.tile.clone().into());
        self.map.insert(mov.place, mov.tile);

        self.places_available.remove(&mov.place);
        self.places_available.extend(self.neighbour_edges_empty(&mov.place));

        for structure in mov.tile.structure_links.get_structures() {
            self.add_structure(Structure { 
                id: 0, 
                followers: move., 
                complete: (), 
                structure_type: *structure.get_structure_type(),
            });
        }

        Ok(())
    }

    pub fn add_structure(
        &mut self, 
        place: &Place,
        structure_links: &StructureLinks,
        tile: &FixedTile, 
        follower: bool,
    ) -> usize {
        for structure in tile.structure_links.get_structures() {
            let connected_structures =  structure_links
                .connects_to(structure)
                .filter_map(|dir|  self.map.get(&place.neighbour(dir)))
                .map(|neighbour| neighbour.structure_links.get_structure(dir));
        }

        let new_id = self.structures.len();
        self.structures.insert(new_id, Structure { 
            id: new_id, 
            followers: (), 
            complete: (), 
            structure_type 
        });
        new_id
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (min_x, max_x, min_y, max_y) = self.map.keys().fold(
        (i32::MAX, i32::MIN, i32::MAX, i32::MIN),
            |(min_x, max_x, min_y, max_y), p| {
                (min_x.min(p.x), max_x.max(p.x), min_y.min(p.y), max_y.max(p.y))
            },
        );

        let mut grid: Vec<Vec<Option<&FixedTile>>> = Vec::new();
        for y in min_y..max_y+1 {
            let mut row = Vec::new();
            for x in min_x..max_x+1 {
                row.push(self.map.get(&Place { x, y }));
            }
            grid.push(row);
        }

        let empty_tile = "      \n      \n      \n".to_owned();

        let mut chars: Vec<Vec<char>> = Vec::new();
        for row in grid {
            chars.push(Vec::new());
            chars.push(Vec::new());
            chars.push(Vec::new());

            let chars_len = chars.len();
            for tile in row {
                let tile_str = match tile {
                    Some(tile) => tile.to_string(),
                    None => empty_tile.clone(),
                };
                
                let lines: Vec<&str> = tile_str.lines().take(3).collect();

                chars[chars_len-3].extend(lines[0].chars());
                chars[chars_len-2].extend(lines[1].chars());
                chars[chars_len-1].extend(lines[2].chars());
            }
        }

        let str = chars
            .iter()
            .map(|v| v.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n");

        write!(f, "{}", str)?;
        Ok(())
    }
}