use std::fmt;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Locale(pub String);

impl Locale {
    pub fn default_locale() -> Locale {
        Locale(String::new())
    }

    pub fn parse(s: &str) -> Option<Locale> {
        if s.is_empty() {
            return Some(Locale::default_locale());
        }

        if s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            Some(Locale(s.to_string()))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Locale {
    fn from(s: &str) -> Self {
        Locale(s.to_string())
    }
}

impl From<String> for Locale {
    fn from(s: String) -> Self {
        Locale(s)
    }
}

#[cfg(test)]
mod tests;
