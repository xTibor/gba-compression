pub fn consecutive_count<T: Eq>(buffer: &[T], max_length: usize) -> usize {
    let mut count = 0;
    while (count < buffer.len()) && (count < max_length) && (buffer[0] == buffer[count]) {
        count += 1;
    }
    count
}

pub fn non_consecutive_count<T: Eq>(buffer: &[T], max_length: usize, consecutive_threshold: usize) -> usize {
    let mut count = 0;
    while (count < buffer.len()) && (count < max_length) && (consecutive_count(&buffer[count..], max_length) < consecutive_threshold) {
        count += 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use utils::{consecutive_count, non_consecutive_count};

    #[test]
    fn test_consecutive_count() {
        assert_eq!(consecutive_count::<u8>(&vec![                            ], 9), 0);
        assert_eq!(consecutive_count::<u8>(&vec![0x00                        ], 9), 1);
        assert_eq!(consecutive_count::<u8>(&vec![0x00, 0x00, 0x00, 0x00      ], 9), 4);
        assert_eq!(consecutive_count::<u8>(&vec![0x00, 0x00, 0x00, 0x00, 0x01], 9), 4);
        assert_eq!(consecutive_count::<u8>(&vec![0x00, 0x00, 0x00, 0x00      ], 3), 3);
        assert_eq!(consecutive_count::<u8>(&vec![0x00, 0x00                  ], 3), 2);
        assert_eq!(consecutive_count::<u8>(&vec![0x00, 0x00, 0x01            ], 3), 2);
    }

    #[test]
    fn test_non_consecutive_count() {
        assert_eq!(non_consecutive_count::<u8>(&vec![                                  ], 9, 2), 0);
        assert_eq!(non_consecutive_count::<u8>(&vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05], 9, 2), 6);
        assert_eq!(non_consecutive_count::<u8>(&vec![0x00, 0x00, 0x01, 0x01, 0x02, 0x02], 9, 3), 6);
        assert_eq!(non_consecutive_count::<u8>(&vec![0x00, 0x00, 0x01, 0x01, 0x02, 0x02], 3, 3), 3);
        assert_eq!(non_consecutive_count::<u8>(&vec![0x00, 0x00, 0x00, 0x01, 0x01, 0x01], 9, 3), 0);
    }
}
