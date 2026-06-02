use std::collections::HashMap;

use crate::{engines::{carcassonne_engine::CarcassonneEngine, greedy_engine::GreedyEngine, random_engine::RandomEngine}, game_logic::{game::{Game, PlayerId}, standard_tileset::STANDARD_TILESET}};

mod game_logic;
mod engines;

fn main() {
    let tileset = STANDARD_TILESET.clone();
    let mut game = Game::new(&tileset, 2, 8);

    let random_engine = RandomEngine::new();
    let greedy_engine = GreedyEngine::new();
    
    let players_ids = game
        .get_players()
        .collect::<Vec<PlayerId>>();
    let [player1, player2] = players_ids.as_slice() else {
        panic!();
    };

    let mut current_player_gen = [
        player1,
        player2,
    ].into_iter().cycle();

    let mut players = HashMap::from([
        (player1, Box::new(random_engine) as Box<dyn CarcassonneEngine>),
        (player2, Box::new(greedy_engine)),
    ]);

    println!("Starting map");
    println!("{}", game);

    loop {
        let current_player = current_player_gen.next().unwrap();
        let (moves, structures) = game.get_moves(*current_player); 
        if moves.len() == 0 {
            break;
        }
        // println!("Moves: {:#?}", moves);

        let chosen_move = players.get_mut(current_player).unwrap()
            .play_move(moves, structures, *current_player);

        println!("Move {}", chosen_move.get_move_num());
        println!("Playing move: {:#?}", chosen_move);
        game.play_move(chosen_move)
            .unwrap_or_else(|err| panic!("Bład gry: {}", err));
        println!("{}", game);
        println!();
    }   

    game.end_game();
    println!("Game over");
    println!("{}", game);
}
