use std::io::{Read, Cursor, Result, Error, ErrorKind};
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt, WriteBytesExt};
use bios::{BiosCompressionType, bios_compression_type};
use num::FromPrimitive;

enum_from_primitive! {
    #[derive(Debug, Eq, PartialEq)]
    enum StreamType {
        Diff8  = 1,
        Diff16 = 2,
    }
}

pub fn filter_diff8(input: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(input.len() + 4);

    output.write_u8(((BiosCompressionType::DiffFilter as u8) << 4) | (StreamType::Diff8 as u8))?;
    output.write_u24::<LittleEndian>(input.len() as u32)?;

    if input.len() > 0 {
        output.write_u8(input[0])?;
        for i in 1..input.len() {
            output.write_u8(input[i].wrapping_sub(input[i - 1]))?;
        }
    }

    Ok(output)
}

pub fn unfilter_diff8(input: &[u8]) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(input);
    let header = cursor.read_u8()?;

    if (bios_compression_type(header) != Some(BiosCompressionType::DiffFilter)) ||
        (StreamType::from_u8(header & 0xF) != Some(StreamType::Diff8)) {
        return Err(Error::new(ErrorKind::InvalidData, "filter header mismatch"));
    }

    let data_size: usize = cursor.read_u24::<LittleEndian>()? as usize;

    let mut output: Vec<u8> = vec![0; data_size];
    cursor.read_exact(&mut output)?;

    for i in 1..output.len() {
        let data = output[i - 1].wrapping_add(output[i]);
        output[i] = data;
    }

    assert_eq!(output.len(), data_size);
    Ok(output)
}

pub fn filter_diff16(input: &[u8]) -> Result<Vec<u8>> {
    if input.len() % 2 == 0 {
        let mut output = Vec::with_capacity(input.len() + 4);

        output.write_u8(((BiosCompressionType::DiffFilter as u8) << 4) | (StreamType::Diff16 as u8))?;
        output.write_u24::<LittleEndian>(input.len() as u32)?;

        let mut input16: Vec<u16> = vec![0; input.len() / 2];
        LittleEndian::read_u16_into(input, &mut input16[..]);

        if input16.len() > 0 {
            output.write_u16::<LittleEndian>(input16[0])?;
            for i in 1..input16.len() {
                output.write_u16::<LittleEndian>(input16[i].wrapping_sub(input16[i - 1]))?;
            }
        }

        Ok(output)
    } else {
        Err(Error::new(ErrorKind::InvalidData, "data size must be some multiple of 2"))
    }
}

pub fn unfilter_diff16(input: &[u8]) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(input);
    let header = cursor.read_u8()?;

    if (bios_compression_type(header) != Some(BiosCompressionType::DiffFilter)) ||
        (StreamType::from_u8(header & 0xF) != Some(StreamType::Diff16)) {
        return Err(Error::new(ErrorKind::InvalidData, "filter header mismatch"));
    }

    let data_size: usize = cursor.read_u24::<LittleEndian>()? as usize;
    if data_size % 2 != 0 {
        return Err(Error::new(ErrorKind::InvalidData, "data size must be some multiple of 2"));
    }

    let mut buffer: Vec<u16> = vec![0; data_size / 2];
    cursor.read_u16_into::<LittleEndian>(&mut buffer)?;

    for i in 1..buffer.len() {
        let data = buffer[i - 1].wrapping_add(buffer[i]);
        buffer[i] = data;
    }

    let mut output: Vec<u8> = vec![0; data_size];
    LittleEndian::write_u16_into(&buffer, &mut output[..]);

    assert_eq!(buffer.len(), data_size / 2);
    assert_eq!(output.len(), data_size);

    Ok(output)
}
