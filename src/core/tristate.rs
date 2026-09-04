use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tristate {
    #[default]
    Unknown,
    False,
    True,
}

impl Tristate {
    pub fn is_true(self) -> bool {
        matches!(self, Tristate::True)
    }

    pub fn is_true_or_unknown(self) -> bool {
        !matches!(self, Tristate::False)
    }

    pub fn is_false(self) -> bool {
        matches!(self, Tristate::False)
    }

    pub fn is_false_or_unknown(self) -> bool {
        !matches!(self, Tristate::True)
    }

    pub fn is_unknown(self) -> bool {
        matches!(self, Tristate::Unknown)
    }

    pub fn default_if_unknown(self, value: Tristate) -> Tristate {
        if self.is_unknown() { value } else { self }
    }
}

impl From<bool> for Tristate {
    fn from(b: bool) -> Self {
        if b { Tristate::True } else { Tristate::False }
    }
}

impl Serialize for Tristate {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Tristate::True => serializer.serialize_bool(true),
            Tristate::False => serializer.serialize_bool(false),
            Tristate::Unknown => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for Tristate {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match Option::<bool>::deserialize(deserializer)? {
            Some(true) => Ok(Tristate::True),
            Some(false) => Ok(Tristate::False),
            None => Ok(Tristate::Unknown),
        }
    }
}

pub fn bool_to_tristate(b: bool) -> Tristate {
    Tristate::from(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tristate_basics() {
        assert!(Tristate::True.is_true());
        assert!(Tristate::False.is_false());
        assert!(Tristate::Unknown.is_unknown());
        assert!(Tristate::True.is_true_or_unknown());
        assert!(Tristate::Unknown.is_true_or_unknown());
        assert!(!Tristate::False.is_true_or_unknown());
    }

    #[test]
    fn default_if_unknown() {
        assert_eq!(
            Tristate::Unknown.default_if_unknown(Tristate::True),
            Tristate::True
        );
        assert_eq!(
            Tristate::False.default_if_unknown(Tristate::True),
            Tristate::False
        );
    }
}
