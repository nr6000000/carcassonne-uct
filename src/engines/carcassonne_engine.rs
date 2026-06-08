use rapidhash::RapidHashMap;

use crate::game_logic::{Index, game::{Move, PlayerId}, structures::Structure};

pub trait CarcassonneEngine {
    fn play_move(
        &mut self, 
        moves: Vec<Move>,
        structures: RapidHashMap<Index, Structure>,
        player: PlayerId,
    ) -> Move;
}