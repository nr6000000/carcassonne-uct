use crate::engine::{game::Game, random_game::gen_random_game, standard_tileset::STANDARD_TILESET};

mod engine;

fn main() {
    let tileset = STANDARD_TILESET.clone();
    let mut game = Game::new(&tileset, 2);
    gen_random_game(&mut game);
}
