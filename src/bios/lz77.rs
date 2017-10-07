use std::io::{Write, Cursor, Result, Error, ErrorKind};
use std::cmp;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use compressor::Compressor;
use bios::{BiosCompressionType, bios_compression_type};
use utils::same_count;

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

        while index < input.len() {
            // When decompressing to VRAM the previous byte cannot be referenced in
            // the uncompressed data because it may have not written to the memory yet.
            // The data to the VRAM is written in 16-bit words due to 16-bit data bus.
            let min_offset = if self.vram_safe { 2 } else { 1 };
            let max_offset = cmp::min(index, 4096);

            let min_length = 3;
            let max_length = cmp::min(input.len() - index, 18);

            let mut best_reference: Option<(usize, usize)> = None;

            for current_offset in min_offset..=max_offset {
                let current_length = same_count(&input[index..], &input[index - current_offset..], max_length);

                if current_length >= min_length {
                    if let Some((_, best_length)) = best_reference {
                        if current_length > best_length {
                            best_reference = Some((current_offset, current_length));
                        }
                    } else {
                        best_reference = Some((current_offset, current_length));
                    }
                }
            }

            if let Some((best_offset, best_length)) = best_reference {
                blocks.push(Block::Backreference {
                    offset: best_offset as u16,
                    length: best_length as u8,
                });
                index += best_length;
            } else {
                blocks.push(Block::Uncompressed { data: input[index] });
                index += 1;
            }
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
