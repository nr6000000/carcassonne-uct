use std::collections::HashMap;

use rand::rngs::ThreadRng;
use rand::{RngExt};

use crate::game_logic::Index;
use crate::game_logic::game::Structure;
use crate::{engines::carcassonne_engine::CarcassonneEngine, game_logic::game::{Move, PlayerId}};

pub struct RandomEngine {
    rng: ThreadRng
}

impl RandomEngine {
    pub fn new() -> Self {
        Self{ rng: rand::rng() }
    }
}

impl CarcassonneEngine for RandomEngine {
    fn play_move(
        &mut self, 
        moves: Vec<Move>, 
        _structures: HashMap<Index, Structure>,
        _player: PlayerId,
    ) -> Move {
        let idx = self.rng.random_range(0..moves.len());
        let chosen_move = moves.get(idx).unwrap();
        *chosen_move
    }
}