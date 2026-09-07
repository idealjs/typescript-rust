#![allow(unused_imports)]

use super::*;

#[derive(Debug, Default)]
pub struct EmitResult {
    pub emit_skipped: bool,
    pub emitted_files: Vec<String>,
    pub diagnostics: Vec<String>,
}

pub struct EmitOptions {
    pub write_file: Option<Box<dyn Fn(&str, &str) -> std::io::Result<()> + Send + Sync>>,
}

pub(crate) fn compute_line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, b) in text.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

pub(crate) fn offset_to_line(line_starts: &[usize], offset: usize) -> (i32, usize) {
    let line = line_starts.partition_point(|&start| start <= offset) - 1;
    (line as i32, line_starts[line])
}

pub(crate) fn utf16_column(text: &str, line_start: usize, offset: usize) -> i32 {
    text[line_start..offset]
        .chars()
        .map(|c| c.len_utf16() as i32)
        .sum()
}

pub(crate) const UNMAPPED: u32 = u32::MAX;

pub(crate) struct SourceMapTracker<'a> {
    pub(crate) output: String,

    pub(crate) src_offsets: Vec<u32>,
    pub(crate) source: &'a str,
}

impl<'a> SourceMapTracker<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        SourceMapTracker {
            output: String::new(),
            src_offsets: Vec::new(),
            source,
        }
    }

    pub(crate) fn push_generated(&mut self, text: &str) {
        let char_count = text.chars().count();
        self.output.push_str(text);
        self.src_offsets
            .resize(self.src_offsets.len() + char_count, UNMAPPED);
    }

    pub(crate) fn push_source(&mut self, start: usize, end: usize) {
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

    pub(crate) fn push_source_mapped(&mut self, text: &str, src_pos: usize) {
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

    pub(crate) fn finish(self) -> (String, Vec<u32>) {
        (self.output, self.src_offsets)
    }
}

pub(crate) trait EmitSink {
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

pub(crate) fn emit_js_with_sourcemap(
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

pub(crate) fn get_js_output_path(
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
