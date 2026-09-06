use std::fmt;

pub const MAX_SAFE_INTEGER: Number = Number(9007199254740991.0);
pub const MIN_SAFE_INTEGER: Number = Number(-9007199254740991.0);

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Number(pub f64);

impl Eq for Number {}

impl std::hash::Hash for Number {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {

        if self.0.is_nan() {
            f64::NAN.to_bits().hash(state);
        } else if self.0 == 0.0 {

            0.0f64.to_bits().hash(state);
        } else {
            self.0.to_bits().hash(state);
        }
    }
}

impl Number {
    pub fn nan() -> Number {
        Number(f64::NAN)
    }

    pub fn is_nan(self) -> bool {
        self.0.is_nan()
    }

    pub fn inf(sign: i32) -> Number {
        Number(f64::INFINITY.copysign(sign as f64))
    }

    pub fn is_inf(self) -> bool {
        self.0.is_infinite()
    }

    pub fn is_finite(self) -> bool {
        self.0.is_finite()
    }

    pub fn to_int32(self) -> i32 {
        let x = self.0;

        let smi = x as i32;
        if smi as f64 == x {
            return smi;
        }

        if is_non_finite(x) {
            return 0;
        }
        let x = x.trunc();
        let x = x.rem_euclid(4294967296.0);

        if x >= 2147483648.0 {
            (x - 4294967296.0) as i32
        } else {
            x as i32
        }
    }

    pub fn to_uint32(self) -> u32 {
        self.to_int32() as u32
    }

    fn to_shift_count(self) -> u32 {
        self.to_uint32() & 31
    }

    pub fn signed_right_shift(self, y: Number) -> Number {
        Number((self.to_int32() >> y.to_shift_count()) as f64)
    }

    pub fn unsigned_right_shift(self, y: Number) -> Number {
        Number((self.to_uint32() >> y.to_shift_count()) as f64)
    }

    pub fn left_shift(self, y: Number) -> Number {
        Number((self.to_int32() << y.to_shift_count()) as f64)
    }

    pub fn bitwise_not(self) -> Number {
        Number((!self.to_int32()) as f64)
    }

    pub fn bitwise_or(self, y: Number) -> Number {
        Number((self.to_int32() | y.to_int32()) as f64)
    }

    pub fn bitwise_and(self, y: Number) -> Number {
        Number((self.to_int32() & y.to_int32()) as f64)
    }

    pub fn bitwise_xor(self, y: Number) -> Number {
        Number((self.to_int32() ^ y.to_int32()) as f64)
    }

    pub fn floor(self) -> Number {
        Number(self.0.floor())
    }

    pub fn abs(self) -> Number {
        Number(self.0.abs())
    }

    pub fn trunc(self) -> Number {
        Number(self.0.trunc())
    }

    pub fn remainder(self, d: Number) -> Number {
        if self.is_nan() || d.is_nan() {
            return Number::nan();
        }
        if self.is_inf() {
            return Number::nan();
        }
        if d.is_inf() {
            return self;
        }
        if d.0 == 0.0 {
            return Number::nan();
        }
        if self.0 == 0.0 {
            return self;
        }
        Number(self.0 % d.0)
    }

