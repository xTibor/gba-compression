use std::io::{Write, Cursor, Result, Error, ErrorKind};
use std::cmp;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use compressor::Compressor;
use bios::{BiosCompressionType, bios_compression_type};

#[derive(Default)]
pub struct Lz77Compressor {
    pub vram_safe: bool,
}

impl Compressor for Lz77Compressor {
    fn decompress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
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
                        let offset = (((block & 0xF) << 8) | ((block >> 8) & 0xFF)) + 1;

                        if buffer.len() + length > decompressed_size {
                            return Err(Error::new(ErrorKind::InvalidData, "Length out of bounds"));
                        }

                        if offset > buffer.len() {
                            return Err(Error::new(ErrorKind::InvalidData, "Offset out of bounds"));
                        }

                        for _ in 0..length {
                            let index = buffer.len() - offset;
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

    fn compress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
        enum Block {
            Uncompressed {
                data: u8,
            },
            Backreference {
                offset: u16,
                length: u8,
            }
        }

        let mut blocks: Vec<Block> = Vec::new();
        let mut index = 0;

        'outer: while index < input.len() {
            let block_max_length = cmp::min(input.len() - index, 18);
            for length in (3..block_max_length + 1).rev() {
                // When decompressing to VRAM the previous byte cannot be referenced in
                // the uncompressed data because it may have not written to the memory yet.
                // The data to the VRAM is written in 16-bit words due to 16-bit data bus.
                let mut offset = if self.vram_safe { 2 } else { 1 };

                while (offset <= 4096) && (index >= offset) {
                    let index_back = index - offset;
                    if input[index..index + length] == input[index_back..index_back + length] {
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
                        if self.vram_safe {
                            assert!((offset >= 2) & (offset <= 4096), "offset must be between 2 and 4096");
                        } else {
                            assert!((offset >= 1) & (offset <= 4096), "offset must be between 1 and 4096");
                        }

                        let data = (((offset - 1) & 0xFF) << 8) | ((length - 3) << 4) as u16 | ((offset - 1) >> 8);
                        output.write_u16::<LittleEndian>(data)?;
                    },
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use compressor::Compressor;
    use bios::Lz77Compressor;

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
        let compressor = Lz77Compressor::default();

        compressor.decompress(&input, &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_decompress_2() {
        let input: Vec<u8> = vec![
            0x10, 0x00, 0x00, 0x00,
        ];

        let mut output: Vec<u8> = Vec::new();
        let compressor = Lz77Compressor::default();

        compressor.decompress(&input, &mut output).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_compress_1() {
        let input: Vec<u8> = Vec::new();
        let expected_output: Vec<u8> = vec![
            0x10, 0x00, 0x00, 0x00,
        ];

        let mut output: Vec<u8> = Vec::new();
        let compressor = Lz77Compressor::default();

        compressor.compress(&input, &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_compress_and_decompress_1() {
        let input: Vec<u8> = Vec::new();

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        let compressor = Lz77Compressor::default();

        compressor.compress(&input, &mut immediate).unwrap();
        compressor.decompress(&immediate, &mut output).unwrap();
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
        let compressor = Lz77Compressor::default();

        compressor.compress(&input, &mut immediate).unwrap();
        compressor.decompress(&immediate, &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_compress_and_decompress_3() {
        let input: Vec<u8> = vec![0x13; 4096];

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        let compressor = Lz77Compressor::default();

        compressor.compress(&input, &mut immediate).unwrap();
        compressor.decompress(&immediate, &mut output).unwrap();
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
        let compressor = Lz77Compressor::default();

        compressor.compress(&input, &mut immediate).unwrap();
        compressor.decompress(&immediate, &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_compress_vram_safe_1() {
        let input: Vec<u8> = vec![0xFF; 16];
        let expected_output: Vec<u8> = vec![
            0x10, 0x10, 0x00, 0x00,
            0x20, 0xFF, 0xFF, 0xB0, 0x01
        ];

        let mut output: Vec<u8> = Vec::new();
        let compressor = Lz77Compressor { vram_safe: true };

        compressor.compress(&input, &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_compress_vram_safe_2() {
        let input: Vec<u8> = vec![0xFF; 4096];

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        let compressor = Lz77Compressor::default();

        compressor.compress(&input, &mut immediate).unwrap();
        compressor.decompress(&immediate, &mut output).unwrap();
        assert_eq!(input, output);
    }
}
