pub(crate) use std::fmt;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Locale(pub String);

impl Locale {
    pub fn default_locale() -> Locale { crate::fntrace::enter("default_locale"); 
        Locale(String::new())
    }

    pub fn parse(s: &str) -> Option<Locale> { crate::fntrace::enter("parse"); 
        if s.is_empty() {
            return Some(Locale::default_locale());
        }

        if s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            Some(Locale(s.to_string()))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &str { crate::fntrace::enter("as_str"); 
        &self.0
    }

    pub fn is_empty(&self) -> bool { crate::fntrace::enter("is_empty"); 
        self.0.is_empty()
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { crate::fntrace::enter("fmt"); 
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Locale {
    fn from(s: &str) -> Self { crate::fntrace::enter("from"); 
        Locale(s.to_string())
    }
}

impl From<String> for Locale {
    fn from(s: String) -> Self { crate::fntrace::enter("from"); 
        Locale(s)
    }
}

#[cfg(test)]
pub(crate) mod tests;

// r 轮接线:迁移批次模块
pub mod mig;
