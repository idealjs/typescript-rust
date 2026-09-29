#![allow(unused_imports)]

use std::sync::Arc;

use super::m4q::r33k12_defs::{TextPos, TextRange, compute_ecma_line_starts, get_new_line_kind, new_text_range, set_parent_in_children};
use tsox_core::core::compiler_options_kinds::ScriptTarget;
use tsox_core::core::tristate::Tristate;
use tsox_core::stringutil::is_white_space_like;
use tsox_frontend::ast::mig::m3b_2;
use tsox_frontend::ast::node::{LineMap, NodeList, Node};
use tsox_frontend::ast::node_source_file::{LanguageVariant, ScriptKind, SourceFile};
use tsox_frontend::ast::Symbol;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use crate::mig::m4s::EmitTextWriter;
use crate::printer::mig::m4m::{new_change_tracker_writer, ChangeTrackerWriter, TriviaPositionKey};
use tsox_frontend::format::mig::m4o_2::{new_printer, EmitContext, PrintHandlers};
use tsox_frontend::format::mig::m4o_2;
use super::m4q::r33k12_defs::PrinterOptions;

pub struct SingleLineStringWriter {
    builder: String,
    last_written: String,
}

pub fn get_single_line_string_writer() -> (SingleLineStringWriter, Box<dyn FnOnce()>) {
    let mut w = SingleLineStringWriter {
        builder: String::new(),
        last_written: String::new(),
    };
    w.clear();
    (w, Box::new(|| {}))
}

impl SingleLineStringWriter {
    fn append(&mut self, s: &str) {
        self.last_written = s.to_string();
        self.builder.push_str(s);
    }
}

impl EmitTextWriter for SingleLineStringWriter {
    fn clear(&mut self) {
        self.last_written = String::new();
        self.builder.clear();
    }

    fn decrease_indent(&mut self) {}

    fn get_column(&self) -> usize {
        0
    }

    fn get_indent(&self) -> usize {
        0
    }

    fn get_line(&self) -> usize {
        0
    }

    fn string(&self) -> String {
        self.builder.clone()
    }

    fn get_text_pos(&self) -> usize {
        self.builder.len()
    }

    fn has_trailing_comment(&self) -> bool {
        false
    }

    fn has_trailing_whitespace(&self) -> bool {
        if self.builder.is_empty() {
            return false;
        }
        match self.last_written.chars().next_back() {
            Some(ch) if ch != char::REPLACEMENT_CHARACTER => is_white_space_like(ch),
            _ => false,
        }
    }

    fn increase_indent(&mut self) {}

    fn is_at_start_of_line(&self) -> bool {
        false
    }

    fn raw_write(&mut self, s: &str) {
        self.append(s);
    }

    fn write(&mut self, s: &str) {
        self.append(s);
    }

    fn write_comment(&mut self, text: &str) {
        self.append(text);
    }

    fn write_keyword(&mut self, text: &str) {
        self.append(text);
    }

    fn write_line(&mut self) {
        self.append(" ");
    }

    fn write_line_force(&mut self, _force: bool) {
        self.append(" ");
    }

    fn write_literal(&mut self, s: &str) {
        self.append(s);
    }

    fn write_operator(&mut self, text: &str) {
        self.append(text);
    }

    fn write_parameter(&mut self, text: &str) {
        self.append(text);
    }

    fn write_property(&mut self, text: &str) {
        self.append(text);
    }

    fn write_punctuation(&mut self, text: &str) {
        self.append(text);
    }

    fn write_space(&mut self, text: &str) {
        self.append(text);
    }

    fn write_string_literal(&mut self, text: &str) {
        self.append(text);
    }

    fn write_symbol(&mut self, text: &str, _symbol: Option<&Symbol>) {
        self.append(text);
    }

    fn write_trailing_semicolon(&mut self, text: &str) {
        self.append(text);
    }
}

pub fn print_and_position_node(
    factory: &crate::printer::NodeFactory<'_>,
    node: &Arc<Node>,
    source_file: Option<&Arc<SourceFile>>,
    new_line: &str,
    indent_size: usize,
    emit_context: &EmitContext,
) -> (String, Arc<Node>) {
    let mut writer = new_change_tracker_writer(new_line, indent_size as i32);
    let handlers = crate::mig::m4s_4::r39k17b_defs::change_tracker_print_handlers(&mut writer);
    let text_writer = m4o_2::new_text_writer(new_line.to_string(), indent_size);
    new_printer(
        PrinterOptions {
            new_line: get_new_line_kind(new_line),
            remove_comments: false,
            omit_trailing_semicolon: false,
            no_emit_helpers: false,
            target: ScriptTarget::default(),
            source_map: false,
            inline_source_map: false,
            inline_sources: false,
            omit_brace_source_map_positions: false,
            only_print_jsdoc_style: false,
            never_ascii_escape: true,
            preserve_source_newlines: true,
            terminate_unterminated_literals: true,
        },
        handlers,
        emit_context.clone(),
    )
    .write(node, source_file, text_writer.clone(), None);

    let mut text = text_writer.string();
    if let Some(stripped) = text.strip_suffix(new_line) {
        text = stripped.to_string();
    }
    let positioned = writer
        .assign_positions_to_node(Some(node), factory)
        .unwrap_or_else(|| node.clone());
    (text, positioned)
}

pub fn create_synthetic_source_file(
    factory: &tsox_frontend::ast::mig::m3b_2::NodeFactory,
    node: &Arc<Node>,
    text: &str,
    parse_options: m3b_2::SourceFileParseOptions,
) -> SourceFile {
    let synthetic_file_name = parse_options.file_name.clone();
    let mut eof = factory.new_token(SyntaxKind::EndOfFile);
    Arc::get_mut(&mut eof).unwrap().loc = new_text_range(text.len(), text.len());
    let mut statements = factory.new_node_list(vec![node.clone()]);
    statements.loc = new_text_range(node.pos(), node.end());
    let mut synthetic_file = factory.new_source_file(
        parse_options,
        text.to_string(),
        Arc::new(statements),
        eof,
    );
    Arc::get_mut(&mut synthetic_file).unwrap().loc = new_text_range(0, text.len());
    set_parent_in_children(&synthetic_file);
    SourceFile {
        node: synthetic_file,
        file_name: synthetic_file_name,
        text: text.to_string(),
        line_map: LineMap::from_text(text),
        language_variant: LanguageVariant::default(),
        script_kind: ScriptKind::default(),
        comment_directives: Vec::new(),
        jsdoc_cache: std::sync::RwLock::new(std::collections::HashMap::new()),
        has_lazy_jsdoc: false,
        is_declaration_file: false,
        imports: Vec::new(),
        module_augmentations: Vec::new(),
        ambient_module_names: Vec::new(),
        parse_error_spans: Vec::new(),
        external_module_indicator: None,
        common_js_module_indicator: None,
        uses_uri_style_node_core_modules: Tristate::Unknown,
        has_parse_diagnostics: false,
        referenced_files: Vec::new(),
        type_reference_directives: Vec::new(),
        lib_reference_directives: Vec::new(),
        supplemental_source_files: Vec::new(),
    }
}
