#![allow(unused_imports)]

use super::*;

pub(crate) fn import_resolution_mode_override(
    import_node: &Arc<tsox_frontend::ast::Node>,
) -> tsox_core::core::compiler_options::ModuleKind {
    use tsox_core::core::compiler_options::ModuleKind;
    let Some(decl) = import_node.parent() else {
        return ModuleKind::None;
    };
    let (attributes, type_only) = match &decl.data {
        tsox_frontend::ast::NodeData::ImportDeclaration(d) => {
            let type_only = d.import_clause.as_ref().is_some_and(|c| {
                matches!(&c.data, tsox_frontend::ast::NodeData::ImportClause(ic)
                    if ic.phase_modifier == Some(tsox_frontend::ast::SyntaxKind::TypeKeyword))
            });
            (d.attributes.as_ref(), type_only)
        }
        tsox_frontend::ast::NodeData::ExportDeclaration(d) => {
            (d.attributes.as_ref(), d.is_type_only)
        }
        _ => return ModuleKind::None,
    };
    let Some(attrs) = attributes else {
        return ModuleKind::None;
    };
    if !type_only {
        return ModuleKind::None;
    }
    let tsox_frontend::ast::NodeData::ImportAttributes(data) = &attrs.data else {
        return ModuleKind::None;
    };
    if data.attributes.len() != 1 {
        return ModuleKind::None;
    }
    let tsox_frontend::ast::NodeData::ImportAttribute(attr) = &data.attributes.nodes[0].data else {
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
        tsox_frontend::ast::ScriptKind::Js | tsox_frontend::ast::ScriptKind::Jsx
    )
}

pub(crate) fn read_and_parse(
    file_name: &str,
    host: &dyn CompilerHost,
) -> Result<
    (
        Arc<SourceFile>,
        Vec<tsox_frontend::parser::ParserDiagnostic>,
    ),
    String,
> {
    let text = host
        .fs()
        .read_file(file_name)
        .ok_or_else(|| format!("Cannot read file '{file_name}'."))?;
    read_and_parse_text(file_name, text)
}

pub(crate) fn cached_parse(
    file_name: &str,
    text: &str,
) -> (
    Arc<SourceFile>,
    Vec<tsox_frontend::parser::ParserDiagnostic>,
) {
    // 只缓存 bundled lib 文件：内容稳定、被每个 Program 重复解析，
    // 收益集中于此。用户/测试文件一律不缓存，否则每个唯一文件名都会
    // 永久钉住一棵 AST（fourslash 每用例默认文件名唯一，全量跑即缓慢
    // 泄漏至数十 GiB）。lib 文件版本替换与容量上限做双保险。
    if !tsox_checker::bundled::is_bundled(file_name) {
        let (file, diags) =
            Parser::parse_source_file_text_with_diagnostics(file_name, text.to_string());
        return (Arc::new(file), diags);
    }
    static CACHE: std::sync::OnceLock<
        Mutex<
            HashMap<
                String,
                (
                    Arc<SourceFile>,
                    Vec<tsox_frontend::parser::ParserDiagnostic>,
                ),
            >,
        >,
    > = std::sync::OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let key = file_name.to_string();
    {
        let map = cache.lock().unwrap();
        if let Some(hit) = map.get(&key) {
            if hit.0.text == text {
                return (Arc::clone(&hit.0), hit.1.clone());
            }
        }
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
) -> Result<
    (
        Arc<SourceFile>,
        Vec<tsox_frontend::parser::ParserDiagnostic>,
    ),
    String,
> {
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
    let normalized = tsox_core::tspath::normalize_path(file_name);
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
    let normalized = tsox_core::tspath::normalize_path(file_name);
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
    for ref_dir in &refs {
        if let Some((message, args)) =
            check_reference_path_loadable(host, &ref_dir.resolved, &ref_dir.raw, &normalized)
        {
            diagnostics.push(Arc::new(Diagnostic::new(
                Some(Arc::clone(&file)),
                TextRange::new(ref_dir.value_range.0, ref_dir.value_range.1),
                message,
                args,
            )));
            continue;
        }
        load_source_file_with_references(
            &ref_dir.resolved,
            host,
            source_files,
            by_name,
            diagnostics,
            allow_js,
        );
    }

    source_files.push(file);
}

pub(crate) struct ReferencePathDirective {
    pub(crate) resolved: String,
    pub(crate) raw: String,
    pub(crate) value_range: (usize, usize),
}

fn supported_reference_extensions() -> &'static [&'static str] {
    &[".ts", ".tsx", ".d.ts"]
}

