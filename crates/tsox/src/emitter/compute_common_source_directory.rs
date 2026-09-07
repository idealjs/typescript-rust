#![allow(unused_imports)]

use super::*;

pub(crate) fn compute_common_source_directory(options: &CompilerOptions) -> String {
    let common_dir = if !options.root_dir.is_empty() {
        options.root_dir.clone()
    } else if !options.config_file_path.is_empty() {
        tspath::get_directory_path(&options.config_file_path)
    } else {
        return String::new();
    };
    tspath::ensure_trailing_directory_separator(&common_dir)
}

pub(crate) fn get_source_file_path_in_new_dir(
    file_name: &str,
    new_dir_path: &str,
    common_source_directory: &str,
) -> String {
    if common_source_directory.is_empty() {
        return tspath::combine_paths(
            new_dir_path,
            &[tspath::get_base_file_name(file_name).as_str()],
        );
    }

    let common_with_sep = tspath::ensure_trailing_directory_separator(common_source_directory);
    let normalized_file = tspath::normalize_slashes(file_name);
    if let Some(stripped) = normalized_file.strip_prefix(&common_with_sep) {
        return tspath::combine_paths(new_dir_path, &[stripped]);
    }

    if normalized_file == common_with_sep.trim_end_matches('/') {
        return new_dir_path.to_string();
    }

    let abs_file = tspath::get_normalized_absolute_path(file_name, "");
    let abs_common = tspath::get_normalized_absolute_path(&common_with_sep, "");
    let abs_common_with_sep = tspath::ensure_trailing_directory_separator(&abs_common);
    if let Some(stripped) = abs_file.strip_prefix(&abs_common_with_sep) {
        return tspath::combine_paths(new_dir_path, &[stripped]);
    }

    tspath::combine_paths(
        new_dir_path,
        &[tspath::get_base_file_name(file_name).as_str()],
    )
}

pub(crate) fn get_output_extension(file_name: &str) -> &'static str {
    if tspath::file_extension_is(file_name, ".json") {
        return ".json";
    }
    if tspath::file_extension_is_one_of(file_name, &[".mts", ".mjs"]) {
        return ".mjs";
    }
    if tspath::file_extension_is_one_of(file_name, &[".cts", ".cjs"]) {
        return ".cjs";
    }
    ".js"
}

pub(crate) fn emit_js_text(source_file: &SourceFile, options: &CompilerOptions) -> String {
    let mut output = String::new();
    emit_js_text_inner(source_file, options, &mut output);

    output = rewrite_import_extensions(&output);

    output = add_implicit_semicolons(&output);

    if options.remove_comments.is_true() {
        while output.starts_with(|c: char| c == ' ' || c == '\t' || c == '\n' || c == '\r') {
            output.remove(0);
        }
    }
    output
}

pub(crate) fn emit_js_text_tracked(
    source_file: &SourceFile,
    options: &CompilerOptions,
    generator: &mut Generator,
    source_index: SourceIndex,
) -> String {
    let source = &source_file.text;
    let source_line_starts = compute_line_starts(source);

    let mut tracker = SourceMapTracker::new(source);
    emit_js_text_inner(source_file, options, &mut tracker);
    let (text, src_offsets) = tracker.finish();

    let (text, src_offsets) = rewrite_import_extensions_tracked(&text, &src_offsets);
    let (js_text, src_offsets) = normalize_js_output_tracked(&text, &src_offsets);

    generate_source_map_from_offsets(
        generator,
        source_index,
        &js_text,
        &src_offsets,
        source,
        &source_line_starts,
        source_file,
    );

    js_text
}

