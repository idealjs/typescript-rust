use crate::scanner::regexp::RegExpParser;
use crate::scanner::{DiagnosticKind, ScannerError};

impl<'a> RegExpParser<'a> {
    pub(super) fn inc_pos(&mut self, n: i32) { ::tsox_core::fntrace::enter("inc_pos"); 
        if n >= 0 {
            self.pos = self.pos.wrapping_add(n as usize);
        } else {
            self.pos = self.pos.saturating_sub((-n) as usize);
        }
    }

    pub(super) fn char(&self) -> char { ::tsox_core::fntrace::enter("char"); 
        if self.pos < self.body_end {
            self.text.as_bytes()[self.pos] as char
        } else {
            '\0'
        }
    }

    pub(super) fn char_at(&self, offset: usize) -> char { ::tsox_core::fntrace::enter("char_at"); 
        match self.pos.checked_add(offset) {
            Some(p) if p < self.body_end => self.text.as_bytes()[p] as char,
            _ => '\0',
        }
    }

    #[allow(dead_code)]
    pub fn text(&self) -> &str { ::tsox_core::fntrace::enter("text"); 
        self.text
    }

    pub(super) fn two_chars_at(&self, pos: usize) -> Option<[u8; 2]> {
        if pos + 1 < self.body_end {
            let bytes = self.text.as_bytes();
            Some([bytes[pos], bytes[pos + 1]])
        } else {
            None
        }
    }

    pub(super) fn is_class_content_exit(&self, ch: char) -> bool { ::tsox_core::fntrace::enter("is_class_content_exit"); 
        ch == ']' || self.pos >= self.body_end
    }

    pub(super) fn error(
        &mut self,
        msg: tsox_core::diagnostics::Message,
        pos: usize,
        length: usize,
    ) { ::tsox_core::fntrace::enter("error"); 
        self.errors.push(ScannerError {
            kind: DiagnosticKind::RegexMessage(msg),
            pos,
            length,
        });
    }

    pub(super) fn error_arg(
        &mut self,
        msg: tsox_core::diagnostics::Message,
        pos: usize,
        length: usize,
        arg: char,
    ) { ::tsox_core::fntrace::enter("error_arg"); 
        self.errors.push(ScannerError {
            kind: DiagnosticKind::RegexMessageWithArg(msg, arg),
            pos,
            length,
        });
    }

    pub(super) fn scan_expected_char(&mut self, ch: char) { ::tsox_core::fntrace::enter("scan_expected_char"); 
        if self.char() == ch {
            self.inc_pos(1);
        } else {
            self.error(tsox_core::diagnostics::X_0_EXPECTED, self.pos, 0);
        }
    }

    pub(super) fn check_regular_expression_flag_availability(
        &mut self,
        flag: u16,
        pos: usize,
        size: usize,
    ) { ::tsox_core::fntrace::enter("check_regular_expression_flag_availability"); 
        let available_from = match flag {
            crate::scanner::REG_EXP_FLAG_D => {
                Some(tsox_core::core::compiler_options::ScriptTarget::ES2022)
            }
            crate::scanner::REG_EXP_FLAG_S => {
                Some(tsox_core::core::compiler_options::ScriptTarget::ES2018)
            }
            crate::scanner::REG_EXP_FLAG_V => {
                Some(tsox_core::core::compiler_options::ScriptTarget::ES2024)
            }
            _ => None,
        };
        if let Some(target) = available_from {
            if self.script_target < target {
                self.error(
                    tsox_core::diagnostics::THIS_REGULAR_EXPRESSION_FLAG_IS_ONLY_AVAILABLE_WHEN_TARGETING_0_OR_LATER,
                    pos,
                    size,
                );
            }
        }
    }

    pub(super) fn scan_source_character(&mut self) -> String { ::tsox_core::fntrace::enter("scan_source_character"); 
        if self.pos >= self.body_end {
            return String::new();
        }
        let (c, size) = crate::scanner::regexp::decode_rune_at(self.text, self.pos);
        self.pos += size;
        c.to_string()
    }
}
