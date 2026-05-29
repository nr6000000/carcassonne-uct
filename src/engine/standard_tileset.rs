use std::sync::LazyLock;

use tileset_format::TileSet;

static STANDARD_TILESET_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/tileset.bin"));

pub static STANDARD_TILESET: LazyLock<TileSet> = LazyLock::new(|| {
    postcard::from_bytes(STANDARD_TILESET_BYTES).unwrap()
});
