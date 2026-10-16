use super::unicode_data::is_unicode_cased;
use super::unicode_data::is_unicode_case_ignorable;
use super::unicode_data::special_casing_mappings;
use super::unicode_data::SpecialCasingCondition;
use super::unicode_data::SpecialCasingMapping;

pub fn to_lower_js(s: &str) -> String { crate::fntrace::enter("to_lower_js"); 
    if let Some(ascii) = to_lower_ascii(s) {
        return ascii;
    }
    let mut builder = String::with_capacity(s.len());
    let mut cased_before = false;
    let mut i = 0;
    while i < s.len() {
        let (r, size) = decode_js_string_rune(&s[i..]);
        i += size;
        if is_surrogate(r) {
            builder.push_str(&encode_js_string_rune(r));
        } else if let Some(mapping) = special_casing_mappings(r) {
            if mapping.condition == Some(SpecialCasingCondition::FinalSigma)
                && is_final_sigma_context(cased_before, s, i)
            {
                builder.push_str(mapping.conditional_lower);
            } else {
                builder.push_str(mapping.lower);
            }
        } else if let Some(c) = char::from_u32(r) {
            builder.push(c);
        }
        if !is_unicode_case_ignorable(r) {
            cased_before = is_sigma_cased(r);
        }
    }
    builder
}

pub fn to_upper_js(s: &str) -> String { crate::fntrace::enter("to_upper_js"); 
    if let Some(ascii) = to_upper_ascii(s) {
        return ascii;
    }
    let mut builder = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let (r, size) = decode_js_string_rune(&s[i..]);
        if is_surrogate(r) {
            builder.push_str(&encode_js_string_rune(r));
        } else if let Some(mapping) = special_casing_mappings(r) {
            builder.push_str(mapping.upper);
        } else if let Some(c) = char::from_u32(r) {
            builder.push(c);
        }
        i += size;
    }
    builder
}

fn to_lower_ascii(s: &str) -> Option<String> { crate::fntrace::enter("to_lower_ascii"); 
    let mut needs_mapping = false;
    for &ch in s.as_bytes() {
        if ch >= 0x80 {
            return None;
        }
        needs_mapping = needs_mapping || ch.is_ascii_uppercase();
    }
    if !needs_mapping {
        return Some(s.to_string());
    }
    Some(s.chars().map(|c| c.to_ascii_lowercase()).collect())
}

fn to_upper_ascii(s: &str) -> Option<String> { crate::fntrace::enter("to_upper_ascii"); 
    let mut needs_mapping = false;
    for &ch in s.as_bytes() {
        if ch >= 0x80 {
            return None;
        }
        needs_mapping = needs_mapping || ch.is_ascii_lowercase();
    }
    if !needs_mapping {
        return Some(s.to_string());
    }
    Some(s.chars().map(|c| c.to_ascii_uppercase()).collect())
}

fn is_final_sigma_context(cased_before: bool, s: &str, after_offset: usize) -> bool { crate::fntrace::enter("is_final_sigma_context"); 
    cased_before && !has_sigma_cased_after(s, after_offset)
}

fn has_sigma_cased_after(s: &str, start: usize) -> bool { crate::fntrace::enter("has_sigma_cased_after"); 
    let mut i = start;
    while i < s.len() {
        let (r, size) = decode_js_string_rune(&s[i..]);
        i += size;
        if is_unicode_case_ignorable(r) {
            continue;
        }
        return is_sigma_cased(r);
    }
    false
}

fn is_sigma_cased(r: u32) -> bool { crate::fntrace::enter("is_sigma_cased"); 
    is_unicode_cased(r)
}

pub fn guess_indentation(lines: &[String]) -> usize { crate::fntrace::enter("guess_indentation"); 
    const MAX_SMI_X86: usize = 0x3fff_ffff;
    let mut indentation = MAX_SMI_X86;
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let mut i = 0;
        while i < line.len() && i < indentation {
            let (r, size) = decode_js_string_rune(&line[i..]);
            let ch = char::from_u32(r).unwrap_or('\u{FFFD}');
            if !is_white_space_like(ch) {
                break;
            }
            i += size;
        }
        if i < indentation {
            indentation = i;
        }
        if indentation == 0 {
            return 0;
        }
    }
    if indentation == MAX_SMI_X86 {
        return 0;
    }
    indentation
}

fn is_white_space_like(ch: char) -> bool { crate::fntrace::enter("is_white_space_like"); 
    super::super::is_white_space_like(ch)
}

