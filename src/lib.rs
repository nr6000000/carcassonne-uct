pub mod game_logic;
pub mod engines;
mod js_binds;

use rapidhash::RapidHashMap;
use wasm_bindgen::prelude::*;

use crate::{engines::{carcassonne_engine::CarcassonneEngine, greedy_engine::GreedyCompleterEngine, random_engine::RandomEngine}, game_logic::{game::{Game, GameSettings, PlayerId}, standard_tileset::STANDARD_TILESET}, js_binds::log::log};

fn run() -> Game {
    let tileset = STANDARD_TILESET.clone();
    let mut game = Game::new(&tileset, GameSettings::default());

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

    loop {
        let current_player = current_player_gen.next().unwrap();

        let chosen_move = players.get_mut(current_player).unwrap()
            .play_move(&mut game);

        game.play_move(chosen_move)
            .unwrap_or_else(|err| panic!("Bład gry: {}", err));

        if game.tiles_left.len() == 0 {
            break;
        }
    }   

    game.end_game();
    game
}

#[wasm_bindgen]
pub fn wasm_test_main() {
    let game = run();
    log(&game.to_string());
}