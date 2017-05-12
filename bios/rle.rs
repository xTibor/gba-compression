use std::io::{Read, Write, Cursor, Result, Error, ErrorKind};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use compression::bios::{BiosCompressionType, bios_compression_type};
use compression::{consecutive_count, non_consecutive_count};

/// Decompression routine for GBA BIOS run-length encoded data
/// Compiles but untested
#[allow(dead_code)]
pub fn decompress_rle<R: Read, W: Write>(input: &mut R, output: &mut W) -> Result<()> {
    if bios_compression_type(input.read_u8()?) != Some(BiosCompressionType::Rle) {
        return Err(Error::new(ErrorKind::InvalidData, "Not an run-length encoded stream"));
    }

    let decompressed_size: usize = input.read_u24::<LittleEndian>()? as usize;
    let mut buffer: Vec<u8> = Vec::with_capacity(decompressed_size);

    while buffer.len() < decompressed_size {
        let block = input.read_u8()? as usize;
        if block & 0x80 == 0 {
            // Uncompressed
            let length = (block & 0x7F) + 1;
            if buffer.len() + length > decompressed_size {
                return Err(Error::new(ErrorKind::InvalidData, "Length out of bounds"));
            }

            for _ in 0..length {
                buffer.push(input.read_u8()?);
            }
        } else {
            // Run-length encoded
            let length = (block & 0x7F) + 3;
            if buffer.len() + length > decompressed_size {
                return Err(Error::new(ErrorKind::InvalidData, "Length out of bounds"));
            }

            let data = input.read_u8()?;
            for _ in 0..length {
                buffer.push(data);
            }
        }
    }

    assert_eq!(buffer.len(), decompressed_size);
    output.write_all(&buffer)
}

#[allow(dead_code)]
pub fn compress_rle<R: Read, W: Write>(input: &mut R, output: &mut W) -> Result<()> {
    let mut buffer: Vec<u8> = Vec::new();
    input.read_to_end(&mut buffer)?;

    output.write_u8((BiosCompressionType::Rle as u8) << 4)?;
    output.write_u24::<LittleEndian>(buffer.len() as u32)?;

    let mut offset = 0;
    while offset < buffer.len() {
        let length = consecutive_count(&buffer[offset..], 0x82);
        if length < 3 {
            let length = non_consecutive_count(&buffer[offset..], 0x80, 3);
            output.write_u8(length as u8 - 1)?;
            output.write_all(&buffer[offset..offset+length])?;
            offset += length;
        } else {
            output.write_u8(0x80 | (length as u8 - 3))?;
            output.write_u8(buffer[offset])?;
            offset += length;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use compression::bios::{decompress_rle, compress_rle};
    use std::io::{Cursor, Seek, SeekFrom};

    #[test]
    fn test_decompress_1() {
        let input: Vec<u8> = vec![
            0x30, 0x08, 0x00, 0x00,
            0x03, 0x01, 0x02, 0x03, 0x04,
            0x81, 0x05,
        ];
        let expected_output: Vec<u8> = vec![
            0x01, 0x02, 0x03, 0x04,
            0x05, 0x05, 0x05, 0x05,
        ];

        let mut output: Vec<u8> = Vec::new();
        decompress_rle(&mut Cursor::new(&input[..]), &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_decompress_2() {
        let input: Vec<u8> = vec![
            0x30, 0x04, 0x00, 0x00,
            0x04, 0x01, 0x02, 0x03, 0x04, 0x05,
        ];

        let mut output: Vec<u8> = Vec::new();
        assert!(decompress_rle(&mut Cursor::new(&input[..]), &mut output).is_err());
    }

    #[test]
    fn test_decompress_3() {
        let input: Vec<u8> = vec![
            0x30, 0x04, 0x00, 0x00,
            0x82, 0x01,
        ];

        let mut output: Vec<u8> = Vec::new();
        assert!(decompress_rle(&mut Cursor::new(&input[..]), &mut output).is_err());
    }

    #[test]
    fn test_decompress_4() {
        let input: Vec<u8> = vec![
            0x30, 0x00, 0x00, 0x00,
        ];

        let mut output: Vec<u8> = Vec::new();
        decompress_rle(&mut Cursor::new(&input[..]), &mut output).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_compress_and_decompress_1() {
        let input: Vec<u8> = Vec::new();

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        compress_rle(&mut Cursor::new(&input[..]), &mut immediate).unwrap();
        decompress_rle(&mut Cursor::new(&immediate[..]), &mut output).unwrap();
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
        compress_rle(&mut Cursor::new(&input[..]), &mut immediate).unwrap();
        decompress_rle(&mut Cursor::new(&immediate[..]), &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_compress_and_decompress_3() {
        let input: Vec<u8> = vec![0x42; 4096];

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        compress_rle(&mut Cursor::new(&input[..]), &mut immediate).unwrap();
        decompress_rle(&mut Cursor::new(&immediate[..]), &mut output).unwrap();
        assert_eq!(input, output);
    }
}