pub fn get_byte_order_mark_length(text: &str) -> usize { crate::fntrace::enter("get_byte_order_mark_length"); 
    let bytes = text.as_bytes();
    if !bytes.is_empty() {
        let ch0 = bytes[0];
        if ch0 == 0xfe {
            if bytes.len() >= 2 && bytes[1] == 0xff {
                return 2;
            }
            return 0;
        }
        if ch0 == 0xff {
            if bytes.len() >= 2 && bytes[1] == 0xfe {
                return 2;
            }
            return 0;
        }
        if ch0 == 0xef {
            if bytes.len() >= 3 && bytes[1] == 0xbb && bytes[2] == 0xbf {
                return 3;
            }
            return 0;
        }
    }
    0
}

pub fn remove_byte_order_mark(text: &str) -> &str { crate::fntrace::enter("remove_byte_order_mark"); 
    let length = get_byte_order_mark_length(text);
    if length > 0 {
        &text[length..]
    } else {
        text
    }
}

pub fn add_utf8_byte_order_mark(text: &str) -> String { crate::fntrace::enter("add_utf8_byte_order_mark"); 
    if get_byte_order_mark_length(text) == 0 {
        let mut result = String::with_capacity(text.len() + 3);
        result.push('\u{FEFF}');
        result.push_str(text);
        return result;
    }
    text.to_string()
}

fn strip_quotes(name: &str) -> &str { crate::fntrace::enter("strip_quotes"); 
    if name.len() < 2 {
        return name;
    }
    let first_char = name.chars().next().unwrap_or('\0');
    let last_char = name.chars().next_back().unwrap_or('\0');
    if first_char == last_char && (first_char == '\'' || first_char == '"' || first_char == '`') {
        let end = name.len() - last_char.len_utf8();
        return &name[first_char.len_utf8()..end];
    }
    name
}

fn match_slash_replacer(input: &str) -> &str { crate::fntrace::enter("match_slash_replacer"); 
    &input[1..]
}

pub fn unquote_string(s: &str) -> String { crate::fntrace::enter("unquote_string"); 
    let inner = strip_quotes(s);
    let mut result = String::with_capacity(inner.len());
    let mut i = 0;
    while i < inner.len() {
        if inner.as_bytes()[i] == b'\\' {
            let match_end = next_rune_boundary(inner, i + 1);
            let repl = match_slash_replacer(&inner[i..match_end]);
            result.push_str(repl);
            i = match_end;
        } else {
            let (_, size) = decode_rune(&inner[i..]);
            result.push_str(&inner[i..i + size]);
            i += size;
        }
    }
    result
}

fn next_rune_boundary(s: &str, start: usize) -> usize { crate::fntrace::enter("next_rune_boundary"); 
    let bytes = s.as_bytes();
    let mut end = start + 1;
    while end < bytes.len() && (bytes[end] & 0xC0) == 0x80 {
        end += 1;
    }
    end
}

fn decode_rune(s: &str) -> (u32, usize) { crate::fntrace::enter("decode_rune"); 
    match s.chars().next() {
        Some(c) => (c as u32, c.len_utf8()),
        None => (0xFFFD, 0),
    }
}

pub fn lower_first_char(s: &str) -> String { crate::fntrace::enter("lower_first_char"); 
    if let Some(c) = s.chars().next() {
        let mut result = String::with_capacity(s.len());
        result.extend(c.to_lowercase());
        result.push_str(&s[c.len_utf8()..]);
        return result;
    }
    s.to_string()
}

pub fn truncate_by_runes(s: &str, max_length: usize) -> &str { crate::fntrace::enter("truncate_by_runes"); 
    if s.len() < max_length {
        return s;
    }
    if max_length == 0 {
        return "";
    }
    let mut rune_count = 0;
    for (i, _) in s.char_indices() {
        rune_count += 1;
        if rune_count > max_length {
            return &s[..i];
        }
    }
    s
}

pub fn split_lines(text: &str) -> Vec<String> { crate::fntrace::enter("split_lines"); 
    let mut lines = Vec::with_capacity(text.matches('\n').count() + 1);
    let mut start = 0;
    let mut pos = 0;
    let bytes = text.as_bytes();
    while pos < bytes.len() {
        match bytes[pos] {
            b'\r' if pos + 1 < bytes.len() && bytes[pos + 1] == b'\n' => {
                lines.push(text[start..pos].to_string());
                pos += 2;
                start = pos;
            }
            b'\r' | b'\n' => {
                lines.push(text[start..pos].to_string());
                pos += 1;
                start = pos;
            }
            _ => {
                pos += 1;
            }
        }
    }
    if start < bytes.len() {
        lines.push(text[start..].to_string());
    }
    lines
}

