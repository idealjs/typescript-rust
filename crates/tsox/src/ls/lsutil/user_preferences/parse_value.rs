use crate::core::tristate::Tristate;
use crate::modulespecifiers;

use serde_json::Value;

pub(super) fn parse_tristate(value: &Value) -> Tristate {
    match value {
        Value::Bool(b) => Tristate::from(*b),
        _ => Tristate::Unknown,
    }
}

pub(super) fn parse_bool(value: &Value) -> bool {
    matches!(value, Value::Bool(true))
}

pub(super) fn parse_i32(value: &Value) -> i32 {
    match value {
        Value::Number(n) => n.as_i64().map(|i| i as i32).unwrap_or(0),
        _ => 0,
    }
}

pub(super) fn parse_string(value: &Value) -> String {
    if let Value::String(s) = value {
        s.clone()
    } else {
        String::new()
    }
}

pub(super) fn parse_string_array(value: &Value) -> Vec<String> {
    if let Value::Array(arr) = value {
        arr.iter()
            .filter_map(|item| {
                if let Value::String(s) = item {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .collect()
    } else {
        Vec::new()
    }
}

pub(super) fn parse_module_specifier_preference(
    value: &Value,
) -> modulespecifiers::ImportModuleSpecifierPreference {
    if let Value::String(s) = value {
        match s.to_ascii_lowercase().as_str() {
            "project-relative" => {
                modulespecifiers::IMPORT_MODULE_SPECIFIER_PREFERENCE_PROJECT_RELATIVE.to_string()
            }
            "relative" => modulespecifiers::IMPORT_MODULE_SPECIFIER_PREFERENCE_RELATIVE.to_string(),
            "non-relative" => {
                modulespecifiers::IMPORT_MODULE_SPECIFIER_PREFERENCE_NON_RELATIVE.to_string()
            }
            _ => modulespecifiers::IMPORT_MODULE_SPECIFIER_PREFERENCE_SHORTEST.to_string(),
        }
    } else {
        modulespecifiers::IMPORT_MODULE_SPECIFIER_PREFERENCE_SHORTEST.to_string()
    }
}

pub(super) fn parse_module_specifier_ending(
    value: &Value,
) -> modulespecifiers::ImportModuleSpecifierEndingPreference {
    if let Value::String(s) = value {
        match s.to_ascii_lowercase().as_str() {
            "minimal" => {
                modulespecifiers::IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_MINIMAL.to_string()
            }
            "index" => {
                modulespecifiers::IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_INDEX.to_string()
            }
            "js" => modulespecifiers::IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_JS.to_string(),
            _ => modulespecifiers::IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_AUTO.to_string(),
        }
    } else {
        modulespecifiers::IMPORT_MODULE_SPECIFIER_ENDING_PREFERENCE_AUTO.to_string()
    }
}

pub(super) fn parse_case_sensitivity(value: &Value) -> Tristate {
    if let Value::String(s) = value {
        return match s.to_ascii_lowercase().as_str() {
            "caseinsensitive" => Tristate::True,
            "casesensitive" => Tristate::False,
            _ => Tristate::Unknown,
        };
    }
    parse_tristate(value)
}
