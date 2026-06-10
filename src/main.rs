use rapidhash::RapidHashMap;

use crate::{engines::{carcassonne_engine::CarcassonneEngine, greedy_engine::{GreedyCompleterEngine}, random_engine::RandomEngine}, game_logic::{game::{Game, GameSettings, PlayerId}, standard_tileset::STANDARD_TILESET}};

mod game_logic;
mod engines;

fn main() {
    let tileset = STANDARD_TILESET.clone();
    let mut settings = GameSettings::default();
    settings.farmers_enabled = false;
    let mut game = Game::new(&tileset, settings);

    let random_engine = RandomEngine::new();
    let greedy_engine = GreedyCompleterEngine::new();
    
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

    let mut players = RapidHashMap::from_iter([
        (player1, Box::new(random_engine) as Box<dyn CarcassonneEngine>),
        (player2, Box::new(greedy_engine)),
    ]);

    println!("Starting map");
    println!("{}", game);

    loop {
        let current_player = current_player_gen.next().unwrap();
        // println!("Moves: {:#?}", moves);

        let chosen_move = players.get_mut(current_player).unwrap()
            .play_move(&mut game, *current_player);

        println!("Move {}", chosen_move.get_move_num());
        println!("Playing move: {:#?}", chosen_move);
        game.play_move(chosen_move)
            .unwrap_or_else(|err| panic!("Bład gry: {}", err));
        println!("{}", game);
        println!();

        if game.tiles_left.len() == 0 {
            break;
        }
    }   

    game.end_game();
    println!("Game over");
    println!("{}", game);
}
