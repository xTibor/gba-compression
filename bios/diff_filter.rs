use std::io::{Read, Write, Result, Error, ErrorKind};
use byteorder::{ByteOrder, BigEndian, LittleEndian, ReadBytesExt};
use compression::bios::{BiosCompressionType, bios_compression_type};
use utils::{ReadBytesExtExt, WriteBytesExtExt};

#[allow(dead_code)]
pub fn decompress_diff_filter<R: Read, W: Write>(input: &mut R, output: &mut W) -> Result<()> {
    let header = input.read_u8()?;
    if bios_compression_type(header) != Some(BiosCompressionType::DiffFilter) {
        return Err(Error::new(ErrorKind::InvalidData, "Not a diff filtered data stream"));
    }

    let stream_type = header & 0xF;

    //let decompressed_size: usize = input.read_u24::<LittleEndian>()?;
    let decompressed_size: usize = (input.read_u8()? as usize) +
        ((input.read_u8()? as usize) << 8) +
        ((input.read_u8()? as usize) << 16);

    if stream_type == 1 {
        // 8-bit stream
        let mut buffer: Vec<u8> = vec![0; decompressed_size];
        input.read_exact(&mut buffer)?;

        for i in 1..buffer.len() {
            let data = buffer[i - 1].wrapping_add(buffer[i]);
            buffer[i] = data;
        }

        assert_eq!(buffer.len(), decompressed_size);
        output.write_all(&buffer)
    } else if stream_type == 2 {
        // 16-bit stream
        if decompressed_size % 2 != 0 {
            return Err(Error::new(ErrorKind::InvalidData, "Output size must be a multiple of 2 for 16-bit streams"));
        }

        let mut buffer: Vec<u16> = vec![0; decompressed_size / 2];
        input.read_exact_u16::<LittleEndian>(&mut buffer)?;

        for i in 1..buffer.len() {
            let data = buffer[i - 1].wrapping_add(buffer[i]);
            buffer[i] = data;
        }

        assert_eq!(buffer.len(), decompressed_size / 2);
        output.write_all_u16::<LittleEndian>(&buffer)
    } else {
        Err(Error::new(ErrorKind::InvalidData, "Unknown stream type"))
    }
}

#[allow(dead_code)]
pub fn compress_diff_filter<R: Read, W: Write>(_input: &mut R, _output: &mut W) -> Result<()> {
    Err(Error::new(ErrorKind::Other, "Unimplemented compression routine"))
}

#[cfg(test)]
mod tests {
    use compression::bios::{decompress_diff_filter, compress_diff_filter};
    use std::io::{Cursor, Seek, SeekFrom};

    #[test]
    fn test_decompress_1() {
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
        decompress_diff_filter(&mut Cursor::new(&input[..]), &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_decompress_2() {
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
        decompress_diff_filter(&mut Cursor::new(&input[..]), &mut output).unwrap();
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_compress_1() {
    }
}
