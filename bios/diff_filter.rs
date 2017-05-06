use std::io::{Read, Write, Result, Error, ErrorKind};
use byteorder::{ByteOrder, BigEndian, LittleEndian, ReadBytesExt};
use compression::bios::{BiosCompressionType, bios_compression_type};
use utils::{ReadBytesExtExt, WriteBytesExtExt};

#[allow(dead_code)]
// Compiles but untested.
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
        let mut buffer: Vec<u8> = Vec::with_capacity(decompressed_size);
        input.read_exact(&mut buffer)?;

        // TODO: Is this signed? How does it handle wrapping?
        for i in 1..buffer.len() {
            let data = buffer[i - 1] + buffer[i];
            buffer[i] = data;
        }

        assert_eq!(buffer.len(), decompressed_size);
        output.write_all(&buffer)
    } else if stream_type == 2 {
        // 16-bit stream
        // TODO: Should this check if decomp_size is divisible by two?
        let mut buffer: Vec<u16> = Vec::with_capacity(decompressed_size / 2);
        input.read_exact_u16::<LittleEndian>(&mut buffer)?;

        // TODO: Is this signed? How does it handle wrapping?
        for i in 1..buffer.len() {
            let data = buffer[i - 1] + buffer[i];
            buffer[i] = data;
        }

        assert_eq!(buffer.len(), decompressed_size);
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

    #[test]
    fn test_decompress_diff_filter_1() {
    }

    #[test]
    fn test_compress_diff_filter_1() {
    }
}
