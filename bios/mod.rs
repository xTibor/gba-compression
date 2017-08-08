mod diff_filter;
mod huffman;
mod lz77;
mod rle;

pub use self::diff_filter::{unfilter_diff, filter_diff8, filter_diff16};
pub use self::huffman::{decompress_huffman, compress_huffman};
pub use self::lz77::Lz77Compressor;
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
