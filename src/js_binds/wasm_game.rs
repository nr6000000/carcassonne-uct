use std::collections::HashMap;

use serde::Serialize;
use wasm_bindgen::prelude::*;
use tileset_format::TILE_SIZE;

use crate::engines::carcassonne_engine::CarcassonneEngine;
use crate::engines::minimax_engine::MinimaxEngine;
use crate::engines::greedy_engine::GreedyCompleterEngine;
use crate::game_logic::game::{
    Game, GameSettings, Move, Place, PlayerId, Rotation, place_index
};
use crate::game_logic::standard_tileset::STANDARD_TILESET;
use crate::game_logic::tile::TileId;
use crate::game_logic::tilepixel_ext::TilePixelExt;
use crate::game_logic::Index;
use crate::game_logic::TilePixel;

// ─── Serializable DTOs ───────────────────────────────────────────────────────

#[derive(Serialize)]
struct StartingInfo {
    tile_name: String,
    x: isize,
    y: isize,
}

#[derive(Serialize)]
struct TileInfo {
    count: u32,
    placements: Vec<PlacementInfo>,
}

#[derive(Serialize)]
struct PlacementInfo {
    x: isize,
    y: isize,
    rotation: u32,
}

#[derive(Serialize)]
struct HumanMovesResponse {
    game_over: bool,
    tiles: HashMap<String, TileInfo>,
}

#[derive(Serialize)]
struct MeepleOption {
    index: usize,
    structure_type: String,
    frac_x: f64,
    frac_y: f64,
}

#[derive(Serialize)]
struct MeepleOptionsResponse {
    options: Vec<MeepleOption>,
    can_skip: bool,
    followers_left: u32,
}

#[derive(Serialize)]
struct MoveResultResponse {
    success: bool,
    error: Option<String>,
    scores: ScoresInfo,
    game_over: bool,
}

#[derive(Serialize)]
struct BotMoveResponse {
    tile_name: String,
    x: isize,
    y: isize,
    rotation: u32,
    meeple: Option<MeeplePlacement>,
    scores: ScoresInfo,
    game_over: bool,
    no_moves: bool,
}

#[derive(Serialize, Clone)]
struct MeeplePlacement {
    structure_type: String,
    frac_x: f64,
    frac_y: f64,
    player: String,
}

#[derive(Serialize, Clone)]
struct ScoresInfo {
    human: u32,
    bot: u32,
    human_followers: u32,
    bot_followers: u32,
}

#[derive(Serialize, Clone)]
struct PlacedTileRecord {
    tile_name: String,
    x: isize,
    y: isize,
    rotation: u32,
    meeples: Vec<MeeplePlacement>,
}

#[derive(Serialize)]
struct BoardStateResponse {
    placed_tiles: Vec<PlacedTileRecord>,
    scores: ScoresInfo,
    free_places: Vec<FreePlaceInfo>,
}

#[derive(Serialize)]
struct FreePlaceInfo {
    x: isize,
    y: isize,
}

#[derive(Serialize)]
struct EndGameResponse {
    scores: ScoresInfo,
    winner: String,
}

// ─── WasmGame ────────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub struct WasmGame {
    game: Game,
    tile_name_to_id: HashMap<String, TileId>,
    tile_id_to_name: HashMap<TileId, String>,
    placed_tiles: Vec<PlacedTileRecord>,
    cached_moves: Vec<Move>,
    cached_structures: rapidhash::RapidHashMap<(Place, TileId, Rotation), Vec<crate::game_logic::structures::Structure>>,
    human_player: PlayerId,
    bot_player: PlayerId,
    bot_engine: MinimaxEngine,
    greedy_engine: GreedyCompleterEngine,
    // Track tile name -> remaining count from tileset
    tile_name_to_count: HashMap<String, u32>,
}

