
pub(crate) fn write_word_at(start: usize, source: &[u8; 2], destination: &mut Vec<u8>) {
    destination[start] = source[0];
    destination[start+1] = source[1];
}



#[test]
    fn test_write_word_at() {
        let mut destination = vec![0u8; 10];
        let source: [u8; 2] = [42, 99];
        write_word_at(3, &source, &mut destination);
        assert_eq!(destination[3], 42);
        assert_eq!(destination[4], 99);
    }