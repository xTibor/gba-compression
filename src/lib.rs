pub mod bios;
pub mod game_specific;

use std::io::Result;

pub trait Compressor {
    fn compress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()>;

    fn decompress(&self, input: &[u8], output: &mut Vec<u8>) -> Result<()>;
}

pub fn consecutive_count<T: ::std::cmp::Eq>(buffer: &[T], max_length: usize) -> usize {
    let mut count = 0;
    while (count < buffer.len()) && (count < max_length) && (buffer[0] == buffer[count]) {
        count += 1;
    }
    count
}

pub fn non_consecutive_count<T: ::std::cmp::Eq>(buffer: &[T], max_length: usize, consecutive_threshold: usize) -> usize {
    let mut count = 0;
    while (count < buffer.len()) && (count < max_length) && (consecutive_count(&buffer[count..], max_length) < consecutive_threshold) {
        count += 1;
    }
    count
}
