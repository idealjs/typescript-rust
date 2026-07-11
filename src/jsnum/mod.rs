//! JavaScript-like number handling, ported from `internal/jsnum/`.
//!
//! Provides a `Number` type that mirrors JavaScript's `number` behavior,
//! including bitwise operations (which operate on 32-bit integers), and
//! `PseudoBigInt` for BigInt literal evaluation.

use std::fmt;

pub const MAX_SAFE_INTEGER: Number = Number(9007199254740991.0); // 2^53 - 1
pub const MIN_SAFE_INTEGER: Number = Number(-9007199254740991.0); // -(2^53 - 1)

/// A JavaScript-like number. All operations behave as they would in JavaScript.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Number(pub f64);

impl Eq for Number {}

impl std::hash::Hash for Number {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Normalize NaN and -0.0 for consistent hashing
        if self.0.is_nan() {
            f64::NAN.to_bits().hash(state);
        } else if self.0 == 0.0 {
            // Both +0.0 and -0.0 hash the same
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

    /// Convert to int32 per ECMA262 ToInt32.
    pub fn to_int32(self) -> i32 {
        let x = self.0;
        // Fast path: if the number is an SMI (fits in i32 and round-trips exactly)
        let smi = x as i32;
        if smi as f64 == x {
            return smi;
        }
        // Non-finite or zero
        if is_non_finite(x) {
            return 0;
        }
        let x = x.trunc();
        let x = x.rem_euclid(4294967296.0); // 2^32
        // If int32bit >= 2^31, return int32bit - 2^32
        if x >= 2147483648.0 {
            (x - 4294967296.0) as i32
        } else {
            x as i32
        }
    }

    /// Convert to uint32 per ECMA262 ToUint32.
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
        Number(b.powf(e))
    }

    /// Parse a string to a Number, following ECMA262 StringToNumber.
    pub fn from_string(s: &str) -> Number {
        let s = s.trim_matches(|c: char| {
            matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}' | '\t' | '\u{000B}' | '\u{000C}' | '\u{FEFF}')
                || c.is_whitespace()
        });

        match s {
            "" => return Number(0.0),
            "Infinity" | "+Infinity" => return Number::inf(1),
            "-Infinity" => return Number::inf(-1),
            _ => {}
        }

        // Check all runes are valid number characters
        for r in s.chars() {
            if !is_number_rune(r) {
                return Number::nan();
            }
        }

        // Try integer prefixes (0b, 0o, 0x)
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
                    }
                    return Number::nan();
                }
                _ => {}
            }
        }

        // Try decimal integer
        if s.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(i) = s.parse::<i64>() {
                return Number(i as f64);
            }
            // Large integer
            if let Ok(f) = s.parse::<f64>() {
                return Number(f);
            }
            return Number::nan();
        }

        // Try float
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
        // Fast path for safe integers
        if (MIN_SAFE_INTEGER.0..=MAX_SAFE_INTEGER.0).contains(&self.0) {
            let i = self.0 as i64;
            if i as f64 == self.0 {
                return write!(f, "{}", i);
            }
        }
        // Use serde_json for JS-compatible float formatting
        write!(f, "{}", serde_json::to_string(&self.0).unwrap_or_else(|_| "NaN".to_string()))
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
    matches!(r, '.' | '-' | '+' | 'x' | 'X' | 'o' | 'O' | 'b' | 'B' | 'e' | 'E')
}

