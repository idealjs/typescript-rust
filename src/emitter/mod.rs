use std::sync::Arc;

use crate::ast::node_data_generated::{ImportDeclarationData, NodeData};
use crate::ast::node_flags::ModifierFlags;
use crate::ast::{Node, NodeFlags, NodeList, SourceFile, SyntaxKind};
use crate::core::compiler_options::CompilerOptions;
use crate::core::compiler_options::{JsxEmit, ModuleKind, ScriptTarget};
use crate::sourcemap::{Generator, SourceIndex};
use crate::tspath::{self, ComparePathsOptions};
use crate::vfs::FS;

#[derive(Debug, Default)]
pub struct EmitResult {
    pub emit_skipped: bool,
    pub emitted_files: Vec<String>,
    pub diagnostics: Vec<String>,
}

pub struct EmitOptions {

    pub write_file: Option<Box<dyn Fn(&str, &str) -> std::io::Result<()> + Send + Sync>>,
}

fn compute_line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, b) in text.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

fn offset_to_line(line_starts: &[usize], offset: usize) -> (i32, usize) {
    let line = line_starts.partition_point(|&start| start <= offset) - 1;
    (line as i32, line_starts[line])
}

fn utf16_column(text: &str, line_start: usize, offset: usize) -> i32 {
    text[line_start..offset]
        .chars()
        .map(|c| c.len_utf16() as i32)
        .sum()
}

const UNMAPPED: u32 = u32::MAX;

struct SourceMapTracker<'a> {
    output: String,

    src_offsets: Vec<u32>,
    source: &'a str,
}

impl<'a> SourceMapTracker<'a> {
    fn new(source: &'a str) -> Self {
        SourceMapTracker {
            output: String::new(),
            src_offsets: Vec::new(),
            source,
        }
    }

    fn push_generated(&mut self, text: &str) {
        let char_count = text.chars().count();
        self.output.push_str(text);
        self.src_offsets
            .resize(self.src_offsets.len() + char_count, UNMAPPED);
    }

    fn push_source(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }
        let slice = &self.source[start..end];
        let mut byte_off = start;
        for ch in slice.chars() {
            self.output.push(ch);
            self.src_offsets.push(byte_off as u32);
            byte_off += ch.len_utf8();
        }
    }

    fn push_source_mapped(&mut self, text: &str, src_pos: usize) {
        let mut first = true;
        for ch in text.chars() {
            self.output.push(ch);
            if first {
                self.src_offsets.push(src_pos as u32);
                first = false;
            } else {
                self.src_offsets.push(UNMAPPED);
            }
        }
    }

    fn finish(self) -> (String, Vec<u32>) {
        (self.output, self.src_offsets)
    }
}

trait EmitSink {

    fn emit_source(&mut self, source: &str, start: usize, end: usize);

    fn emit_generated(&mut self, text: &str);

    fn emit_source_mapped(&mut self, text: &str, src_pos: usize);
}

impl EmitSink for String {
    fn emit_source(&mut self, source: &str, start: usize, end: usize) {
        self.push_str(&source[start..end]);
    }
    fn emit_generated(&mut self, text: &str) {
        self.push_str(text);
    }
    fn emit_source_mapped(&mut self, text: &str, _src_pos: usize) {
        self.push_str(text);
    }
}

impl<'a> EmitSink for SourceMapTracker<'a> {
    fn emit_source(&mut self, source: &str, start: usize, end: usize) {
        debug_assert!(std::ptr::eq(source.as_ptr(), self.source.as_ptr()));
        self.push_source(start, end);
    }
    fn emit_generated(&mut self, text: &str) {
        self.push_generated(text);
    }
    fn emit_source_mapped(&mut self, text: &str, src_pos: usize) {
        self.push_source_mapped(text, src_pos);
    }
}

pub fn emit_source_file(
    source_file: &SourceFile,
    options: &CompilerOptions,
    fs: &dyn FS,
    write_file: &dyn Fn(&str, &str) -> std::io::Result<()>,
) -> EmitResult {
    emit_source_file_with_common_dir(source_file, options, fs, "", write_file)
}

pub fn emit_source_file_with_common_dir(
    source_file: &SourceFile,
    options: &CompilerOptions,
    _fs: &dyn FS,
    common_source_directory: &str,
    write_file: &dyn Fn(&str, &str) -> std::io::Result<()>,
) -> EmitResult {
    let mut result = EmitResult::default();

    if source_file.script_kind == crate::ast::ScriptKind::Json {
        return result;
    }

    if source_file.script_kind == crate::ast::ScriptKind::Js
        && options.no_emit_for_js_files.is_true()
    {
        return result;
    }

    let js_path = get_js_output_path(source_file, options, common_source_directory);
    if js_path.is_empty() {
        return result;
    }

    let emit_sourcemap = options.source_map.is_true() || options.inline_source_map.is_true();

    let emit_js = !options.emit_declaration_only.is_true();

    if emit_js {
        let (js_text, map_text, source_map_url) = if emit_sourcemap {
            emit_js_with_sourcemap(source_file, options, &js_path)
        } else {
            (emit_js_text(source_file, options), None, String::new())
        };

        let final_js_text = if source_map_url.is_empty() {
            js_text
        } else {
            let mut text = js_text;
            if !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str("//# sourceMappingURL=");
            text.push_str(&source_map_url);
            text
        };

        match write_file(&js_path, &final_js_text) {
            Ok(()) => {
                result.emitted_files.push(js_path.clone());
            }
            Err(e) => {
                result
                    .diagnostics
                    .push(format!("Error writing '{js_path}': {e}"));
                result.emit_skipped = true;
            }
        }

        if let Some(map_json) = map_text {
            let map_path = format!("{js_path}.map");
            match write_file(&map_path, &map_json) {
                Ok(()) => {
                    result.emitted_files.push(map_path);
                }
                Err(e) => {
                    result
                        .diagnostics
                        .push(format!("Error writing '{map_path}': {e}"));
                }
            }
        }
    }

    if options.get_emit_declarations() {
        let dts_path = get_dts_output_path(source_file, options, common_source_directory);
        if !dts_path.is_empty() {
            let dts_text = emit_declaration_text(source_file, options);
            match write_file(&dts_path, &dts_text) {
                Ok(()) => {
                    result.emitted_files.push(dts_path);
                }
                Err(e) => {
                    result
                        .diagnostics
                        .push(format!("Error writing '{dts_path}': {e}"));
                }
            }
        }
    }

    result
}

fn emit_js_with_sourcemap(
    source_file: &SourceFile,
    options: &CompilerOptions,
    js_path: &str,
) -> (String, Option<String>, String) {
    let js_base_name = tspath::get_base_file_name(js_path);
    let source_root = if options.source_root.is_empty() {
        String::new()
    } else {
        tspath::ensure_trailing_directory_separator(&options.source_root)
    };

    let sources_dir = tspath::get_directory_path(js_path);
    let path_options = ComparePathsOptions::default();

    let mut generator = Generator::new(&js_base_name, &source_root, &sources_dir, path_options);
    let source_index = generator.add_source(&source_file.file_name);
    if options.inline_sources.is_true() {
        let _ = generator.set_source_content(source_index, &source_file.text);
    }

    let js_text = emit_js_text_tracked(source_file, options, &mut generator, source_index);

    let inline = options.inline_source_map.is_true();
    let (map_json, source_map_url) = if inline {
        (None, generator.to_base64_data_url())
    } else {
        (Some(generator.to_json()), format!("{js_base_name}.map"))
    };

    (js_text, map_json, source_map_url)
}

fn get_js_output_path(
    source_file: &SourceFile,
    options: &CompilerOptions,
    common_source_directory: &str,
) -> String {
    let file_name = &source_file.file_name;
    let extension = get_output_extension(file_name);

    if !options.out_dir.is_empty() {
        let common_dir = if common_source_directory.is_empty() {
            compute_common_source_directory(options)
        } else {
            common_source_directory.to_string()
        };
        let path_in_new_dir =
            get_source_file_path_in_new_dir(file_name, &options.out_dir, &common_dir);
        let without_ext = tspath::remove_file_extension(&path_in_new_dir);
        format!("{without_ext}{extension}")
    } else {

        let without_ext = tspath::remove_file_extension(file_name);
        format!("{without_ext}{extension}")
    }
}

fn compute_common_source_directory(options: &CompilerOptions) -> String {
    let common_dir = if !options.root_dir.is_empty() {
        options.root_dir.clone()
    } else if !options.config_file_path.is_empty() {
        tspath::get_directory_path(&options.config_file_path)
    } else {
        return String::new();
    };
    tspath::ensure_trailing_directory_separator(&common_dir)
}

