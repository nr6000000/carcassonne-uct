use std::collections::HashMap;

use crate::game_logic::{Index, game::{Move, PlayerId, Structure}};

pub trait CarcassonneEngine {
    fn play_move(
        &mut self, 
        moves: Vec<Move>,
        structures: HashMap<Index, Structure>,
        player: PlayerId,
    ) -> Move;
}