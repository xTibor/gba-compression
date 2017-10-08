use std::io::{Write, Result, Error, ErrorKind, Cursor};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num::FromPrimitive;
use utils::{consecutive_count, non_consecutive_count};

enum_from_primitive! {
    #[derive(Debug, Eq, PartialEq)]
    enum RleType {
        Rle8  = 1,
        Rle16 = 2,
    }
}

pub fn compress_wl4_rle8(input: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    output.write_u8(RleType::Rle8 as u8)?;

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

    output.write_u8(0)?;
    Ok(output)
}

pub fn decompress_wl4_rle8(input: &[u8]) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(input);
    let mut output = Vec::new();

    let rle_type = RleType::from_u8(cursor.read_u8()?);
    if rle_type == Some(RleType::Rle8) {
        loop {
            let block = cursor.read_u8()?;
            if block == 0 {
                // End of data
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

        Ok(output)
    } else {
        Err(Error::new(ErrorKind::InvalidData, "compression header mismatch"))
    }
}

pub fn compress_wl4_rle16(input: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    output.write_u8(RleType::Rle16 as u8)?;

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

    output.write_u16::<BigEndian>(0)?;
    Ok(output)
}

pub fn decompress_wl4_rle16(input: &[u8]) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(input);
    let mut output = Vec::new();

    let rle_type = RleType::from_u8(cursor.read_u8()?);
    if rle_type == Some(RleType::Rle16) {
        loop {
            let block = cursor.read_u16::<BigEndian>()?;
            if block == 0 {
                // End of data
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

        Ok(output)
    } else {
        Err(Error::new(ErrorKind::InvalidData, "compression header mismatch"))
    }
}

pub fn compress_wl4_rle(input: &[u8]) -> Result<Vec<u8>> {
    let output_rle8 = compress_wl4_rle8(input)?;
    let output_rle16 = compress_wl4_rle16(input)?;

    if output_rle8.len() < output_rle16.len() {
        Ok(output_rle8)
    } else {
        Ok(output_rle16)
    }
}

pub fn decompress_wl4_rle(input: &[u8]) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(input);
    let rle_type = RleType::from_u8(cursor.read_u8()?);

    match rle_type {
        Some(RleType::Rle8) => decompress_wl4_rle8(input),
        Some(RleType::Rle16) => decompress_wl4_rle16(input),
        None => Err(Error::new(ErrorKind::InvalidData, "unknown compression header")),
    }
}
