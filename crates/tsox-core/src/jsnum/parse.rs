use super::number::Number;

fn is_number_rune(r: char) -> bool {
    if r.is_ascii_digit() {
        return true;
    }
    if ('a'..='f').contains(&r) || ('A'..='F').contains(&r) {
        return true;
    }
    matches!(
        r,
        '.' | '-' | '+' | 'x' | 'X' | 'o' | 'O' | 'b' | 'B' | 'e' | 'E'
    )
}

impl Number {
    pub fn from_string(s: &str) -> Number {
        let s = s.trim_matches(|c: char| {
            matches!(
                c,
                '\n' | '\r' | '\u{2028}' | '\u{2029}' | '\t' | '\u{000B}' | '\u{000C}' | '\u{FEFF}'
            ) || c.is_whitespace()
        });

        match s {
            "" => return Number(0.0),
            "Infinity" | "+Infinity" => return Number::inf(1),
            "-Infinity" => return Number::inf(-1),
            _ => {}
        }

        for r in s.chars() {
            if !is_number_rune(r) {
                return Number::nan();
            }
        }

        if s.len() > 2 {
            let prefix = &s[..2];
            let rest = &s[2..];
            match prefix {
                "0b" | "0B" => {
                    if rest.chars().all(|c| c == '0' || c == '1') {
                        if let Ok(i) = i64::from_str_radix(rest, 2) {
                            return Number(i as f64);
                        }
                    }
                    return Number::nan();
                }
                "0o" | "0O" => {
                    if rest.chars().all(|c| ('0'..='7').contains(&c)) {
                        if let Ok(i) = i64::from_str_radix(rest, 8) {
                            return Number(i as f64);
                        }
                    }
                    return Number::nan();
                }
                "0x" | "0X" => {
                    if rest.chars().all(|c| c.is_ascii_hexdigit()) {
                        if let Ok(i) = i64::from_str_radix(rest, 16) {
                            return Number(i as f64);
                        }

                        if let Ok(n) = u128::from_str_radix(rest, 16) {
                            return Number(n as f64);
                        }
                    }
                    return Number::nan();
                }
                _ => {}
            }
        }

        if s.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(i) = s.parse::<i64>() {
                return Number(i as f64);
            }

            if let Ok(f) = s.parse::<f64>() {
                return Number(f);
            }
            return Number::nan();
        }

        if let Ok(f) = s.parse::<f64>() {
            return Number(f);
        }

        Number::nan()
    }
}
