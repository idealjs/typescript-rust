#[derive(Debug)]
pub struct PositionMap {
    ascii_only: bool,
    entries: Vec<PositionMapEntry>,
}

#[derive(Debug)]
struct PositionMapEntry {

    utf8_pos: usize,

    delta: usize,
}

pub fn compute_position_map(text: &str) -> PositionMap {
    let mut entries = Vec::new();
    let mut delta = 0usize;

    for (byte_offset, ch) in text.char_indices() {
        let utf8_size = ch.len_utf8();
        if utf8_size <= 1 {
            continue;
        }
        let utf16_size = ch.len_utf16();
        delta += utf8_size - utf16_size;
        entries.push(PositionMapEntry {
            utf8_pos: byte_offset + utf8_size,
            delta,
        });
    }

    let ascii_only = entries.is_empty();
    PositionMap {
        ascii_only,
        entries,
    }
}

impl PositionMap {

    pub fn is_ascii_only(&self) -> bool {
        self.ascii_only
    }

    pub fn utf8_to_utf16(&self, utf8_offset: usize) -> usize {
        if self.ascii_only {
            return utf8_offset;
        }

        let lo = self.entries.partition_point(|e| e.utf8_pos <= utf8_offset);
        if lo == 0 {
            return utf8_offset;
        }
        utf8_offset - self.entries[lo - 1].delta
    }

    pub fn utf16_to_utf8(&self, utf16_offset: usize) -> usize {
        if self.ascii_only {
            return utf16_offset;
        }

        let lo = self
            .entries
            .partition_point(|e| e.utf8_pos - e.delta <= utf16_offset);
        if lo == 0 {
            return utf16_offset;
        }
        utf16_offset + self.entries[lo - 1].delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_map_ascii() {
        let text = "const x = 1;";
        let pm = compute_position_map(text);
        assert!(pm.is_ascii_only());
        for i in 0..=text.len() {
            assert_eq!(pm.utf8_to_utf16(i), i, "UTF8ToUTF16({})", i);
            assert_eq!(pm.utf16_to_utf8(i), i, "UTF16ToUTF8({})", i);
        }
    }

    #[test]
    fn test_position_map_two_byte() {

        let text = "const café = 1;\nconst x = 2;";
        let pm = compute_position_map(text);
        assert!(!pm.is_ascii_only());

        for i in 0..10 {
            assert_eq!(pm.utf8_to_utf16(i), i, "before é: UTF8ToUTF16({})", i);
        }

        assert_eq!(pm.utf8_to_utf16(9), 9, "at é: UTF8ToUTF16(9)");

        assert_eq!(pm.utf8_to_utf16(11), 10, "after é: UTF8ToUTF16(11)");

        let x_utf8 = text.rfind('x').unwrap();
        assert_eq!(pm.utf8_to_utf16(x_utf8), x_utf8 - 1, "at x: UTF8ToUTF16");

        let x_utf16 = x_utf8 - 1;
        assert_eq!(
            pm.utf16_to_utf8(x_utf16),
            x_utf8,
            "reverse at x: UTF16ToUTF8"
        );
    }

    #[test]
    fn test_position_map_four_byte() {

        let text = "const a = \"🎉\";\nconst b = 2;";
        let pm = compute_position_map(text);
        assert!(!pm.is_ascii_only());

        let b_utf8 = text.rfind('b').unwrap();
        let b_utf16 = b_utf8 - 2;
        assert_eq!(pm.utf8_to_utf16(b_utf8), b_utf16, "at b: UTF8ToUTF16");
        assert_eq!(
            pm.utf16_to_utf8(b_utf16),
            b_utf8,
            "reverse at b: UTF16ToUTF8"
        );
    }

    #[test]
    fn test_position_map_multiple_non_ascii() {

        let text = "à🎉x";
        let pm = compute_position_map(text);

        let tests = [
            (0usize, 0usize),
            (2, 1),
            (6, 3),
            (7, 4),
        ];
        for &(utf8, utf16) in &tests {
            assert_eq!(pm.utf8_to_utf16(utf8), utf16, "UTF8ToUTF16({})", utf8);
            assert_eq!(pm.utf16_to_utf8(utf16), utf8, "UTF16ToUTF8({})", utf16);
        }
    }

    #[test]
    fn test_position_map_roundtrip() {
        let text = "let café = \"🎉\"; // naïve";
        let pm = compute_position_map(text);

        let utf16_len = pm.utf8_to_utf16(text.len());
        for i in 0..=utf16_len {
            let utf8_pos = pm.utf16_to_utf8(i);
            let back = pm.utf8_to_utf16(utf8_pos);
            assert_eq!(
                back, i,
                "roundtrip UTF16->UTF8->UTF16: {} -> {} -> {}",
                i, utf8_pos, back
            );
        }
    }

    #[test]
    fn test_position_map_three_byte_cjk() {

        let text = "let 快 = 1;";
        let pm = compute_position_map(text);
        assert!(!pm.is_ascii_only());

        let one_utf8 = text.rfind('1').unwrap();
        let one_utf16 = one_utf8 - 2;
        assert_eq!(pm.utf8_to_utf16(one_utf8), one_utf16, "at 1: UTF8ToUTF16");
        assert_eq!(
            pm.utf16_to_utf8(one_utf16),
            one_utf8,
            "reverse at 1: UTF16ToUTF8"
        );
    }

    #[test]
    fn test_position_map_lone_surrogate_sentinel() {

        let text = "a\u{10000}b";
        let pm = compute_position_map(text);
        assert!(!pm.is_ascii_only());

        assert_eq!(text.len(), 6);

        assert_eq!(pm.utf8_to_utf16(text.len()), 4);

        assert_eq!(pm.utf16_to_utf8(3), text.len() - 1);
    }
}
