use super::number::Number;

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

impl Number {
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
}
