#![allow(dead_code, unused_imports, unused_variables)]

use std::cell::RefCell;
use std::sync::Arc;

use tsox_frontend::ast::node::{Node, SourceFile};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::astnav::get_line_and_character_of_position;
use tsox_frontend::scanner::CommentRange;

use crate::mig::m4q::Printer;
use crate::mig::m4q::r33k12_defs::{CommentSeparator, TokenEmitFlags};
use crate::mig::m4r::EmitTextWriter;
use crate::printer::mig::w7pr::is_jsdoc_like_text;
use crate::sourcemap::Generator;

const NEW_LINE: &str = "\n";

pub struct TextWriter39k04 {
    text: String,
    indent: usize,
    indent_string: String,
    line: i32,
    column: i32,
    at_start_of_line: bool,
}

impl Default for TextWriter39k04 {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        Self {
            text: String::new(),
            indent: 0,
            indent_string: String::new(),
            line: 0,
            column: 0,
            at_start_of_line: true,
        }
    }
}

impl TextWriter39k04 {
    fn raw_write_str(&mut self, s: &str) { ::tsox_core::fntrace::enter("raw_write_str"); 
        for ch in s.chars() {
            if ch == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
        }
        self.text.push_str(s);
        if !s.is_empty() {
            self.at_start_of_line = false;
        }
    }
}

impl EmitTextWriter for TextWriter39k04 {
    fn write(&mut self, s: &str) { ::tsox_core::fntrace::enter("write"); 
        if self.at_start_of_line && self.indent > 0 {
            let indent_string = self.indent_string.clone();
            self.raw_write_str(&indent_string);
        }
        self.raw_write_str(s);
    }

    fn write_trailing_semicolon(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_trailing_semicolon"); 
        self.write(text);
    }

    fn write_comment(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_comment"); 
        self.write(text);
    }

    fn write_keyword(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_keyword"); 
        self.write(text);
    }

    fn write_operator(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_operator"); 
        self.write(text);
    }

    fn write_punctuation(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_punctuation"); 
        self.write(text);
    }

    fn write_space(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_space"); 
        self.write(text);
    }

    fn write_string_literal(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_string_literal"); 
        self.write(text);
    }

    fn write_parameter(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_parameter"); 
        self.write(text);
    }

    fn write_property(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_property"); 
        self.write(text);
    }

    fn write_symbol(&mut self, text: &str, _symbol: &Arc<tsox_frontend::ast::Symbol>) { ::tsox_core::fntrace::enter("write_symbol"); 
        self.write(text);
    }

    fn write_line(&mut self) { ::tsox_core::fntrace::enter("write_line"); 
        if !self.at_start_of_line {
            self.raw_write_str(NEW_LINE);
        }
    }

    fn write_line_force(&mut self, force: bool) { ::tsox_core::fntrace::enter("write_line_force"); 
        if force || !self.at_start_of_line {
            self.raw_write_str(NEW_LINE);
        }
    }

    fn increase_indent(&mut self) { ::tsox_core::fntrace::enter("increase_indent"); 
        self.indent += 1;
        self.indent_string = "    ".repeat(self.indent);
    }

    fn decrease_indent(&mut self) { ::tsox_core::fntrace::enter("decrease_indent"); 
        self.indent = self.indent.saturating_sub(1);
        self.indent_string = "    ".repeat(self.indent);
    }

    fn clear(&mut self) { ::tsox_core::fntrace::enter("clear"); 
        self.text.clear();
        self.line = 0;
        self.column = 0;
        self.at_start_of_line = true;
    }

    fn string(&self) -> String { ::tsox_core::fntrace::enter("string"); 
        self.text.clone()
    }

    fn raw_write(&mut self, s: &str) { ::tsox_core::fntrace::enter("raw_write"); 
        self.raw_write_str(s);
    }

    fn write_literal(&mut self, s: &str) { ::tsox_core::fntrace::enter("write_literal"); 
        self.write(s);
    }

    fn get_text_pos(&self) -> i32 { ::tsox_core::fntrace::enter("get_text_pos"); 
        self.text.len() as i32
    }

