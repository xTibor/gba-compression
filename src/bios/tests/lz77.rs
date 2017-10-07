use compressor::Compressor;
use bios::Lz77Compressor;

// TODO: Add tests for out of bounds cases

#[test]
fn test_decompress_1() {
    let input: Vec<u8> = vec![
        0x10, 0x10, 0x00, 0x00,
        0x0C,
        0x01, 0x02, 0x03, 0x04,
        0x10, 0x03,
        0x50, 0x07,

    ];
    let expected_output: Vec<u8> = vec![
        0x01, 0x02, 0x03, 0x04,
        0x01, 0x02, 0x03, 0x04,
        0x01, 0x02, 0x03, 0x04,
        0x01, 0x02, 0x03, 0x04,
    ];

    let compressor = Lz77Compressor::default();
    let output = compressor.decompress(&input).unwrap();
    assert_eq!(output, expected_output);
}

#[test]
fn test_decompress_2() {
    let input: Vec<u8> = vec![
        0x10, 0x00, 0x00, 0x00,
    ];

    let compressor = Lz77Compressor::default();
    let output = compressor.decompress(&input).unwrap();
    assert!(output.is_empty());
}

#[test]
fn test_compress_1() {
    let input: Vec<u8> = Vec::new();
    let expected_output: Vec<u8> = vec![
        0x10, 0x00, 0x00, 0x00,
    ];

    let compressor = Lz77Compressor::default();
    let output = compressor.compress(&input).unwrap();
    assert_eq!(output, expected_output);
}

#[test]
fn test_compress_and_decompress_1() {
    let input: Vec<u8> = Vec::new();

    let compressor = Lz77Compressor::default();
    let immediate = compressor.compress(&input).unwrap();
    let output = compressor.decompress(&immediate).unwrap();
    assert_eq!(input, output);
}

#[test]
fn test_compress_and_decompress_2() {
    let input: Vec<u8> = vec![
        0x01, 0x02, 0x03, 0x04,
        0x05, 0x05, 0x05, 0x05,
        0x01, 0x02, 0x03, 0x04,
        0x05, 0x05, 0x05, 0x05,
    ];

    let compressor = Lz77Compressor::default();
    let immediate = compressor.compress(&input).unwrap();
    let output = compressor.decompress(&immediate).unwrap();
    assert_eq!(input, output);
}

#[test]
fn test_compress_and_decompress_3() {
    let input: Vec<u8> = vec![0x13; 4096];

    let compressor = Lz77Compressor::default();
    let immediate = compressor.compress(&input).unwrap();
    let output = compressor.decompress(&immediate).unwrap();
    assert_eq!(input, output);
}

#[test]
fn test_compress_and_decompress_4() {
    let input: Vec<u8> = vec![
        0x01, 0x02, 0x03, 0x04,
        0x05, 0x06, 0x07, 0x08,
        0x02, 0x03, 0x01, 0x02,
    ];

    let compressor = Lz77Compressor::default();
    let immediate = compressor.compress(&input).unwrap();
    let output = compressor.decompress(&immediate).unwrap();
    assert_eq!(input, output);
}

#[test]
fn test_compress_vram_safe_1() {
    let input: Vec<u8> = vec![0xFF; 16];
    let expected_output: Vec<u8> = vec![
        0x10, 0x10, 0x00, 0x00,
        0x20, 0xFF, 0xFF, 0xB0, 0x01
    ];

    let compressor = Lz77Compressor { vram_safe: true };
    let output = compressor.compress(&input).unwrap();
    assert_eq!(output, expected_output);
}

#[test]
fn test_compress_vram_safe_2() {
    let input: Vec<u8> = vec![0xFF; 4096];

    let compressor = Lz77Compressor { vram_safe: true };
    let immediate = compressor.compress(&input).unwrap();
    let output = compressor.decompress(&immediate).unwrap();
    assert_eq!(input, output);
}