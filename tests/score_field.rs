use carcossonne_uct::game_logic::{game::{Game, Place, PlayerId}, standard_tileset::STANDARD_TILESET};

use crate::common::play_move_pic;

mod common;

#[test]
fn score_field() {
    let tileset = STANDARD_TILESET.clone();
    let mut game = Game::new(&tileset, 2, 8);
    
    let players_ids = game
        .get_players()
        .collect::<Vec<PlayerId>>();
    let [player1, player2] = players_ids.as_slice() else {
        panic!();
    };

    play_move_pic(&mut game, &Place{x: 0, y: -1}, player1,
       "········██\n\
        ········▒▒\n\
        ····🯅···▒▒\n\
        ········▒▒\n\
        ██▒▒▒▒▒▒██\n".to_owned()
    );

    play_move_pic(&mut game, &Place{x: -1, y: -1}, player2,
       "····░░····\n\
        ····░░····\n\
        ░░░░░░····\n\
        ··········\n\
        ··········\n".to_owned()
    );

    play_move_pic(&mut game, &Place{x: 1, y: -1}, player1,
       "██········\n\
        ▒▒········\n\
        ▒▒▒▒······\n\
        ▒▒▒▒▒▒····\n\
        ▒▒▒▒▒▒▒▒██\n".to_owned()
    );

    play_move_pic(&mut game, &Place{x: -1, y: -2}, player2,
       "··········\n\
        ··········\n\
        ░░░░░░····\n\
        ····░░····\n\
        ····░░····\n".to_owned()
    );

    play_move_pic(&mut game, &Place{x: 0, y: -2}, player1,
       "········██\n\
        ········▒▒\n\
        ········▒▒\n\
        ········▒▒\n\
        ········██\n".to_owned()
    );

    play_move_pic(&mut game, &Place{x: 1, y: -2}, player2,
       "██······██\n\
        ▒▒······▒▒\n\
        ▒▒······▒▒\n\
        ▒▒······▒▒\n\
        ██······██\n".to_owned()
    );

    play_move_pic(&mut game, &Place{x: 2, y: -2}, player1,
       "██········\n\
        ▒▒········\n\
        ▒▒········\n\
        ▒▒········\n\
        ██········\n".to_owned()
    );

    play_move_pic(&mut game, &Place{x: -1, y: -3}, player2,
       "····░░····\n\
        ····░░····\n\
        ░░░░██░░░░\n\
        ··········\n\
        ··········\n".to_owned()
    );

    play_move_pic(&mut game, &Place{x: 0, y: -3}, player1,
       "██▒▒▒▒▒▒██\n\
        ··········\n\
        ░░░░░░░░░░\n\
        ··········\n\
        ··········\n".to_owned()
    );

    play_move_pic(&mut game, &Place{x: -2, y: -3}, player2,
       "▓▓▓▓▓▓▓▓██\n\
        ▓▓▓▓▓▓▓▓··\n\
        ▓▓▓▓▓▓▓▓░░\n\
        ▓▓▓▓▓▓▓▓··\n\
        ▓▓▓▓▓▓▓▓██\n".to_owned()
    );

    play_move_pic(&mut game, &Place{x: -1, y: -4}, player1,
       "····░░··██\n\
        ····░░··▒▒\n\
        ░░░░██··▒▒\n\
        ····░░··▒▒\n\
        ····░░··██\n".to_owned()
    );

    play_move_pic(&mut game, &Place{x: 0, y: -4}, player1,
       "██········\n\
        ▓▓········\n\
        ▓▓▓▓······\n\
        ▓▓▓▓▓▓····\n\
        ▓▓▓▓▓▓▓▓██\n".to_owned()
    );

    game.end_game();

    assert_eq!(6, game.get_score()[player1]);
    assert_eq!(0, game.get_score()[player2]);
}


// ·· ░░ ▒▒ ▓▓ ██ ✝⌂ 🯅