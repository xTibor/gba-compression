use bios::{compress_rle, decompress_rle};

#[test]
fn test_decompress_1() {
    let input: Vec<u8> = vec![
        0x30, 0x08, 0x00, 0x00,
        0x03, 0x01, 0x02, 0x03, 0x04,
        0x81, 0x05,
    ];
    let expected_output: Vec<u8> = vec![
        0x01, 0x02, 0x03, 0x04,
        0x05, 0x05, 0x05, 0x05,
    ];

    let output = decompress_rle(&input).unwrap();
    assert_eq!(output, expected_output);
}

#[test]
fn test_decompress_2() {
    let input: Vec<u8> = vec![
        0x30, 0x04, 0x00, 0x00,
        0x04, 0x01, 0x02, 0x03, 0x04, 0x05,
    ];

    assert!(decompress_rle(&input).is_err());
}

#[test]
fn test_decompress_3() {
    let input: Vec<u8> = vec![
        0x30, 0x04, 0x00, 0x00,
        0x82, 0x01,
    ];

    assert!(decompress_rle(&input).is_err());
}

#[test]
fn test_decompress_4() {
    let input: Vec<u8> = vec![
        0x30, 0x00, 0x00, 0x00,
    ];

    let output = decompress_rle(&input).unwrap();
    assert!(output.is_empty());
}

#[test]
fn test_compress_and_decompress_1() {
    let input: Vec<u8> = Vec::new();

    let immediate = compress_rle(&input).unwrap();
    let output = decompress_rle(&immediate).unwrap();
    assert_eq!(input, output);
}

#[test]
fn test_compress_and_decompress_2() {
    let input: Vec<u8> = vec![
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
        0x03, 0x04, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05,
        0x06, 0x07, 0x08, 0x09, 0x09, 0x09, 0x09, 0x09,
    ];

    let immediate = compress_rle(&input).unwrap();
    let output = decompress_rle(&immediate).unwrap();
    assert_eq!(input, output);
}

#[test]
fn test_compress_and_decompress_3() {
    let input: Vec<u8> = vec![0x42; 4096];

    let immediate = compress_rle(&input).unwrap();
    let output = decompress_rle(&immediate).unwrap();
    assert_eq!(input, output);
}
