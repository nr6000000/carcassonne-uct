pub mod game;
pub mod standard_tileset;
mod datastructures;
pub(crate) mod tilepixel_ext;
pub mod tile;
mod flood_fill;
pub mod structures;

pub use datastructures::index::Index;
pub use tileset_format::TilePixel;

type RapidIndexMap<K, V> = indexmap::IndexMap<K, V, rapidhash::fast::RandomState>;
type RapidIndexSet<T> = indexmap::IndexSet<T, rapidhash::fast::RandomState>;