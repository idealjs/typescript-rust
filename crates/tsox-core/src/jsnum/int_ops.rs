use super::number::Number;

fn is_non_finite(x: f64) -> bool { crate::fntrace::enter("is_non_finite"); 
    x.is_nan() || x.is_infinite()
}

impl Number {
    pub fn to_int32(self) -> i32 { crate::fntrace::enter("to_int32"); 
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

    pub fn to_uint32(self) -> u32 { crate::fntrace::enter("to_uint32"); 
        self.to_int32() as u32
    }

    fn to_shift_count(self) -> u32 { crate::fntrace::enter("to_shift_count"); 
        self.to_uint32() & 31
    }

    pub fn signed_right_shift(self, y: Number) -> Number { crate::fntrace::enter("signed_right_shift"); 
        Number((self.to_int32() >> y.to_shift_count()) as f64)
    }

    pub fn unsigned_right_shift(self, y: Number) -> Number { crate::fntrace::enter("unsigned_right_shift"); 
        Number((self.to_uint32() >> y.to_shift_count()) as f64)
    }

    pub fn left_shift(self, y: Number) -> Number { crate::fntrace::enter("left_shift"); 
        Number((self.to_int32() << y.to_shift_count()) as f64)
    }

    pub fn bitwise_not(self) -> Number { crate::fntrace::enter("bitwise_not"); 
        Number((!self.to_int32()) as f64)
    }

    pub fn bitwise_or(self, y: Number) -> Number { crate::fntrace::enter("bitwise_or"); 
        Number((self.to_int32() | y.to_int32()) as f64)
    }

    pub fn bitwise_and(self, y: Number) -> Number { crate::fntrace::enter("bitwise_and"); 
        Number((self.to_int32() & y.to_int32()) as f64)
    }

    pub fn bitwise_xor(self, y: Number) -> Number { crate::fntrace::enter("bitwise_xor"); 
        Number((self.to_int32() ^ y.to_int32()) as f64)
    }
}