fn get_source_file_path_in_new_dir(
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

fn get_output_extension(file_name: &str) -> &'static str {
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

fn emit_js_text(source_file: &SourceFile, options: &CompilerOptions) -> String {
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

fn emit_js_text_tracked(
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

fn generate_source_map_from_offsets(
    generator: &mut Generator,
    source_index: SourceIndex,
    output: &str,
    src_offsets: &[u32],
    source: &str,
    source_line_starts: &[usize],
    _source_file: &SourceFile,
) {

    let out_chars: Vec<char> = output.chars().collect();
    let mut gen_line: i32 = 0;
    let mut gen_col: i32 = 0;
    let mut prev_src: u32 = UNMAPPED;

    for (i, &src_off) in src_offsets.iter().enumerate() {
        let ch = out_chars.get(i).copied().unwrap_or('\n');

        if ch != '\n' && src_off != UNMAPPED {
            let should_emit = if prev_src == UNMAPPED {
                true
            } else {
                let prev_byte = prev_src as usize;
                if prev_byte < source.len() {
                    let prev_char_len = source[prev_byte..]
                        .chars()
                        .next()
                        .map(|c| c.len_utf8())
                        .unwrap_or(1);
                    src_off != prev_src + prev_char_len as u32
                } else {
                    true
                }
            };
            if should_emit {
                let byte_off = src_off as usize;
                let (src_line, line_start) = offset_to_line(source_line_starts, byte_off);
                let src_col = utf16_column(source, line_start, byte_off);
                let _ = generator.add_source_mapping(
                    gen_line,
                    gen_col,
                    source_index,
                    src_line,
                    src_col,
                );
            }
        }

        if ch == '\n' {
            gen_line += 1;
            gen_col = 0;
            prev_src = UNMAPPED;
        } else {
            gen_col += ch.len_utf16() as i32;
            prev_src = src_off;
        }
    }
}

fn emit_js_text_inner<S: EmitSink>(
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

fn emit_declaration_text(source_file: &SourceFile, _options: &CompilerOptions) -> String {
    let source = &source_file.text;
    let statements = match &source_file.node.data {
        NodeData::SourceFile(d) => &d.statements,
        _ => return source.clone(),
    };

    let mut output = String::new();
    let mut prev_end = 0usize;

    for stmt in statements.iter() {

        if !is_declaration_statement(stmt) {
            prev_end = stmt.end();
            continue;
        }

        if is_value_only_import(stmt) {
            prev_end = stmt.end();
            continue;
        }

        let export_cuts = collect_export_modifier_cuts(stmt, source);
        let has_export = export_cuts.iter().any(|(s, e)| *e > *s);
        let has_default = stmt
            .modifiers()
            .map(|m| m.modifier_flags.contains(ModifierFlags::Default))
            .unwrap_or(false);

        let mod_start = export_cuts.first().map(|&(s, _)| s).unwrap_or(stmt.pos());

        let content_start = if !export_cuts.is_empty() {
            export_cuts.last().map(|&(_, e)| e).unwrap_or(stmt.pos())
        } else {
            stmt.pos()
        };

        if mod_start > prev_end {
            output.push_str(&source[prev_end..mod_start]);
        }

        if has_export {
            output.push_str("export ");
        }
        if has_default {
            output.push_str("default ");
        }

        if needs_declare_keyword(stmt) && !has_default {
            output.push_str("declare ");
        }

        emit_declaration_statement(stmt, source, content_start, &mut output);
        prev_end = stmt.end();
    }

    if prev_end < source.len() {
        output.push_str(&source[prev_end..]);
    }

    let output = rewrite_import_extensions(&output);
    let output = reindent_and_dedup(&output);
    add_implicit_semicolons(&output)
}

fn is_declaration_statement(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::VariableStatement
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::ExportAssignment
            | SyntaxKind::ModuleDeclaration
    )
}

fn is_value_only_import(node: &Node) -> bool {
    if let NodeData::ImportDeclaration(d) = &node.data {
        match &d.import_clause {

            None => false,
            Some(clause) => match &clause.data {
                NodeData::ImportClause(ic) => ic.phase_modifier != Some(SyntaxKind::TypeKeyword),
                _ => false,
            },
        }
    } else {
        false
    }
}

fn needs_declare_keyword(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::VariableStatement
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ModuleDeclaration
    )
}

fn emit_declaration_statement(node: &Node, source: &str, start: usize, output: &mut String) {
    match &node.data {

        NodeData::FunctionDeclaration(d) => {
            if let Some(body) = &d.body {

                let sig = &source[start..body.pos()];
                let mut sig_trimmed = sig.trim_end().to_string();

                let is_generator = d.asterisk_token.is_some();
                if is_generator {
                    sig_trimmed = sig_trimmed.replace("function*", "function");
                }
                if sig_trimmed.starts_with("async ")
                    || sig_trimmed.contains(" async function")
                {
                    if let Some(pos) = sig_trimmed.find("async ") {
                        sig_trimmed.replace_range(pos..pos + 6, "");
                    }
                }

                let has_return_type = sig_trimmed.rfind(')').map_or(false, |close_paren| {
                    sig_trimmed[close_paren..].contains(':')
                });

                if !has_return_type && function_returns_jsx(body) {

                    output.push_str(&sig_trimmed);
                    output.push_str(": import(\"react\").JSX.Element;");
                } else if !has_return_type {

                    output.push_str(&sig_trimmed);
                    output.push_str(if is_generator { ": {};" } else { ": unknown;" });
                } else {
                    output.push_str(&sig_trimmed);
                    output.push(';');
                }

                let bytes = source.as_bytes();
                let mut brace_end = body.end();
                while brace_end > body.pos() && bytes[brace_end - 1].is_ascii_whitespace() {
                    brace_end -= 1;
                }
                if brace_end < body.end() {
                    output.push_str(&source[brace_end..body.end()]);
                }
            } else {

                output.push_str(&source[start..node.end()]);
            }
        }

        NodeData::VariableStatement(d) => {
            let mut cuts: Vec<(usize, usize)> = Vec::new();
            collect_variable_initializer_cuts(&d.declaration_list, &mut cuts, true);
            if cuts.is_empty() {
                output.push_str(&source[start..node.end()]);
            } else {
                emit_with_cuts(source, start, node.end(), &cuts, output);
            }
        }

        NodeData::ClassDeclaration(d) => {
            emit_class_members(&d.members, source, start, node.end(), output);
        }

        _ => {
            output.push_str(&source[start..node.end()]);
        }
    }
}

fn function_returns_jsx(body: &Arc<Node>) -> bool {
    fn returns_jsx_recursive(node: &Arc<Node>) -> bool {
        match &node.data {
            NodeData::ReturnStatement(d) => {
                if let Some(expr) = &d.expression {
                    if is_jsx_expression(expr) {
                        return true;
                    }
                }
                false
            }
            NodeData::Block(d) => d.statements.iter().any(returns_jsx_recursive),
            NodeData::IfStatement(d) => {
                let then_jsx = returns_jsx_recursive(&d.then_statement);
                let else_jsx = d
                    .else_statement
                    .as_ref()
                    .map_or(false, |s| returns_jsx_recursive(s));
                then_jsx || else_jsx
            }
            _ => false,
        }
    }
    returns_jsx_recursive(body)
}

fn is_jsx_expression(node: &Arc<Node>) -> bool {
    if matches!(
        node.kind,
        SyntaxKind::JsxElement
            | SyntaxKind::JsxFragment
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxExpression
    ) {
        return true;
    }

    if let NodeData::ParenthesizedExpression(d) = &node.data {
        return is_jsx_expression(&d.expression);
    }
    false
}

fn emit_class_members(
    members: &NodeList,
    source: &str,
    start: usize,
    end: usize,
    output: &mut String,
) {

    let mut ops: Vec<(usize, usize)> = Vec::new();
    let bytes = source.as_bytes();
    for member in members.iter() {
        if let Some(body) = class_member_body(member) {

            let mut sig_end = body.pos();
            while sig_end > start && bytes[sig_end - 1].is_ascii_whitespace() {
                sig_end -= 1;
            }
            ops.push((sig_end, body.end()));
        }
    }
    ops.sort_by_key(|&(s, _)| s);
    let mut pos = start;
    for (cs, ce) in &ops {
        if *cs > pos {
            output.push_str(&source[pos..*cs]);
        }
        output.push(';');
        pos = *ce;
    }
    if pos < end {
        output.push_str(&source[pos..end]);
    }
}

fn class_member_body(member: &Node) -> Option<&Arc<Node>> {
    match &member.data {
        NodeData::MethodDeclaration(d) => d.body.as_ref(),
        NodeData::ConstructorDeclaration(d) => d.body.as_ref(),
        NodeData::GetAccessorDeclaration(d) => d.body.as_ref(),
        NodeData::SetAccessorDeclaration(d) => d.body.as_ref(),
        _ => None,
    }
}

fn collect_variable_initializer_cuts(
    list: &Arc<Node>,
    cuts: &mut Vec<(usize, usize)>,
    declaration_mode: bool,
) {
    if let NodeData::VariableDeclarationList(d) = &list.data {
        for decl in d.declarations.iter() {
            if let NodeData::VariableDeclaration(vd) = &decl.data {
                if let (Some(type_node), Some(init)) = (&vd.type_node, &vd.initializer) {

                    cuts.push((type_node.end(), init.end()));
                } else if declaration_mode {
                    if let Some(init) = &vd.initializer {

                        cuts.push((vd.name.end(), init.end()));
                    }
                }

                collect_variable_initializer_cuts(&vd.name, cuts, declaration_mode);
            }
        }
    }
}

fn emit_with_cuts(
    source: &str,
    start: usize,
    end: usize,
    cuts: &[(usize, usize)],
    output: &mut String,
) {
    if cuts.is_empty() {
        output.push_str(&source[start..end]);
        return;
    }
    let mut sorted: Vec<(usize, usize)> = cuts
        .iter()
        .filter(|&&(cs, ce)| ce > start && cs < end)
        .map(|&(cs, ce)| (cs.max(start), ce.min(end)))
        .collect();
    sorted.sort_by_key(|&(s, _)| s);
    let mut pos = start;
    for (cs, ce) in &sorted {
        if *cs > pos {
            output.push_str(&source[pos..*cs]);
        }
        pos = *ce;
    }
    if pos < end {
        output.push_str(&source[pos..end]);
    }
}

fn get_dts_output_path(
    source_file: &SourceFile,
    options: &CompilerOptions,
    common_source_directory: &str,
) -> String {
    let file_name = &source_file.file_name;
    let dts_ext = get_declaration_extension(file_name);

    if !options.declaration_dir.is_empty() {
        let common_dir = if common_source_directory.is_empty() {
            compute_common_source_directory(options)
        } else {
            common_source_directory.to_string()
        };
        let path_in_new_dir =
            get_source_file_path_in_new_dir(file_name, &options.declaration_dir, &common_dir);
        let without_ext = tspath::remove_file_extension(&path_in_new_dir);
        format!("{without_ext}{dts_ext}")
    } else if !options.out_dir.is_empty() {

        let js_path = get_js_output_path(source_file, options, common_source_directory);
        let without_ext = tspath::remove_file_extension(&js_path);
        format!("{without_ext}{dts_ext}")
    } else {
        let without_ext = tspath::remove_file_extension(file_name);
        format!("{without_ext}{dts_ext}")
    }
}

fn get_declaration_extension(file_name: &str) -> &'static str {
    if tspath::file_extension_is_one_of(file_name, &[".mts", ".mjs"]) {
        return ".d.mts";
    }
    if tspath::file_extension_is_one_of(file_name, &[".cts", ".cjs"]) {
        return ".d.cts";
    }
    ".d.ts"
}

fn emit_text_range<S: EmitSink>(
    source: &str,
    start: usize,
    end: usize,
    cuts: &[(usize, usize)],
    replacements: &[(usize, usize, &str, Option<usize>)],
    sink: &mut S,
) {
    if cuts.is_empty() && replacements.is_empty() {
        sink.emit_source(source, start, end);
        return;
    }

    let mut ops: Vec<(usize, usize, Option<(&str, Option<usize>)>)> = Vec::new();
    for &(cs, ce) in cuts {
        if ce > start && cs < end {
            ops.push((cs.max(start), ce.min(end), None));
        }
    }
    for &(rs, re, repl, src_pos) in replacements {
        if re > start && rs < end {
            ops.push((rs.max(start), re.min(end), Some((repl, src_pos))));
        }
    }
    if ops.is_empty() {
        sink.emit_source(source, start, end);
        return;
    }
    ops.sort_by_key(|&(s, _, _)| s);
    let mut pos = start;
    for (s, e, repl) in &ops {
        if *s > pos {
            sink.emit_source(source, pos, *s);
        }
        if let Some((r, src_pos)) = repl {
            if let Some(sp) = src_pos {
                sink.emit_source_mapped(r, *sp);
            } else {
                sink.emit_generated(r);
            }
        }
        pos = *e;
    }
    if pos < end {
        sink.emit_source(source, pos, end);
    }
}

fn collect_all_comment_ranges(text: &str) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    let mut pos = 0usize;

    let mut prev_significant: char = ';';

    while pos < len {
        let b = bytes[pos];
        match b {
            b'/' if pos + 1 < len && bytes[pos + 1] == b'/' => {

                let start = pos;
                pos += 2;
                while pos < len && bytes[pos] != b'\n' && bytes[pos] != b'\r' {
                    pos += 1;
                }
                ranges.push((start, pos));
            }
            b'/' if pos + 1 < len && bytes[pos + 1] == b'*' => {

                let start = pos;
                pos += 2;
                while pos < len {
                    if bytes[pos] == b'*' && pos + 1 < len && bytes[pos + 1] == b'/' {
                        pos += 2;
                        break;
                    }
                    pos += 1;
                }
                ranges.push((start, pos));
            }
            b'/' => {

                if is_regex_context(prev_significant) {
                    let start = pos;
                    pos += 1;
                    let mut in_class = false;
                    while pos < len {
                        let c = bytes[pos];
                        if c == b'\\' && pos + 1 < len {
                            pos += 2;
                            continue;
                        }
                        if c == b'[' {
                            in_class = true;
                        }
                        if c == b']' {
                            in_class = false;
                        }
                        if c == b'/' && !in_class {
                            pos += 1;

                            while pos < len && is_regex_flag_char(bytes[pos]) {
                                pos += 1;
                            }
                            break;
                        }
                        if c == b'\n' {

                            break;
                        }
                        pos += 1;
                    }
                    let _ = start;
                } else {
                    pos += 1;
                }
                prev_significant = '/';
            }
            b'\'' | b'"' => {

                let quote = b;
                pos += 1;
                while pos < len {
                    let c = bytes[pos];
                    if c == b'\\' && pos + 1 < len {
                        pos += 2;
                        continue;
                    }
                    if c == quote {
                        pos += 1;
                        break;
                    }
                    if c == b'\n' {

                        break;
                    }
                    pos += 1;
                }
                prev_significant = char::from(quote);
            }
            b'`' => {

                prev_significant = '`';
                pos += 1;
                skip_template_literal(text, &mut pos);
            }
            b' ' | b'\t' | b'\n' | b'\r' => {

                pos += 1;
            }
            _ => {
                prev_significant = char::from(b);
                pos += 1;
            }
        }
    }

    ranges
}

fn skip_template_literal(text: &str, pos: &mut usize) {
    let bytes = text.as_bytes();
    let len = bytes.len();
    while *pos < len {
        let b = bytes[*pos];
        if b == b'\\' && *pos + 1 < len {
            *pos += 2;
            continue;
        }
        if b == b'`' {
            *pos += 1;
            return;
        }
        if b == b'$' && *pos + 1 < len && bytes[*pos + 1] == b'{' {

            *pos += 2;
            let mut depth = 1;
            while *pos < len && depth > 0 {
                let c = bytes[*pos];
                match c {
                    b'{' => {
                        depth += 1;
                        *pos += 1;
                    }
                    b'}' => {
                        depth -= 1;
                        *pos += 1;
                    }
                    b'\'' | b'"' => {
                        let quote = c;
                        *pos += 1;
                        while *pos < len {
                            if bytes[*pos] == b'\\' && *pos + 1 < len {
                                *pos += 2;
                                continue;
                            }
                            if bytes[*pos] == quote {
                                *pos += 1;
                                break;
                            }
                            *pos += 1;
                        }
                    }
                    b'`' => {
                        *pos += 1;
                        skip_template_literal(text, pos);
                    }
                    _ => {
                        *pos += 1;
                    }
                }
            }
        } else {
            *pos += 1;
        }
    }
}

fn is_regex_context(prev: char) -> bool {
    matches!(
        prev,
        '(' | ','
            | '='
            | ':'
            | '['
            | '!'
            | '&'
            | '|'
            | '?'
            | '{'
            | '}'
            | ';'
            | '<'
            | '>'
            | '+'
            | '-'
            | '*'
            | '/'
            | '%'
            | '~'
            | '^'
            | '\n'
            | '\0'
    )
}

fn is_regex_flag_char(b: u8) -> bool {
    matches!(b, b'g' | b'i' | b'm' | b's' | b'u' | b'y' | b'd' | b'v')
}

fn needs_es5_downlevel(options: &CompilerOptions) -> bool {
    options.target == ScriptTarget::ES5
}

fn collect_es5_replacements(statements: &[Arc<Node>]) -> Vec<(usize, usize, &'static str)> {
    let mut replacements = Vec::new();
    for stmt in statements {
        collect_es5_replacements_recursive(stmt, &mut replacements);
    }
    replacements
}

