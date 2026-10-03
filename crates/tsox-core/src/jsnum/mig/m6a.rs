use crate::jsnum::number::Number;
use crate::stringutil;

pub fn is_all_hex_digits(s: &str) -> bool { crate::fntrace::enter("is_all_hex_digits"); 
    s.chars().all(|r| stringutil::is_hex_digit(r))
}

pub fn is_all_octal_digits(s: &str) -> bool { crate::fntrace::enter("is_all_octal_digits"); 
    s.chars().all(|r| stringutil::is_octal_digit(r))
}

pub fn is_str_white_space(r: char) -> bool { crate::fntrace::enter("is_str_white_space"); 
    // LineTerminator
    if matches!(r, '\n' | '\r' | '\u{2028}' | '\u{2029}') {
        return true;
    }
    // WhiteSpace
    if matches!(r, '\t' | '\u{000B}' | '\u{000C}' | '\u{FEFF}') {
        return true;
    }
    // WhiteSpace: unicode.Zs
    is_unicode_space_separator(r)
}

fn is_unicode_space_separator(r: char) -> bool { crate::fntrace::enter("is_unicode_space_separator"); 
    matches!(
        r,
        '\u{0020}' | '\u{00A0}' | '\u{1680}' | '\u{2000}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}'
    )
}

pub fn try_parse_int(s: &str) -> Option<Number> { crate::fntrace::enter("try_parse_int"); 
    let mut i: i64 = 0;
    let mut err: Option<()> = None;
    let mut has_int_result = false;
    let mut rest_input = s;

    if s.len() > 2 {
        let prefix = &s[..2];
        let rest = &s[2..];
        match prefix {
            "0b" | "0B" => {
                if !is_all_binary_digits(rest) {
                    return Some(Number::nan());
                }
                match i64::from_str_radix(rest, 2) {
                    Ok(v) => i = v,
                    Err(_) => err = Some(()),
                }
                has_int_result = true;
            }
            "0o" | "0O" => {
                if !is_all_octal_digits(rest) {
                    return Some(Number::nan());
                }
                match i64::from_str_radix(rest, 8) {
                    Ok(v) => i = v,
                    Err(_) => err = Some(()),
                }
                has_int_result = true;
            }
            "0x" | "0X" => {
                if !is_all_hex_digits(rest) {
                    return Some(Number::nan());
                }
                match i64::from_str_radix(rest, 16) {
                    Ok(v) => i = v,
                    Err(_) => err = Some(()),
                }
                has_int_result = true;
            }
            _ => {}
        }
    }

    if !has_int_result {
        rest_input = trim_leading_zeros(s);
        if !is_all_digits(rest_input) {
            return None;
        }
        match rest_input.parse::<i64>() {
            Ok(v) => i = v,
            Err(_) => err = Some(()),
        }
        has_int_result = true;
    }

    if has_int_result && err.is_none() {
        return Some(Number(i as f64));
    }

    let bi = crate::jsnum::pseudo_big_int::PseudoBigInt::parse(rest_input);
    let f = pseudo_big_int_to_f64(&bi);
    Some(Number(f))
}

fn pseudo_big_int_to_f64(bi: &crate::jsnum::pseudo_big_int::PseudoBigInt) -> f64 { crate::fntrace::enter("pseudo_big_int_to_f64"); 
    if bi.is_zero() {
        return 0.0;
    }
    let magnitude: f64 = bi.base10_value.parse().unwrap_or(f64::INFINITY);
    if bi.negative {
        -magnitude
    } else {
        magnitude
    }
}

pub fn parse_float_string(s: &str) -> f64 { crate::fntrace::enter("parse_float_string"); 
    let mut a: &str;
    let mut b: &str = "";
    let mut c: &str = "";
    let mut has_dot = false;
    let mut has_exp = false;

    match s.split_once('.') {
        Some((before, rest)) => {
            a = before;
            has_dot = true;
            let (bb, cc, found) = cut_any(rest, "eE");
            b = bb;
            c = cc;
            has_exp = found;
        }
        None => {
            let (aa, cc, found) = cut_any(s, "eE");
            a = aa;
            c = cc;
            has_exp = found;
        }
    }

    let mut sb = String::with_capacity(a.len() + b.len() + c.len() + 3);

    if a.is_empty() {
        if has_dot && b.is_empty() {
            return f64::NAN;
        }
        if has_exp && c.is_empty() {
            return f64::NAN;
        }
        sb.push('0');
    } else {
        a = trim_leading_zeros(a);
        if !is_all_digits(a) {
            return f64::NAN;
        }
        sb.push_str(a);
    }

    if has_dot {
        sb.push('.');
        if b.is_empty() {
            sb.push('0');
        } else {
            b = trim_trailing_zeros(b);
            if !is_all_digits(b) {
                return f64::NAN;
            }
            sb.push_str(b);
        }
    }

    if has_exp {
        sb.push('e');
        let (c, negative) = match c.strip_prefix('-') {
            Some(c) => (c, true),
            None => (c.strip_prefix('+').unwrap_or(c), false),
        };
        if negative {
            sb.push('-');
        }
        let c = trim_leading_zeros(c);
        if !is_all_digits(c) {
            return f64::NAN;
        }
        sb.push_str(c);
    }

    string_to_float64(&sb)
}

fn cut_any<'a>(s: &'a str, cutset: &str) -> (&'a str, &'a str, bool) { crate::fntrace::enter("cut_any"); 
    match s.find(|r: char| cutset.contains(r)) {
        Some(i) => (&s[..i], &s[i + 1..], true),
        None => (s, "", false),
    }
}

pub fn trim_leading_zeros(s: &str) -> &str { crate::fntrace::enter("trim_leading_zeros"); 
    if s.starts_with('0') {
        let s = s.trim_start_matches('0');
        if s.is_empty() {
            return "0";
        }
        return s;
    }
    s
}

pub fn trim_trailing_zeros(s: &str) -> &str { crate::fntrace::enter("trim_trailing_zeros"); 
    if s.ends_with('0') {
        let s = s.trim_end_matches('0');
        if s.is_empty() {
            return "0";
        }
        return s;
    }
    s
}

pub fn string_to_float64(s: &str) -> f64 { crate::fntrace::enter("string_to_float64"); 
    match s.parse::<f64>() {
        Ok(f) => f,
        Err(_) => f64::NAN,
    }
}

pub fn is_all_digits(s: &str) -> bool { crate::fntrace::enter("is_all_digits"); 
    s.chars().all(|r| stringutil::is_digit(r))
}

pub fn is_all_binary_digits(s: &str) -> bool { crate::fntrace::enter("is_all_binary_digits"); 
    s.chars().all(|r| r == '0' || r == '1')
}
