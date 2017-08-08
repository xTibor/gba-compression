use std::io::{Read, Write, Cursor, Result, Error, ErrorKind};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use compression::bios::{BiosCompressionType, bios_compression_type};
use compression::{Compressor, consecutive_count, non_consecutive_count};

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



#[cfg(test)]
mod tests {
    use compression::Compressor;
    use compression::bios::RleCompressor;
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
        let compressor = RleCompressor::default();

        compressor.decompress(&input, &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_decompress_2() {
        let input: Vec<u8> = vec![
            0x30, 0x04, 0x00, 0x00,
            0x04, 0x01, 0x02, 0x03, 0x04, 0x05,
        ];

        let mut output: Vec<u8> = Vec::new();
        let compressor = RleCompressor::default();

        assert!(compressor.decompress(&input, &mut output).is_err());
    }

    #[test]
    fn test_decompress_3() {
        let input: Vec<u8> = vec![
            0x30, 0x04, 0x00, 0x00,
            0x82, 0x01,
        ];

        let mut output: Vec<u8> = Vec::new();
        let compressor = RleCompressor::default();

        assert!(compressor.decompress(&input, &mut output).is_err());
    }

    #[test]
    fn test_decompress_4() {
        let input: Vec<u8> = vec![
            0x30, 0x00, 0x00, 0x00,
        ];

        let mut output: Vec<u8> = Vec::new();
        let compressor = RleCompressor::default();

        compressor.decompress(&input, &mut output).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_compress_and_decompress_1() {
        let input: Vec<u8> = Vec::new();

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        let compressor = RleCompressor::default();

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
        let compressor = RleCompressor::default();

        compressor.compress(&input, &mut immediate).unwrap();
        compressor.decompress(&immediate, &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_compress_and_decompress_3() {
        let input: Vec<u8> = vec![0x42; 4096];

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        let compressor = RleCompressor::default();

        compressor.compress(&input, &mut immediate).unwrap();
        compressor.decompress(&immediate, &mut output).unwrap();
        assert_eq!(input, output);
    }
}
