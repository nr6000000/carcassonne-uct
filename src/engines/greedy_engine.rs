use itertools::Itertools;
use rapidhash::RapidHashMap;

use crate::game_logic::Index;
use crate::game_logic::structures::Structure;
use crate::{engines::carcassonne_engine::CarcassonneEngine, game_logic::game::{Move, PlayerId}};

pub struct GreedyEngine {}

impl GreedyEngine {
    pub fn new() -> Self {
        Self {}
    }
}

fn get_points(structure: &Structure, player: PlayerId) -> u32 {
    structure.followers_number.iter()
        .max_set_by_key(|(_, number)| *number)
        .into_iter()
        .map(|(player, _)| player)
        .any(|p| *p == player)
        .then(|| structure.points)
        .unwrap_or(0)
}

impl CarcassonneEngine for GreedyEngine {
    fn play_move(
        &mut self, 
        moves: Vec<Move>, 
        structures: RapidHashMap<Index, Structure>,
        player: PlayerId,
    ) -> Move {
        let chosen_move = moves.iter()
            .max_by_key(|mov| mov.get_follower()
                .map(|idx| get_points(&structures[&idx], player))
            ).unwrap();
        *chosen_move
    }
}