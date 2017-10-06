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
    fn test_consecutive_count_1() {
        let input: Vec<u8> = Vec::new();
        assert_eq!(consecutive_count(&input, 256), 0);
    }

    #[test]
    fn test_consecutive_count_2() {
        let input: Vec<u8> = vec![0x00];
        assert_eq!(consecutive_count(&input, 256), 1);
    }

    #[test]
    fn test_consecutive_count_3() {
        let input: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00];
        assert_eq!(consecutive_count(&input, 256), 4);
    }

    #[test]
    fn test_consecutive_count_4() {
        let input: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x01];
        assert_eq!(consecutive_count(&input, 256), 4);
    }

    #[test]
    fn test_consecutive_count_5() {
        let input: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00];
        assert_eq!(consecutive_count(&input, 3), 3);
    }

    #[test]
    fn test_consecutive_count_6() {
        let input: Vec<u8> = vec![0x00, 0x00];
        assert_eq!(consecutive_count(&input, 3), 2);
    }

    #[test]
    fn test_consecutive_count_7() {
        let input: Vec<u8> = vec![0x00, 0x00, 0x01];
        assert_eq!(consecutive_count(&input, 3), 2);
    }

    #[test]
    fn test_non_consecutive_count_1() {
        let input: Vec<u8> = Vec::new();
        assert_eq!(non_consecutive_count(&input, 256, 2), 0);
    }

    #[test]
    fn test_non_consecutive_count_2() {
        let input: Vec<u8> = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
        assert_eq!(non_consecutive_count(&input, 256, 2), 6);
    }

    #[test]
    fn test_non_consecutive_count_3() {
        let input: Vec<u8> = vec![0x00, 0x00, 0x01, 0x01, 0x02, 0x02];
        assert_eq!(non_consecutive_count(&input, 256, 3), 6);
    }

    #[test]
    fn test_non_consecutive_count_4() {
        let input: Vec<u8> = vec![0x00, 0x00, 0x01, 0x01, 0x02, 0x02];
        assert_eq!(non_consecutive_count(&input, 3, 3), 3);
    }

    #[test]
    fn test_non_consecutive_count_5() {
        let input: Vec<u8> = vec![0x00, 0x00, 0x00, 0x01, 0x01, 0x01];
        assert_eq!(non_consecutive_count(&input, 256, 3), 0);
    }
}
