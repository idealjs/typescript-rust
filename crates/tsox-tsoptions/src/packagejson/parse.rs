use super::expected::Expected;
use super::exports::ExportsOrImports;
use super::fields::Fields;
use std::collections::HashMap;

pub fn parse(data: &str) -> Result<Fields, serde_json::Error> {
    let value: serde_json::Value = serde_json::from_str(data)?;
    let obj = value
        .as_object()
        .ok_or_else(|| serde::de::Error::custom("package.json must be a JSON object"))?;

    let mut fields = Fields::default();

    if let Some(v) = obj.get("name") {
        fields.header_fields.name = parse_expected_string(v);
    }
    if let Some(v) = obj.get("version") {
        fields.header_fields.version = parse_expected_string(v);
    }
    if let Some(v) = obj.get("type") {
        fields.header_fields.r#type = parse_expected_string(v);
    }

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
            object_kind: super::exports::ObjectKind::Unknown,
        };
    }
    if let Some(v) = obj.get("exports") {
        fields.path_fields.exports = ExportsOrImports {
            json_value: v.clone().into(),
            object_kind: super::exports::ObjectKind::Unknown,
        };
    }

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
