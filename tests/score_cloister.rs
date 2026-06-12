use carcossonne_uct::game_logic::{game::{Game, GameSettings, Place, PlayerId}, standard_tileset::STANDARD_TILESET};

use crate::common::play_move_pic;

mod common;

#[test]
fn score_cloister() {
    let tileset = STANDARD_TILESET.clone();
    let mut game = Game::new(&tileset, GameSettings::default());
    
    let players_ids = game
        .get_players()
        .collect::<Vec<PlayerId>>();
    let [player1, player2] = players_ids.as_slice() else {
        panic!();
    };

    play_move_pic(&mut game, &Place{x: 0, y: -1},
       "▒▒▒▒▒▒▒▒██\n\
        ▒▒▒▒▒▒▒▒··\n\
        ▒▒▒▒▒▒▒▒░░\n\
        ▒▒▒▒▒▒▒▒··\n\
        ▒▒▒▒▒▒▒▒██\n".to_owned()
    );

    play_move_pic(&mut game, &Place{x: 1, y: -1},
       "··········\n\
        ··········\n\
        ░░░░✝🯅····\n\
        ··········\n\
        ··········\n".to_owned()
    );

    play_move_pic(&mut game, &Place{x: 1, y: 0},
       "··········\n\
        ··········\n\
        ░░░░██░░░░\n\
        ····░░····\n\
        ····░░····\n".to_owned()
    );

    play_move_pic(&mut game, &Place{x: 2, y: 0},
       "····░░··██\n\
        ····░░··▒▒\n\
        ░░░░░░··▒▒\n\
        ········▒▒\n\
        ██▒▒▒▒▒▒▒▒\n".to_owned()
    );

    play_move_pic(&mut game, &Place{x: 2, y: -1},
       "··········\n\
        ··········\n\
        ····░░░░░░\n\
        ····░░····\n\
        ····░░····\n".to_owned()
    );

    play_move_pic(&mut game, &Place{x: 2, y: -2},
       "██▒▒▒▒▒▒██\n\
        ··········\n\
        ░░░░░░░░░░\n\
        ··········\n\
        ··········\n".to_owned()
    );

    play_move_pic(&mut game, &Place{x: 1, y: -2},
       "····░░····\n\
        ····░░····\n\
        ░░░░██░░░░\n\
        ··········\n\
        ··········\n".to_owned()
    );


    play_move_pic(&mut game, &Place{x: 0, y: -2},
       "██··░░····\n\
        ▓▓··░░····\n\
        ▓▓··░░░░░░\n\
        ▓▓········\n\
        ▓▓▓▓▓▓▓▓██\n".to_owned()
    );


    assert_eq!(0, game.get_score()[player1]);
    assert_eq!(9, game.get_score()[player2]);
    assert_eq!(8, game.get_followers()[player1]);
    assert_eq!(8, game.get_followers()[player2]);
}
