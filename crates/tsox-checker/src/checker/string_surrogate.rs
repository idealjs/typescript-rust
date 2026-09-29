pub(crate) fn lone_surrogate_escape_at(s: &str, pos: usize) -> Option<u32> {
    let b = s.as_bytes();
    if pos + 6 > b.len() || b[pos] != b'\\' || b[pos + 1] != b'u' {
        return None;
    }
    let hex = &s[pos + 2..pos + 6];
    if !hex.bytes().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let n = u32::from_str_radix(hex, 16).ok()?;
    (0xD800..=0xDFFF).contains(&n).then_some(n)
}

pub(crate) fn combine_surrogate_pairs(s: &str) -> String {
    if !s.as_bytes().contains(&b'\\') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if let Some(high) = lone_surrogate_escape_at(s, i) {
            if high <= 0xDBFF
                && let Some(low) = lone_surrogate_escape_at(s, i + 6)
                && (0xDC00..=0xDFFF).contains(&low)
            {
                let cp = 0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00);
                out.push(char::from_u32(cp).unwrap_or('\u{FFFD}'));
                i += 12;
                continue;
            }
            out.push_str(&s[i..i + 6]);
            i += 6;
            continue;
        }
        let c = s[i..].chars().next().unwrap();
        out.push(c);
        i += c.len_utf8();
    }
    out
}

pub(crate) fn advance_js_code_point(s: &str, pos: usize) -> usize {
    if lone_surrogate_escape_at(s, pos).is_some() {
        return 6;
    }
    s[pos..].chars().next().map(|c| c.len_utf8()).unwrap_or(0)
}
