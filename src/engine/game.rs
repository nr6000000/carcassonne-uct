use std::{collections::{HashMap, HashSet}, fmt::{self, Display}};

use heapless::{index_map::FnvIndexMap, Vec as ArrayVec};
use strum::{IntoEnumIterator, VariantArray};
use thiserror::Error;

use crate::engine::{datastructures::{direction::Direction, multi_hashset::MultiHashSet}, fixed_tile::FixedTile, structure_links::{RelStructureLinks, StructureLinks}, tile::{Rotation, Tile}, tile_set::TileSet};

pub struct Game {
    move_num: u32,
    map: HashMap<Place, FixedTile>,
    tiles_left: MultiHashSet<Tile>,
    places_available: HashSet<Place>,
    structures: HashMap<Tile, RelStructureLinks>,
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
}

impl Move {
    pub fn get_move_num(&self) -> u32 {
        self.move_num
    }
}

impl Game {
    pub fn new(tileset: TileSet) -> Game {        
        let mut game = Game{
            move_num: 0,
            map: HashMap::new(),
            tiles_left: tileset.tiles.iter().cloned().collect(),
            places_available: HashSet::new(),
            structures: tileset.structures,
        };

        let starting_place = Place{x: 0, y: 0};
        game.map.insert(
            starting_place,
            tileset.starting_tile.clone(),
        );
        game.tiles_left.take(&tileset.starting_tile.clone().into());
        game.places_available.extend([
            starting_place.neighbour(&Direction::North),
            starting_place.neighbour(&Direction::East),
            starting_place.neighbour(&Direction::South),
            starting_place.neighbour(&Direction::West),
        ]);
        
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
                    let fixed_tile = tile.fix_rotation(rotation, &self.structures);

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
        let fixed_tile = tile.fix_rotation(&rotation, &self.structures);
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

        Ok(())
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