use std::io::Result;

pub trait Compressor {
    fn compress(&self, input: &[u8]) -> Result<Vec<u8>>;

    fn decompress(&self, input: &[u8]) -> Result<Vec<u8>>;
}
