#![allow(unused_imports)]

use super::*;

pub(crate) fn import_resolution_mode_override(
    import_node: &Arc<ast::Node>,
) -> crate::core::compiler_options::ModuleKind {
    use crate::core::compiler_options::ModuleKind;
    let Some(decl) = import_node.parent.as_ref() else {
        return ModuleKind::None;
    };
    let (attributes, type_only) = match &decl.data {
        ast::NodeData::ImportDeclaration(d) => {
            let type_only = d.import_clause.as_ref().is_some_and(|c| {
                matches!(&c.data, ast::NodeData::ImportClause(ic)
                    if ic.phase_modifier == Some(ast::SyntaxKind::TypeKeyword))
            });
            (d.attributes.as_ref(), type_only)
        }
        ast::NodeData::ExportDeclaration(d) => (d.attributes.as_ref(), d.is_type_only),
        _ => return ModuleKind::None,
    };
    let Some(attrs) = attributes else {
        return ModuleKind::None;
    };
    if !type_only {
        return ModuleKind::None;
    }
    let ast::NodeData::ImportAttributes(data) = &attrs.data else {
        return ModuleKind::None;
    };
    if data.attributes.len() != 1 {
        return ModuleKind::None;
    }
    let ast::NodeData::ImportAttribute(attr) = &data.attributes.nodes[0].data else {
        return ModuleKind::None;
    };
    if attr.name.text() != "resolution-mode" {
        return ModuleKind::None;
    }
    match attr.value.text() {
        "import" => ModuleKind::ESNext,
        "require" => ModuleKind::CommonJS,
        _ => ModuleKind::None,
    }
}

pub fn is_external_library_file(file_name: &str) -> bool {
    file_name.contains("/node_modules/") || file_name.contains("\\node_modules\\")
}

pub(crate) fn is_plain_js_file(file: &SourceFile, check_js: Tristate) -> bool {
    matches!(file.script_kind, ScriptKind::Js | ScriptKind::Jsx) && check_js.is_unknown()
}

pub(crate) const PLAIN_JS_ERROR_CODES: &[i32] = &[
    2451, 2528, 2753, 2752, 1262, 1214, 1359, 18012, 1102, 1210, 1215, 1100, 1344, 1101, 1105,
    1116, 1211, 1248, 1171, 1104, 1115, 1113, 1258, 1255, 1182, 1054, 2501, 2566, 1186, 2462, 1048,
    1014, 1013, 18041, 1053, 1049, 1474, 1193, 1473, 1191, 1162, 1325, 2803, 2492, 1197, 18036,
    1174, 18006, 1312, 1114, 1450, 18038, 17000, 17001, 18007, 2633, 1107, 1200, 1184, 1091, 1188,
    18016, 1451, 18013, 1358, 1106, 1189, 1190, 1009, 1123, 5076, 1005, 17012, 1097, 1030, 1089,
    1044, 1090, 1031, 1042, 1029, 1156, 1155, 1172, 2480, 1341, 1368, 1308, 2852, 1111, 2839,
];

pub(crate) fn should_skip_js_file(file_name: &str, allow_js: bool) -> bool {
    if allow_js || !is_external_library_file(file_name) {
        return false;
    }
    matches!(
        script_kind_from_file_name(file_name),
        crate::ast::ScriptKind::Js | crate::ast::ScriptKind::Jsx
    )
}

pub(crate) fn read_and_parse(
    file_name: &str,
    host: &dyn CompilerHost,
) -> Result<(Arc<SourceFile>, Vec<crate::parser::ParserDiagnostic>), String> {
    let text = host
        .fs()
        .read_file(file_name)
        .ok_or_else(|| format!("Cannot read file '{file_name}'."))?;
    read_and_parse_text(file_name, text)
}

pub(crate) fn cached_parse(
    file_name: &str,
    text: &str,
) -> (Arc<SourceFile>, Vec<crate::parser::ParserDiagnostic>) {
    static CACHE: std::sync::OnceLock<
        Mutex<HashMap<(String, u64), (Arc<SourceFile>, Vec<crate::parser::ParserDiagnostic>)>>,
    > = std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    let key = (file_name.to_string(), hasher.finish());
    if let Some(hit) = cache.lock().unwrap().get(&key) {
        return (Arc::clone(&hit.0), hit.1.clone());
    }
    let (file, diags) =
        Parser::parse_source_file_text_with_diagnostics(file_name, text.to_string());
    let file = Arc::new(file);
    cache
        .lock()
        .unwrap()
        .insert(key, (Arc::clone(&file), diags.clone()));
    (file, diags)
}

