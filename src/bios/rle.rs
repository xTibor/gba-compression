use std::io::{Write, Cursor, Result, Error, ErrorKind};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use bios::{BiosCompressionType, bios_compression_type};
use compressor::Compressor;
use utils::{consecutive_count, non_consecutive_count};

#[derive(Default)]
pub struct RleCompressor;

impl Compressor for RleCompressor {
    fn decompress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
        let mut cursor = Cursor::new(input);

        if bios_compression_type(cursor.read_u8()?) != Some(BiosCompressionType::Rle) {
            return Err(Error::new(ErrorKind::InvalidData, "Not an run-length encoded stream"));
        }

        let decompressed_size: usize = cursor.read_u24::<LittleEndian>()? as usize;
        let mut buffer: Vec<u8> = Vec::with_capacity(decompressed_size);

        while buffer.len() < decompressed_size {
            let block = cursor.read_u8()? as usize;
            if block & 0x80 == 0 {
                // Uncompressed
                let length = (block & 0x7F) + 1;
                if buffer.len() + length > decompressed_size {
                    return Err(Error::new(ErrorKind::InvalidData, "Length out of bounds"));
                }

                for _ in 0..length {
                    buffer.push(cursor.read_u8()?);
                }
            } else {
                // Run-length encoded
                let length = (block & 0x7F) + 3;
                if buffer.len() + length > decompressed_size {
                    return Err(Error::new(ErrorKind::InvalidData, "Length out of bounds"));
                }

                let data = cursor.read_u8()?;
                for _ in 0..length {
                    buffer.push(data);
                }
            }
        }

        assert_eq!(buffer.len(), decompressed_size);
        output.write_all(&buffer)
    }

    fn compress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
        output.write_u8((BiosCompressionType::Rle as u8) << 4)?;
        output.write_u24::<LittleEndian>(input.len() as u32)?;

        let mut offset = 0;
        while offset < input.len() {
            let length = consecutive_count(&input[offset..], 0x82);
            if length < 3 {
                let length = non_consecutive_count(&input[offset..], 0x80, 3);
                output.write_u8(length as u8 - 1)?;
                output.write_all(&input[offset..offset+length])?;
                offset += length;
            } else {
                output.write_u8(0x80 | (length as u8 - 3))?;
                output.write_u8(input[offset])?;
                offset += length;
            }
        }

        Ok(())
    }
}
