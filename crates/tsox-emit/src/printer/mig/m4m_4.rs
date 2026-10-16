#![allow(unused_imports)]
#![allow(dead_code)]
use super::m4m::ChangeTrackerWriter;
use crate::mig::m4s::EmitTextWriter;
use tsox_frontend::ast::Symbol;

pub type Utf16Offset = usize;

impl ChangeTrackerWriter {
    pub fn write(&mut self, text: &str) { ::tsox_core::fntrace::enter("write"); 
        self.writer.write(text);
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_trailing_semicolon(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_trailing_semicolon"); 
        self.writer.write_trailing_semicolon(text);
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_comment(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_comment"); 
        self.writer.write_comment(text);
    }

    pub fn write_keyword(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_keyword"); 
        self.writer.write_keyword(text);
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_operator(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_operator"); 
        self.writer.write_operator(text);
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_punctuation(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_punctuation"); 
        self.writer.write_punctuation(text);
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_space(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_space"); 
        self.writer.write_space(text);
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_string_literal(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_string_literal"); 
        self.writer.write_string_literal(text);
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_parameter(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_parameter"); 
        self.writer.write_parameter(text);
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_property(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_property"); 
        self.writer.write_property(text);
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_symbol(&mut self, text: &str, symbol: &Symbol) { ::tsox_core::fntrace::enter("write_symbol"); 
        self.writer.write_symbol(text, Some(symbol));
        self.set_last_non_trivia_position(text, false);
    }

    pub fn write_line(&mut self) { ::tsox_core::fntrace::enter("write_line"); 
        self.writer.write_line();
    }

    pub fn write_line_force(&mut self, force: bool) { ::tsox_core::fntrace::enter("write_line_force"); 
        self.writer.write_line_force(force);
    }

    pub fn increase_indent(&mut self) { ::tsox_core::fntrace::enter("increase_indent"); 
        self.writer.increase_indent();
    }

    pub fn decrease_indent(&mut self) { ::tsox_core::fntrace::enter("decrease_indent"); 
        self.writer.decrease_indent();
    }

    pub fn clear(&mut self) { ::tsox_core::fntrace::enter("clear"); 
        self.writer.clear();
        self.last_non_trivia_position = 0;
    }

    pub fn string(&self) -> String { ::tsox_core::fntrace::enter("string"); 
        self.writer.string()
    }

    pub fn raw_write(&mut self, s: &str) { ::tsox_core::fntrace::enter("raw_write"); 
        self.writer.raw_write(s);
        self.set_last_non_trivia_position(s, false);
    }

    pub fn write_literal(&mut self, s: &str) { ::tsox_core::fntrace::enter("write_literal"); 
        self.writer.write_literal(s);
        self.set_last_non_trivia_position(s, true);
    }

    pub fn get_text_pos(&self) -> usize { ::tsox_core::fntrace::enter("get_text_pos"); 
        self.writer.get_text_pos()
    }

    pub fn get_line(&self) -> usize { ::tsox_core::fntrace::enter("get_line"); 
        self.writer.get_line()
    }

    pub fn get_column(&self) -> Utf16Offset { ::tsox_core::fntrace::enter("get_column"); 
        self.writer.get_column()
    }

    pub fn get_indent(&self) -> usize { ::tsox_core::fntrace::enter("get_indent"); 
        self.writer.get_indent()
    }

    pub fn is_at_start_of_line(&self) -> bool { ::tsox_core::fntrace::enter("is_at_start_of_line"); 
        self.writer.is_at_start_of_line()
    }

    pub fn has_trailing_comment(&self) -> bool { ::tsox_core::fntrace::enter("has_trailing_comment"); 
        self.writer.has_trailing_comment()
    }

    pub fn has_trailing_whitespace(&self) -> bool { ::tsox_core::fntrace::enter("has_trailing_whitespace"); 
        self.writer.has_trailing_whitespace()
    }
}
