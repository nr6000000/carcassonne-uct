use criterion::{Criterion, Throughput, criterion_group, criterion_main};

use carcossonne_uct::{engines::{carcassonne_engine::CarcassonneEngine, greedy_engine::GreedyEngine, random_engine::RandomEngine}, game_logic::{game::{Game, GameSettings, PlayerId}, standard_tileset::STANDARD_TILESET}};
use tileset_format::TileSet;

fn play_game_random(mut game: Game, mut engine: Box<dyn CarcassonneEngine>) {    
    let mut current_player_gen = game
        .get_players()
        .collect::<Vec<PlayerId>>()
        .into_iter()
        .cycle();

    loop {
        let current_player = current_player_gen.next().unwrap();
        let (moves, structures) = game.get_moves(current_player); 
        if moves.len() == 0 {
            break;
        }

        let chosen_move = engine.play_move(moves, structures, current_player);

        game.play_move(chosen_move)
            .unwrap_or_else(|err| panic!("Bład gry: {}", err));
    }   

    game.end_game();
}

fn play_game_random_nofields(mut game: Game, mut engine: Box<dyn CarcassonneEngine>) {
    let mut current_player_gen = game
        .get_players()
        .collect::<Vec<PlayerId>>()
        .into_iter()
        .cycle();

    loop {
        let current_player = current_player_gen.next().unwrap();
        let (moves, structures) = game.get_moves(current_player); 
        if moves.len() == 0 {
            break;
        }

        let chosen_move = engine.play_move(moves, structures, current_player);

        game.play_move(chosen_move)
            .unwrap_or_else(|err| panic!("Bład gry: {}", err));
    }   

    game.end_game();
}

fn play_game_greedy(mut game: Game, mut engine: Box<dyn CarcassonneEngine>) {    
    let mut current_player_gen = game
        .get_players()
        .collect::<Vec<PlayerId>>()
        .into_iter()
        .cycle();

    loop {
        let current_player = current_player_gen.next().unwrap();
        let (moves, structures) = game.get_moves(current_player); 
        if moves.len() == 0 {
            break;
        }

        let chosen_move = engine.play_move(moves, structures, current_player);

        game.play_move(chosen_move)
            .unwrap_or_else(|err| panic!("Bład gry: {}", err));
    }   

    game.end_game();
}

fn bench(c: &mut Criterion) {
    let tileset = STANDARD_TILESET.clone();
    let game = Game::new(&tileset, GameSettings::default());

    let mut settings_no_fields = GameSettings::default();
    settings_no_fields.calculate_move_score_field = false;
    let game_nofields = Game::new(&tileset, settings_no_fields);

    let mut group = c.benchmark_group("game-logic-throughput");
    group.throughput(Throughput::Elements(1));
    group.bench_function(
        "play_game_random",
        |b| b.iter(|| play_game_random(game.clone(), Box::new(RandomEngine::new())))
    );
    group.bench_function(
        "play_game_random_nofields", 
        |b| b.iter(|| play_game_random_nofields(game_nofields.clone(), Box::new(RandomEngine::new())))
    );
    group.bench_function(
        "play_game_greedy", |b| b.iter(|| play_game_greedy(game.clone(), Box::new(GreedyEngine::new())))
    );
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
