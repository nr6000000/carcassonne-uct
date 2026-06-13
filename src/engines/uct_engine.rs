use rand::rngs::ThreadRng;
use rand::seq::IndexedRandom;
use rapidhash::RapidHashSet;
use tinyvec::{TinyVec};

use crate::engines::carcassonne_engine::CarcassonneEngine;
use crate::game_logic::game::{Game, Move, Place, PlayerId, Rotation};
use crate::game_logic::tile::TileId;

pub struct UctEngine {
    pub iterations: u32,
    pub c_constant: f32,
    pub k_constant: f32,
    pub rave: bool,
    nodes: Vec<UctNode>,
    root: usize,
    rng: ThreadRng,
}

struct UctNode {
    mov: Move,
    children: Vec<usize>,
    parent: usize,
    games: u32,
    eval: f32,
    rave_games: u32,
    rave_eval: f32,
}

impl UctNode {
    fn create(mov: Move, parent: usize) -> Self {
        Self {
            mov,
            children: Vec::new(),
            parent,
            games: 0,
            eval: 0.,
            rave_games: 0,
            rave_eval: 0.,
        }
    }
}

const EMPIRICAL_MAX_SCORE: u32 = 100;
const BRANCHING_FACTOR: u32 = 55;

fn normalize_score(our_score: u32, opponent_score: u32) -> f32 {
    // let score_diff = our_score as f32 - opponent_score as f32;
    // let normalized_score_diff = score_diff as f32 / (2.*EMPIRICAL_MAX_SCORE as f32) + 0.5;
    // // println!("{}", normalized_score_diff);
    // normalized_score_diff.clamp(0., 1.)
    // let bias = ((our_score-opponent_score+EMPIRICAL_MAX_SCORE) as f32 / 
    //     (2*EMPIRICAL_MAX_SCORE) as f32) / 4.;
    
    // let eval = match our_score.cmp(&opponent_score) {
    //     std::cmp::Ordering::Less => 0.,
    //     std::cmp::Ordering::Equal => 0.5,
    //     std::cmp::Ordering::Greater => 1.,
    // };
    // eval

    // eval.clamp(0., 2.)

    let diff = our_score as f32 - opponent_score as f32;
    // temperature controls how fast it saturates to 0/1
    let temperature = 10.; 
    1. / (1. + (-diff / temperature).exp())
}

impl UctEngine {
    pub fn new(iterations: u32, c_constant: f32, k_constant: f32, rave: bool) -> Self {
        Self { 
            iterations,
            c_constant,
            k_constant,
            nodes: Vec::new(),
            root: 0,
            rng: rand::rng(),
            rave,
        }
    }

    pub fn new_basic(iterations: u32) -> UctEngine {
        Self::new(iterations, 2f32.sqrt(), 0., false)
    }

    pub fn new_rave(iterations: u32, k_constant: f32) -> UctEngine {
        Self::new(iterations, 2f32.sqrt(), k_constant, true)
    }

    fn restart(&mut self, game: &mut Game) {
        self.nodes = Vec::new();

        let dummy_move = Move {
            move_num: 0,
            place: Place{x: 0, y: 0},
            tile: TileId(0),
            rotation: Rotation::Rot0,
            follower: None,
            player: PlayerId(0),
        };
        let root = UctNode::create(dummy_move, 0);
        self.nodes.push(root);
        self.expand(self.nodes.len()-1, game);
    }

    fn uct(&self, parent: usize, child: usize) -> f32 {
        let eval = self.nodes[child].eval as f32;
        let child_games = self.nodes[child].games as f32;
        let parent_games = self.nodes[parent].games as f32;
        let c = self.c_constant;

        if child_games == 0. {
            return f32::MAX;
        }

        if self.rave {
            let k = self.k_constant;
            let beta = (k / (3.*child_games + k)).sqrt();
            let rave_eval = self.nodes[child].rave_eval as f32;
            let rave_games = self.nodes[child].rave_games as f32;

            if rave_games == 0. {
                return f32::MAX;
            }

            (1.-beta) * eval/child_games + 
            beta * rave_eval/rave_games + 
            c * (parent_games.ln()/child_games).sqrt()
        } else {
            eval/child_games + c*(parent_games.ln()/child_games).sqrt()
        }
    }

