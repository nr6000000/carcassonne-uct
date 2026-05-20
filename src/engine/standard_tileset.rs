use std::sync::LazyLock;

use crate::engine::tile::{Feature, Rotation, Tile, TileSet};
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

static FFFF_CLOISTER: LazyLock<Tile> = LazyLock::new(|| tile!(F F F F CLOISTER));
static CFFF: LazyLock<Tile> = LazyLock::new(|| tile!(C F F F));
static CFCF_DISCONNECTED: LazyLock<Tile> = LazyLock::new(|| tile!(C F C F CITIES_DISCONNECTED));
static CCFF_DISCONNECTED: LazyLock<Tile> = LazyLock::new(|| tile!(C C F F CITIES_DISCONNECTED));
static FCFC: LazyLock<Tile> = LazyLock::new(|| tile!(F C F C));
static FCFC_PENNANT: LazyLock<Tile> = LazyLock::new(|| tile!(F C F C PENNANT));
static CCFF: LazyLock<Tile> = LazyLock::new(|| tile!(C C F F));
static CCFF_PENNANT: LazyLock<Tile> = LazyLock::new(|| tile!(C C F F PENNANT));
static CCFC: LazyLock<Tile> = LazyLock::new(|| tile!(C C F C));
static CCFC_PENNANT: LazyLock<Tile> = LazyLock::new(|| tile!(C C F C PENNANT));
static CCCC_PENNANT: LazyLock<Tile> = LazyLock::new(|| tile!(C C C C PENNANT));

static FFRF_CLOISTER: LazyLock<Tile> = LazyLock::new(|| tile!(F F R F CLOISTER ROADS_DISCONNECTED));
static CCRC: LazyLock<Tile> = LazyLock::new(|| tile!(C C R C));
static CCRC_PENNANT: LazyLock<Tile> = LazyLock::new(|| tile!(C C R C PENNANT));

static FRFR: LazyLock<Tile> = LazyLock::new(|| tile!(F R F R));
static FFRR: LazyLock<Tile> = LazyLock::new(|| tile!(F F R R));
static CRFR: LazyLock<Tile> = LazyLock::new(|| tile!(C R F R));
static CFRR: LazyLock<Tile> = LazyLock::new(|| tile!(C F R R));
static CRRF: LazyLock<Tile> = LazyLock::new(|| tile!(C R R F));
static CCRR: LazyLock<Tile> = LazyLock::new(|| tile!(C C R R));
static CCRR_PENNANT: LazyLock<Tile> = LazyLock::new(|| tile!(C C R R PENNANT));

static FRRR: LazyLock<Tile> = LazyLock::new(|| tile!(F R R R ROADS_DISCONNECTED));
static CRRR: LazyLock<Tile> = LazyLock::new(|| tile!(C R R R ROADS_DISCONNECTED));
static RRRR: LazyLock<Tile> = LazyLock::new(|| tile!(R R R R ROADS_DISCONNECTED));

pub static STANDARD_TILESET: LazyLock<TileSet> = LazyLock::new(|| TileSet::new(
    CRFR.fix_rotation(&Rotation::Rot0),
    vec![
        // 4x
        FFFF_CLOISTER.clone(),
        FFFF_CLOISTER.clone(),
        FFFF_CLOISTER.clone(),
        FFFF_CLOISTER.clone(),
        // 5x
        CFFF.clone(),
        CFFF.clone(),
        CFFF.clone(),
        CFFF.clone(),
        CFFF.clone(),
        // 3x
        CFCF_DISCONNECTED.clone(),
        CFCF_DISCONNECTED.clone(),
        CFCF_DISCONNECTED.clone(),
        // 2x
        CCFF_DISCONNECTED.clone(),
        CCFF_DISCONNECTED.clone(),
        // 1x
        FCFC.clone(),
        // 2x
        FCFC_PENNANT.clone(),
        FCFC_PENNANT.clone(),
        // 3x
        CCFF.clone(),
        CCFF.clone(),
        CCFF.clone(),
        // 2x
        CCFF_PENNANT.clone(),
        CCFF_PENNANT.clone(),
        // 3x
        CCFC.clone(),
        CCFC.clone(),
        CCFC.clone(),
        // 1x
        CCFC_PENNANT.clone(),
        // 1x
        CCCC_PENNANT.clone(),
        // 2x
        FFRF_CLOISTER.clone(),
        FFRF_CLOISTER.clone(),
        // 1x
        CCRC.clone(),
        // 2x
        CCRC_PENNANT.clone(),
        CCRC_PENNANT.clone(),
        // 8x
        FRFR.clone(),
        FRFR.clone(),
        FRFR.clone(),
        FRFR.clone(),
        FRFR.clone(),
        FRFR.clone(),
        FRFR.clone(),
        FRFR.clone(),
        // 9x
        FFRR.clone(),
        FFRR.clone(),
        FFRR.clone(),
        FFRR.clone(),
        FFRR.clone(),
        FFRR.clone(),
        FFRR.clone(),
        FFRR.clone(),
        FFRR.clone(),
        // 4x
        CRFR.clone(),
        CRFR.clone(),
        CRFR.clone(),
        CRFR.clone(),
        // 3x
        CFRR.clone(),
        CFRR.clone(),
        CFRR.clone(),
        // 3x
        CRRF.clone(),
        CRRF.clone(),
        CRRF.clone(),
        // 3x
        CCRR.clone(),
        CCRR.clone(),
        CCRR.clone(),
        // 2x
        CCRR_PENNANT.clone(),
        CCRR_PENNANT.clone(),
        // 4x
        FRRR.clone(),
        FRRR.clone(),
        FRRR.clone(),
        FRRR.clone(),
        // 3x
        CRRR.clone(),
        CRRR.clone(),
        CRRR.clone(),
        // 1x
        RRRR.clone(),
    ],
));