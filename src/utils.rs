pub fn consecutive_count<T: Eq>(buf: &[T], max_length: usize) -> usize {
    let mut i = 0;
    while (i < buf.len()) && (i < max_length) && (buf[0] == buf[i]) {
        i += 1;
    }
    i
}

pub fn non_consecutive_count<T: Eq>(buf: &[T], max_length: usize, consecutive_threshold: usize) -> usize {
    let mut i = 0;
    while (i < buf.len()) && (i < max_length) && (consecutive_count(&buf[i..], max_length) < consecutive_threshold) {
        i += 1;
    }
    i
}

pub fn same_count<T: Eq>(buf1: &[T], buf2: &[T], max_length: usize) -> usize {
    let mut i = 0;
    while (i < buf1.len()) && (i < buf2.len()) && (i < max_length) && (buf1[i] == buf2[i]) {
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use utils::{consecutive_count, non_consecutive_count, same_count};

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

    #[test]
    fn test_same_count() {
        assert_eq!(same_count::<u8>(&vec![                ], &vec![                ], 6), 0);
        assert_eq!(same_count::<u8>(&vec![0x00, 0x01      ], &vec![                ], 6), 0);
        assert_eq!(same_count::<u8>(&vec![                ], &vec![0x00, 0x01      ], 6), 0);
        assert_eq!(same_count::<u8>(&vec![0x00, 0x01      ], &vec![0x00, 0x01      ], 6), 2);
        assert_eq!(same_count::<u8>(&vec![0x00, 0x01      ], &vec![0x02, 0x03      ], 6), 0);
        assert_eq!(same_count::<u8>(&vec![0x00, 0x01, 0x02], &vec![0x00, 0x01, 0x03], 6), 2);
        assert_eq!(same_count::<u8>(&vec![0x00, 0x01, 0x02], &vec![0x00, 0x01, 0x02], 2), 2);
        assert_eq!(same_count::<u8>(&vec![0x00, 0x01, 0x02], &vec![0x00, 0x01, 0x02], 4), 3);
    }
}