pub(crate) fn emit_js_text_inner<S: EmitSink>(
    source_file: &SourceFile,
    options: &CompilerOptions,
    sink: &mut S,
) {
    let source = &source_file.text;
    let statements = match &source_file.node.data {
        NodeData::SourceFile(d) => &d.statements,
        _ => {
            sink.emit_source(source, 0, source.len());
            return;
        }
    };

    let comment_cuts: Vec<(usize, usize)> = if options.remove_comments.is_true() {
        collect_all_comment_ranges(source)
    } else {
        Vec::new()
    };

    let replacements: Vec<(usize, usize, &'static str)> = if needs_es5_downlevel(options) {
        collect_es5_replacements(&statements.nodes)
    } else {
        Vec::new()
    };

    let jsx_enabled = needs_jsx_transform(options, source_file);
    let mut jsx_usage = JsxRuntimeUsage::default();
    let jsx_replacements: Vec<(usize, usize, String)> = if jsx_enabled {
        collect_jsx_replacements(&statements.nodes, source, &mut jsx_usage)
    } else {
        Vec::new()
    };

    let mut all_replacements: Vec<(usize, usize, &str, Option<usize>)> = Vec::new();
    for &(s, e, r) in &replacements {
        all_replacements.push((s, e, r, None));
    }
    for (s, e, r) in &jsx_replacements {
        all_replacements.push((*s, *e, r.as_str(), Some(*s)));
    }

    let commonjs = options.module == ModuleKind::CommonJS;

    let mut prev_end = 0usize;

    if commonjs {
        sink.emit_generated("\"use strict\";\n");
    }

    if !jsx_replacements.is_empty() {
        let import_source: &str = if options.jsx_import_source.is_empty() {
            "react"
        } else {
            &options.jsx_import_source
        };
        sink.emit_generated(&build_jsx_import(&jsx_usage, import_source, commonjs));
    }

    for stmt in statements.iter() {
        if is_type_only_statement(stmt) {
            prev_end = stmt.end();
            continue;
        }

        let mut modifier_cuts: Vec<(usize, usize)> = if commonjs {
            collect_export_modifier_cuts(stmt, source)
        } else {
            Vec::new()
        };

        collect_modifier_cuts(stmt, source, &mut modifier_cuts);

        let effective_cuts: Vec<(usize, usize)> = if modifier_cuts.is_empty() {
            comment_cuts.clone()
        } else {
            let mut cuts = comment_cuts.clone();
            cuts.extend(modifier_cuts);
            cuts
        };

        if stmt.pos() > prev_end {
            emit_text_range(
                source,
                prev_end,
                stmt.pos(),
                &effective_cuts,
                &all_replacements,
                sink,
            );
        }

        if commonjs {
            if let Some(transformed) = transform_commonjs_import(stmt, source) {
                prev_end = stmt.end();
                if !transformed.is_empty() {
                    sink.emit_generated(&transformed);
                    sink.emit_generated("\n");
                }
                continue;
            }

            if let Some(transformed) = transform_commonjs_export(stmt, source) {
                prev_end = stmt.end();
                if !transformed.is_empty() {
                    sink.emit_generated(&transformed);
                    sink.emit_generated("\n");
                }
                continue;
            }
        }

        emit_statement(stmt, source, &effective_cuts, &all_replacements, sink);
        prev_end = stmt.end();

        if commonjs {
            if let Some(append) = transform_commonjs_export_declaration(stmt, source) {
                sink.emit_generated(&append);
                sink.emit_generated("\n");
            }
        }
    }

    if prev_end < source.len() {
        emit_text_range(
            source,
            prev_end,
            source.len(),
            &comment_cuts,
            &all_replacements,
            sink,
        );
    }
}

pub(crate) fn generate_element_call(
    tag_name: &Arc<Node>,
    attributes: &Arc<Node>,
    children: Option<&Arc<NodeList>>,
    source: &str,
    usage: &mut JsxRuntimeUsage,
) -> String {
    let tag_str = tag_name_to_string(tag_name, source);

    let (props, key_arg) = attributes_to_props(attributes, source, usage);

    let children_prop = children.and_then(|c| convert_children(c, source, usage));

    let is_static = children.map_or(false, |c| is_static_children(c));

    let mut all_props = props;
    if let Some(children_str) = children_prop {
        all_props.push(format!("children: {}", children_str));
    }
    let props_str = if all_props.is_empty() {
        "{}".to_string()
    } else {
        format!("{{ {} }}", all_props.join(", "))
    };

    let callee = if is_static {
        usage.used_jsxs = true;
        "_jsxs"
    } else {
        usage.used_jsx = true;
        "_jsx"
    };

    let mut result = format!("{}({}, {}", callee, tag_str, props_str);
    if let Some(key) = key_arg {
        result.push_str(&format!(", {}", key));
    }
    result.push(')');
    result
}