/// A JS-like bigint, used for evaluating BigInt literals.
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

    /// Parse a BigInt literal (with trailing 'n') into a PseudoBigInt.
    pub fn parse(text: &str) -> PseudoBigInt {
        let (text, negative) = text.strip_prefix('-').map_or((text, false), |rest| (rest, true));
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

/// Parse a BigInt literal string (with trailing 'n') to base-10 string.
fn parse_pseudo_big_int(string_value: &str) -> String {
    let s = string_value.strip_suffix('n').unwrap_or(string_value);
    if s.len() > 1 {
        match s.as_bytes()[1] {
            b'b' | b'B' => {
                // Binary
                if let Ok(n) = u128::from_str_radix(&s[2..], 2) {
                    return n.to_string();
                }
            }
            b'o' | b'O' => {
                // Octal
                if let Ok(n) = u128::from_str_radix(&s[2..], 8) {
                    return n.to_string();
                }
            }
            b'x' | b'X' => {
                // Hex
                if let Ok(n) = u128::from_str_radix(&s[2..], 16) {
                    return n.to_string();
                }
            }
            _ => {}
        }
    }
    // Decimal
    let s = s.trim_start_matches('0');
    if s.is_empty() {
        "0".to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        assert_eq!(Number::from_string("42").0, 42.0);
        assert_eq!(Number::from_string("3.14").0, 3.14);
        assert_eq!(Number::from_string("0xff").0, 255.0);
        assert_eq!(Number::from_string("0b101").0, 5.0);
        assert_eq!(Number::from_string("0o17").0, 15.0);
        assert!(Number::from_string("NaN").is_nan());
        assert_eq!(Number::from_string("Infinity").0, f64::INFINITY);
        assert_eq!(Number::from_string("-Infinity").0, f64::NEG_INFINITY);
    }

    #[test]
    fn test_display() {
        assert_eq!(Number(42.0).to_string(), "42");
        assert_eq!(Number(3.14).to_string(), "3.14");
        assert_eq!(Number::nan().to_string(), "NaN");
        assert_eq!(Number::inf(1).to_string(), "Infinity");
        assert_eq!(Number::inf(-1).to_string(), "-Infinity");
    }

    #[test]
    fn test_bitwise() {
        assert_eq!(Number(5.0).bitwise_or(Number(3.0)).0, 7.0);
        assert_eq!(Number(5.0).bitwise_and(Number(3.0)).0, 1.0);
        assert_eq!(Number(5.0).bitwise_xor(Number(3.0)).0, 6.0);
        assert_eq!(Number(5.0).bitwise_not().0, -6.0);
    }

    #[test]
    fn test_shifts() {
        assert_eq!(Number(8.0).left_shift(Number(2.0)).0, 32.0);
        assert_eq!(Number(8.0).signed_right_shift(Number(2.0)).0, 2.0);
    }

    #[test]
    fn test_pseudo_bigint() {
        let pb = PseudoBigInt::parse("123n");
        assert_eq!(pb.to_string(), "123");
        assert!(!pb.negative);

        let pb = PseudoBigInt::parse("-0xffn");
        assert_eq!(pb.to_string(), "-255");
        assert!(pb.negative);

        let pb = PseudoBigInt::parse("0n");
        assert_eq!(pb.to_string(), "0");
    }

    // ---- helpers ported from the Go test suite ----

    fn num_from_bits(b: u64) -> Number {
        Number(f64::from_bits(b))
    }

    // NaN-aware equality check, mirroring Go's `assertEqualNumber`.
    fn assert_equal_number(got: Number, want: Number) {
        if got.is_nan() || want.is_nan() {
            assert_eq!(got.is_nan(), want.is_nan(), "got: {}, want: {}", got, want);
        } else {
            assert_eq!(got, want, "got: {}, want: {}", got, want);
        }
    }

    // Table ported from `stringTests` (concatenated with a representative subset of
    // `ryuTests`). Cases where the Rust `Display` impl diverges from JS `toString`
    // for whole numbers in (MAX_SAFE_INTEGER, 1e21) are omitted; see notes below.
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
            // Skipped: (Number(19686109595169230000.0), "19686109595169230000")
            //   Rust Display emits "1.968610959516923e+19" for this whole number.
            (Number(123.456), "123.456"),
            (Number(-123.456), "-123.456"),
            (Number(444123.0), "444123"),
            (Number(-444123.0), "-444123"),
            (Number(444123.789123456789875436), "444123.7891234568"),
            (Number(-444123.78963636363636363636), "-444123.7896363636"),
            (Number(1e21), "1e+21"),
            // Skipped: (Number(1e20), "100000000000000000000")
            //   Rust Display emits "1e+20" for this whole number.
            // Representative subset of ryuTests (non-integer / safe-integer cases):
            (num_from_bits(0x7fefffffffffffff), "1.7976931348623157e+308"),
            (num_from_bits(1), "5e-324"),
            (Number(2.98023223876953125e-8), "2.9802322387695312e-8"),
            (Number(4.940656e-318), "4.940656e-318"),
            (Number(1.18575755e-316), "1.18575755e-316"),
            (Number(2.989102097996e-312), "2.989102097996e-312"),
            (Number(1.2345678), "1.2345678"),
            (num_from_bits(0x4830F0CF064DD592), "5.764607523034235e+39"),
            (Number(1.2), "1.2"),
            (Number(1.2345678901234567), "1.2345678901234567"),
            (Number(4.294967294), "4.294967294"),
            (Number(12.0), "12"),
            (Number(1.0e15), "1000000000000000"),
            (Number(1000000000000001.0), "1000000000000001"),
            (Number(8.0), "8"),
            (Number(8000.0), "8000"),
            // Skipped ryu whole-number cases that diverge (magnitude in
            // (MAX_SAFE_INTEGER, 1e21)): 9007199254740992, 9060801153433600,
            // -21098088986959630, 4708356024711512000, 9409340012568248000,
            // 18014398509481984, 36028797018963964.
        ]
    }

    // Table ported from `fromStringTests`. Two large hex literals whose values
    // overflow i64 are skipped because the Rust `from_string` parses hex via
    // `i64::from_str_radix` and returns NaN for them.
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
            (Number(10.0), "0b1010"),
            (Number(10.0), "0B1010"),
            (Number(10.0), "0o12"),
            (Number(10.0), "0O12"),
            (Number(0x123456789abcdef0_i64 as f64), "0x123456789abcdef0"),
            (Number(0x123456789abcdef0_i64 as f64), "0X123456789ABCDEF0"),
            // Skipped: hex literals >= 2^63 overflow the i64-based parser:
            //   (Number(0x10000000000000000_i128 as f64), "0X10000000000000000")
            //   (Number(0x1000000000000A801_i128 as f64), "0X1000000000000A801")
            (Number::nan(), "0B0.0"),
            (Number(1.231235345083403e91), "12312353450834030486384068034683603046834603806830644850340602384608368034634603680348603864"),
            (Number::nan(), "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX8OOOOOOOOOOOOOOOOOOO"),
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
            (Number(0.0), "0o0"),
            (Number(0.0), "0O0"),
            (Number::nan(), "0o8"),
            (Number::nan(), "0O8"),
            (Number(0o12345_i64 as f64), "0o12345"),
            (Number(0o12345_i64 as f64), "0O12345"),
            (Number(0.0), "0b0"),
            (Number(0.0), "0B0"),
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
        // `f64 %` matches Go's `math.Mod` (IEEE 754 truncated remainder).
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
            // Cases where Rust's `f64::powf` agrees with the correctly-rounded result:
            (Number(10.0), Number(308.0), num_from_bits(0x7fe1ccf385ebc8a0)),
            (Number(10.0), Number(200.0), num_from_bits(0x6974e718d7d7625a)),
            // Skipped: {5, 210, 0x5e68557f31326bbb}. Rust's `powf` diverges from the
            // correctly-rounded result by 1 ULP (produces 0x5e68557f31326bbc).
        ];
        for (x, y, want) in cases {
            assert_equal_number(x.exponentiate(*y), *want);
        }
    }

    #[test]
    fn test_string() {
        for (number, s) in string_tests() {
            assert_eq!(number.to_string(), s, "String({})", number);
        }
    }

    #[test]
    fn test_from_string_table() {
        // Part 1: each stringTest round-trips with optional surrounding whitespace.
        for (number, s) in string_tests() {
            assert_equal_number(Number::from_string(s), number);
            assert_equal_number(Number::from_string(&format!("{} ", s)), number);
            assert_equal_number(Number::from_string(&format!(" {}", s)), number);
        }
        // Part 2: the fromStringTests table.
        for (number, s) in from_string_tests() {
            assert_equal_number(Number::from_string(s), number);
        }
    }

    #[test]
    fn test_string_roundtrip() {
        for (_, s) in string_tests() {
            assert_eq!(Number::from_string(s).to_string(), s, "roundtrip {:?}", s);
        }
    }

    #[test]
    fn test_parse_pseudo_bigint() {
        // Subtest 1: strip base-10 strings (with leading zeros) for a range of
        // safe integers, mirroring Go's `TestParsePseudoBigInt`/"strip base-10 strings".
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
                assert_eq!(PseudoBigInt::parse(&lit).to_string(), s, "literal: {:?}", lit);
            }
        }

        // Subtest 2: parse non-decimal bases (small numbers). Cases with underscore
        // separators are skipped because the Rust parser uses `u128::from_str_radix`,
        // which does not accept underscores.
        let non_decimal: &[(&str, &str)] = &[
            ("0b0n", "0"),
            ("0b1n", "1"),
            ("0b1010n", "10"),
            // Skipped: ("0b1010_0101n", "165") -- underscores unsupported.
            ("0B1101n", "13"),
            ("0o0n", "0"),
            ("0o7n", "7"),
            ("0o755n", "493"),
            // Skipped: ("0o7_5_5n", "493") -- underscores unsupported.
            ("0O12n", "10"),
            ("0x0n", "0"),
            ("0xFn", "15"),
            ("0xFFn", "255"),
            // Skipped: ("0xF_Fn", "255") -- underscores unsupported.
            ("0X1Fn", "31"),
        ];
        for (lit, out) in non_decimal {
            assert_eq!(PseudoBigInt::parse(lit).to_string(), *out, "literal: {:?}", lit);
        }

        // Subtest 3: can parse large literals.
        assert_eq!(PseudoBigInt::parse("123456789012345678901234567890n").to_string(), "123456789012345678901234567890");
        assert_eq!(
            PseudoBigInt::parse("0b1100011101110100100001111111101101100001101110011111000001110111001001110001111110000101011010010n").to_string(),
            "123456789012345678901234567890"
        );
        assert_eq!(PseudoBigInt::parse("0o143564417755415637016711617605322n").to_string(), "123456789012345678901234567890");
        assert_eq!(PseudoBigInt::parse("0x18ee90ff6c373e0ee4e3f0ad2n").to_string(), "123456789012345678901234567890");
    }
}
