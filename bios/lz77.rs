use std::io::{Read, Write, Cursor, Result, Error, ErrorKind};
use std::cmp;
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
pub fn decompress_lz77(input: &[u8], output: &mut Vec<u8>) -> Result<()> {
    let mut cursor = Cursor::new(input);

    if bios_compression_type(cursor.read_u8()?) != Some(BiosCompressionType::Lz77) {
        return Err(Error::new(ErrorKind::InvalidData, "Not an LZ77 encoded stream"));
    }

    let decompressed_size: usize = cursor.read_u24::<LittleEndian>()? as usize;
    let mut buffer: Vec<u8> = Vec::with_capacity(decompressed_size);

    while buffer.len() < decompressed_size {
        let block_types = cursor.read_u8()?;

        for i in 0..8 {
            if buffer.len() < decompressed_size {
                if block_types & (0x80 >> i) == 0 {
                    // Uncompressed
                    buffer.push(cursor.read_u8()?);
                } else {
                    // Backreference
                    let block = cursor.read_u16::<LittleEndian>()? as usize;
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
pub fn compress_lz77(input: &[u8], output: &mut Vec<u8>) -> Result<()> {
    let mut blocks: Vec<Block> = Vec::new();
    let mut index = 0;

    'outer: while index < input.len() {
        let block_max_length = cmp::min(input.len() - index, 18);
        for length in (3..block_max_length + 1).rev() {
            // TODO: Is overlapping tolerated by the BIOS decompression routine?
            // The `offset` counter has to start at `length` if not.
            let mut offset = 1;
            while (offset <= 4096) && (index >= offset) {
                let index_back = index - offset;
                if &input[index..index + length] == &input[index_back..index_back + length] {
                    blocks.push(Block::Backreference {
                        offset: offset as u16,
                        length: length as u8,
                    });
                    index += length;
                    continue 'outer;
                }
                offset += 1;
            }
        }
        blocks.push(Block::Uncompressed { data: input[index] });
        index += 1;
    }

    output.write_u8((BiosCompressionType::Lz77 as u8) << 4)?;
    output.write_u24::<LittleEndian>(input.len() as u32)?;

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
                    assert!((offset >= 1) & (offset <= 4096), "offset must be between 1 and 4096");
                    let data = (((offset - 1) & 0xFF) << 8) | ((length - 3) << 4) as u16 | ((offset - 1) >> 8);
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
        decompress_lz77(&input, &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_decompress_2() {
        let input: Vec<u8> = vec![
            0x10, 0x00, 0x00, 0x00,
        ];

        let mut output: Vec<u8> = Vec::new();
        decompress_lz77(&input, &mut output).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_compress_1() {
        let input: Vec<u8> = Vec::new();
        let expected_output: Vec<u8> = vec![
            0x10, 0x00, 0x00, 0x00,
        ];

        let mut output: Vec<u8> = Vec::new();
        compress_lz77(&input, &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_compress_and_decompress_1() {
        let input: Vec<u8> = Vec::new();

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        compress_lz77(&input, &mut immediate).unwrap();
        decompress_lz77(&immediate, &mut output).unwrap();
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
        compress_lz77(&input, &mut immediate).unwrap();
        decompress_lz77(&immediate, &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_compress_and_decompress_3() {
        let input: Vec<u8> = vec![0x13; 4096];

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        compress_lz77(&input, &mut immediate).unwrap();
        decompress_lz77(&immediate, &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_compress_and_decompress_4() {
        let input: Vec<u8> = vec![
            0x01, 0x02, 0x03, 0x04,
            0x05, 0x06, 0x07, 0x08,
            0x02, 0x03, 0x01, 0x02,
        ];

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        compress_lz77(&input, &mut immediate).unwrap();
        decompress_lz77(&immediate, &mut output).unwrap();
        assert_eq!(input, output);
    }
}
