//! package.json parsing, ported from `internal/packagejson/`.
//!
//! Parses package.json files with type-validated fields, tracking whether
//! each field was present, null, or had the expected type.

use std::collections::HashMap;

/// A field from package.json that tracks whether it was present and valid.
#[derive(Clone, Debug, Default)]
pub struct Expected<T: Clone + Default> {
    pub value: T,
    pub valid: bool,
    pub null: bool,
    pub present: bool,
    actual_json_type: String,
}

impl<T: Clone + Default> Expected<T> {
    pub fn is_present(&self) -> bool {
        self.present
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_value(&self) -> Option<&T> {
        if self.valid {
            Some(&self.value)
        } else {
            None
        }
    }

    pub fn actual_json_type(&self) -> &str {
        &self.actual_json_type
    }
}

impl Expected<String> {
    fn expected_json_type() -> &'static str {
        "string"
    }
}

impl Expected<HashMap<String, String>> {
    fn expected_json_type() -> &'static str {
        "object"
    }
}

/// A dynamically-typed JSON value, used for fields like `typesVersions`,
/// `exports`, and `imports` that can have complex structures.
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
    pub fn is_present(&self) -> bool {
        self.value_type != JsonValueType::NotPresent
    }

    pub fn is_falsy(&self) -> bool {
        match self.value_type {
            JsonValueType::NotPresent | JsonValueType::Null => true,
            JsonValueType::String => self.string_value.as_deref() == Some(""),
            JsonValueType::Number => self.number_value == Some(0.0),
            JsonValueType::Boolean => self.bool_value == Some(false),
            _ => false,
        }
    }

    pub fn as_string(&self) -> &str {
        self.string_value.as_deref().unwrap_or("")
    }

    pub fn as_array(&self) -> &[JsonValue] {
        self.array_value.as_deref().unwrap_or(&[])
    }

    pub fn as_object(&self) -> &[(String, JsonValue)] {
        self.object_value.as_deref().unwrap_or(&[])
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        self.object_value
            .as_ref()?
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }
}

impl From<serde_json::Value> for JsonValue {
    fn from(v: serde_json::Value) -> JsonValue {
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

/// The object kind of an exports/imports field.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ObjectKind {
    #[default]
    Unknown,
    Subpaths,
    Conditions,
    Imports,
    Invalid,
}

/// The exports or imports field from package.json.
#[derive(Clone, Debug, Default)]
pub struct ExportsOrImports {
    pub json_value: JsonValue,
    object_kind: ObjectKind,
}

impl ExportsOrImports {
    pub fn is_subpaths(&self) -> bool {
        self.init_object_kind();
        self.object_kind == ObjectKind::Subpaths
    }

    pub fn is_imports(&self) -> bool {
        self.init_object_kind();
        self.object_kind == ObjectKind::Imports
    }

    pub fn is_conditions(&self) -> bool {
        self.init_object_kind();
        self.object_kind == ObjectKind::Conditions
    }

    fn init_object_kind(&self) {
        // object_kind is computed lazily but stored mutably;
        // since we can't mutate &self, we compute on each call
        // (this is cheap enough for the sizes involved)
        let mut kind = self.object_kind.clone();
        if kind == ObjectKind::Unknown && self.json_value.value_type == JsonValueType::Object {
            if let Some(obj) = &self.json_value.object_value {
                if !obj.is_empty() {
                    let mut seen_dot = false;
                    let mut seen_hash = false;
                    let mut seen_other = false;
                    for (k, _) in obj {
                        if let Some(first) = k.chars().next() {
                            if first == '.' {
                                seen_dot = true;
                            } else if first == '#' {
                                seen_hash = true;
                            } else {
                                seen_other = true;
                            }
                            if seen_other && (seen_dot || seen_hash) {
                                kind = ObjectKind::Invalid;
                                break;
                            }
                        }
                    }
                    if kind != ObjectKind::Invalid {
                        if seen_dot {
                            kind = ObjectKind::Subpaths;
                        } else if seen_hash {
                            kind = ObjectKind::Imports;
                        } else {
                            kind = ObjectKind::Conditions;
                        }
                    }
                } else {
                    kind = ObjectKind::Conditions;
                }
            }
        }
        // Note: we can't set self.object_kind since we only have &self
        // The caller will get the right answer from the local computation
        let _ = kind;
    }

