use crate::game_logic::{game::{Game, Move, PlayerId}};

pub trait CarcassonneEngine {
    fn play_move(
        &mut self, 
        game: &mut Game,
    ) -> Move;
}