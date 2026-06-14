use std::time::{Duration, Instant};

use crate::{
    engines::{
        carcassonne_engine::CarcassonneEngine,
        heurestic_engine::HeuresticEngine,
        uct_engine::UctEngine,
    },
    game_logic::{
        game::{Game, GameSettings, PlayerId},
        standard_tileset::STANDARD_TILESET,
    },
};

mod game_logic;
mod engines;

type EngineFactory = Box<dyn Fn() -> Box<dyn CarcassonneEngine>>;

struct EngineDef {
    name: &'static str,
    factory: EngineFactory,
}

// Returns (score1, score2, think_time1, think_time2)
fn run_game(
    factory1: &EngineFactory,
    factory2: &EngineFactory,
    engine1_first: bool,
) -> (u32, u32, Duration, Duration) {
    let tileset = STANDARD_TILESET.clone();
    let mut settings = GameSettings::default();
    settings.farmers_enabled = false;
    let mut game = Game::new(&tileset, settings);

    let players: Vec<PlayerId> = game.get_players().collect();
    let player1 = players[0];
    let player2 = players[1];

    let mut engine_a = factory1();
    let mut engine_b = factory2();
    let mut time_a = Duration::ZERO;
    let mut time_b = Duration::ZERO;

    loop {
        let current = game.get_current_player();
        let use_a = (current == player1) == engine1_first;

        let t = Instant::now();
        let mov = if use_a {
            engine_a.play_move(&mut game)
        } else {
            engine_b.play_move(&mut game)
        };
        if use_a { time_a += t.elapsed(); } else { time_b += t.elapsed(); }

        game.play_move(mov).unwrap();

        if game.tiles_left.len() == 0 {
            break;
        }
    }

    game.end_game();
    let scores = game.get_score();

    // time_a = engine1 (engine_a), time_b = engine2 (engine_b) — zawsze
    if engine1_first {
        (scores[&player1], scores[&player2], time_a, time_b)
    } else {
        (scores[&player2], scores[&player1], time_a, time_b)
    }
}

// Returns (wins1, wins2, draws)
fn run_matchup(
    name1: &str,
    factory1: &EngineFactory,
    name2: &str,
    factory2: &EngineFactory,
    n_games: usize,
) -> (usize, usize, usize) {
    let mut wins1 = 0;
    let mut wins2 = 0;
    let mut draws = 0;
    let mut scores1_sum = 0u32;
    let mut scores2_sum = 0u32;

    println!("  >> {} vs {}", name1, name2);

    let start = Instant::now();

    for i in 0..n_games {
        let game_start = Instant::now();
        let (s1, s2, t1, t2) = run_game(factory1, factory2, i % 2 == 0);
        scores1_sum += s1;
        scores2_sum += s2;
        let result = if s1 > s2 {
            wins1 += 1;
            "1 wygrywa"
        } else if s2 > s1 {
            wins2 += 1;
            "2 wygrywa"
        } else {
            draws += 1;
            "remis"
        };
        println!(
            "     [{:2}/{}] wynik: {:>3} - {:>3}  ({})  czas: {:.1}s vs {:.1}s  [lacznie {:.1}s]",
            i + 1, n_games, s1, s2, result,
            t1.as_secs_f64(), t2.as_secs_f64(),
            game_start.elapsed().as_secs_f64()
        );
    }

    let elapsed = start.elapsed();
    println!(
        "  WYNIK: {} W:{} D:{} L:{}  |  {} W:{} D:{} L:{}   sr.pkt: {:.1} vs {:.1}  [{:.0}s lacznie]\n",
        name1, wins1, draws, wins2,
        name2, wins2, draws, wins1,
        scores1_sum as f64 / n_games as f64,
        scores2_sum as f64 / n_games as f64,
        elapsed.as_secs_f64()
    );

    (wins1, wins2, draws)
}