#[wasm_bindgen]
impl WasmGame {
    #[wasm_bindgen(constructor)]
    pub fn new(plain_mode: bool, bot_depth: u32) -> WasmGame {
        let mut tileset = STANDARD_TILESET.clone();
        
        if plain_mode {
            let keep = vec!["CRFR", "FFRR", "FRRR_DISCONNECTED"];
            tileset.tiles.retain(|k, _| keep.contains(&k.as_str()));
            tileset.tile_numbers.retain(|k, _| keep.contains(&k.as_str()));
            
            if let Some(c) = tileset.tile_numbers.get_mut("CRFR") {
                *c = 0;
            }
            if let Some(c) = tileset.tile_numbers.get_mut("FFRR") {
                *c = 36;
            }
            if let Some(c) = tileset.tile_numbers.get_mut("FRRR_DISCONNECTED") {
                *c = 36;
            }
        }
        
        let mut settings = GameSettings::default();
        settings.farmers_enabled = false;
        let game = Game::new(&tileset, settings);

        let mut tile_name_to_id: HashMap<String, TileId> = HashMap::new();
        let mut tile_id_to_name: HashMap<TileId, String> = HashMap::new();

        // Rebuild the same name-to-id mapping that Game::new creates
        let mut tile_id_gen = 0u32..;
        for tile_name in tileset.tiles.keys() {
            let id = TileId(tile_id_gen.next().unwrap());
            tile_name_to_id.insert(tile_name.clone(), id);
            tile_id_to_name.insert(id, tile_name.clone());
        }

        // Initial tile counts from tileset (starting tile already placed)
        let mut tile_name_to_count: HashMap<String, u32> = HashMap::new();
        for (name, &count) in &tileset.tile_numbers {
            tile_name_to_count.insert(name.clone(), count);
        }
        // Starting tile is already placed
        let starting = &tileset.starting_tile;
        if let Some(c) = tile_name_to_count.get_mut(starting) {
            *c = c.saturating_sub(1);
        }

        // Record the starting tile placement
        let starting_tile_record = PlacedTileRecord {
            tile_name: starting.clone(),
            x: 0,
            y: 0,
            rotation: 0,
            meeples: vec![],
        };

        WasmGame {
            game,
            tile_name_to_id,
            tile_id_to_name,
            placed_tiles: vec![starting_tile_record],
            cached_moves: vec![],
            cached_structures: rapidhash::RapidHashMap::default(),
            human_player: PlayerId(0),
            bot_player: PlayerId(1),
            bot_engine: MinimaxEngine::new(bot_depth),
            greedy_engine: GreedyCompleterEngine::new(),
            tile_name_to_count,
        }
    }

    pub fn set_bot_depth(&mut self, depth: u32) {
        self.bot_engine.depth = depth;
    }

    /// Returns starting info as JSON
    pub fn get_starting_info(&self) -> String {
        let info = StartingInfo {
            tile_name: "CRFR".to_string(),
            x: 0,
            y: 0,
        };
        serde_json::to_string(&info).unwrap()
    }

    /// Compute all valid moves for the human player.
    /// Returns available tiles with their valid placements.
    pub fn compute_human_moves(&mut self) -> String {
        let (moves, structures) = self.game.get_moves();

        // Group moves by tile name and (place, rotation)
        let mut tiles: HashMap<String, TileInfo> = HashMap::new();

        // Track unique placements per tile (without duplicates from follower variants)
        let mut seen_placements: HashMap<String, Vec<(isize, isize, u32)>> = HashMap::new();

        for mov in &moves {
            let tile_name = match self.tile_id_to_name.get(&mov.tile) {
                Some(name) => name.clone(),
                None => continue,
            };

            let rot_val = mov.rotation as u32;
            let placement_key = (mov.place.x, mov.place.y, rot_val);

            let entry = seen_placements.entry(tile_name.clone()).or_default();
            if !entry.contains(&placement_key) {
                entry.push(placement_key);

                let tile_info = tiles.entry(tile_name.clone()).or_insert_with(|| {
                    let count = self.tile_name_to_count.get(&tile_name).copied().unwrap_or(0);
                    TileInfo {
                        count,
                        placements: vec![],
                    }
                });

                tile_info.placements.push(PlacementInfo {
                    x: mov.place.x,
                    y: mov.place.y,
                    rotation: rot_val,
                });
            }
        }

        self.cached_moves = moves;
        self.cached_structures = structures;

        let response = HumanMovesResponse {
            game_over: tiles.is_empty(),
            tiles,
        };

        serde_json::to_string(&response).unwrap()
    }

