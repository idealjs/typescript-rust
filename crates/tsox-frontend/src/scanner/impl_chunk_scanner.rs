#![allow(unused_imports)]

use crate::scanner::impl_chunk::*;

impl Scanner {
    pub(crate) fn save_state(&self) -> ScannerState { ::tsox_core::fntrace::enter("save_state"); 
        ScannerState {
            pos: self.pos,
            end: self.end,
            token: self.token,
            token_pos: self.token_pos,
            token_end: self.token_end,
            full_start_pos: self.full_start_pos,
            preceding_line_break: self.preceding_line_break,
            has_preceding_line_break: self.has_preceding_line_break,
            binary_marker_pos: self.binary_marker_pos,
            token_flags: self.token_flags,
            skip_jsdoc_leading_asterisks: self.skip_jsdoc_leading_asterisks,
            errors_len: self.errors.len(),
            comment_directives_len: self.comment_directives.len(),
        }
    }

    pub(crate) fn restore_state(&mut self, state: ScannerState) { ::tsox_core::fntrace::enter("restore_state"); 
        self.pos = state.pos;
        self.end = state.end;
        self.token = state.token;
        self.token_pos = state.token_pos;
        self.token_end = state.token_end;
        self.full_start_pos = state.full_start_pos;
        self.preceding_line_break = state.preceding_line_break;
        self.has_preceding_line_break = state.has_preceding_line_break;
        self.binary_marker_pos = state.binary_marker_pos;
        self.token_flags = state.token_flags;
        self.skip_jsdoc_leading_asterisks = state.skip_jsdoc_leading_asterisks;
        self.errors.truncate(state.errors_len);
        self.comment_directives
            .truncate(state.comment_directives_len);
    }

    pub fn new(text: impl Into<String>) -> Self { ::tsox_core::fntrace::enter("new"); 
        let text: std::sync::Arc<str> = std::sync::Arc::from(text.into());
        let len = text.len();
        Self {
            text,
            pos: 0,
            end: len,
            token: SyntaxKind::Unknown,
            token_pos: 0,
            token_end: 0,
            full_start_pos: 0,
            preceding_line_break: false,
            has_preceding_line_break: false,
            binary_marker_pos: None,
            token_flags: TOKEN_FLAGS_NONE,
            skip_jsdoc_leading_asterisks: 0,
            error_callback: None,
            errors: Vec::new(),
            comment_directives: Vec::new(),
            script_target: tsox_core::core::compiler_options::ScriptTarget::ESNext,
            language_variant: crate::ast::LanguageVariant::Standard,
            identifier_value: None,
            skip_trivia: true,
        }
    }

    pub fn with_error_callback(mut self, cb: ErrorCallback) -> Self { ::tsox_core::fntrace::enter("with_error_callback"); 
        self.error_callback = Some(cb);
        self
    }

    pub fn set_script_target(&mut self, target: tsox_core::core::compiler_options::ScriptTarget) { ::tsox_core::fntrace::enter("set_script_target"); 
        self.script_target = target;
    }

    pub fn set_language_variant(&mut self, variant: crate::ast::LanguageVariant) { ::tsox_core::fntrace::enter("set_language_variant"); 
        self.language_variant = variant;
    }

    /// Go SetSkipTrivia
    pub fn set_skip_trivia(&mut self, skip: bool) { ::tsox_core::fntrace::enter("set_skip_trivia"); 
        self.skip_trivia = skip;
    }

    pub(crate) fn report_error(&mut self, kind: DiagnosticKind, pos: usize, length: usize) { ::tsox_core::fntrace::enter("report_error"); 
        if let Some(cb) = self.error_callback {
            cb(kind, pos, length);
        }
        self.errors.push(ScannerError { kind, pos, length });
    }

    pub fn take_errors(&mut self) -> Vec<ScannerError> { ::tsox_core::fntrace::enter("take_errors"); 
        std::mem::take(&mut self.errors)
    }

    pub fn comment_directives(&self) -> &[CommentDirective] { ::tsox_core::fntrace::enter("comment_directives"); 
        &self.comment_directives
    }

    pub(crate) fn process_comment_directive(&mut self, start: usize, end: usize, multiline: bool) { ::tsox_core::fntrace::enter("process_comment_directive"); 
        let text = self.text.as_bytes();
        let mut pos = start;
        if multiline {
            while pos < end && (text[pos] == b' ' || text[pos] == b'\t') {
                pos += 1;
            }

            while pos < end && (text[pos] == b'/' || text[pos] == b'*') {
                pos += 1;
            }
        } else {
            pos += 2;

            while pos < end && text[pos] == b'/' {
                pos += 1;
            }
        }

        while pos < end && (text[pos] == b' ' || text[pos] == b'\t') {
            pos += 1;
        }

        if !(pos < end && text[pos] == b'@') {
            return;
        }
        pos += 1;
        let rest = &self.text[pos..end];
        let kind = if rest.starts_with("ts-expect-error") {
            CommentDirectiveKind::ExpectError
        } else if rest.starts_with("ts-ignore") {
            CommentDirectiveKind::Ignore
        } else {
            return;
        };
        self.comment_directives.push(CommentDirective {
            pos: start,
            end,
            kind,
        });
    }

