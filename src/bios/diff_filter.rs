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
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::from(input);

        for i in (1..buffer.len()).rev() {
            let data = buffer[i].wrapping_sub(buffer[i - 1]);
            buffer[i] = data;
        }

        let mut output = Vec::new();

        output.write_u8(((BiosCompressionType::DiffFilter as u8) << 4) | (StreamType::Diff8 as u8))?;
        output.write_u24::<LittleEndian>(buffer.len() as u32)?;
        output.write_all(&buffer)?;

        Ok(output)
    }

    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>> {
        let mut cursor = Cursor::new(input);
        let header = cursor.read_u8()?;

        if (bios_compression_type(header) != Some(BiosCompressionType::DiffFilter)) ||
            (StreamType::from_u8(header & 0xF) != Some(StreamType::Diff8)) {
            return Err(Error::new(ErrorKind::InvalidData, "Not a Diff8 stream"));
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
}

impl Compressor for Diff16Filter {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>> {
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

            let mut output = Vec::new();

            output.write_u8(((BiosCompressionType::DiffFilter as u8) << 4) | (StreamType::Diff16 as u8))?;
            output.write_u24::<LittleEndian>(buffer8.len() as u32)?;
            output.write_all(&buffer8)?;

            Ok(output)
        } else {
            Err(Error::new(ErrorKind::InvalidData, "Input must be a multiple of 2"))
        }
    }

    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>> {
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
        let mut output: Vec<u8> = vec![0; data_size];
        LittleEndian::write_u16_into(&buffer, &mut output[..]);

        assert_eq!(buffer.len(), data_size / 2);
        assert_eq!(output.len(), data_size);

        Ok(output)
    }
}
