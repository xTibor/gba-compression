use std::io::{Read, Write, Result, Error, ErrorKind};
use byteorder::{ByteOrder, BigEndian, LittleEndian, ReadBytesExt};
use compression::bios::{BiosCompressionType, bios_compression_type};
use utils::{ReadBytesExtExt, WriteBytesExtExt};

#[allow(dead_code)]
pub fn decompress_huffman(input: &[u8], output: &mut Vec<u8>) -> Result<()> {
    Err(Error::new(ErrorKind::Other, "Unimplemented decompression routine"))
}

#[allow(dead_code)]
pub fn compress_huffman(input: &[u8], output: &mut Vec<u8>) -> Result<()> {
    Err(Error::new(ErrorKind::Other, "Unimplemented compression routine"))
}

#[cfg(test)]
mod tests {
    use compression::bios::{decompress_huffman, compress_huffman};
    use std::io::{Cursor, Seek, SeekFrom};

    #[test]
    fn test_decompress_1() {
    }

    #[test]
    fn test_compress_1() {
    }
}
