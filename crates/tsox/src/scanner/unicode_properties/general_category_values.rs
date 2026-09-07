use std::collections::HashSet;
use std::sync::LazyLock;

use super::*;

pub static GENERAL_CATEGORY_VALUES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "C",
        "Other",
        "Cc",
        "Control",
        "cntrl",
        "Cf",
        "Format",
        "Cn",
        "Unassigned",
        "Co",
        "Private_Use",
        "Cs",
        "Surrogate",
        "L",
        "Letter",
        "LC",
        "Cased_Letter",
        "Ll",
        "Lowercase_Letter",
        "Lm",
        "Modifier_Letter",
        "Lo",
        "Other_Letter",
        "Lt",
        "Titlecase_Letter",
        "Lu",
        "Uppercase_Letter",
        "M",
        "Mark",
        "Combining_Mark",
        "Mc",
        "Spacing_Mark",
        "Me",
        "Enclosing_Mark",
        "Mn",
        "Nonspacing_Mark",
        "N",
        "Number",
        "Nd",
        "Decimal_Number",
        "digit",
        "Nl",
        "Letter_Number",
        "No",
        "Other_Number",
        "P",
        "Punctuation",
        "punct",
        "Pc",
        "Connector_Punctuation",
        "Pd",
        "Dash_Punctuation",
        "Pe",
        "Close_Punctuation",
        "Pf",
        "Final_Punctuation",
        "Pi",
        "Initial_Punctuation",
        "Po",
        "Other_Punctuation",
        "Ps",
        "Open_Punctuation",
        "S",
        "Symbol",
        "Sc",
        "Currency_Symbol",
        "Sk",
        "Modifier_Symbol",
        "Sm",
        "Math_Symbol",
        "So",
        "Other_Symbol",
        "Z",
        "Separator",
        "Zl",
        "Line_Separator",
        "Zp",
        "Paragraph_Separator",
        "Zs",
        "Space_Separator",
    ])
});

pub fn non_binary_property_canonical(name: &str) -> Option<&'static str> {
    NON_BINARY_UNICODE_PROPERTIES
        .iter()
        .find(|(alias, _)| *alias == name)
        .map(|(_, canonical)| *canonical)
}

pub fn is_binary_unicode_property(name: &str) -> bool {
    BINARY_UNICODE_PROPERTIES.contains(name)
}

pub fn is_binary_unicode_property_of_strings(name: &str) -> bool {
    BINARY_UNICODE_PROPERTIES_OF_STRINGS.contains(name)
}

pub fn is_valid_unicode_property_value(property: &str, value: &str) -> bool {
    match property {
        "General_Category" => GENERAL_CATEGORY_VALUES.contains(value),
        "Script" | "Script_Extensions" => SCRIPT_VALUES.contains(value),
        _ => false,
    }
}
