use std::io::{Read, Write, Result, Error, ErrorKind};
use byteorder::{LittleEndian, ReadBytesExt};

/// Decompression routine for Wario Land 4 run-length encoded data
/// Operates on 8-bit data but run-lengths can be 8 or 16-bit.
/// Based on my old FPC/Lazarus code. Compiles but untested.
#[allow(dead_code)]
pub fn decompress_wl4_rle<R: Read, W: Write>(input: &mut R, output: &mut W) -> Result<()> {
    let mut buffer: Vec<u8> = Vec::with_capacity(4096);

    let stream_type = input.read_u8()?;
    if stream_type == 1 {
        // 8-bit RLE stream
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
    } else if stream_type == 2 {
        // 16-bit RLE stream
        loop {
            let block = input.read_u16::<LittleEndian>()?;
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

#[allow(dead_code)]
pub fn compress_wl4_rle<R: Read, W: Write>(_input: &mut R, _output: &mut W) -> Result<()> {
    Err(Error::new(ErrorKind::Other, "Unimplemented compression routine"))
}

#[cfg(test)]
mod tests {
    use compression::game_specific::wario_land_4::{decompress_wl4_rle, compress_wl4_rle};

    #[test]
    fn test_decompress_wl4_rle_1() {
    }

    #[test]
    fn test_compress_wl4_rle_1() {
    }
}
