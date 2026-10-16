use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuotePreference {
    #[default]
    Unknown,
    Auto,
    Double,
    Single,
}

impl QuotePreference {
    pub fn as_str(self) -> &'static str { ::tsox_core::fntrace::enter("as_str"); 
        match self {
            QuotePreference::Unknown => "",
            QuotePreference::Auto => "auto",
            QuotePreference::Double => "double",
            QuotePreference::Single => "single",
        }
    }

    pub fn parse(value: &Value) -> QuotePreference { ::tsox_core::fntrace::enter("parse"); 
        if let Value::String(s) = value {
            return match s.to_ascii_lowercase().as_str() {
                "auto" => QuotePreference::Auto,
                "double" => QuotePreference::Double,
                "single" => QuotePreference::Single,
                _ => QuotePreference::Unknown,
            };
        }
        QuotePreference::Unknown
    }
}

impl Serialize for QuotePreference {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for QuotePreference {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        Ok(match String::deserialize(deserializer)?.to_ascii_lowercase().as_str() {
            "auto" => Self::Auto,
            "double" => Self::Double,
            "single" => Self::Single,
            _ => Self::Unknown,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JsxAttributeCompletionStyle {
    #[default]
    Unknown,
    Auto,
    Braces,
    None,
}

impl JsxAttributeCompletionStyle {
    pub fn as_str(self) -> &'static str { ::tsox_core::fntrace::enter("as_str"); 
        match self {
            JsxAttributeCompletionStyle::Unknown => "",
            JsxAttributeCompletionStyle::Auto => "auto",
            JsxAttributeCompletionStyle::Braces => "braces",
            JsxAttributeCompletionStyle::None => "none",
        }
    }

    pub fn parse(value: &Value) -> JsxAttributeCompletionStyle { ::tsox_core::fntrace::enter("parse"); 
        if let Value::String(s) = value {
            return match s.to_ascii_lowercase().as_str() {
                "braces" => JsxAttributeCompletionStyle::Braces,
                "none" => JsxAttributeCompletionStyle::None,
                _ => JsxAttributeCompletionStyle::Auto,
            };
        }
        JsxAttributeCompletionStyle::Auto
    }
}

impl Serialize for JsxAttributeCompletionStyle {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for JsxAttributeCompletionStyle {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        Ok(match String::deserialize(deserializer)?.as_str() {
            "braces" => Self::Braces,
            "none" => Self::None,
            "" => Self::Unknown,
            _ => Self::Auto,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IncludeInlayParameterNameHints {
    #[default]
    None,
    All,
    Literals,
}

impl IncludeInlayParameterNameHints {
    pub fn as_str(self) -> &'static str { ::tsox_core::fntrace::enter("as_str"); 
        match self {
            IncludeInlayParameterNameHints::None => "",
            IncludeInlayParameterNameHints::All => "all",
            IncludeInlayParameterNameHints::Literals => "literals",
        }
    }

    pub fn parse(value: &Value) -> IncludeInlayParameterNameHints { ::tsox_core::fntrace::enter("parse"); 
        if let Value::String(s) = value {
            return match s.as_str() {
                "all" => IncludeInlayParameterNameHints::All,
                "literals" => IncludeInlayParameterNameHints::Literals,
                _ => IncludeInlayParameterNameHints::None,
            };
        }
        IncludeInlayParameterNameHints::None
    }
}

impl Serialize for IncludeInlayParameterNameHints {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for IncludeInlayParameterNameHints {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        Ok(match String::deserialize(deserializer)?.as_str() {
            "all" => Self::All,
            "literals" => Self::Literals,
            _ => Self::None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum OrganizeImportsSort {
    #[default]
    Auto = 0,
    Ordinal = 1,
    OrdinalIgnoreCase = 2,
    Natural = 3,
    NaturalIgnoreCase = 4,
}

impl OrganizeImportsSort {
    pub fn parse(value: &Value) -> OrganizeImportsSort { ::tsox_core::fntrace::enter("parse"); 
        if let Value::String(s) = value {
            return match s.to_ascii_lowercase().as_str() {
                "ordinal" => OrganizeImportsSort::Ordinal,
                "ordinalignorecase" => OrganizeImportsSort::OrdinalIgnoreCase,
                "natural" => OrganizeImportsSort::Natural,
                "naturalignorecase" => OrganizeImportsSort::NaturalIgnoreCase,
                _ => OrganizeImportsSort::Auto,
            };
        }
        OrganizeImportsSort::Auto
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrganizeImportsCollation {
    #[default]
    Ordinal,
    Unicode,
}

impl OrganizeImportsCollation {
    pub fn parse(value: &Value) -> OrganizeImportsCollation { ::tsox_core::fntrace::enter("parse"); 
        if let Value::String(s) = value {
            if s.to_ascii_lowercase() == "unicode" {
                return OrganizeImportsCollation::Unicode;
            }
        }
        OrganizeImportsCollation::Ordinal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum OrganizeImportsCaseFirst {
    #[default]
    False = 0,
    Lower = 1,
    Upper = 2,
}

impl OrganizeImportsCaseFirst {
    pub fn parse(value: &Value) -> OrganizeImportsCaseFirst { ::tsox_core::fntrace::enter("parse"); 
        if let Value::String(s) = value {
            return match s.as_str() {
                "lower" => OrganizeImportsCaseFirst::Lower,
                "upper" => OrganizeImportsCaseFirst::Upper,
                _ => OrganizeImportsCaseFirst::False,
            };
        }
        OrganizeImportsCaseFirst::False
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum OrganizeImportsTypeOrder {
    #[default]
    Auto = 0,
    Last = 1,
    Inline = 2,
    First = 3,
}

impl OrganizeImportsTypeOrder {
    pub fn parse(value: &Value) -> OrganizeImportsTypeOrder { ::tsox_core::fntrace::enter("parse"); 
        if let Value::String(s) = value {
            return match s.as_str() {
                "last" => OrganizeImportsTypeOrder::Last,
                "inline" => OrganizeImportsTypeOrder::Inline,
                "first" => OrganizeImportsTypeOrder::First,
                _ => OrganizeImportsTypeOrder::Auto,
            };
        }
        OrganizeImportsTypeOrder::Auto
    }
}

impl Serialize for OrganizeImportsSort {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        serializer.serialize_i32(*self as i32)
    }
}

impl<'de> Deserialize<'de> for OrganizeImportsSort {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        Ok(match i32::deserialize(deserializer)? {
            1 => Self::Ordinal,
            2 => Self::OrdinalIgnoreCase,
            3 => Self::Natural,
            4 => Self::NaturalIgnoreCase,
            _ => Self::Auto,
        })
    }
}

impl Serialize for OrganizeImportsCollation {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        serializer.serialize_bool(*self == Self::Unicode)
    }
}

impl<'de> Deserialize<'de> for OrganizeImportsCollation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        Ok(if bool::deserialize(deserializer)? {
            Self::Unicode
        } else {
            Self::Ordinal
        })
    }
}

impl Serialize for OrganizeImportsCaseFirst {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        serializer.serialize_i32(*self as i32)
    }
}

impl<'de> Deserialize<'de> for OrganizeImportsCaseFirst {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        Ok(match i32::deserialize(deserializer)? {
            1 => Self::Lower,
            2 => Self::Upper,
            _ => Self::False,
        })
    }
}

impl Serialize for OrganizeImportsTypeOrder {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { ::tsox_core::fntrace::enter("serialize"); 
        serializer.serialize_i32(*self as i32)
    }
}

impl<'de> Deserialize<'de> for OrganizeImportsTypeOrder {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> { ::tsox_core::fntrace::enter("deserialize"); 
        Ok(match i32::deserialize(deserializer)? {
            1 => Self::Last,
            2 => Self::Inline,
            3 => Self::First,
            _ => Self::Auto,
        })
    }
}