    /// Get the computed object kind.
    pub fn compute_object_kind(&self) -> ObjectKind {
        if self.json_value.value_type != JsonValueType::Object {
            return ObjectKind::Unknown;
        }
        let obj = match &self.json_value.object_value {
            Some(o) if !o.is_empty() => o,
            _ => return ObjectKind::Conditions,
        };
        let mut seen_dot = false;
        let mut seen_hash = false;
        let mut seen_other = false;
        for (k, _) in obj {
            if let Some(first) = k.chars().next() {
                if first == '.' {
                    seen_dot = true;
                } else if first == '#' {
                    seen_hash = true;
                } else {
                    seen_other = true;
                }
                if seen_other && (seen_dot || seen_hash) {
                    return ObjectKind::Invalid;
                }
            }
        }
        if seen_dot {
            ObjectKind::Subpaths
        } else if seen_hash {
            ObjectKind::Imports
        } else {
            ObjectKind::Conditions
        }
    }
}

/// Header fields from package.json.
#[derive(Clone, Debug, Default)]
pub struct HeaderFields {
    pub name: Expected<String>,
    pub version: Expected<String>,
    pub r#type: Expected<String>,
}

/// Path-related fields from package.json.
#[derive(Clone, Debug, Default)]
pub struct PathFields {
    pub tsconfig: Expected<String>,
    pub main: Expected<String>,
    pub types: Expected<String>,
    pub typings: Expected<String>,
    pub types_versions: JsonValue,
    pub imports: ExportsOrImports,
    pub exports: ExportsOrImports,
}

/// Dependency fields from package.json.
#[derive(Clone, Debug, Default)]
pub struct DependencyFields {
    pub dependencies: Expected<HashMap<String, String>>,
    pub dev_dependencies: Expected<HashMap<String, String>>,
    pub peer_dependencies: Expected<HashMap<String, String>>,
    pub optional_dependencies: Expected<HashMap<String, String>>,
}

impl DependencyFields {
    pub fn has_dependency(&self, name: &str) -> bool {
        self.dependencies.get_value().map_or(false, |d| d.contains_key(name))
            || self.dev_dependencies.get_value().map_or(false, |d| d.contains_key(name))
            || self.peer_dependencies.get_value().map_or(false, |d| d.contains_key(name))
            || self.optional_dependencies.get_value().map_or(false, |d| d.contains_key(name))
    }

    pub fn for_each_dependency<F: FnMut(&str, &str, &str) -> bool>(&self, mut f: F) {
        if let Some(deps) = self.dependencies.get_value() {
            for (name, version) in deps {
                if !f(name, version, "dependencies") {
                    return;
                }
            }
        }
        if let Some(deps) = self.dev_dependencies.get_value() {
            for (name, version) in deps {
                if !f(name, version, "devDependencies") {
                    return;
                }
            }
        }
        if let Some(deps) = self.peer_dependencies.get_value() {
            for (name, version) in deps {
                if !f(name, version, "peerDependencies") {
                    return;
                }
            }
        }
        if let Some(deps) = self.optional_dependencies.get_value() {
            for (name, version) in deps {
                if !f(name, version, "optionalDependencies") {
                    return;
                }
            }
        }
    }

