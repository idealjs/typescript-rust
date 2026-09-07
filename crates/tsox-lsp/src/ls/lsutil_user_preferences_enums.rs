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
    pub fn as_str(self) -> &'static str {
        match self {
            QuotePreference::Unknown => "",
            QuotePreference::Auto => "auto",
            QuotePreference::Double => "double",
            QuotePreference::Single => "single",
        }
    }

    pub fn parse(value: &Value) -> QuotePreference {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JsxAttributeCompletionStyle {
    #[default]
    Unknown,
    Auto,
    Braces,
    None,
}

impl JsxAttributeCompletionStyle {
    pub fn parse(value: &Value) -> JsxAttributeCompletionStyle {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IncludeInlayParameterNameHints {
    #[default]
    None,
    All,
    Literals,
}

impl IncludeInlayParameterNameHints {
    pub fn parse(value: &Value) -> IncludeInlayParameterNameHints {
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
    pub fn parse(value: &Value) -> OrganizeImportsSort {
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
    pub fn parse(value: &Value) -> OrganizeImportsCollation {
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
    pub fn parse(value: &Value) -> OrganizeImportsCaseFirst {
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
    pub fn parse(value: &Value) -> OrganizeImportsTypeOrder {
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
