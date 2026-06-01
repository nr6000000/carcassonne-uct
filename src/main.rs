use crate::{engines::{carcassonne_engine::CarcassonneEngine, random_engine::RandomEngine}, game::{game::{Game, PlayerId}, standard_tileset::STANDARD_TILESET}};

mod game;
mod engines;

fn main() {
    let tileset = STANDARD_TILESET.clone();
    let mut game = Game::new(&tileset, 2, 8);

    let mut random_engine = RandomEngine::new();
    
    let mut current_player_gen = game
        .get_players()
        .collect::<Vec<PlayerId>>()
        .into_iter()
        .cycle();

    println!("Starting map");
    println!("{}", game);

    loop {
        let current_player = current_player_gen.next().unwrap();
        let moves = game.get_moves(current_player); 
        if moves.len() == 0 {
            break;
        }
        // println!("Moves: {:#?}", moves);

        let chosen_move = random_engine.play_move(moves, current_player);

        println!("Move {}", chosen_move.get_move_num());
        println!("Playing move: {:#?}", chosen_move);
        game.play_move(chosen_move)
            .unwrap_or_else(|err| panic!("Bład gry: {}", err));
        println!("{}", game);
        println!();
    }   

    game.end_game();
    println!("Game over");
    println!("{}", game)
}
