use std::io::{Read, Write, Result, Error, ErrorKind};
use byteorder::{LittleEndian, ReadBytesExt};
use compression::bios::{BiosCompressionType, bios_compression_type};

/// Decompression routine for GBA BIOS LZ77 encoded data
#[allow(dead_code)]
pub fn decompress_lz77<R: Read, W: Write>(input: &mut R, output: &mut W) -> Result<()> {
    if bios_compression_type(input.read_u8()?) != Some(BiosCompressionType::Lz77) {
        return Err(Error::new(ErrorKind::InvalidData, "Not an LZ77 encoded stream"));
    }

    //let decompressed_size: usize = input.read_u24::<LittleEndian>()?;
    let decompressed_size: usize = (input.read_u8()? as usize) +
        ((input.read_u8()? as usize) << 8) +
        ((input.read_u8()? as usize) << 16);

    let mut buffer: Vec<u8> = Vec::with_capacity(decompressed_size);

    while buffer.len() < decompressed_size {
        let block_types = input.read_u8()?;

        for i in 0..8 {
            if buffer.len() < decompressed_size {
                if block_types & (0x80 >> i) == 0 {
                    // Uncompressed
                    buffer.push(input.read_u8()?);
                } else {
                    // Backreference
                    let block = input.read_u16::<LittleEndian>()? as usize;
                    let length = ((block >> 4) & 0xF) + 3;
                    let offset = ((block & 0xF) << 8) | ((block >> 8) & 0xFF);

                    if buffer.len() + length > decompressed_size {
                        return Err(Error::new(ErrorKind::InvalidData, "Length out of bounds"));
                    }

                    // TODO: I'm not sure about the following condition, write tests for it.
                    if offset + 1 > buffer.len() {
                        return Err(Error::new(ErrorKind::InvalidData, "Offset out of bounds"));
                    }

                    for _ in 0..length {
                        let index = buffer.len() - offset - 1;
                        let byte = buffer[index];
                        buffer.push(byte);
                    }
                }
            }
        }
    }

    assert_eq!(buffer.len(), decompressed_size);
    output.write_all(&buffer)
}

#[allow(dead_code)]
pub fn compress_lz77<R: Read, W: Write>(_input: &mut R, _output: &mut W) -> Result<()> {
    Err(Error::new(ErrorKind::Other, "Unimplemented compression routine"))
}

#[cfg(test)]
mod tests {
    use compression::bios::{decompress_lz77, compress_lz77};

    #[test]
    fn test_decompress_1() {
    }

    #[test]
    fn test_compress_1() {
    }
}
