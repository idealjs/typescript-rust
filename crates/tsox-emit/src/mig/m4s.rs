#![allow(unused_imports)]

use super::m4q::r33k12_defs::compute_ecma_line_starts_seq;
use tsox_core::stringutil::is_white_space_like;
use tsox_core::core::mig::x12::utf16_len;
use tsox_frontend::ast::Symbol;

pub trait EmitTextWriter {
    fn write(&mut self, s: &str);
    fn write_trailing_semicolon(&mut self, text: &str);
    fn write_comment(&mut self, text: &str);
    fn write_keyword(&mut self, text: &str);
    fn write_operator(&mut self, text: &str);
    fn write_punctuation(&mut self, text: &str);
    fn write_space(&mut self, text: &str);
    fn write_string_literal(&mut self, text: &str);
    fn write_parameter(&mut self, text: &str);
    fn write_property(&mut self, text: &str);
    fn write_symbol(&mut self, text: &str, symbol: Option<&Symbol>);
    fn write_line(&mut self);
    fn write_line_force(&mut self, force: bool);
    fn increase_indent(&mut self);
    fn decrease_indent(&mut self);
    fn clear(&mut self);
    fn string(&self) -> String;
    fn raw_write(&mut self, s: &str);
    fn write_literal(&mut self, s: &str);
    fn get_text_pos(&self) -> usize;
    fn get_line(&self) -> usize;
    fn get_column(&self) -> usize;
    fn get_indent(&self) -> usize;
    fn is_at_start_of_line(&self) -> bool;
    fn has_trailing_comment(&self) -> bool;
    fn has_trailing_whitespace(&self) -> bool;
}

pub struct TextWriter {
    new_line: String,
    indent_size: usize,
    builder: String,
    last_written: String,
    indent: usize,
    line_start: bool,
    line_count: usize,
    line_pos: usize,
    has_trailing_comment_state: bool,
}

impl TextWriter {
    fn grow(&mut self, n: usize) { ::tsox_core::fntrace::enter("grow"); 
        self.builder.reserve(n);
    }

    fn update_line_count_and_pos_for(&mut self, s: &str) { ::tsox_core::fntrace::enter("update_line_count_and_pos_for"); 
        let mut count = 0;
        let mut last_line_start = 0;

        for line_start in compute_ecma_line_starts_seq(s) {
            count += 1;
            last_line_start = line_start;
        }

        if count > 1 {
            self.line_count += count - 1;
            let cur_len = self.builder.len();
            self.line_pos = cur_len - s.len() + last_line_start as usize;
            self.line_start = self.line_pos == cur_len;
            return;
        }
        self.line_start = false;
    }

    fn write_text(&mut self, s: &str) { ::tsox_core::fntrace::enter("write_text"); 
        if !s.is_empty() {
            if self.line_start {
                self.builder
                    .push_str(&get_indent_string(self.indent, self.indent_size));
                self.line_start = false;
            }
            self.builder.push_str(s);
            self.last_written = s.to_string();
            self.update_line_count_and_pos_for(s);
        }
    }

    fn write_line_raw(&mut self) { ::tsox_core::fntrace::enter("write_line_raw"); 
        self.builder.push_str(&self.new_line);
        self.last_written = self.new_line.clone();
        self.line_count += 1;
        self.line_pos = self.builder.len();
        self.line_start = true;
        self.has_trailing_comment_state = false;
    }
}

impl EmitTextWriter for TextWriter {
    fn clear(&mut self) { ::tsox_core::fntrace::enter("clear"); 
        *self = TextWriter {
            new_line: std::mem::take(&mut self.new_line),
            indent_size: self.indent_size,
            builder: String::new(),
            last_written: String::new(),
            indent: 0,
            line_start: true,
            line_count: 0,
            line_pos: 0,
            has_trailing_comment_state: false,
        };
    }

    fn decrease_indent(&mut self) { ::tsox_core::fntrace::enter("decrease_indent"); 
        self.indent -= 1;
    }

