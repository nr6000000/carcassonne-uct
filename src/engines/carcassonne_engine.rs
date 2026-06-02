use crate::game_logic::game::{Move, PlayerId};

pub trait CarcassonneEngine {
    fn play_move(&mut self, moves: Vec<Move>, player: PlayerId) -> Move;
}