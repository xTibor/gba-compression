use std::io::Result;

pub trait Compressor {
    fn compress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()>;

    fn decompress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()>;
}
