pub(crate) use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tristate {
    #[default]
    Unknown,
    False,
    True,
}

impl Tristate {
    pub fn is_true(self) -> bool { crate::fntrace::enter("is_true"); 
        matches!(self, Tristate::True)
    }

    pub fn is_true_or_unknown(self) -> bool { crate::fntrace::enter("is_true_or_unknown"); 
        !matches!(self, Tristate::False)
    }

    pub fn is_false(self) -> bool { crate::fntrace::enter("is_false"); 
        matches!(self, Tristate::False)
    }

    pub fn is_false_or_unknown(self) -> bool { crate::fntrace::enter("is_false_or_unknown"); 
        !matches!(self, Tristate::True)
    }

    pub fn is_unknown(self) -> bool { crate::fntrace::enter("is_unknown"); 
        matches!(self, Tristate::Unknown)
    }

    pub fn default_if_unknown(self, value: Tristate) -> Tristate { crate::fntrace::enter("default_if_unknown"); 
        if self.is_unknown() { value } else { self }
    }
}

impl From<bool> for Tristate {
    fn from(b: bool) -> Self { crate::fntrace::enter("from"); 
        if b { Tristate::True } else { Tristate::False }
    }
}

impl Serialize for Tristate {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { crate::fntrace::enter("serialize"); 
        match self {
            Tristate::True => serializer.serialize_bool(true),
            Tristate::False => serializer.serialize_bool(false),
            Tristate::Unknown => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for Tristate {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { crate::fntrace::enter("deserialize"); 
        match Option::<bool>::deserialize(deserializer)? {
            Some(true) => Ok(Tristate::True),
            Some(false) => Ok(Tristate::False),
            None => Ok(Tristate::Unknown),
        }
    }
}

pub fn bool_to_tristate(b: bool) -> Tristate { crate::fntrace::enter("bool_to_tristate"); 
    Tristate::from(b)
}
