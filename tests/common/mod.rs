use carcossonne_uct::game_logic::{Index, game::{Game, Place, PlayerId}, tile::Tile};
use tileset_format::{TILE_SIZE, TilePixel};

#[derive(Debug, Clone, Copy)]
pub struct MovePixel {
    pixel: TilePixel,
    follower: bool,
}

fn parse_tile(str: String) -> Result<([[TilePixel; TILE_SIZE]; TILE_SIZE], Option<Index>), ()> {
    let mut data_iter = str.chars();
    let mut pixels = [[TilePixel::Nothing; TILE_SIZE]; TILE_SIZE];
    let mut follower = None;

    for row in 0..TILE_SIZE {
        for column in 0..TILE_SIZE {
            let pixel1 = data_iter.next().ok_or(())?; 
            let pixel2 = data_iter.next().ok_or(())?;

            if pixel1 == '🯅' || pixel2 == '🯅' {
                follower = Some(Index{x: column as isize,  y: row as isize});
            }

            match (pixel1, pixel2) {
                ('·', '·') | ('·', '🯅') | ('🯅', '·') => pixels[row][column] = TilePixel::Field,
                ('░', '░') | ('░', '🯅') | ('🯅', '░') => pixels[row][column] = TilePixel::Road,
                ('▒', '▒') | ('▒', '🯅') | ('🯅', '▒') => pixels[row][column] = TilePixel::City,
                ('▓', '▓') | ('▓', '🯅') | ('🯅', '▓') => pixels[row][column] = TilePixel::PennantCity,
                ('█', '█') | ('█', '🯅') | ('🯅', '█') => pixels[row][column] = TilePixel::Blockade,
                ('✝', '⌂') | ('✝', '🯅') | ('🯅', '⌂') => pixels[row][column] = TilePixel::Cloister,
                _ => Err(())?,
            }
        }

        data_iter.next()
            .filter(|c| *c == '\n')
            .ok_or(())?;
    }

    Ok((pixels, follower))
}

pub fn play_move_pic(
    game: &mut Game, 
    place: &Place,
    player: &PlayerId,
    tile: String,
) {
    let (tile, follower) = parse_tile(tile).unwrap();
    game.play_custom_move(place, Tile::new(tile), follower, *player).unwrap();
}