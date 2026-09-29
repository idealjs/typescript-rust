#![allow(unused_imports)]
#![allow(dead_code)]
use super::m4m::ChangeTrackerWriter;
use crate::mig::m4s::EmitTextWriter;
use tsox_frontend::ast::Symbol;

pub type Utf16Offset = usize;

impl ChangeTrackerWriter {
    pub fn write(&mut self, text: &str) {
        self.writer.write(text);
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_trailing_semicolon(&mut self, text: &str) {
        self.writer.write_trailing_semicolon(text);
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_comment(&mut self, text: &str) {
        self.writer.write_comment(text);
    }

    pub fn write_keyword(&mut self, text: &str) {
        self.writer.write_keyword(text);
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_operator(&mut self, text: &str) {
        self.writer.write_operator(text);
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_punctuation(&mut self, text: &str) {
        self.writer.write_punctuation(text);
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_space(&mut self, text: &str) {
        self.writer.write_space(text);
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_string_literal(&mut self, text: &str) {
        self.writer.write_string_literal(text);
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_parameter(&mut self, text: &str) {
        self.writer.write_parameter(text);
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_property(&mut self, text: &str) {
        self.writer.write_property(text);
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_symbol(&mut self, text: &str, symbol: &Symbol) {
        self.writer.write_symbol(text, Some(symbol));
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_line(&mut self) {
        self.writer.write_line();
    }

    pub fn write_line_force(&mut self, force: bool) {
        self.writer.write_line_force(force);
    }

    pub fn increase_indent(&mut self) {
        self.writer.increase_indent();
    }

    pub fn decrease_indent(&mut self) {
        self.writer.decrease_indent();
    }

    pub fn clear(&mut self) {
        self.writer.clear();
        self.last_non_trivia_position = 0;
    }

    pub fn string(&self) -> String {
        self.writer.string()
    }

    pub fn raw_write(&mut self, s: &str) {
        self.writer.raw_write(s);
        self.set_last_non_trivia_position(s, false);
    }

    pub fn write_literal(&mut self, s: &str) {
        self.writer.write_literal(s);
        self.set_last_non_trivia_position(s, true);
    }

    pub fn get_text_pos(&self) -> usize {
        self.writer.get_text_pos()
    }

    pub fn get_line(&self) -> usize {
        self.writer.get_line()
    }

    pub fn get_column(&self) -> Utf16Offset {
        self.writer.get_column()
    }

    pub fn get_indent(&self) -> usize {
        self.writer.get_indent()
    }

    pub fn is_at_start_of_line(&self) -> bool {
        self.writer.is_at_start_of_line()
    }

    pub fn has_trailing_comment(&self) -> bool {
        self.writer.has_trailing_comment()
    }

    pub fn has_trailing_whitespace(&self) -> bool {
        self.writer.has_trailing_whitespace()
    }
}
