use crate::engine::{game::Game};
use rand::{RngExt};

pub fn gen_random_game(game: &mut Game) {
    let mut rng = rand::rng();

    println!("Starting map");
    println!("{}", game);
    let mut moves = game.get_moves();
    while !moves.is_empty() {
        moves = game.get_moves(); 
        if !moves.is_empty() {
            // println!("Moves: {:#?}", moves.len());
            let idx = rng.random_range(0..moves.len());
            let chosen_move = moves.swap_remove(idx);

            println!("Move {}", chosen_move.get_move_num());
            println!("Playing move: {:#?}", chosen_move);
            println!("{}", game);
            println!();
            game.play_move(chosen_move)
                .unwrap_or_else(|err| panic!("Bład gry: {}", err));
        }
    }   
}