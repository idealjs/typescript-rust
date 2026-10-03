pub fn is_white_space_like(ch: char) -> bool { crate::fntrace::enter("is_white_space_like"); 
    is_white_space_single_line(ch) || is_line_break(ch)
}

pub fn is_white_space_single_line(ch: char) -> bool { crate::fntrace::enter("is_white_space_single_line"); 
    matches!(
        ch,
        ' ' | '\t'
            | '\u{000B}'
            | '\u{000C}'
            | '\u{0085}'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'
            | '\u{2001}'
            | '\u{2002}'
            | '\u{2003}'
            | '\u{2004}'
            | '\u{2005}'
            | '\u{2006}'
            | '\u{2007}'
            | '\u{2008}'
            | '\u{2009}'
            | '\u{200A}'
            | '\u{200B}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
            | '\u{FEFF}'
    )
}

pub fn is_line_break(ch: char) -> bool { crate::fntrace::enter("is_line_break"); 
    matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

pub fn is_digit(ch: char) -> bool { crate::fntrace::enter("is_digit"); 
    ch.is_ascii_digit()
}

pub fn is_octal_digit(ch: char) -> bool { crate::fntrace::enter("is_octal_digit"); 
    ('0'..='7').contains(&ch)
}

pub fn is_hex_digit(ch: char) -> bool { crate::fntrace::enter("is_hex_digit"); 
    ch.is_ascii_hexdigit()
}

pub fn is_ascii_letter(ch: char) -> bool { crate::fntrace::enter("is_ascii_letter"); 
    ch.is_ascii_alphabetic()
}

pub fn contains_non_ascii(s: &str) -> bool { crate::fntrace::enter("contains_non_ascii"); 
    s.bytes().any(|b| b >= 0x80)
}

pub fn equate_string_case_insensitive(a: &str, b: &str) -> bool { crate::fntrace::enter("equate_string_case_insensitive"); 
    a.eq_ignore_ascii_case(b)
}

pub fn equate_string_case_sensitive(a: &str, b: &str) -> bool { crate::fntrace::enter("equate_string_case_sensitive"); 
    a == b
}

pub fn compare_strings_case_insensitive(a: &str, b: &str) -> i32 { crate::fntrace::enter("compare_strings_case_insensitive"); 
    a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()) as i32
}

pub fn compare_strings_case_sensitive(a: &str, b: &str) -> i32 { crate::fntrace::enter("compare_strings_case_sensitive"); 
    a.cmp(b) as i32
}

pub fn split_lines(text: &str) -> Vec<String> { crate::fntrace::enter("split_lines"); 
    let mut lines = Vec::new();
    let mut start = 0;
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            lines.push(text[start..i].to_string());
            start = i + 1;
        } else if bytes[i] == b'\r' {
            lines.push(text[start..i].to_string());
            start = i + 1;
            if start < bytes.len() && bytes[start] == b'\n' {
                start += 1;
                i += 1;
            }
        }
        i += 1;
    }
    lines.push(text[start..].to_string());
    lines
}

pub(crate) const UPPER_HEX: &[u8] = b"0123456789ABCDEF";

pub fn encode_uri(s: &str) -> String { crate::fntrace::enter("encode_uri"); 
    let mut result = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        if should_escape_for_encode_uri(b) {
            result.push('%');
            result.push(UPPER_HEX[(b >> 4) as usize] as char);
            result.push(UPPER_HEX[(b & 0x0f) as usize] as char);
        } else {
            result.push(b as char);
        }
    }
    result
}

pub(crate) fn should_escape_for_encode_uri(b: u8) -> bool { crate::fntrace::enter("should_escape_for_encode_uri"); 
    if b.is_ascii_alphanumeric() {
        return false;
    }
    !matches!(
        b,
        b';' | b'/'
            | b'?'
            | b':'
            | b'@'
            | b'&'
            | b'='
            | b'+'
            | b'$'
            | b','
            | b'#'
            | b'-'
            | b'_'
            | b'.'
            | b'!'
            | b'~'
            | b'*'
            | b'\''
            | b'('
            | b')'
    )
}

pub fn to_lower_js(s: &str) -> String { crate::fntrace::enter("to_lower_js"); 
    s.chars().flat_map(char::to_lowercase).collect()
}

pub fn to_upper_js(s: &str) -> String { crate::fntrace::enter("to_upper_js"); 
    s.chars().flat_map(char::to_uppercase).collect()
}

pub fn encode_js_string_rune(ch: u32) -> String { crate::fntrace::enter("encode_js_string_rune"); 
    if let Some(c) = char::from_u32(ch) {
        c.to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
pub(crate) mod tests;

pub fn edit_distance(a: &str, b: &str) -> usize { crate::fntrace::enter("edit_distance"); 
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();

    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        curr[0] = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

// r 轮接线:迁移批次模块
pub mod mig;