    fn get_line(&self) -> i32 { ::tsox_core::fntrace::enter("get_line"); 
        self.line
    }

    fn get_column(&self) -> i32 { ::tsox_core::fntrace::enter("get_column"); 
        self.column
    }

    fn get_indent(&self) -> i32 { ::tsox_core::fntrace::enter("get_indent"); 
        self.indent as i32
    }

    fn is_at_start_of_line(&self) -> bool { ::tsox_core::fntrace::enter("is_at_start_of_line"); 
        self.at_start_of_line
    }

    fn has_trailing_comment(&self) -> bool { ::tsox_core::fntrace::enter("has_trailing_comment"); 
        false
    }

    fn has_trailing_whitespace(&self) -> bool { ::tsox_core::fntrace::enter("has_trailing_whitespace"); 
        self.at_start_of_line
            || self
                .text
                .chars()
                .last()
                .is_some_and(|ch| ch.is_whitespace())
    }
}

#[derive(Clone)]
pub struct LineCharCache39k04 {
    source: Arc<SourceFile>,
}

impl LineCharCache39k04 {
    pub fn new(source: &Arc<SourceFile>) -> Self { ::tsox_core::fntrace::enter("new"); 
        Self {
            source: Arc::clone(source),
        }
    }

    pub fn get_line_and_character(&self, pos: usize) -> (i32, i32) { ::tsox_core::fntrace::enter("get_line_and_character"); 
        let (line, character) = get_line_and_character_of_position(&self.source, pos);
        (line as i32, character as i32)
    }
}

pub struct PrinterEmitState39k04 {
    pub writer: Box<dyn EmitTextWriter>,
    pub source_maps_disabled: bool,
    pub source_map_source: Option<Arc<SourceFile>>,
    pub source_map_source_index: i32,
    pub source_map_source_is_json: bool,
    pub source_map_line_char_cache: Option<LineCharCache39k04>,
    pub source_map_generator: Option<Generator>,
    pub map_source_position:
        Option<Box<dyn FnMut(&Arc<SourceFile>, usize) -> Option<(Arc<SourceFile>, usize)>>>,
}

impl Default for PrinterEmitState39k04 {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        Self {
            writer: Box::new(TextWriter39k04::default()),
            source_maps_disabled: false,
            source_map_source: None,
            source_map_source_index: -1,
            source_map_source_is_json: false,
            source_map_line_char_cache: None,
            source_map_generator: None,
            map_source_position: None,
        }
    }
}

thread_local! {
    static PRINTER_EMIT_STATE_39K04: RefCell<PrinterEmitState39k04> =
        RefCell::new(PrinterEmitState39k04::default());
}

impl Printer {
    pub(crate) fn emit_state39k04<R>(&self, f: impl FnOnce(&PrinterEmitState39k04) -> R) -> R { ::tsox_core::fntrace::enter("emit_state39k04"); 
        PRINTER_EMIT_STATE_39K04.with(|state| f(&state.borrow()))
    }

    pub(crate) fn emit_state39k04_mut<R>(
        &self,
        f: impl FnOnce(&mut PrinterEmitState39k04) -> R,
    ) -> R { ::tsox_core::fntrace::enter("emit_state39k04_mut"); 
        PRINTER_EMIT_STATE_39K04.with(|state| f(&mut state.borrow_mut()))
    }

    pub(crate) fn set_writer39k04(&mut self, writer: Box<dyn EmitTextWriter>) { ::tsox_core::fntrace::enter("set_writer39k04"); 
        self.emit_state39k04_mut(|state| state.writer = writer);
    }

    pub(crate) fn set_source_map_generator39k04(&mut self, generator: Generator) { ::tsox_core::fntrace::enter("set_source_map_generator39k04"); 
        self.emit_state39k04_mut(|state| state.source_map_generator = Some(generator));
    }

    pub(crate) fn set_map_source_position39k04(
        &mut self,
        map_source_position: Box<dyn FnMut(&Arc<SourceFile>, usize) -> Option<(Arc<SourceFile>, usize)>>,
    ) { ::tsox_core::fntrace::enter("set_map_source_position39k04"); 
        self.emit_state39k04_mut(|state| state.map_source_position = Some(map_source_position));
    }

