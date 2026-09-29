#![allow(dead_code, unused_imports, unused_variables)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadRangeRuneError {
    BadRange,
    InvalidUtf8,
}

fn read_range_rune(input: &str) -> (char, usize, Option<ReadRangeRuneError>) {
    match input.chars().next() {
        Some(c) => (c, c.len_utf8(), None),
        None => ('\0', 0, Some(ReadRangeRuneError::BadRange)),
    }
}
