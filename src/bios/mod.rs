#[cfg(test)]
mod tests;

mod diff;
mod huffman;
mod lz77;
mod rle;

pub use self::diff::{filter_diff8, unfilter_diff8};
pub use self::diff::{filter_diff16, unfilter_diff16};
pub use self::huffman::{compress_huffman, decompress_huffman};
pub use self::lz77::{compress_lz77, decompress_lz77};
pub use self::rle::{compress_rle, decompress_rle};

use num::FromPrimitive;

enum_from_primitive! {
    #[derive(Debug, Eq, PartialEq)]
    pub enum BiosCompressionType {
        Lz77       = 1,
        Huffman    = 2,
        Rle        = 3,
        DiffFilter = 8,
    }
}

pub fn bios_compression_type(value: u8) -> Option<BiosCompressionType> {
    BiosCompressionType::from_u8(value >> 4)
}