    pub(crate) fn write_line(&mut self) { ::tsox_core::fntrace::enter("write_line"); 
        self.emit_state39k04_mut(|state| state.writer.write_line());
    }

    pub(crate) fn write_space(&mut self) { ::tsox_core::fntrace::enter("write_space"); 
        self.emit_state39k04_mut(|state| state.writer.write_space(" "));
    }

    pub(crate) fn should_write_comment(&self, comment: &CommentRange) -> bool { ::tsox_core::fntrace::enter("should_write_comment"); 
        if !self.options.only_print_jsdoc_style {
            return true;
        }
        let Some(source_file) = self.current_source_file.as_ref() else {
            return false;
        };
        let text = source_file.text.as_str();
        is_jsdoc_like_text(text, comment) || text[comment.pos..comment.end].starts_with("/*!")
    }

    pub(crate) fn emit_comment(&mut self, comment: &CommentRange) { ::tsox_core::fntrace::enter("emit_comment"); 
        let Some(source_file) = self.current_source_file.clone() else {
            return;
        };
        let text = source_file.text[comment.pos..comment.end].to_string();
        self.emit_state39k04_mut(|state| state.writer.write_comment(&text));
    }

    pub(crate) fn emit_comments(&mut self, comments: &[CommentRange], separator: CommentSeparator) { ::tsox_core::fntrace::enter("emit_comments"); 
        for (index, comment) in comments.iter().enumerate() {
            if index > 0 && separator == CommentSeparator::After {
                self.write_space();
            }
            self.emit_comment(comment);
            if separator == CommentSeparator::Before || index + 1 < comments.len() {
                if comment.has_trailing_new_line {
                    self.write_line();
                } else {
                    self.write_space();
                }
            }
        }
    }

    pub(crate) fn set_source_map_source(&mut self, source: &Arc<SourceFile>) { ::tsox_core::fntrace::enter("set_source_map_source"); 
        if self.emit_state39k04(|state| state.source_maps_disabled) {
            return;
        }

        self.emit_state39k04_mut(|state| {
            state.source_map_source = Some(Arc::clone(source));
            state.source_map_line_char_cache = Some(LineCharCache39k04::new(source));
            state.source_map_source_is_json = source.file_name.ends_with(".json");
        });
        if self.emit_state39k04(|state| state.source_map_source_is_json) {
            return;
        }

        let file_name = source.file_name.clone();
        let source_index = self.emit_state39k04_mut(|state| {
            state
                .source_map_generator
                .as_mut()
                .map(|generator| generator.add_source(&file_name))
        });
        if let Some(source_index) = source_index {
            self.emit_state39k04_mut(|state| state.source_map_source_index = source_index);
        }
        if self.options.inline_sources {
            let text = source.text.clone();
            self.emit_state39k04_mut(|state| {
                if let Some(generator) = state.source_map_generator.as_mut() {
                    generator
                        .set_source_content(state.source_map_source_index, &text)
                        .unwrap();
                }
            });
        }
    }

    pub(crate) fn should_emit_source_maps(&self, node: &Node) -> bool { ::tsox_core::fntrace::enter("should_emit_source_maps"); 
        self.emit_state39k04(|state| {
            !state.source_maps_disabled
                && state.source_map_source.is_some()
                && node.kind != SyntaxKind::SourceFile
                && !tsox_frontend::ast::utilities::is_in_json_file(node)
        })
    }

    pub(crate) fn should_emit_token_source_maps(
        &self,
        token: SyntaxKind,
        _pos: usize,
        context_node: &Node,
        flags: TokenEmitFlags,
    ) -> bool { ::tsox_core::fntrace::enter("should_emit_token_source_maps"); 
        flags.0 & TokenEmitFlags::NO_SOURCE_MAPS.0 == 0
            && self.should_emit_source_maps(context_node)
            && !self.options.omit_brace_source_map_positions
            && (token == SyntaxKind::OpenBraceToken || token == SyntaxKind::CloseBraceToken)
    }
}
