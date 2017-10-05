use std::io::{Read, Write, Cursor, Result, Error, ErrorKind};
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt, WriteBytesExt};
use bios::{BiosCompressionType, bios_compression_type};
use compressor::Compressor;
use num::FromPrimitive;

#[derive(Default)]
pub struct Diff8Filter;

#[derive(Default)]
pub struct Diff16Filter;

enum_from_primitive! {
    #[derive(Debug, Eq, PartialEq)]
    enum StreamType {
        Diff8  = 1,
        Diff16 = 2,
    }
}

impl Compressor for Diff8Filter {
    fn compress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
        let mut buffer: Vec<u8> = Vec::from(input);

        for i in (1..buffer.len()).rev() {
            let data = buffer[i].wrapping_sub(buffer[i - 1]);
            buffer[i] = data;
        }

        output.write_u8(((BiosCompressionType::DiffFilter as u8) << 4) | (StreamType::Diff8 as u8))?;
        output.write_u24::<LittleEndian>(buffer.len() as u32)?;
        output.write_all(&buffer)
    }

    fn decompress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
        let mut cursor = Cursor::new(input);
        let header = cursor.read_u8()?;

        if (bios_compression_type(header) != Some(BiosCompressionType::DiffFilter)) ||
            (StreamType::from_u8(header & 0xF) != Some(StreamType::Diff8)) {
            return Err(Error::new(ErrorKind::InvalidData, "Not a Diff8 stream"));
        }

        let data_size: usize = cursor.read_u24::<LittleEndian>()? as usize;

        let mut buffer: Vec<u8> = vec![0; data_size];
        cursor.read_exact(&mut buffer)?;

        for i in 1..buffer.len() {
            let data = buffer[i - 1].wrapping_add(buffer[i]);
            buffer[i] = data;
        }

        assert_eq!(buffer.len(), data_size);
        output.write_all(&buffer)
    }
}

impl Compressor for Diff16Filter {
    fn compress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
        if input.len() % 2 == 0 {
            let mut buffer: Vec<u16> = vec![0; input.len() / 2];
            LittleEndian::read_u16_into(input, &mut buffer[..]);

            for i in (1..buffer.len()).rev() {
                let data = buffer[i].wrapping_sub(buffer[i - 1]);
                buffer[i] = data;
            }

            // {byteorder_rant}
            // I have to convert the `Vec<u16>` buffer back to `Vec<u8>`
            // to be able to append it later to `output`.
            // It's a shame that the `byteorder` has no `write_all` equivalent
            // for u16 ints in `WriteBytesExt` to do this without a temp buffer.
            // It has only in-place variants in `ByteOrder` but I want something that
            // operates on a writer.
            let mut buffer8: Vec<u8> = vec![0; input.len()];
            LittleEndian::write_u16_into(&buffer, &mut buffer8[..]);

            output.write_u8(((BiosCompressionType::DiffFilter as u8) << 4) | (StreamType::Diff16 as u8))?;
            output.write_u24::<LittleEndian>(buffer8.len() as u32)?;
            output.write_all(&buffer8)
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Input must be a multiple of 2"))
        }
    }

    fn decompress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()> {
        let mut cursor = Cursor::new(input);
        let header = cursor.read_u8()?;

        if (bios_compression_type(header) != Some(BiosCompressionType::DiffFilter)) ||
            (StreamType::from_u8(header & 0xF) != Some(StreamType::Diff16)) {
            return Err(Error::new(ErrorKind::InvalidData, "Not a Diff16 stream"));
        }

        let data_size: usize = cursor.read_u24::<LittleEndian>()? as usize;
        if data_size % 2 != 0 {
            return Err(Error::new(ErrorKind::InvalidData, "Size must be a multiple of 2"));
        }

        let mut buffer: Vec<u16> = vec![0; data_size / 2];
        cursor.read_u16_into::<LittleEndian>(&mut buffer)?;

        for i in 1..buffer.len() {
            let data = buffer[i - 1].wrapping_add(buffer[i]);
            buffer[i] = data;
        }

        // See: {byteorder_rant}
        let mut buffer8: Vec<u8> = vec![0; data_size];
        LittleEndian::write_u16_into(&buffer, &mut buffer8[..]);

        assert_eq!(buffer.len(), data_size / 2);
        assert_eq!(buffer8.len(), data_size);
        output.write_all(&buffer8)
    }
}

#[cfg(test)]
mod tests {
    use compressor::Compressor;
    use bios::{Diff8Filter, Diff16Filter};
    use std::io::{Cursor, Seek, SeekFrom};

