use std::io::{Read, Write, Result, Error, ErrorKind};
use byteorder::ReadBytesExt;
use compression::bios::{BiosCompressionType, bios_compression_type};

/// Decompression routine for GBA BIOS run-length encoded data
/// Compiles but untested
#[allow(dead_code)]
pub fn decompress_rle<R: Read, W: Write>(input: &mut R, output: &mut W) -> Result<()> {
    if bios_compression_type(input.read_u8()?) != Some(BiosCompressionType::Rle) {
        return Err(Error::new(ErrorKind::InvalidData, "Not an run-length encoded stream"));
    }

    //let decompressed_size: usize = input.read_u24::<LittleEndian>()?;
    let decompressed_size: usize = (input.read_u8()? as usize) +
        ((input.read_u8()? as usize) << 8) +
        ((input.read_u8()? as usize) << 16);

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
pub fn compress_rle<R: Read, W: Write>(_input: &mut R, _output: &mut W) -> Result<()> {
    Err(Error::new(ErrorKind::Other, "Unimplemented compression routine"))
}

#[cfg(test)]
mod tests {
    use compression::bios::{decompress_rle, compress_rle};

    #[test]
    fn test_decompress_rle_1() {
    }

    #[test]
    fn test_compress_rle_1() {
    }
}
