mod engine;

use crate::engine::{game::Game, random_game::gen_random_game, standard_tileset::StandardTileSet};

fn main() {
    let tileset = StandardTileSet::new();
    let mut game = Game::new(&tileset);
    gen_random_game::<StandardTileSet>(&mut game);
}
