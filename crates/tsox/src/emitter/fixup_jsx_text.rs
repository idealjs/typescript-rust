#![allow(unused_imports)]

use super::*;

pub(crate) fn fixup_jsx_text(text: &str) -> String {
    let decoded = decode_jsx_entities(text);
    if !decoded.contains('\n') {
        return decoded;
    }
    let lines: Vec<&str> = decoded.split('\n').collect();
    let n = lines.len();
    let mut parts: Vec<String> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = if i == 0 {
            line.trim_end()
        } else if i == n - 1 {
            line.trim_start()
        } else {
            line.trim()
        };
        if !trimmed.is_empty() {
            parts.push(trimmed.to_string());
        }
    }
    parts.join(" ")
}

pub(crate) fn decode_jsx_entities(text: &str) -> String {
    if !text.contains('&') {
        return text.to_string();
    }
    let mut result = text.to_string();
    for (entity, replacement) in &[
        ("&amp;", "&"),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&apos;", "'"),
        ("&nbsp;", "\u{00A0}"),
    ] {
        result = result.replace(entity, replacement);
    }
    result
}

pub(crate) fn escape_js_string(text: &str) -> String {
    let mut result = String::new();
    for c in text.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }
    result
}

pub(crate) fn build_jsx_import(
    usage: &JsxRuntimeUsage,
    import_source: &str,
    commonjs: bool,
) -> String {
    let mut specs: Vec<&str> = Vec::new();
    let mut bindings: Vec<&str> = Vec::new();
    if usage.used_fragment {
        specs.push("Fragment as _Fragment");
        bindings.push("Fragment: _Fragment");
    }
    if usage.used_jsx {
        specs.push("jsx as _jsx");
        bindings.push("jsx: _jsx");
    }
    if usage.used_jsxs {
        specs.push("jsxs as _jsxs");
        bindings.push("jsxs: _jsxs");
    }
    let runtime = format!("{}/jsx-runtime", import_source);
    if commonjs {
        format!(
            "const {{ {} }} = require(\"{}\");\n",
            bindings.join(", "),
            runtime
        )
    } else {
        format!("import {{ {} }} from \"{}\";\n", specs.join(", "), runtime)
    }
}

pub fn emit_program(
    source_files: &[Arc<SourceFile>],
    options: &CompilerOptions,
    fs: &dyn FS,
    write_file: &dyn Fn(&str, &str) -> std::io::Result<()>,
) -> EmitResult {
    let common_source_directory = compute_program_common_source_directory(source_files, options);
    let mut result = EmitResult::default();
    for source_file in source_files {
        let file_result = emit_source_file_with_common_dir(
            source_file,
            options,
            fs,
            &common_source_directory,
            write_file,
        );
        result.emitted_files.extend(file_result.emitted_files);
        result.diagnostics.extend(file_result.diagnostics);
        if file_result.emit_skipped {
            result.emit_skipped = true;
        }
    }
    result
}

pub fn compute_program_common_source_directory(
    source_files: &[Arc<SourceFile>],
    options: &CompilerOptions,
) -> String {
    let common_dir = if !options.root_dir.is_empty() {
        options.root_dir.clone()
    } else if !options.config_file_path.is_empty() {
        tspath::get_directory_path(&options.config_file_path)
    } else {
        compute_common_source_directory_of_filenames(
            &source_files
                .iter()
                .map(|sf| sf.file_name.clone())
                .collect::<Vec<_>>(),
        )
    };
    if common_dir.is_empty() {
        common_dir
    } else {
        tspath::ensure_trailing_directory_separator(&common_dir)
    }
}

pub(crate) fn compute_common_source_directory_of_filenames(file_names: &[String]) -> String {
    let mut common_components: Option<Vec<String>> = None;
    for file_name in file_names {
        let mut components = tspath::get_path_components(file_name, "");

        components.pop();
        match &mut common_components {
            None => {
                common_components = Some(components);
            }
            Some(common) => {
                let n = std::cmp::min(common.len(), components.len());
                let mut last_match = 0;
                for i in 0..n {
                    if common[i] != components[i] {
                        break;
                    }
                    last_match = i + 1;
                }
                common.truncate(last_match);
            }
        }
    }
    match common_components {
        Some(c) if !c.is_empty() => tspath::get_path_from_path_components(&c),
        _ => String::new(),
    }
}
