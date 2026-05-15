use crate::engine::{game::Game, tile::TileSet};
use rand::prelude::IndexedRandom;

pub fn gen_random_game<T: TileSet>(game: &mut Game) {
    let mut rng = rand::rng();

    println!("Starting map");
    println!("{}", game);
    let mut moves = game.get_moves();
    while !moves.is_empty() {
        moves = game.get_moves(); 
        // println!("Moves: {:#?}", moves.len());
        let chosen_move = moves.choose(&mut rng);
        match chosen_move {
            Some(mov) => {
                game.play_move(mov)
                    .unwrap_or_else(|err| panic!("Bład gry: {}", err));
                println!("Move {}", mov.get_move_num());
                println!("Playing move: {:#?}", mov);
                println!("{}", game);
                println!();
            },
            None => break,
        }
    }   
}