    pub fn token(&self) -> SyntaxKind { ::tsox_core::fntrace::enter("token"); 
        self.token
    }

    pub fn token_pos(&self) -> usize { ::tsox_core::fntrace::enter("token_pos"); 
        self.token_pos
    }

    pub fn full_start_pos(&self) -> usize { ::tsox_core::fntrace::enter("full_start_pos"); 
        self.full_start_pos
    }

    pub fn token_end(&self) -> usize { ::tsox_core::fntrace::enter("token_end"); 
        self.token_end
    }

    pub fn token_text(&self) -> &str { ::tsox_core::fntrace::enter("token_text"); 
        &self.text[self.token_pos..self.token_end]
    }

    pub fn token_value(&self) -> String { ::tsox_core::fntrace::enter("token_value"); 
        if let Some(cooked) = &self.identifier_value {
            return cooked.clone();
        }
        let text = self.token_text();
        if text.len() >= 2 {
            let first = text.as_bytes()[0];
            let last = text.as_bytes()[text.len() - 1];
            if (first == b'"' && last == b'"')
                || (first == b'\'' && last == b'\'')
                || (first == b'`' && last == b'`')
            {
                return unescape_string(&text[1..text.len() - 1]);
            }
        }
        text.to_string()
    }

    pub fn has_preceding_line_break(&self) -> bool { ::tsox_core::fntrace::enter("has_preceding_line_break"); 
        self.has_preceding_line_break
    }

    pub fn token_flags(&self) -> TokenFlags { ::tsox_core::fntrace::enter("token_flags"); 
        self.token_flags
    }

    pub fn has_preceding_jsdoc_comment(&self) -> bool { ::tsox_core::fntrace::enter("has_preceding_jsdoc_comment"); 
        token_flags_contains(self.token_flags, TOKEN_FLAGS_PRECEDING_JSDOC_COMMENT)
    }

    pub fn has_preceding_jsdoc_leading_asterisks(&self) -> bool { ::tsox_core::fntrace::enter("has_preceding_jsdoc_leading_asterisks"); 
        token_flags_contains(
            self.token_flags,
            TOKEN_FLAGS_PRECEDING_JSDOC_LEADING_ASTERISKS,
        )
    }

    pub fn has_preceding_jsdoc_with_deprecated_tag(&self) -> bool { ::tsox_core::fntrace::enter("has_preceding_jsdoc_with_deprecated_tag"); 
        token_flags_contains(
            self.token_flags,
            TOKEN_FLAGS_PRECEDING_JSDOC_WITH_DEPRECATED,
        )
    }

    pub fn has_preceding_jsdoc_with_see_or_link(&self) -> bool { ::tsox_core::fntrace::enter("has_preceding_jsdoc_with_see_or_link"); 
        token_flags_contains(
            self.token_flags,
            TOKEN_FLAGS_PRECEDING_JSDOC_WITH_SEE_OR_LINK,
        )
    }

    pub fn set_skip_jsdoc_leading_asterisks(&mut self, skip: bool) { ::tsox_core::fntrace::enter("set_skip_jsdoc_leading_asterisks"); 
        if skip {
            self.skip_jsdoc_leading_asterisks += 1;
        } else {
            self.skip_jsdoc_leading_asterisks -= 1;
        }
    }

    pub fn pos(&self) -> usize { ::tsox_core::fntrace::enter("pos"); 
        self.pos
    }

    pub fn end(&self) -> usize { ::tsox_core::fntrace::enter("end"); 
        self.end
    }

    pub fn skip_jsdoc_leading_asterisks_raw(&self) -> i32 { ::tsox_core::fntrace::enter("skip_jsdoc_leading_asterisks_raw"); 
        self.skip_jsdoc_leading_asterisks
    }

    pub fn set_skip_jsdoc_leading_asterisks_raw(&mut self, value: i32) { ::tsox_core::fntrace::enter("set_skip_jsdoc_leading_asterisks_raw"); 
        self.skip_jsdoc_leading_asterisks = value;
    }

    pub fn text(&self) -> &str { ::tsox_core::fntrace::enter("text"); 
        &self.text
    }

    pub fn set_range(&mut self, pos: usize, end: usize) { ::tsox_core::fntrace::enter("set_range"); 
        self.pos = pos;
        self.end = end;
        self.full_start_pos = pos;
        self.token_pos = pos;
        self.token_end = pos;
        self.token = SyntaxKind::Unknown;
        self.token_flags = TOKEN_FLAGS_NONE;
        self.preceding_line_break = false;
        self.has_preceding_line_break = false;
    }
}
