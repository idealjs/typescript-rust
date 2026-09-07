use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Expected<T: Clone + Default> {
    pub value: T,
    pub valid: bool,
    pub null: bool,
    pub present: bool,
    pub(super) actual_json_type: String,
}

impl<T: Clone + Default> Expected<T> {
    pub fn is_present(&self) -> bool {
        self.present
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_value(&self) -> Option<&T> {
        if self.valid { Some(&self.value) } else { None }
    }

    pub fn actual_json_type(&self) -> &str {
        &self.actual_json_type
    }
}

impl Expected<String> {
    #[allow(dead_code)]
    fn expected_json_type() -> &'static str {
        "string"
    }
}

impl Expected<HashMap<String, String>> {
    #[allow(dead_code)]
    fn expected_json_type() -> &'static str {
        "object"
    }
}
