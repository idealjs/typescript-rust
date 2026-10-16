use crate::packagejson::expected::Expected;
use std::collections::HashMap;

pub trait ExpectedJsonTypeName {
    fn expected_json_type() -> &'static str;
}

impl ExpectedJsonTypeName for String {
    fn expected_json_type() -> &'static str { ::tsox_core::fntrace::enter("expected_json_type"); 
        "string"
    }
}

impl ExpectedJsonTypeName for bool {
    fn expected_json_type() -> &'static str { ::tsox_core::fntrace::enter("expected_json_type"); 
        "boolean"
    }
}

impl<T: ExpectedJsonTypeName> ExpectedJsonTypeName for Vec<T> {
    fn expected_json_type() -> &'static str { ::tsox_core::fntrace::enter("expected_json_type"); 
        "array"
    }
}

impl ExpectedJsonTypeName for HashMap<String, String> {
    fn expected_json_type() -> &'static str { ::tsox_core::fntrace::enter("expected_json_type"); 
        "object"
    }
}

impl ExpectedJsonTypeName for f64 {
    fn expected_json_type() -> &'static str { ::tsox_core::fntrace::enter("expected_json_type"); 
        "number"
    }
}

impl ExpectedJsonTypeName for i64 {
    fn expected_json_type() -> &'static str { ::tsox_core::fntrace::enter("expected_json_type"); 
        "number"
    }
}

pub fn expected_of<T: Clone + Default + ExpectedJsonTypeName>(value: T) -> Expected<T> { ::tsox_core::fntrace::enter("expected_of"); 
    Expected {
        value,
        valid: true,
        null: false,
        present: true,
        actual_json_type: T::expected_json_type().to_string(),
    }
}
