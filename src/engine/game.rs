use std::{collections::{HashMap, HashSet}, fmt::{self, Display}};

use strum::{VariantArray};
use thiserror::Error;

use crate::engine::{multi_hashset::MultiHashSet, tile::{FixedTile, Place, Rotation, Tile, TileSet}};

pub struct Game {
    move_num: u32,
    map: HashMap<Place, FixedTile>,
    tiles_left: MultiHashSet<Tile>,
    places_available: HashSet<Place>,
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

struct Neighbours<'a> {
    north: Option<&'a FixedTile>,
    northeast: Option<&'a FixedTile>,
    east: Option<&'a FixedTile>,
    southeast: Option<&'a FixedTile>,
    south: Option<&'a FixedTile>,
    southwest: Option<&'a FixedTile>,
    west: Option<&'a FixedTile>,
    northwest: Option<&'a FixedTile>,
}

impl Neighbours<'_> {
    fn number(&self) -> i32 {
        (
            if self.north.is_some() { 1 } else { 0 } +
            if self.east.is_some() { 1 } else { 0 } +
            if self.south.is_some() { 1 } else { 0 } +
            if self.west.is_some() { 1 } else { 0 }
        )
    }

    fn number_diag(&self) -> i32 {
        (
            if self.north.is_some() { 1 } else { 0 } +
            if self.northeast.is_some() { 1 } else { 0 } +
            if self.east.is_some() { 1 } else { 0 } +
            if self.southeast.is_some() { 1 } else { 0 } +
            if self.south.is_some() { 1 } else { 0 } +
            if self.southwest.is_some() { 1 } else { 0 } +
            if self.west.is_some() { 1 } else { 0 } +
            if self.northwest.is_some() { 1 } else { 0 }
        )
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
            places_available: HashSet::new()
        };

        let starting_place = Place{x: 0, y: 0};
        game.map.insert(
            starting_place,
            tileset.starting_tile.clone(),
        );
        game.tiles_left.take(&tileset.starting_tile.clone().into());
        game.places_available.extend([
            starting_place.north(),
            starting_place.east(),
            starting_place.south(),
            starting_place.west(),
        ]);
        
        game
    }

    fn neighbours(&self, place: &Place) -> Neighbours<'_> {
        let north = self.map.get(&place.north());
        let northeast = self.map.get(&place.northeast());
        let east = self.map.get(&place.east());
        let southeast = self.map.get(&place.southeast());
        let south = self.map.get(&place.south());
        let southwest = self.map.get(&place.southwest());
        let west = self.map.get(&place.west());
        let northwest = self.map.get(&place.northwest());
        
        Neighbours {
            north,
            northeast,
            east,
            southeast,
            south,
            southwest,
            west,
            northwest,
        }
    }

    fn empty_neighbours(&self, place: &Place) -> Vec<Place> {
        let neighbours = self.neighbours(place);
        let mut empty: Vec<Place> = Vec::new();

        if let None = neighbours.north {
            empty.push(place.north());
        }

        if let None = neighbours.east {
            empty.push(place.east());
        }

        if let None = neighbours.south {
            empty.push(place.south());
        }

        if let None = neighbours.west {
            empty.push(place.west());
        }

        empty
    }

    pub fn get_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new(); 
        for tile in self.tiles_left.elements() {
            for place in self.places_available.iter() {
                for rotation in Rotation::VARIANTS {
                    let fixed_tile = tile.fix_rotation(rotation);

                    if let Ok(_) = self.check_tile(&fixed_tile, place) {
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
        if self.map.get(&place).is_some() {
            return Err(TileError::PlaceOccupied);
        }

        let neighbours = self.neighbours(&place);
        if neighbours.number() == 0
        {
            return Err(TileError::Disconnected);
        }

        if let Some(neighbour) = neighbours.north {
            if neighbour.south != tile.north {
                return Err(TileError::DoesntFit);
            }
        }

        if let Some(neighbour) = neighbours.east {
            if neighbour.west != tile.east {
                return Err(TileError::DoesntFit);
            }
        }

        if let Some(neighbour) = neighbours.south {
            if neighbour.north != tile.south {
                return Err(TileError::DoesntFit);
            }
        }

        if let Some(neighbour) = neighbours.west {
            if neighbour.east != tile.west {
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
        let fixed_tile = tile.fix_rotation(&rotation);
        self.check_tile(&fixed_tile, &place)?;
        self.play_move(Move { 
            move_num: self.move_num,
            place: place, 
            tile: fixed_tile 
        })?;

        return Ok(());
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
        self.places_available.extend(self.empty_neighbours(&mov.place));

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