    pub fn exponentiate(self, exponent: Number) -> Number {
        let b = self.0;
        let e = exponent.0;
        if (b == 1.0 || b == -1.0) && e.is_infinite() {
            return Number::nan();
        }
        if b == 1.0 && e.is_nan() {
            return Number::nan();
        }

        if b.is_finite()
            && b.fract() == 0.0
            && b.abs() < (1u64 << 53) as f64
            && e.is_finite()
            && e >= 0.0
            && e.fract() == 0.0
            && e <= 1100.0
        {
            let base = b.abs() as u64;
            let exp = e as u32;
            let result = pow_exact_f64(base, exp);
            return Number(if b.is_sign_negative() && exp % 2 == 1 {
                -result
            } else {
                result
            });
        }
        Number(b.powf(e))
    }

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

impl std::ops::Add for Number {
    type Output = Number;
    fn add(self, rhs: Number) -> Number {
        Number(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Number {
    type Output = Number;
    fn sub(self, rhs: Number) -> Number {
        Number(self.0 - rhs.0)
    }
}

impl std::ops::Mul for Number {
    type Output = Number;
    fn mul(self, rhs: Number) -> Number {
        Number(self.0 * rhs.0)
    }
}

impl std::ops::Div for Number {
    type Output = Number;
    fn div(self, rhs: Number) -> Number {
        Number(self.0 / rhs.0)
    }
}

impl std::ops::Neg for Number {
    type Output = Number;
    fn neg(self) -> Number {
        Number(-self.0)
    }
}

impl From<i32> for Number {
    fn from(v: i32) -> Number {
        Number(v as f64)
    }
}

impl From<i64> for Number {
    fn from(v: i64) -> Number {
        Number(v as f64)
    }
}

impl From<f64> for Number {
    fn from(v: f64) -> Number {
        Number(v)
    }
}

fn is_non_finite(x: f64) -> bool {
    x.is_nan() || x.is_infinite()
}

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

fn big_mul(a: &[u32], b: &[u32]) -> Vec<u32> {
    if a.iter().all(|&x| x == 0) || b.iter().all(|&x| x == 0) {
        return vec![0];
    }
    let mut result = vec![0u32; a.len() + b.len()];
    for i in 0..a.len() {
        let mut carry: u64 = 0;
        for j in 0..b.len() {
            let prod = a[i] as u64 * b[j] as u64 + result[i + j] as u64 + carry;
            result[i + j] = prod as u32;
            carry = prod >> 32;
        }
        let mut k = i + b.len();
        while carry > 0 {
            let sum = result[k] as u64 + carry;
            result[k] = sum as u32;
            carry = sum >> 32;
            k += 1;
        }
    }
    result
}

fn big_to_f64(limbs: &[u32]) -> f64 {
    let n = limbs.iter().rposition(|&x| x != 0).map_or(0, |i| i + 1);
    if n == 0 {
        return 0.0;
    }
    let limbs = &limbs[..n];

    let top = (n - 1) * 32 + (31 - limbs[n - 1].leading_zeros() as usize);

    let mut mantissa: u64 = 0;
    for i in 0..52 {
        if i >= top {
            break;
        }
        let pos = top - 1 - i;
        mantissa |= (((limbs[pos / 32] >> (pos % 32)) & 1) as u64) << (51 - i);
    }

    let mut exponent = top as u64 + 1023;

    if top >= 53 {
        let round_pos = top - 53;
        let round_bit = (limbs[round_pos / 32] >> (round_pos % 32)) & 1;

        let sticky = if round_pos == 0 {
            false
        } else {
            let limb = round_pos / 32;
            let bit = round_pos % 32;
            let mut found = bit > 0 && limbs[limb] & ((1u32 << bit) - 1) != 0;
            if !found {
                found = limbs[..limb].iter().any(|&x| x != 0);
            }
            found
        };

        if round_bit != 0 && (sticky || mantissa & 1 == 1) {
            mantissa += 1;
            if mantissa >= (1u64 << 52) {
                mantissa = 0;
                exponent += 1;
            }
        }
    }

    if exponent >= 2047 {
        return f64::INFINITY;
    }

    f64::from_bits(exponent << 52 | mantissa)
}

fn pow_exact_f64(base: u64, exp: u32) -> f64 {
    if exp == 0 {
        return 1.0;
    }
    if base == 0 {
        return 0.0;
    }

    let mut b = vec![base as u32, (base >> 32) as u32];
    while b.len() > 1 && *b.last().unwrap() == 0 {
        b.pop();
    }

    let mut result: Vec<u32> = vec![1];
    let mut e = exp;

    while e > 0 {
        if e & 1 == 1 {
            result = big_mul(&result, &b);
        }
        e >>= 1;
        if e > 0 {
            b = big_mul(&b, &b);
        }
    }

    big_to_f64(&result)
}

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

#[cfg(test)]
mod tests {
    use super::*;

    const MAX_MANTISSA: u64 = (1 << 53) - 1;

    fn num_from_bits(b: u64) -> Number {
        Number(f64::from_bits(b))
    }

    fn ieee_parts_2_double(sign: bool, ieee_exponent: u32, ieee_mantissa: u64) -> Number {
        let sign_bit: u64 = if sign { 1 } else { 0 };
        num_from_bits((sign_bit << 63) | (u64::from(ieee_exponent) << 52) | ieee_mantissa)
    }

    fn assert_equal_number(got: Number, want: Number) {
        if got.is_nan() || want.is_nan() {
            assert_eq!(got.is_nan(), want.is_nan(), "got: {}, want: {}", got, want);
        } else {
            assert_eq!(got, want, "got: {}, want: {}", got, want);
        }
    }

    fn string_tests() -> Vec<(Number, &'static str)> {
        vec![

            (Number::nan(), "NaN"),
            (Number::inf(1), "Infinity"),
            (Number::inf(-1), "-Infinity"),
            (Number(0.0), "0"),
            (Number(-0.0), "0"),
            (Number(1.0), "1"),
            (Number(-1.0), "-1"),
            (Number(0.3), "0.3"),
            (Number(-0.3), "-0.3"),
            (Number(1.5), "1.5"),
            (Number(-1.5), "-1.5"),
            (Number(1e308), "1e+308"),
            (Number(-1e308), "-1e+308"),
            (Number(std::f64::consts::PI), "3.141592653589793"),
            (Number(-std::f64::consts::PI), "-3.141592653589793"),
            (MAX_SAFE_INTEGER, "9007199254740991"),
            (MIN_SAFE_INTEGER, "-9007199254740991"),
            (num_from_bits(0x000FFFFFFFFFFFFF), "2.225073858507201e-308"),
            (num_from_bits(0x0010000000000000), "2.2250738585072014e-308"),
            (Number(1234567.8), "1234567.8"),
            (Number(123.456), "123.456"),
            (Number(-123.456), "-123.456"),
            (Number(444123.0), "444123"),
            (Number(-444123.0), "-444123"),
            (Number(444123.789123456789875436), "444123.7891234568"),
            (Number(-444123.78963636363636363636), "-444123.7896363636"),
            (Number(1e21), "1e+21"),

            (Number(2.2250738585072014e-308), "2.2250738585072014e-308"),
            (num_from_bits(0x7fefffffffffffff), "1.7976931348623157e+308"),
            (num_from_bits(1), "5e-324"),
            (Number(2.98023223876953125e-8), "2.9802322387695312e-8"),
            (Number(4.940656e-318), "4.940656e-318"),
            (Number(1.18575755e-316), "1.18575755e-316"),
            (Number(2.989102097996e-312), "2.989102097996e-312"),
            (Number(1.2345678), "1.2345678"),
            (num_from_bits(0x4830F0CF064DD592), "5.764607523034235e+39"),
            (num_from_bits(0x4840F0CF064DD592), "1.152921504606847e+40"),
            (num_from_bits(0x4850F0CF064DD592), "2.305843009213694e+40"),
            (Number(1.2), "1.2"),
            (Number(1.23), "1.23"),
            (Number(1.234), "1.234"),
            (Number(1.2345), "1.2345"),
            (Number(1.23456), "1.23456"),
            (Number(1.234567), "1.234567"),
            (Number(1.2345678), "1.2345678"),
            (Number(1.23456789), "1.23456789"),
            (Number(1.234567895), "1.234567895"),
            (Number(1.2345678901), "1.2345678901"),
            (Number(1.23456789012), "1.23456789012"),
            (Number(1.234567890123), "1.234567890123"),
            (Number(1.2345678901234), "1.2345678901234"),
            (Number(1.23456789012345), "1.23456789012345"),
            (Number(1.234567890123456), "1.234567890123456"),
            (Number(1.2345678901234567), "1.2345678901234567"),
            (Number(4.294967294), "4.294967294"),
            (Number(4.294967295), "4.294967295"),
            (Number(4.294967296), "4.294967296"),
            (Number(4.294967297), "4.294967297"),
            (Number(4.294967298), "4.294967298"),
            (ieee_parts_2_double(false, 4, 0), "1.7800590868057611e-307"),
            (
                ieee_parts_2_double(false, 6, MAX_MANTISSA),
                "2.8480945388892175e-306",
            ),
            (ieee_parts_2_double(false, 41, 0), "2.446494580089078e-296"),
            (
                ieee_parts_2_double(false, 40, MAX_MANTISSA),
                "4.8929891601781557e-296",
            ),
            (ieee_parts_2_double(false, 307, 0), "2.900835519859558e-216"),
            (
                ieee_parts_2_double(false, 306, MAX_MANTISSA),
                "5.801671039719115e-216",
            ),
            (
                ieee_parts_2_double(false, 934, 0x000FA7161A4D6E0C),
                "3.196104012172126e-27",
            ),
            (MAX_SAFE_INTEGER, "9007199254740991"),
            (Number(1.0), "1"),
            (Number(12.0), "12"),
            (Number(123.0), "123"),
            (Number(1234.0), "1234"),
            (Number(12345.0), "12345"),
            (Number(123456.0), "123456"),
            (Number(1234567.0), "1234567"),
            (Number(12345678.0), "12345678"),
            (Number(123456789.0), "123456789"),
            (Number(1234567890.0), "1234567890"),
            (Number(1234567895.0), "1234567895"),
            (Number(12345678901.0), "12345678901"),
            (Number(123456789012.0), "123456789012"),
            (Number(1234567890123.0), "1234567890123"),
            (Number(12345678901234.0), "12345678901234"),
            (Number(123456789012345.0), "123456789012345"),
            (Number(1234567890123456.0), "1234567890123456"),
            (Number(1.0), "1"),
            (Number(10.0), "10"),
            (Number(100.0), "100"),
            (Number(1000.0), "1000"),
            (Number(10000.0), "10000"),
            (Number(100000.0), "100000"),
            (Number(1000000.0), "1000000"),
            (Number(10000000.0), "10000000"),
            (Number(100000000.0), "100000000"),
            (Number(1000000000.0), "1000000000"),
            (Number(10000000000.0), "10000000000"),
            (Number(100000000000.0), "100000000000"),
            (Number(1000000000000.0), "1000000000000"),
            (Number(10000000000000.0), "10000000000000"),
            (Number(100000000000000.0), "100000000000000"),
            (Number(1000000000000000.0), "1000000000000000"),
            (Number(1000000000000001.0), "1000000000000001"),
            (Number(1000000000000010.0), "1000000000000010"),
            (Number(1000000000000100.0), "1000000000000100"),
            (Number(1000000000001000.0), "1000000000001000"),
            (Number(1000000000010000.0), "1000000000010000"),
            (Number(1000000000100000.0), "1000000000100000"),
            (Number(1000000001000000.0), "1000000001000000"),
            (Number(1000000010000000.0), "1000000010000000"),
            (Number(1000000100000000.0), "1000000100000000"),
            (Number(1000001000000000.0), "1000001000000000"),
            (Number(1000010000000000.0), "1000010000000000"),
            (Number(1000100000000000.0), "1000100000000000"),
            (Number(1001000000000000.0), "1001000000000000"),
            (Number(1010000000000000.0), "1010000000000000"),
            (Number(1100000000000000.0), "1100000000000000"),
            (Number(8.0), "8"),
            (Number(64.0), "64"),
            (Number(512.0), "512"),
            (Number(8192.0), "8192"),
            (Number(65536.0), "65536"),
            (Number(524288.0), "524288"),
            (Number(8388608.0), "8388608"),
            (Number(67108864.0), "67108864"),
            (Number(536870912.0), "536870912"),
            (Number(8589934592.0), "8589934592"),
            (Number(68719476736.0), "68719476736"),
            (Number(549755813888.0), "549755813888"),
            (Number(8796093022208.0), "8796093022208"),
            (Number(70368744177664.0), "70368744177664"),
            (Number(562949953421312.0), "562949953421312"),
            (Number(8000.0), "8000"),
            (Number(64000.0), "64000"),
            (Number(512000.0), "512000"),
            (Number(8192000.0), "8192000"),
            (Number(65536000.0), "65536000"),
            (Number(524288000.0), "524288000"),
            (Number(8388608000.0), "8388608000"),
            (Number(67108864000.0), "67108864000"),
            (Number(536870912000.0), "536870912000"),
            (Number(8589934592000.0), "8589934592000"),
            (Number(68719476736000.0), "68719476736000"),
            (Number(549755813888000.0), "549755813888000"),
            (Number(8796093022208000.0), "8796093022208000"),
        ]
    }

    fn string_tests_display_divergent() -> Vec<(Number, &'static str)> {
        vec![
            (Number(19686109595169230000.0), "19686109595169230000"),
            (Number(1e20), "100000000000000000000"),
            (Number(-21098088986959630.0), "-21098088986959630"),
            (Number(9060801153433600.0), "9060801153433600"),
            (Number(4708356024711512000.0), "4708356024711512000"),
            (Number(9409340012568248000.0), "9409340012568248000"),
            (ieee_parts_2_double(false, 1077, 0), "18014398509481984"),
            (
                ieee_parts_2_double(false, 1076, MAX_MANTISSA),
                "36028797018963964",
            ),
            (Number(9007199254740992.0), "9007199254740992"),
        ]
    }

    fn from_string_tests() -> Vec<(Number, &'static str)> {
        vec![
            (Number::nan(), "    NaN"),
            (Number::inf(1), "Infinity    "),
            (Number::inf(-1), "    -Infinity"),
            (Number(1.0), "1."),
            (Number(1.0), "1.0   "),
            (Number(1.0), "+1"),
            (Number(1.0), "+1."),
            (Number(1.0), "+1.0"),
            (Number::nan(), "whoops"),
            (Number(0.0), ""),
            (Number(0.0), "0"),
            (Number(0.0), "0."),
            (Number(0.0), "0.0"),
            (Number(0.0), "0.0000"),
            (Number(0.0), ".0000"),
            (Number(-0.0), "-0"),
            (Number(-0.0), "-0."),
            (Number(-0.0), "-0.0"),
            (Number(-0.0), "-.0"),
            (Number::nan(), "."),
            (Number::nan(), "e"),
            (Number::nan(), ".e"),
            (Number::nan(), "+"),
            (Number(0.0), "0X0"),
            (Number::nan(), "e0"),
            (Number::nan(), "E0"),
            (Number::nan(), "1e"),
            (Number::nan(), "1e+"),
            (Number::nan(), "1e-"),
            (Number(1.0), "1e+0"),
            (Number::nan(), "++0"),
            (Number::nan(), "0_0"),
            (Number::inf(1), "1e1000"),
            (Number::inf(-1), "-1e1000"),
            (Number(0.0), ".0e0"),
            (Number::nan(), "0e++0"),
            (Number(10.0), "0XA"),
            (Number(0b1010_i64 as f64), "0b1010"),
            (Number(0b1010_i64 as f64), "0B1010"),
            (Number(0o12_i64 as f64), "0o12"),
            (Number(0o12_i64 as f64), "0O12"),
            (Number(0x123456789abcdef0_i64 as f64), "0x123456789abcdef0"),
            (Number(0x123456789abcdef0_i64 as f64), "0X123456789ABCDEF0"),

            (Number::nan(), "0B0.0"),
            (
                Number(1.231235345083403e91),
                "12312353450834030486384068034683603046834603806830644850340602384608368034634603680348603864",
            ),
            (
                Number::nan(),
                "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX8OOOOOOOOOOOOOOOOOOO",
            ),
            (Number::inf(1), "+Infinity"),
            (Number(1234.56), "  \t1234.56  "),
            (Number::nan(), "\u{200b}"),
            (Number(0.0), " "),
            (Number(0.0), "\n"),
            (Number(0.0), "\r"),
            (Number(0.0), "\r\n"),
            (Number(0.0), "\u{2028}"),
            (Number(0.0), "\u{2029}"),
            (Number(0.0), "\t"),
            (Number(0.0), "\u{0B}"),
            (Number(0.0), "\u{0C}"),
            (Number(0.0), "\u{FEFF}"),
            (Number(0.0), "\u{00A0}"),
            (Number(1e19), "010000000000000000000"),
            (Number::nan(), "0x1.fffffffffffffp1023"),
            (Number::nan(), "0X_1FFFP-16"),
            (Number::nan(), "1_000"),
            (Number(0.0), "0x0"),
            (Number(0.0), "0X0"),
            (Number::nan(), "0xOOPS"),
            (Number(0xABCDEF_i64 as f64), "0xABCDEF"),
            (Number(0xABCDEF_i64 as f64), "0xABCDEF"),
            (Number(0.0), "0o0"),
            (Number(0.0), "0O0"),
            (Number::nan(), "0o8"),
            (Number::nan(), "0O8"),
            (Number(0o12345_i64 as f64), "0o12345"),
            (Number(0o12345_i64 as f64), "0O12345"),
            (Number(0.0), "0b0"),
            (Number(0.0), "0B0"),
            (Number::nan(), "0b2"),
            (Number::nan(), "0b2"),
            (Number(0b10101_i64 as f64), "0b10101"),
            (Number(0b10101_i64 as f64), "0B10101"),
            (Number::nan(), "1.f"),
            (Number::nan(), "1.e"),
            (Number::nan(), "1.0ef"),
            (Number::nan(), "1.0e"),
            (Number::nan(), ".f"),
            (Number::nan(), ".e"),
            (Number::nan(), ".0ef"),
            (Number::nan(), ".0e"),
            (Number::nan(), "a.f"),
            (Number::nan(), "a.e"),
            (Number::nan(), "a.0ef"),
            (Number::nan(), "a.0e"),
        ]
    }

    #[test]
    fn test_parse_pseudo_bigint() {

        let mut test_numbers: Vec<Number> = Vec::new();
        for i in 0..1000_i64 {
            test_numbers.push(Number(i as f64));
        }
        for bits in 0..53_i32 {
            let p = 1i64 << bits;
            test_numbers.push(Number(p as f64));
            test_numbers.push(Number((p - 1) as f64));
        }
        for test_number in &test_numbers {
            let s = test_number.to_string();
            for leading_zeros in 0..10 {
                let lit = format!("{}{}n", "0".repeat(leading_zeros), s);
                assert_eq!(
                    PseudoBigInt::parse(&lit).to_string(),
                    s,
                    "literal: {:?}",
                    lit
                );
            }
        }

        let non_decimal: &[(&str, &str)] = &[
            ("0b0n", "0"),
            ("0b1n", "1"),
            ("0b1010n", "10"),
            ("0B1101n", "13"),
            ("0o0n", "0"),
            ("0o7n", "7"),
            ("0o755n", "493"),
            ("0O12n", "10"),
            ("0x0n", "0"),
            ("0xFn", "15"),
            ("0xFFn", "255"),
            ("0X1Fn", "31"),
        ];
        for (lit, out) in non_decimal {
            assert_eq!(
                PseudoBigInt::parse(lit).to_string(),
                *out,
                "literal: {:?}",
                lit
            );
        }

        assert_eq!(
            PseudoBigInt::parse("123456789012345678901234567890n").to_string(),
            "123456789012345678901234567890"
        );
        assert_eq!(
            PseudoBigInt::parse("0b1100011101110100100001111111101101100001101110011111000001110111001001110001111110000101011010010n").to_string(),
            "123456789012345678901234567890"
        );
        assert_eq!(
            PseudoBigInt::parse("0o143564417755415637016711617605322n").to_string(),
            "123456789012345678901234567890"
        );
        assert_eq!(
            PseudoBigInt::parse("0x18ee90ff6c373e0ee4e3f0ad2n").to_string(),
            "123456789012345678901234567890"
        );
    }

    #[test]
    fn test_parse_pseudo_bigint_underscores() {
        let cases: &[(&str, &str)] = &[
            ("0b1010_0101n", "165"),
            ("0o7_5_5n", "493"),
            ("0xF_Fn", "255"),
        ];
        for (lit, out) in cases {
            assert_eq!(
                PseudoBigInt::parse(lit).to_string(),
                *out,
                "literal: {:?}",
                lit
            );
        }
    }

    #[test]
    fn test_to_int32() {
        let cases: &[(Number, i32)] = &[
            (Number(0.0), 0),
            (Number(-0.0), 0),
            (Number::nan(), 0),
            (Number::inf(1), 0),
            (Number::inf(-1), 0),
            (Number(i32::MAX as f64), i32::MAX),
            (Number((i32::MAX as i64 + 1) as f64), i32::MIN),
            (Number(i32::MIN as f64), i32::MIN),
            (Number((i32::MIN as i64 - 1) as f64), i32::MAX),
            (MIN_SAFE_INTEGER, 1),
            (MIN_SAFE_INTEGER - Number(1.0), 0),
            (MIN_SAFE_INTEGER + Number(1.0), 2),
            (MAX_SAFE_INTEGER, -1),
            (MAX_SAFE_INTEGER - Number(1.0), -2),
            (MAX_SAFE_INTEGER + Number(1.0), 0),
            (Number(-8589934590.0), 2),
            (Number(0xDEADBEEF_u32 as f64), -559038737),
            (Number(4294967808.0), 512),
            (Number(-0.4), 0),
            (Number(f64::from_bits(1)), 0),
            (Number(-f64::from_bits(1)), 0),
            (Number(f64::MAX), 0),
            (Number(-f64::MAX), 0),
            (num_from_bits(0x000FFFFFFFFFFFFF), 0),
            (num_from_bits(0x0010000000000000), 0),
            (Number(1.0), 1),
            (Number(-1.0), -1),
            (Number(1e308), 0),
            (Number(-1e308), 0),
            (Number(std::f64::consts::PI), 3),
            (Number(-std::f64::consts::PI), -3),
            (Number(std::f64::consts::E), 2),
            (Number(-std::f64::consts::E), -2),
            (Number(0.5), 0),
            (Number(-0.5), 0),
            (Number(0.49999999999999994), 0),
            (Number(-0.49999999999999994), 0),
            (Number(0.5000000000000001), 0),
            (Number(-0.5000000000000001), 0),
            (Number(2147483648.5), -2147483648),
            (Number(-2147483648.5), -2147483648),
            (Number(1099511627776.0), 0),
            (Number(-1099511627776.0), 0),
            (Number(536624127.0), 536624127),
        ];
        for (input, want) in cases {
            assert_eq!(input.to_int32(), *want, "to_int32({})", input);
        }
    }

    #[test]
    fn test_bitwise_not() {
        let cases: &[(Number, Number)] = &[
            (Number(-2147483649.0), Number(-2147483648.0)),
            (Number(2147483647.0), Number(-2147483648.0)),
            (Number(-4294967296.0), Number(-1.0)),
            (Number(0.0), Number(-1.0)),
            (Number(2147483648.0), Number(2147483647.0)),
            (Number(-2147483648.0), Number(2147483647.0)),
            (Number(4294967296.0), Number(-1.0)),
        ];
        for (x, want) in cases {
            assert_equal_number(x.bitwise_not(), *want);
        }
    }

    #[test]
    fn test_bitwise_and() {
        let cases: &[(Number, Number, Number)] = &[
            (Number(0.0), Number(0.0), Number(0.0)),
            (Number(0.0), Number(1.0), Number(0.0)),
            (Number(1.0), Number(0.0), Number(0.0)),
            (Number(1.0), Number(1.0), Number(1.0)),
        ];
        for (x, y, want) in cases {
            assert_equal_number(x.bitwise_and(*y), *want);
        }
    }

    #[test]
    fn test_bitwise_or() {
        let cases: &[(Number, Number, Number)] = &[
            (Number(0.0), Number(0.0), Number(0.0)),
            (Number(0.0), Number(1.0), Number(1.0)),
            (Number(1.0), Number(0.0), Number(1.0)),
            (Number(1.0), Number(1.0), Number(1.0)),
        ];
        for (x, y, want) in cases {
            assert_equal_number(x.bitwise_or(*y), *want);
        }
    }

    #[test]
    fn test_bitwise_xor() {
        let cases: &[(Number, Number, Number)] = &[
            (Number(0.0), Number(0.0), Number(0.0)),
            (Number(0.0), Number(1.0), Number(1.0)),
            (Number(1.0), Number(0.0), Number(1.0)),
            (Number(1.0), Number(1.0), Number(0.0)),
        ];
        for (x, y, want) in cases {
            assert_equal_number(x.bitwise_xor(*y), *want);
        }
    }

    #[test]
    fn test_signed_right_shift() {
        let cases: &[(Number, Number, Number)] = &[
            (Number(1.0), Number(0.0), Number(1.0)),
            (Number(1.0), Number(1.0), Number(0.0)),
            (Number(1.0), Number(2.0), Number(0.0)),
            (Number(1.0), Number(31.0), Number(0.0)),
            (Number(1.0), Number(32.0), Number(1.0)),
            (Number(-4.0), Number(0.0), Number(-4.0)),
            (Number(-4.0), Number(1.0), Number(-2.0)),
            (Number(-4.0), Number(2.0), Number(-1.0)),
            (Number(-4.0), Number(3.0), Number(-1.0)),
            (Number(-4.0), Number(4.0), Number(-1.0)),
            (Number(-4.0), Number(31.0), Number(-1.0)),
            (Number(-4.0), Number(32.0), Number(-4.0)),
            (Number(-4.0), Number(33.0), Number(-2.0)),
        ];
        for (x, y, want) in cases {
            assert_equal_number(x.signed_right_shift(*y), *want);
        }
    }

    #[test]
    fn test_unsigned_right_shift() {
        let cases: &[(Number, Number, Number)] = &[
            (Number(1.0), Number(0.0), Number(1.0)),
            (Number(1.0), Number(1.0), Number(0.0)),
            (Number(1.0), Number(2.0), Number(0.0)),
            (Number(1.0), Number(31.0), Number(0.0)),
            (Number(1.0), Number(32.0), Number(1.0)),
            (Number(-4.0), Number(0.0), Number(4294967292.0)),
            (Number(-4.0), Number(1.0), Number(2147483646.0)),
            (Number(-4.0), Number(2.0), Number(1073741823.0)),
            (Number(-4.0), Number(3.0), Number(536870911.0)),
            (Number(-4.0), Number(4.0), Number(268435455.0)),
            (Number(-4.0), Number(31.0), Number(1.0)),
            (Number(-4.0), Number(32.0), Number(4294967292.0)),
            (Number(-4.0), Number(33.0), Number(2147483646.0)),
        ];
        for (x, y, want) in cases {
            assert_equal_number(x.unsigned_right_shift(*y), *want);
        }
    }

    #[test]
    fn test_left_shift() {
        let cases: &[(Number, Number, Number)] = &[
            (Number(1.0), Number(0.0), Number(1.0)),
            (Number(1.0), Number(1.0), Number(2.0)),
            (Number(1.0), Number(2.0), Number(4.0)),
            (Number(1.0), Number(31.0), Number(-2147483648.0)),
            (Number(1.0), Number(32.0), Number(1.0)),
            (Number(-4.0), Number(0.0), Number(-4.0)),
            (Number(-4.0), Number(1.0), Number(-8.0)),
            (Number(-4.0), Number(2.0), Number(-16.0)),
            (Number(-4.0), Number(3.0), Number(-32.0)),
            (Number(-4.0), Number(31.0), Number(0.0)),
            (Number(-4.0), Number(32.0), Number(-4.0)),
        ];
        for (x, y, want) in cases {
            assert_equal_number(x.left_shift(*y), *want);
        }
    }

    #[test]
    fn test_remainder() {

        let cases: &[(Number, Number, Number)] = &[
            (Number::nan(), Number(1.0), Number::nan()),
            (Number(1.0), Number::nan(), Number::nan()),
            (Number::inf(1), Number(1.0), Number::nan()),
            (Number::inf(-1), Number(1.0), Number::nan()),
            (Number(123.0), Number::inf(1), Number(123.0)),
            (Number(123.0), Number::inf(-1), Number(123.0)),
            (Number(123.0), Number(0.0), Number::nan()),
            (Number(123.0), Number(-0.0), Number::nan()),
            (Number(0.0), Number(123.0), Number(0.0)),
            (Number(-0.0), Number(123.0), Number(-0.0)),
            (Number(10.0), Number(3.0), Number(1.0)),
            (Number(-10.0), Number(3.0), Number(-1.0)),
            (Number(10.0), Number(-3.0), Number(1.0)),
            (Number(-10.0), Number(-3.0), Number(-1.0)),
            (Number(5.5), Number(2.0), Number(1.5)),
            (Number(-5.5), Number(2.0), Number(-1.5)),
            (Number(1.0), Number(0.5), Number(0.0)),
            (Number(-1.0), Number(0.5), Number(-0.0)),
            (Number(1.5), Number(1.0), Number(0.5)),
            (Number(-1.5), Number(1.0), Number(-0.5)),
            (Number(7.0), Number(0.1), Number(7.0 % 0.1)),
            (Number(7.0), Number(0.2), Number(7.0 % 0.2)),
            (Number(7.0), Number(0.3), Number(7.0 % 0.3)),
            (Number(100.0), Number(0.3), Number(100.0 % 0.3)),
        ];
        for (x, y, want) in cases {
            assert_equal_number(x.remainder(*y), *want);
        }
    }

    #[test]
    fn test_exponentiate() {
        let cases: &[(Number, Number, Number)] = &[
            (Number(2.0), Number(3.0), Number(8.0)),
            (Number::inf(1), Number(3.0), Number::inf(1)),
            (Number::inf(1), Number(-5.0), Number(0.0)),
            (Number::inf(-1), Number(3.0), Number::inf(-1)),
            (Number::inf(-1), Number(4.0), Number::inf(1)),
            (Number::inf(-1), Number(-3.0), Number(-0.0)),
            (Number::inf(-1), Number(-4.0), Number(0.0)),
            (Number(0.0), Number(3.0), Number(0.0)),
            (Number(0.0), Number(-10.0), Number::inf(1)),
            (Number(-0.0), Number(3.0), Number(-0.0)),
            (Number(-0.0), Number(4.0), Number(0.0)),
            (Number(-0.0), Number(-3.0), Number::inf(-1)),
            (Number(-0.0), Number(-4.0), Number::inf(1)),
            (Number(3.0), Number::inf(1), Number::inf(1)),
            (Number(-3.0), Number::inf(1), Number::inf(1)),
            (Number(3.0), Number::inf(-1), Number(0.0)),
            (Number(-3.0), Number::inf(-1), Number(0.0)),
            (Number::nan(), Number(3.0), Number::nan()),
            (Number(1.0), Number::inf(1), Number::nan()),
            (Number(1.0), Number::inf(-1), Number::nan()),
            (Number(-1.0), Number::inf(1), Number::nan()),
            (Number(-1.0), Number::inf(-1), Number::nan()),
            (Number(1.0), Number::nan(), Number::nan()),

            (
                Number(10.0),
                Number(308.0),
                num_from_bits(0x7fe1ccf385ebc8a0),
            ),
            (
                Number(10.0),
                Number(200.0),
                num_from_bits(0x6974e718d7d7625a),
            ),
        ];
        for (x, y, want) in cases {
            assert_equal_number(x.exponentiate(*y), *want);
        }
    }

    #[test]
    fn test_exponentiate_ulp_divergence() {
        assert_equal_number(
            Number(5.0).exponentiate(Number(210.0)),
            num_from_bits(0x5e68557f31326bbb),
        );
    }

    #[test]
    fn test_string() {
        for (number, s) in string_tests() {
            assert_eq!(number.to_string(), s, "String({})", number);
        }
    }

    #[test]
    fn test_string_display_divergent() {
        for (number, s) in string_tests_display_divergent() {
            assert_eq!(number.to_string(), s, "String({})", number);
        }
    }

    #[test]
    fn test_from_string() {

        for (number, s) in string_tests() {
            assert_equal_number(Number::from_string(s), number);
            assert_equal_number(Number::from_string(&format!("{} ", s)), number);
            assert_equal_number(Number::from_string(&format!(" {}", s)), number);
        }
        for (number, s) in string_tests_display_divergent() {
            assert_equal_number(Number::from_string(s), number);
            assert_equal_number(Number::from_string(&format!("{} ", s)), number);
            assert_equal_number(Number::from_string(&format!(" {}", s)), number);
        }

        for (number, s) in from_string_tests() {
            assert_equal_number(Number::from_string(s), number);
        }
    }

    #[test]
    fn test_from_string_hex_overflow() {
        let cases: &[(Number, &str)] = &[
            (Number(18446744073709552000.0), "0X10000000000000000"),
            (Number(18446744073709597000.0), "0X1000000000000A801"),
        ];
        for (number, s) in cases {
            assert_equal_number(Number::from_string(s), *number);
        }
    }

    #[test]
    fn test_string_roundtrip() {
        for (_, s) in string_tests() {
            assert_eq!(Number::from_string(s).to_string(), s, "roundtrip {:?}", s);
        }
    }

    #[test]
    fn test_string_js() {
        let cases: &[(Number, &str)] = &[
            (Number(0.0), "0"),

            (Number(-0.0), "0"),
            (Number(100.0), "100"),
            (Number(1.5), "1.5"),
            (Number(0.1 + 0.2), "0.30000000000000004"),

            (Number(1e20), "100000000000000000000"),

            (Number(1e21), "1e+21"),

            (Number(1e-7), "1e-7"),

            (Number(5e-324), "5e-324"),
            (Number(f64::NAN), "NaN"),
            (Number(f64::INFINITY), "Infinity"),
            (Number(f64::NEG_INFINITY), "-Infinity"),
        ];
        for (number, expected) in cases {
            assert_eq!(
                number.to_string(),
                *expected,
                "Number({}).to_string() should be {expected:?}",
                number.0
            );
        }

        for (number, s) in string_tests() {
            assert_eq!(number.to_string(), s, "stringTests roundtrip {s:?}");
        }
    }
}
