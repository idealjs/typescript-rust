use super::number::{MAX_SAFE_INTEGER, MIN_SAFE_INTEGER, Number};
use std::fmt;

fn expand_exponential(s: &str) -> String {
    let (negative, rest) = if let Some(r) = s.strip_prefix('-') {
        (true, r)
    } else {
        (false, s)
    };

    let e_pos = match rest.find(|c| c == 'e' || c == 'E') {
        Some(p) => p,
        None => return s.to_string(),
    };

    let mantissa = &rest[..e_pos];
    let exp_str = &rest[e_pos + 1..];
    let exp_str = exp_str.strip_prefix('+').unwrap_or(exp_str);
    let exp: i32 = exp_str.parse().unwrap_or(0);

    let (int_part, frac_part) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let all_digits = format!("{}{}", int_part, frac_part);
    let decimal_pos = int_part.len() as i32 + exp;

    let mut result = String::new();
    if negative {
        result.push('-');
    }

    if decimal_pos <= 0 {
        result.push_str("0.");
        for _ in 0..(-decimal_pos) {
            result.push('0');
        }
        result.push_str(&all_digits);
    } else if (decimal_pos as usize) >= all_digits.len() {
        result.push_str(&all_digits);
        for _ in 0..(decimal_pos - all_digits.len() as i32) {
            result.push('0');
        }
    } else {
        result.push_str(&all_digits[..decimal_pos as usize]);
        result.push('.');
        result.push_str(&all_digits[decimal_pos as usize..]);
    }

    result
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_nan() {
            return write!(f, "NaN");
        }
        if self.is_inf() {
            return if self.0 < 0.0 {
                write!(f, "-Infinity")
            } else {
                write!(f, "Infinity")
            };
        }

        if (MIN_SAFE_INTEGER.0..=MAX_SAFE_INTEGER.0).contains(&self.0) {
            let i = self.0 as i64;
            if i as f64 == self.0 {
                return write!(f, "{}", i);
            }
        }

        if self.0.abs() < 1e21 && self.0.fract() == 0.0 {
            let s = format!("{}", self.0);
            if s.contains('e') || s.contains('E') {
                return write!(f, "{}", expand_exponential(&s));
            }
            return write!(f, "{}", s);
        }

        write!(
            f,
            "{}",
            serde_json::to_string(&self.0).unwrap_or_else(|_| "NaN".to_string())
        )
    }
}

impl fmt::Debug for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
