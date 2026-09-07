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
    assert_eq!(
        fields.header_fields.name.get_value(),
        Some(&"my-package".to_string())
    );
    assert_eq!(
        fields.header_fields.version.get_value(),
        Some(&"1.0.0".to_string())
    );
    assert_eq!(
        fields.header_fields.r#type.get_value(),
        Some(&"module".to_string())
    );
    assert_eq!(
        fields.path_fields.main.get_value(),
        Some(&"./index.js".to_string())
    );
    assert_eq!(
        fields.path_fields.types.get_value(),
        Some(&"./index.d.ts".to_string())
    );
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
    assert_eq!(
        fields.path_fields.exports.compute_object_kind(),
        ObjectKind::Subpaths
    );
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
    assert_eq!(
        fields.path_fields.exports.compute_object_kind(),
        ObjectKind::Conditions
    );
}

#[test]
fn exports_classification_fixed() {
    let e = ExportsOrImports {
        json_value: JsonValue {
            value_type: JsonValueType::Object,
            object_value: Some(vec![
                (".".to_string(), JsonValue::default()),
                ("./feature".to_string(), JsonValue::default()),
            ]),
            ..Default::default()
        },
        object_kind: ObjectKind::Unknown,
    };
    assert!(e.is_subpaths());
    assert!(!e.is_conditions());

    let e = ExportsOrImports {
        json_value: JsonValue {
            value_type: JsonValueType::Object,
            object_value: Some(vec![
                ("import".to_string(), JsonValue::default()),
                ("require".to_string(), JsonValue::default()),
            ]),
            ..Default::default()
        },
        object_kind: ObjectKind::Unknown,
    };
    assert!(e.is_conditions());
    assert!(!e.is_subpaths());
}

#[test]
fn parse_duplicate_names() {
    let content = r#"{
        "name": "test-package",
        "name": "test-package",
        "version": "1.0.0"
    }"#;
    let fields = parse(content).unwrap();
    assert_eq!(
        fields.header_fields.name.get_value(),
        Some(&"test-package".to_string())
    );
    assert!(fields.header_fields.name.is_valid());
    assert_eq!(
        fields.header_fields.version.get_value(),
        Some(&"1.0.0".to_string())
    );
    assert!(fields.header_fields.version.is_valid());
}

#[test]
fn expected_field_tracking() {
    let json = r#"{
        "name": "test",
        "version": 2,
        "exports": null
    }"#;
    let fields = parse(json).unwrap();

    assert!(fields.header_fields.name.is_valid());
    assert_eq!(fields.header_fields.name.value, "test");

    assert!(!fields.header_fields.version.is_valid());
    assert_eq!(fields.header_fields.version.value, "");

    assert_eq!(
        fields.path_fields.exports.json_value.value_type,
        JsonValueType::Null
    );

    assert!(!fields.path_fields.main.is_valid());
    assert!(!fields.path_fields.main.null);
    assert_eq!(fields.path_fields.main.value, "");
}

#[test]
fn exports_and_imports_navigation() {
    let json = r##"{
        "imports": {
            "#foo": {
                "import": "./foo.ts"
            }
        },
        "exports": {
            ".": {
                "import": "./test.ts",
                "default": "./test.ts"
            },
            "./test": [
                "./test1.ts",
                "./test2.ts",
                null
            ],
            "./null": null
        }
    }"##;
    let fields = parse(json).unwrap();

    let exports = &fields.path_fields.exports;
    let imports = &fields.path_fields.imports;

    assert!(exports.is_subpaths());
    assert_eq!(exports.json_value.as_object().len(), 3);

    let dot = exports.json_value.get(".").unwrap();
    let dot_eoi = ExportsOrImports {
        json_value: dot.clone(),
        object_kind: ObjectKind::Unknown,
    };
    assert!(dot_eoi.is_conditions());
    assert_eq!(dot.get("import").unwrap().value_type, JsonValueType::String);

    let test_arr = exports.json_value.get("./test").unwrap();
    assert_eq!(test_arr.value_type, JsonValueType::Array);
    assert_eq!(test_arr.as_array()[2].value_type, JsonValueType::Null);

    assert_eq!(
        exports.json_value.get("./null").unwrap().value_type,
        JsonValueType::Null
    );

    assert!(imports.is_imports());
    assert_eq!(imports.json_value.as_object().len(), 1);
    let foo = imports.json_value.get("#foo").unwrap();
    let foo_eoi = ExportsOrImports {
        json_value: foo.clone(),
        object_kind: ObjectKind::Unknown,
    };
    assert!(foo_eoi.is_conditions());
    assert_eq!(foo.get("import").unwrap().value_type, JsonValueType::String);
}

#[test]
fn json_value_types() {
    let json = r#"{
        "private": true,
        "false": false,
        "name": "test",
        "version": 2,
        "exports": {
            ".": {
                "import": "./test.ts",
                "default": "./test.ts"
            },
            "./test": [
                "./test1.ts",
                "./test2.ts",
                null
            ],
            "./null": null
        },
        "imports": null
    }"#;
    let raw: serde_json::Value = serde_json::from_str(json).unwrap();
    let obj = raw.as_object().unwrap();
    let to_jv = |k: &str| -> JsonValue { obj.get(k).unwrap().clone().into() };

    let private = to_jv("private");
    let false_val = to_jv("false");
    let name = to_jv("name");
    let version = to_jv("version");
    let exports = to_jv("exports");
    let imports = to_jv("imports");
    let not_present = JsonValue::default();

    assert_eq!(private.value_type, JsonValueType::Boolean);
    assert_eq!(private.bool_value, Some(true));

    assert_eq!(false_val.value_type, JsonValueType::Boolean);
    assert_eq!(false_val.bool_value, Some(false));

    assert_eq!(name.value_type, JsonValueType::String);
    assert_eq!(name.as_string(), "test");

    assert_eq!(version.value_type, JsonValueType::Number);
    assert_eq!(version.number_value, Some(2.0));

    assert_eq!(exports.value_type, JsonValueType::Object);
    assert_eq!(exports.as_object().len(), 3);

    let dot = exports.get(".").unwrap();
    assert_eq!(dot.value_type, JsonValueType::Object);
    assert_eq!(dot.get("import").unwrap().as_string(), "./test.ts");

    let test_arr = exports.get("./test").unwrap();
    assert_eq!(test_arr.value_type, JsonValueType::Array);
    assert_eq!(test_arr.as_array().len(), 3);
    assert_eq!(test_arr.as_array()[0].as_string(), "./test1.ts");
    assert_eq!(test_arr.as_array()[1].as_string(), "./test2.ts");
    assert_eq!(test_arr.as_array()[2].value_type, JsonValueType::Null);

    assert_eq!(
        exports.get("./null").unwrap().value_type,
        JsonValueType::Null
    );

    assert_eq!(imports.value_type, JsonValueType::Null);

    assert_eq!(not_present.value_type, JsonValueType::NotPresent);
}
