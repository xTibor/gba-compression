mod diff_filter;
mod huffman;
mod lz77;
mod rle;

pub use self::diff_filter::{decompress_diff_filter, compress_diff_filter};
pub use self::huffman::{decompress_huffman, compress_huffman};
pub use self::lz77::{decompress_lz77, compress_lz77};
pub use self::rle::{decompress_rle, compress_rle};

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
