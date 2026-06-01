use std::iter;

use crate::engine::game::{Game, PlayerId};
use rand::{RngExt};

pub fn gen_random_game(game: &mut Game) {
    let mut rng = rand::rng();
    let mut current_player = game
        .get_players()
        .collect::<Vec<PlayerId>>()
        .into_iter()
        .cycle();

    println!("Starting map");
    println!("{}", game);

    let mut moves = game.get_moves(current_player.next().unwrap());
    while !moves.is_empty() {
        moves = game.get_moves(current_player.next().unwrap()); 
        if !moves.is_empty() {
            // println!("Moves: {:#?}", moves);
            let idx = rng.random_range(0..moves.len());
            let chosen_move = moves.swap_remove(idx);

            println!("Move {}", chosen_move.get_move_num());
            println!("Playing move: {:#?}", chosen_move);
            game.play_move(chosen_move)
                .unwrap_or_else(|err| panic!("Bład gry: {}", err));
            println!("{}", game);
            println!();
        }
    }   
}