fn has_supported_reference_extension(path: &str) -> bool {
    supported_reference_extensions()
        .iter()
        .any(|ext| path.ends_with(ext))
}

/// Go getSourceFileFromReference：按引用文本的扩展名形态决定可加载性与失败
/// 诊断；诊断统一定位在引用文件 directive 的路径串上，参数用原始引用文本
fn check_reference_path_loadable(
    host: &dyn CompilerHost,
    resolved: &str,
    raw: &str,
    containing_file: &str,
) -> Option<(
    tsox_core::diagnostics::Message,
    Vec<String>,
)> {
    use tsox_core::diagnostics::messages_generated as msg;
    let has_extension = resolved.rsplit('/').next().unwrap_or("").contains('.');
    let raw_normalized = raw.replace('\\', "/");
    if has_extension {
        if !has_supported_reference_extension(resolved) {
            if resolved.ends_with(".js") || resolved.ends_with(".jsx") {
                return Some((msg::FILE_0_IS_A_JAVASCRIPT_FILE_DID_YOU_MEAN_TO_ENABLE_THE_ALLOWJS_OPTION, vec![raw_normalized]));
            }
            let joined = supported_reference_extensions()
                .iter()
                .map(|e| format!("'{e}'"))
                .collect::<Vec<_>>()
                .join(", ");
            return Some((msg::FILE_0_HAS_AN_UNSUPPORTED_EXTENSION_THE_ONLY_SUPPORTED_EXTENSIONS_ARE_1, vec![raw_normalized, joined]));
        }
        if !host.fs().file_exists(resolved) {
            return Some((tsox_core::diagnostics::FILE_0_NOT_FOUND, vec![raw_normalized]));
        }
        let canonical = |p: &str| {
            p.rsplit('/')
                .next()
                .unwrap_or("")
                .to_ascii_lowercase()
        };
        if canonical(resolved) == canonical(containing_file) {
            return Some((msg::A_FILE_CANNOT_HAVE_A_REFERENCE_TO_ITSELF, Vec::new()));
        }
        return None;
    }
    for ext in supported_reference_extensions() {
        let candidate = format!("{resolved}{ext}");
        if host.fs().file_exists(&candidate) {
            return None;
        }
    }
    let joined = supported_reference_extensions()
        .iter()
        .map(|e| format!("'{e}'"))
        .collect::<Vec<_>>()
        .join(", ");
    Some((msg::COULD_NOT_RESOLVE_THE_PATH_0_WITH_THE_EXTENSIONS_COLON_1, vec![raw_normalized, joined]))
}

pub(crate) fn extract_reference_path_directives(
    text: &str,
    containing_file: &str,
) -> Vec<ReferencePathDirective> {
    let mut refs = Vec::new();
    let base_dir = tsox_core::tspath::get_directory_path(containing_file);
    let mut line_start = 0usize;
    for raw_line in text.split('\n') {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let trimmed = line.trim_start();
        let leading = line.len() - trimmed.len();
        let Some(rest) = trimmed.strip_prefix("///") else {
            line_start += raw_line.len() + 1;
            continue;
        };
        if !rest.trim_start().starts_with("<reference") {
            line_start += raw_line.len() + 1;
            continue;
        }
        let Some(prefix_len) = ["path=\"", "path='"]
            .iter()
            .find_map(|m| rest.find(*m).map(|i| (i, m.len()))) else
        {
            line_start += raw_line.len() + 1;
            continue;
        };
        let (start, marker_len) = prefix_len;
        let after = &rest[start + marker_len..];
        let quote = rest[start + 5..].chars().next().unwrap_or('"');
        if let Some(end) = after.find(quote) {
            let path = &after[..end];
            let resolved = if tsox_core::tspath::is_rooted_disk_path(path) {
                tsox_core::tspath::normalize_path(path)
            } else {
                tsox_core::tspath::normalize_path(&tsox_core::tspath::combine_paths(
                    &base_dir,
                    &[path],
                ))
            };
            let value_start = line_start + leading + 3 + start + marker_len;
            refs.push(ReferencePathDirective {
                resolved,
                raw: path.to_string(),
                value_range: (value_start, value_start + end),
            });
        }
        line_start += raw_line.len() + 1;
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
