#![allow(unused_imports)]

use crate::parser::impl_chunk::*;

impl Parser {
    pub fn new(source_text: impl Into<String>) -> Self {
        let text = source_text.into();
        let mut scanner = Scanner::new(text);
        let token = scanner.scan();
        let mut parser = Self {
            scanner,
            token,
            diagnostics: Vec::new(),
            language_variant: LanguageVariant::Standard,
            javascript_file: false,
            last_template_literal_was_middle: false,
            yield_context: false,
            await_context: false,
            decorator_context: false,
            disallow_in_context: false,
            parsing_contexts: 0,
        };

        parser.drain_scanner_errors();
        parser
    }

    pub(crate) fn new_with_language_variant(
        source_text: impl Into<String>,
        language_variant: LanguageVariant,
    ) -> Self {
        let mut parser = Self::new(source_text);
        parser.language_variant = language_variant;
        parser.scanner.set_language_variant(language_variant);
        parser
    }

    pub(crate) fn set_javascript_file(&mut self, javascript_file: bool) {
        self.javascript_file = javascript_file;
    }

    pub(crate) fn allow_in<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let outer = self.disallow_in_context;
        self.disallow_in_context = false;
        let result = f(self);
        self.disallow_in_context = outer;
        result
    }

    pub fn parse_source_file(file_name: impl Into<String>) -> SourceFile {
        let file_name = file_name.into();
        let text = std::fs::read_to_string(&file_name).unwrap_or_default();
        Self::parse_source_file_text(&file_name, text)
    }

    pub fn parse_source_file_text(file_name: &str, text: String) -> SourceFile {
        Self::parse_source_file_text_with_diagnostics(file_name, text).0
    }

    pub fn parse_source_file_text_with_diagnostics(
        file_name: &str,
        text: String,
    ) -> (SourceFile, Vec<ParserDiagnostic>) {
        let line_map = LineMap::from_text(&text);
        let script_kind = script_kind_from_file_name(file_name);
        let language_variant = match script_kind {
            ScriptKind::Jsx | ScriptKind::Tsx => LanguageVariant::Jsx,
            _ => LanguageVariant::Standard,
        };
        let mut parser = Parser::new_with_language_variant(text.clone(), language_variant);
        parser.set_javascript_file(matches!(script_kind, ScriptKind::Js | ScriptKind::Jsx));
        let (statements, end_of_file) = if matches!(script_kind, ScriptKind::Json) {
            parser.parse_json_text()
        } else {
            let statements =
                parser.parse_list(ParsingContext::SourceElements, Parser::parse_statement);
            let end_of_file = parser.create_token_node();
            (statements, end_of_file)
        };
        let pos = 0usize;
        let end = end_of_file.end();

        parser.drain_scanner_errors();

        let has_statements = !statements.is_empty();
        if let Some(p) = parser
            .scanner
            .binary_marker_pos()
            .filter(|_| has_statements)
        {
            let len = parser
                .scanner
                .text()
                .get(p..)
                .and_then(|rest| rest.chars().next())
                .map_or(1, |c| c.len_utf8());
            parser.diagnostics.push(ParserDiagnostic {
                message: tsox_core::diagnostics::DECLARATION_OR_STATEMENT_EXPECTED,
                message_args: Vec::new(),
                range: TextRange::new(p, p + len),
            });
        }

        let mut context_flags = crate::ast::node_flags::NodeFlags::empty();
        if matches!(script_kind, ScriptKind::Js | ScriptKind::Jsx) {
            context_flags |= crate::ast::node_flags::NodeFlags::JavaScriptFile;
        }
        if matches!(script_kind, ScriptKind::Json) {
            context_flags |= crate::ast::node_flags::NodeFlags::JavaScriptFile;
            context_flags |= crate::ast::node_flags::NodeFlags::JsonFile;
        }
        let node = Arc::new(Node::with_loc_flags(
            SyntaxKind::SourceFile,
            NodeData::SourceFile(SourceFileData {
                statements: Arc::new(statements),
                end_of_file_token: end_of_file,
            }),
            TextRange::new(pos, end),
            context_flags,
        ));
        let is_declaration_file = tsox_core::tspath::is_declaration_file_name(file_name);
        let mut file = SourceFile {
            node,
            file_name: file_name.to_string(),
            text,
            line_map,
            language_variant,
            script_kind,
            comment_directives: parser.scanner.comment_directives().to_vec(),
            jsdoc_cache: std::sync::RwLock::new(std::collections::HashMap::new()),
            has_lazy_jsdoc: !matches!(script_kind, ScriptKind::Js | ScriptKind::Jsx),
            is_declaration_file,
            imports: Vec::new(),
            module_augmentations: Vec::new(),
            ambient_module_names: Vec::new(),
            parse_error_spans: parser.diagnostics.iter().map(|d| d.range).collect(),
            external_module_indicator: None,
            common_js_module_indicator: None,
            uses_uri_style_node_core_modules: tsox_core::core::tristate::Tristate::Unknown,
            has_parse_diagnostics: !parser.diagnostics.is_empty(),
        };

        references::set_external_module_indicator(&mut file);
        crate::parser::reparse_await::reparse_top_level_await(&mut file, &mut parser.diagnostics);
        references::collect_external_module_references(&mut file);

        Self::apply_jsdoc_reparser(&mut file);
        (file, parser.diagnostics)
    }

    pub(crate) fn apply_jsdoc_reparser(file: &mut SourceFile) {
        let statements = match &file.node.data {
            NodeData::SourceFile(d) => &d.statements.nodes,
            _ => return,
        };

        let mut new_statements: Vec<Arc<Node>> = Vec::with_capacity(statements.len());
        let mut has_reparsed = false;

        for stmt in statements {
            let js_docs = file.resolve_jsdoc(stmt);
            if !js_docs.is_empty() {
                let reparsed = reparse_tags(stmt, &js_docs);
                if !reparsed.is_empty() {
                    has_reparsed = true;
                    new_statements.extend(reparsed);
                }
            }
            new_statements.push(stmt.clone());
        }

        if !has_reparsed {
            return;
        }

        let (end_of_file_token, old_loc, old_flags) = match &file.node.data {
            NodeData::SourceFile(d) => {
                (d.end_of_file_token.clone(), file.node.loc, file.node.flags)
            }
            _ => return,
        };
        let new_statements_node_list = Arc::new(NodeList {
            loc: TextRange::new(
                old_loc.pos(),
                new_statements
                    .last()
                    .map(|s| s.end())
                    .unwrap_or(old_loc.pos()),
            ),
            nodes: new_statements,
        });
        let new_node = Arc::new(Node::with_loc_flags(
            SyntaxKind::SourceFile,
            NodeData::SourceFile(SourceFileData {
                statements: new_statements_node_list,
                end_of_file_token,
            }),
            old_loc,
            old_flags,
        ));
        file.node = new_node;
    }

    pub(crate) fn next_token(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan();
        self.drain_scanner_errors();
        self.token
    }

    pub(crate) fn drain_scanner_errors(&mut self) {
        for err in self.scanner.take_errors() {
            self.push_scanner_error(err);
        }
    }

    pub(crate) fn push_scanner_error(&mut self, err: crate::scanner::ScannerError) {
        let message = match err.kind {
            crate::scanner::DiagnosticKind::InvalidCharacter => tsox_core::diagnostics::INVALID_CHARACTER,
            crate::scanner::DiagnosticKind::FileAppearsToBeBinary => {
                tsox_core::diagnostics::FILE_APPEARS_TO_BE_BINARY
            }
            crate::scanner::DiagnosticKind::UnterminatedStringLiteral => {
                tsox_core::diagnostics::UNTERMINATED_STRING_LITERAL
            }
            crate::scanner::DiagnosticKind::HexadecimalDigitExpected => {
                tsox_core::diagnostics::HEXADECIMAL_DIGIT_EXPECTED
            }
            crate::scanner::DiagnosticKind::UnexpectedEndOfText => {
                tsox_core::diagnostics::UNEXPECTED_END_OF_TEXT
            }
            crate::scanner::DiagnosticKind::UnicodeEscapeOutOfRange => {
                tsox_core::diagnostics::AN_EXTENDED_UNICODE_ESCAPE_VALUE_MUST_BE_BETWEEN_0X0_AND_0X10FFFF_INCLUSIVE
            }
            crate::scanner::DiagnosticKind::UnterminatedUnicodeEscape => {
                tsox_core::diagnostics::UNTERMINATED_UNICODE_ESCAPE_SEQUENCE
            }
            crate::scanner::DiagnosticKind::UnterminatedTemplateLiteral => {
                tsox_core::diagnostics::UNTERMINATED_TEMPLATE_LITERAL
            }
            crate::scanner::DiagnosticKind::UnterminatedRegularExpression => {
                tsox_core::diagnostics::UNTERMINATED_REGULAR_EXPRESSION_LITERAL
            }
            crate::scanner::DiagnosticKind::UnknownRegularExpressionFlag => {
                tsox_core::diagnostics::UNKNOWN_REGULAR_EXPRESSION_FLAG
            }
            crate::scanner::DiagnosticKind::DuplicateRegularExpressionFlag => {
                tsox_core::diagnostics::DUPLICATE_REGULAR_EXPRESSION_FLAG
            }
            crate::scanner::DiagnosticKind::UnicodeUAndVFlagsMutuallyExclusive => {
                tsox_core::diagnostics::THE_UNICODE_U_FLAG_AND_THE_UNICODE_SETS_V_FLAG_CANNOT_BE_SET_SIMULTANEOUSLY
            }
            crate::scanner::DiagnosticKind::OctalLiteralNotAllowed => {
                tsox_core::diagnostics::OCTAL_LITERALS_ARE_NOT_ALLOWED_USE_THE_SYNTAX_0
            }
            crate::scanner::DiagnosticKind::DecimalWithLeadingZero => {
                tsox_core::diagnostics::DECIMALS_WITH_LEADING_ZEROS_ARE_NOT_ALLOWED
            }
            crate::scanner::DiagnosticKind::NumericSeparatorNotAllowed => {
                tsox_core::diagnostics::NUMERIC_SEPARATORS_ARE_NOT_ALLOWED_HERE
            }
            crate::scanner::DiagnosticKind::RegexMessage(msg) => msg,
            crate::scanner::DiagnosticKind::RegexMessageWithArg(msg, _arg) => msg,
        };
        let args: Vec<String> = match err.kind {
            crate::scanner::DiagnosticKind::OctalLiteralNotAllowed => {
                let token_text = &self.scanner.text()
                    [err.pos..(err.pos + err.length).min(self.scanner.text().len())];
                let octal_digits = token_text.strip_prefix('-').unwrap_or(token_text);
                let digits = octal_digits.strip_prefix('0').unwrap_or(octal_digits);
                // Go scanner：按八进制解析后重新格式化，去前导零
                let val = i64::from_str_radix(digits, 8).unwrap_or(0);
                let sign = if token_text.starts_with('-') { "-" } else { "" };
                vec![format!("{sign}0o{val:o}")]
            }
            crate::scanner::DiagnosticKind::RegexMessageWithArg(_, arg) => vec![arg.to_string()],
            _ => Vec::new(),
        };
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        self.parse_error_at_range(
            TextRange::new(err.pos, err.pos + err.length),
            message,
            &arg_refs,
        );
    }

    pub(crate) fn look_ahead_token(&self) -> SyntaxKind {
        let mut scanner = self.scanner.clone();
        scanner.scan()
    }

    pub(crate) fn look_ahead_2_tokens(&self) -> SyntaxKind {
        let mut scanner = self.scanner.clone();
        scanner.scan();
        scanner.scan()
    }

    pub(crate) fn look_ahead_3_tokens(&self) -> SyntaxKind {
        let mut scanner = self.scanner.clone();
        scanner.scan();
        scanner.scan();
        scanner.scan()
    }

    pub(crate) fn next_template_token(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan_template_continuation();
        self.drain_scanner_errors();
        self.token
    }

    pub(crate) fn token_pos(&self) -> usize {
        self.scanner.token_pos()
    }

    /// Go Parser.nodePos：节点 end 取当前 token 的 fullStart（即刚消费完的
    /// 语法 token 的真实结尾），不得吞下一 token 的前导 trivia
    pub(crate) fn node_pos(&self) -> usize {
        self.scanner.full_start_pos()
    }

    pub(crate) fn token_end(&self) -> usize {
        self.scanner.token_end()
    }

    pub(crate) fn has_preceding_line_break(&self) -> bool {
        self.scanner.has_preceding_line_break()
    }

    pub(crate) fn re_scan_greater_than(&mut self) {
        self.token = self.scanner.re_scan_greater_than();
        self.drain_scanner_errors();
    }

    pub(crate) fn re_scan_slash_token(&mut self) -> SyntaxKind {
        self.token = self.scanner.re_scan_slash_token();
        self.drain_scanner_errors();
        self.token
    }

    pub(crate) fn scan_jsx_text(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan_jsx_token();
        self.drain_scanner_errors();
        self.token
    }
}