    /// Get meeple placement options for a specific tile placement.
    pub fn get_meeple_options(&self, tile_name: &str, x: i32, y: i32, rotation: u32) -> String {
        let place = Place {
            x: x as isize,
            y: y as isize,
        };

        // Find moves matching this placement that have followers
        let mut options: Vec<MeepleOption> = Vec::new();
        let mut seen_seeds: Vec<Index> = Vec::new();

        for mov in &self.cached_moves {
            let name = match self.tile_id_to_name.get(&mov.tile) {
                Some(n) => n,
                None => continue,
            };

            if name != tile_name
                || mov.place != place
                || (mov.rotation as u32) != rotation
            {
                continue;
            }

            if let Some(follower_idx) = mov.follower {
                // Avoid duplicate seeds
                if seen_seeds.contains(&follower_idx) {
                    continue;
                }
                seen_seeds.push(follower_idx);

                // Convert global pixel index to tile-local fraction
                let tile_origin = place_index(&place);
                let local_x = follower_idx.x - tile_origin.x;
                let local_y = follower_idx.y - tile_origin.y;
                let frac_x = (local_x as f64 + 0.5) / TILE_SIZE as f64;
                let frac_y = (local_y as f64 + 0.5) / TILE_SIZE as f64;

                // Determine structure type from the cached structures
                let structure_type = pixel_to_structure_type(self.game.map[follower_idx]);

                options.push(MeepleOption {
                    index: options.len(),
                    structure_type,
                    frac_x,
                    frac_y,
                });
            }
        }

        let followers_left = self.game.get_followers()
            .get(&self.human_player)
            .copied()
            .unwrap_or(0);

        let response = MeepleOptionsResponse {
            options,
            can_skip: true,
            followers_left,
        };

        serde_json::to_string(&response).unwrap()
    }

    /// Play a human move.
    /// meeple_index: -1 to skip meeple, otherwise index into meeple options
    pub fn play_human_move(
        &mut self,
        tile_name: &str,
        x: i32,
        y: i32,
        rotation: u32,
        meeple_index: i32,
    ) -> String {
        let place = Place {
            x: x as isize,
            y: y as isize,
        };

        // Find the matching move from cache
        let matching_move = self.find_matching_move(tile_name, &place, rotation, meeple_index);

        match matching_move {
            Some(mov) => {
                let meeple_info = self.extract_meeple_info(&mov, &place, "human");

                match self.game.play_move(mov) {
                    Ok(()) => {
                        // Update tile count
                        if let Some(c) = self.tile_name_to_count.get_mut(tile_name) {
                            *c = c.saturating_sub(1);
                        }

                        // Record placement
                        self.placed_tiles.push(PlacedTileRecord {
                            tile_name: tile_name.to_string(),
                            x: place.x,
                            y: place.y,
                            rotation,
                            meeples: meeple_info.into_iter().collect(),
                        });

                        let scores = self.get_scores_info();
                        let game_over = self.check_game_over();

                        let response = MoveResultResponse {
                            success: true,
                            error: None,
                            scores,
                            game_over,
                        };
                        serde_json::to_string(&response).unwrap()
                    }
                    Err(e) => {
                        let response = MoveResultResponse {
                            success: false,
                            error: Some(format!("{}", e)),
                            scores: self.get_scores_info(),
                            game_over: false,
                        };
                        serde_json::to_string(&response).unwrap()
                    }
                }
            }
            None => {
                let response = MoveResultResponse {
                    success: false,
                    error: Some("No matching move found".to_string()),
                    scores: self.get_scores_info(),
                    game_over: false,
                };
                serde_json::to_string(&response).unwrap()
            }
        }
    }

    /// Bot plays its turn using GreedyEngine
    pub fn bot_play(&mut self) -> String {
        let (moves, structures) = self.game.get_moves();

        if moves.is_empty() {
            return serde_json::to_string(&BotMoveResponse {
                tile_name: String::new(),
                x: 0,
                y: 0,
                rotation: 0,
                meeple: None,
                scores: self.get_scores_info(),
                game_over: true,
                no_moves: true,
            })
            .unwrap();
        }

        let chosen_move = self.bot_engine.play_move(&mut self.game);

        let tile_name = self.tile_id_to_name
            .get(&chosen_move.tile)
            .cloned()
            .unwrap_or_default();

        let meeple_info = self.extract_meeple_info(&chosen_move, &chosen_move.place, "bot");

        self.game.play_move(chosen_move).unwrap();

        // Update tile count
        if let Some(c) = self.tile_name_to_count.get_mut(&tile_name) {
            *c = c.saturating_sub(1);
        }

        // Record placement
        self.placed_tiles.push(PlacedTileRecord {
            tile_name: tile_name.clone(),
            x: chosen_move.place.x,
            y: chosen_move.place.y,
            rotation: chosen_move.rotation as u32,
            meeples: meeple_info.clone().into_iter().collect(),
        });

        let game_over = self.check_game_over();

        let response = BotMoveResponse {
            tile_name,
            x: chosen_move.place.x,
            y: chosen_move.place.y,
            rotation: chosen_move.rotation as u32,
            meeple: meeple_info.into_iter().next(),
            scores: self.get_scores_info(),
            game_over,
            no_moves: false,
        };

        serde_json::to_string(&response).unwrap()
    }

