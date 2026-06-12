use itertools::Itertools;
use rapidhash::RapidHashMap;

use crate::game_logic::Index;
use crate::game_logic::game::Game;
use crate::game_logic::structures::Structure;
use crate::{engines::carcassonne_engine::CarcassonneEngine, game_logic::game::{Move, PlayerId}};

pub struct GreedyBuilderEngine {}

impl GreedyBuilderEngine {
    pub fn new() -> Self {
        Self {}
    }
}

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

impl CarcassonneEngine for GreedyBuilderEngine {
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

                        let player = game.get_current_player();
                        get_points(structure, player, new_follower)
                    })
                    .sum::<u32>();
                points
            } 
            ).unwrap();
        *chosen_move
    }
}

pub struct GreedyCompleterEngine {}

impl GreedyCompleterEngine {
    pub fn new() -> Self {
        Self {}
    }
}

impl CarcassonneEngine for GreedyCompleterEngine {
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

                        if !structure.completed {
                            return new_follower as u32;
                        }

                        let player = game.get_current_player();
                        get_points(structure, player, new_follower)
                    })
                    .sum::<u32>();
                points
            } 
            ).unwrap();
        *chosen_move
    }
}
