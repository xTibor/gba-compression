use std::io::{Write, Result, Error, ErrorKind, Cursor};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num::FromPrimitive;
use compressor::Compressor;
use utils::{consecutive_count, non_consecutive_count};

enum_from_primitive! {
    #[derive(Debug, Eq, PartialEq)]
    enum StreamType {
        Rle8  = 1,
        Rle16 = 2,
    }
}

#[derive(Default)]
pub struct Wl4Rle8Compressor;

#[derive(Default)]
pub struct Wl4Rle16Compressor;

#[derive(Default)]
pub struct Wl4RleCompressor {
    rle8_compressor: Wl4Rle8Compressor,
    rle16_compressor: Wl4Rle16Compressor,
}

impl Compressor for Wl4Rle8Compressor {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        output.write_u8(StreamType::Rle8 as u8)?;

        let mut offset = 0;
        while offset < input.len() {
            let length = consecutive_count(&input[offset..], 0x7F);
            if length == 1 {
                let length = non_consecutive_count(&input[offset..], 0x7F, 2);
                output.write_u8(length as u8)?;
                output.write_all(&input[offset..offset+length])?;
                offset += length;
            } else {
                output.write_u8(0x80 | length as u8)?;
                output.write_u8(input[offset])?;
                offset += length;
            }
        }

        output.write_u8(0)?;
        Ok(output)
    }

    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>> {
        let mut cursor = Cursor::new(input);
        let mut output = Vec::new();

        let stream_type = StreamType::from_u8(cursor.read_u8()?);
        if stream_type == Some(StreamType::Rle8) {
            loop {
                let block = cursor.read_u8()?;
                if block == 0 {
                    // End of stream
                    break;
                } else if block & 0x80 == 0 {
                    // Uncompressed
                    let length = block & 0x7F;
                    for _ in 0..length {
                        output.push(cursor.read_u8()?);
                    }
                } else {
                    // Run-length encoded
                    let length = block & 0x7F;
                    let data = cursor.read_u8()?;
                    for _ in 0..length {
                        output.push(data);
                    }
                }
            }

            Ok(output)
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Not a Wl4Rle8 stream"))
        }
    }
}

impl Compressor for Wl4Rle16Compressor {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        output.write_u8(StreamType::Rle16 as u8)?;

        let mut offset = 0;
        while offset < input.len() {
            let length = consecutive_count(&input[offset..], 0x7FFF);
            if length == 1 {
                let length = non_consecutive_count(&input[offset..], 0x7FFF, 2);
                output.write_u16::<BigEndian>(length as u16)?;
                output.write_all(&input[offset..offset+length])?;
                offset += length;
            } else {
                output.write_u16::<BigEndian>(0x8000 | length as u16)?;
                output.write_u8(input[offset])?;
                offset += length;
            }
        }

        output.write_u16::<BigEndian>(0)?;
        Ok(output)
    }

    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>> {
        let mut cursor = Cursor::new(input);
        let mut output = Vec::new();

        let stream_type = StreamType::from_u8(cursor.read_u8()?);
        if stream_type == Some(StreamType::Rle16) {
            loop {
                let block = cursor.read_u16::<BigEndian>()?;
                if block == 0 {
                    // End of stream
                    break;
                } else if block & 0x8000 == 0 {
                    // Uncompressed
                    let length = block & 0x7FFF;
                    for _ in 0..length {
                        output.push(cursor.read_u8()?);
                    }
                } else {
                    // Run-length encoded
                    let length = block & 0x7FFF;
                    let data = cursor.read_u8()?;
                    for _ in 0..length {
                        output.push(data);
                    }
                }
            }

            Ok(output)
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Not a Wl4Rle16 stream"))
        }
    }
}

impl Compressor for Wl4RleCompressor {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>> {
        let output_rle8 = self.rle8_compressor.compress(input)?;
        let output_rle16 = self.rle16_compressor.compress(input)?;

        if output_rle8.len() < output_rle16.len() {
            Ok(output_rle8)
        } else {
            Ok(output_rle16)
        }
    }

    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>> {
        let mut cursor = Cursor::new(input);
        let stream_type = StreamType::from_u8(cursor.read_u8()?);

        match stream_type {
            Some(StreamType::Rle8) => self.rle8_compressor.decompress(input),
            Some(StreamType::Rle16) => self.rle16_compressor.decompress(input),
            None => Err(Error::new(ErrorKind::InvalidData, "Unknown WL4 stream type")),
        }
    }
}
