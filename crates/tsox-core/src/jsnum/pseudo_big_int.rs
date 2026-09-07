use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PseudoBigInt {
    pub negative: bool,
    pub base10_value: String,
}

impl PseudoBigInt {
    pub fn new(value: &str, negative: bool) -> PseudoBigInt {
        let value = value.trim_start_matches('0');
        PseudoBigInt {
            negative: negative && !value.is_empty(),
            base10_value: value.to_string(),
        }
    }

    pub fn is_zero(&self) -> bool {
        self.base10_value.is_empty()
    }

    pub fn sign(&self) -> i32 {
        if self.base10_value.is_empty() {
            0
        } else if self.negative {
            -1
        } else {
            1
        }
    }

    pub fn parse(text: &str) -> PseudoBigInt {
        let (text, negative) = text
            .strip_prefix('-')
            .map_or((text, false), |rest| (rest, true));
        let value = parse_pseudo_big_int(text);
        PseudoBigInt::new(&value, negative)
    }
}

impl fmt::Display for PseudoBigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.base10_value.is_empty() {
            return write!(f, "0");
        }
        if self.negative {
            write!(f, "-{}", self.base10_value)
        } else {
            write!(f, "{}", self.base10_value)
        }
    }
}

fn parse_pseudo_big_int(string_value: &str) -> String {
    let s = string_value.strip_suffix('n').unwrap_or(string_value);
    if s.len() > 1 {
        match s.as_bytes()[1] {
            b'b' | b'B' => {
                let digits = s[2..].replace('_', "");
                if let Ok(n) = u128::from_str_radix(&digits, 2) {
                    return n.to_string();
                }
            }
            b'o' | b'O' => {
                let digits = s[2..].replace('_', "");
                if let Ok(n) = u128::from_str_radix(&digits, 8) {
                    return n.to_string();
                }
            }
            b'x' | b'X' => {
                let digits = s[2..].replace('_', "");
                if let Ok(n) = u128::from_str_radix(&digits, 16) {
                    return n.to_string();
                }
            }
            _ => {}
        }
    }

    let s = s.trim_start_matches('0').replace('_', "");
    if s.is_empty() { "0".to_string() } else { s }
}
