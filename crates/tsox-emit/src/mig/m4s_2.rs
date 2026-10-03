#![allow(unused_imports)]

use tsox_frontend::ast::Symbol;

use crate::mig::m4s::EmitTextWriter;

pub struct TrailingSemicolonDeferringWriter {
    inner: Box<dyn EmitTextWriter>,
    has_pending_semicolon: bool,
}

pub fn get_trailing_semicolon_deferring_writer(
    writer: Box<dyn EmitTextWriter>,
) -> TrailingSemicolonDeferringWriter { ::tsox_core::fntrace::enter("get_trailing_semicolon_deferring_writer"); 
    TrailingSemicolonDeferringWriter {
        inner: writer,
        has_pending_semicolon: false,
    }
}

impl TrailingSemicolonDeferringWriter {
    fn commit_semicolon(&mut self) { ::tsox_core::fntrace::enter("commit_semicolon"); 
        if self.has_pending_semicolon {
            self.inner.write_trailing_semicolon(";");
            self.has_pending_semicolon = false;
        }
    }
}

impl EmitTextWriter for TrailingSemicolonDeferringWriter {
    fn write(&mut self, s: &str) { ::tsox_core::fntrace::enter("write"); 
        self.commit_semicolon();
        self.inner.write(s);
    }

    fn write_trailing_semicolon(&mut self, _text: &str) { ::tsox_core::fntrace::enter("write_trailing_semicolon"); 
        self.has_pending_semicolon = true;
    }

    fn write_comment(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_comment"); 
        self.commit_semicolon();
        self.inner.write_comment(text);
    }

    fn write_keyword(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_keyword"); 
        self.commit_semicolon();
        self.inner.write_keyword(text);
    }

    fn write_operator(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_operator"); 
        self.commit_semicolon();
        self.inner.write_operator(text);
    }

    fn write_punctuation(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_punctuation"); 
        self.commit_semicolon();
        self.inner.write_punctuation(text);
    }

    fn write_space(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_space"); 
        self.commit_semicolon();
        self.inner.write_space(text);
    }

    fn write_string_literal(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_string_literal"); 
        self.commit_semicolon();
        self.inner.write_string_literal(text);
    }

    fn write_parameter(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_parameter"); 
        self.commit_semicolon();
        self.inner.write_parameter(text);
    }

    fn write_property(&mut self, text: &str) { ::tsox_core::fntrace::enter("write_property"); 
        self.commit_semicolon();
        self.inner.write_property(text);
    }

    fn write_symbol(&mut self, text: &str, symbol: Option<&Symbol>) { ::tsox_core::fntrace::enter("write_symbol"); 
        self.commit_semicolon();
        self.inner.write_symbol(text, symbol);
    }

    fn write_line(&mut self) { ::tsox_core::fntrace::enter("write_line"); 
        self.commit_semicolon();
        self.inner.write_line();
    }

    fn write_line_force(&mut self, force: bool) { ::tsox_core::fntrace::enter("write_line_force"); 
        self.commit_semicolon();
        self.inner.write_line_force(force);
    }

    fn increase_indent(&mut self) { ::tsox_core::fntrace::enter("increase_indent"); 
        self.commit_semicolon();
        self.inner.increase_indent();
    }

    fn decrease_indent(&mut self) { ::tsox_core::fntrace::enter("decrease_indent"); 
        self.commit_semicolon();
        self.inner.decrease_indent();
    }

    fn clear(&mut self) { ::tsox_core::fntrace::enter("clear"); 
        self.has_pending_semicolon = false;
        self.inner.clear();
    }

    fn string(&self) -> String { ::tsox_core::fntrace::enter("string"); 
        self.inner.string()
    }

    fn raw_write(&mut self, s: &str) { ::tsox_core::fntrace::enter("raw_write"); 
        self.commit_semicolon();
        self.inner.raw_write(s);
    }

    fn write_literal(&mut self, s: &str) { ::tsox_core::fntrace::enter("write_literal"); 
        self.commit_semicolon();
        self.inner.write_literal(s);
    }

    fn get_text_pos(&self) -> usize { ::tsox_core::fntrace::enter("get_text_pos"); 
        self.inner.get_text_pos()
    }

    fn get_line(&self) -> usize { ::tsox_core::fntrace::enter("get_line"); 
        self.inner.get_line()
    }

    fn get_column(&self) -> usize { ::tsox_core::fntrace::enter("get_column"); 
        self.inner.get_column()
    }

    fn get_indent(&self) -> usize { ::tsox_core::fntrace::enter("get_indent"); 
        self.inner.get_indent()
    }

    fn is_at_start_of_line(&self) -> bool { ::tsox_core::fntrace::enter("is_at_start_of_line"); 
        self.inner.is_at_start_of_line()
    }

    fn has_trailing_comment(&self) -> bool { ::tsox_core::fntrace::enter("has_trailing_comment"); 
        self.inner.has_trailing_comment()
    }

    fn has_trailing_whitespace(&self) -> bool { ::tsox_core::fntrace::enter("has_trailing_whitespace"); 
        self.inner.has_trailing_whitespace()
    }
}
