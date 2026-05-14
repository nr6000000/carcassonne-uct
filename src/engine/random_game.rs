use crate::engine::{game::Game, tile::TileSet};
use rand::prelude::IndexedRandom;

pub fn gen_random_game<T: TileSet>(game: &mut Game) {
    let mut rng = rand::rng();

    println!("Move 0");
    println!("{}", game);
    let mut moves = game.get_moves();
    let mut i = 1;
    while !moves.is_empty() {
        moves = game.get_moves(); 
        // println!("Moves: {:#?}", moves.len());
        let chosen_move = moves.choose(&mut rng);
        match chosen_move {
            Some(mov) => game.play_move(mov),
            None => break,
        }
        println!("Move {i}");
        println!("{}", game);
        i += 1;
    }   
}