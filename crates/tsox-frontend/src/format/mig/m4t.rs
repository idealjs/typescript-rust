use crate::ast::symbol::Symbol;
use tsox_core::core::mig::m3j::compute_ecma_line_starts_seq;
use std::sync::Arc;

const DEFAULT_INDENT_SIZE: i32 = 4;

pub fn get_default_indent_size() -> i32 { ::tsox_core::fntrace::enter("get_default_indent_size"); 
    DEFAULT_INDENT_SIZE
}

pub struct TextWriter {
    pub new_line: String,
    pub indent_size: i32,
    pub builder: String,
    pub last_written: String,
    pub indent: i32,
    pub line_start: bool,
    pub line_count: i32,
    pub line_pos: i32,
    pub has_trailing_comment_state: bool,
}

pub fn get_indent_string(indent: i32, indent_size: i32) -> String { ::tsox_core::fntrace::enter("get_indent_string"); 
    if indent == 0 {
        return String::new();
    }
    " ".repeat((indent * indent_size) as usize)
}

impl TextWriter {
    pub fn update_line_count_and_pos_for(&mut self, s: &str) { ::tsox_core::fntrace::enter("update_line_count_and_pos_for"); 
        let mut count = 0;
        let mut last_line_start = 0;

        for line_start in compute_ecma_line_starts_seq(s) {
            count += 1;
            last_line_start = line_start;
        }

        if count > 1 {
            self.line_count += count - 1;
            let cur_len = self.builder.len() as i32;
            self.line_pos = cur_len - (s.len() as i32) + last_line_start;
            self.line_start = (self.line_pos - cur_len) == 0;
            return;
        }
        self.line_start = false;
    }

    pub fn write_text(&mut self, s: &str) { ::tsox_core::fntrace::enter("write_text"); 
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

    pub fn write_line_raw(&mut self) { ::tsox_core::fntrace::enter("write_line_raw"); 
        self.builder.push_str(&self.new_line.clone());
        self.last_written = self.new_line.clone();
        self.line_count += 1;
        self.line_pos = self.builder.len() as i32;
        self.line_start = true;
        self.has_trailing_comment_state = false;
    }

    pub fn write_parameter(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_parameter"); 
        self.write_text(text);
    }

    pub fn write_property(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_property"); 
        self.write_text(text);
    }

    pub fn write_punctuation(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_punctuation"); 
        self.write_text(text);
    }

    pub fn write_space(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_space"); 
        self.write_text(text);
    }

    pub fn write_string_literal(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_string_literal"); 
        self.write_text(text);
    }

    pub fn write_symbol(&mut self, text: &str, _symbol: Option<&Arc<Symbol>>) { ::tsox_core::fntrace::enter("write_symbol"); 
        self.write_text(text);
    }

    pub fn write_trailing_semicolon(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_trailing_semicolon"); 
        self.write_text(text);
    }
}
