use std::sync::LazyLock;

use crate::engine::tile::{Feature, Rotation, Tile};
use crate::engine::datastructures::tile_features::TileFeatures;
use crate::engine::tile_set::TileSet;

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
                city_road: false,
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
    (@flag $t:ident, CITY_ROAD) => { $t.city_road = true; };
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
static CCRC: LazyLock<Tile> = LazyLock::new(|| tile!(C C R C ROADS_DISCONNECTED CITY_ROAD));
static CCRC_PENNANT: LazyLock<Tile> = LazyLock::new(|| tile!(C C R C PENNANT ROADS_DISCONNECTED CITY_ROAD));

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
        *FFFF_CLOISTER,
        *FFFF_CLOISTER,
        *FFFF_CLOISTER,
        *FFFF_CLOISTER,
        // 5x
        *CFFF,
        *CFFF,
        *CFFF,
        *CFFF,
        *CFFF,
        // 3x
        *CFCF_DISCONNECTED,
        *CFCF_DISCONNECTED,
        *CFCF_DISCONNECTED,
        // 2x
        *CCFF_DISCONNECTED,
        *CCFF_DISCONNECTED,
        // 1x
        *FCFC,
        // 2x
        *FCFC_PENNANT,
        *FCFC_PENNANT,
        // 3x
        *CCFF,
        *CCFF,
        *CCFF,
        // 2x
        *CCFF_PENNANT,
        *CCFF_PENNANT,
        // 3x
        *CCFC,
        *CCFC,
        *CCFC,
        // 1x
        *CCFC_PENNANT,
        // 1x
        *CCCC_PENNANT,
        // 2x
        *FFRF_CLOISTER,
        *FFRF_CLOISTER,
        // 1x
        *CCRC,
        // 2x
        *CCRC_PENNANT,
        *CCRC_PENNANT,
        // 8x
        *FRFR,
        *FRFR,
        *FRFR,
        *FRFR,
        *FRFR,
        *FRFR,
        *FRFR,
        *FRFR,
        // 9x
        *FFRR,
        *FFRR,
        *FFRR,
        *FFRR,
        *FFRR,
        *FFRR,
        *FFRR,
        *FFRR,
        *FFRR,
        // 4x
        *CRFR,
        *CRFR,
        *CRFR,
        *CRFR,
        // 3x
        *CFRR,
        *CFRR,
        *CFRR,
        // 3x
        *CRRF,
        *CRRF,
        *CRRF,
        // 3x
        *CCRR,
        *CCRR,
        *CCRR,
        // 2x
        *CCRR_PENNANT,
        *CCRR_PENNANT,
        // 4x
        *FRRR,
        *FRRR,
        *FRRR,
        *FRRR,
        // 3x
        *CRRR,
        *CRRR,
        *CRRR,
        // 1x
        *RRRR,
    ],
));