fn round_robin(engines: &[EngineDef], n_games: usize) {
    // punkty: 2 za wygrana, 1 za remis, 0 za przegrana
    let mut points: Vec<usize> = vec![0; engines.len()];
    let mut total_games: Vec<usize> = vec![0; engines.len()];

    for i in 0..engines.len() {
        for j in (i + 1)..engines.len() {
            let (w1, w2, d) = run_matchup(
                engines[i].name,
                &engines[i].factory,
                engines[j].name,
                &engines[j].factory,
                n_games,
            );
            points[i] += w1 * 2 + d;
            points[j] += w2 * 2 + d;
            total_games[i] += n_games;
            total_games[j] += n_games;
        }
    }

    let max_points: Vec<usize> = total_games.iter().map(|g| g * 2).collect();

    println!("  Ranking:");
    let mut ranking: Vec<usize> = (0..engines.len()).collect();
    ranking.sort_by(|&a, &b| points[b].cmp(&points[a]));

    for (rank, &idx) in ranking.iter().enumerate() {
        println!(
            "    {}. {:<22}  {:>3}/{:>3} pkt  ({} gier)",
            rank + 1,
            engines[idx].name,
            points[idx],
            max_points[idx],
            total_games[idx],
        );
    }
    println!();
}

fn main() {
    const N: usize = 3;

    println!("=== Carcassonne Engine Tournament ===");
    println!("Ustawienia: rolnicy wylaczeni, 2 graczy, {} gier per para\n", N);

    // ── HIPOTEZA 1 ────────────────────────────────────────────────────────────
    // Budzety czasowe wyrownane na podstawie pomiarow:
    //   RAVE(2000) ≘ UCT(2000) ≘ UCT-ECO(3400,d=10) pod wzgledem czasu/ruch
    println!("================================================================");
    println!("HIPOTEZA 1 — Ranking (rowny budzet czasowy ~17s/silnik/gre)");
    println!("  Oczekiwany ranking: UCT-ECO > RAVE > UCT > Heurystyka");
    println!("================================================================");

    let h1: Vec<EngineDef> = vec![
        EngineDef { name: "UCT-ECO(3400,d=10)", factory: Box::new(|| Box::new(UctEngine::new_eco(3400, 10))) },
        EngineDef { name: "RAVE(2000,k=300)",   factory: Box::new(|| Box::new(UctEngine::new_rave(2000, 300.0))) },
        EngineDef { name: "UCT(2000)",           factory: Box::new(|| Box::new(UctEngine::new_basic(2000))) },
        EngineDef { name: "Heurystyka",          factory: Box::new(|| Box::new(HeuresticEngine::new())) },
    ];
    round_robin(&h1, N);

    // ── HIPOTEZA 2a ───────────────────────────────────────────────────────────
    // ~50 iteracji per silnik (ECO dostaje proporcjonalnie wiecej bo tansze)
    println!("================================================================");
    println!("HIPOTEZA 2a — Mala liczba symulacji (~50 iteracji UCT/RAVE)");
    println!("  Oczekiwane: RAVE i UCT-ECO najlepsze przy malej liczbie sym.");
    println!("================================================================");

    let h2a: Vec<EngineDef> = vec![
        EngineDef { name: "UCT-ECO(170,d=10)", factory: Box::new(|| Box::new(UctEngine::new_eco(170, 10))) },
        EngineDef { name: "RAVE(50,k=20)",     factory: Box::new(|| Box::new(UctEngine::new_rave(50, 20.0))) },
        EngineDef { name: "UCT(50)",            factory: Box::new(|| Box::new(UctEngine::new_basic(50))) },
        EngineDef { name: "Heurystyka",         factory: Box::new(|| Box::new(HeuresticEngine::new())) },
    ];
    round_robin(&h2a, N);

    // ── HIPOTEZA 2b ───────────────────────────────────────────────────────────
    // ~1000 iteracji per silnik (rowny budzet)
    println!("================================================================");
    println!("HIPOTEZA 2b — Duza liczba symulacji (~1000 iteracji UCT/RAVE)");
    println!("  Oczekiwane: UCT zbliza sie do RAVE i UCT-ECO");
    println!("================================================================");

    let h2b: Vec<EngineDef> = vec![
        EngineDef { name: "UCT-ECO(1700,d=10)", factory: Box::new(|| Box::new(UctEngine::new_eco(1700, 10))) },
        EngineDef { name: "RAVE(1000,k=200)",   factory: Box::new(|| Box::new(UctEngine::new_rave(1000, 200.0))) },
        EngineDef { name: "UCT(1000)",           factory: Box::new(|| Box::new(UctEngine::new_basic(1000))) },
    ];
    round_robin(&h2b, N);

    println!("================================================================");
    println!("HIPOTEZY 3 i 4 wymagaja danych z gier z czlowiekiem (UI).");
    println!("================================================================");
}
