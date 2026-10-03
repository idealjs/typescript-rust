#[derive(Clone, Debug, Default)]
pub struct JsonValue {
    pub value_type: JsonValueType,
    pub string_value: Option<String>,
    pub number_value: Option<f64>,
    pub bool_value: Option<bool>,
    pub array_value: Option<Vec<JsonValue>>,
    pub object_value: Option<Vec<(String, JsonValue)>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum JsonValueType {
    #[default]
    NotPresent,
    Null,
    String,
    Number,
    Boolean,
    Array,
    Object,
}

impl JsonValue {
    pub fn is_present(&self) -> bool { ::tsox_core::fntrace::enter("is_present"); 
        self.value_type != JsonValueType::NotPresent
    }

    pub fn is_falsy(&self) -> bool { ::tsox_core::fntrace::enter("is_falsy"); 
        match self.value_type {
            JsonValueType::NotPresent | JsonValueType::Null => true,
            JsonValueType::String => self.string_value.as_deref() == Some(""),
            JsonValueType::Number => self.number_value == Some(0.0),
            JsonValueType::Boolean => self.bool_value == Some(false),
            _ => false,
        }
    }

    pub fn as_string(&self) -> &str { ::tsox_core::fntrace::enter("as_string"); 
        self.string_value.as_deref().unwrap_or("")
    }

    pub fn as_array(&self) -> &[JsonValue] { ::tsox_core::fntrace::enter("as_array"); 
        self.array_value.as_deref().unwrap_or(&[])
    }

    pub fn as_object(&self) -> &[(String, JsonValue)] { ::tsox_core::fntrace::enter("as_object"); 
        self.object_value.as_deref().unwrap_or(&[])
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> { ::tsox_core::fntrace::enter("get"); 
        self.object_value
            .as_ref()?
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }
}

impl From<serde_json::Value> for JsonValue {
    fn from(v: serde_json::Value) -> JsonValue { ::tsox_core::fntrace::enter("from"); 
        match v {
            serde_json::Value::Null => JsonValue {
                value_type: JsonValueType::Null,
                ..Default::default()
            },
            serde_json::Value::Bool(b) => JsonValue {
                value_type: JsonValueType::Boolean,
                bool_value: Some(b),
                ..Default::default()
            },
            serde_json::Value::Number(n) => JsonValue {
                value_type: JsonValueType::Number,
                number_value: n.as_f64(),
                ..Default::default()
            },
            serde_json::Value::String(s) => JsonValue {
                value_type: JsonValueType::String,
                string_value: Some(s),
                ..Default::default()
            },
            serde_json::Value::Array(arr) => JsonValue {
                value_type: JsonValueType::Array,
                array_value: Some(arr.into_iter().map(JsonValue::from).collect()),
                ..Default::default()
            },
            serde_json::Value::Object(obj) => JsonValue {
                value_type: JsonValueType::Object,
                object_value: Some(obj.into_iter().map(|(k, v)| (k, v.into())).collect()),
                ..Default::default()
            },
        }
    }
}