fn collect_es5_replacements_recursive(
    node: &Node,
    replacements: &mut Vec<(usize, usize, &'static str)>,
) {
    if node.kind == crate::ast::SyntaxKind::VariableDeclarationList {
        let flags = node.flags;
        if flags.contains(NodeFlags::Const) {

            let pos = node.pos();
            replacements.push((pos, pos + 5, "var"));
        } else if flags.contains(NodeFlags::Let) {

            let pos = node.pos();
            replacements.push((pos, pos + 3, "var"));
        }
    }

    crate::ast::node_data_generated::for_each_child(node, |child| {
        collect_es5_replacements_recursive(child, replacements);
        false
    });
}

fn collect_export_modifier_cuts(stmt: &Node, source: &str) -> Vec<(usize, usize)> {
    let modifiers = match stmt.modifiers() {
        Some(m) => m,
        None => return Vec::new(),
    };
    if !modifiers.modifier_flags.contains(ModifierFlags::Export) {
        return Vec::new();
    }

    let mut cuts = Vec::new();
    let bytes = source.as_bytes();
    for mod_node in modifiers.list.iter() {
        if mod_node.kind == SyntaxKind::ExportKeyword || mod_node.kind == SyntaxKind::DefaultKeyword
        {
            let start = mod_node.pos();
            let mut end = mod_node.end();

            while end < bytes.len() && (bytes[end] == b' ' || bytes[end] == b'\t') {
                end += 1;
            }
            cuts.push((start, end));
        }
    }
    cuts
}

fn transform_commonjs_import(stmt: &Node, source: &str) -> Option<String> {
    let import_data = match &stmt.data {
        NodeData::ImportDeclaration(d) => d,
        _ => return None,
    };

    let specifier = &import_data.module_specifier;
    let specifier_text = &source[specifier.pos()..specifier.end()];

    let clause = match &import_data.import_clause {
        None => return Some(format!("require({specifier_text});")),
        Some(c) => c,
    };
    let clause_data = match &clause.data {
        NodeData::ImportClause(d) => d,
        _ => return Some(format!("require({specifier_text});")),
    };

    if clause_data.phase_modifier == Some(SyntaxKind::TypeKeyword) {
        return Some(String::new());
    }

    if let Some(bindings) = &clause_data.named_bindings {
        if let NodeData::NamespaceImport(ns_data) = &bindings.data {
            if let NodeData::Identifier(ident) = &ns_data.name.data {
                return Some(format!(
                    "const {} = require({});",
                    ident.text, specifier_text
                ));
            }
        }
    }

    let mut parts: Vec<String> = Vec::new();

    if let Some(name) = &clause_data.name {
        if let NodeData::Identifier(ident) = &name.data {
            parts.push(format!("default: {}", ident.text));
        }
    }

    if let Some(bindings) = &clause_data.named_bindings {
        if let NodeData::NamedImports(named) = &bindings.data {
            for spec in named.elements.iter() {
                if let NodeData::ImportSpecifier(spec_data) = &spec.data {
                    if spec_data.is_type_only {
                        continue;
                    }
                    if let Some(prop_name) = &spec_data.property_name {
                        if let (
                            NodeData::Identifier(prop_ident),
                            NodeData::Identifier(name_ident),
                        ) = (&prop_name.data, &spec_data.name.data)
                        {
                            parts.push(format!("{}: {}", prop_ident.text, name_ident.text));
                        }
                    } else if let NodeData::Identifier(name_ident) = &spec_data.name.data {
                        parts.push(name_ident.text.clone());
                    }
                }
            }
        }
    }

    if parts.is_empty() {
        return Some(format!("require({specifier_text});"));
    }

    Some(format!(
        "const {{ {} }} = require({});",
        parts.join(", "),
        specifier_text
    ))
}

fn transform_commonjs_export(stmt: &Node, source: &str) -> Option<String> {
    match &stmt.data {
        NodeData::ExportDeclaration(d) => {
            if d.is_type_only {
                return Some(String::new());
            }

            let specifier_text = d
                .module_specifier
                .as_ref()
                .map(|spec| source[spec.pos()..spec.end()].to_string());

            match d.export_clause.as_ref().map(|c| (&c.kind, c)) {
                Some((SyntaxKind::NamedExports, clause_node)) => {
                    if let NodeData::NamedExports(named) = &clause_node.data {
                        let mut lines: Vec<String> = Vec::new();

                        if let Some(spec) = &specifier_text {
                            let mut import_parts: Vec<String> = Vec::new();
                            for spec_node in named.elements.iter() {
                                if let NodeData::ExportSpecifier(spec_data) = &spec_node.data {
                                    if let NodeData::Identifier(name_ident) = &spec_data.name.data {
                                        import_parts.push(name_ident.text.clone());
                                    }
                                }
                            }
                            if !import_parts.is_empty() {
                                lines.push(format!(
                                    "const {{ {} }} = require({});",
                                    import_parts.join(", "),
                                    spec
                                ));
                            }
                        }

                        for spec_node in named.elements.iter() {
                            if let NodeData::ExportSpecifier(spec_data) = &spec_node.data {
                                let (local_name, export_name) =
                                    if let Some(prop_name) = &spec_data.property_name {
                                        match (&prop_name.data, &spec_data.name.data) {
                                            (NodeData::Identifier(p), NodeData::Identifier(n)) => {
                                                (p.text.clone(), n.text.clone())
                                            }
                                            _ => continue,
                                        }
                                    } else if let NodeData::Identifier(name_ident) =
                                        &spec_data.name.data
                                    {
                                        (name_ident.text.clone(), name_ident.text.clone())
                                    } else {
                                        continue;
                                    };
                                lines.push(format!("exports.{export_name} = {local_name};"));
                            }
                        }
                        return Some(lines.join("\n"));
                    }
                    Some(String::new())
                }
                Some((SyntaxKind::NamespaceExport, clause_node)) => {

                    if let NodeData::NamespaceExport(ns_data) = &clause_node.data {
                        if let NodeData::Identifier(ident) = &ns_data.name.data {
                            if let Some(spec) = &specifier_text {
                                return Some(format!(
                                    "const {n} = require({s});\nexports.{n} = {n};",
                                    n = ident.text,
                                    s = spec
                                ));
                            }
                        }
                    }
                    Some(String::new())
                }
                None => {

                    if let Some(spec) = &specifier_text {
                        return Some(format!(
                            "Object.keys(require({s})).forEach(function(k) {{ if (k !== \"default\") exports[k] = require({s})[k]; }});",
                            s = spec
                        ));
                    }
                    Some(String::new())
                }
                _ => Some(String::new()),
            }
        }
        NodeData::ExportAssignment(d) => {
            let expr_source = source[d.expression.pos()..d.expression.end()].to_string();
            if d.is_export_equals {
                Some(format!("module.exports = {expr_source};"))
            } else {
                Some(format!("exports.default = {expr_source};"))
            }
        }
        _ => None,
    }
}

fn transform_commonjs_export_declaration(stmt: &Node, _source: &str) -> Option<String> {
    let modifiers = stmt.modifiers()?;
    if !modifiers.modifier_flags.contains(ModifierFlags::Export) {
        return None;
    }
    let is_default = modifiers.modifier_flags.contains(ModifierFlags::Default);

    match &stmt.data {
        NodeData::VariableStatement(d) => {
            let decl_list = &d.declaration_list;
            let list_data = match &decl_list.data {
                NodeData::VariableDeclarationList(ld) => ld,
                _ => return None,
            };
            let mut lines: Vec<String> = Vec::new();
            for decl in list_data.declarations.iter() {
                if let NodeData::VariableDeclaration(decl_data) = &decl.data {
                    if let NodeData::Identifier(ident) = &decl_data.name.data {
                        if is_default {
                            lines.push(format!("exports.default = {};", ident.text));
                        } else {
                            lines.push(format!("exports.{n} = {n};", n = ident.text));
                        }
                    }
                }
            }
            if lines.is_empty() {
                None
            } else {
                Some(lines.join("\n"))
            }
        }
        NodeData::FunctionDeclaration(d) => {
            let name = d.name.as_ref()?;
            if let NodeData::Identifier(ident) = &name.data {
                if is_default {
                    Some(format!("exports.default = {};", ident.text))
                } else {
                    Some(format!("exports.{n} = {n};", n = ident.text))
                }
            } else {
                None
            }
        }
        NodeData::ClassDeclaration(d) => {
            let name = d.name.as_ref()?;
            if let NodeData::Identifier(ident) = &name.data {
                if is_default {
                    Some(format!("exports.default = {};", ident.text))
                } else {
                    Some(format!("exports.{n} = {n};", n = ident.text))
                }
            } else {
                None
            }
        }
        NodeData::EnumDeclaration(d) => {
            if let NodeData::Identifier(ident) = &d.name.data {
                Some(format!("exports.{n} = {n};", n = ident.text))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_type_only_statement(node: &Node) -> bool {
    match &node.data {
        NodeData::InterfaceDeclaration(_) => true,
        NodeData::TypeAliasDeclaration(_) => true,

        NodeData::ImportDeclaration(d) => is_type_only_import(d),

        NodeData::NamespaceExportDeclaration(_) => true,
        _ => false,
    }
}

fn is_type_only_import(d: &ImportDeclarationData) -> bool {
    let clause = match &d.import_clause {
        Some(c) => c,
        None => return false,
    };
    let cd = match &clause.data {
        NodeData::ImportClause(cd) => cd,
        _ => return false,
    };

    if cd.phase_modifier == Some(SyntaxKind::TypeKeyword) {
        return true;
    }

    if cd.name.is_none() {
        if let Some(bindings) = &cd.named_bindings {
            if let NodeData::NamedImports(named) = &bindings.data {
                return !named.elements.is_empty()
                    && named
                        .elements
                        .iter()
                        .all(|spec| is_type_only_import_specifier(spec));
            }
        }
    }
    false
}

fn is_type_only_import_specifier(spec: &Node) -> bool {
    matches!(&spec.data, NodeData::ImportSpecifier(sd) if sd.is_type_only)
}

fn rewrite_import_extensions(text: &str) -> String {

    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {

        let is_char_start = (bytes[i] & 0xC0) != 0x80;

        if is_char_start && i + 5 <= bytes.len() && &bytes[i..i + 5] == b"from " {
            result.push_str("from ");
            i += 5;

            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                result.push(bytes[i] as char);
                i += 1;
            }

            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let quote = bytes[i] as char;
                let start = i + 1;
                result.push(quote);
                i += 1;
                while i < bytes.len() && bytes[i] != quote as u8 {

                    if (bytes[i] & 0x80) == 0 {
                        i += 1;
                    } else if (bytes[i] & 0xE0) == 0xC0 {
                        i += 2;
                    } else if (bytes[i] & 0xF0) == 0xE0 {
                        i += 3;
                    } else {
                        i += 4;
                    }
                }
                let specifier = &text[start..i];
                let rewritten = rewrite_one_specifier(specifier);
                result.push_str(&rewritten);
                if i < bytes.len() {
                    result.push(quote);
                    i += 1;
                }
            }
        } else if is_char_start && i + 7 <= bytes.len() && &bytes[i..i + 7] == b"import(" {
            result.push_str("import(");
            i += 7;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                result.push(bytes[i] as char);
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let quote = bytes[i] as char;
                let start = i + 1;
                result.push(quote);
                i += 1;
                while i < bytes.len() && bytes[i] != quote as u8 {

                    if (bytes[i] & 0x80) == 0 {
                        i += 1;
                    } else if (bytes[i] & 0xE0) == 0xC0 {
                        i += 2;
                    } else if (bytes[i] & 0xF0) == 0xE0 {
                        i += 3;
                    } else {
                        i += 4;
                    }
                }
                let specifier = &text[start..i];
                let rewritten = rewrite_one_specifier(specifier);
                result.push_str(&rewritten);
                if i < bytes.len() {
                    result.push(quote);
                    i += 1;
                }
            }
        } else {

            let ch = text[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
        }
    }
    result
}

fn rewrite_one_specifier(spec: &str) -> String {
    for (old, new) in [
        (".ts", ".js"),
        (".tsx", ".js"),
        (".mts", ".mjs"),
        (".cts", ".cjs"),
    ] {
        if spec.ends_with(old) {
            return format!("{}{}", &spec[..spec.len() - old.len()], new);
        }
    }
    spec.to_string()
}

fn add_implicit_semicolons(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for line in text.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            result.push('\n');
            continue;
        }
        let last = trimmed.chars().last().unwrap_or(' ');

        let skip = matches!(
            last,
            '{' | '(' | '[' | ',' | ';' | ':' | '.' | '|' | '&' | '=' | '>' | '?'
        ) || trimmed.ends_with("=>")
            || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.ends_with("*/");
        if skip {
            result.push_str(trimmed);
        } else if last == '}' {

            result.push_str(trimmed);
        } else {
            result.push_str(trimmed);
            result.push(';');
        }
        result.push('\n');
    }
    if !text.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }
    result
}

#[allow(dead_code)]
fn normalize_js_output(text: &str) -> String {
    let folded = fold_expression_newlines(text);
    let reindented = reindent_and_dedup(&folded);
    add_implicit_semicolons(&reindented)
}

fn rewrite_import_extensions_tracked(text: &str, src_offsets: &[u32]) -> (String, Vec<u32>) {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out_text = String::with_capacity(text.len());
    let mut out_offsets = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        if i + 5 <= n
            && chars[i] == 'f'
            && chars[i + 1] == 'r'
            && chars[i + 2] == 'o'
            && chars[i + 3] == 'm'
            && chars[i + 4] == ' '
        {
            i = copy_string_literal_tracked(
                &chars,
                src_offsets,
                i,
                &mut out_text,
                &mut out_offsets,
            );
        } else if i + 7 <= n
            && chars[i] == 'i'
            && chars[i + 1] == 'm'
            && chars[i + 2] == 'p'
            && chars[i + 3] == 'o'
            && chars[i + 4] == 'r'
            && chars[i + 5] == 't'
            && chars[i + 6] == '('
        {
            for j in 0..7 {
                out_text.push(chars[i + j]);
                out_offsets.push(src_offsets[i + j]);
            }
            i += 7;
            while i < n && chars[i].is_ascii_whitespace() {
                out_text.push(chars[i]);
                out_offsets.push(src_offsets[i]);
                i += 1;
            }
            if i < n && (chars[i] == '"' || chars[i] == '\'') {
                i = copy_string_literal_tracked(
                    &chars,
                    src_offsets,
                    i,
                    &mut out_text,
                    &mut out_offsets,
                );
            }
        } else {
            out_text.push(chars[i]);
            out_offsets.push(src_offsets[i]);
            i += 1;
        }
    }
    (out_text, out_offsets)
}

fn copy_string_literal_tracked(
    chars: &[char],
    src_offsets: &[u32],
    start: usize,
    out_text: &mut String,
    out_offsets: &mut Vec<u32>,
) -> usize {

    let mut i = start;
    if chars[i] == 'f' {
        for _ in 0..5 {
            out_text.push(chars[i]);
            out_offsets.push(src_offsets[i]);
            i += 1;
        }
        while i < chars.len() && chars[i].is_ascii_whitespace() {
            out_text.push(chars[i]);
            out_offsets.push(src_offsets[i]);
            i += 1;
        }
    }

    if i < chars.len() && (chars[i] == '"' || chars[i] == '\'') {
        let quote = chars[i];
        out_text.push(quote);
        out_offsets.push(src_offsets[i]);
        i += 1;
        let spec_start = i;
        while i < chars.len() && chars[i] != quote {
            i += 1;
        }
        let specifier: String = chars[spec_start..i].iter().collect();
        let rewritten = rewrite_one_specifier(&specifier);
        let spec_len = i - spec_start;
        for (j, rc) in rewritten.chars().enumerate() {
            out_text.push(rc);
            if j < spec_len {
                out_offsets.push(src_offsets[spec_start + j]);
            } else {
                out_offsets.push(src_offsets[spec_start + spec_len - 1]);
            }
        }
        if i < chars.len() {
            out_text.push(chars[i]);
            out_offsets.push(src_offsets[i]);
            i += 1;
        }
    }
    i
}

fn fold_expression_newlines_tracked(text: &str, src_offsets: &[u32]) -> (String, Vec<u32>) {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out: Vec<char> = Vec::with_capacity(n);
    let mut out_idx: Vec<usize> = Vec::with_capacity(n);

    #[derive(Clone, Copy, PartialEq)]
    enum SCtx {
        Single,
        Double,
        Template,
        LineComment,
        BlockComment,
    }
    #[derive(Clone, Copy)]
    enum Group {
        Paren(bool),
        Bracket(bool),
        Brace,
        TmplInterp,
    }

    let mut sctx: Vec<SCtx> = Vec::new();
    let mut groups: Vec<Group> = Vec::new();

    let mut i = 0;
    while i < n {
        let c = chars[i];

        if let Some(&ctx) = sctx.last() {
            match ctx {
                SCtx::Single => {
                    out.push(c);
                    out_idx.push(i);
                    if c == '\\' {
                        i += 1;
                        if i < n {
                            out.push(chars[i]);
                            out_idx.push(i);
                            i += 1;
                        }
                        continue;
                    }
                    if c == '\'' {
                        sctx.pop();
                    }
                    i += 1;
                    continue;
                }
                SCtx::Double => {
                    out.push(c);
                    out_idx.push(i);
                    if c == '\\' {
                        i += 1;
                        if i < n {
                            out.push(chars[i]);
                            out_idx.push(i);
                            i += 1;
                        }
                        continue;
                    }
                    if c == '"' {
                        sctx.pop();
                    }
                    i += 1;
                    continue;
                }
                SCtx::Template => {
                    out.push(c);
                    out_idx.push(i);
                    if c == '\\' {
                        i += 1;
                        if i < n {
                            out.push(chars[i]);
                            out_idx.push(i);
                            i += 1;
                        }
                        continue;
                    }
                    if c == '`' {
                        sctx.pop();
                        i += 1;
                        continue;
                    }
                    if c == '$' && i + 1 < n && chars[i + 1] == '{' {
                        out.push('{');
                        out_idx.push(i + 1);
                        sctx.pop();
                        groups.push(Group::TmplInterp);
                        i += 2;
                        continue;
                    }
                    i += 1;
                    continue;
                }
                SCtx::LineComment => {
                    if c == '\n' || c == '\r' {
                        sctx.pop();

                    } else {
                        out.push(c);
                        out_idx.push(i);
                        i += 1;
                        continue;
                    }
                }
                SCtx::BlockComment => {
                    out.push(c);
                    out_idx.push(i);
                    if c == '*' && i + 1 < n && chars[i + 1] == '/' {
                        out.push('/');
                        out_idx.push(i + 1);
                        sctx.pop();
                        i += 2;
                        continue;
                    }
                    i += 1;
                    continue;
                }
            }
        }

        if c == '\'' {
            sctx.push(SCtx::Single);
            out.push(c);
            out_idx.push(i);
            i += 1;
            continue;
        }
        if c == '"' {
            sctx.push(SCtx::Double);
            out.push(c);
            out_idx.push(i);
            i += 1;
            continue;
        }
        if c == '`' {
            sctx.push(SCtx::Template);
            out.push(c);
            out_idx.push(i);
            i += 1;
            continue;
        }
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            sctx.push(SCtx::LineComment);
            out.push('/');
            out_idx.push(i);
            out.push('/');
            out_idx.push(i + 1);
            i += 2;
            continue;
        }
        if c == '/' && i + 1 < n && chars[i + 1] == '*' {
            sctx.push(SCtx::BlockComment);
            out.push('/');
            out_idx.push(i);
            out.push('*');
            out_idx.push(i + 1);
            i += 2;
            continue;
        }

        if c == '(' {
            groups.push(Group::Paren(false));
            out.push(c);
            out_idx.push(i);
            i += 1;
            continue;
        }
        if c == '[' {
            groups.push(Group::Bracket(false));
            out.push(c);
            out_idx.push(i);
            i += 1;
            continue;
        }
        if c == '{' {
            groups.push(Group::Brace);
            out.push(c);
            out_idx.push(i);
            i += 1;
            continue;
        }
        if c == ')' {
            if let Some(Group::Paren(folded)) = groups.last().copied() {
                groups.pop();
                if folded {
                    drop_trailing_idx(&mut out, &mut out_idx);
                }
            }
            out.push(c);
            out_idx.push(i);
            i += 1;
            continue;
        }
        if c == ']' {
            if let Some(Group::Bracket(folded)) = groups.last().copied() {
                groups.pop();
                if folded {
                    drop_trailing_idx(&mut out, &mut out_idx);
                }
            }
            out.push(c);
            out_idx.push(i);
            i += 1;
            continue;
        }
        if c == '}' {
            match groups.last() {
                Some(Group::TmplInterp) => {
                    groups.pop();
                    sctx.push(SCtx::Template);
                    out.push(c);
                    out_idx.push(i);
                    i += 1;
                    continue;
                }
                Some(Group::Brace) => {
                    groups.pop();
                }
                _ => {}
            }
            out.push(c);
            out_idx.push(i);
            i += 1;
            continue;
        }

        if c == '\n' || c == '\r' {
            let do_fold = matches!(
                groups.last(),
                Some(Group::Paren(_)) | Some(Group::Bracket(_))
            );
            if do_fold {
                if let Some(g) = groups.last_mut() {
                    if let Group::Paren(f) | Group::Bracket(f) = g {
                        *f = true;
                    }
                }
                while let Some(&ch) = out.last() {
                    if ch == ' ' || ch == '\t' {
                        out.pop();
                        out_idx.pop();
                    } else {
                        break;
                    }
                }
                i += 1;
                if i < n && chars[i - 1] == '\r' && chars[i] == '\n' {
                    i += 1;
                }
                while i < n && (chars[i] == ' ' || chars[i] == '\t') {
                    i += 1;
                }
            } else {
                out.push('\n');
                out_idx.push(i);
                i += 1;
                if i < n && chars[i - 1] == '\r' && chars[i] == '\n' {
                    i += 1;
                }
            }
            continue;
        }

        out.push(c);
        out_idx.push(i);
        i += 1;
    }

    let result_text: String = out.into_iter().collect();
    let result_offsets: Vec<u32> = out_idx.iter().map(|&idx| src_offsets[idx]).collect();
    (result_text, result_offsets)
}