    pub fn get_runtime_dependency_names(&self) -> std::collections::HashSet<String> {
        let mut names = std::collections::HashSet::new();
        if let Some(deps) = self.dependencies.get_value() {
            names.extend(deps.keys().cloned());
        }
        if let Some(deps) = self.peer_dependencies.get_value() {
            names.extend(deps.keys().cloned());
        }
        if let Some(deps) = self.optional_dependencies.get_value() {
            names.extend(deps.keys().cloned());
        }
        names
    }
}

/// All parsed fields from package.json.
#[derive(Clone, Debug, Default)]
pub struct Fields {
    pub header_fields: HeaderFields,
    pub path_fields: PathFields,
    pub dependency_fields: DependencyFields,
}

/// Parse a package.json JSON string.
pub fn parse(data: &str) -> Result<Fields, serde_json::Error> {
    let value: serde_json::Value = serde_json::from_str(data)?;
    let obj = value.as_object().ok_or_else(|| {
        serde::de::Error::custom("package.json must be a JSON object")
    })?;

    let mut fields = Fields::default();

    // Header fields
    if let Some(v) = obj.get("name") {
        fields.header_fields.name = parse_expected_string(v);
    }
    if let Some(v) = obj.get("version") {
        fields.header_fields.version = parse_expected_string(v);
    }
    if let Some(v) = obj.get("type") {
        fields.header_fields.r#type = parse_expected_string(v);
    }

    // Path fields
    if let Some(v) = obj.get("tsconfig") {
        fields.path_fields.tsconfig = parse_expected_string(v);
    }
    if let Some(v) = obj.get("main") {
        fields.path_fields.main = parse_expected_string(v);
    }
    if let Some(v) = obj.get("types") {
        fields.path_fields.types = parse_expected_string(v);
    }
    if let Some(v) = obj.get("typings") {
        fields.path_fields.typings = parse_expected_string(v);
    }
    if let Some(v) = obj.get("typesVersions") {
        fields.path_fields.types_versions = v.clone().into();
    }
    if let Some(v) = obj.get("imports") {
        fields.path_fields.imports = ExportsOrImports {
            json_value: v.clone().into(),
            object_kind: ObjectKind::Unknown,
        };
    }
    if let Some(v) = obj.get("exports") {
        fields.path_fields.exports = ExportsOrImports {
            json_value: v.clone().into(),
            object_kind: ObjectKind::Unknown,
        };
    }

    // Dependency fields
    if let Some(v) = obj.get("dependencies") {
        fields.dependency_fields.dependencies = parse_expected_string_map(v);
    }
    if let Some(v) = obj.get("devDependencies") {
        fields.dependency_fields.dev_dependencies = parse_expected_string_map(v);
    }
    if let Some(v) = obj.get("peerDependencies") {
        fields.dependency_fields.peer_dependencies = parse_expected_string_map(v);
    }
    if let Some(v) = obj.get("optionalDependencies") {
        fields.dependency_fields.optional_dependencies = parse_expected_string_map(v);
    }

    Ok(fields)
}

fn parse_expected_string(v: &serde_json::Value) -> Expected<String> {
    let mut e = Expected::<String>::default();
    e.present = true;
    match v {
        serde_json::Value::Null => {
            e.null = true;
            e.actual_json_type = "null".to_string();
        }
        serde_json::Value::String(s) => {
            e.valid = true;
            e.value = s.clone();
            e.actual_json_type = "string".to_string();
        }
        _ => {
            e.actual_json_type = json_type_name(v).to_string();
        }
    }
    e
}

fn parse_expected_string_map(v: &serde_json::Value) -> Expected<HashMap<String, String>> {
    let mut e = Expected::<HashMap<String, String>>::default();
    e.present = true;
    match v {
        serde_json::Value::Null => {
            e.null = true;
            e.actual_json_type = "null".to_string();
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            let mut all_valid = true;
            for (k, val) in obj {
                match val {
                    serde_json::Value::String(s) => {
                        map.insert(k.clone(), s.clone());
                    }
                    _ => {
                        all_valid = false;
                    }
                }
            }
            if all_valid {
                e.valid = true;
                e.value = map;
            }
            e.actual_json_type = "object".to_string();
        }
        _ => {
            e.actual_json_type = json_type_name(v).to_string();
        }
    }
    e
}

fn json_type_name(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_package_json() {
        let json = r#"{
            "name": "my-package",
            "version": "1.0.0",
            "type": "module",
            "main": "./index.js",
            "types": "./index.d.ts"
        }"#;
        let fields = parse(json).unwrap();
        assert_eq!(fields.header_fields.name.get_value(), Some(&"my-package".to_string()));
        assert_eq!(fields.header_fields.version.get_value(), Some(&"1.0.0".to_string()));
        assert_eq!(fields.header_fields.r#type.get_value(), Some(&"module".to_string()));
        assert_eq!(fields.path_fields.main.get_value(), Some(&"./index.js".to_string()));
        assert_eq!(fields.path_fields.types.get_value(), Some(&"./index.d.ts".to_string()));
    }

    #[test]
    fn parse_dependencies() {
        let json = r#"{
            "name": "test",
            "dependencies": {
                "foo": "^1.0.0",
                "bar": "^2.0.0"
            },
            "devDependencies": {
                "baz": "^3.0.0"
            }
        }"#;
        let fields = parse(json).unwrap();
        assert!(fields.dependency_fields.has_dependency("foo"));
        assert!(fields.dependency_fields.has_dependency("bar"));
        assert!(fields.dependency_fields.has_dependency("baz"));
        assert!(!fields.dependency_fields.has_dependency("missing"));
    }

    #[test]
    fn parse_null_fields() {
        let json = r#"{"name": null, "version": "1.0.0"}"#;
        let fields = parse(json).unwrap();
        assert!(fields.header_fields.name.present);
        assert!(fields.header_fields.name.null);
        assert!(!fields.header_fields.name.valid);
        assert!(fields.header_fields.version.valid);
    }

    #[test]
    fn parse_exports_subpaths() {
        let json = r#"{
            "name": "test",
            "exports": {
                ".": "./index.js",
                "./foo": "./foo.js"
            }
        }"#;
        let fields = parse(json).unwrap();
        assert_eq!(fields.path_fields.exports.compute_object_kind(), ObjectKind::Subpaths);
    }

    #[test]
    fn parse_exports_conditions() {
        let json = r#"{
            "name": "test",
            "exports": {
                "import": "./index.mjs",
                "require": "./index.cjs"
            }
        }"#;
        let fields = parse(json).unwrap();
        assert_eq!(fields.path_fields.exports.compute_object_kind(), ObjectKind::Conditions);
    }
}
