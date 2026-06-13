use itertools::Itertools;
use tileset_format::TilePixel;

use crate::{engines::carcassonne_engine::CarcassonneEngine, game_logic::{game::{Game, Move, PlayerId}, structures::Structure}};

fn get_points(structure: &Structure, player: PlayerId, new_follower: bool) -> u32 {
    let mut new_followers_number = structure.followers_number.clone();
    *new_followers_number.entry(player).or_default() += new_follower as u32;
    new_followers_number.iter()
        .filter(|(_, number)| **number > 0)
        .max_set_by_key(|(_, number)| **number)
        .into_iter()
        .map(|(player, _)| player)
        .any(|p| *p == player)
        .then(|| structure.points)
        .unwrap_or(0)
}

fn heurestic(game: &Game, structure: &Structure, new_follower: bool) -> u32 {
    let in_game_points = |points: u32| -> u32 {
        if !structure.completed && (
            game.map[structure.seed] == TilePixel::City ||
            game.map[structure.seed] == TilePixel::PennantCity
        ) {
            points / 2
        } else {
            points
        }
    };

    let player = game.get_current_player();
    let opponent = game.get_players().find(|p| *p != player).unwrap();

    let player_points = in_game_points(get_points(structure, player, new_follower));
    let opponent_points = in_game_points(get_points(structure, opponent, false));
    
    player_points-opponent_points
}

pub struct HeuresticEngine {}

impl HeuresticEngine {
    pub fn new() -> Self {
        Self {}
    }
}

impl CarcassonneEngine for HeuresticEngine {
    fn play_move(
        &mut self, 
        game: &mut Game,
    ) -> Move {
        let (moves, structures) = game.get_moves();

        let chosen_move = moves.iter()
            .max_by_key(|mov| {
                let points = structures[&(mov.place, mov.tile, mov.rotation)].iter()
                    .map(|structure| {
                        let new_follower = mov.follower
                            .is_some_and(|follower| follower == structure.seed);
                        heurestic(game, structure, new_follower)
                    })
                    .sum::<u32>();
                points
            } 
            ).unwrap();
        *chosen_move
    }
}
