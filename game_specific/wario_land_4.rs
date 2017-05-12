use std::io::{Read, Write, Result, Error, ErrorKind, Cursor};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num::FromPrimitive;
use compression::{consecutive_count, non_consecutive_count};

enum_from_primitive! {
    #[derive(Debug, Eq, PartialEq)]
    enum StreamType {
        Rle8  = 1,
        Rle16 = 2,
    }
}

/// Decompression routine for Wario Land 4 run-length encoded data
/// Operates on 8-bit data but run-lengths can be 8 or 16-bit.
/// Based on my old FPC/Lazarus code.
#[allow(dead_code)]
pub fn decompress_wl4_rle<R: Read, W: Write>(input: &mut R, output: &mut W) -> Result<()> {
    let mut buffer: Vec<u8> = Vec::with_capacity(4096);

    let stream_type = StreamType::from_u8(input.read_u8()?);
    if stream_type == Some(StreamType::Rle8) {
        loop {
            let block = input.read_u8()?;
            if block == 0 {
                // End of stream
                break;
            } else if block & 0x80 == 0 {
                // Uncompressed
                let length = block & 0x7F;
                for _ in 0..length {
                    buffer.push(input.read_u8()?);
                }
            } else {
                // Run-length encoded
                let length = block & 0x7F;
                let data = input.read_u8()?;
                for _ in 0..length {
                    buffer.push(data);
                }
            }
        }
    } else if stream_type == Some(StreamType::Rle16) {
        loop {
            let block = input.read_u16::<BigEndian>()?;
            if block == 0 {
                // End of stream
                break;
            } else if block & 0x8000 == 0 {
                // Uncompressed
                let length = block & 0x7FFF;
                for _ in 0..length {
                    buffer.push(input.read_u8()?);
                }
            } else {
                // Run-length encoded
                let length = block & 0x7FFF;
                let data = input.read_u8()?;
                for _ in 0..length {
                    buffer.push(data);
                }
            }
        }
    } else {
        return Err(Error::new(ErrorKind::InvalidData, "Unknown stream type"));
    }

    output.write_all(&buffer)
}

// TODO: Extract this stream comparison function to the parent module
#[allow(dead_code)]
pub fn compress_wl4_rle<R: Read, W: Write>(input: &mut R, output: &mut W) -> Result<()> {
    let mut buffer_input: Vec<u8> = Vec::new();
    let mut buffer_rle8: Vec<u8> = Vec::new();
    let mut buffer_rle16: Vec<u8> = Vec::new();

    input.read_to_end(&mut buffer_input)?;
    compress_wl4_rle8(&mut Cursor::new(&buffer_input[..]), &mut buffer_rle8)?;
    compress_wl4_rle16(&mut Cursor::new(&buffer_input[..]), &mut buffer_rle16)?;

    if buffer_rle8.len() < buffer_rle16.len() {
        output.write_all(&buffer_rle8)
    } else {
        output.write_all(&buffer_rle16)
    }
}

#[allow(dead_code)]
pub fn compress_wl4_rle8<R: Read, W: Write>(input: &mut R, output: &mut W) -> Result<()> {
    let mut buffer: Vec<u8> = Vec::new();
    input.read_to_end(&mut buffer)?;

    output.write_u8(StreamType::Rle8 as u8)?;
    let mut offset = 0;
    while offset < buffer.len() {
        let length = consecutive_count(&buffer[offset..], 0x7F);
        if length == 1 {
            let length = non_consecutive_count(&buffer[offset..], 0x7F, 2);
            output.write_u8(length as u8)?;
            output.write_all(&buffer[offset..offset+length])?;
            offset += length;
        } else {
            output.write_u8(0x80 | length as u8)?;
            output.write_u8(buffer[offset])?;
            offset += length;
        }
    }

    output.write_u8(0)
}

#[allow(dead_code)]
pub fn compress_wl4_rle16<R: Read, W: Write>(input: &mut R, output: &mut W) -> Result<()> {
    let mut buffer: Vec<u8> = Vec::new();
    input.read_to_end(&mut buffer)?;

    output.write_u8(StreamType::Rle16 as u8)?;
    let mut offset = 0;
    while offset < buffer.len() {
        let length = consecutive_count(&buffer[offset..], 0x7FFF);
        if length == 1 {
            let length = non_consecutive_count(&buffer[offset..], 0x7FFF, 2);
            output.write_u16::<BigEndian>(length as u16)?;
            output.write_all(&buffer[offset..offset+length])?;
            offset += length;
        } else {
            output.write_u16::<BigEndian>(0x8000 | length as u16)?;
            output.write_u8(buffer[offset])?;
            offset += length;
        }
    }

    output.write_u16::<BigEndian>(0)
}

#[cfg(test)]
mod tests {
    use compression::game_specific::wario_land_4::{decompress_wl4_rle, compress_wl4_rle};
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
        decompress_wl4_rle(&mut Cursor::new(&input[..]), &mut output).unwrap();
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
        decompress_wl4_rle(&mut Cursor::new(&input[..]), &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_decompress_3() {
        let input: Vec<u8> = vec![
            0x01, 0x00,
        ];

        let mut output: Vec<u8> = Vec::new();
        decompress_wl4_rle(&mut Cursor::new(&input[..]), &mut output).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_decompress_4() {
        let input: Vec<u8> = vec![
            0x02, 0x00, 0x00,
        ];

        let mut output: Vec<u8> = Vec::new();
        decompress_wl4_rle(&mut Cursor::new(&input[..]), &mut output).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_compress_and_decompress_1() {
        let input: Vec<u8> = Vec::new();

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        compress_wl4_rle(&mut Cursor::new(&input[..]), &mut immediate).unwrap();
        decompress_wl4_rle(&mut Cursor::new(&immediate[..]), &mut output).unwrap();
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
        compress_wl4_rle(&mut Cursor::new(&input[..]), &mut immediate).unwrap();
        decompress_wl4_rle(&mut Cursor::new(&immediate[..]), &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_compress_and_decompress_3() {
        let input: Vec<u8> = vec![0x42; 4096];

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        compress_wl4_rle(&mut Cursor::new(&input[..]), &mut immediate).unwrap();
        decompress_wl4_rle(&mut Cursor::new(&immediate[..]), &mut output).unwrap();
        assert_eq!(input, output);
    }
}
