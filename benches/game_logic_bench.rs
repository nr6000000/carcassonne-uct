use criterion::{Criterion, Throughput, criterion_group, criterion_main};

// use carcossonne_uct::{engines::{carcassonne_engine::CarcassonneEngine, random_engine::RandomEngine}, game_logic::{game::{Game, PlayerId}, standard_tileset::STANDARD_TILESET}};

// fn play_game(game: &mut Game, engine: &mut impl CarcassonneEngine) {
//     let mut current_player_gen = game
//         .get_players()
//         .collect::<Vec<PlayerId>>()
//         .into_iter()
//         .cycle();

//     loop {
//         let current_player = current_player_gen.next().unwrap();
//         let (moves, structures) = game.get_moves(current_player); 
//         if moves.len() == 0 {
//             break;
//         }

//         let chosen_move = engine.play_move(moves, structures, current_player);

//         game.play_move(chosen_move)
//             .unwrap_or_else(|err| panic!("Bład gry: {}", err));
//     }   

//     game.end_game();
// }

fn bench(c: &mut Criterion) {
    // let tileset = STANDARD_TILESET.clone();
    // let mut game = Game::new(&tileset, 2, 8);
    // let mut random_engine = RandomEngine::new();

    let mut group = c.benchmark_group("throughput-example");
    group.throughput(Throughput::Elements(1));
    // group.bench_function("play_game", |b| b.iter(|| play_game(&mut game, &mut random_engine)));
    group.bench_function("play_game", |b| b.iter(|| {}));
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