    #[test]
    fn test_unfilter_1() {
        let input: Vec<u8> = vec![
            0x81, 0x10, 0x00, 0x00,
            0x10, 0x10, 0x10, 0x10,
            0x10, 0x10, 0x10, 0x10,
            0x10, 0x10, 0x10, 0x10,
            0x10, 0x10, 0x10, 0x10,

        ];
        let expected_output: Vec<u8> = vec![
            0x10, 0x20, 0x30, 0x40,
            0x50, 0x60, 0x70, 0x80,
            0x90, 0xA0, 0xB0, 0xC0,
            0xD0, 0xE0, 0xF0, 0x00,
        ];

        let mut output: Vec<u8> = Vec::new();
        let compressor = Diff8Filter::default();

        compressor.decompress(&input, &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_unfilter_2() {
        let input: Vec<u8> = vec![
            0x82, 0x10, 0x00, 0x00,
            0x10, 0x10, 0x01, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x01, 0x00,
            0x01, 0x00, 0x01, 0x00,

        ];
        let expected_output: Vec<u8> = vec![
            0x10, 0x10, 0x11, 0x10,
            0x12, 0x10, 0x13, 0x10,
            0x14, 0x10, 0x15, 0x10,
            0x16, 0x10, 0x17, 0x10,
        ];

        let mut output: Vec<u8> = Vec::new();
        let compressor = Diff16Filter::default();

        compressor.decompress(&input, &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_filter_diff8_1() {
        let input: Vec<u8> = vec![
            0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, 0x20,

        ];
        let expected_output: Vec<u8> = vec![
            0x81, 0x08, 0x00, 0x00,
            0x20, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let mut output: Vec<u8> = Vec::new();
        let compressor = Diff8Filter::default();

        compressor.compress(&input, &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_filter_diff8_2() {
        let input: Vec<u8> = vec![
            0x20, 0x21, 0x22, 0x23,
            0x24, 0x25, 0x26, 0x27,

        ];
        let expected_output: Vec<u8> = vec![
            0x81, 0x08, 0x00, 0x00,
            0x20, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x01,
        ];

        let mut output: Vec<u8> = Vec::new();
        let compressor = Diff8Filter::default();

        compressor.compress(&input, &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_filter_diff8_3() {
        let input: Vec<u8> = vec![
            0x20, 0x1F, 0x1E, 0x1D,
            0x1C, 0x1B, 0x1A, 0x19,

        ];
        let expected_output: Vec<u8> = vec![
            0x81, 0x08, 0x00, 0x00,
            0x20, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF,
        ];

        let mut output: Vec<u8> = Vec::new();
        let compressor = Diff8Filter::default();

        compressor.compress(&input, &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_filter_diff8_4() {
        let input: Vec<u8> = Vec::new();
        let expected_output: Vec<u8> = vec![
            0x81, 0x00, 0x00, 0x00,
        ];

        let mut output: Vec<u8> = Vec::new();
        let compressor = Diff8Filter::default();

        compressor.compress(&input, &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_filter_unfilter_diff8_1() {
        let input: Vec<u8> = vec![
            0xAD, 0x98, 0xFB, 0x7F, 0xCE, 0x66, 0x76, 0x8F,
            0xDC, 0x0C, 0x9A, 0x8C, 0x70, 0x66, 0x95, 0x84,
            0xC7, 0x23, 0x89, 0xED, 0x92, 0xD5, 0x06, 0x8C,
            0x8C, 0xA1, 0xD4, 0x48, 0xEA, 0xC9, 0x9E, 0x90,
        ];

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        let compressor = Diff8Filter::default();

        compressor.compress(&input, &mut immediate).unwrap();
        compressor.decompress(&immediate, &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_filter_unfilter_diff8_2() {
        let input: Vec<u8> = Vec::new();

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        let compressor = Diff8Filter::default();

        compressor.compress(&input, &mut immediate).unwrap();
        compressor.decompress(&immediate, &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_filter_unfilter_diff8_3() {
        let input: Vec<u8> = vec![0x42; 4096];

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        let compressor = Diff8Filter::default();

        compressor.compress(&input, &mut immediate).unwrap();
        compressor.decompress(&immediate, &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_filter_unfilter_diff16_1() {
        let input: Vec<u8> = vec![
            0xAD, 0x98, 0xFB, 0x7F, 0xCE, 0x66, 0x76, 0x8F,
            0xDC, 0x0C, 0x9A, 0x8C, 0x70, 0x66, 0x95, 0x84,
            0xC7, 0x23, 0x89, 0xED, 0x92, 0xD5, 0x06, 0x8C,
            0x8C, 0xA1, 0xD4, 0x48, 0xEA, 0xC9, 0x9E, 0x90,
        ];

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        let compressor = Diff16Filter::default();

        compressor.compress(&input, &mut immediate).unwrap();
        compressor.decompress(&immediate, &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_filter_unfilter_diff16_2() {
        let input: Vec<u8> = Vec::new();

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        let compressor = Diff16Filter::default();

        compressor.compress(&input, &mut immediate).unwrap();
        compressor.decompress(&immediate, &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_filter_unfilter_diff16_3() {
        let input: Vec<u8> = vec![0x42; 4096];

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        let compressor = Diff16Filter::default();

        compressor.compress(&input, &mut immediate).unwrap();
        compressor.decompress(&immediate, &mut output).unwrap();
        assert_eq!(input, output);
    }
}
