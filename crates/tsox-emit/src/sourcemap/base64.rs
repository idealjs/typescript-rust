pub(super) fn base64_format_encode(value: i32) -> char {
    match value {
        0..=25 => (b'A' + value as u8) as char,
        26..=51 => (b'a' + (value - 26) as u8) as char,
        52..=61 => (b'0' + (value - 52) as u8) as char,
        62 => '+',
        63 => '/',
        _ => panic!("not a base64 value: {value}"),
    }
}

pub(super) fn base64_format_decode(ch: u8) -> i32 {
    match ch {
        b'A'..=b'Z' => (ch - b'A') as i32,
        b'a'..=b'z' => (ch - b'a' + 26) as i32,
        b'0'..=b'9' => (ch - b'0' + 52) as i32,
        b'+' => 62,
        b'/' => 63,
        _ => -1,
    }
}