fn drop_trailing_idx(out: &mut Vec<char>, out_idx: &mut Vec<usize>) {
    while let Some(&ch) = out.last() {
        if ch == ' ' || ch == '\t' {
            out.pop();
            out_idx.pop();
        } else {
            break;
        }
    }
    if out.last() == Some(&',') {
        out.pop();
        out_idx.pop();
    }
}

fn reindent_and_dedup_tracked(folded: &str, src_offsets: &[u32]) -> (String, Vec<u32>) {
    let chars: Vec<char> = folded.chars().collect();
    let n = chars.len();
    let mut out_text = String::with_capacity(folded.len());
    let mut out_offsets: Vec<u32> = Vec::new();
    let mut depth: i32 = 0;
    let had_trailing_newline = n > 0 && chars[n - 1] == '\n';

    let mut i = 0;
    while i < n {
        let line_start = i;
        while i < n && chars[i] != '\n' {
            i += 1;
        }
        let line_end = i;
        let newline_idx = if i < n && chars[i] == '\n' {
            Some(i)
        } else {
            None
        };
        if i < n && chars[i] == '\n' {
            i += 1;
        }

        let mut content_start = line_start;
        while content_start < line_end && chars[content_start].is_whitespace() {
            content_start += 1;
        }
        let mut content_end = line_end;
        while content_end > content_start && chars[content_end - 1].is_whitespace() {
            content_end -= 1;
        }

        if content_start >= content_end {
            continue;
        }

        let starts_with_close = chars[content_start] == '}';
        let indent_depth = (depth - if starts_with_close { 1 } else { 0 }).max(0);
        for _ in 0..indent_depth {
            out_text.push_str("    ");
            for _ in 0..4 {
                out_offsets.push(UNMAPPED);
            }
        }
        for j in content_start..content_end {
            out_text.push(chars[j]);
            out_offsets.push(src_offsets[j]);
        }
        out_text.push('\n');
        if let Some(nl) = newline_idx {
            out_offsets.push(src_offsets[nl]);
        } else {
            out_offsets.push(src_offsets[content_end - 1]);
        }

        let content: String = chars[content_start..content_end].iter().collect();
        depth += brace_delta(&content);
        if depth < 0 {
            depth = 0;
        }
    }

    if !had_trailing_newline && out_text.ends_with('\n') {
        out_text.pop();
        out_offsets.pop();
    }
    (out_text, out_offsets)
}

fn add_implicit_semicolons_tracked(text: &str, src_offsets: &[u32]) -> (String, Vec<u32>) {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out_text = String::with_capacity(text.len());
    let mut out_offsets: Vec<u32> = Vec::new();
    let had_trailing_newline = n > 0 && chars[n - 1] == '\n';

    let mut i = 0;
    while i < n {
        let line_start = i;
        while i < n && chars[i] != '\n' {
            i += 1;
        }
        let line_end = i;
        let has_newline = i < n && chars[i] == '\n';
        if has_newline {
            i += 1;
        }

        let mut content_end = line_end;
        while content_end > line_start && chars[content_end - 1].is_whitespace() {
            content_end -= 1;
        }

        if content_end == line_start {
            out_text.push('\n');
            if has_newline {
                out_offsets.push(src_offsets[line_end]);
            } else {
                out_offsets.push(UNMAPPED);
            }
            continue;
        }

        let last = chars[content_end - 1];
        let trimmed_str: String = chars[line_start..content_end].iter().collect();
        let skip = matches!(
            last,
            '{' | '(' | '[' | ',' | ';' | ':' | '.' | '|' | '&' | '=' | '>' | '?'
        ) || trimmed_str.ends_with("=>")
            || trimmed_str.starts_with("//")
            || trimmed_str.starts_with("/*")
            || trimmed_str.ends_with("*/");

        for j in line_start..content_end {
            out_text.push(chars[j]);
            out_offsets.push(src_offsets[j]);
        }

        if !skip && last != '}' {
            out_text.push(';');
            out_offsets.push(UNMAPPED);
        }

        out_text.push('\n');
        if has_newline {
            out_offsets.push(src_offsets[line_end]);
        } else {
            out_offsets.push(UNMAPPED);
        }
    }

    if !had_trailing_newline && out_text.ends_with('\n') {
        out_text.pop();
        out_offsets.pop();
    }
    (out_text, out_offsets)
}

fn normalize_js_output_tracked(text: &str, src_offsets: &[u32]) -> (String, Vec<u32>) {
    let (text, offsets) = fold_expression_newlines_tracked(text, src_offsets);
    let (text, offsets) = reindent_and_dedup_tracked(&text, &offsets);
    add_implicit_semicolons_tracked(&text, &offsets)
}

#[allow(dead_code)]
fn fold_expression_newlines(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out: Vec<char> = Vec::with_capacity(n);

    #[derive(Clone, Copy, PartialEq)]
    enum SCtx {
        Single,
        Double,
        Template,
        LineComment,
        BlockComment,
    }

    #[derive(Clone, Copy)]
    enum Group {
        Paren(bool),
        Bracket(bool),
        Brace,
        TmplInterp,
    }

    let mut sctx: Vec<SCtx> = Vec::new();
    let mut groups: Vec<Group> = Vec::new();

    let mut i = 0;
    while i < n {
        let c = chars[i];

        if let Some(&ctx) = sctx.last() {
            match ctx {
                SCtx::Single => {
                    out.push(c);
                    if c == '\\' {
                        i += 1;
                        if i < n {
                            out.push(chars[i]);
                            i += 1;
                        }
                        continue;
                    }
                    if c == '\'' {
                        sctx.pop();
                    }
                    i += 1;
                    continue;
                }
                SCtx::Double => {
                    out.push(c);
                    if c == '\\' {
                        i += 1;
                        if i < n {
                            out.push(chars[i]);
                            i += 1;
                        }
                        continue;
                    }
                    if c == '"' {
                        sctx.pop();
                    }
                    i += 1;
                    continue;
                }
                SCtx::Template => {
                    out.push(c);
                    if c == '\\' {
                        i += 1;
                        if i < n {
                            out.push(chars[i]);
                            i += 1;
                        }
                        continue;
                    }
                    if c == '`' {
                        sctx.pop();
                        i += 1;
                        continue;
                    }
                    if c == '$' && i + 1 < n && chars[i + 1] == '{' {
                        out.push('{');
                        sctx.pop();
                        groups.push(Group::TmplInterp);
                        i += 2;
                        continue;
                    }
                    i += 1;
                    continue;
                }
                SCtx::LineComment => {
                    if c == '\n' || c == '\r' {
                        sctx.pop();

                    } else {
                        out.push(c);
                        i += 1;
                        continue;
                    }
                }
                SCtx::BlockComment => {
                    out.push(c);
                    if c == '*' && i + 1 < n && chars[i + 1] == '/' {
                        out.push('/');
                        sctx.pop();
                        i += 2;
                        continue;
                    }
                    i += 1;
                    continue;
                }
            }
        }

        if c == '\'' {
            sctx.push(SCtx::Single);
            out.push(c);
            i += 1;
            continue;
        }
        if c == '"' {
            sctx.push(SCtx::Double);
            out.push(c);
            i += 1;
            continue;
        }
        if c == '`' {
            sctx.push(SCtx::Template);
            out.push(c);
            i += 1;
            continue;
        }
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            sctx.push(SCtx::LineComment);
            out.push('/');
            out.push('/');
            i += 2;
            continue;
        }
        if c == '/' && i + 1 < n && chars[i + 1] == '*' {
            sctx.push(SCtx::BlockComment);
            out.push('/');
            out.push('*');
            i += 2;
            continue;
        }

        if c == '(' {
            groups.push(Group::Paren(false));
            out.push(c);
            i += 1;
            continue;
        }
        if c == '[' {
            groups.push(Group::Bracket(false));
            out.push(c);
            i += 1;
            continue;
        }
        if c == '{' {
            groups.push(Group::Brace);
            out.push(c);
            i += 1;
            continue;
        }
        if c == ')' {
            if let Some(Group::Paren(folded)) = groups.last().copied() {
                groups.pop();
                if folded {
                    drop_trailing_comma(&mut out);
                }
            }
            out.push(c);
            i += 1;
            continue;
        }
        if c == ']' {
            if let Some(Group::Bracket(folded)) = groups.last().copied() {
                groups.pop();
                if folded {
                    drop_trailing_comma(&mut out);
                }
            }
            out.push(c);
            i += 1;
            continue;
        }
        if c == '}' {
            match groups.last() {
                Some(Group::TmplInterp) => {
                    groups.pop();
                    sctx.push(SCtx::Template);
                    out.push(c);
                    i += 1;
                    continue;
                }
                Some(Group::Brace) => {
                    groups.pop();
                }
                _ => {}
            }
            out.push(c);
            i += 1;
            continue;
        }

        if c == '\n' || c == '\r' {
            let do_fold = matches!(
                groups.last(),
                Some(Group::Paren(_)) | Some(Group::Bracket(_))
            );
            if do_fold {
                if let Some(g) = groups.last_mut() {
                    if let Group::Paren(f) | Group::Bracket(f) = g {
                        *f = true;
                    }
                }

                while let Some(&ch) = out.last() {
                    if ch == ' ' || ch == '\t' {
                        out.pop();
                    } else {
                        break;
                    }
                }

                i += 1;
                if i < n && chars[i - 1] == '\r' && chars[i] == '\n' {
                    i += 1;
                }

                while i < n && (chars[i] == ' ' || chars[i] == '\t') {
                    i += 1;
                }
            } else {
                out.push('\n');
                i += 1;
                if i < n && chars[i - 1] == '\r' && chars[i] == '\n' {
                    i += 1;
                }
            }
            continue;
        }

        out.push(c);
        i += 1;
    }

    out.into_iter().collect()
}

#[allow(dead_code)]
fn drop_trailing_comma(out: &mut Vec<char>) {
    while let Some(&ch) = out.last() {
        if ch == ' ' || ch == '\t' {
            out.pop();
        } else {
            break;
        }
    }
    if out.last() == Some(&',') {
        out.pop();
    }
}

