use compressor::Compressor;
use bios::{Diff8Filter, Diff16Filter};

#[test]
fn test_unfilter_1() {
    let input: Vec<u8> = vec![
        0x81, 0x10, 0x00, 0x00,
        0x10, 0x10, 0x10, 0x10,
        0x10, 0x10, 0x10, 0x10,
        0x10, 0x10, 0x10, 0x10,
        0x10, 0x10, 0x10, 0x10,

    ];
    let expected_output: Vec<u8> = vec![
        0x10, 0x20, 0x30, 0x40,
        0x50, 0x60, 0x70, 0x80,
        0x90, 0xA0, 0xB0, 0xC0,
        0xD0, 0xE0, 0xF0, 0x00,
    ];

    let compressor = Diff8Filter::default();
    let output = compressor.decompress(&input).unwrap();
    assert_eq!(output, expected_output);
}

#[test]
fn test_unfilter_2() {
    let input: Vec<u8> = vec![
        0x82, 0x10, 0x00, 0x00,
        0x10, 0x10, 0x01, 0x00,
        0x01, 0x00, 0x01, 0x00,
        0x01, 0x00, 0x01, 0x00,
        0x01, 0x00, 0x01, 0x00,

    ];
    let expected_output: Vec<u8> = vec![
        0x10, 0x10, 0x11, 0x10,
        0x12, 0x10, 0x13, 0x10,
        0x14, 0x10, 0x15, 0x10,
        0x16, 0x10, 0x17, 0x10,
    ];

    let compressor = Diff16Filter::default();
    let output = compressor.decompress(&input).unwrap();
    assert_eq!(output, expected_output);
}

#[test]
fn test_filter_diff8_1() {
    let input: Vec<u8> = vec![
        0x20, 0x20, 0x20, 0x20,
        0x20, 0x20, 0x20, 0x20,

    ];
    let expected_output: Vec<u8> = vec![
        0x81, 0x08, 0x00, 0x00,
        0x20, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];

    let compressor = Diff8Filter::default();
    let output = compressor.compress(&input).unwrap();
    assert_eq!(output, expected_output);
}

#[test]
fn test_filter_diff8_2() {
    let input: Vec<u8> = vec![
        0x20, 0x21, 0x22, 0x23,
        0x24, 0x25, 0x26, 0x27,

    ];
    let expected_output: Vec<u8> = vec![
        0x81, 0x08, 0x00, 0x00,
        0x20, 0x01, 0x01, 0x01,
        0x01, 0x01, 0x01, 0x01,
    ];

    let compressor = Diff8Filter::default();
    let output = compressor.compress(&input).unwrap();
    assert_eq!(output, expected_output);
}

#[test]
fn test_filter_diff8_3() {
    let input: Vec<u8> = vec![
        0x20, 0x1F, 0x1E, 0x1D,
        0x1C, 0x1B, 0x1A, 0x19,

    ];
    let expected_output: Vec<u8> = vec![
        0x81, 0x08, 0x00, 0x00,
        0x20, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF,
    ];

    let compressor = Diff8Filter::default();
    let output = compressor.compress(&input).unwrap();
    assert_eq!(output, expected_output);
}

#[test]
fn test_filter_diff8_4() {
    let input: Vec<u8> = Vec::new();
    let expected_output: Vec<u8> = vec![
        0x81, 0x00, 0x00, 0x00,
    ];

    let compressor = Diff8Filter::default();
    let output = compressor.compress(&input).unwrap();
    assert_eq!(output, expected_output);
}

#[test]
fn test_filter_unfilter_diff8_1() {
    let input: Vec<u8> = vec![
        0xAD, 0x98, 0xFB, 0x7F, 0xCE, 0x66, 0x76, 0x8F,
        0xDC, 0x0C, 0x9A, 0x8C, 0x70, 0x66, 0x95, 0x84,
        0xC7, 0x23, 0x89, 0xED, 0x92, 0xD5, 0x06, 0x8C,
        0x8C, 0xA1, 0xD4, 0x48, 0xEA, 0xC9, 0x9E, 0x90,
    ];

    let compressor = Diff8Filter::default();
    let immediate = compressor.compress(&input).unwrap();
    let output = compressor.decompress(&immediate).unwrap();
    assert_eq!(input, output);
}

#[test]
fn test_filter_unfilter_diff8_2() {
    let input: Vec<u8> = Vec::new();

    let compressor = Diff8Filter::default();
    let immediate = compressor.compress(&input).unwrap();
    let output = compressor.decompress(&immediate).unwrap();
    assert_eq!(input, output);
}

#[test]
fn test_filter_unfilter_diff8_3() {
    let input: Vec<u8> = vec![0x42; 4096];

    let compressor = Diff8Filter::default();
    let immediate = compressor.compress(&input).unwrap();
    let output = compressor.decompress(&immediate).unwrap();
    assert_eq!(input, output);
}

#[test]
fn test_filter_unfilter_diff16_1() {
    let input: Vec<u8> = vec![
        0xAD, 0x98, 0xFB, 0x7F, 0xCE, 0x66, 0x76, 0x8F,
        0xDC, 0x0C, 0x9A, 0x8C, 0x70, 0x66, 0x95, 0x84,
        0xC7, 0x23, 0x89, 0xED, 0x92, 0xD5, 0x06, 0x8C,
        0x8C, 0xA1, 0xD4, 0x48, 0xEA, 0xC9, 0x9E, 0x90,
    ];

    let compressor = Diff16Filter::default();
    let immediate = compressor.compress(&input).unwrap();
    let output = compressor.decompress(&immediate).unwrap();
    assert_eq!(input, output);
}

#[test]
fn test_filter_unfilter_diff16_2() {
    let input: Vec<u8> = Vec::new();

    let compressor = Diff16Filter::default();
    let immediate = compressor.compress(&input).unwrap();
    let output = compressor.decompress(&immediate).unwrap();
    assert_eq!(input, output);
}

#[test]
fn test_filter_unfilter_diff16_3() {
    let input: Vec<u8> = vec![0x42; 4096];

    let compressor = Diff16Filter::default();
    let immediate = compressor.compress(&input).unwrap();
    let output = compressor.decompress(&immediate).unwrap();
    assert_eq!(input, output);
}
