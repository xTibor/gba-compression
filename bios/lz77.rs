use std::io::{Read, Write, Result, Error, ErrorKind};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use compression::bios::{BiosCompressionType, bios_compression_type};

enum Block {
    Uncompressed {
        data: u8,
    },
    Backreference {
        offset: u16,
        length: u8,
    }
}

/// Decompression routine for GBA BIOS LZ77 encoded data
#[allow(dead_code)]
pub fn decompress_lz77<R: Read, W: Write>(input: &mut R, output: &mut W) -> Result<()> {
    if bios_compression_type(input.read_u8()?) != Some(BiosCompressionType::Lz77) {
        return Err(Error::new(ErrorKind::InvalidData, "Not an LZ77 encoded stream"));
    }

    let decompressed_size: usize = input.read_u24::<LittleEndian>()? as usize;
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
pub fn compress_lz77<R: Read, W: Write>(input: &mut R, output: &mut W) -> Result<()> {
    let mut buffer: Vec<u8> = Vec::new();
    input.read_to_end(&mut buffer)?;

    let mut blocks: Vec<Block> = Vec::new();
    let mut offset = 0;
    while offset < buffer.len() {
        // TODO: Find backreferences
        blocks.push(Block::Uncompressed { data: buffer[offset] });

        offset += 1;
    }

    output.write_u8((BiosCompressionType::Lz77 as u8) << 4)?;
    output.write_u24::<LittleEndian>(buffer.len() as u32)?;

    for chunk in blocks.chunks(8) {
        let mut block_types = 0;
        for (i, block) in chunk.iter().enumerate() {
            if let Block::Backreference { .. } = *block  {
                block_types |= 0x80 >> i;
            }
        }
        output.write_u8(block_types)?;

        for block in chunk {
            match *block {
                Block::Uncompressed { data } => {
                    output.write_u8(data)?;
                },
                Block::Backreference { offset, length } => {
                    assert!((length >= 3) & (length <= 18), "length must be between 3 and 18");
                    assert_eq!(offset >> 12, 0, "offset must be 12-bit");
                    let data = ((offset & 0xFF) << 8) | ((length - 3) << 4) as u16 | (offset >> 8);
                    output.write_u16::<LittleEndian>(data)?;
                },
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use compression::bios::{decompress_lz77, compress_lz77};
    use std::io::{Cursor, Seek, SeekFrom};

    // TODO: Add tests for out of bounds cases

    #[test]
    fn test_decompress_1() {
        let input: Vec<u8> = vec![
            0x10, 0x10, 0x00, 0x00,
            0x0C,
            0x01, 0x02, 0x03, 0x04,
            0x10, 0x03,
            0x50, 0x07,

        ];
        let expected_output: Vec<u8> = vec![
            0x01, 0x02, 0x03, 0x04,
            0x01, 0x02, 0x03, 0x04,
            0x01, 0x02, 0x03, 0x04,
            0x01, 0x02, 0x03, 0x04,
        ];

        let mut output: Vec<u8> = Vec::new();
        decompress_lz77(&mut Cursor::new(&input[..]), &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_decompress_2() {
        let input: Vec<u8> = vec![
            0x10, 0x00, 0x00, 0x00,
        ];

        let mut output: Vec<u8> = Vec::new();
        decompress_lz77(&mut Cursor::new(&input[..]), &mut output).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_compress_1() {
        let input: Vec<u8> = Vec::new();
        let expected_output: Vec<u8> = vec![
            0x10, 0x00, 0x00, 0x00,
        ];

        let mut output: Vec<u8> = Vec::new();
        compress_lz77(&mut Cursor::new(&input[..]), &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_compress_and_decompress_1() {
        let input: Vec<u8> = Vec::new();

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        compress_lz77(&mut Cursor::new(&input[..]), &mut immediate).unwrap();
        decompress_lz77(&mut Cursor::new(&immediate[..]), &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_compress_and_decompress_2() {
        let input: Vec<u8> = vec![
            0x01, 0x02, 0x03, 0x04,
            0x05, 0x05, 0x05, 0x05,
            0x01, 0x02, 0x03, 0x04,
            0x05, 0x05, 0x05, 0x05,
        ];

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        compress_lz77(&mut Cursor::new(&input[..]), &mut immediate).unwrap();
        decompress_lz77(&mut Cursor::new(&immediate[..]), &mut output).unwrap();
        assert_eq!(input, output);
    }
}