pub(crate) fn read_and_parse_text(
    file_name: &str,
    text: String,
) -> Result<(Arc<SourceFile>, Vec<crate::parser::ParserDiagnostic>), String> {
    let (file, diags) = cached_parse(file_name, &text);
    Ok((file, diags))
}

pub(crate) fn load_source_file(
    file_name: &str,
    host: &dyn CompilerHost,
    source_files: &mut Vec<Arc<SourceFile>>,
    by_name: &mut HashMap<String, Arc<SourceFile>>,
    diagnostics: &mut Vec<Arc<Diagnostic>>,
    allow_js: bool,
) -> Option<Arc<SourceFile>> {
    let normalized = tspath::normalize_path(file_name);
    if let Some(existing) = by_name.get(&normalized) {
        return Some(Arc::clone(existing));
    }

    if should_skip_js_file(&normalized, allow_js) {
        return None;
    }

    let (file, parse_diags) = match read_and_parse(&normalized, host) {
        Ok(result) => result,
        Err(msg) => {
            diagnostics.push(Arc::new(file_error_diagnostic(&normalized, &msg)));
            return None;
        }
    };

    for pd in &parse_diags {
        diagnostics.push(Arc::new(parser_diagnostic_to_diagnostic(
            Arc::clone(&file),
            pd,
        )));
    }

    by_name.insert(normalized.clone(), Arc::clone(&file));
    source_files.push(Arc::clone(&file));
    Some(file)
}

pub(crate) fn load_source_file_with_references(
    file_name: &str,
    host: &dyn CompilerHost,
    source_files: &mut Vec<Arc<SourceFile>>,
    by_name: &mut HashMap<String, Arc<SourceFile>>,
    diagnostics: &mut Vec<Arc<Diagnostic>>,
    allow_js: bool,
) {
    let normalized = tspath::normalize_path(file_name);
    if by_name.contains_key(&normalized) {
        return;
    }

    if should_skip_js_file(&normalized, allow_js) {
        return;
    }

    let (file, parse_diags) = match read_and_parse(&normalized, host) {
        Ok(result) => result,
        Err(msg) => {
            diagnostics.push(Arc::new(file_error_diagnostic(&normalized, &msg)));
            return;
        }
    };

    for pd in &parse_diags {
        diagnostics.push(Arc::new(parser_diagnostic_to_diagnostic(
            Arc::clone(&file),
            pd,
        )));
    }

    by_name.insert(normalized.clone(), Arc::clone(&file));

    let text = file.text.as_str();
    let refs = extract_reference_path_directives(text, &normalized);
    for ref_path in &refs {
        load_source_file_with_references(
            ref_path,
            host,
            source_files,
            by_name,
            diagnostics,
            allow_js,
        );
    }

    source_files.push(file);
}

pub(crate) fn extract_reference_path_directives(text: &str, containing_file: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let base_dir = tspath::get_directory_path(containing_file);
    for line in text.lines() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("///") else {
            continue;
        };

        if !rest.trim_start().starts_with("<reference") {
            continue;
        }
        if let Some(start) = rest.find("path=\"") {
            let after = &rest[start + 6..];
            if let Some(end) = after.find('"') {
                let path = &after[..end];
                let resolved = if tspath::is_rooted_disk_path(path) {
                    tspath::normalize_path(path)
                } else {
                    tspath::normalize_path(&tspath::combine_paths(&base_dir, &[path]))
                };
                refs.push(resolved);
            }
        } else if let Some(start) = rest.find("path='") {
            let after = &rest[start + 6..];
            if let Some(end) = after.find('\'') {
                let path = &after[..end];
                let resolved = if tspath::is_rooted_disk_path(path) {
                    tspath::normalize_path(path)
                } else {
                    tspath::normalize_path(&tspath::combine_paths(&base_dir, &[path]))
                };
                refs.push(resolved);
            }
        }
    }
    refs
}

pub(crate) struct ReferenceTypesDirective {
    pub(crate) name: String,
    pub(crate) mode_value: Option<String>,
    #[allow(dead_code)]
    pub(crate) mode_value_range: (usize, usize),

    pub(crate) types_value_range: (usize, usize),
}
