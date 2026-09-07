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
