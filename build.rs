use std::{env, fs, path::Path};

use tileset_format::load::{ParseError, parse_tilesets};

fn main() -> Result<(), ParseError> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("tileset.bin");

    let tilesets = parse_tilesets(Path::new("tilesets/standard"))?;
    let bytes = postcard::to_stdvec(&tilesets).unwrap();
    fs::write(out_path, bytes).unwrap();

    Ok(())
}