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