    fn get_column(&self) -> usize { ::tsox_core::fntrace::enter("get_column"); 
        if self.line_start {
            return self.indent * self.indent_size;
        }
        utf16_len(&self.builder[self.line_pos..])
    }

    fn get_indent(&self) -> usize { ::tsox_core::fntrace::enter("get_indent"); 
        self.indent
    }

    fn get_line(&self) -> usize { ::tsox_core::fntrace::enter("get_line"); 
        self.line_count
    }

    fn string(&self) -> String { ::tsox_core::fntrace::enter("string"); 
        self.builder.clone()
    }

    fn get_text_pos(&self) -> usize { ::tsox_core::fntrace::enter("get_text_pos"); 
        self.builder.len()
    }

    fn has_trailing_comment(&self) -> bool { ::tsox_core::fntrace::enter("has_trailing_comment"); 
        self.has_trailing_comment_state
    }

    fn has_trailing_whitespace(&self) -> bool { ::tsox_core::fntrace::enter("has_trailing_whitespace"); 
        if self.builder.is_empty() {
            return false;
        }
        match self.last_written.chars().next_back() {
            Some(ch) if ch != char::REPLACEMENT_CHARACTER => is_white_space_like(ch),
            _ => false,
        }
    }

    fn increase_indent(&mut self) { ::tsox_core::fntrace::enter("increase_indent"); 
        self.indent += 1;
    }

    fn is_at_start_of_line(&self) -> bool { ::tsox_core::fntrace::enter("is_at_start_of_line"); 
        self.line_start
    }

    fn raw_write(&mut self, s: &str) { ::tsox_core::fntrace::enter("raw_write"); 
        if !s.is_empty() {
            self.builder.push_str(s);
            self.last_written = s.to_string();
            self.has_trailing_comment_state = false;
        }
        self.update_line_count_and_pos_for(s);
    }

    fn write(&mut self, s: &str) { ::tsox_core::fntrace::enter("write"); 
        if !s.is_empty() {
            self.has_trailing_comment_state = false;
        }
        self.write_text(s);
    }

    fn write_trailing_semicolon(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_trailing_semicolon"); 
        self.write(text);
    }

    fn write_comment(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_comment"); 
        if !text.is_empty() {
            self.has_trailing_comment_state = true;
        }
        self.write_text(text);
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

    fn write_symbol(&mut self, text: &str, _symbol: Option<&Symbol>) { ::tsox_core::fntrace::enter("write_symbol"); 
        self.write(text);
    }

    fn write_line(&mut self) { ::tsox_core::fntrace::enter("write_line"); 
        if !self.line_start {
            self.write_line_raw();
        }
    }

    fn write_line_force(&mut self, force: bool) { ::tsox_core::fntrace::enter("write_line_force"); 
        if !self.line_start || force {
            self.write_line_raw();
        }
    }

    fn write_literal(&mut self, s: &str) { ::tsox_core::fntrace::enter("write_literal"); 
        self.write(s);
    }
}

pub const DEFAULT_INDENT_SIZE: usize = 4;

pub fn get_default_indent_size() -> usize { ::tsox_core::fntrace::enter("get_default_indent_size"); 
    DEFAULT_INDENT_SIZE
}

pub fn get_indent_string(indent: usize, indent_size: usize) -> String { ::tsox_core::fntrace::enter("get_indent_string"); 
    if indent == 0 {
        return String::new();
    }
    " ".repeat(indent * indent_size)
}

pub fn new_text_writer(new_line: &str, indent_size: usize) -> TextWriter { ::tsox_core::fntrace::enter("new_text_writer"); 
    let indent_size = if indent_size == 0 { 4 } else { indent_size };
    let mut w = TextWriter {
        new_line: new_line.to_string(),
        indent_size,
        builder: String::new(),
        last_written: String::new(),
        indent: 0,
        line_start: false,
        line_count: 0,
        line_pos: 0,
        has_trailing_comment_state: false,
    };
    w.clear();
    w
}
