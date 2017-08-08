use std::io::{Read, Write, Result, Error, ErrorKind};
use byteorder::{ByteOrder, BigEndian, LittleEndian, ReadBytesExt};
use compression::Compressor;
use compression::bios::{BiosCompressionType, bios_compression_type};
use utils::{ReadBytesExtExt, WriteBytesExtExt};

#[derive(Default)]
pub struct HuffmanCompressor;

impl Compressor for HuffmanCompressor {
    fn decompress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Unimplemented decompression routine"))
    }

    fn compress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
        Err(Error::new(ErrorKind::Other, "Unimplemented compression routine"))
    }
}


#[cfg(test)]
mod tests {
    use compression::bios::HuffmanCompressor;
    use std::io::{Cursor, Seek, SeekFrom};

    #[test]
    fn test_decompress_1() {
    }

    #[test]
    fn test_compress_1() {
    }
}
