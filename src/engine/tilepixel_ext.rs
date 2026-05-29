use tileset_format::TilePixel;

pub trait TilePixelFits {
    fn fits(&self, other: &TilePixel) -> bool;
}

impl TilePixelFits for TilePixel {
    fn fits(&self, other: &TilePixel) -> bool {
        self == &TilePixel::Nothing ||
        other == &TilePixel::Nothing ||
        self == other ||
        self == &TilePixel::Blockade ||
        other == &TilePixel::Blockade ||
        (self == &TilePixel::City && other == &TilePixel::PennantCity) ||
        (self == &TilePixel::PennantCity && other == &TilePixel::City)
    }
}