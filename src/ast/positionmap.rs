//! Bidirectional mapping between UTF-8 byte offsets and UTF-16 code unit offsets.
//!
//! Ported from `internal/ast/positionmap.go`. JavaScript/TypeScript uses
//! UTF-16 code unit offsets for positions, while Rust uses UTF-8 byte offsets.
//! This module provides O(log n) conversion in either direction.

/// Provides bidirectional mapping between UTF-8 byte offsets and UTF-16 code unit offsets.
///
/// For ASCII-only text, the two are identical. For text containing non-ASCII characters,
/// the offsets diverge because multi-byte UTF-8 sequences map to different numbers of
/// UTF-16 code units:
///   - U+0000..U+007F:   1 byte  in UTF-8, 1 code unit  in UTF-16
///   - U+0080..U+07FF:   2 bytes in UTF-8, 1 code unit  in UTF-16
///   - U+0800..U+FFFF:   3 bytes in UTF-8, 1 code unit  in UTF-16
///   - U+10000..U+10FFFF: 4 bytes in UTF-8, 2 code units in UTF-16 (surrogate pair)
#[derive(Debug)]
pub struct PositionMap {
    ascii_only: bool,
    entries: Vec<PositionMapEntry>,
}

#[derive(Debug)]
struct PositionMapEntry {
    /// UTF-8 byte offset AFTER this multi-byte character.
    utf8_pos: usize,
    /// Cumulative (utf8 - utf16) offset difference after this character.
    delta: usize,
}

/// Build a `PositionMap` for the given text.
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
    /// Returns true if the text is ASCII-only, meaning UTF-8 and UTF-16 offsets are identical.
    pub fn is_ascii_only(&self) -> bool {
        self.ascii_only
    }

    /// Converts a UTF-8 byte offset to a UTF-16 code unit offset.
    pub fn utf8_to_utf16(&self, utf8_offset: usize) -> usize {
        if self.ascii_only {
            return utf8_offset;
        }
        // Binary search: find the last entry where utf8_pos <= utf8_offset.
        let lo = self
            .entries
            .partition_point(|e| e.utf8_pos <= utf8_offset);
        if lo == 0 {
            return utf8_offset;
        }
        utf8_offset - self.entries[lo - 1].delta
    }

    /// Converts a UTF-16 code unit offset to a UTF-8 byte offset.
    pub fn utf16_to_utf8(&self, utf16_offset: usize) -> usize {
        if self.ascii_only {
            return utf16_offset;
        }
        // We need the last entry where (utf8_pos - delta) <= utf16_offset.
        // (utf8_pos - delta) is the UTF-16 offset of that entry's character.
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
        // "café" — é (U+00E9) is 2 bytes UTF-8, 1 code unit UTF-16
        let text = "const café = 1;\nconst x = 2;";
        let pm = compute_position_map(text);
        assert!(!pm.is_ascii_only());

        // Everything before é (byte offset 9) should be identity
        for i in 0..10 {
            assert_eq!(pm.utf8_to_utf16(i), i, "before é: UTF8ToUTF16({})", i);
        }

        // é starts at UTF-8 byte 9, UTF-16 offset 9: same
        assert_eq!(pm.utf8_to_utf16(9), 9, "at é: UTF8ToUTF16(9)");

        // After é (byte 11 in UTF-8 = code unit 10 in UTF-16), delta is 1
        // ' ' after café: UTF-8 byte 11, UTF-16 offset 10
        assert_eq!(pm.utf8_to_utf16(11), 10, "after é: UTF8ToUTF16(11)");

        // 'x' on second line: UTF-8 byte 23, UTF-16 offset 22
        let x_utf8 = text.rfind('x').unwrap();
        assert_eq!(pm.utf8_to_utf16(x_utf8), x_utf8 - 1, "at x: UTF8ToUTF16");

        // Reverse: UTF-16 offset 22 should map to UTF-8 byte 23
        let x_utf16 = x_utf8 - 1;
        assert_eq!(pm.utf16_to_utf8(x_utf16), x_utf8, "reverse at x: UTF16ToUTF8");
    }

    #[test]
    fn test_position_map_four_byte() {
        // 🎉 (U+1F389) is 4 bytes UTF-8, 2 code units UTF-16
        let text = "const a = \"🎉\";\nconst b = 2;";
        let pm = compute_position_map(text);
        assert!(!pm.is_ascii_only());

        // 🎉 starts at byte 11 (after `const a = "`)
        // UTF-8: bytes 11-14 (4 bytes), UTF-16: units 11-12 (2 code units)
        // After 🎉: UTF-8 byte 15, UTF-16 offset 13. Delta = 2.

        // 'b' on second line
        let b_utf8 = text.rfind('b').unwrap();
        let b_utf16 = b_utf8 - 2; // delta of 2 from emoji
        assert_eq!(pm.utf8_to_utf16(b_utf8), b_utf16, "at b: UTF8ToUTF16");
        assert_eq!(pm.utf16_to_utf8(b_utf16), b_utf8, "reverse at b: UTF16ToUTF8");
    }

    #[test]
    fn test_position_map_multiple_non_ascii() {
        // Mix of 2-byte and 4-byte characters
        // "à" (U+00E0) = 2 bytes UTF-8, 1 code unit UTF-16 (delta +1)
        // "🎉" (U+1F389) = 4 bytes UTF-8, 2 code units UTF-16 (delta +2)
        let text = "à🎉x";
        let pm = compute_position_map(text);

        // à: UTF-8 [0,2), UTF-16 [0,1)
        // 🎉: UTF-8 [2,6), UTF-16 [1,3)
        // x: UTF-8 [6,7), UTF-16 [3,4)
        let tests = [
            (0usize, 0usize),
            (2, 1),  // start of 🎉
            (6, 3),  // x
            (7, 4),  // end
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

        // Convert every valid UTF-16 position to UTF-8 and back
        let utf16_len = pm.utf8_to_utf16(text.len());
        for i in 0..=utf16_len {
            let utf8_pos = pm.utf16_to_utf8(i);
            let back = pm.utf8_to_utf16(utf8_pos);
            assert_eq!(back, i, "roundtrip UTF16->UTF8->UTF16: {} -> {} -> {}", i, utf8_pos, back);
        }
    }

    #[test]
    fn test_position_map_three_byte_cjk() {
        // CJK characters are 3 bytes UTF-8, 1 code unit UTF-16
        // 快 (U+5FEB) = 3 bytes UTF-8, 1 code unit UTF-16 (delta +2)
        let text = "let 快 = 1;";
        let pm = compute_position_map(text);
        assert!(!pm.is_ascii_only());

        // 快 starts at byte 4 (after "let ")
        // UTF-8: bytes 4-6 (3 bytes), UTF-16: unit 4 (1 code unit)
        // After 快: byte 7, UTF-16 offset 5. Delta = 2.

        // '1' at the end
        let one_utf8 = text.rfind('1').unwrap();
        let one_utf16 = one_utf8 - 2;
        assert_eq!(pm.utf8_to_utf16(one_utf8), one_utf16, "at 1: UTF8ToUTF16");
        assert_eq!(pm.utf16_to_utf8(one_utf16), one_utf8, "reverse at 1: UTF16ToUTF8");
    }
}
