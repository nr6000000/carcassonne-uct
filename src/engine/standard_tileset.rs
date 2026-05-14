use crate::engine::tile::{Feature, FixedTile, Rotation, Tile, TileSet};
use crate::engine::tile_features::TileFeatures;

macro_rules! feature {
    (C) => { Feature::City  };
    (R) => { Feature::Road  };
    (F) => { Feature::Field };
}

macro_rules! tile {
    // Entry point
    ($n:ident $e:ident $s:ident $w:ident $($flags:ident)*) => {
        {
            let mut t = Tile {
                features: TileFeatures::new(
                    feature!($n),
                    feature!($e),
                    feature!($s),
                    feature!($w),
                ),
                cloister: false,
                pennant: false,
                cities_connected: true,
                roads_connected: true,
            };
            // Silent mut warning
            t.cloister = false;
            $( tile!(@flag t, $flags); )*
            t
        }
    };
    // Flag dispatch
    (@flag $t:ident, CLOISTER) => { $t.cloister = true; };
    (@flag $t:ident, PENNANT) => { $t.pennant = true; };
    (@flag $t:ident, CITIES_DISCONNECTED) => { $t.cities_connected = false; };
    (@flag $t:ident, ROADS_DISCONNECTED) => { $t.roads_connected = false; };
}

pub struct StandardTileSet {
    starting_tile: FixedTile,
    tiles: Vec<Tile>,
}

impl StandardTileSet {
    #[allow(non_snake_case)]
    pub fn new() -> Self {
        // Kolejność jak w https://en.wikipedia.org/wiki/Carcassonne_(board_game)#Tiles
        let FFFF_CLOISTER: Tile = tile!(F F F F CLOISTER);
        let CFFF: Tile = tile!(C F F F);
        let CFCF_DISCONNECTED: Tile = tile!(C F C F CITIES_DISCONNECTED);
        let CCFF_DISCONNECTED: Tile = tile!(C C F F CITIES_DISCONNECTED);
        let FCFC: Tile = tile!(F C F C);
        let FCFC_PENNANT: Tile = tile!(F C F C PENNANT);
        let CCFF: Tile = tile!(C C F F);
        let CCFF_PENNANT: Tile = tile!(C C F F PENNANT);
        let CCFC: Tile = tile!(C C F C);
        let CCFC_PENNANT: Tile = tile!(C C F C PENNANT);
        let CCCC_PENNANT: Tile = tile!(C C C C PENNANT);

        let FFRF_CLOISTER: Tile = tile!(F F R F CLOISTER);
        let CCRC: Tile = tile!(C C R C);
        let CCRC_PENNANT: Tile = tile!(C C R C PENNANT);

        let FRFR: Tile = tile!(F R F R);
        let FFRR: Tile = tile!(F F R R);
        let CRFR: Tile = tile!(C R F R);
        let CFRR: Tile = tile!(C F R R);
        let CRRF: Tile = tile!(C R R F);
        let CCRR: Tile = tile!(C C R R);
        let CCRR_PENNANT: Tile = tile!(C C R R PENNANT);

        let FRRR: Tile = tile!(F R R R ROADS_DISCONNECTED);
        let CRRR: Tile = tile!(C R R R ROADS_DISCONNECTED);
        let RRRR: Tile = tile!(R R R R ROADS_DISCONNECTED);

        let starting_tile = CRFR.fix_rotation(&Rotation::Rot0);
        let tiles = vec![
            // 4x
            FFFF_CLOISTER,
            FFFF_CLOISTER,
            FFFF_CLOISTER,
            FFFF_CLOISTER,
            // 5x
            CFFF,
            CFFF,
            CFFF,
            CFFF,
            CFFF,
            // 3x
            CFCF_DISCONNECTED,
            CFCF_DISCONNECTED,
            CFCF_DISCONNECTED,
            // 2x
            CCFF_DISCONNECTED,
            CCFF_DISCONNECTED,
            // 1x
            FCFC,
            // 2x
            FCFC_PENNANT,
            FCFC_PENNANT,
            // 3x
            CCFF,
            CCFF,
            CCFF,
            // 2x
            CCFF_PENNANT,
            CCFF_PENNANT,
            // 3x
            CCFC,
            CCFC,
            CCFC,
            // 1x
            CCFC_PENNANT,
            // 1x
            CCCC_PENNANT,
            // 2x
            FFRF_CLOISTER,
            FFRF_CLOISTER,
            // 1x
            CCRC,
            // 2x
            CCRC_PENNANT,
            CCRC_PENNANT,
            // 8x
            FRFR,
            FRFR,
            FRFR,
            FRFR,
            FRFR,
            FRFR,
            FRFR,
            FRFR,
            // 9x
            FFRR,
            FFRR,
            FFRR,
            FFRR,
            FFRR,
            FFRR,
            FFRR,
            FFRR,
            FFRR,
            // 4x
            CRFR,
            CRFR,
            CRFR,
            CRFR,
            // 3x
            CFRR,
            CFRR,
            CFRR,
            // 3x
            CRRF,
            CRRF,
            CRRF,
            // 3x
            CCRR,
            CCRR,
            CCRR,
            // 2x
            CCRR_PENNANT,
            CCRR_PENNANT,
            // 4x
            FRRR,
            FRRR,
            FRRR,
            FRRR,
            // 3x
            CRRR,
            CRRR,
            CRRR,
            // 1x
            RRRR,
        ];

        Self { starting_tile, tiles }
    }
}

impl TileSet for StandardTileSet {
    fn starting_tile(&self) -> FixedTile {
        self.starting_tile
    }

    fn tiles(&self) -> &[Tile] {
        self.tiles.as_slice()
    }
}