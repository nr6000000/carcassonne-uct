use crate::engines::carcassonne_engine::CarcassonneEngine;
use crate::game_logic::game::{Game, Move, PlayerId};

pub struct MinimaxEngine {
    pub depth: u32,
}

impl MinimaxEngine {
    pub fn new(depth: u32) -> Self {
        Self { depth }
    }

    fn minimax(
        &self,
        game: &mut Game,
        depth: u32,
        is_maximizing: bool,
        player: PlayerId,
        opponent: PlayerId,
        mut alpha: i32,
        mut beta: i32,
    ) -> (Option<Move>, i32) {
        if depth == 0 {
            let mut eval_game = game.clone();
            eval_game.end_game();
            let score_self = eval_game.get_score().get(&player).copied().unwrap_or(0) as i32;
            let score_opp = eval_game.get_score().get(&opponent).copied().unwrap_or(0) as i32;
            return (None, score_self - score_opp);
        }

        let current_player = if is_maximizing { player } else { opponent };
        let (moves, _) = game.get_moves(current_player);

        if moves.is_empty() {
            let mut eval_game = game.clone();
            eval_game.end_game();
            let score_self = eval_game.get_score().get(&player).copied().unwrap_or(0) as i32;
            let score_opp = eval_game.get_score().get(&opponent).copied().unwrap_or(0) as i32;
            return (None, score_self - score_opp);
        }

        let mut best_move = None;

        if is_maximizing {
            let mut max_eval = i32::MIN;
            for mov in moves {
                let mut next_game = game.clone();
                if next_game.play_move(mov).is_err() {
                    continue;
                }
                let (_, eval) = self.minimax(&mut next_game, depth - 1, false, player, opponent, alpha, beta);
                if eval > max_eval {
                    max_eval = eval;
                    best_move = Some(mov);
                }
                alpha = alpha.max(eval);
                if beta <= alpha {
                    break;
                }
            }
            (best_move, max_eval)
        } else {
            let mut min_eval = i32::MAX;
            for mov in moves {
                let mut next_game = game.clone();
                if next_game.play_move(mov).is_err() {
                    continue;
                }
                let (_, eval) = self.minimax(&mut next_game, depth - 1, true, player, opponent, alpha, beta);
                if eval < min_eval {
                    min_eval = eval;
                    best_move = Some(mov);
                }
                beta = beta.min(eval);
                if beta <= alpha {
                    break;
                }
            }
            (best_move, min_eval)
        }
    }
}

impl CarcassonneEngine for MinimaxEngine {
    fn play_move(
        &mut self, 
        game: &mut Game,
        player: PlayerId,
    ) -> Move {
        let players: Vec<PlayerId> = game.get_players().collect();
        let opponent = *players.iter().find(|&&p| p != player).unwrap_or(&player);
        
        let (best_move, _) = self.minimax(game, self.depth, true, player, opponent, i32::MIN, i32::MAX);
        
        best_move.unwrap_or_else(|| {
            let (moves, _) = game.get_moves(player);
            moves[0]
        })
    }
}