    fn select(&mut self, game: &mut Game) -> usize {
        let mut current_id = self.root;

        loop {
            let current = &self.nodes[current_id];
            
            if current.children.len() == 0 {
                if current.games == 0 {
                    // Leaf node
                    break;
                } else {
                    if !self.expand(current_id, game) {
                        break;
                    }
                }
            } else {
                // let next_id = current.children.iter()
                //     .max_by_key(|node_id| self.uct(current_id, **node_id).to_bits()).unwrap();

                // Randomly break ties among unvisited children
                let unvisited: Vec<_> = current.children.iter()
                    .filter(|id| self.nodes[**id].games == 0)
                    .collect();

                let next_id = if !unvisited.is_empty() {
                    *unvisited.choose(&mut self.rng).unwrap()
                } else {
                    current.children.iter()
                        .max_by_key(|id| self.uct(current_id, **id).to_bits())
                        .unwrap()
                };

                game.play_move(self.nodes[*next_id].mov).unwrap();
                current_id = *next_id;
            }
        }

        current_id
    }

    fn expand(&mut self, node_id: usize, game: &mut Game) -> bool {
        let (moves, _) = game.get_moves();
        if moves.len() == 0 {
            return false;
        }

        moves.iter()
            .map(|mov| UctNode::create(*mov, node_id))
            .for_each(|node| {
                self.nodes.push(node);
                let child_id = self.nodes.len()-1;
                self.nodes[node_id].children.push(child_id);
            });
        
        true
    }

    fn backpropagate(&mut self, start: usize, mut eval: f32) {
        let mut current_id = start;
        loop {
            self.nodes[current_id].games += 1;
            self.nodes[current_id].eval += eval;

            if current_id == self.root {
                break;
            }
            current_id = self.nodes[current_id].parent;
            eval = 1. - eval;
        }
    }

    fn backpropagate_rave(&mut self, start: usize, mut eval: f32, rave_set: RapidHashSet<Move>) {
        let mut current_id = start;
        loop {
            self.nodes[current_id].games += 1;
            self.nodes[current_id].eval += eval;

            let children: TinyVec<[usize; BRANCHING_FACTOR as usize]> = self.nodes[current_id]
                .children.iter().copied().collect();

            for child_id in children {
                if rave_set.contains(&self.nodes[child_id].mov) {
                    self.nodes[child_id].rave_games += 1;
                    self.nodes[child_id].rave_eval += 1. - eval;
                }
            }

            if current_id == self.root {
                break;
            }
            current_id = self.nodes[current_id].parent;
            eval = 1. - eval;
        }
    }

    fn get_eval(&self, game: &Game, current_id: usize) -> f32 {
        assert!(self.root != current_id);

        let score = game.get_score();

        let current_player = self.nodes[current_id].mov.player;
        let other_player = game
            .get_players()
            .skip_while(|p| *p == current_player)
            .next().unwrap();

        normalize_score(score[&current_player], score[&other_player])
    }

    fn play_once(&mut self, mut game: Game) {
        let current_id = self.select(&mut game);
        if self.rave {
            let moves = game.play_random_game_get_moves();
            let eval = self.get_eval(&game, current_id);
            self.backpropagate_rave(current_id, eval, moves);
        } else {
            game.play_random_game();
            let eval = self.get_eval(&game, current_id);
            self.backpropagate(current_id, eval);
        }
    }
}

impl CarcassonneEngine for UctEngine {
    fn play_move(
        &mut self, 
        game: &mut Game,
    ) -> Move {
        self.restart(game);
        
        for _i in 0..self.iterations {
            // println!("{}", _i);
            self.play_once(game.clone());
        }

        let eval = self.nodes[self.root].eval;
        let games = self.nodes[self.root].games;
        println!(
            "Engine eval: {}/{} - {:.2}%", 
            games as f32 - eval,
            games,
            (1. - (eval/games as f32)) * 100.,
        );

        let chosen_node = *self.nodes[self.root].children.iter()
            .max_by_key(|node_id| {
                let node = &self.nodes[**node_id];
                node.games
            }).unwrap();
        
        self.nodes[chosen_node].mov
    }
}
