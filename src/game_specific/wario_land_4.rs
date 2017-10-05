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
    fn compress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
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

        output.write_u8(0)
    }

    fn decompress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
        let mut cursor = Cursor::new(input);

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

            Ok(())
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Not a Wl4Rle8 stream"))
        }
    }
}

impl Compressor for Wl4Rle16Compressor {
    fn compress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
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

        output.write_u16::<BigEndian>(0)
    }

    fn decompress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
        let mut cursor = Cursor::new(input);

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

            Ok(())
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Not a Wl4Rle16 stream"))
        }
    }
}

impl Compressor for Wl4RleCompressor {
    fn compress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
        let mut output_rle8: Vec<u8> = Vec::new();
        let mut output_rle16: Vec<u8> = Vec::new();

        self.rle8_compressor.compress(input, &mut output_rle8)?;
        self.rle16_compressor.compress(input, &mut output_rle16)?;

        if output_rle8.len() < output_rle16.len() {
            output.write_all(&output_rle8)
        } else {
            output.write_all(&output_rle16)
        }
    }

    fn decompress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
        let mut cursor = Cursor::new(input);
        let stream_type = StreamType::from_u8(cursor.read_u8()?);

        match stream_type {
            Some(StreamType::Rle8) => self.rle8_compressor.decompress(input, output),
            Some(StreamType::Rle16) => self.rle16_compressor.decompress(input, output),
            None => Err(Error::new(ErrorKind::InvalidData, "Unknown WL4 stream type")),
        }
    }
}

#[cfg(test)]
mod tests {
    use compressor::Compressor;
    use game_specific::wario_land_4::{Wl4RleCompressor, Wl4Rle8Compressor, Wl4Rle16Compressor};
    use std::io::{Cursor, Seek, SeekFrom};

    #[test]
    fn test_decompress_1() {
        let input: Vec<u8> = vec![
            0x01,
            0x04, 0x01, 0x02, 0x03, 0x04,
            0x84, 0x05,
            0x00,
        ];
        let expected_output: Vec<u8> = vec![
            0x01, 0x02, 0x03, 0x04,
            0x05, 0x05, 0x05, 0x05,
        ];

        let mut output: Vec<u8> = Vec::new();
        let compressor = Wl4Rle8Compressor::default();

        compressor.decompress(&input, &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_decompress_2() {
        let input: Vec<u8> = vec![
            0x02,
            0x00, 0x04, 0x01, 0x02, 0x03, 0x04,
            0x80, 0x04, 0x05,
            0x00, 0x00,
        ];
        let expected_output: Vec<u8> = vec![
            0x01, 0x02, 0x03, 0x04,
            0x05, 0x05, 0x05, 0x05,
        ];

        let mut output: Vec<u8> = Vec::new();
        let compressor = Wl4Rle16Compressor::default();

        compressor.decompress(&input, &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_decompress_3() {
        let input: Vec<u8> = vec![
            0x01, 0x00,
        ];

        let mut output: Vec<u8> = Vec::new();
        let compressor = Wl4Rle8Compressor::default();

        compressor.decompress(&input, &mut output).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_decompress_4() {
        let input: Vec<u8> = vec![
            0x02, 0x00, 0x00,
        ];

        let mut output: Vec<u8> = Vec::new();
        let compressor = Wl4Rle16Compressor::default();

        compressor.decompress(&input, &mut output).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_compress_and_decompress_1() {
        let input: Vec<u8> = Vec::new();

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        let compressor = Wl4RleCompressor::default();

        compressor.compress(&input, &mut immediate).unwrap();
        compressor.decompress(&immediate, &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_compress_and_decompress_2() {
        let input: Vec<u8> = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
            0x03, 0x04, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05,
            0x06, 0x07, 0x08, 0x09, 0x09, 0x09, 0x09, 0x09,
        ];

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        let compressor = Wl4RleCompressor::default();

        compressor.compress(&input, &mut immediate).unwrap();
        compressor.decompress(&immediate, &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_compress_and_decompress_3() {
        let input: Vec<u8> = vec![0x42; 4096];

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        let compressor = Wl4RleCompressor::default();

        compressor.compress(&input, &mut immediate).unwrap();
        compressor.decompress(&immediate, &mut output).unwrap();
        assert_eq!(input, output);
    }
}