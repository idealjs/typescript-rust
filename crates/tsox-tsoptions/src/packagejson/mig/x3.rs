use crate::packagejson::expected::Expected;
use std::collections::HashMap;

pub trait ExpectedJsonTypeName {
    fn expected_json_type() -> &'static str;
}

impl ExpectedJsonTypeName for String {
    fn expected_json_type() -> &'static str {
        "string"
    }
}

impl ExpectedJsonTypeName for bool {
    fn expected_json_type() -> &'static str {
        "boolean"
    }
}

impl<T: ExpectedJsonTypeName> ExpectedJsonTypeName for Vec<T> {
    fn expected_json_type() -> &'static str {
        "array"
    }
}

impl ExpectedJsonTypeName for HashMap<String, String> {
    fn expected_json_type() -> &'static str {
        "object"
    }
}

impl ExpectedJsonTypeName for f64 {
    fn expected_json_type() -> &'static str {
        "number"
    }
}

impl ExpectedJsonTypeName for i64 {
    fn expected_json_type() -> &'static str {
        "number"
    }
}

pub fn expected_of<T: Clone + Default + ExpectedJsonTypeName>(value: T) -> Expected<T> {
    Expected {
        value,
        valid: true,
        null: false,
        present: true,
        actual_json_type: T::expected_json_type().to_string(),
    }
}
