use rand::rngs::ThreadRng;
use rand::{RngExt};
use rapidhash::RapidHashMap;

use crate::game_logic::Index;
use crate::game_logic::game::Game;
use crate::game_logic::structures::Structure;
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
        game: &mut Game,
    ) -> Move {
        game.get_random_move().unwrap()
    }
}