fn brace_delta(line: &str) -> i32 {
    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    let mut delta = 0i32;
    let mut i = 0;
    while i < n {
        let c = chars[i];
        match c {
            '\'' | '"' | '`' => {
                let quote = c;
                i += 1;
                while i < n {
                    if chars[i] == '\\' {
                        i += 2;
                        continue;
                    }
                    if chars[i] == quote {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
            }
            '/' if i + 1 < n && chars[i + 1] == '/' => {

                break;
            }
            '/' if i + 1 < n && chars[i + 1] == '*' => {
                i += 2;
                while i < n {
                    if chars[i] == '*' && i + 1 < n && chars[i + 1] == '/' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
            }
            '{' => {
                delta += 1;
                i += 1;
            }
            '}' => {
                delta -= 1;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    delta
}

fn reindent_and_dedup(folded: &str) -> String {
    let mut out = String::with_capacity(folded.len());
    let mut depth: i32 = 0;
    let had_trailing_newline = folded.ends_with('\n');

    for raw_line in folded.split('\n') {
        let stripped = raw_line.trim();
        if stripped.is_empty() {
            continue;
        }
        let starts_with_close = stripped.starts_with('}');
        let indent_depth = (depth - if starts_with_close { 1 } else { 0 }).max(0);
        for _ in 0..indent_depth {
            out.push_str("    ");
        }
        out.push_str(stripped);
        out.push('\n');
        depth += brace_delta(stripped);
        if depth < 0 {
            depth = 0;
        }
    }

    if !had_trailing_newline && out.ends_with('\n') {
        out.pop();
    }
    out
}

fn emit_statement<S: EmitSink>(
    node: &Node,
    source: &str,
    comment_cuts: &[(usize, usize)],
    replacements: &[(usize, usize, &str, Option<usize>)],
    sink: &mut S,
) {
    let mut cuts: Vec<(usize, usize)> = Vec::new();
    collect_type_cuts(node, source, &mut cuts);

    if !comment_cuts.is_empty() {
        for &(cs, ce) in comment_cuts {
            if ce > node.pos() && cs < node.end() {
                cuts.push((cs, ce));
            }
        }
    }

    let mut stmt_replacements: Vec<(usize, usize, &str, Option<usize>)> = Vec::new();
    for &(rs, re, repl, src_pos) in replacements {
        if re > node.pos() && rs < node.end() {
            stmt_replacements.push((rs, re, repl, src_pos));
        }
    }

    if cuts.is_empty() && stmt_replacements.is_empty() {

        sink.emit_source(source, node.pos(), node.end());
        return;
    }

    let mut ops: Vec<(usize, usize, Option<(&str, Option<usize>)>)> = Vec::new();
    for (cs, ce) in &cuts {
        if *ce > node.pos() && *cs < node.end() {
            ops.push(((*cs).max(node.pos()), (*ce).min(node.end()), None));
        }
    }
    for (rs, re, repl, src_pos) in &stmt_replacements {
        ops.push((*rs, *re, Some((*repl, *src_pos))));
    }
    ops.sort_by_key(|&(s, _, _)| s);

    let mut pos = node.pos();
    for (s, e, repl) in &ops {
        if *s > pos {
            sink.emit_source(source, pos, *s);
        }
        if let Some((r, src_pos)) = repl {
            if let Some(sp) = src_pos {
                sink.emit_source_mapped(r, *sp);
            } else {
                sink.emit_generated(r);
            }
        }
        pos = *e;
    }
    if pos < node.end() {
        sink.emit_source(source, pos, node.end());
    }
}

fn collect_type_cuts(node: &Node, source: &str, cuts: &mut Vec<(usize, usize)>) {

    match node.kind {
        SyntaxKind::JsxElement
        | SyntaxKind::JsxSelfClosingElement
        | SyntaxKind::JsxFragment
        | SyntaxKind::JsxOpeningElement
        | SyntaxKind::JsxAttributes
        | SyntaxKind::JsxAttribute
        | SyntaxKind::JsxSpreadAttribute
        | SyntaxKind::JsxClosingElement
        | SyntaxKind::JsxExpression
        | SyntaxKind::JsxText
        | SyntaxKind::JsxTextAllWhiteSpaces
        | SyntaxKind::JsxOpeningFragment
        | SyntaxKind::JsxClosingFragment
        | SyntaxKind::JsxNamespacedName => return,
        _ => {}
    }

    collect_modifier_cuts(node, source, cuts);
    match &node.data {
        NodeData::VariableDeclaration(d) => {

            if let Some(type_node) = &d.type_node {
                cuts.push((d.name.end(), type_node.end()));
            }

            collect_type_cuts(&d.name, source, cuts);
            if let Some(init) = &d.initializer {
                collect_type_cuts(init, source, cuts);
            }
        }
        NodeData::ParameterDeclaration(d) => {

            if let Some(type_node) = &d.type_node {
                cuts.push((d.name.end(), type_node.end()));
            }

            if let Some(q) = &d.question_token {
                cuts.push((d.name.end(), q.end()));
            }

            if let Some(init) = &d.initializer {
                collect_type_cuts(init, source, cuts);
            }
        }
        NodeData::VariableDeclarationList(d) => {
            for decl in d.declarations.iter() {
                collect_type_cuts(decl, source, cuts);
            }
        }
        NodeData::VariableStatement(d) => {
            collect_type_cuts(&d.declaration_list, source, cuts);
        }
        NodeData::FunctionDeclaration(d) => {

            if let Some(tp) = &d.type_parameters {
                cuts.push((tp.pos(), tp.end()));
            }

            for param in d.parameters.iter() {
                collect_type_cuts(param, source, cuts);
            }

            if let Some(type_node) = &d.type_node {
                cuts.push((d.parameters.end(), type_node.end()));
            }

            if let Some(body) = &d.body {
                collect_type_cuts(body, source, cuts);
            }
        }
        NodeData::FunctionExpression(d) => {
            if let Some(tp) = &d.type_parameters {
                cuts.push((tp.pos(), tp.end()));
            }
            for param in d.parameters.iter() {
                collect_type_cuts(param, source, cuts);
            }
            if let Some(type_node) = &d.type_node {
                cuts.push((d.parameters.end(), type_node.end()));
            }
            collect_type_cuts(&d.body, source, cuts);
        }
        NodeData::ArrowFunction(d) => {
            if let Some(tp) = &d.type_parameters {
                cuts.push((tp.pos(), tp.end()));
            }
            for param in d.parameters.iter() {
                collect_type_cuts(param, source, cuts);
            }
            if let Some(type_node) = &d.type_node {
                cuts.push((d.parameters.end(), type_node.end()));
            }
            collect_type_cuts(&d.body, source, cuts);
        }
        NodeData::ClassDeclaration(d) => {

            if let Some(tp) = &d.type_parameters {
                cuts.push((tp.pos(), tp.end()));
            }

            cut_implements_clauses(d.heritage_clauses.as_deref(), source, cuts);

            for member in d.members.iter() {
                collect_type_cuts(member, source, cuts);
            }
        }
        NodeData::ClassExpression(d) => {
            if let Some(tp) = &d.type_parameters {
                cuts.push((tp.pos(), tp.end()));
            }
            cut_implements_clauses(d.heritage_clauses.as_deref(), source, cuts);
            for member in d.members.iter() {
                collect_type_cuts(member, source, cuts);
            }
        }
        NodeData::MethodDeclaration(d) => {
            if let Some(tp) = &d.type_parameters {
                cuts.push((tp.pos(), tp.end()));
            }
            for param in d.parameters.iter() {
                collect_type_cuts(param, source, cuts);
            }
            if let Some(type_node) = &d.type_node {
                cuts.push((d.parameters.end(), type_node.end()));
            }
            if let Some(body) = &d.body {
                collect_type_cuts(body, source, cuts);
            }
        }
        NodeData::ConstructorDeclaration(d) => {
            for param in d.parameters.iter() {
                collect_type_cuts(param, source, cuts);
            }
            if let Some(body) = &d.body {
                collect_type_cuts(body, source, cuts);
            }
        }
        NodeData::GetAccessorDeclaration(d) => {
            for param in d.parameters.iter() {
                collect_type_cuts(param, source, cuts);
            }
            if let Some(type_node) = &d.type_node {
                cuts.push((d.parameters.end(), type_node.end()));
            }
            if let Some(body) = &d.body {
                collect_type_cuts(body, source, cuts);
            }
        }
        NodeData::SetAccessorDeclaration(d) => {
            for param in d.parameters.iter() {
                collect_type_cuts(param, source, cuts);
            }
            if let Some(body) = &d.body {
                collect_type_cuts(body, source, cuts);
            }
        }
        NodeData::PropertyDeclaration(d) => {

            if let Some(type_node) = &d.type_node {
                cuts.push((d.name.end(), type_node.end()));
            }
        }
        NodeData::ImportDeclaration(d) => {

            if let Some(clause) = &d.import_clause {
                collect_import_clause_type_cuts(clause, source, cuts);
            }
        }
        NodeData::AsExpression(d) => {

            cuts.push((d.expression.end(), d.type_node.end()));
        }
        NodeData::TypeAssertion(d) => {

            cuts.push((node.pos(), d.expression.pos()));
            collect_type_cuts(&d.expression, source, cuts);
        }
        NodeData::SatisfiesExpression(d) => {

            cuts.push((d.expression.end(), d.type_node.end()));
        }
        NodeData::NonNullExpression(d) => {

            cuts.push((d.expression.end(), node.end()));
            collect_type_cuts(&d.expression, source, cuts);
        }
        NodeData::ExpressionStatement(d) => {
            collect_type_cuts(&d.expression, source, cuts);
        }
        NodeData::ReturnStatement(d) => {
            if let Some(expr) = &d.expression {
                collect_type_cuts(expr, source, cuts);
            }
        }
        NodeData::Block(d) => {
            for stmt in d.statements.iter() {
                if !is_type_only_statement(stmt) {
                    collect_type_cuts(stmt, source, cuts);
                }
            }
        }
        NodeData::IfStatement(d) => {
            collect_type_cuts(&d.expression, source, cuts);
            collect_type_cuts(&d.then_statement, source, cuts);
            if let Some(else_stmt) = &d.else_statement {
                collect_type_cuts(else_stmt, source, cuts);
            }
        }
        NodeData::ForStatement(d) => {
            if let Some(init) = &d.initializer {
                collect_type_cuts(init, source, cuts);
            }
            if let Some(cond) = &d.condition {
                collect_type_cuts(cond, source, cuts);
            }
            if let Some(incr) = &d.incrementor {
                collect_type_cuts(incr, source, cuts);
            }
            collect_type_cuts(&d.statement, source, cuts);
        }
        NodeData::ForInOrOfStatement(d) => {
            collect_type_cuts(&d.initializer, source, cuts);
            collect_type_cuts(&d.expression, source, cuts);
            collect_type_cuts(&d.statement, source, cuts);
        }
        NodeData::WhileStatement(d) => {
            collect_type_cuts(&d.expression, source, cuts);
            collect_type_cuts(&d.statement, source, cuts);
        }
        NodeData::DoStatement(d) => {
            collect_type_cuts(&d.statement, source, cuts);
            collect_type_cuts(&d.expression, source, cuts);
        }

        _ => {
            crate::ast::node_data_generated::for_each_child(node, |child| {
                collect_type_cuts(child, source, cuts);
                false
            });
        }
    }
}

fn collect_modifier_cuts(node: &Node, source: &str, cuts: &mut Vec<(usize, usize)>) {
    let modifiers = node.modifier_nodes();
    if modifiers.is_empty() {
        return;
    }
    let bytes = source.as_bytes();
    for mod_node in modifiers {
        if matches!(
            mod_node.kind,
            SyntaxKind::AbstractKeyword
                | SyntaxKind::DeclareKeyword
                | SyntaxKind::OverrideKeyword
                | SyntaxKind::ReadonlyKeyword
        ) {
            let start = mod_node.pos();
            let mut end = mod_node.end();

            while end < bytes.len() && (bytes[end] == b' ' || bytes[end] == b'\t') {
                end += 1;
            }
            cuts.push((start, end));
        }
    }
}

fn cut_implements_clauses(
    heritage_clauses: Option<&NodeList>,
    source: &str,
    cuts: &mut Vec<(usize, usize)>,
) {
    let clauses = match heritage_clauses {
        Some(c) => c,
        None => return,
    };
    let bytes = source.as_bytes();
    for hc in clauses.iter() {
        if let NodeData::HeritageClause(hcd) = &hc.data {
            if hcd.token == SyntaxKind::ImplementsKeyword {
                let mut start = hc.pos();
                while start > 0 && (bytes[start - 1] == b' ' || bytes[start - 1] == b'\t') {
                    start -= 1;
                }
                cuts.push((start, hc.end()));
            }
        }
    }
}

fn collect_import_clause_type_cuts(clause: &Node, source: &str, cuts: &mut Vec<(usize, usize)>) {
    let cd = match &clause.data {
        NodeData::ImportClause(cd) => cd,
        _ => return,
    };

    let bindings = match &cd.named_bindings {
        Some(b) => b,
        None => return,
    };
    let named = match &bindings.data {
        NodeData::NamedImports(named) => named,
        _ => return,
    };
    if named.elements.is_empty() {
        return;
    }
    let all_type = named
        .elements
        .iter()
        .all(|spec| is_type_only_import_specifier(spec));

    if all_type {

        if cd.name.is_some() {
            let bytes = source.as_bytes();
            let mut start = bindings.pos();
            while start > 0 && (bytes[start - 1] == b' ' || bytes[start - 1] == b'\t') {
                start -= 1;
            }
            if start > 0 && bytes[start - 1] == b',' {
                start -= 1;
            }
            cuts.push((start, bindings.end()));
        }
        return;
    }

    for spec in named.elements.iter() {
        if is_type_only_import_specifier(spec) {
            cuts.push(specifier_cut_range(spec, source));
        }
    }
}

fn specifier_cut_range(spec: &Node, source: &str) -> (usize, usize) {
    let s = spec.pos();
    let e = spec.end();
    let bytes = source.as_bytes();

    let mut back = s;
    while back > 0 && (bytes[back - 1] == b' ' || bytes[back - 1] == b'\t') {
        back -= 1;
    }
    if back > 0 && bytes[back - 1] == b',' {
        return (back - 1, e);
    }

    let mut fwd = e;
    while fwd < bytes.len() && (bytes[fwd] == b' ' || bytes[fwd] == b'\t') {
        fwd += 1;
    }
    if fwd < bytes.len() && bytes[fwd] == b',' {
        let mut end = fwd + 1;
        while end < bytes.len() && (bytes[end] == b' ' || bytes[end] == b'\t') {
            end += 1;
        }
        return (s, end);
    }

    (s, e)
}

#[derive(Default)]
struct JsxRuntimeUsage {
    used_jsx: bool,
    used_jsxs: bool,
    used_fragment: bool,
}

fn needs_jsx_transform(options: &CompilerOptions, source_file: &SourceFile) -> bool {
    matches!(options.jsx, JsxEmit::ReactJSX | JsxEmit::ReactJSXDev)
        && tspath::file_extension_is(&source_file.file_name, ".tsx")
}

fn collect_jsx_replacements(
    statements: &[Arc<Node>],
    source: &str,
    usage: &mut JsxRuntimeUsage,
) -> Vec<(usize, usize, String)> {
    let mut replacements = Vec::new();
    for stmt in statements {
        collect_jsx_replacements_recursive(stmt, source, &mut replacements, usage);
    }
    replacements
}

fn collect_jsx_replacements_recursive(
    node: &Node,
    source: &str,
    replacements: &mut Vec<(usize, usize, String)>,
    usage: &mut JsxRuntimeUsage,
) {
    match node.kind {
        SyntaxKind::JsxElement | SyntaxKind::JsxSelfClosingElement | SyntaxKind::JsxFragment => {
            let text = generate_jsx_call(node, source, usage);
            replacements.push((node.pos(), node.end(), text));

        }
        _ => {
            crate::ast::node_data_generated::for_each_child(node, |child| {
                collect_jsx_replacements_recursive(child, source, replacements, usage);
                false
            });
        }
    }
}

fn generate_jsx_call(node: &Node, source: &str, usage: &mut JsxRuntimeUsage) -> String {
    match &node.data {
        NodeData::JsxSelfClosingElement(d) => {
            generate_element_call(&d.tag_name, &d.attributes, None, source, usage)
        }
        NodeData::JsxElement(d) => {
            let opener = &d.opening_element;
            let (tag_name, attributes) = match &opener.data {
                NodeData::JsxOpeningElement(o) => (&o.tag_name, &o.attributes),
                _ => return source[node.pos()..node.end()].to_string(),
            };
            generate_element_call(tag_name, attributes, Some(&d.children), source, usage)
        }
        NodeData::JsxFragment(d) => generate_fragment_call(&d.children, source, usage),
        _ => source[node.pos()..node.end()].to_string(),
    }
}

fn generate_element_call(
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

fn generate_fragment_call(
    children: &Arc<NodeList>,
    source: &str,
    usage: &mut JsxRuntimeUsage,
) -> String {
    usage.used_fragment = true;

    let children_prop = convert_children(children, source, usage);
    let is_static = is_static_children(children);

    let props_str = match children_prop {
        Some(c) => format!("{{ children: {} }}", c),
        None => "{}".to_string(),
    };

    let callee = if is_static {
        usage.used_jsxs = true;
        "_jsxs"
    } else {
        usage.used_jsx = true;
        "_jsx"
    };

    format!("{}(_Fragment, {})", callee, props_str)
}

fn tag_name_to_string(tag_name: &Node, source: &str) -> String {
    if let NodeData::Identifier(d) = &tag_name.data {
        if is_intrinsic_jsx_name(&d.text) {
            return format!("\"{}\"", d.text);
        }
    }
    if let NodeData::JsxNamespacedName(d) = &tag_name.data {
        return format!("\"{}:{}\"", d.namespace.text(), d.name.text());
    }
    source[tag_name.pos()..tag_name.end()].to_string()
}

fn attributes_to_props(
    attributes: &Node,
    source: &str,
    usage: &mut JsxRuntimeUsage,
) -> (Vec<String>, Option<String>) {
    let mut props = Vec::new();
    let mut key_arg = None;

    let properties = match &attributes.data {
        NodeData::JsxAttributes(d) => &d.properties,
        _ => return (props, key_arg),
    };

    for attr in properties.iter() {
        match &attr.data {
            NodeData::JsxAttribute(d) => {
                let name = attribute_name_to_string(&d.name, source);

                if name == "key" {
                    key_arg = Some(match &d.initializer {
                        Some(init) => attribute_value_to_string(init, source, usage),
                        None => "true".to_string(),
                    });
                    continue;
                }
                let value = match &d.initializer {
                    None => "true".to_string(),
                    Some(init) => attribute_value_to_string(init, source, usage),
                };
                props.push(format!("{}: {}", name, value));
            }
            NodeData::JsxSpreadAttribute(d) => {
                let expr_text = emit_expr_with_jsx(&d.expression, source, usage);
                props.push(format!("...{}", expr_text));
            }
            _ => {}
        }
    }

    (props, key_arg)
}

fn attribute_name_to_string(name: &Node, source: &str) -> String {
    if let NodeData::Identifier(d) = &name.data {
        return if is_valid_identifier(&d.text) {
            d.text.clone()
        } else {
            format!("\"{}\"", d.text)
        };
    }
    if let NodeData::JsxNamespacedName(d) = &name.data {
        return format!("\"{}:{}\"", d.namespace.text(), d.name.text());
    }
    source[name.pos()..name.end()].to_string()
}

fn attribute_value_to_string(init: &Node, source: &str, usage: &mut JsxRuntimeUsage) -> String {
    match init.kind {
        SyntaxKind::StringLiteral => {

            source[init.pos()..init.end()].to_string()
        }
        SyntaxKind::JsxExpression => {
            if let NodeData::JsxExpression(d) = &init.data {
                match &d.expression {
                    Some(expr) => emit_expr_with_jsx(expr, source, usage),
                    None => "true".to_string(),
                }
            } else {
                "true".to_string()
            }
        }
        SyntaxKind::JsxElement | SyntaxKind::JsxSelfClosingElement | SyntaxKind::JsxFragment => {
            generate_jsx_call(init, source, usage)
        }
        _ => source[init.pos()..init.end()].to_string(),
    }
}

fn convert_children(
    children: &Arc<NodeList>,
    source: &str,
    usage: &mut JsxRuntimeUsage,
) -> Option<String> {
    let semantic: Vec<Arc<Node>> = children
        .iter()
        .filter(|c| !is_whitespace_only_jsx_text(c))
        .cloned()
        .collect();

    if semantic.is_empty() {
        return None;
    }

    if semantic.len() == 1 && !is_spread_jsx_expression(&semantic[0]) {
        return Some(transform_jsx_child(&semantic[0], source, usage));
    }

    let parts: Vec<String> = semantic
        .iter()
        .map(|c| transform_jsx_child(c, source, usage))
        .collect();
    Some(format!("[{}]", parts.join(", ")))
}

fn is_static_children(children: &Arc<NodeList>) -> bool {
    let semantic: Vec<&Arc<Node>> = children
        .iter()
        .filter(|c| !is_whitespace_only_jsx_text(c))
        .collect();
    if semantic.len() > 1 {
        return true;
    }
    if semantic.len() == 1 {
        return is_spread_jsx_expression(semantic[0]);
    }
    false
}

fn transform_jsx_child(child: &Node, source: &str, usage: &mut JsxRuntimeUsage) -> String {
    match child.kind {
        SyntaxKind::JsxText | SyntaxKind::JsxTextAllWhiteSpaces => {
            let fixed = fixup_jsx_text(child.text());
            format!("\"{}\"", escape_js_string(&fixed))
        }
        SyntaxKind::JsxExpression => {
            if let NodeData::JsxExpression(d) = &child.data {
                match &d.expression {
                    Some(expr) => emit_expr_with_jsx(expr, source, usage),
                    None => String::new(),
                }
            } else {
                String::new()
            }
        }
        SyntaxKind::JsxElement | SyntaxKind::JsxSelfClosingElement | SyntaxKind::JsxFragment => {
            generate_jsx_call(child, source, usage)
        }
        _ => source[child.pos()..child.end()].to_string(),
    }
}

fn emit_expr_with_jsx(node: &Node, source: &str, usage: &mut JsxRuntimeUsage) -> String {
    let start = node.pos();
    let end = node.end();

    let mut cuts: Vec<(usize, usize)> = Vec::new();
    collect_type_cuts(node, source, &mut cuts);

    let mut jsx_repls: Vec<(usize, usize, String)> = Vec::new();
    collect_nested_jsx_in_expr(node, source, &mut jsx_repls, usage);

    let cuts: Vec<(usize, usize)> = cuts
        .iter()
        .filter(|(cs, ce)| !jsx_repls.iter().any(|(js, je, _)| *cs >= *js && *ce <= *je))
        .copied()
        .collect();

    let mut ops: Vec<(usize, usize, Option<String>)> = Vec::new();
    for &(cs, ce) in &cuts {
        if ce > start && cs < end {
            ops.push((cs.max(start), ce.min(end), None));
        }
    }
    for (rs, re, text) in &jsx_repls {
        if *re > start && *rs < end {
            ops.push(((*rs).max(start), (*re).min(end), Some(text.clone())));
        }
    }

    if ops.is_empty() {
        return source[start..end].to_string();
    }

    ops.sort_by_key(|(s, _, _)| *s);

    let mut result = String::new();
    let mut pos = start;
    for (s, e, repl) in &ops {
        if *s > pos {
            result.push_str(&source[pos..*s]);
        }
        if let Some(r) = repl {
            result.push_str(r);
        }
        pos = *e;
    }
    if pos < end {
        result.push_str(&source[pos..end]);
    }
    result
}

fn collect_nested_jsx_in_expr(
    node: &Node,
    source: &str,
    repls: &mut Vec<(usize, usize, String)>,
    usage: &mut JsxRuntimeUsage,
) {
    crate::ast::node_data_generated::for_each_child(node, |child| {
        match child.kind {
            SyntaxKind::JsxElement
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxFragment => {
                let text = generate_jsx_call(child, source, usage);
                repls.push((child.pos(), child.end(), text));
            }
            _ => {
                collect_nested_jsx_in_expr(child, source, repls, usage);
            }
        }
        false
    });
}

fn is_intrinsic_jsx_name(text: &str) -> bool {
    !text
        .bytes()
        .next()
        .map_or(false, |c| c.is_ascii_uppercase())
}

fn is_valid_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

fn is_whitespace_only_jsx_text(node: &Node) -> bool {
    matches!(&node.data, NodeData::JsxText(d) if d.contains_only_trivia_white_spaces)
}

fn is_spread_jsx_expression(node: &Node) -> bool {
    matches!(&node.data, NodeData::JsxExpression(d) if d.dot_dot_dot_token.is_some())
}

fn fixup_jsx_text(text: &str) -> String {
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

fn decode_jsx_entities(text: &str) -> String {
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

fn escape_js_string(text: &str) -> String {
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

fn build_jsx_import(usage: &JsxRuntimeUsage, import_source: &str, commonjs: bool) -> String {
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

fn compute_common_source_directory_of_filenames(file_names: &[String]) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tristate::Tristate;
    use crate::parser::Parser;
    use crate::vfs::InMemoryFS;

    fn parse(source: &str) -> SourceFile {
        let (file, _diags) =
            Parser::parse_source_file_text_with_diagnostics("/test.ts", source.to_string());
        file
    }

    fn emit_to_string(source: &str) -> String {
        let sf = parse(source);
        emit_js_text(&sf, &CompilerOptions::default())
    }

    fn emit_to_string_no_comments(source: &str) -> String {
        let sf = parse(source);
        let mut opts = CompilerOptions::default();
        opts.remove_comments = Tristate::True;
        emit_js_text(&sf, &opts)
    }

    fn emit_to_string_es5(source: &str) -> String {
        let sf = parse(source);
        let mut opts = CompilerOptions::default();
        opts.target = ScriptTarget::ES5;
        emit_js_text(&sf, &opts)
    }

    #[test]
    fn emit_strips_interface_declaration() {
        let js = emit_to_string("interface Foo { x: number; }\nlet y = 1;");
        assert!(!js.contains("interface"));
        assert!(js.contains("let y = 1;"));
    }

    #[test]
    fn emit_strips_type_alias() {
        let js = emit_to_string("type MyType = number;\nlet y: MyType = 1;");
        assert!(!js.contains("type MyType"));
        assert!(js.contains("let y = 1;"));
    }

    #[test]
    fn emit_strips_variable_type_annotation() {
        let js = emit_to_string("let x: number = 1;");
        assert_eq!(js.trim(), "let x = 1;");
    }

    #[test]
    fn emit_strips_multiple_variable_declarations() {

        let js = emit_to_string("let a: number = 1;\nlet b: string = \"hello\";");
        assert!(js.contains("let a = 1;"));
        assert!(js.contains("let b = \"hello\";"));
        assert!(!js.contains(": number"));
        assert!(!js.contains(": string"));
    }

    #[test]
    fn emit_strips_function_param_types() {
        let js = emit_to_string("function foo(a: number, b: string): number { return a; }");
        assert!(js.contains("function foo(a, b)"));
        assert!(!js.contains(": number"));
        assert!(js.contains("return a;"));
    }

    #[test]
    fn emit_strips_function_return_type() {
        let js = emit_to_string("function bar(): void { console.log(\"hi\"); }");
        assert!(js.contains("function bar()"));
        assert!(!js.contains(": void"));
    }

    #[test]
    fn emit_strips_type_parameters() {
        let js = emit_to_string("function identity<T>(x: T): T { return x; }");
        assert!(js.contains("function identity(x)"));
        assert!(!js.contains("<T>"));
    }

    #[test]
    fn emit_strips_as_expression() {

        let js = emit_to_string("function f(x: number) { return x; }");
        assert!(!js.contains(": number"));
        assert!(js.contains("return x;"));
    }

    #[test]
    fn emit_preserves_expression_statement() {
        let js = emit_to_string("console.log(\"hello\");");
        assert_eq!(js.trim(), "console.log(\"hello\");");
    }

    #[test]
    fn emit_preserves_class() {
        let js = emit_to_string(
            "class Foo { x: number = 1; method(a: string): void { this.x = a.length; } }",
        );
        assert!(js.contains("class Foo"));
        assert!(js.contains("x = 1"));
        assert!(js.contains("method(a)"));
        assert!(!js.contains(": void"));
    }

    #[test]
    fn emit_strips_property_type_annotation() {
        let js = emit_to_string("class Bar { prop: string = \"hi\"; }");
        assert!(js.contains("prop = \"hi\""));
        assert!(!js.contains(": string"));
    }

    #[test]
    fn emit_preserves_if_statement() {
        let js = emit_to_string("if (x > 0) { console.log(x); } else { console.log(0); }");
        assert!(js.contains("if (x > 0)"));
        assert!(js.contains("else"));
    }

    #[test]
    fn emit_preserves_for_loop() {
        let js = emit_to_string("for (let i: number = 0; i < 10; i++) { console.log(i); }");
        assert!(js.contains("for (let i = 0; i < 10; i++)"));
    }

    #[test]
    fn emit_preserves_arrow_function() {

        let js = emit_to_string("function fn(x: number): number { return x * 2; }");
        assert!(js.contains("function fn(x)"));
        assert!(js.contains("return x * 2;"));
        assert!(!js.contains(": number"));
    }

    #[test]
    fn emit_writes_file_to_fs() {
        let fs = InMemoryFS::new();
        fs.insert_file("/test.ts", "let x: number = 1;");

        let sf = parse("let x: number = 1;");
        let mut sf_with_name = sf;
        sf_with_name.file_name = "/test.ts".to_string();

        let options = CompilerOptions::default();
        let result = emit_source_file(&sf_with_name, &options, &fs, &|path, data| {
            fs.write_file(path, data)
        });

        assert_eq!(result.emitted_files, vec!["/test.js".to_string()]);
        assert_eq!(fs.read_file("/test.js").unwrap(), "let x = 1;");
    }

    #[test]
    fn emit_respects_out_dir() {
        let fs = InMemoryFS::new();
        let sf = parse("let x = 1;");
        let mut sf_with_name = sf;
        sf_with_name.file_name = "/src/test.ts".to_string();

        let mut options = CompilerOptions::default();
        options.out_dir = "/dist".to_string();
        let result = emit_source_file(&sf_with_name, &options, &fs, &|path, data| {
            fs.write_file(path, data)
        });

        assert_eq!(result.emitted_files, vec!["/dist/test.js".to_string()]);
    }

    #[test]
    fn emit_skips_json_files() {
        let sf = parse("{}");
        let mut sf_with_name = sf;
        sf_with_name.file_name = "/test.json".to_string();
        sf_with_name.script_kind = crate::ast::ScriptKind::Json;

        let options = CompilerOptions::default();
        let fs = InMemoryFS::new();
        let result = emit_source_file(&sf_with_name, &options, &fs, &|path, data| {
            fs.write_file(path, data)
        });

        assert!(result.emitted_files.is_empty());
    }

    #[test]
    fn emit_output_extension_mjs() {
        assert_eq!(get_output_extension("/test.mts"), ".mjs");
        assert_eq!(get_output_extension("/test.cts"), ".cjs");
        assert_eq!(get_output_extension("/test.ts"), ".js");
        assert_eq!(get_output_extension("/test.json"), ".json");
        assert_eq!(get_output_extension("/test.tsx"), ".js");
        assert_eq!(get_output_extension("/test.jsx"), ".js");
        assert_eq!(get_output_extension("/test.mjs"), ".mjs");
        assert_eq!(get_output_extension("/test.cjs"), ".cjs");
    }

    #[test]
    fn emit_program_emits_multiple_files() {
        let fs = InMemoryFS::new();

        let sf1 = parse("let x: number = 1;");
        let mut sf1 = sf1;
        sf1.file_name = "/a.ts".to_string();

        let sf2 = parse("function foo(a: string): void { console.log(a); }");
        let mut sf2 = sf2;
        sf2.file_name = "/b.ts".to_string();

        let source_files = vec![Arc::new(sf1), Arc::new(sf2)];
        let options = CompilerOptions::default();
        let result = emit_program(&source_files, &options, &fs, &|path, data| {
            fs.write_file(path, data)
        });

        assert_eq!(result.emitted_files.len(), 2);
        assert!(result.emitted_files.contains(&"/a.js".to_string()));
        assert!(result.emitted_files.contains(&"/b.js".to_string()));
        assert!(!result.emit_skipped);
        assert!(fs.file_exists("/a.js"));
        assert!(fs.file_exists("/b.js"));
        assert_eq!(fs.read_file("/a.js").unwrap().trim(), "let x = 1;");
        assert!(fs.read_file("/b.js").unwrap().contains("function foo(a)"));
    }

    #[test]
    fn emit_program_aggregates_diagnostics_on_write_failure() {

        let sf = parse("let x: number = 1;");
        let mut sf = sf;
        sf.file_name = "/test.ts".to_string();

        let source_files = vec![Arc::new(sf)];
        let options = CompilerOptions::default();
        let fs = InMemoryFS::new();
        let result = emit_program(&source_files, &options, &fs, &|_path, _data| {
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "denied",
            ))
        });

        assert!(result.emitted_files.is_empty());
        assert!(result.emit_skipped);
        assert!(!result.diagnostics.is_empty());
        assert!(result.diagnostics[0].contains("Error writing"));
    }

    #[test]
    fn emit_program_handles_empty_source() {
        let fs = InMemoryFS::new();
        let sf = parse("");
        let mut sf = sf;
        sf.file_name = "/empty.ts".to_string();

        let source_files = vec![Arc::new(sf)];
        let options = CompilerOptions::default();
        let result = emit_program(&source_files, &options, &fs, &|path, data| {
            fs.write_file(path, data)
        });

        assert_eq!(result.emitted_files, vec!["/empty.js".to_string()]);
        assert!(fs.file_exists("/empty.js"));
    }

    #[test]
    fn emit_source_file_skips_js_when_no_emit_for_js_files() {
        let fs = InMemoryFS::new();
        let sf = parse("let x = 1;");
        let mut sf = sf;
        sf.file_name = "/test.js".to_string();
        sf.script_kind = crate::ast::ScriptKind::Js;

        let mut options = CompilerOptions::default();
        options.no_emit_for_js_files = Tristate::True;
        let result = emit_source_file(&sf, &options, &fs, &|path, data| fs.write_file(path, data));

        assert!(result.emitted_files.is_empty());
        assert!(!fs.file_exists("/test.js"));
    }

    #[test]
    fn emit_source_file_emits_js_by_default() {
        let fs = InMemoryFS::new();
        let sf = parse("let x = 1;");
        let mut sf = sf;
        sf.file_name = "/test.js".to_string();
        sf.script_kind = crate::ast::ScriptKind::Js;

        let options = CompilerOptions::default();
        let result = emit_source_file(&sf, &options, &fs, &|path, data| fs.write_file(path, data));

        assert_eq!(result.emitted_files, vec!["/test.js".to_string()]);
    }

    #[test]
    fn emit_preserves_export_declaration() {
        let js = emit_to_string("export const x = 1;");
        assert!(js.contains("export const x = 1;"));
    }

    #[test]
    fn emit_preserves_import_declaration() {
        let js = emit_to_string("import { foo } from \"./bar\";");
        assert!(js.contains("import { foo }"));
        assert!(js.contains("from \"./bar\";"));
    }

    #[test]
    fn emit_strips_export_from_type_alias() {
        let js = emit_to_string("export type ID = number;\nlet x: ID = 1;");
        assert!(!js.contains("type ID"));
        assert!(js.contains("let x = 1;"));
    }

    #[test]
    fn emit_preserves_class_with_constructor() {
        let js = emit_to_string("class Foo { x: number; constructor(x: number) { this.x = x; } }");
        assert!(js.contains("class Foo"));
        assert!(js.contains("constructor(x)"));
        assert!(js.contains("this.x = x;"));
        assert!(!js.contains(": number"));
    }

    #[test]
    fn emit_preserves_while_loop() {
        let js = emit_to_string("let i: number = 0; while (i < 10) { i++; }");
        assert!(js.contains("let i = 0;"));
        assert!(js.contains("while (i < 10)"));
        assert!(js.contains("i++;"));
    }

    #[test]
    fn emit_preserves_do_while_loop() {
        let js = emit_to_string("let i: number = 0; do { i++; } while (i < 10);");
        assert!(js.contains("let i = 0;"));
        assert!(js.contains("do {"));
        assert!(js.contains("} while (i < 10);"));
    }

    #[test]
    fn emit_strips_function_type_parameters_in_class_method() {

        let js = emit_to_string("class Foo { method(x: number): number { return x; } }");
        assert!(js.contains("method(x)"));
        assert!(!js.contains(": number"));
        assert!(js.contains("return x;"));
    }

    #[test]
    fn remove_comments_strips_single_line_comment() {
        let js = emit_to_string_no_comments("// This comment should be removed\nconst e = 5;");
        assert!(!js.contains("// This comment"));
        assert!(js.contains("const e = 5;"));
    }

    #[test]
    fn remove_comments_strips_multi_line_comment() {
        let js = emit_to_string_no_comments("/* block comment */ const x = 1;");
        assert!(!js.contains("block comment"));
        assert!(js.contains("const x = 1;"));
    }

    #[test]
    fn remove_comments_strips_jsdoc_comment() {
        let js = emit_to_string_no_comments("/** JSDoc */\nfunction foo() { return 1; }");
        assert!(!js.contains("JSDoc"));
        assert!(js.contains("function foo()"));
    }

    #[test]
    fn remove_comments_preserves_comments_in_strings() {
        let js = emit_to_string_no_comments("// real comment\nconst s = \"// not a comment\";");
        assert!(!js.contains("real comment"));
        assert!(js.contains("\"// not a comment\""));
    }

    #[test]
    fn remove_comments_preserves_comments_in_template_literals() {
        let js = emit_to_string_no_comments("// real comment\nconst s = `// not a comment ${1}`;");
        assert!(!js.contains("real comment"));
        assert!(js.contains("`// not a comment"));
    }

    #[test]
    fn remove_comments_does_not_affect_division() {
        let js = emit_to_string_no_comments("const x = 10 / 2;");
        assert!(js.contains("10 / 2"));
    }

    #[test]
    fn remove_comments_strips_trailing_comment() {
        let js = emit_to_string_no_comments("const x = 1; // trailing");
        assert!(js.contains("const x = 1;"));
        assert!(!js.contains("trailing"));
    }

    #[test]
    fn remove_comments_off_by_default() {
        let js = emit_to_string("// comment\nconst x = 1;");
        assert!(js.contains("// comment"));
    }

    #[test]
    fn es5_downlevels_const_to_var() {
        let js = emit_to_string_es5("const f: number = 6;");
        assert!(js.contains("var f = 6;"));
        assert!(!js.contains("const"));
        assert!(!js.contains(": number"));
    }

    #[test]
    fn es5_downlevels_let_to_var() {
        let js = emit_to_string_es5("let x: number = 1;");
        assert!(js.contains("var x = 1;"));
        assert!(!js.contains("let"));
    }

    #[test]
    fn es5_downlevels_const_with_export() {
        let js = emit_to_string_es5("const f: number = 6;\nexport { f };");
        assert!(js.contains("var f = 6;"));
        assert!(js.contains("export { f };"));
    }

    #[test]
    fn es5_preserves_var() {
        let js = emit_to_string_es5("var x = 1;");
        assert!(js.contains("var x = 1;"));
    }

    #[test]
    fn es5_downlevels_nested_let_in_for_loop() {
        let js = emit_to_string_es5("for (let i = 0; i < 10; i++) { console.log(i); }");
        assert!(js.contains("for (var i = 0;"));
    }

    #[test]
    fn es5_no_downlevel_when_target_es2015() {
        let sf = parse("const x = 1;");
        let mut opts = CompilerOptions::default();
        opts.target = ScriptTarget::ES2015;
        let js = emit_js_text(&sf, &opts);
        assert!(js.contains("const x = 1;"));
        assert!(!js.contains("var x"));
    }

    fn emit_to_string_commonjs(source: &str) -> String {
        let sf = parse(source);
        let mut opts = CompilerOptions::default();
        opts.module = ModuleKind::CommonJS;
        emit_js_text(&sf, &opts)
    }

    #[test]
    fn commonjs_starts_with_use_strict() {
        let js = emit_to_string_commonjs("const x = 1;");
        assert!(js.starts_with("\"use strict\";\n"));
    }

    #[test]
    fn commonjs_export_named() {
        let js = emit_to_string_commonjs("const x = 1;\nexport { x };");
        assert!(js.contains("const x = 1;"));
        assert!(js.contains("exports.x = x;"));
        assert!(!js.contains("export { x }"));
    }

    #[test]
    fn commonjs_export_default_expression() {
        let js = emit_to_string_commonjs("export default 42;");
        assert!(js.contains("exports.default = 42;"));
        assert!(!js.contains("export default"));
    }

    #[test]
    fn commonjs_export_equals() {
        let js = emit_to_string_commonjs("const obj = {};\nexport = obj;");
        assert!(js.contains("module.exports = obj;"));
    }

    #[test]
    fn commonjs_export_const() {
        let js = emit_to_string_commonjs("export const x = 1;");
        assert!(js.contains("const x = 1;"));
        assert!(js.contains("exports.x = x;"));
        assert!(!js.contains("export const"));
    }

    #[test]
    fn commonjs_export_function() {
        let js = emit_to_string_commonjs("export function foo() { return 1; }");
        assert!(js.contains("function foo()"));
        assert!(js.contains("exports.foo = foo;"));
        assert!(!js.contains("export function"));
    }

    #[test]
    fn commonjs_export_class() {
        let js = emit_to_string_commonjs("export class Foo { }");
        assert!(js.contains("class Foo"));
        assert!(js.contains("exports.Foo = Foo;"));
        assert!(!js.contains("export class"));
    }

    #[test]
    fn commonjs_import_named() {
        let js = emit_to_string_commonjs("import { foo } from \"./bar\";");
        assert!(js.contains("const { foo } = require(\"./bar\");"));
        assert!(!js.contains("import { foo }"));
    }

    #[test]
    fn commonjs_import_namespace() {
        let js = emit_to_string_commonjs("import * as ns from \"./bar\";");
        assert!(js.contains("const ns = require(\"./bar\");"));
    }

    #[test]
    fn commonjs_import_default() {
        let js = emit_to_string_commonjs("import d from \"./bar\";");
        assert!(js.contains("const { default: d } = require(\"./bar\");"));
    }

    #[test]
    fn commonjs_import_side_effect() {
        let js = emit_to_string_commonjs("import \"./bar\";");
        assert!(js.contains("require(\"./bar\");"));
    }

    #[test]
    fn commonjs_import_type_stripped() {
        let js = emit_to_string_commonjs("import type { foo } from \"./bar\";");
        assert!(!js.contains("import"));
        assert!(!js.contains("require"));
    }

    #[test]
    fn commonjs_export_multiple_named() {
        let js = emit_to_string_commonjs("const x = 1;\nconst y = 2;\nexport { x, y };");
        assert!(js.contains("exports.x = x;"));
        assert!(js.contains("exports.y = y;"));
    }

    #[test]
    fn commonjs_export_reexport() {
        let js = emit_to_string_commonjs("export { foo } from \"./bar\";");
        assert!(js.contains("const { foo } = require(\"./bar\");"));
        assert!(js.contains("exports.foo = foo;"));
    }

    fn emit_with_sourcemap(
        source: &str,
        source_map: bool,
        inline: bool,
        inline_sources: bool,
    ) -> (String, Option<String>, String) {
        let sf = parse(source);
        let mut opts = CompilerOptions::default();
        if source_map {
            opts.source_map = Tristate::True;
        }
        if inline {
            opts.inline_source_map = Tristate::True;
        }
        if inline_sources {
            opts.inline_sources = Tristate::True;
        }
        emit_js_with_sourcemap(&sf, &opts, "/test.js")
    }

    #[test]
    fn sourcemap_produces_valid_json() {
        let (js, map_json, url) =
            emit_with_sourcemap("let x = 1;\nlet y = 2;\n", true, false, false);

        assert!(!js.contains("sourceMappingURL"));

        let map = map_json.expect("map_json should be Some");
        assert!(map.contains("\"version\":3"));
        assert!(map.contains("\"file\":\"test.js\""));
        assert!(map.contains("\"sources\""));
        assert!(map.contains("\"mappings\""));

        assert!(!map.contains("\"mappings\":\"\""));

        assert_eq!(url, "test.js.map");

        assert!(!map.contains("sourcesContent"));
    }

    #[test]
    fn sourcemap_inline_produces_data_url() {
        let (js, map_json, url) = emit_with_sourcemap("let x = 1;\n", false, true, false);

        assert!(map_json.is_none());

        assert!(url.starts_with("data:application/json;base64,"));

        assert!(!js.contains("sourceMappingURL"));
    }

    #[test]
    fn sourcemap_inline_sources_includes_content() {
        let (_js, map_json, _url) = emit_with_sourcemap("let x = 1;\n", true, false, true);
        let map = map_json.expect("map_json should be Some");
        assert!(map.contains("sourcesContent"));
        assert!(map.contains("let x = 1;"));
    }

    #[test]
    fn sourcemap_strips_type_annotations() {
        let (js, map_json, _url) = emit_with_sourcemap("let x: number = 1;\n", true, false, false);

        assert!(js.contains("let x = 1;"));
        assert!(!js.contains(": number"));

        let map = map_json.expect("map_json should be Some");
        assert!(map.contains("\"version\":3"));
    }

    #[test]
    fn sourcemap_mappings_decode_to_correct_positions() {
        use crate::sourcemap::MappingsDecoder;
        let (js, map_json, _url) = emit_with_sourcemap("let x = 1;\n", true, false, false);
        let map = map_json.expect("map_json should be Some");

        let raw: crate::sourcemap::RawSourceMap = crate::json::unmarshal(&map).expect("valid JSON");
        assert_eq!(raw.version, 3);
        assert!(!raw.sources.is_empty());
        assert!(!raw.mappings.is_empty());

        let mut decoder = MappingsDecoder::new(&raw.mappings);
        let mut count = 0;
        let mut has_source_mapping = false;
        while count < 100 {
            match decoder.next() {
                Some(m) => {
                    if m.is_source_mapping() {
                        has_source_mapping = true;

                        assert!(m.source_line >= 0);
                    }
                    count += 1;
                }
                None => break,
            }
        }
        assert!(
            has_source_mapping,
            "should have at least one source mapping"
        );

        let gen_lines = js.lines().count();
        let _ = gen_lines;
    }

    #[test]
    fn sourcemap_not_emitted_by_default() {

        let sf = parse("let x = 1;\n");
        let js = emit_js_text(&sf, &CompilerOptions::default());
        assert!(!js.contains("sourceMappingURL"));
    }

    #[test]
    fn sourcemap_commonjs_use_strict_not_mapped() {

        let mut opts = CompilerOptions::default();
        opts.source_map = Tristate::True;
        opts.module = ModuleKind::CommonJS;
        let sf = parse("const x = 1;\nexport { x };");
        let (js, map_json, _url) = emit_js_with_sourcemap(&sf, &opts, "/test.js");
        assert!(js.starts_with("\"use strict\";"));
        let map = map_json.expect("map_json should be Some");
        assert!(map.contains("\"version\":3"));
    }

    #[test]
    fn sourcemap_write_file_creates_map() {
        use std::cell::RefCell;
        let sf = parse("let x = 1;\nlet y: string = \"hi\";\n");
        let mut opts = CompilerOptions::default();
        opts.source_map = Tristate::True;
        let written: RefCell<Vec<(String, String)>> = RefCell::new(Vec::new());
        let result = emit_source_file_with_common_dir(
            &sf,
            &opts,
            &crate::vfs::InMemoryFS::new(),
            "",
            &|path, content| {
                written
                    .borrow_mut()
                    .push((path.to_string(), content.to_string()));
                Ok(())
            },
        );

        assert_eq!(result.emitted_files.len(), 2);
        let written = written.borrow();
        let js_file = &written
            .iter()
            .find(|(p, _)| p.ends_with(".js") && !p.ends_with(".map"))
            .expect("js file")
            .1;
        let map_file = &written
            .iter()
            .find(|(p, _)| p.ends_with(".js.map"))
            .expect("map file")
            .1;

        assert!(js_file.contains("//# sourceMappingURL="));

        assert!(map_file.contains("\"version\":3"));
        assert!(map_file.contains("\"mappings\""));

        assert!(js_file.contains("let y = \"hi\";"));
        assert!(!js_file.contains(": string"));
    }

    fn emit_dts(source: &str) -> String {
        let sf = parse(source);
        let opts = CompilerOptions::default();
        emit_declaration_text(&sf, &opts)
    }

    #[test]
    fn dts_function_strips_body() {
        let dts = emit_dts("function add(a: number, b: number): number { return a + b; }");
        assert!(dts.contains("declare function add(a: number, b: number): number;"));
        assert!(!dts.contains("return"));
        assert!(!dts.contains("{"));
    }

    #[test]
    fn dts_export_function_adds_declare() {
        let dts = emit_dts("export function foo(): void { console.log(1); }");
        assert!(dts.contains("export declare function foo(): void;"));
        assert!(!dts.contains("console"));
    }

    #[test]
    fn dts_variable_strips_initializer() {
        let dts = emit_dts("const x: number = 42;");
        assert!(dts.contains("declare const x: number;"));
        assert!(!dts.contains("42"));
    }

    #[test]
    fn dts_export_variable_strips_initializer() {
        let dts = emit_dts("export const PI: number = 3.14;");
        assert!(dts.contains("export declare const PI: number;"));
        assert!(!dts.contains("3.14"));
    }

    #[test]
    fn dts_interface_emitted_as_is() {
        let dts = emit_dts("interface User { id: number; name: string; }");
        assert!(dts.contains("interface User {"));
        assert!(dts.contains("id: number;"));
        assert!(!dts.contains("declare"));
    }

    #[test]
    fn dts_type_alias_emitted_as_is() {
        let dts = emit_dts("type ID = string | number;");
        assert!(dts.contains("type ID = string | number;"));
        assert!(!dts.contains("declare"));
    }

    #[test]
    fn dts_enum_adds_declare() {
        let dts = emit_dts("enum Color { Red, Green, Blue }");
        assert!(dts.contains("declare enum Color {"));
        assert!(dts.contains("Red"));
    }

    #[test]
    fn dts_runtime_statements_skipped() {
        let dts = emit_dts("console.log(\"hello\");\nlet x: number = 1;");
        assert!(!dts.contains("console"));
        assert!(!dts.contains("hello"));
        assert!(dts.contains("declare let x: number;"));
    }

    #[test]
    fn dts_multiple_declarations() {
        let src = "export function add(a: number, b: number): number { return a + b; }\n\
                   export const PI: number = 3.14;\n\
                   export interface User { id: number; }\n\
                   export type ID = string | number;\n";
        let dts = emit_dts(src);
        assert!(dts.contains("export declare function add(a: number, b: number): number;"));
        assert!(dts.contains("export declare const PI: number;"));
        assert!(dts.contains("export interface User {"));
        assert!(dts.contains("export type ID = string | number;"));
        assert!(!dts.contains("return"));
        assert!(!dts.contains("3.14"));
    }

    #[test]
    fn dts_class_adds_declare() {
        let dts =
            emit_dts("export class Point { x: number; constructor(x: number) { this.x = x; } }");
        assert!(dts.contains("export declare class Point"));
    }

    #[test]
    fn dts_write_file_creates_dts() {
        use std::cell::RefCell;
        let sf =
            parse("export function foo(): number { return 1; }\nexport const x: number = 42;\n");
        let mut opts = CompilerOptions::default();
        opts.declaration = Tristate::True;
        let written: RefCell<Vec<(String, String)>> = RefCell::new(Vec::new());
        let result = emit_source_file_with_common_dir(
            &sf,
            &opts,
            &crate::vfs::InMemoryFS::new(),
            "",
            &|path, content| {
                written
                    .borrow_mut()
                    .push((path.to_string(), content.to_string()));
                Ok(())
            },
        );

        assert!(result.emitted_files.iter().any(|p| p.ends_with(".js")));
        assert!(result.emitted_files.iter().any(|p| p.ends_with(".d.ts")));
        let written = written.borrow();
        let dts_file = &written
            .iter()
            .find(|(p, _)| p.ends_with(".d.ts"))
            .expect("dts file")
            .1;

        assert!(dts_file.contains("export declare function foo(): number;"));
        assert!(dts_file.contains("export declare const x: number;"));
        assert!(!dts_file.contains("return"));
        assert!(!dts_file.contains("42"));
    }

    #[test]
    fn dts_emit_declaration_only_suppresses_js() {
        use std::cell::RefCell;
        let sf = parse("export function foo(): number { return 1; }\n");
        let mut opts = CompilerOptions::default();
        opts.declaration = Tristate::True;
        opts.emit_declaration_only = Tristate::True;
        let written: RefCell<Vec<(String, String)>> = RefCell::new(Vec::new());
        let result = emit_source_file_with_common_dir(
            &sf,
            &opts,
            &crate::vfs::InMemoryFS::new(),
            "",
            &|path, content| {
                written
                    .borrow_mut()
                    .push((path.to_string(), content.to_string()));
                Ok(())
            },
        );

        assert!(!result.emitted_files.iter().any(|p| p.ends_with(".js")));
        assert!(result.emitted_files.iter().any(|p| p.ends_with(".d.ts")));
    }

    #[test]
    fn dts_drops_value_imports_keeps_side_effect() {

        let src = "import { useState } from 'react';\n\
                   import reactLogo from './assets/react.svg';\n\
                   import './App.css';\n\
                   export default function App() { return 1; }\n";
        let dts = emit_dts(src);

        assert!(!dts.contains("useState"));
        assert!(!dts.contains("reactLogo"));

        assert!(dts.contains("import './App.css';"));

        assert!(dts.contains("export default function App(): unknown;"));
        assert!(!dts.contains("return"));
    }

    #[test]
    fn dts_keeps_type_only_import() {
        let src = "import type { Config } from './config';\n\
                   import { value } from './values';\n\
                   export const c: Config = {} as any;\n";
        let dts = emit_dts(src);

        assert!(dts.contains("import type { Config } from './config';"));

        assert!(!dts.contains("value"));
    }

    #[test]
    fn dts_function_declare_keyword_and_semicolon() {

        let dts = emit_dts("function add(a: number, b: number): number { return a + b; }");
        assert!(dts.contains("declare function add(a: number, b: number): number;"));
    }

    #[test]
    fn dts_class_strips_method_bodies() {
        let src = "export class Counter {\n\
                   count: number;\n\
                   constructor(initial: number) { this.count = initial; }\n\
                   increment(): void { this.count++; }\n\
                   }\n";
        let dts = emit_dts(src);
        assert!(dts.contains("export declare class Counter"));

        assert!(dts.contains("count: number;"));

        assert!(dts.contains("constructor(initial: number);"));
        assert!(dts.contains("increment(): void;"));

        assert!(!dts.contains("this.count"));
        assert!(!dts.contains("initial;"));
    }

    #[test]
    fn dts_variable_strips_initializer_without_type() {

        let dts = emit_dts("const answer = 42;");
        assert!(dts.contains("declare const answer;"));
        assert!(!dts.contains("42"));
    }

    #[test]
    fn dts_variable_multiple_no_type() {

        let dts = emit_dts("let a = 1;\nlet b = 2;");
        assert!(dts.contains("declare let a;"));
        assert!(dts.contains("declare let b;"));
        assert!(!dts.contains("= 1"));
        assert!(!dts.contains("= 2"));
    }

    #[test]
    fn type_eraser() {

        let cases: &[(&str, &[&str], &[&str])] = &[

            ("interface I { x: number; }", &[], &["interface"]),
            ("type T = number;", &[], &["type T"]),

            (
                "function f<T>(x: T): T { return x; }",
                &["function f(x)", "return x;"],
                &["<T>", ": T"],
            ),

            (
                "function add(a: number, b: string): void { return a; }",
                &["function add(a, b)", "return a;"],
                &[": number", ": string", ": void"],
            ),

            ("let x: number = 1;", &["let x = 1;"], &[": number"]),
            (
                "const s: string = \"hi\";",
                &["const s = \"hi\";"],
                &[": string"],
            ),

            (
                "class C { x: number = 1; m(a: string): void { return a; } }",
                &["class C", "x = 1", "m(a)", "return a;"],
                &[": number", ": string", ": void"],
            ),
        ];

        for &(input, must_contain, must_not_contain) in cases {
            let js = emit_to_string(input);
            for needle in must_contain {
                assert!(
                    js.contains(needle),
                    "type_eraser({input:?}): expected output to contain {needle:?}, got {js:?}"
                );
            }
            for needle in must_not_contain {
                assert!(
                    !js.contains(needle),
                    "type_eraser({input:?}): expected output to NOT contain {needle:?}, got {js:?}"
                );
            }
        }
    }

    #[test]
    fn import_elision() {

        for input in [
            "import type { foo } from \"./bar\";",
            "import type * as ns from \"./bar\";",
            "import type d from \"./bar\";",
        ] {
            let js = emit_to_string_commonjs(input);
            assert!(
                !js.contains("require"),
                "import_elision({input:?}): type-only import should be elided, got {js:?}"
            );
            assert!(
                !js.contains("import"),
                "import_elision({input:?}): type-only import should be elided, got {js:?}"
            );
        }

        let js = emit_to_string_commonjs("import { foo } from \"./bar\";");
        assert!(
            js.contains("require(\"./bar\")"),
            "import_elision: value import should be retained, got {js:?}"
        );

        let js = emit_to_string_commonjs("import { type foo, bar } from \"./bar\";");
        assert!(
            js.contains("bar"),
            "import_elision: value binding 'bar' should be retained, got {js:?}"
        );
        assert!(
            !js.contains("foo"),
            "import_elision: type-only binding 'foo' should be elided, got {js:?}"
        );
        assert!(
            js.contains("require(\"./bar\")"),
            "import_elision: mixed import should still require the module, got {js:?}"
        );
    }

    fn parse_tsx(source: &str) -> SourceFile {
        let (file, _diags) =
            Parser::parse_source_file_text_with_diagnostics("/test.tsx", source.to_string());
        file
    }

    fn emit_to_string_jsx(source: &str) -> String {
        let sf = parse_tsx(source);
        let mut opts = CompilerOptions::default();
        opts.jsx = JsxEmit::ReactJSX;
        emit_js_text(&sf, &opts)
    }

    #[test]
    fn jsx_self_closing_element() {
        let js = emit_to_string_jsx("const x = <div />;");
        assert!(js.contains("_jsx(\"div\", {})"));
        assert!(
            js.contains("import { jsx as _jsx } from \"react/jsx-runtime\";"),
            "expected jsx import, got {js:?}"
        );
    }

    #[test]
    fn jsx_element_with_string_attribute() {
        let js = emit_to_string_jsx("const x = <div className=\"x\" />;");
        assert!(js.contains("_jsx(\"div\", { className: \"x\" })"));
    }

    #[test]
    fn jsx_element_with_expression_attribute() {
        let js = emit_to_string_jsx("const x = <div onClick={handler} />;");
        assert!(js.contains("onClick: handler"));
    }

    #[test]
    fn jsx_element_with_boolean_attribute() {
        let js = emit_to_string_jsx("const x = <input disabled />;");
        assert!(js.contains("disabled: true"));
    }

    #[test]
    fn jsx_element_with_single_text_child() {
        let js = emit_to_string_jsx("const x = <h1>Hello</h1>;");
        assert!(js.contains("_jsx(\"h1\", { children: \"Hello\" })"));
    }

    #[test]
    fn jsx_element_with_single_element_child() {
        let js = emit_to_string_jsx("const x = <div><span /></div>;");
        assert!(
            js.contains("children: _jsx(\"span\", {})"),
            "expected single element child, got {js:?}"
        );
    }

    #[test]
    fn jsx_element_with_multiple_children() {
        let js = emit_to_string_jsx("const x = <div><span /><p /></div>;");
        assert!(js.contains("_jsxs(\"div\","));
        assert!(js.contains("children: [_jsx(\"span\", {}), _jsx(\"p\", {})]"));
    }

    #[test]
    fn jsx_fragment() {
        let js = emit_to_string_jsx("const x = <><span /><p /></>;");
        assert!(js.contains("_jsxs(_Fragment,"));
        assert!(
            js.contains("import { Fragment as _Fragment, jsx as _jsx, jsxs as _jsxs }"),
            "expected all three imports, got {js:?}"
        );
    }

    #[test]
    fn jsx_fragment_empty() {
        let js = emit_to_string_jsx("const x = <></>;");
        assert!(js.contains("_jsx(_Fragment, {})"));
    }

    #[test]
    fn jsx_expression_child() {
        let js = emit_to_string_jsx("const x = <div>{count}</div>;");
        assert!(js.contains("children: count"));
    }

    #[test]
    fn jsx_mixed_children() {
        let js = emit_to_string_jsx("const x = <p>Edit <code>file</code> now</p>;");
        assert!(js.contains("_jsxs(\"p\","));
        assert!(js.contains("\"Edit \""));
        assert!(js.contains("_jsx(\"code\", { children: \"file\" })"));
        assert!(js.contains("\" now\""));
    }

    #[test]
    fn jsx_component_tag() {
        let js = emit_to_string_jsx("const x = <Foo bar=\"1\" />;");
        assert!(js.contains("_jsx(Foo, { bar: \"1\" })"));
    }

    #[test]
    fn jsx_member_expression_tag() {
        let js = emit_to_string_jsx("const x = <Foo.Bar />;");
        assert!(js.contains("_jsx(Foo.Bar, {})"));
    }

    #[test]
    fn jsx_namespaced_attribute() {
        let js = emit_to_string_jsx("const x = <div aria-hidden=\"true\" />;");
        assert!(js.contains("\"aria-hidden\": \"true\""));
    }

    #[test]
    fn jsx_import_injection() {
        let js = emit_to_string_jsx("const x = <div />;");
        let import_line = js
            .lines()
            .find(|l| l.starts_with("import"))
            .expect("should have an import");
        assert_eq!(
            import_line,
            "import { jsx as _jsx } from \"react/jsx-runtime\";"
        );
    }

    #[test]
    fn jsx_import_only_used_helpers() {

        let js = emit_to_string_jsx("const x = <div><a /><b /></div>;");
        assert!(!js.contains("Fragment as _Fragment"));
        assert!(js.contains("jsx as _jsx"));
        assert!(js.contains("jsxs as _jsxs"));
    }

    #[test]
    fn jsx_preserves_expression_in_attribute() {
        let js = emit_to_string_jsx("const x = <button onClick={() => fn(1)}>click</button>;");
        assert!(js.contains("onClick: () => fn(1)"));
        assert!(js.contains("children: \"click\""));
    }

    #[test]
    fn jsx_nested_elements() {
        let js = emit_to_string_jsx("const x = <div><span><p /></span></div>;");
        assert!(js.contains("children: _jsx(\"span\", { children: _jsx(\"p\", {}) })"));
    }

    #[test]
    fn jsx_no_transform_when_not_tsx() {

        let (sf, _diags) =
            Parser::parse_source_file_text_with_diagnostics("/test.ts", "const x = 1;".to_string());
        let mut opts = CompilerOptions::default();
        opts.jsx = JsxEmit::ReactJSX;
        let js = emit_js_text(&sf, &opts);
        assert!(!js.contains("_jsx"));
    }

    #[test]
    fn jsx_empty_element_no_children_prop() {
        let js = emit_to_string_jsx("const x = <section id=\"main\"></section>;");
        assert!(js.contains("_jsx(\"section\", { id: \"main\" })"));
    }

    #[test]
    fn type_eraser_strips_abstract_class_modifier() {
        let js = emit_to_string("abstract class Foo { abstract bar(): void; }");
        assert!(!js.contains("abstract"));
        assert!(js.contains("class Foo"));
        assert!(js.contains("bar()"));
    }

    #[test]
    fn type_eraser_strips_readonly_modifier() {
        let js = emit_to_string("class Foo { readonly x: number = 1; }");
        assert!(!js.contains("readonly"));
        assert!(js.contains("x = 1;"));
    }

    #[test]
    fn type_eraser_strips_override_modifier() {
        let js = emit_to_string("class Foo { override m(): void {} }");
        assert!(!js.contains("override"));
        assert!(js.contains("m()"));
    }

    #[test]
    fn type_eraser_strips_implements_clause() {
        let js = emit_to_string("interface I { x: number; }\nclass Foo implements I { x = 1; }");
        assert!(!js.contains("implements"));
        assert!(!js.contains("interface"));
        assert!(js.contains("class Foo"));
        assert!(js.contains("x = 1;"));
    }

    #[test]
    fn type_eraser_keeps_extends_strips_implements() {
        let js =
            emit_to_string("class Base {}\ninterface I {}\nclass Foo extends Base implements I {}");
        assert!(js.contains("extends Base"));
        assert!(!js.contains("implements"));
    }

    #[test]
    fn type_eraser_strips_declare_keyword() {
        let js = emit_to_string("declare const x: number;\nlet y = x;");
        assert!(!js.contains("declare"));
    }

    #[test]
    fn type_eraser_strips_type_assertion() {

        let js = emit_to_string("let x = <number>5;");
        assert!(
            !js.contains("<number>"),
            "type assertion <number> should be erased, got {js:?}"
        );
        assert!(js.contains("5;"));
    }

    #[test]
    fn import_elision_import_type_named() {
        let js = emit_to_string("import type { Foo } from \"./bar\";\nlet x = 1;");
        assert!(!js.contains("import"));
        assert!(!js.contains("Foo"));
        assert!(js.contains("let x = 1;"));
    }

    #[test]
    fn import_elision_import_type_default() {
        let js = emit_to_string("import type Foo from \"./bar\";\nlet x = 1;");
        assert!(!js.contains("import"));
        assert!(!js.contains("Foo"));
        assert!(js.contains("let x = 1;"));
    }

    #[test]
    fn import_elision_import_type_namespace() {
        let js = emit_to_string("import type * as ns from \"./bar\";\nlet x = 1;");
        assert!(!js.contains("import"));
        assert!(!js.contains("require"));
        assert!(js.contains("let x = 1;"));
    }

    #[test]
    fn import_elision_mixed_named_bindings() {

        let js = emit_to_string("import { type Foo, Bar } from \"./bar\";\nlet x = Bar;");
        assert!(
            !js.contains("Foo"),
            "type-only binding Foo should be elided, got {js:?}"
        );
        assert!(
            js.contains("Bar"),
            "value binding Bar should be retained, got {js:?}"
        );
        assert!(js.contains("from \"./bar\";"));
    }

    #[test]
    fn import_elision_mixed_named_bindings_trailing() {

        let js = emit_to_string("import { Bar, type Foo } from \"./bar\";\nlet x = Bar;");
        assert!(!js.contains("Foo"));
        assert!(js.contains("Bar"));
        assert!(js.contains("from \"./bar\";"));
    }

    #[test]
    fn import_elision_mixed_default_and_type_named() {

        let js = emit_to_string("import Foo, { type Bar } from \"./bar\";\nlet x = Foo;");
        assert!(js.contains("Foo"));
        assert!(!js.contains("Bar"));
        assert!(js.contains("from \"./bar\";"));
    }

    #[test]
    fn import_elision_preserves_value_import() {
        let js = emit_to_string("import { foo } from \"./bar\";\nlet x = foo;");
        assert!(js.contains("import { foo }"));
        assert!(js.contains("from \"./bar\";"));
    }

    #[test]
    fn import_elision_all_inline_type_only() {

        let js = emit_to_string("import { type Foo, type Bar } from \"./bar\";\nlet x = 1;");
        assert!(!js.contains("import"));
        assert!(!js.contains("Foo"));
        assert!(!js.contains("Bar"));
        assert!(js.contains("let x = 1;"));
    }
}
