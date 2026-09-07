#![allow(unused_imports)]

use super::*;

pub(crate) fn show_config(sys: &dyn System, config: &ParsedCommandLine) {
    use crate::json::Value;
    use crate::tsoptions as opts;

    let options = &config.compiler_options;
    let mut map = crate::json::Map::new();

    insert_enum_options(&mut map, options);
    insert_path_and_string_options(&mut map, options, &config.config_file_name);
    insert_list_options(&mut map, options);
    insert_bool_options(&mut map, options);

    let mut top = crate::json::Map::new();
    if !map.is_empty() {
        top.insert("compilerOptions".to_string(), Value::Object(map));
    }
    if config.has_files_spec {
        top.insert(
            "files".to_string(),
            Value::Array(
                config
                    .files_spec
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }
    if config.has_include_spec {
        top.insert(
            "include".to_string(),
            Value::Array(
                config
                    .include
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }
    if config.has_exclude_spec {
        top.insert(
            "exclude".to_string(),
            Value::Array(
                config
                    .exclude
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect(),
            ),
        );
    }
    if !config.references.is_empty() {
        let refs: Vec<Value> = config
            .references
            .iter()
            .map(|r| {
                let mut obj = crate::json::Map::new();
                obj.insert("path".to_string(), Value::String(r.original_path.clone()));
                if r.circular {
                    obj.insert("circular".to_string(), Value::Bool(true));
                }
                Value::Object(obj)
            })
            .collect();
        top.insert("references".to_string(), Value::Array(refs));
    }
    if config.compile_on_save == Some(true) {
        top.insert("compileOnSave".to_string(), Value::Bool(true));
    }

    let json = Value::Object(top);
    let mut writer = sys.writer();
    let _ = crate::json::marshal_indent_write(&mut writer, &json, "    ");
    let _ = writeln!(writer);
}