    /// Get the current board state for rendering
    pub fn get_board_state(&self) -> String {
        let scores = self.get_scores_info();
        let free_places: Vec<FreePlaceInfo> = self.game.free_places
            .iter()
            .map(|p| FreePlaceInfo { x: p.x, y: p.y })
            .collect();

        let response = BoardStateResponse {
            placed_tiles: self.placed_tiles.clone(),
            scores,
            free_places,
        };
        serde_json::to_string(&response).unwrap()
    }

    /// Get current scores
    pub fn get_scores(&self) -> String {
        serde_json::to_string(&self.get_scores_info()).unwrap()
    }

    /// Check if the game is over (no moves for either player)
    pub fn is_game_over(&mut self) -> bool {
        self.check_game_over()
    }

    /// End the game, compute final scores
    pub fn end_game(&mut self) -> String {
        self.game.end_game();
        let scores = self.get_scores_info();
        let winner = if scores.human > scores.bot {
            "human".to_string()
        } else if scores.bot > scores.human {
            "bot".to_string()
        } else {
            "tie".to_string()
        };

        let response = EndGameResponse { scores, winner };
        serde_json::to_string(&response).unwrap()
    }
}

// ─── Private helpers ─────────────────────────────────────────────────────────

impl WasmGame {
    fn get_scores_info(&self) -> ScoresInfo {
        let scores = self.game.get_score();
        let followers = self.game.get_followers();

        ScoresInfo {
            human: scores.get(&self.human_player).copied().unwrap_or(0),
            bot: scores.get(&self.bot_player).copied().unwrap_or(0),
            human_followers: followers.get(&self.human_player).copied().unwrap_or(0),
            bot_followers: followers.get(&self.bot_player).copied().unwrap_or(0),
        }
    }

    fn check_game_over(&mut self) -> bool {
        // Check if tiles_left is empty or no valid moves for either player
        let (human_moves, _) = self.game.get_moves();
        if !human_moves.is_empty() {
            return false;
        }
        let (bot_moves, _) = self.game.get_moves();
        bot_moves.is_empty()
    }

    fn find_matching_move(
        &self,
        tile_name: &str,
        place: &Place,
        rotation: u32,
        meeple_index: i32,
    ) -> Option<Move> {
        // Collect all moves matching this tile + position + rotation
        let matching: Vec<&Move> = self.cached_moves.iter().filter(|mov| {
            let name = match self.tile_id_to_name.get(&mov.tile) {
                Some(n) => n,
                None => return false,
            };
            name == tile_name
                && mov.place == *place
                && (mov.rotation as u32) == rotation
        }).collect();

        if meeple_index < 0 {
            // Find the move without a follower
            matching.iter()
                .find(|m| m.follower.is_none())
                .copied()
                .copied()
        } else {
            // Find moves with followers, deduplicate by seed, pick by index
            let mut seen_seeds: Vec<Index> = Vec::new();
            let mut follower_moves: Vec<&Move> = Vec::new();

            for mov in &matching {
                if let Some(follower_idx) = mov.follower {
                    if !seen_seeds.contains(&follower_idx) {
                        seen_seeds.push(follower_idx);
                        follower_moves.push(mov);
                    }
                }
            }

            follower_moves
                .get(meeple_index as usize)
                .copied()
                .copied()
        }
    }

    fn extract_meeple_info(&self, mov: &Move, place: &Place, player: &str) -> Vec<MeeplePlacement> {
        if let Some(follower_idx) = mov.follower {
            let tile_origin = place_index(place);
            let local_x = follower_idx.x - tile_origin.x;
            let local_y = follower_idx.y - tile_origin.y;
            let frac_x = (local_x as f64 + 0.5) / TILE_SIZE as f64;
            let frac_y = (local_y as f64 + 0.5) / TILE_SIZE as f64;

            let structure_type = pixel_to_structure_type(self.game.map[follower_idx]);

            vec![MeeplePlacement {
                structure_type,
                frac_x,
                frac_y,
                player: player.to_string(),
            }]
        } else {
            vec![]
        }
    }
}

fn pixel_to_structure_type(pixel: TilePixel) -> String {
    match pixel {
        TilePixel::City | TilePixel::PennantCity => "City".to_string(),
        TilePixel::Road => "Road".to_string(),
        TilePixel::Field => "Field".to_string(),
        TilePixel::Cloister => "Cloister".to_string(),
        _ => "Unknown".to_string(),
    }
}