pub const SURROGATE_LOW_START: u32 = 0xDC00;

pub fn is_high_surrogate(ch: u32) -> bool { crate::fntrace::enter("is_high_surrogate"); 
    is_surrogate(ch) && ch < SURROGATE_LOW_START
}

pub fn is_low_surrogate(ch: u32) -> bool { crate::fntrace::enter("is_low_surrogate"); 
    is_surrogate(ch) && ch >= SURROGATE_LOW_START
}

pub fn is_surrogate(ch: u32) -> bool { crate::fntrace::enter("is_surrogate"); 
    (0xD800..=0xDFFF).contains(&ch)
}

pub fn surrogate_pair_to_code_point(high: u32, low: u32) -> u32 { crate::fntrace::enter("surrogate_pair_to_code_point"); 
    if is_high_surrogate(high) && is_low_surrogate(low) {
        (((high - 0xD800) << 10) | (low - 0xDC00)) + 0x10000
    } else {
        0xFFFD
    }
}

pub fn code_point_to_surrogate_pair(ch: u32) -> (u32, u32) { crate::fntrace::enter("code_point_to_surrogate_pair"); 
    if (0x10000..=0x10FFFF).contains(&ch) {
        let ch = ch - 0x10000;
        (0xD800 + (ch >> 10), 0xDC00 + (ch & 0x3FF))
    } else {
        (0xFFFD, 0xFFFD)
    }
}

const SURROGATE_UTF8_LEAD: u8 = 0xED;
const SURROGATE_UTF8_LEAD_BITS: u32 = 0xD000;
const UTF8_CONT_MARKER: u8 = 0x80;
const UTF8_CONT_MAX: u8 = 0xBF;
const UTF8_CONT_MASK: u8 = 0x3F;
const SURROGATE_UTF8_BYTE1_MIN: u8 = 0xA0;
const SURROGATE_UTF8_BYTE1_MAX: u8 = 0xBF;

pub fn encode_js_string_rune(ch: u32) -> String { crate::fntrace::enter("encode_js_string_rune"); 
    if is_surrogate(ch) {
        let bytes = [
            SURROGATE_UTF8_LEAD,
            UTF8_CONT_MARKER | ((ch >> 6) as u8 & UTF8_CONT_MASK),
            UTF8_CONT_MARKER | (ch as u8 & UTF8_CONT_MASK),
        ];
        return unsafe { String::from_utf8_unchecked(bytes.to_vec()) };
    }
    match char::from_u32(ch) {
        Some(c) => c.to_string(),
        None => String::new(),
    }
}

pub fn decode_js_string_rune(s: &str) -> (u32, usize) { crate::fntrace::enter("decode_js_string_rune"); 
    let bytes = s.as_bytes();
    if bytes.len() >= 3
        && bytes[0] == SURROGATE_UTF8_LEAD
        && bytes[1] >= SURROGATE_UTF8_BYTE1_MIN
        && bytes[1] <= SURROGATE_UTF8_BYTE1_MAX
        && bytes[2] >= UTF8_CONT_MARKER
        && bytes[2] <= UTF8_CONT_MAX
    {
        return (
            SURROGATE_UTF8_LEAD_BITS
                | ((bytes[1] & UTF8_CONT_MASK) as u32) << 6
                | (bytes[2] & UTF8_CONT_MASK) as u32,
            3,
        );
    }
    decode_rune(s)
}

pub fn combine_surrogate_pairs(s: &str) -> String { crate::fntrace::enter("combine_surrogate_pairs"); 
    if !s.as_bytes().contains(&SURROGATE_UTF8_LEAD) {
        return s.to_string();
    }
    let mut b = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let (r, size) = decode_js_string_rune(&s[i..]);
        if is_high_surrogate(r) {
            let (low, low_size) = decode_js_string_rune(&s[i + size..]);
            if is_low_surrogate(low) {
                let cp = surrogate_pair_to_code_point(r, low);
                if let Some(c) = char::from_u32(cp) {
                    b.push(c);
                }
                i += size + low_size;
                continue;
            }
        }
        b.push_str(&s[i..i + size]);
        i += size;
    }
    b
}
