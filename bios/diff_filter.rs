use std::io::{Read, Write, Cursor, Result, Error, ErrorKind};
use byteorder::{ByteOrder, BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use compression::bios::{BiosCompressionType, bios_compression_type};
use utils::{ReadBytesExtExt, WriteBytesExtExt};
use num::FromPrimitive;

enum_from_primitive! {
    #[derive(Debug, Eq, PartialEq)]
    enum StreamType {
        Diff8  = 1,
        Diff16 = 2,
    }
}

#[allow(dead_code)]
pub fn unfilter_diff<R: Read, W: Write>(input: &mut R, output: &mut W) -> Result<()> {
    let header = input.read_u8()?;
    if bios_compression_type(header) != Some(BiosCompressionType::DiffFilter) {
        return Err(Error::new(ErrorKind::InvalidData, "Not a diff filtered data stream"));
    }

    let stream_type = StreamType::from_u8(header & 0xF);
    let unfiltered_size: usize = input.read_u24::<LittleEndian>()? as usize;

    if stream_type == Some(StreamType::Diff8) {
        let mut buffer: Vec<u8> = vec![0; unfiltered_size];
        input.read_exact(&mut buffer)?;

        for i in 1..buffer.len() {
            let data = buffer[i - 1].wrapping_add(buffer[i]);
            buffer[i] = data;
        }

        assert_eq!(buffer.len(), unfiltered_size);
        output.write_all(&buffer)
    } else if stream_type == Some(StreamType::Diff16) {
        if unfiltered_size % 2 != 0 {
            return Err(Error::new(ErrorKind::InvalidData, "Output size must be a multiple of 2 for 16-bit streams"));
        }

        let mut buffer: Vec<u16> = vec![0; unfiltered_size / 2];
        input.read_exact_u16::<LittleEndian>(&mut buffer)?;

        for i in 1..buffer.len() {
            let data = buffer[i - 1].wrapping_add(buffer[i]);
            buffer[i] = data;
        }

        assert_eq!(buffer.len(), unfiltered_size / 2);
        output.write_all_u16::<LittleEndian>(&buffer)
    } else {
        Err(Error::new(ErrorKind::InvalidData, "Unknown stream type"))
    }
}

#[allow(dead_code)]
pub fn filter_diff8<R: Read, W: Write>(input: &mut R, output: &mut W) -> Result<()> {
    let mut buffer: Vec<u8> = Vec::new();
    input.read_to_end(&mut buffer)?;

    for i in (1..buffer.len()).rev() {
        let data = buffer[i].wrapping_sub(buffer[i - 1]);
        buffer[i] = data;
    }

    output.write_u8(((BiosCompressionType::DiffFilter as u8) << 4) | (StreamType::Diff8 as u8))?;
    output.write_u24::<LittleEndian>(buffer.len() as u32)?;
    output.write_all(&buffer)
}

#[allow(dead_code)]
pub fn filter_diff16<R: Read, W: Write>(input: &mut R, output: &mut W) -> Result<()> {
    let mut buffer: Vec<u8> = Vec::new();
    input.read_to_end(&mut buffer)?;

    if buffer.len() % 2 != 0 {
        return Err(Error::new(ErrorKind::InvalidData, "Input size must be divisible by 2"));
    }

    let mut buffer16: Vec<u16> = vec![0; buffer.len() / 2];
    Cursor::new(buffer).read_exact_u16::<LittleEndian>(&mut buffer16)?;

    for i in (1..buffer16.len()).rev() {
        let data = buffer16[i].wrapping_sub(buffer16[i - 1]);
        buffer16[i] = data;
    }

    output.write_u8(((BiosCompressionType::DiffFilter as u8) << 4) | (StreamType::Diff16 as u8))?;
    output.write_u24::<LittleEndian>(buffer16.len() as u32 * 2)?;
    output.write_all_u16::<LittleEndian>(&buffer16)
}

#[cfg(test)]
mod tests {
    use compression::bios::{unfilter_diff, filter_diff8, filter_diff16};
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
        unfilter_diff(&mut Cursor::new(&input[..]), &mut output).unwrap();
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
        unfilter_diff(&mut Cursor::new(&input[..]), &mut output).unwrap();
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
        filter_diff8(&mut Cursor::new(&input[..]), &mut output).unwrap();
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
        filter_diff8(&mut Cursor::new(&input[..]), &mut output).unwrap();
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
        filter_diff8(&mut Cursor::new(&input[..]), &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_filter_diff8_4() {
        let input: Vec<u8> = Vec::new();
        let expected_output: Vec<u8> = vec![
            0x81, 0x00, 0x00, 0x00,
        ];

        let mut output: Vec<u8> = Vec::new();
        filter_diff8(&mut Cursor::new(&input[..]), &mut output).unwrap();
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
        filter_diff8(&mut Cursor::new(&input[..]), &mut immediate).unwrap();
        unfilter_diff(&mut Cursor::new(&immediate[..]), &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_filter_unfilter_diff8_2() {
        let input: Vec<u8> = Vec::new();

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        filter_diff8(&mut Cursor::new(&input[..]), &mut immediate).unwrap();
        unfilter_diff(&mut Cursor::new(&immediate[..]), &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_filter_unfilter_diff8_3() {
        let input: Vec<u8> = vec![0x42; 4096];

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        filter_diff8(&mut Cursor::new(&input[..]), &mut immediate).unwrap();
        unfilter_diff(&mut Cursor::new(&immediate[..]), &mut output).unwrap();
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
        filter_diff16(&mut Cursor::new(&input[..]), &mut immediate).unwrap();
        unfilter_diff(&mut Cursor::new(&immediate[..]), &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_filter_unfilter_diff16_2() {
        let input: Vec<u8> = Vec::new();

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        filter_diff16(&mut Cursor::new(&input[..]), &mut immediate).unwrap();
        unfilter_diff(&mut Cursor::new(&immediate[..]), &mut output).unwrap();
        assert_eq!(input, output);
    }

    #[test]
    fn test_filter_unfilter_diff16_3() {
        let input: Vec<u8> = vec![0x42; 4096];

        let mut immediate: Vec<u8> = Vec::new();
        let mut output: Vec<u8> = Vec::new();
        filter_diff16(&mut Cursor::new(&input[..]), &mut immediate).unwrap();
        unfilter_diff(&mut Cursor::new(&immediate[..]), &mut output).unwrap();
        assert_eq!(input, output);
    }
}
