mod jsdoc;
mod references;
mod reparser;

pub use jsdoc::parse_jsdoc_for_node;
pub use references::{collect_external_module_references, set_external_module_indicator};
pub use reparser::reparse_tags;

use crate::ast::*;
use crate::core::text::TextRange;
use crate::diagnostics::{self, Message};
use crate::scanner::{Scanner, token_to_string};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsingContext {
    SourceElements,
    BlockStatements,
    SwitchClauses,
    SwitchClauseStatements,
    TypeMembers,
    ClassMembers,
    EnumMembers,
    HeritageClauseElement,
    VariableDeclarations,
    ObjectBindingElements,
    ArrayBindingElements,
    ArgumentExpressions,
    ObjectLiteralMembers,
    JsxAttributes,
    JsxChildren,
    ArrayLiteralMembers,
    Parameters,
    JSDocParameters,
    RestProperties,
    TypeParameters,
    TypeArguments,
    TupleElementTypes,
    HeritageClauses,
    ImportOrExportSpecifiers,
    ImportAttributes,
    JSDocComment,
}

pub struct Parser {
    scanner: Scanner,
    token: SyntaxKind,
    diagnostics: Vec<ParserDiagnostic>,
    language_variant: LanguageVariant,

    last_template_literal_was_middle: bool,

    yield_context: bool,

    await_context: bool,

    parsing_contexts: u32,
}

#[derive(Debug, Clone)]
pub struct ParserDiagnostic {
    pub message: Message,
    pub message_args: Vec<String>,
    pub range: TextRange,
}

pub fn script_kind_from_file_name(file_name: &str) -> ScriptKind {
    let ext = file_name.rfind('.').map(|i| &file_name[i..]).unwrap_or("");
    match ext {
        ".ts" | ".mts" | ".cts" => ScriptKind::Ts,
        ".tsx" => ScriptKind::Tsx,
        ".js" | ".mjs" | ".cjs" => ScriptKind::Js,
        ".jsx" => ScriptKind::Jsx,
        ".json" => ScriptKind::Json,
        _ => ScriptKind::Unknown,
    }
}

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
            last_template_literal_was_middle: false,
            yield_context: false,
            await_context: false,
            parsing_contexts: 0,
        };

        parser.drain_scanner_errors();
        parser
    }

    fn new_with_language_variant(
        source_text: impl Into<String>,
        language_variant: LanguageVariant,
    ) -> Self {
        let mut parser = Self::new(source_text);
        parser.language_variant = language_variant;
        parser
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
        let statements = parser.parse_list(ParsingContext::SourceElements, Parser::parse_statement);
        let end_of_file = parser.create_token_node();
        let pos = 0usize;
        let end = end_of_file.end();

        parser.drain_scanner_errors();

        let has_statements = !statements.is_empty();
        if let Some(p) = parser.scanner.binary_marker_pos().filter(|_| has_statements) {
            let len = parser
                .scanner
                .text()
                .get(p..)
                .and_then(|rest| rest.chars().next())
                .map_or(1, |c| c.len_utf8());
            parser.diagnostics.push(ParserDiagnostic {
                message: diagnostics::DECLARATION_OR_STATEMENT_EXPECTED,
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
        let is_declaration_file = crate::tspath::is_declaration_file_name(file_name);
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
            parse_error_spans: parser
                .diagnostics
                .iter()
                .map(|d| d.range)
                .collect(),
            external_module_indicator: None,
            common_js_module_indicator: None,
            uses_uri_style_node_core_modules: crate::core::tristate::Tristate::Unknown,
            has_parse_diagnostics: !parser.diagnostics.is_empty(),
        };

        references::set_external_module_indicator(&mut file);
        references::collect_external_module_references(&mut file);

        Self::apply_jsdoc_reparser(&mut file);
        (file, parser.diagnostics)
    }

    fn apply_jsdoc_reparser(file: &mut SourceFile) {
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

        let (end_of_file_token, old_loc) = match &file.node.data {
            NodeData::SourceFile(d) => (d.end_of_file_token.clone(), file.node.loc),
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
        let new_node = Arc::new(Node::with_loc(
            SyntaxKind::SourceFile,
            NodeData::SourceFile(SourceFileData {
                statements: new_statements_node_list,
                end_of_file_token,
            }),
            old_loc,
        ));
        file.node = new_node;
    }

    fn next_token(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan();
        self.drain_scanner_errors();
        self.token
    }

    fn drain_scanner_errors(&mut self) {
        for err in self.scanner.take_errors() {
            self.push_scanner_error(err);
        }
    }

    fn push_scanner_error(&mut self, err: crate::scanner::ScannerError) {
        let message = match err.kind {
            crate::scanner::DiagnosticKind::InvalidCharacter => diagnostics::INVALID_CHARACTER,
            crate::scanner::DiagnosticKind::FileAppearsToBeBinary => {
                diagnostics::FILE_APPEARS_TO_BE_BINARY
            }
            crate::scanner::DiagnosticKind::UnterminatedStringLiteral => {
                diagnostics::UNTERMINATED_STRING_LITERAL
            }
            crate::scanner::DiagnosticKind::UnterminatedTemplateLiteral => {
                diagnostics::UNTERMINATED_TEMPLATE_LITERAL
            }
            crate::scanner::DiagnosticKind::UnterminatedRegularExpression => {
                diagnostics::UNTERMINATED_REGULAR_EXPRESSION_LITERAL
            }
            crate::scanner::DiagnosticKind::UnknownRegularExpressionFlag => {
                diagnostics::UNKNOWN_REGULAR_EXPRESSION_FLAG
            }
            crate::scanner::DiagnosticKind::DuplicateRegularExpressionFlag => {
                diagnostics::DUPLICATE_REGULAR_EXPRESSION_FLAG
            }
            crate::scanner::DiagnosticKind::UnicodeUAndVFlagsMutuallyExclusive => {
                diagnostics::THE_UNICODE_U_FLAG_AND_THE_UNICODE_SETS_V_FLAG_CANNOT_BE_SET_SIMULTANEOUSLY
            }
            crate::scanner::DiagnosticKind::OctalLiteralNotAllowed => {
                diagnostics::OCTAL_LITERALS_ARE_NOT_ALLOWED_USE_THE_SYNTAX_0
            }
            crate::scanner::DiagnosticKind::DecimalWithLeadingZero => {
                diagnostics::DECIMALS_WITH_LEADING_ZEROS_ARE_NOT_ALLOWED
            }
            crate::scanner::DiagnosticKind::NumericSeparatorNotAllowed => {
                diagnostics::NUMERIC_SEPARATORS_ARE_NOT_ALLOWED_HERE
            }
            crate::scanner::DiagnosticKind::RegexMessage(msg) => msg,
        };
        let args: Vec<String> = match err.kind {
            crate::scanner::DiagnosticKind::OctalLiteralNotAllowed => {

                let token_text =
                    &self.scanner.text()[err.pos..(err.pos + err.length).min(self.scanner.text().len())];
                let octal_digits = token_text.strip_prefix('-').unwrap_or(token_text);
                let digits = octal_digits.strip_prefix('0').unwrap_or(octal_digits);
                vec![format!("0o{digits}")]
            }
            _ => Vec::new(),
        };
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        self.parse_error_at_range(TextRange::new(err.pos, err.pos + err.length), message, &arg_refs);
    }

    fn look_ahead_token(&self) -> SyntaxKind {
        let mut scanner = self.scanner.clone();
        scanner.scan()
    }

    fn look_ahead_2_tokens(&self) -> SyntaxKind {
        let mut scanner = self.scanner.clone();
        scanner.scan();
        scanner.scan()
    }

    fn look_ahead_3_tokens(&self) -> SyntaxKind {
        let mut scanner = self.scanner.clone();
        scanner.scan();
        scanner.scan();
        scanner.scan()
    }

    fn next_template_token(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan_template_continuation();
        self.drain_scanner_errors();
        self.token
    }

    fn token_pos(&self) -> usize {
        self.scanner.token_pos()
    }

    fn token_end(&self) -> usize {
        self.scanner.token_end()
    }

    fn has_preceding_line_break(&self) -> bool {
        self.scanner.has_preceding_line_break()
    }

    fn re_scan_greater_than(&mut self) {
        self.token = self.scanner.re_scan_greater_than();
        self.drain_scanner_errors();
    }

    fn re_scan_slash_token(&mut self) -> SyntaxKind {
        self.token = self.scanner.re_scan_slash_token();
        self.drain_scanner_errors();
        self.token
    }

    fn scan_jsx_text(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan_jsx_token();
        self.drain_scanner_errors();
        self.token
    }

    fn scan_jsx_identifier(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan_jsx_identifier();
        self.drain_scanner_errors();
        self.token
    }

    #[allow(dead_code)]
    fn scan_jsx_attribute_value(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan_jsx_attribute_value();
        self.drain_scanner_errors();
        self.token
    }

    fn token_range(&self) -> TextRange {
        TextRange::new(self.token_pos(), self.token_end())
    }

    fn parse_error_at_range(&mut self, range: TextRange, message: Message, args: &[&str]) {
        if let Some(last) = self.diagnostics.last() {
            if last.range.pos() == range.pos() {
                return;
            }
        }
        self.diagnostics.push(ParserDiagnostic {
            message,
            message_args: args.iter().map(|s| s.to_string()).collect(),
            range,
        });
    }

    fn parse_error_at(&mut self, pos: usize, end: usize, message: Message, args: &[&str]) {
        self.parse_error_at_range(TextRange::new(pos, end), message, args);
    }

    fn parse_error_at_current_token(&mut self, message: Message, args: &[&str]) {
        self.parse_error_at_range(self.token_range(), message, args);
    }

    fn expect(&mut self, expected: SyntaxKind) {
        if self.token == expected {
            self.next_token();
        } else {
            self.parse_error_at_current_token(
                diagnostics::X_0_EXPECTED,
                &[token_to_string(expected)],
            );
        }
    }

    fn expect_without_advancing(&mut self, expected: SyntaxKind) -> bool {
        if self.token == expected {
            true
        } else {
            self.parse_error_at_current_token(
                diagnostics::X_0_EXPECTED,
                &[token_to_string(expected)],
            );
            false
        }
    }

    fn parse_optional(&mut self, kind: SyntaxKind) -> bool {
        if self.token == kind {
            self.next_token();
            true
        } else {
            false
        }
    }

    fn parse_optional_token(&mut self, kind: SyntaxKind) -> Option<Arc<Node>> {
        if self.token == kind {
            let node = self.create_token_node();
            self.next_token();
            Some(node)
        } else {
            None
        }
    }

    fn create_token_node(&self) -> Arc<Node> {
        Arc::new(Node::with_loc(
            self.token,
            NodeData::Token,
            TextRange::new(self.token_pos(), self.token_end()),
        ))
    }

    fn create_template_token_node(&self) -> Arc<Node> {
        let raw = self.scanner.token_text();
        let cooked = match self.token {
            SyntaxKind::TemplateHead => {

                let s = raw.strip_prefix('`').unwrap_or(raw);
                s.strip_suffix("${").unwrap_or(s).to_string()
            }
            SyntaxKind::TemplateMiddle => {

                let s = raw.strip_prefix('}').unwrap_or(raw);
                s.strip_suffix("${").unwrap_or(s).to_string()
            }
            SyntaxKind::TemplateTail => {

                let s = raw.strip_prefix('}').unwrap_or(raw);
                s.strip_suffix('`').unwrap_or(s).to_string()
            }
            _ => raw.to_string(),
        };
        let data = match self.token {
            SyntaxKind::TemplateHead => NodeData::TemplateHead(TemplateHeadData {
                text: cooked.clone(),
                raw_text: raw.to_string(),
                template_flags: 0,
            }),
            SyntaxKind::TemplateMiddle => NodeData::TemplateMiddle(TemplateMiddleData {
                text: cooked.clone(),
                raw_text: raw.to_string(),
                template_flags: 0,
            }),
            SyntaxKind::TemplateTail => NodeData::TemplateTail(TemplateTailData {
                text: cooked.clone(),
                raw_text: raw.to_string(),
                template_flags: 0,
            }),
            _ => NodeData::Token,
        };
        Arc::new(Node::with_loc(
            self.token,
            data,
            TextRange::new(self.token_pos(), self.token_end()),
        ))
    }

    fn missing_node(&self, pos: usize) -> Arc<Node> {
        Arc::new(Node::with_loc(
            SyntaxKind::MissingDeclaration,
            NodeData::MissingDeclaration(MissingDeclarationData { modifiers: None }),
            TextRange::new(pos, pos),
        ))
    }

    fn can_parse_semicolon(&self) -> bool {
        self.token == SyntaxKind::SemicolonToken
            || self.token == SyntaxKind::CloseBraceToken
            || self.token == SyntaxKind::EndOfFile
            || self.has_preceding_line_break()
    }

    fn try_parse_semicolon(&mut self) -> bool {
        if !self.can_parse_semicolon() {
            return false;
        }
        if self.token == SyntaxKind::SemicolonToken {
            self.next_token();
        }
        true
    }

    fn parse_semicolon(&mut self) -> bool {
        self.try_parse_semicolon() || {
            self.expect(SyntaxKind::SemicolonToken);
            false
        }
    }

    fn parsing_context_errors(&mut self, context: ParsingContext) {
        match context {
            ParsingContext::SourceElements => {
                if self.token == SyntaxKind::DefaultKeyword {
                    self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &["export"]);
                } else {
                    self.parse_error_at_current_token(
                        diagnostics::DECLARATION_OR_STATEMENT_EXPECTED,
                        &[],
                    );
                }
            }
            ParsingContext::BlockStatements => {
                self.parse_error_at_current_token(
                    diagnostics::DECLARATION_OR_STATEMENT_EXPECTED,
                    &[],
                );
            }
            ParsingContext::SwitchClauses => {
                self.parse_error_at_current_token(diagnostics::X_CASE_OR_DEFAULT_EXPECTED, &[]);
            }
            ParsingContext::SwitchClauseStatements => {
                self.parse_error_at_current_token(diagnostics::STATEMENT_EXPECTED, &[]);
            }
            ParsingContext::RestProperties | ParsingContext::TypeMembers => {
                self.parse_error_at_current_token(diagnostics::PROPERTY_OR_SIGNATURE_EXPECTED, &[]);
            }
            ParsingContext::ClassMembers => {
                self.parse_error_at_current_token(
                    diagnostics::UNEXPECTED_TOKEN_A_CONSTRUCTOR_METHOD_ACCESSOR_OR_PROPERTY_WAS_EXPECTED,
                    &[],
                );
            }
            ParsingContext::EnumMembers => {
                self.parse_error_at_current_token(diagnostics::ENUM_MEMBER_EXPECTED, &[]);
            }
            ParsingContext::HeritageClauseElement => {
                self.parse_error_at_current_token(diagnostics::EXPRESSION_EXPECTED, &[]);
            }
            ParsingContext::VariableDeclarations => {
                if is_keyword_kind(self.token) {
                    self.parse_error_at_current_token(
                        diagnostics::X_0_IS_NOT_ALLOWED_AS_A_VARIABLE_DECLARATION_NAME,
                        &[token_to_string(self.token)],
                    );
                } else {
                    self.parse_error_at_current_token(
                        diagnostics::VARIABLE_DECLARATION_EXPECTED,
                        &[],
                    );
                }
            }
            ParsingContext::ObjectBindingElements => {
                self.parse_error_at_current_token(
                    diagnostics::PROPERTY_DESTRUCTURING_PATTERN_EXPECTED,
                    &[],
                );
            }
            ParsingContext::ArrayBindingElements => {
                self.parse_error_at_current_token(
                    diagnostics::ARRAY_ELEMENT_DESTRUCTURING_PATTERN_EXPECTED,
                    &[],
                );
            }
            ParsingContext::ArgumentExpressions => {
                self.parse_error_at_current_token(diagnostics::ARGUMENT_EXPRESSION_EXPECTED, &[]);
            }
            ParsingContext::ObjectLiteralMembers => {
                self.parse_error_at_current_token(diagnostics::PROPERTY_ASSIGNMENT_EXPECTED, &[]);
            }
            ParsingContext::ArrayLiteralMembers => {
                self.parse_error_at_current_token(diagnostics::EXPRESSION_OR_COMMA_EXPECTED, &[]);
            }
            ParsingContext::JSDocParameters => {
                self.parse_error_at_current_token(diagnostics::PARAMETER_DECLARATION_EXPECTED, &[]);
            }
            ParsingContext::Parameters => {
                if is_keyword_kind(self.token) {
                    self.parse_error_at_current_token(
                        diagnostics::X_0_IS_NOT_ALLOWED_AS_A_PARAMETER_NAME,
                        &[token_to_string(self.token)],
                    );
                } else {
                    self.parse_error_at_current_token(
                        diagnostics::PARAMETER_DECLARATION_EXPECTED,
                        &[],
                    );
                }
            }
            ParsingContext::TypeParameters => {
                self.parse_error_at_current_token(
                    diagnostics::TYPE_PARAMETER_DECLARATION_EXPECTED,
                    &[],
                );
            }
            ParsingContext::TypeArguments => {
                self.parse_error_at_current_token(diagnostics::TYPE_ARGUMENT_EXPECTED, &[]);
            }
            ParsingContext::TupleElementTypes => {
                self.parse_error_at_current_token(diagnostics::TYPE_EXPECTED, &[]);
            }
            ParsingContext::HeritageClauses => {
                self.parse_error_at_current_token(diagnostics::UNEXPECTED_TOKEN_EXPECTED, &[]);
            }
            ParsingContext::ImportOrExportSpecifiers => {
                if self.token == SyntaxKind::FromKeyword {
                    self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &["}"]);
                } else {
                    self.parse_error_at_current_token(diagnostics::IDENTIFIER_EXPECTED, &[]);
                }
            }
            ParsingContext::JsxAttributes
            | ParsingContext::JsxChildren
            | ParsingContext::JSDocComment => {
                self.parse_error_at_current_token(diagnostics::IDENTIFIER_EXPECTED, &[]);
            }
            ParsingContext::ImportAttributes => {
                self.parse_error_at_current_token(
                    diagnostics::IDENTIFIER_OR_STRING_LITERAL_EXPECTED,
                    &[],
                );
            }
        }
    }

    fn abort_parsing_list_or_move_to_next_token(&mut self, context: ParsingContext) -> bool {
        self.parsing_context_errors(context);
        if self.is_in_some_parsing_context() {
            true
        } else {
            self.next_token();
            false
        }
    }

    fn parse_list(
        &mut self,
        context: ParsingContext,
        parse_element: fn(&mut Self) -> Arc<Node>,
    ) -> NodeList {
        let pos = self.token_pos();

        let save_contexts = self.parsing_contexts;
        self.parsing_contexts |= 1 << (context as u32);
        let mut nodes = Vec::new();
        while !self.is_list_terminator(context) {
            if self.is_list_element(context, false) {
                let element = parse_element(self);
                nodes.push(element);
            } else if self.abort_parsing_list_or_move_to_next_token(context) {
                break;
            }
        }
        self.parsing_contexts = save_contexts;
        let end = self.token_pos();
        NodeList {
            loc: TextRange::new(pos, end),
            nodes,
        }
    }

    fn parse_delimited_list(
        &mut self,
        context: ParsingContext,
        parse_element: fn(&mut Self) -> Arc<Node>,
    ) -> NodeList {
        let pos = self.token_pos();
        let save_contexts = self.parsing_contexts;
        self.parsing_contexts |= 1 << (context as u32);
        let mut nodes = Vec::new();
        loop {
            if self.is_list_element(context, false) {
                let element_start = self.token_pos();
                let element = parse_element(self);
                nodes.push(element);
                if self.parse_optional(SyntaxKind::CommaToken) {
                    continue;
                }
                if self.is_list_terminator(context) {
                    break;
                }

                self.expect(SyntaxKind::CommaToken);

                if element_start == self.token_pos() {
                    self.next_token();
                }
                continue;
            }
            if self.is_list_terminator(context) {
                break;
            }

            if self.abort_parsing_list_or_move_to_next_token(context) {
                break;
            }
        }
        self.parsing_contexts = save_contexts;
        let end = self.token_pos();
        NodeList {
            loc: TextRange::new(pos, end),
            nodes,
        }
    }

    #[allow(non_snake_case)]
    fn parse_bracketedList(
        &mut self,
        context: ParsingContext,
        parse_element: fn(&mut Self) -> Arc<Node>,
        opening: SyntaxKind,
        closing: SyntaxKind,
    ) -> NodeList {
        if self.parse_optional(opening) {
            let list = self.parse_delimited_list(context, parse_element);
            self.expect(closing);
            list
        } else {
            NodeList::default()
        }
    }

    fn is_list_terminator(&self, context: ParsingContext) -> bool {
        if self.token == SyntaxKind::EndOfFile {
            return true;
        }
        match context {
            ParsingContext::BlockStatements
            | ParsingContext::SwitchClauses
            | ParsingContext::TypeMembers
            | ParsingContext::ClassMembers
            | ParsingContext::EnumMembers
            | ParsingContext::ObjectLiteralMembers
            | ParsingContext::ObjectBindingElements
            | ParsingContext::ImportOrExportSpecifiers
            | ParsingContext::ImportAttributes => self.token == SyntaxKind::CloseBraceToken,
            ParsingContext::SwitchClauseStatements => {
                self.token == SyntaxKind::CloseBraceToken
                    || self.token == SyntaxKind::CaseKeyword
                    || self.token == SyntaxKind::DefaultKeyword
            }
            ParsingContext::HeritageClauseElement => {
                self.token == SyntaxKind::OpenBraceToken
                    || self.token == SyntaxKind::ExtendsKeyword
                    || self.token == SyntaxKind::ImplementsKeyword
            }
            ParsingContext::VariableDeclarations => {

                self.can_parse_semicolon()
                    || self.token == SyntaxKind::InKeyword
                    || self.token == SyntaxKind::OfKeyword
                    || self.token == SyntaxKind::EqualsGreaterThanToken
            }
            ParsingContext::TypeParameters => {
                self.token == SyntaxKind::GreaterThanToken
                    || self.token == SyntaxKind::OpenParenToken
                    || self.token == SyntaxKind::OpenBraceToken
                    || self.token == SyntaxKind::ExtendsKeyword
                    || self.token == SyntaxKind::ImplementsKeyword
            }

            ParsingContext::TypeArguments => self.token != SyntaxKind::CommaToken,
            ParsingContext::ArgumentExpressions => {
                self.token == SyntaxKind::CloseParenToken
                    || self.token == SyntaxKind::SemicolonToken
            }
            ParsingContext::ArrayLiteralMembers
            | ParsingContext::TupleElementTypes
            | ParsingContext::ArrayBindingElements => self.token == SyntaxKind::CloseBracketToken,
            ParsingContext::JSDocParameters
            | ParsingContext::Parameters
            | ParsingContext::RestProperties => {
                self.token == SyntaxKind::CloseParenToken
                    || self.token == SyntaxKind::CloseBracketToken
            }
            ParsingContext::HeritageClauses => {
                self.token == SyntaxKind::OpenBraceToken
                    || self.token == SyntaxKind::CloseBraceToken
            }
            ParsingContext::JsxAttributes => {
                self.token == SyntaxKind::GreaterThanToken || self.token == SyntaxKind::SlashToken
            }
            ParsingContext::JsxChildren => self.token == SyntaxKind::LessThanToken,
            _ => false,
        }
    }

    fn is_list_element(&self, context: ParsingContext, in_error_recovery: bool) -> bool {
        match context {
            ParsingContext::SourceElements
            | ParsingContext::BlockStatements
            | ParsingContext::SwitchClauseStatements => {
                !(self.token == SyntaxKind::SemicolonToken && in_error_recovery)
                    && self.is_start_of_statement()
            }
            ParsingContext::SwitchClauses => {
                self.token == SyntaxKind::CaseKeyword || self.token == SyntaxKind::DefaultKeyword
            }
            ParsingContext::TypeMembers => !self.is_list_terminator(context),
            ParsingContext::ClassMembers => {

                self.look_ahead_class_member_start()
                    || (self.token == SyntaxKind::SemicolonToken && !in_error_recovery)
            }
            ParsingContext::EnumMembers | ParsingContext::ObjectLiteralMembers => {
                !self.is_list_terminator(context)
            }
            ParsingContext::RestProperties => self.is_literal_property_name(),
            ParsingContext::ObjectBindingElements => {
                self.token == SyntaxKind::OpenBracketToken
                    || self.token == SyntaxKind::DotDotDotToken
                    || self.is_literal_property_name()
            }
            ParsingContext::VariableDeclarations => self.is_binding_identifier_or_pattern(),
            ParsingContext::ArrayBindingElements => {
                self.token == SyntaxKind::CommaToken
                    || self.token == SyntaxKind::DotDotDotToken
                    || self.is_binding_identifier_or_pattern()
            }
            ParsingContext::TypeParameters => {
                is_identifier_or_keyword(self.token)
                    || self.token == SyntaxKind::InKeyword
                    || self.token == SyntaxKind::ConstKeyword
            }
            ParsingContext::ArgumentExpressions => {
                self.token == SyntaxKind::DotDotDotToken || self.is_start_of_expression()
            }
            ParsingContext::ArrayLiteralMembers => {

                if self.token == SyntaxKind::CommaToken || self.token == SyntaxKind::DotToken {
                    return true;
                }
                self.token == SyntaxKind::DotDotDotToken || self.is_start_of_expression()
            }
            ParsingContext::Parameters => self.is_start_of_parameter(),
            ParsingContext::JSDocParameters => self.is_start_of_parameter(),
            ParsingContext::TypeArguments | ParsingContext::TupleElementTypes => {
                self.token == SyntaxKind::CommaToken || self.is_start_of_type()
            }
            ParsingContext::HeritageClauseElement => {

                self.is_start_of_left_hand_side_expression()
            }
            ParsingContext::HeritageClauses => {
                self.token == SyntaxKind::ExtendsKeyword
                    || self.token == SyntaxKind::ImplementsKeyword
            }
            ParsingContext::ImportOrExportSpecifiers => {

                if self.token == SyntaxKind::FromKeyword
                    && self.look_ahead_token() == SyntaxKind::StringLiteral
                {
                    return false;
                }
                if self.token == SyntaxKind::StringLiteral {
                    return true;
                }
                is_identifier_or_keyword(self.token)
            }
            ParsingContext::ImportAttributes => {
                is_identifier_or_keyword(self.token) || self.token == SyntaxKind::StringLiteral
            }
            ParsingContext::JsxAttributes => {
                is_identifier_or_keyword(self.token) || self.token == SyntaxKind::OpenBraceToken
            }
            ParsingContext::JsxChildren => true,
            ParsingContext::JSDocComment => true,
        }
    }

    fn is_in_some_parsing_context(&self) -> bool {
        const CONTEXTS: [ParsingContext; 26] = [
            ParsingContext::SourceElements,
            ParsingContext::BlockStatements,
            ParsingContext::SwitchClauses,
            ParsingContext::SwitchClauseStatements,
            ParsingContext::TypeMembers,
            ParsingContext::ClassMembers,
            ParsingContext::EnumMembers,
            ParsingContext::HeritageClauseElement,
            ParsingContext::VariableDeclarations,
            ParsingContext::ObjectBindingElements,
            ParsingContext::ArrayBindingElements,
            ParsingContext::ArgumentExpressions,
            ParsingContext::ObjectLiteralMembers,
            ParsingContext::JsxAttributes,
            ParsingContext::JsxChildren,
            ParsingContext::ArrayLiteralMembers,
            ParsingContext::Parameters,
            ParsingContext::JSDocParameters,
            ParsingContext::RestProperties,
            ParsingContext::TypeParameters,
            ParsingContext::TypeArguments,
            ParsingContext::TupleElementTypes,
            ParsingContext::HeritageClauses,
            ParsingContext::ImportOrExportSpecifiers,
            ParsingContext::ImportAttributes,
            ParsingContext::JSDocComment,
        ];
        for kind in CONTEXTS {
            if self.parsing_contexts & (1 << (kind as u32)) != 0
                && (self.is_list_element(kind, true) || self.is_list_terminator(kind))
            {
                return true;
            }
        }
        false
    }

    fn is_start_of_statement(&self) -> bool {
        match self.token {
            SyntaxKind::AtToken
            | SyntaxKind::SemicolonToken
            | SyntaxKind::OpenBraceToken
            | SyntaxKind::VarKeyword
            | SyntaxKind::LetKeyword
            | SyntaxKind::UsingKeyword
            | SyntaxKind::FunctionKeyword
            | SyntaxKind::ClassKeyword
            | SyntaxKind::EnumKeyword
            | SyntaxKind::IfKeyword
            | SyntaxKind::DoKeyword
            | SyntaxKind::WhileKeyword
            | SyntaxKind::ForKeyword
            | SyntaxKind::ContinueKeyword
            | SyntaxKind::BreakKeyword
            | SyntaxKind::ReturnKeyword
            | SyntaxKind::WithKeyword
            | SyntaxKind::SwitchKeyword
            | SyntaxKind::ThrowKeyword
            | SyntaxKind::TryKeyword
            | SyntaxKind::DebuggerKeyword
            | SyntaxKind::CatchKeyword
            | SyntaxKind::FinallyKeyword
            | SyntaxKind::ConstKeyword
            | SyntaxKind::ExportKeyword
            | SyntaxKind::ImportKeyword
            | SyntaxKind::InterfaceKeyword
            | SyntaxKind::TypeKeyword
            | SyntaxKind::ModuleKeyword
            | SyntaxKind::NamespaceKeyword
            | SyntaxKind::DeclareKeyword
            | SyntaxKind::AsyncKeyword
            | SyntaxKind::GlobalKeyword
            | SyntaxKind::DeferKeyword
            | SyntaxKind::AccessorKeyword
            | SyntaxKind::PublicKeyword
            | SyntaxKind::PrivateKeyword
            | SyntaxKind::ProtectedKeyword
            | SyntaxKind::StaticKeyword
            | SyntaxKind::ReadonlyKeyword => true,
            _ => self.is_start_of_expression(),
        }
    }

    fn is_start_of_expression(&self) -> bool {
        if self.is_start_of_left_hand_side_expression() {
            return true;
        }
        matches!(
            self.token,
            SyntaxKind::PlusToken
                | SyntaxKind::MinusToken
                | SyntaxKind::TildeToken
                | SyntaxKind::ExclamationToken
                | SyntaxKind::DeleteKeyword
                | SyntaxKind::TypeOfKeyword
                | SyntaxKind::VoidKeyword
                | SyntaxKind::PlusPlusToken
                | SyntaxKind::MinusMinusToken
                | SyntaxKind::LessThanToken
                | SyntaxKind::AwaitKeyword
                | SyntaxKind::YieldKeyword
                | SyntaxKind::PrivateIdentifier
                | SyntaxKind::AtToken
        ) || self.is_identifier()
    }

    fn is_start_of_left_hand_side_expression(&self) -> bool {
        matches!(
            self.token,
            SyntaxKind::ThisKeyword
                | SyntaxKind::SuperKeyword
                | SyntaxKind::NullKeyword
                | SyntaxKind::TrueKeyword
                | SyntaxKind::FalseKeyword
                | SyntaxKind::NumericLiteral
                | SyntaxKind::BigIntLiteral
                | SyntaxKind::StringLiteral
                | SyntaxKind::NoSubstitutionTemplateLiteral
                | SyntaxKind::TemplateHead
                | SyntaxKind::OpenParenToken
                | SyntaxKind::OpenBracketToken
                | SyntaxKind::OpenBraceToken
                | SyntaxKind::FunctionKeyword
                | SyntaxKind::ClassKeyword
                | SyntaxKind::NewKeyword
                | SyntaxKind::SlashToken
                | SyntaxKind::SlashEqualsToken
                | SyntaxKind::Identifier
        ) || self.token == SyntaxKind::ImportKeyword
    }

    #[allow(dead_code)]
    fn is_literal(&self) -> bool {
        matches!(
            self.token,
            SyntaxKind::NumericLiteral
                | SyntaxKind::StringLiteral
                | SyntaxKind::NoSubstitutionTemplateLiteral
                | SyntaxKind::TemplateHead
                | SyntaxKind::BigIntLiteral
        )
    }

    fn is_let_declaration(&self) -> bool {
        let mut s = self.scanner.clone();
        s.scan();
        let t = s.token();
        Self::is_binding_identifier_token(t)
            || t == SyntaxKind::OpenBraceToken
            || t == SyntaxKind::OpenBracketToken
    }

    fn is_using_declaration(&self) -> bool {
        let mut scanner = self.scanner.clone();
        scanner.scan();
        let next = scanner.token();
        let no_line_break = !scanner.has_preceding_line_break();
        (Self::is_binding_identifier_token(next) || next == SyntaxKind::OpenBraceToken)
            && no_line_break
    }

    fn is_await_using_declaration(&self) -> bool {
        let mut scanner = self.scanner.clone();
        scanner.scan();
        if scanner.token() != SyntaxKind::UsingKeyword {
            return false;
        }
        let no_line_break = !scanner.has_preceding_line_break();
        scanner.scan();
        let next = scanner.token();
        let no_line_break_2 = !scanner.has_preceding_line_break();
        (Self::is_binding_identifier_token(next) || next == SyntaxKind::OpenBraceToken)
            && no_line_break
            && no_line_break_2
    }

    fn is_binding_identifier_token(token: SyntaxKind) -> bool {

        token == SyntaxKind::Identifier || (token as i16) > (SyntaxKind::WithKeyword as i16)
    }

    fn clone_state(&self) -> Parser {
        Parser {
            scanner: self.scanner.clone(),
            token: self.token,
            diagnostics: Vec::new(),
            language_variant: self.language_variant,
            last_template_literal_was_middle: self.last_template_literal_was_middle,
            yield_context: self.yield_context,
            await_context: self.await_context,
            parsing_contexts: self.parsing_contexts,
        }
    }

    fn is_start_of_declaration(&self) -> bool {
        let mut p = self.clone_state();
        p.scan_start_of_declaration()
    }

    fn scan_start_of_declaration(&mut self) -> bool {
        loop {
            match self.token {
                SyntaxKind::VarKeyword
                | SyntaxKind::LetKeyword
                | SyntaxKind::ConstKeyword
                | SyntaxKind::FunctionKeyword
                | SyntaxKind::ClassKeyword
                | SyntaxKind::EnumKeyword => return true,
                SyntaxKind::InterfaceKeyword | SyntaxKind::TypeKeyword => {
                    return self.next_token_is_identifier_on_same_line();
                }
                SyntaxKind::ModuleKeyword | SyntaxKind::NamespaceKeyword => {
                    return self.next_token_is_identifier_or_string_literal_on_same_line();
                }
                SyntaxKind::AbstractKeyword
                | SyntaxKind::AccessorKeyword
                | SyntaxKind::AsyncKeyword
                | SyntaxKind::DeclareKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::ProtectedKeyword
                | SyntaxKind::PublicKeyword
                | SyntaxKind::ReadonlyKeyword => {
                    let previous_token = self.token;
                    self.next_token();

                    if self.has_preceding_line_break() {
                        return false;
                    }
                    if previous_token == SyntaxKind::DeclareKeyword
                        && self.token == SyntaxKind::TypeKeyword
                    {
                        return true;
                    }
                    continue;
                }
                SyntaxKind::GlobalKeyword => {
                    self.next_token();
                    return self.token == SyntaxKind::OpenBraceToken
                        || self.token == SyntaxKind::Identifier
                        || self.token == SyntaxKind::ExportKeyword;
                }

                SyntaxKind::StaticKeyword => {
                    self.next_token();
                    continue;
                }
                SyntaxKind::ImportKeyword => {
                    self.next_token();
                    return self.token == SyntaxKind::StringLiteral
                        || self.token == SyntaxKind::AsteriskToken
                        || self.token == SyntaxKind::OpenBraceToken
                        || is_identifier_or_keyword(self.token);
                }
                SyntaxKind::ExportKeyword => {
                    self.next_token();
                    if self.token == SyntaxKind::EqualsToken
                        || self.token == SyntaxKind::AsteriskToken
                        || self.token == SyntaxKind::OpenBraceToken
                        || self.token == SyntaxKind::DefaultKeyword
                        || self.token == SyntaxKind::AsKeyword
                        || self.token == SyntaxKind::AtToken
                    {
                        return true;
                    }
                    if self.token == SyntaxKind::TypeKeyword {
                        self.next_token();
                        return self.token == SyntaxKind::AsteriskToken
                            || self.token == SyntaxKind::OpenBraceToken
                            || (self.is_identifier() && !self.has_preceding_line_break());
                    }
                    return self.is_start_of_declaration();
                }
                _ => return false,
            }
        }
    }

    fn next_token_is_identifier_on_same_line(&self) -> bool {
        let mut s = self.scanner.clone();
        s.scan();
        !s.has_preceding_line_break() && is_identifier_or_keyword(s.token())
    }

    fn next_token_is_identifier_or_string_literal_on_same_line(&self) -> bool {
        let mut s = self.scanner.clone();
        s.scan();
        !s.has_preceding_line_break()
            && (is_identifier_or_keyword(s.token()) || s.token() == SyntaxKind::StringLiteral)
    }

    pub fn parse_statement(&mut self) -> Arc<Node> {
        match self.token {
            SyntaxKind::SemicolonToken => self.parse_empty_statement(),
            SyntaxKind::OpenBraceToken => self.parse_block(),
            SyntaxKind::VarKeyword => self.parse_variable_statement(),
            SyntaxKind::LetKeyword if self.is_let_declaration() => self.parse_variable_statement(),
            SyntaxKind::UsingKeyword if self.is_using_declaration() => {
                self.parse_variable_statement()
            }
            SyntaxKind::AwaitKeyword if self.is_await_using_declaration() => {
                self.parse_variable_statement()
            }
            SyntaxKind::IfKeyword => self.parse_if_statement(),
            SyntaxKind::DoKeyword => self.parse_do_statement(),
            SyntaxKind::WhileKeyword => self.parse_while_statement(),
            SyntaxKind::ForKeyword => self.parse_for_statement(),
            SyntaxKind::ContinueKeyword => self.parse_continue_statement(),
            SyntaxKind::BreakKeyword => self.parse_break_statement(),
            SyntaxKind::ReturnKeyword => self.parse_return_statement(),
            SyntaxKind::SwitchKeyword => self.parse_switch_statement(),
            SyntaxKind::ThrowKeyword => self.parse_throw_statement(),
            SyntaxKind::TryKeyword => self.parse_try_statement(),
            SyntaxKind::FunctionKeyword => self.parse_function_declaration(),
            SyntaxKind::ClassKeyword => self.parse_class_declaration(),

            SyntaxKind::InterfaceKeyword if self.is_start_of_declaration() => {
                self.parse_interface_declaration()
            }
            SyntaxKind::TypeKeyword if self.is_start_of_declaration() => {
                self.parse_type_alias_declaration()
            }
            SyntaxKind::EnumKeyword => self.parse_enum_declaration(),
            SyntaxKind::NamespaceKeyword | SyntaxKind::ModuleKeyword
                if self.is_start_of_declaration() =>
            {
                self.parse_namespace_declaration()
            }
            SyntaxKind::DeclareKeyword if self.is_start_of_declaration() => {
                self.parse_declaration_with_modifiers(Vec::new())
            }
            SyntaxKind::AtToken => self.parse_declaration_with_modifiers(Vec::new()),
            SyntaxKind::ImportKeyword if self.is_start_of_declaration() => {
                self.parse_import_declaration()
            }
            SyntaxKind::ExportKeyword => self.parse_export_declaration(),
            SyntaxKind::DebuggerKeyword => self.parse_debugger_statement(),

            SyntaxKind::AsyncKeyword
            | SyntaxKind::ConstKeyword
            | SyntaxKind::AbstractKeyword
            | SyntaxKind::AccessorKeyword
            | SyntaxKind::StaticKeyword
            | SyntaxKind::ReadonlyKeyword
            | SyntaxKind::PublicKeyword
            | SyntaxKind::PrivateKeyword
            | SyntaxKind::ProtectedKeyword
            | SyntaxKind::GlobalKeyword
                if self.is_start_of_declaration() =>
            {
                self.parse_declaration_with_modifiers(Vec::new())
            }
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_empty_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        Arc::new(Node::with_loc(
            SyntaxKind::EmptyStatement,
            NodeData::EmptyStatement,
            TextRange::new(pos, self.token_pos()),
        ))
    }

    fn parse_block(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let multi_line = self.has_preceding_line_break();
        let statements = self.parse_list(ParsingContext::BlockStatements, Parser::parse_statement);
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::Block,
            NodeData::Block(BlockData {
                statements: Arc::new(statements),
                multi_line,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_variable_statement(&mut self) -> Arc<Node> {
        self.parse_variable_statement_with_modifiers(None)
    }

    fn parse_variable_statement_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        let declaration_list = self.parse_variable_declaration_list(false);
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::VariableStatement,
            NodeData::VariableStatement(VariableStatementData {
                modifiers,
                declaration_list,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_variable_declaration_list(&mut self, _in_for: bool) -> Arc<Node> {
        let pos = self.token_pos();
        let flags = match self.token {
            SyntaxKind::VarKeyword => NodeFlags::empty(),
            SyntaxKind::LetKeyword => NodeFlags::Let,
            SyntaxKind::ConstKeyword => NodeFlags::Const,
            SyntaxKind::UsingKeyword => NodeFlags::Using,
            SyntaxKind::AwaitKeyword => {

                NodeFlags::AwaitUsing
            }
            _ => NodeFlags::empty(),
        };
        if self.token == SyntaxKind::AwaitKeyword {
            self.next_token();
        }
        if self.token == SyntaxKind::UsingKeyword {
            self.next_token();
        } else {
            self.next_token();
        }
        let declarations = self.parse_delimited_list(
            ParsingContext::VariableDeclarations,
            if _in_for {
                Parser::parse_variable_declaration
            } else {
                Parser::parse_variable_declaration_allow_exclamation
            },
        );
        let end = self.token_pos();
        let mut node = Node::with_loc(
            SyntaxKind::VariableDeclarationList,
            NodeData::VariableDeclarationList(VariableDeclarationListData {
                declarations: Arc::new(declarations),
            }),
            TextRange::new(pos, end),
        );
        node.flags = flags;
        Arc::new(node)
    }

    fn parse_variable_declaration(&mut self) -> Arc<Node> {
        self.parse_variable_declaration_worker(false)
    }

    fn parse_variable_declaration_allow_exclamation(&mut self) -> Arc<Node> {
        self.parse_variable_declaration_worker(true)
    }

    fn parse_variable_declaration_worker(&mut self, allow_exclamation: bool) -> Arc<Node> {
        let pos = self.token_pos();
        let name = self.parse_identifier_or_pattern_with_diagnostic(Some(
            &diagnostics::PRIVATE_IDENTIFIERS_ARE_NOT_ALLOWED_IN_VARIABLE_DECLARATIONS,
        ));

        let exclamation_token = if allow_exclamation
            && name.kind == SyntaxKind::Identifier
            && self.token == SyntaxKind::ExclamationToken
            && !self.has_preceding_line_break()
        {
            let token = self.create_token_node();
            self.next_token();
            Some(token)
        } else {
            None
        };
        let type_node = self.parse_optional_type_annotation();
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();

            Some(self.parse_assignment_expression())
        } else {
            None
        };
        let mut end = name.end();
        if let Some(t) = &exclamation_token {
            end = end.max(t.end());
        }
        if let Some(t) = &type_node {
            end = end.max(t.end());
        }
        if let Some(n) = &initializer {
            end = end.max(n.end());
        }
        Arc::new(Node::with_loc(
            SyntaxKind::VariableDeclaration,
            NodeData::VariableDeclaration(VariableDeclarationData {
                name,
                exclamation_token,
                type_node,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_identifier_or_pattern(&mut self) -> Arc<Node> {
        self.parse_identifier_or_pattern_with_diagnostic(None)
    }

    fn parse_identifier_or_pattern_with_diagnostic(
        &mut self,
        private_msg: Option<&'static crate::diagnostics::Message>,
    ) -> Arc<Node> {
        if self.token == SyntaxKind::OpenBracketToken {
            self.parse_array_binding_pattern()
        } else if self.token == SyntaxKind::OpenBraceToken {
            self.parse_object_binding_pattern()
        } else {
            self.parse_identifier_with_private_diagnostic(private_msg)
        }
    }

    fn parse_array_binding_pattern(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBracketToken);
        let elements = self.parse_delimited_list(
            ParsingContext::ArrayBindingElements,
            Parser::parse_array_binding_element,
        );
        self.expect(SyntaxKind::CloseBracketToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ArrayBindingPattern,
            NodeData::BindingPattern(BindingPatternData {
                elements: Arc::new(elements),
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_array_binding_element(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);
        let name = if self.token != SyntaxKind::CommaToken {
            Some(self.parse_identifier_or_pattern())
        } else {
            None
        };
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();

            Some(self.parse_assignment_expression())
        } else {
            None
        };
        let end = initializer
            .as_ref()
            .map_or_else(|| name.as_ref().map_or(pos, |n| n.end()), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::BindingElement,
            NodeData::BindingElement(BindingElementData {
                dot_dot_dot_token,
                property_name: None,
                name,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_object_binding_pattern(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let elements = self.parse_delimited_list(
            ParsingContext::ObjectBindingElements,
            Parser::parse_object_binding_element,
        );
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ObjectBindingPattern,
            NodeData::BindingPattern(BindingPatternData {
                elements: Arc::new(elements),
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_object_binding_element(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);
        let is_identifier = self.is_identifier();
        let property_name = self.parse_property_name();
        let (property_name, name) = if is_identifier && self.token != SyntaxKind::ColonToken {
            (None, Some(property_name))
        } else {
            self.expect(SyntaxKind::ColonToken);
            (
                Some(property_name),
                Some(self.parse_identifier_or_pattern()),
            )
        };
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();

            Some(self.parse_assignment_expression())
        } else {
            None
        };
        let end = initializer
            .as_ref()
            .map_or_else(|| name.as_ref().map_or(pos, |n| n.end()), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::BindingElement,
            NodeData::BindingElement(BindingElementData {
                dot_dot_dot_token,
                property_name,
                name,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_optional_type_annotation(&mut self) -> Option<Arc<Node>> {
        if self.token == SyntaxKind::ColonToken {
            self.next_token();
            Some(self.parse_type())
        } else {
            None
        }
    }

    fn parse_optional_return_type(&mut self) -> Option<Arc<Node>> {
        if self.token == SyntaxKind::ColonToken {
            self.next_token();
            Some(self.parse_type_or_type_predicate())
        } else {
            None
        }
    }

    fn parse_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let mut type_node = self.parse_union_type_or_higher();
        if self.parse_optional(SyntaxKind::ExtendsKeyword) {
            let extends_type = self.parse_type();
            self.expect(SyntaxKind::QuestionToken);
            let true_type = self.parse_type();
            self.expect(SyntaxKind::ColonToken);
            let false_type = self.parse_type();
            let end = false_type.end();
            type_node = Arc::new(Node::with_loc(
                SyntaxKind::ConditionalType,
                NodeData::ConditionalTypeNode(ConditionalTypeNodeData {
                    check_type: type_node,
                    extends_type,
                    true_type,
                    false_type,
                }),
                TextRange::new(pos, end),
            ));
        }
        type_node
    }

    fn parse_type_or_type_predicate(&mut self) -> Arc<Node> {

        if self.token == SyntaxKind::Identifier
            || self.token == SyntaxKind::ObjectKeyword
            || self.token == SyntaxKind::ThisKeyword
        {
            let mut scanner = self.scanner.clone();
            scanner.scan();
            if scanner.token() == SyntaxKind::IsKeyword && !scanner.has_preceding_line_break() {
                let pos = self.token_pos();
                let parameter_name = self.parse_identifier();
                self.expect(SyntaxKind::IsKeyword);
                let type_node = self.parse_type();
                let end = type_node.end();
                return Arc::new(Node::with_loc(
                    SyntaxKind::TypePredicate,
                    NodeData::TypePredicateNode(TypePredicateNodeData {
                        asserts_modifier: None,
                        parameter_name,
                        type_node: Some(type_node),
                    }),
                    TextRange::new(pos, end),
                ));
            }
        }
        self.parse_type()
    }

    fn parse_union_type_or_higher(&mut self) -> Arc<Node> {
        self.parse_union_or_intersection_type(
            SyntaxKind::BarToken,
            SyntaxKind::UnionType,
            Parser::parse_intersection_type_or_higher,
        )
    }

    fn parse_intersection_type_or_higher(&mut self) -> Arc<Node> {
        self.parse_union_or_intersection_type(
            SyntaxKind::AmpersandToken,
            SyntaxKind::IntersectionType,
            Parser::parse_type_operator_or_higher,
        )
    }

    fn parse_union_or_intersection_type(
        &mut self,
        operator: SyntaxKind,
        node_kind: SyntaxKind,
        parse_constituent: fn(&mut Self) -> Arc<Node>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        let has_leading_operator = self.parse_optional(operator);
        let mut types = vec![parse_constituent(self)];
        while self.parse_optional(operator) {
            types.push(parse_constituent(self));
        }
        if types.len() == 1 && !has_leading_operator {
            return types.pop().unwrap();
        }
        let end = types.last().map_or(pos, |n| n.end());
        let list = Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: types,
        });
        let data = if node_kind == SyntaxKind::UnionType {
            NodeData::UnionTypeNode(UnionTypeNodeData { types: list })
        } else {
            NodeData::IntersectionTypeNode(IntersectionTypeNodeData { types: list })
        };
        Arc::new(Node::with_loc(node_kind, data, TextRange::new(pos, end)))
    }

    fn parse_type_operator_or_higher(&mut self) -> Arc<Node> {
        match self.token {
            SyntaxKind::KeyOfKeyword | SyntaxKind::UniqueKeyword | SyntaxKind::ReadonlyKeyword => {
                let pos = self.token_pos();
                let operator = self.token;
                self.next_token();
                let type_node = self.parse_type_operator_or_higher();
                let end = type_node.end();
                Arc::new(Node::with_loc(
                    SyntaxKind::TypeOperator,
                    NodeData::TypeOperatorNode(TypeOperatorNodeData {
                        operator,
                        type_node,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            _ => self.parse_postfix_type_or_higher(),
        }
    }

    fn parse_postfix_type_or_higher(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let mut type_node = self.parse_non_array_type();
        loop {
            if self.token == SyntaxKind::OpenBracketToken {
                self.next_token();
                if self.token == SyntaxKind::CloseBracketToken {
                    self.next_token();
                    let end = self.token_pos();
                    type_node = Arc::new(Node::with_loc(
                        SyntaxKind::ArrayType,
                        NodeData::ArrayTypeNode(ArrayTypeNodeData {
                            element_type: type_node,
                        }),
                        TextRange::new(pos, end),
                    ));
                    continue;
                }
                let index_type = self.parse_type();
                self.expect(SyntaxKind::CloseBracketToken);
                let end = self.token_pos();
                type_node = Arc::new(Node::with_loc(
                    SyntaxKind::IndexedAccessType,
                    NodeData::IndexedAccessTypeNode(IndexedAccessTypeNodeData {
                        object_type: type_node,
                        index_type,
                    }),
                    TextRange::new(pos, end),
                ));
                continue;
            }
            break;
        }
        type_node
    }

    fn parse_non_array_type(&mut self) -> Arc<Node> {
        match self.token {
            SyntaxKind::AnyKeyword
            | SyntaxKind::UnknownKeyword
            | SyntaxKind::StringKeyword
            | SyntaxKind::NumberKeyword
            | SyntaxKind::BigIntKeyword
            | SyntaxKind::SymbolKeyword
            | SyntaxKind::BooleanKeyword
            | SyntaxKind::UndefinedKeyword
            | SyntaxKind::NeverKeyword
            | SyntaxKind::ObjectKeyword => {

                if self.look_ahead_token() == SyntaxKind::DotToken {
                    return self.parse_type_reference();
                }
                let pos = self.token_pos();
                let end = self.token_end();
                let kind = self.token;
                self.next_token();
                Arc::new(Node::with_loc(
                    kind,
                    NodeData::KeywordTypeNode,
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::VoidKeyword => {
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::VoidKeyword,
                    NodeData::KeywordTypeNode,
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::NullKeyword
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::StringLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral => self.parse_literal_type_node(),
            SyntaxKind::MinusToken => {

                self.parse_literal_type_node_with_negative(true)
            }
            SyntaxKind::ThisKeyword => {
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                let this_keyword = Arc::new(Node::with_loc(
                    SyntaxKind::ThisType,
                    NodeData::ThisTypeNode,
                    TextRange::new(pos, end),
                ));

                if self.token == SyntaxKind::IsKeyword && !self.has_preceding_line_break() {
                    return self.parse_this_type_predicate(this_keyword);
                }
                this_keyword
            }
            SyntaxKind::TypeOfKeyword => {

                if self.look_ahead_token() == SyntaxKind::ImportKeyword {
                    self.parse_import_type()
                } else {
                    self.parse_type_query()
                }
            }
            SyntaxKind::ImportKeyword => self.parse_import_type(),
            SyntaxKind::AssertsKeyword => {

                if is_identifier_or_keyword(self.look_ahead_token()) {
                    self.parse_asserts_type_predicate()
                } else {
                    self.parse_type_reference()
                }
            }
            SyntaxKind::InferKeyword => self.parse_infer_type(),
            SyntaxKind::TemplateHead => self.parse_template_type(),
            SyntaxKind::OpenBraceToken => {

                if self.next_is_start_of_mapped_type() {
                    self.parse_mapped_type()
                } else {
                    self.parse_type_literal()
                }
            }
            SyntaxKind::OpenBracketToken => self.parse_tuple_type(),
            SyntaxKind::OpenParenToken => self.parse_parenthesized_or_function_type(),
            SyntaxKind::LessThanToken => self.parse_function_type(),
            SyntaxKind::NewKeyword | SyntaxKind::AbstractKeyword => self.parse_constructor_type(),
            _ => self.parse_type_reference(),
        }
    }

    fn parse_literal_type_node(&mut self) -> Arc<Node> {
        self.parse_literal_type_node_with_negative(false)
    }

    fn parse_literal_type_node_with_negative(&mut self, negative: bool) -> Arc<Node> {
        let pos = self.token_pos();
        if negative {

            self.next_token();
        }
        let literal = match self.token {
            SyntaxKind::NullKeyword | SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword => {
                self.parse_keyword_expression(self.token)
            }
            _ => self.parse_primary_expression(),
        };
        let end = literal.end();
        Arc::new(Node::with_loc(
            SyntaxKind::LiteralType,
            NodeData::LiteralTypeNode(LiteralTypeNodeData { literal }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_type_reference(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let type_name = self.parse_entity_name();
        let type_arguments = self.parse_optional_type_arguments();
        let end = type_arguments
            .as_ref()
            .map_or_else(|| type_name.end(), |args| args.end());
        Arc::new(Node::with_loc(
            SyntaxKind::TypeReference,
            NodeData::TypeReferenceNode(TypeReferenceNodeData {
                type_name,
                type_arguments,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_this_type_predicate(&mut self, lhs: Arc<Node>) -> Arc<Node> {

        let pos = lhs.pos();
        self.expect(SyntaxKind::IsKeyword);
        let type_node = self.parse_type();
        let end = type_node.end();
        Arc::new(Node::with_loc(
            SyntaxKind::TypePredicate,
            NodeData::TypePredicateNode(TypePredicateNodeData {
                asserts_modifier: None,
                parameter_name: lhs,
                type_node: Some(type_node),
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_asserts_type_predicate(&mut self) -> Arc<Node> {

        let pos = self.token_pos();

        let asserts_node = self.create_token_node();
        self.next_token();
        let parameter_name = self.parse_identifier();
        let mut type_node = None;
        if self.token == SyntaxKind::IsKeyword && !self.has_preceding_line_break() {
            self.next_token();
            type_node = Some(self.parse_type());
        }
        let end = type_node.as_ref().map_or(parameter_name.end(), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::TypePredicate,
            NodeData::TypePredicateNode(TypePredicateNodeData {
                asserts_modifier: Some(asserts_node),
                parameter_name,
                type_node,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_infer_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::InferKeyword);
        let type_parameter = self.parse_type_parameter();
        let end = type_parameter.end();
        Arc::new(Node::with_loc(
            SyntaxKind::InferType,
            NodeData::InferTypeNode(InferTypeNodeData { type_parameter }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_type_query(&mut self) -> Arc<Node> {

        let pos = self.token_pos();
        self.expect(SyntaxKind::TypeOfKeyword);
        let expr_name = self.parse_entity_name();

        let type_arguments = if !self.has_preceding_line_break() {
            self.parse_optional_type_arguments()
        } else {
            None
        };
        let end = type_arguments.as_ref().map_or(expr_name.end(), |a| a.end());
        Arc::new(Node::with_loc(
            SyntaxKind::TypeQuery,
            NodeData::TypeQueryNode(TypeQueryNodeData {
                expr_name,
                type_arguments,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_import_type(&mut self) -> Arc<Node> {

        let pos = self.token_pos();
        let is_type_of = self.parse_optional(SyntaxKind::TypeOfKeyword);
        self.expect(SyntaxKind::ImportKeyword);
        self.expect(SyntaxKind::OpenParenToken);

        let argument = self.parse_type();
        let attributes = if self.parse_optional(SyntaxKind::CommaToken) {
            self.expect(SyntaxKind::OpenBraceToken);
            let token = self.token;
            if matches!(token, SyntaxKind::WithKeyword | SyntaxKind::AssertKeyword) {
                self.next_token();
            } else {

                let with_str =
                    crate::scanner::token_to_string(SyntaxKind::WithKeyword).to_string();
                self.parse_error_at_current_token(
                    crate::diagnostics::messages_generated::X_0_EXPECTED,
                    &[&with_str],
                );
            }
            self.expect(SyntaxKind::ColonToken);
            let attrs = self.parse_import_attributes(token, true);
            self.parse_optional(SyntaxKind::CommaToken);
            self.expect(SyntaxKind::CloseBraceToken);
            Some(attrs)
        } else {
            None
        };
        self.expect(SyntaxKind::CloseParenToken);

        let qualifier = if self.parse_optional(SyntaxKind::DotToken) {
            Some(self.parse_entity_name())
        } else {
            None
        };
        let type_arguments = self.parse_optional_type_arguments();
        let end = type_arguments.as_ref().map_or_else(
            || {
                qualifier.as_ref().map_or_else(
                    || argument.end(),
                    |q| q.end(),
                )
            },
            |a: &Arc<NodeList>| a.end(),
        );
        Arc::new(Node::with_loc(
            SyntaxKind::ImportType,
            NodeData::ImportTypeNode(ImportTypeNodeData {
                is_type_of,
                argument,
                attributes,
                qualifier,
                type_arguments,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_template_type(&mut self) -> Arc<Node> {

        let pos = self.token_pos();
        let head = self.create_template_token_node();
        self.next_token();
        let template_spans = self.parse_template_type_spans();
        let end = template_spans.end();
        Arc::new(Node::with_loc(
            SyntaxKind::TemplateLiteralType,
            NodeData::TemplateLiteralTypeNode(TemplateLiteralTypeNodeData {
                head,
                template_spans,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_template_type_spans(&mut self) -> Arc<NodeList> {

        let pos = self.token_pos();
        let mut spans = Vec::new();
        loop {
            let span = self.parse_template_type_span();

            let is_middle = self.last_template_literal_was_middle;
            spans.push(span);
            if !is_middle {
                break;
            }
        }
        let end = spans.last().map_or(pos, |n| n.end());
        Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: spans,
        })
    }

    fn parse_template_type_span(&mut self) -> Arc<Node> {

        let pos = self.token_pos();
        let type_node = self.parse_type();

        let literal = if self.token == SyntaxKind::CloseBraceToken {
            self.next_template_token();
            self.last_template_literal_was_middle = self.token == SyntaxKind::TemplateMiddle;
            let lit = self.create_template_token_node();
            self.next_token();
            lit
        } else {

            self.last_template_literal_was_middle = false;
            self.missing_node(self.token_pos())
        };
        let end = literal.end();
        Arc::new(Node::with_loc(
            SyntaxKind::TemplateLiteralTypeSpan,
            NodeData::TemplateLiteralTypeSpan(TemplateLiteralTypeSpanData { type_node, literal }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_entity_name(&mut self) -> Arc<Node> {
        let pos = self.token_pos();

        match self.token {
            SyntaxKind::NullKeyword
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword => {
                let text = self.scanner.token_text().to_string();
                let text_str = text.as_str();
                self.parse_error_at_current_token(
                    crate::diagnostics::messages_generated::
                        IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE,
                    &[text_str],
                );
            }
            SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::StringLiteral => {
                self.parse_error_at_current_token(
                    crate::diagnostics::messages_generated::IDENTIFIER_EXPECTED,
                    &[],
                );
            }
            _ => {}
        }
        let mut left = self.parse_identifier();
        while self.parse_optional(SyntaxKind::DotToken) {
            let right = self.parse_identifier();
            let end = right.end();
            left = Arc::new(Node::with_loc(
                SyntaxKind::QualifiedName,
                NodeData::QualifiedName(QualifiedNameData { left, right }),
                TextRange::new(pos, end),
            ));
        }
        left
    }

    fn next_is_start_of_mapped_type(&self) -> bool {
        let mut scanner = self.scanner.clone();

        let t1 = scanner.scan();

        if t1 == SyntaxKind::PlusToken || t1 == SyntaxKind::MinusToken {
            return scanner.scan() == SyntaxKind::ReadonlyKeyword;
        }

        let t2 = if t1 == SyntaxKind::ReadonlyKeyword {
            scanner.scan()
        } else {
            t1
        };

        if t2 != SyntaxKind::OpenBracketToken {
            return false;
        }
        let t3 = scanner.scan();
        if !is_identifier_or_keyword(t3) {
            return false;
        }
        scanner.scan() == SyntaxKind::InKeyword
    }

    fn parse_mapped_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);

        let readonly_token = match self.token {
            SyntaxKind::ReadonlyKeyword | SyntaxKind::PlusToken | SyntaxKind::MinusToken => {
                let token = self.create_token_node();
                self.next_token();
                if token.kind != SyntaxKind::ReadonlyKeyword {
                    self.expect(SyntaxKind::ReadonlyKeyword);
                }
                Some(token)
            }
            _ => None,
        };

        self.expect(SyntaxKind::OpenBracketToken);
        let type_parameter = self.parse_mapped_type_parameter();
        let name_type = if self.parse_optional(SyntaxKind::AsKeyword) {
            Some(self.parse_type())
        } else {
            None
        };
        self.expect(SyntaxKind::CloseBracketToken);

        let question_token = match self.token {
            SyntaxKind::QuestionToken | SyntaxKind::PlusToken | SyntaxKind::MinusToken => {
                let token = self.create_token_node();
                self.next_token();
                if token.kind != SyntaxKind::QuestionToken {
                    self.expect(SyntaxKind::QuestionToken);
                }
                Some(token)
            }
            _ => None,
        };

        let type_node = self.parse_optional_type_annotation();
        self.parse_semicolon();
        let members = self.parse_list(ParsingContext::TypeMembers, Parser::parse_type_member);
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();

        Arc::new(Node::with_loc(
            SyntaxKind::MappedType,
            NodeData::MappedTypeNode(MappedTypeNodeData {
                readonly_token,
                type_parameter,
                name_type,
                question_token,
                type_node,
                members: Some(Arc::new(members)),
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_mapped_type_parameter(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let name = self.parse_identifier();
        self.expect(SyntaxKind::InKeyword);
        let constraint = self.parse_type();
        let end = constraint.end();
        Arc::new(Node::with_loc(
            SyntaxKind::TypeParameter,
            NodeData::TypeParameterDeclaration(TypeParameterDeclarationData {
                modifiers: None,
                name,
                constraint: Some(constraint),
                expression: None,
                default_type: None,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_type_literal(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let members = self.parse_list(ParsingContext::TypeMembers, Parser::parse_type_member);
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::TypeLiteral,
            NodeData::TypeLiteralNode(TypeLiteralNodeData {
                members: Arc::new(members),
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_tuple_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBracketToken);
        let elements = self.parse_delimited_list(
            ParsingContext::TupleElementTypes,
            Parser::parse_tuple_element_type,
        );
        self.expect(SyntaxKind::CloseBracketToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::TupleType,
            NodeData::TupleTypeNode(TupleTypeNodeData {
                elements: Arc::new(elements),
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_tuple_element_type(&mut self) -> Arc<Node> {

        if self.is_start_of_named_tuple_element() {
            return self.parse_named_tuple_member();
        }

        let pos = self.token_pos();
        if self.parse_optional(SyntaxKind::DotDotDotToken) {
            let type_node = self.parse_type();
            let end = type_node.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::RestType,
                NodeData::RestTypeNode(RestTypeNodeData { type_node }),
                TextRange::new(pos, end),
            ));
        }
        let type_node = self.parse_type();

        if self.parse_optional(SyntaxKind::QuestionToken) {
            let end = self.token_pos();
            return Arc::new(Node::with_loc(
                SyntaxKind::OptionalType,
                NodeData::OptionalTypeNode(OptionalTypeNodeData { type_node }),
                TextRange::new(pos, end),
            ));
        }
        type_node
    }

    fn is_start_of_named_tuple_element(&self) -> bool {
        if self.token == SyntaxKind::DotDotDotToken {

            let next = self.look_ahead_token();
            if !is_identifier_or_keyword(next) {
                return false;
            }

            let after = self.look_ahead_2_tokens();
            return after == SyntaxKind::ColonToken
                || (after == SyntaxKind::QuestionToken
                    && self.look_ahead_3_tokens() == SyntaxKind::ColonToken);
        }
        if is_identifier_or_keyword(self.token) {
            let next = self.look_ahead_token();

            if next == SyntaxKind::ColonToken {
                return true;
            }

            if next == SyntaxKind::QuestionToken {
                return self.look_ahead_2_tokens() == SyntaxKind::ColonToken;
            }
        }
        false
    }

    fn parse_named_tuple_member(&mut self) -> Arc<Node> {

        let pos = self.token_pos();
        let dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);
        let name = self.parse_identifier();
        let question_token = self.parse_optional_token(SyntaxKind::QuestionToken);
        self.expect(SyntaxKind::ColonToken);
        let type_node = self.parse_tuple_element_type();
        let end = type_node.end();
        Arc::new(Node::with_loc(
            SyntaxKind::NamedTupleMember,
            NodeData::NamedTupleMember(NamedTupleMemberData {
                dot_dot_dot_token,
                name,
                question_token,
                type_node,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_parenthesized_or_function_type(&mut self) -> Arc<Node> {

        let pos = self.token_pos();
        if self.is_start_of_function_type_with_open_paren() {

            let parameters = self.parse_parameter_list();
            self.expect(SyntaxKind::EqualsGreaterThanToken);
            let type_node = if self.is_start_of_type() {
                Some(self.parse_type_or_type_predicate())
            } else {
                None
            };
            let end = type_node.as_ref().map_or(self.token_pos(), |n| n.end());
            return Arc::new(Node::with_loc(
                SyntaxKind::FunctionType,
                NodeData::FunctionTypeNode(FunctionTypeNodeData {
                    type_parameters: None,
                    parameters,
                    type_node,
                }),
                TextRange::new(pos, end),
            ));
        }

        self.expect(SyntaxKind::OpenParenToken);
        let type_node = self.parse_type();
        self.expect(SyntaxKind::CloseParenToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ParenthesizedType,
            NodeData::ParenthesizedTypeNode(ParenthesizedTypeNodeData { type_node }),
            TextRange::new(pos, end),
        ))
    }

    fn is_start_of_function_type_with_open_paren(&self) -> bool {

        let mut scanner = self.scanner.clone();
        let t1 = scanner.scan();

        if t1 == SyntaxKind::CloseParenToken || t1 == SyntaxKind::DotDotDotToken {
            return true;
        }

        let mut t = t1;
        while is_modifier_kind(t) {
            let mut probe = scanner.clone();
            let next = probe.scan();
            if probe.has_preceding_line_break() || !Self::token_can_follow_modifier(next) {
                break;
            }
            t = scanner.scan();
        }
        if t == SyntaxKind::DotDotDotToken {
            t = scanner.scan();
        }

        if t == SyntaxKind::OpenBraceToken || t == SyntaxKind::OpenBracketToken {
            let mut depth = 1usize;
            loop {
                match scanner.scan() {
                    SyntaxKind::OpenBraceToken
                    | SyntaxKind::OpenBracketToken
                    | SyntaxKind::OpenParenToken => depth += 1,
                    SyntaxKind::CloseBraceToken
                    | SyntaxKind::CloseBracketToken
                    | SyntaxKind::CloseParenToken => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            break;
                        }
                    }
                    SyntaxKind::EndOfFile => return false,
                    _ => {}
                }
            }
            let t2 = scanner.scan();
            return matches!(
                t2,
                SyntaxKind::ColonToken
                    | SyntaxKind::CommaToken
                    | SyntaxKind::QuestionToken
                    | SyntaxKind::EqualsToken
            ) || (t2 == SyntaxKind::CloseParenToken
                && scanner.scan() == SyntaxKind::EqualsGreaterThanToken);
        }

        if !is_identifier_or_keyword(t) {
            return false;
        }
        let t2 = scanner.scan();
        matches!(
            t2,
            SyntaxKind::ColonToken
                | SyntaxKind::CommaToken
                | SyntaxKind::QuestionToken
                | SyntaxKind::EqualsToken
        ) || (t2 == SyntaxKind::CloseParenToken
            && scanner.scan() == SyntaxKind::EqualsGreaterThanToken)
    }

    fn parse_function_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let type_node = if self.is_start_of_type() {
            Some(self.parse_type_or_type_predicate())
        } else {
            None
        };
        let end = type_node.as_ref().map_or(self.token_pos(), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::FunctionType,
            NodeData::FunctionTypeNode(FunctionTypeNodeData {
                type_parameters,
                parameters,
                type_node,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_constructor_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let modifiers = if self.token == SyntaxKind::AbstractKeyword {
            let modifier_pos = self.token_pos();
            let modifier_end = self.token_end();
            self.next_token();
            Some(self.make_modifier_list(vec![(
                SyntaxKind::AbstractKeyword,
                modifier_pos,
                modifier_end,
            )]))
        } else {
            None
        };
        self.expect(SyntaxKind::NewKeyword);
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let type_node = if self.is_start_of_type() {
            Some(self.parse_type_or_type_predicate())
        } else {
            None
        };
        let end = type_node.as_ref().map_or(self.token_pos(), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::ConstructorType,
            NodeData::ConstructorTypeNode(ConstructorTypeNodeData {
                modifiers,
                type_parameters,
                parameters,
                type_node,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_if_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::IfKeyword);
        self.expect(SyntaxKind::OpenParenToken);
        let expression = self.parse_expression();
        self.expect(SyntaxKind::CloseParenToken);
        let then_statement = self.parse_statement();
        let else_statement = if self.parse_optional(SyntaxKind::ElseKeyword) {
            Some(self.parse_statement())
        } else {
            None
        };
        let end = else_statement
            .as_ref()
            .map_or(then_statement.end(), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::IfStatement,
            NodeData::IfStatement(IfStatementData {
                expression,
                then_statement,
                else_statement,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_do_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::DoKeyword);
        let statement = self.parse_statement();
        self.expect(SyntaxKind::WhileKeyword);
        self.expect(SyntaxKind::OpenParenToken);
        let expression = self.parse_expression();
        self.expect(SyntaxKind::CloseParenToken);
        self.parse_optional(SyntaxKind::SemicolonToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::DoStatement,
            NodeData::DoStatement(DoStatementData {
                statement,
                expression,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_while_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::WhileKeyword);
        self.expect(SyntaxKind::OpenParenToken);
        let expression = self.parse_expression();
        self.expect(SyntaxKind::CloseParenToken);
        let statement = self.parse_statement();
        let end = statement.end();
        Arc::new(Node::with_loc(
            SyntaxKind::WhileStatement,
            NodeData::WhileStatement(WhileStatementData {
                expression,
                statement,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_for_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::ForKeyword);

        let await_modifier = if self.token == SyntaxKind::AwaitKeyword {
            let node = self.create_token_node();
            self.next_token();
            Some(node)
        } else {
            None
        };
        self.expect(SyntaxKind::OpenParenToken);
        let initializer = if self.token != SyntaxKind::SemicolonToken {
            if matches!(
                self.token,
                SyntaxKind::VarKeyword | SyntaxKind::LetKeyword | SyntaxKind::ConstKeyword
            ) {
                Some(self.parse_variable_declaration_list(true))
            } else {
                Some(self.parse_expression())
            }
        } else {
            None
        };

        if self.token == SyntaxKind::InKeyword {
            self.next_token();
            let expression = self.parse_expression();
            self.expect(SyntaxKind::CloseParenToken);
            let statement = self.parse_statement();
            let end = statement.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::ForInStatement,
                NodeData::ForInOrOfStatement(ForInOrOfStatementData {
                    await_modifier: None,
                    initializer: initializer.unwrap(),
                    expression,
                    statement,
                }),
                TextRange::new(pos, end),
            ));
        }
        if self.token == SyntaxKind::OfKeyword {
            self.next_token();
            let expression = self.parse_expression();
            self.expect(SyntaxKind::CloseParenToken);
            let statement = self.parse_statement();
            let end = statement.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::ForOfStatement,
                NodeData::ForInOrOfStatement(ForInOrOfStatementData {
                    await_modifier,
                    initializer: initializer.unwrap(),
                    expression,
                    statement,
                }),
                TextRange::new(pos, end),
            ));
        }

        self.expect(SyntaxKind::SemicolonToken);
        let condition = if self.token != SyntaxKind::SemicolonToken
            && self.token != SyntaxKind::CloseParenToken
        {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect(SyntaxKind::SemicolonToken);
        let incrementor = if self.token != SyntaxKind::CloseParenToken {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect(SyntaxKind::CloseParenToken);
        let statement = self.parse_statement();
        let end = statement.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ForStatement,
            NodeData::ForStatement(ForStatementData {
                initializer,
                condition,
                incrementor,
                statement,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_break_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::BreakKeyword);
        let label = self.parse_identifier_if_not_semicolon();
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::BreakStatement,
            NodeData::BreakStatement(BreakStatementData { label }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_continue_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::ContinueKeyword);
        let label = self.parse_identifier_if_not_semicolon();
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ContinueStatement,
            NodeData::ContinueStatement(ContinueStatementData { label }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_identifier_if_not_semicolon(&mut self) -> Option<Arc<Node>> {
        if !self.can_parse_semicolon() {
            Some(self.parse_identifier())
        } else {
            None
        }
    }

    fn parse_return_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::ReturnKeyword);
        let expression = if !self.can_parse_semicolon() {
            Some(self.parse_expression())
        } else {
            None
        };
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ReturnStatement,
            NodeData::ReturnStatement(ReturnStatementData { expression }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_switch_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::SwitchKeyword);
        self.expect(SyntaxKind::OpenParenToken);
        let expression = self.parse_expression();
        self.expect(SyntaxKind::CloseParenToken);
        let case_block = self.parse_case_block();
        let end = case_block.end();
        Arc::new(Node::with_loc(
            SyntaxKind::SwitchStatement,
            NodeData::SwitchStatement(SwitchStatementData {
                expression,
                case_block,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_case_block(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let clauses = self.parse_list(
            ParsingContext::SwitchClauses,
            Parser::parse_case_or_default_clause,
        );
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::CaseBlock,
            NodeData::CaseBlock(CaseBlockData {
                clauses: Arc::new(clauses),
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_case_or_default_clause(&mut self) -> Arc<Node> {
        if self.token == SyntaxKind::CaseKeyword {
            let pos = self.token_pos();
            self.next_token();
            let expression = self.parse_expression();
            self.expect(SyntaxKind::ColonToken);
            let statements = self.parse_list(
                ParsingContext::SwitchClauseStatements,
                Parser::parse_statement,
            );
            let end = self.token_pos();
            Arc::new(Node::with_loc(
                SyntaxKind::CaseClause,
                NodeData::CaseOrDefaultClause(CaseOrDefaultClauseData {
                    expression,
                    statements: Arc::new(statements),
                }),
                TextRange::new(pos, end),
            ))
        } else {
            let pos = self.token_pos();
            self.expect(SyntaxKind::DefaultKeyword);
            self.expect(SyntaxKind::ColonToken);
            let statements = self.parse_list(
                ParsingContext::SwitchClauseStatements,
                Parser::parse_statement,
            );
            let end = self.token_pos();
            Arc::new(Node::with_loc(
                SyntaxKind::DefaultClause,
                NodeData::CaseOrDefaultClause(CaseOrDefaultClauseData {
                    expression: Arc::new(Node::with_loc(
                        SyntaxKind::Unknown,
                        NodeData::Token,
                        TextRange::new(pos, pos),
                    )),
                    statements: Arc::new(statements),
                }),
                TextRange::new(pos, end),
            ))
        }
    }

    fn parse_throw_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::ThrowKeyword);
        let expression = if !self.has_preceding_line_break() {
            self.parse_expression()
        } else {

            Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData {
                    text: String::new(),
                }),
                TextRange::new(self.token_pos(), self.token_pos()),
            ))
        };

        if !self.try_parse_semicolon() {
            self.parse_error_for_missing_semicolon_after(&expression);
        }
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ThrowStatement,
            NodeData::ThrowStatement(ThrowStatementData { expression }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_try_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::TryKeyword);
        let try_block = self.parse_block();
        let catch_clause = if self.token == SyntaxKind::CatchKeyword {
            Some(self.parse_catch_clause())
        } else {
            None
        };
        let finally_block = if catch_clause.is_none() || self.token == SyntaxKind::FinallyKeyword {
            self.expect(SyntaxKind::FinallyKeyword);
            Some(self.parse_block())
        } else {
            None
        };
        let end = finally_block.as_ref().map_or_else(
            || catch_clause.as_ref().map_or(try_block.end(), |c| c.end()),
            |f| f.end(),
        );
        Arc::new(Node::with_loc(
            SyntaxKind::TryStatement,
            NodeData::TryStatement(TryStatementData {
                try_block,
                catch_clause,
                finally_block,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_catch_clause(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::CatchKeyword);
        let variable_declaration = if self.parse_optional(SyntaxKind::OpenParenToken) {
            let name = self.parse_identifier_or_pattern();
            let type_node = self.parse_optional_type_annotation();
            self.expect(SyntaxKind::CloseParenToken);
            Some(Arc::new(Node::with_loc(
                SyntaxKind::VariableDeclaration,
                NodeData::VariableDeclaration(VariableDeclarationData {
                    name,
                    exclamation_token: None,
                    type_node,
                    initializer: None,
                }),
                TextRange::new(pos, self.token_pos()),
            )))
        } else {
            None
        };
        let block = self.parse_block();
        let end = block.end();
        Arc::new(Node::with_loc(
            SyntaxKind::CatchClause,
            NodeData::CatchClause(CatchClauseData {
                variable_declaration,
                block,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_debugger_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::DebuggerKeyword);
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::DebuggerStatement,
            NodeData::DebuggerStatement,
            TextRange::new(pos, end),
        ))
    }

    fn parse_expression_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let expression = self.parse_expression();

        if self.token == SyntaxKind::ColonToken && expression.kind == SyntaxKind::Identifier {
            self.next_token();
            let statement = self.parse_statement();
            let end = self.token_pos();
            return Arc::new(Node::with_loc(
                SyntaxKind::LabeledStatement,
                NodeData::LabeledStatement(LabeledStatementData {
                    label: expression,
                    statement,
                }),
                TextRange::new(pos, end),
            ));
        }

        if !self.try_parse_semicolon() {
            self.parse_error_for_missing_semicolon_after(&expression);
        }
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ExpressionStatement,
            NodeData::ExpressionStatement(ExpressionStatementData { expression }),
            TextRange::new(pos, end),
        ))
    }

    fn is_identifier(&self) -> bool {
        self.token == SyntaxKind::Identifier || is_keyword(self.token)
    }

    fn is_binding_identifier_or_pattern(&self) -> bool {

        self.is_identifier()
            || self.token == SyntaxKind::PrivateIdentifier
            || self.token == SyntaxKind::OpenBracketToken
            || self.token == SyntaxKind::OpenBraceToken
    }

    fn is_start_of_parameter(&self) -> bool {
        self.token == SyntaxKind::OpenBracketToken
            || self.token == SyntaxKind::OpenBraceToken
            || self.token == SyntaxKind::DotDotDotToken
            || self.is_identifier()
            || self.is_literal_property_name()
    }

    fn is_start_of_type(&self) -> bool {
        matches!(
            self.token,
            SyntaxKind::AnyKeyword
                | SyntaxKind::UnknownKeyword
                | SyntaxKind::StringKeyword
                | SyntaxKind::NumberKeyword
                | SyntaxKind::BigIntKeyword
                | SyntaxKind::BooleanKeyword
                | SyntaxKind::UndefinedKeyword
                | SyntaxKind::NeverKeyword
                | SyntaxKind::ObjectKeyword
                | SyntaxKind::VoidKeyword
                | SyntaxKind::NullKeyword
                | SyntaxKind::TrueKeyword
                | SyntaxKind::FalseKeyword
                | SyntaxKind::ThisKeyword
                | SyntaxKind::TypeOfKeyword
                | SyntaxKind::KeyOfKeyword
                | SyntaxKind::UniqueKeyword
                | SyntaxKind::ReadonlyKeyword
                | SyntaxKind::NewKeyword
                | SyntaxKind::AbstractKeyword
                | SyntaxKind::StringLiteral
                | SyntaxKind::NumericLiteral
                | SyntaxKind::BigIntLiteral
                | SyntaxKind::NoSubstitutionTemplateLiteral
                | SyntaxKind::Identifier
                | SyntaxKind::OpenBraceToken
                | SyntaxKind::OpenBracketToken
                | SyntaxKind::OpenParenToken
                | SyntaxKind::LessThanToken
                | SyntaxKind::BarToken
                | SyntaxKind::AmpersandToken
                | SyntaxKind::AsteriskToken
                | SyntaxKind::QuestionToken
                | SyntaxKind::ExclamationToken
                | SyntaxKind::DotDotDotToken
                | SyntaxKind::MinusToken
                | SyntaxKind::TemplateHead
        ) || is_keyword(self.token)
    }

    fn is_literal_property_name(&self) -> bool {
        is_identifier_or_keyword(self.token)
            || self.token == SyntaxKind::StringLiteral
            || self.token == SyntaxKind::NumericLiteral
            || self.token == SyntaxKind::BigIntLiteral
            || self.token == SyntaxKind::PrivateIdentifier
    }

    fn parse_identifier(&mut self) -> Arc<Node> {
        self.parse_identifier_with_private_diagnostic(None)
    }

    fn parse_identifier_with_private_diagnostic(
        &mut self,
        private_msg: Option<&'static crate::diagnostics::Message>,
    ) -> Arc<Node> {
        if !self.is_identifier() {
            if self.token == SyntaxKind::PrivateIdentifier {
                let msg = private_msg
                    .unwrap_or(&diagnostics::PRIVATE_IDENTIFIERS_ARE_NOT_ALLOWED_OUTSIDE_CLASS_BODIES);
                self.parse_error_at_current_token(*msg, &[]);
            } else {
                self.parse_error_at_current_token(diagnostics::IDENTIFIER_EXPECTED, &[]);
            }
        }
        let text = self.scanner.token_text().to_string();
        let pos = self.token_pos();
        let end = self.token_end();
        self.next_token();
        Arc::new(Node::with_loc(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData { text }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_property_name(&mut self) -> Arc<Node> {
        match self.token {
            SyntaxKind::PrivateIdentifier => {
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::PrivateIdentifier,
                    NodeData::PrivateIdentifier(PrivateIdentifierData { text }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::StringLiteral => {
                let text = self.scanner.token_value();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::StringLiteral,
                    NodeData::StringLiteral(StringLiteralData {
                        text,
                        token_flags: 0,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::NumericLiteral => {
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::NumericLiteral,
                    NodeData::NumericLiteral(NumericLiteralData {
                        text,
                        token_flags: 0,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::BigIntLiteral => {
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::BigIntLiteral,
                    NodeData::BigIntLiteral(BigIntLiteralData {
                        text,
                        token_flags: 0,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::OpenBracketToken => {
                let pos = self.token_pos();
                self.next_token();
                let expression = self.parse_assignment_expression();
                self.expect(SyntaxKind::CloseBracketToken);
                let end = self.token_pos();
                Arc::new(Node::with_loc(
                    SyntaxKind::ComputedPropertyName,
                    NodeData::ComputedPropertyName(ComputedPropertyNameData { expression }),
                    TextRange::new(pos, end),
                ))
            }
            _ => self.parse_identifier(),
        }
    }

    pub fn parse_expression(&mut self) -> Arc<Node> {
        let expr = self.parse_assignment_expression();

        if self.token == SyntaxKind::CommaToken {
            let pos = expr.pos();
            let mut left = expr;
            loop {
                let comma_pos = self.token_pos();
                let comma_end = self.token_end();
                if !self.parse_optional(SyntaxKind::CommaToken) {
                    break;
                }
                let right = self.parse_assignment_expression();
                let end = right.end();
                left = Arc::new(Node::with_loc(
                    SyntaxKind::BinaryExpression,
                    NodeData::BinaryExpression(BinaryExpressionData {
                        modifiers: None,
                        left,
                        type_node: None,
                        operator_token: Arc::new(Node::with_loc(
                            SyntaxKind::CommaToken,
                            NodeData::Token,
                            TextRange::new(comma_pos, comma_end),
                        )),
                        right,
                    }),
                    TextRange::new(pos, end),
                ));
            }
            return left;
        }
        expr
    }

    fn parse_assignment_expression(&mut self) -> Arc<Node> {

        if self.is_yield_expression() {
            return self.parse_yield_expression();
        }

        if self.token == SyntaxKind::LessThanToken
            || (self.token == SyntaxKind::AsyncKeyword
                && self.look_ahead_token() == SyntaxKind::LessThanToken)
        {
            if let Some(arrow) = self.try_parse_generic_arrow_function() {
                return arrow;
            }
        }
        if self.token == SyntaxKind::AsyncKeyword && self.is_async_arrow_function() {

            let async_modifier = self.create_token_node();
            self.next_token();
            if self.token == SyntaxKind::OpenParenToken {
                return self.parse_parenthesized_arrow_function_with_async(async_modifier);
            }
            let identifier = self.parse_identifier();
            return self.parse_simple_arrow_function_with_async(identifier, async_modifier);
        }

        if self.token == SyntaxKind::OpenParenToken && self.is_parenthesized_arrow_function() {
            return self.parse_parenthesized_arrow_function();
        }

        let mut expr = self.parse_binary_expression(0);
        if expr.kind == SyntaxKind::Identifier && self.token == SyntaxKind::EqualsGreaterThanToken {
            return self.parse_simple_arrow_function(expr);
        }

        while self.token == SyntaxKind::AsKeyword || self.token == SyntaxKind::SatisfiesKeyword {
            let pos = expr.pos();
            let kind = self.token;
            self.next_token();

            let type_node = if kind == SyntaxKind::AsKeyword
                && self.token == SyntaxKind::ConstKeyword
                && !self.has_preceding_line_break()
            {
                let tp = self.token_pos();
                let te = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::ConstKeyword,
                    NodeData::KeywordTypeNode,
                    TextRange::new(tp, te),
                ))
            } else {
                self.parse_type()
            };
            let end = type_node.end();
            expr = match kind {
                SyntaxKind::AsKeyword => Arc::new(Node::with_loc(
                    SyntaxKind::AsExpression,
                    NodeData::AsExpression(AsExpressionData {
                        expression: expr,
                        type_node,
                    }),
                    TextRange::new(pos, end),
                )),
                _ => Arc::new(Node::with_loc(
                    SyntaxKind::SatisfiesExpression,
                    NodeData::SatisfiesExpression(SatisfiesExpressionData {
                        expression: expr,
                        type_node,
                    }),
                    TextRange::new(pos, end),
                )),
            };
        }

        if self.token == SyntaxKind::QuestionToken {
            let pos = expr.pos();
            let question_token = self.create_token_node();
            self.next_token();
            let when_true = self.parse_expression();
            let colon_token = self.create_token_node();
            self.expect(SyntaxKind::ColonToken);
            let when_false = self.parse_assignment_expression();
            let end = when_false.end();
            expr = Arc::new(Node::with_loc(
                SyntaxKind::ConditionalExpression,
                NodeData::ConditionalExpression(ConditionalExpressionData {
                    condition: expr,
                    question_token,
                    when_true,
                    colon_token,
                    when_false,
                }),
                TextRange::new(pos, end),
            ));
        }

        if is_assignment_operator(self.token) {
            let pos = expr.pos();
            let operator_token = self.create_token_node();
            self.next_token();
            let right = self.parse_assignment_expression();
            let end = right.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::BinaryExpression,
                NodeData::BinaryExpression(BinaryExpressionData {
                    modifiers: None,
                    left: expr,
                    type_node: None,
                    operator_token,
                    right,
                }),
                TextRange::new(pos, end),
            ));
        }
        expr
    }

    fn is_yield_expression(&self) -> bool {
        if self.token != SyntaxKind::YieldKeyword {
            return false;
        }
        if self.yield_context {
            return true;
        }

        let mut p = self.clone_state();
        p.next_token();
        if p.token == SyntaxKind::AsteriskToken {
            return true;
        }
        !p.has_preceding_line_break() && p.is_start_of_expression()
    }

    fn is_await_expression(&self) -> bool {
        if self.token != SyntaxKind::AwaitKeyword {
            return false;
        }
        if self.await_context {
            return true;
        }

        let mut p = self.clone_state();
        p.next_token();
        !p.has_preceding_line_break() && p.is_start_of_expression()
    }

    fn parse_yield_expression(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let (asterisk_token, expression) = if !self.has_preceding_line_break()
            && (self.token == SyntaxKind::AsteriskToken || self.is_start_of_expression())
        {
            let asterisk = if self.token == SyntaxKind::AsteriskToken {
                let node = self.create_token_node();
                self.next_token();
                Some(node)
            } else {
                None
            };
            let expr = self.parse_assignment_expression();
            (asterisk, Some(expr))
        } else {
            (None, None)
        };
        let end = expression.as_ref().map_or(self.token_pos(), |e| e.end());
        Arc::new(Node::with_loc(
            SyntaxKind::YieldExpression,
            NodeData::YieldExpression(YieldExpressionData {
                asterisk_token,
                expression,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn is_parenthesized_arrow_function(&self) -> bool {
        let mut scanner = self.scanner.clone();
        let mut depth = 1usize;
        let mut scanned_tokens = 0usize;
        loop {
            let token = scanner.scan();
            scanned_tokens += 1;
            match token {
                SyntaxKind::EndOfFile => return false,
                SyntaxKind::OpenParenToken
                | SyntaxKind::OpenBracketToken
                | SyntaxKind::OpenBraceToken => depth += 1,
                SyntaxKind::CloseParenToken => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        let next = scanner.scan();
                        if next == SyntaxKind::EqualsGreaterThanToken {
                            return true;
                        }
                        if next == SyntaxKind::ColonToken {
                            return Self::scanner_reaches_arrow_before_line_end(&mut scanner);
                        }

                        if next == SyntaxKind::OpenBraceToken && scanned_tokens == 1 {
                            return true;
                        }
                        return false;
                    }
                }
                SyntaxKind::CloseBracketToken | SyntaxKind::CloseBraceToken => {
                    depth = depth.saturating_sub(1);
                }
                _ => {}
            }
        }
    }

    fn scanner_reaches_arrow_before_line_end(scanner: &mut Scanner) -> bool {

        let mut depth = 0usize;
        loop {
            let token = scanner.scan();
            if scanner.has_preceding_line_break() {
                return false;
            }
            match token {
                SyntaxKind::EqualsGreaterThanToken if depth == 0 => return true,
                SyntaxKind::OpenBraceToken
                | SyntaxKind::OpenBracketToken
                | SyntaxKind::OpenParenToken => depth += 1,
                SyntaxKind::CloseBraceToken
                | SyntaxKind::CloseBracketToken
                | SyntaxKind::CloseParenToken => {
                    depth = depth.saturating_sub(1);
                }
                SyntaxKind::EndOfFile | SyntaxKind::SemicolonToken | SyntaxKind::CommaToken
                    if depth == 0 =>
                {
                    return false
                }
                _ => {}
            }
        }
    }

    fn is_async_arrow_function(&self) -> bool {
        let mut scanner = self.scanner.clone();
        let next = scanner.scan();
        if scanner.has_preceding_line_break() {
            return false;
        }
        if next == SyntaxKind::OpenParenToken {
            let mut depth = 1usize;
            loop {
                let token = scanner.scan();
                match token {
                    SyntaxKind::EndOfFile => return false,
                    SyntaxKind::OpenParenToken
                    | SyntaxKind::OpenBracketToken
                    | SyntaxKind::OpenBraceToken => depth += 1,
                    SyntaxKind::CloseParenToken => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            let next = scanner.scan();
                            if next == SyntaxKind::EqualsGreaterThanToken {
                                return true;
                            }
                            if next == SyntaxKind::ColonToken {

                                loop {
                                    match scanner.scan() {
                                        SyntaxKind::EqualsGreaterThanToken => return true,
                                        SyntaxKind::EndOfFile
                                        | SyntaxKind::SemicolonToken => return false,
                                        _ => {}
                                    }
                                }
                            }
                            return false;
                        }
                    }
                    SyntaxKind::CloseBracketToken | SyntaxKind::CloseBraceToken => {
                        depth = depth.saturating_sub(1);
                    }
                    _ => {}
                }
            }
        }
        if is_identifier_or_keyword(next) {
            return scanner.scan() == SyntaxKind::EqualsGreaterThanToken;
        }
        false
    }

    fn make_async_modifier_list(&self, async_modifier: Arc<Node>) -> Option<Arc<ModifierList>> {
        Some(Arc::new(ModifierList::new(
            vec![async_modifier],
            ModifierFlags::Async,
        )))
    }

    fn parse_parenthesized_arrow_function_with_async(
        &mut self,
        async_modifier: Arc<Node>,
    ) -> Arc<Node> {
        let modifiers = self.make_async_modifier_list(async_modifier);
        let pos = self.token_pos();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let equals_greater_than_token = self.create_token_node();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let saved_await = self.await_context;
        self.await_context = true;
        let body = if self.token == SyntaxKind::OpenBraceToken {
            self.parse_block()
        } else {
            self.parse_assignment_expression()
        };
        self.await_context = saved_await;
        let end = body.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ArrowFunction,
            NodeData::ArrowFunction(ArrowFunctionData {
                modifiers,
                type_parameters: None,
                parameters,
                type_node,
                equals_greater_than_token,
                body,
                full_signature: None,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn try_parse_generic_arrow_function(&mut self) -> Option<Arc<Node>> {
        let starts_with_async = self.token == SyntaxKind::AsyncKeyword;
        if !starts_with_async
            && (self.token != SyntaxKind::LessThanToken
                || self.language_variant == LanguageVariant::Jsx)
        {
            return None;
        }

        {
            let mut s = self.scanner.clone();
            if starts_with_async {
                let after = s.scan();
                if s.has_preceding_line_break() || after != SyntaxKind::LessThanToken {
                    return None;
                }
            }
            let t1 = s.scan();
            if !(t1 == SyntaxKind::Identifier
                || t1 == SyntaxKind::ConstKeyword
                || (t1 as i16) > (SyntaxKind::WithKeyword as i16))
            {
                return None;
            }
        }

        let saved_scanner = self.scanner.clone();
        let saved_token = self.token;
        let diag_len = self.diagnostics.len();
        let pos = self.token_pos();

        if starts_with_async {
            self.next_token();
        }

        let type_parameters = self.parse_optional_type_parameters();

        if type_parameters.is_none()
            || self.token != SyntaxKind::OpenParenToken
            || self.diagnostics.len() != diag_len
        {
            self.scanner = saved_scanner;
            self.token = saved_token;
            self.diagnostics.truncate(diag_len);
            return None;
        }
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();

        if self.token != SyntaxKind::EqualsGreaterThanToken || self.diagnostics.len() != diag_len {
            self.scanner = saved_scanner;
            self.token = saved_token;
            self.diagnostics.truncate(diag_len);
            return None;
        }

        let equals_greater_than_token = self.create_token_node();
        self.next_token();
        let body = if self.token == SyntaxKind::OpenBraceToken {
            self.parse_block()
        } else {
            self.parse_assignment_expression()
        };
        let end = body.end();
        Some(Arc::new(Node::with_loc(
            SyntaxKind::ArrowFunction,
            NodeData::ArrowFunction(ArrowFunctionData {
                modifiers: None,
                type_parameters,
                parameters,
                type_node,
                equals_greater_than_token,
                body,
                full_signature: None,
            }),
            TextRange::new(pos, end),
        )))
    }

    fn parse_simple_arrow_function_with_async(
        &mut self,
        identifier: Arc<Node>,
        async_modifier: Arc<Node>,
    ) -> Arc<Node> {
        let modifiers = self.make_async_modifier_list(async_modifier);
        let pos = identifier.pos();
        let parameter = Arc::new(Node::with_loc(
            SyntaxKind::Parameter,
            NodeData::ParameterDeclaration(ParameterDeclarationData {
                modifiers: None,
                dot_dot_dot_token: None,
                name: identifier,
                question_token: None,
                type_node: None,
                initializer: None,
            }),
            TextRange::new(pos, self.token_pos()),
        ));
        let parameters = Arc::new(NodeList {
            loc: TextRange::new(pos, self.token_pos()),
            nodes: vec![parameter],
        });
        let equals_greater_than_token = self.create_token_node();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let saved_await = self.await_context;
        self.await_context = true;
        let body = if self.token == SyntaxKind::OpenBraceToken {
            self.parse_block()
        } else {
            self.parse_assignment_expression()
        };
        self.await_context = saved_await;
        let end = body.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ArrowFunction,
            NodeData::ArrowFunction(ArrowFunctionData {
                modifiers,
                type_parameters: None,
                parameters,
                type_node: None,
                equals_greater_than_token,
                body,
                full_signature: None,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_parenthesized_arrow_function(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let equals_greater_than_token = self.create_token_node();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let body = if self.token == SyntaxKind::OpenBraceToken {
            self.parse_block()
        } else {
            self.parse_assignment_expression()
        };
        let end = body.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ArrowFunction,
            NodeData::ArrowFunction(ArrowFunctionData {
                modifiers: None,
                type_parameters: None,
                parameters,
                type_node,
                equals_greater_than_token,
                body,
                full_signature: None,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_simple_arrow_function(&mut self, identifier: Arc<Node>) -> Arc<Node> {
        let pos = identifier.pos();
        let parameter = Arc::new(Node::with_loc(
            SyntaxKind::Parameter,
            NodeData::ParameterDeclaration(ParameterDeclarationData {
                modifiers: None,
                dot_dot_dot_token: None,
                name: identifier,
                question_token: None,
                type_node: None,
                initializer: None,
            }),
            TextRange::new(pos, self.token_pos()),
        ));
        let parameters = Arc::new(NodeList {
            loc: TextRange::new(pos, self.token_pos()),
            nodes: vec![parameter],
        });
        let equals_greater_than_token = self.create_token_node();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let body = if self.token == SyntaxKind::OpenBraceToken {
            self.parse_block()
        } else {
            self.parse_assignment_expression()
        };
        let end = body.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ArrowFunction,
            NodeData::ArrowFunction(ArrowFunctionData {
                modifiers: None,
                type_parameters: None,
                parameters,
                type_node: None,
                equals_greater_than_token,
                body,
                full_signature: None,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_binary_expression(&mut self, min_precedence: u8) -> Arc<Node> {
        let mut left = self.parse_unary_expression();

        loop {
            let precedence = binary_precedence(self.token);
            if precedence == 0 || precedence < min_precedence {
                break;
            }
            let operator_token = self.create_token_node();
            self.next_token();
            let right = self.parse_binary_expression(precedence + 1);
            let loc = TextRange::new(left.pos(), right.end());
            left = Arc::new(Node::with_loc(
                SyntaxKind::BinaryExpression,
                NodeData::BinaryExpression(BinaryExpressionData {
                    modifiers: None,
                    left,
                    type_node: None,
                    operator_token,
                    right,
                }),
                loc,
            ));
        }

        left
    }

    fn parse_unary_expression(&mut self) -> Arc<Node> {
        match self.token {
            SyntaxKind::PlusToken
            | SyntaxKind::MinusToken
            | SyntaxKind::ExclamationToken
            | SyntaxKind::TildeToken
            | SyntaxKind::PlusPlusToken
            | SyntaxKind::MinusMinusToken => {
                let operator = self.token;
                let op_pos = self.token_pos();
                self.next_token();
                let operand = self.parse_unary_expression();
                let loc = TextRange::new(op_pos, operand.end());
                Arc::new(Node::with_loc(
                    SyntaxKind::PrefixUnaryExpression,
                    NodeData::PrefixUnaryExpression(PrefixUnaryExpressionData {
                        operator,
                        operand,
                    }),
                    loc,
                ))
            }
            SyntaxKind::TypeOfKeyword => {

                let pos = self.token_pos();
                self.next_token();
                let expression = self.parse_unary_expression();
                let end = expression.end();
                Arc::new(Node::with_loc(
                    SyntaxKind::TypeOfExpression,
                    NodeData::TypeOfExpression(TypeOfExpressionData { expression }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::VoidKeyword | SyntaxKind::DeleteKeyword => {
                let pos = self.token_pos();
                let is_delete = self.token == SyntaxKind::DeleteKeyword;
                self.next_token();
                let expression = self.parse_unary_expression();
                let end = expression.end();
                if is_delete {
                    Arc::new(Node::with_loc(
                        SyntaxKind::DeleteExpression,
                        NodeData::DeleteExpression(DeleteExpressionData { expression }),
                        TextRange::new(pos, end),
                    ))
                } else {
                    Arc::new(Node::with_loc(
                        SyntaxKind::VoidExpression,
                        NodeData::VoidExpression(VoidExpressionData { expression }),
                        TextRange::new(pos, end),
                    ))
                }
            }
            SyntaxKind::AwaitKeyword if self.is_await_expression() => {
                let pos = self.token_pos();
                self.next_token();
                let expression = self.parse_unary_expression();
                let end = expression.end();
                Arc::new(Node::with_loc(
                    SyntaxKind::AwaitExpression,
                    NodeData::AwaitExpression(AwaitExpressionData { expression }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::LessThanToken if self.language_variant != LanguageVariant::Jsx => {

                self.parse_type_assertion()
            }
            _ => self.parse_postfix_expression(),
        }
    }

    fn parse_type_assertion(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let type_node = self.parse_type();
        self.expect(SyntaxKind::GreaterThanToken);
        let expression = self.parse_unary_expression();
        let end = expression.end();
        Arc::new(Node::with_loc(
            SyntaxKind::TypeAssertionExpression,
            NodeData::TypeAssertion(TypeAssertionData {
                type_node,
                expression,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_postfix_expression(&mut self) -> Arc<Node> {
        let operand = self.parse_left_hand_side_expression();

        if !self.has_preceding_line_break()
            && (self.token == SyntaxKind::PlusPlusToken
                || self.token == SyntaxKind::MinusMinusToken)
        {
            let pos = operand.pos();
            let operator = self.token;
            let op_end = self.token_end();
            self.next_token();
            return Arc::new(Node::with_loc(
                SyntaxKind::PostfixUnaryExpression,
                NodeData::PostfixUnaryExpression(PostfixUnaryExpressionData { operand, operator }),
                TextRange::new(pos, op_end),
            ));
        }
        operand
    }

    fn parse_left_hand_side_expression(&mut self) -> Arc<Node> {
        let expr = if self.token == SyntaxKind::NewKeyword {
            self.parse_new_expression()
        } else {
            self.parse_primary_expression()
        };
        self.parse_call_and_member_chain(expr, false)
    }

    fn parse_member_chain(&mut self, expr: Arc<Node>) -> Arc<Node> {
        self.parse_call_and_member_chain(expr, true)
    }

    fn parse_new_expression(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let expression = if self.token == SyntaxKind::DotToken {
            self.next_token();
            let name = self.parse_identifier();
            let end = name.end();
            Arc::new(Node::with_loc(
                SyntaxKind::PropertyAccessExpression,
                NodeData::PropertyAccessExpression(PropertyAccessExpressionData {
                    expression: Arc::new(Node::with_loc(
                        SyntaxKind::Unknown,
                        NodeData::Token,
                        TextRange::new(pos, pos),
                    )),
                    question_dot_token: None,
                    name,
                }),
                TextRange::new(pos, end),
            ))
        } else {

            let primary = if self.token == SyntaxKind::NewKeyword {
                self.parse_new_expression()
            } else {
                self.parse_primary_expression()
            };
            self.parse_member_chain(primary)
        };
        let type_arguments = self.parse_optional_type_arguments();
        let arguments = if self.token == SyntaxKind::OpenParenToken {
            Some(self.parse_argument_list())
        } else {
            None
        };
        let end = arguments.as_ref().map_or(expression.end(), |a| a.end());
        Arc::new(Node::with_loc(
            SyntaxKind::NewExpression,
            NodeData::NewExpression(NewExpressionData {
                expression,
                type_arguments,
                arguments,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_call_and_member_chain(&mut self, expr: Arc<Node>, member_only: bool) -> Arc<Node> {
        let mut expr = expr;
        loop {

            if member_only
                && matches!(
                    self.token,
                    SyntaxKind::OpenParenToken | SyntaxKind::LessThanToken
                )
            {
                break;
            }
            match self.token {
                SyntaxKind::DotToken => {
                    let pos = expr.pos();
                    self.next_token();
                    let name = self.parse_property_name();
                    let end = name.end();
                    expr = Arc::new(Node::with_loc(
                        SyntaxKind::PropertyAccessExpression,
                        NodeData::PropertyAccessExpression(PropertyAccessExpressionData {
                            expression: expr,
                            question_dot_token: None,
                            name,
                        }),
                        TextRange::new(pos, end),
                    ));
                }
                SyntaxKind::QuestionDotToken => {
                    let pos = expr.pos();
                    let question_dot = self.create_token_node();
                    self.next_token();
                    if self.token == SyntaxKind::OpenParenToken {
                        let arguments = self.parse_argument_list();
                        let end = arguments.end();
                        expr = Arc::new(Node::with_loc(
                            SyntaxKind::CallExpression,
                            NodeData::CallExpression(CallExpressionData {
                                expression: expr,
                                question_dot_token: Some(question_dot),
                                type_arguments: None,
                                arguments,
                            }),
                            TextRange::new(pos, end),
                        ));
                    } else if self.token == SyntaxKind::OpenBracketToken {

                        self.next_token();
                        let argument = self.parse_expression();
                        self.expect(SyntaxKind::CloseBracketToken);
                        let end = self.token_pos();
                        expr = Arc::new(Node::with_loc(
                            SyntaxKind::ElementAccessExpression,
                            NodeData::ElementAccessExpression(ElementAccessExpressionData {
                                expression: expr,
                                question_dot_token: Some(question_dot),
                                argument_expression: argument,
                            }),
                            TextRange::new(pos, end),
                        ));
                    } else {
                        let name = self.parse_property_name();
                        let end = name.end();
                        expr = Arc::new(Node::with_loc(
                            SyntaxKind::PropertyAccessExpression,
                            NodeData::PropertyAccessExpression(PropertyAccessExpressionData {
                                expression: expr,
                                question_dot_token: Some(question_dot),
                                name,
                            }),
                            TextRange::new(pos, end),
                        ));
                    }
                }
                SyntaxKind::OpenParenToken => {
                    let pos = expr.pos();
                    let arguments = self.parse_argument_list();
                    let end = arguments.end();
                    expr = Arc::new(Node::with_loc(
                        SyntaxKind::CallExpression,
                        NodeData::CallExpression(CallExpressionData {
                            expression: expr,
                            question_dot_token: None,
                            type_arguments: None,
                            arguments,
                        }),
                        TextRange::new(pos, end),
                    ));
                }
                SyntaxKind::OpenBracketToken => {
                    let pos = expr.pos();
                    self.next_token();
                    let argument = self.parse_expression();
                    self.expect(SyntaxKind::CloseBracketToken);
                    let end = self.token_pos();
                    expr = Arc::new(Node::with_loc(
                        SyntaxKind::ElementAccessExpression,
                        NodeData::ElementAccessExpression(ElementAccessExpressionData {
                            expression: expr,
                            question_dot_token: None,
                            argument_expression: argument,
                        }),
                        TextRange::new(pos, end),
                    ));
                }
                SyntaxKind::LessThanToken => {
                    let pos = expr.pos();

                    let type_arguments = match self.try_parse_type_arguments(true) {
                        Some(ta) => ta,
                        None => break,
                    };
                    let arguments = self.parse_argument_list();
                    let end = arguments.end();
                    expr = Arc::new(Node::with_loc(
                        SyntaxKind::CallExpression,
                        NodeData::CallExpression(CallExpressionData {
                            expression: expr,
                            question_dot_token: None,
                            type_arguments: Some(type_arguments),
                            arguments,
                        }),
                        TextRange::new(pos, end),
                    ));
                }
                SyntaxKind::ExclamationToken if !self.has_preceding_line_break() => {

                    let pos = expr.pos();
                    self.next_token();
                    let end = self.token_pos();
                    expr = Arc::new(Node::with_loc(
                        SyntaxKind::NonNullExpression,
                        NodeData::NonNullExpression(NonNullExpressionData { expression: expr }),
                        TextRange::new(pos, end),
                    ));
                }
                _ => break,
            }
        }
        expr
    }

    fn parse_argument_list(&mut self) -> Arc<NodeList> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenParenToken);
        let nodes =
            self.parse_delimited_list(ParsingContext::ArgumentExpressions, Parser::parse_argument);
        self.expect(SyntaxKind::CloseParenToken);
        let end = self.token_pos();
        Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: nodes.nodes,
        })
    }

    fn parse_argument(&mut self) -> Arc<Node> {
        if self.parse_optional(SyntaxKind::DotDotDotToken) {
            let pos = self.token_pos();
            let expression = self.parse_assignment_expression();
            let end = expression.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::SpreadElement,
                NodeData::SpreadElement(SpreadElementData { expression }),
                TextRange::new(pos, end),
            ));
        }
        self.parse_assignment_expression()
    }

    fn parse_optional_type_arguments(&mut self) -> Option<Arc<NodeList>> {
        if self.token != SyntaxKind::LessThanToken {
            return None;
        }
        let pos = self.token_pos();
        self.next_token();
        let args = self.parse_delimited_list(ParsingContext::TypeArguments, Parser::parse_type);

        self.re_scan_greater_than();
        self.expect(SyntaxKind::GreaterThanToken);
        let end = self.token_pos();
        Some(Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: args.nodes,
        }))
    }

    fn try_parse_type_arguments(&mut self, require_following_paren: bool) -> Option<Arc<NodeList>> {
        if self.token != SyntaxKind::LessThanToken {
            return None;
        }
        let saved_scanner = self.scanner.clone();
        let saved_token = self.token;
        let diag_len = self.diagnostics.len();
        let pos = self.token_pos();
        self.next_token();
        let args = self.parse_delimited_list(ParsingContext::TypeArguments, Parser::parse_type);
        self.re_scan_greater_than();
        let closed_cleanly =
            self.token == SyntaxKind::GreaterThanToken && self.diagnostics.len() == diag_len;
        if !closed_cleanly {
            self.scanner = saved_scanner;
            self.token = saved_token;
            self.diagnostics.truncate(diag_len);
            return None;
        }
        self.next_token();
        if require_following_paren && self.token != SyntaxKind::OpenParenToken {
            self.scanner = saved_scanner;
            self.token = saved_token;
            self.diagnostics.truncate(diag_len);
            return None;
        }
        let end = self.token_pos();
        Some(Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: args.nodes,
        }))
    }

    fn parse_primary_expression(&mut self) -> Arc<Node> {
        match self.token {
            SyntaxKind::Identifier => {
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::Identifier,
                    NodeData::Identifier(IdentifierData { text }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::NumericLiteral => {
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::NumericLiteral,
                    NodeData::NumericLiteral(NumericLiteralData {
                        text,
                        token_flags: 0,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::BigIntLiteral => {
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::BigIntLiteral,
                    NodeData::BigIntLiteral(BigIntLiteralData {
                        text,
                        token_flags: 0,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::StringLiteral => {
                let text = self.scanner.token_value();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::StringLiteral,
                    NodeData::StringLiteral(StringLiteralData {
                        text,
                        token_flags: 0,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::NoSubstitutionTemplateLiteral => {
                let text = self.scanner.token_value();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::NoSubstitutionTemplateLiteral,
                    NodeData::NoSubstitutionTemplateLiteral(NoSubstitutionTemplateLiteralData {
                        text,
                        template_flags: 0,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::NullKeyword => self.parse_keyword_expression(SyntaxKind::NullKeyword),
            SyntaxKind::TrueKeyword => self.parse_keyword_expression(SyntaxKind::TrueKeyword),
            SyntaxKind::FalseKeyword => self.parse_keyword_expression(SyntaxKind::FalseKeyword),
            SyntaxKind::UndefinedKeyword => {
                self.parse_keyword_expression(SyntaxKind::UndefinedKeyword)
            }
            SyntaxKind::ThisKeyword => self.parse_keyword_expression(SyntaxKind::ThisKeyword),
            SyntaxKind::SuperKeyword => self.parse_keyword_expression(SyntaxKind::SuperKeyword),
            SyntaxKind::OpenParenToken => self.parse_parenthesized_or_arrow(),
            SyntaxKind::OpenBracketToken => self.parse_array_literal(),
            SyntaxKind::OpenBraceToken => self.parse_object_literal(),
            SyntaxKind::LessThanToken if self.language_variant == LanguageVariant::Jsx => {
                self.parse_jsx_element_or_fragment(true)
            }
            SyntaxKind::FunctionKeyword => self.parse_function_expression(),
            SyntaxKind::ClassKeyword => self.parse_class_expression(),

            SyntaxKind::ImportKeyword => {
                if matches!(
                    self.look_ahead_token(),
                    SyntaxKind::OpenParenToken | SyntaxKind::LessThanToken
                ) {
                    self.parse_keyword_expression(SyntaxKind::ImportKeyword)
                } else if self.is_import_meta() {
                    self.parse_import_meta()
                } else {
                    self.parse_fallback_identifier_or_error()
                }
            }

            SyntaxKind::AsyncKeyword if self.is_async_function_expression() => {
                self.parse_async_function_expression()
            }
            SyntaxKind::TemplateHead => self.parse_template_expression(),

            SyntaxKind::PrivateIdentifier => {
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::PrivateIdentifier,
                    NodeData::PrivateIdentifier(PrivateIdentifierData { text }),
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::SlashToken | SyntaxKind::SlashEqualsToken => {
                self.re_scan_slash_token();
                let text = self.scanner.token_text().to_string();
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::RegularExpressionLiteral,
                    NodeData::RegularExpressionLiteral(RegularExpressionLiteralData {
                        text,
                        token_flags: 0,
                    }),
                    TextRange::new(pos, end),
                ))
            }
            _ => {

                self.parse_fallback_identifier_or_error()
            }
        }
    }

    fn parse_fallback_identifier_or_error(&mut self) -> Arc<Node> {
        if is_identifier_or_keyword(self.token)
            && self.token != SyntaxKind::InKeyword
            && self.token != SyntaxKind::InstanceOfKeyword
        {
            let text = self.scanner.token_text().to_string();
            let pos = self.token_pos();
            let end = self.token_end();
            self.next_token();
            Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData { text }),
                TextRange::new(pos, end),
            ))
        } else {
            let pos = self.token_pos();
            let end = self.token_end();
            self.parse_error_at(pos, end, diagnostics::EXPRESSION_EXPECTED, &[]);
            Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData {
                    text: String::new(),
                }),
                TextRange::new(pos, pos),
            ))
        }
    }

    fn parse_keyword_expression(&mut self, kind: SyntaxKind) -> Arc<Node> {
        let pos = self.token_pos();
        let end = self.token_end();
        self.next_token();
        Arc::new(Node::with_loc(
            kind,
            NodeData::Token,
            TextRange::new(pos, end),
        ))
    }

    fn parse_parenthesized_or_arrow(&mut self) -> Arc<Node> {
        if self.is_parenthesized_arrow_function() {
            return self.parse_parenthesized_arrow_function();
        }

        let pos = self.token_pos();
        self.next_token();

        let expr = self.parse_expression();
        self.expect(SyntaxKind::CloseParenToken);
        let end = self.token_pos();

        if self.token == SyntaxKind::EqualsGreaterThanToken {
            let arrow_token = self.create_token_node();
            self.next_token();
            let body = if self.token == SyntaxKind::OpenBraceToken {
                self.parse_block()
            } else {
                self.parse_assignment_expression()
            };
            let end = body.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::ArrowFunction,
                NodeData::ArrowFunction(ArrowFunctionData {
                    modifiers: None,
                    type_parameters: None,
                    parameters: Arc::new(NodeList::default()),
                    type_node: None,
                    equals_greater_than_token: arrow_token,
                    body,
                    full_signature: None,
                }),
                TextRange::new(pos, end),
            ));
        }

        Arc::new(Node::with_loc(
            SyntaxKind::ParenthesizedExpression,
            NodeData::ParenthesizedExpression(ParenthesizedExpressionData { expression: expr }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_array_literal(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBracketToken);
        let elements = self.parse_delimited_list(
            ParsingContext::ArrayLiteralMembers,
            Parser::parse_array_literal_element,
        );
        self.expect(SyntaxKind::CloseBracketToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ArrayLiteralExpression,
            NodeData::ArrayLiteralExpression(ArrayLiteralExpressionData {
                elements: Arc::new(elements),
                multi_line: false,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_array_literal_element(&mut self) -> Arc<Node> {
        if self.parse_optional(SyntaxKind::DotDotDotToken) {
            let pos = self.token_pos();
            let expression = self.parse_assignment_expression();
            let end = expression.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::SpreadElement,
                NodeData::SpreadElement(SpreadElementData { expression }),
                TextRange::new(pos, end),
            ));
        }
        self.parse_assignment_expression()
    }

    fn parse_object_literal(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let members = self.parse_delimited_list(
            ParsingContext::ObjectLiteralMembers,
            Parser::parse_object_literal_element,
        );
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ObjectLiteralExpression,
            NodeData::ObjectLiteralExpression(ObjectLiteralExpressionData {
                properties: Arc::new(members),
                multi_line: false,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_object_literal_element(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);
        if dot_dot_dot_token.is_some() {
            let expression = self.parse_assignment_expression();
            let end = expression.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::SpreadElement,
                NodeData::SpreadElement(SpreadElementData { expression }),
                TextRange::new(pos, end),
            ));
        }

        if self.token == SyntaxKind::GetKeyword || self.token == SyntaxKind::SetKeyword {
            let mut s = self.scanner.clone();
            s.scan();
            if Self::token_can_follow_get_or_set(s.token()) {
                let accessor_kind = self.token;
                self.next_token();
                let name = self.parse_property_name();
                let type_parameters = self.parse_optional_type_parameters();
                let parameters = self.parse_parameter_list();
                let type_node = self.parse_optional_return_type();

                let body = if self.token == SyntaxKind::OpenBraceToken {
                    Some(self.parse_block())
                } else {
                    self.parse_semicolon();
                    None
                };

                let end = body
                    .as_ref()
                    .map_or(self.scanner.full_start_pos(), |b| b.end());
                let range = TextRange::new(pos, end);
                return match accessor_kind {
                    SyntaxKind::GetKeyword => Arc::new(Node::with_loc(
                        SyntaxKind::GetAccessor,
                        NodeData::GetAccessorDeclaration(GetAccessorDeclarationData {
                            modifiers: None,
                            name,
                            type_parameters,
                            parameters,
                            type_node,
                            full_signature: None,
                            body,
                        }),
                        range,
                    )),
                    _ => Arc::new(Node::with_loc(
                        SyntaxKind::SetAccessor,
                        NodeData::SetAccessorDeclaration(SetAccessorDeclarationData {
                            modifiers: None,
                            name,
                            type_parameters,
                            parameters,
                            type_node,
                            full_signature: None,
                            body,
                        }),
                        range,
                    )),
                };
            }
        }

        let is_async = self.token == SyntaxKind::AsyncKeyword;
        if is_async {
            self.next_token();
        }

        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);

        let name = self.parse_property_name();
        if self.token == SyntaxKind::OpenParenToken
            || self.token == SyntaxKind::LessThanToken
            || asterisk_token.is_some()
            || is_async
        {

            let type_parameters = self.parse_optional_type_parameters();
            let parameters = self.parse_parameter_list();
            let type_node = self.parse_optional_return_type();

            let body = if self.token == SyntaxKind::OpenBraceToken {
                Some(self.parse_block())
            } else {
                self.expect(SyntaxKind::OpenBraceToken);
                None
            };
            let end = body.as_ref().map_or(self.token_pos(), |b| b.end());
            return Arc::new(Node::with_loc(
                SyntaxKind::MethodDeclaration,
                NodeData::MethodDeclaration(MethodDeclarationData {
                    modifiers: None,
                    asterisk_token,
                    name,
                    postfix_token: None,
                    type_parameters,
                    parameters,
                    type_node,
                    full_signature: None,
                    body,
                }),
                TextRange::new(pos, end),
            ));
        }

        if self.token == SyntaxKind::ColonToken {
            self.next_token();
            let initializer = self.parse_assignment_expression();
            let end = initializer.end();
            Arc::new(Node::with_loc(
                SyntaxKind::PropertyAssignment,
                NodeData::PropertyAssignment(PropertyAssignmentData {
                    modifiers: None,
                    name,
                    postfix_token: None,
                    type_node: Arc::new(Node::with_loc(
                        SyntaxKind::Unknown,
                        NodeData::Token,
                        TextRange::new(end, end),
                    )),
                    initializer,
                }),
                TextRange::new(pos, end),
            ))
        } else {

            let end = name.end();
            Arc::new(Node::with_loc(
                SyntaxKind::ShorthandPropertyAssignment,
                NodeData::ShorthandPropertyAssignment(ShorthandPropertyAssignmentData {
                    modifiers: None,
                    name,
                    postfix_token: None,
                    type_node: Arc::new(Node::with_loc(
                        SyntaxKind::Unknown,
                        NodeData::Token,
                        TextRange::new(end, end),
                    )),
                    equals_token: None,
                    object_assignment_initializer: None,
                }),
                TextRange::new(pos, end),
            ))
        }
    }

    #[allow(dead_code)]
    fn is_get_or_set_accessor(&self) -> bool {
        let mut scanner = self.scanner.clone();
        let next = scanner.scan();

        matches!(
            next,
            SyntaxKind::Identifier
                | SyntaxKind::StringLiteral
                | SyntaxKind::NumericLiteral
                | SyntaxKind::OpenBracketToken
                | SyntaxKind::PrivateIdentifier
        )
    }

    #[allow(dead_code)]
    fn parse_object_accessor(&mut self, pos: usize, is_get: bool) -> Arc<Node> {
        self.next_token();
        let name = self.parse_property_name();
        let body = self.parse_block();
        let end = body.end();
        let kind = if is_get {
            SyntaxKind::GetAccessor
        } else {
            SyntaxKind::SetAccessor
        };
        let data = if is_get {
            NodeData::GetAccessorDeclaration(GetAccessorDeclarationData {
                modifiers: None,
                name,
                type_parameters: None,
                parameters: Arc::new(NodeList::default()),
                type_node: None,
                full_signature: None,
                body: Some(body),
            })
        } else {
            NodeData::SetAccessorDeclaration(SetAccessorDeclarationData {
                modifiers: None,
                name,
                type_parameters: None,
                parameters: Arc::new(NodeList::default()),
                type_node: None,
                full_signature: None,
                body: Some(body),
            })
        };
        Arc::new(Node::with_loc(kind, data, TextRange::new(pos, end)))
    }

    #[allow(dead_code)]
    fn parse_class_accessor(
        &mut self,
        pos: usize,
        modifiers: Option<Arc<ModifierList>>,
        is_get: bool,
    ) -> Arc<Node> {
        self.next_token();
        let name = self.parse_property_name();
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let body = if self.token == SyntaxKind::OpenBraceToken {
            Some(self.parse_block())
        } else {
            self.parse_semicolon();
            None
        };
        let end = body.as_ref().map_or(self.token_pos(), |b| b.end());
        let kind = if is_get {
            SyntaxKind::GetAccessor
        } else {
            SyntaxKind::SetAccessor
        };
        let data = if is_get {
            NodeData::GetAccessorDeclaration(GetAccessorDeclarationData {
                modifiers,
                name,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                body,
            })
        } else {
            NodeData::SetAccessorDeclaration(SetAccessorDeclarationData {
                modifiers,
                name,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                body,
            })
        };
        Arc::new(Node::with_loc(kind, data, TextRange::new(pos, end)))
    }

    fn parse_jsx_element_or_fragment(&mut self, in_expression_context: bool) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::LessThanToken);

        if self.token == SyntaxKind::GreaterThanToken {
            let opening = Arc::new(Node::with_loc(
                SyntaxKind::JsxOpeningFragment,
                NodeData::JsxOpeningFragment,
                TextRange::new(pos, self.token_end()),
            ));

            self.scan_jsx_text();
            let children = self.parse_jsx_children();
            let closing_pos = self.token_pos();
            self.expect(SyntaxKind::LessThanSlashToken);
            let closing_end = self.token_end();
            self.expect_without_advancing(SyntaxKind::GreaterThanToken);
            if in_expression_context {
                self.next_token();
            } else {
                self.scan_jsx_text();
            }
            let closing = Arc::new(Node::with_loc(
                SyntaxKind::JsxClosingFragment,
                NodeData::JsxClosingFragment,
                TextRange::new(closing_pos, closing_end),
            ));
            return Arc::new(Node::with_loc(
                SyntaxKind::JsxFragment,
                NodeData::JsxFragment(JsxFragmentData {
                    opening_fragment: opening,
                    children,
                    closing_fragment: closing,
                }),
                TextRange::new(pos, closing_end),
            ));
        }

        let tag_name = self.parse_jsx_name();
        let attributes = self.parse_jsx_attributes();
        if self.parse_optional(SyntaxKind::SlashToken) {
            let end = self.token_end();
            self.expect_without_advancing(SyntaxKind::GreaterThanToken);

            if in_expression_context {
                self.next_token();
            } else {
                self.scan_jsx_text();
            }
            return Arc::new(Node::with_loc(
                SyntaxKind::JsxSelfClosingElement,
                NodeData::JsxSelfClosingElement(JsxSelfClosingElementData {
                    tag_name,
                    type_arguments: None,
                    attributes,
                }),
                TextRange::new(pos, end),
            ));
        }

        let opening_end = self.token_end();
        self.expect_without_advancing(SyntaxKind::GreaterThanToken);

        self.scan_jsx_text();
        let opening = Arc::new(Node::with_loc(
            SyntaxKind::JsxOpeningElement,
            NodeData::JsxOpeningElement(JsxOpeningElementData {
                tag_name,
                type_arguments: None,
                attributes,
            }),
            TextRange::new(pos, opening_end),
        ));
        let children = self.parse_jsx_children();
        let closing = self.parse_jsx_closing_element(in_expression_context);
        Arc::new(Node::with_loc(
            SyntaxKind::JsxElement,
            NodeData::JsxElement(JsxElementData {
                opening_element: opening,
                children,
                closing_element: closing,
            }),
            TextRange::new(pos, self.token_pos()),
        ))
    }

    fn parse_jsx_name(&mut self) -> Arc<Node> {
        let pos = self.token_pos();

        self.scan_jsx_identifier();
        let mut name = self.parse_identifier_name_or_keyword();
        while self.parse_optional(SyntaxKind::DotToken) {
            self.scan_jsx_identifier();
            let right = self.parse_identifier_name_or_keyword();
            let end = right.end();
            name = Arc::new(Node::with_loc(
                SyntaxKind::PropertyAccessExpression,
                NodeData::PropertyAccessExpression(PropertyAccessExpressionData {
                    expression: name,
                    question_dot_token: None,
                    name: right,
                }),
                TextRange::new(pos, end),
            ));
        }
        name
    }

    fn parse_jsx_attributes(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let mut properties = Vec::new();
        while self.token != SyntaxKind::GreaterThanToken
            && self.token != SyntaxKind::SlashToken
            && self.token != SyntaxKind::EndOfFile
        {
            properties.push(self.parse_jsx_attribute());
        }
        Arc::new(Node::with_loc(
            SyntaxKind::JsxAttributes,
            NodeData::JsxAttributes(JsxAttributesData {
                properties: Arc::new(NodeList {
                    loc: TextRange::new(pos, self.token_pos()),
                    nodes: properties,
                }),
            }),
            TextRange::new(pos, self.token_pos()),
        ))
    }

    fn parse_jsx_attribute(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        if self.token == SyntaxKind::OpenBraceToken {

            self.next_token();
            self.expect(SyntaxKind::DotDotDotToken);
            let expression = self.parse_expression();
            self.expect(SyntaxKind::CloseBraceToken);
            return Arc::new(Node::with_loc(
                SyntaxKind::JsxSpreadAttribute,
                NodeData::JsxSpreadAttribute(JsxSpreadAttributeData { expression }),
                TextRange::new(pos, self.token_pos()),
            ));
        }

        self.scan_jsx_identifier();
        let name = self.parse_identifier_name_or_keyword();
        let initializer = if self.parse_optional(SyntaxKind::EqualsToken) {
            if self.token == SyntaxKind::StringLiteral {
                Some(self.parse_string_literal_node())
            } else if self.token == SyntaxKind::OpenBraceToken {
                Some(self.parse_jsx_expression(true))
            } else if self.token == SyntaxKind::LessThanToken {
                Some(self.parse_jsx_element_or_fragment(true))
            } else {
                None
            }
        } else {
            None
        };
        Arc::new(Node::with_loc(
            SyntaxKind::JsxAttribute,
            NodeData::JsxAttribute(JsxAttributeData { name, initializer }),
            TextRange::new(pos, self.token_pos()),
        ))
    }

    fn parse_jsx_expression(&mut self, in_expression_context: bool) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let dot_dot_dot_token =
            if !in_expression_context && self.token == SyntaxKind::DotDotDotToken {
                self.parse_optional_token(SyntaxKind::DotDotDotToken)
            } else {
                None
            };
        let expression = if self.token == SyntaxKind::CloseBraceToken {
            None
        } else {
            Some(self.parse_expression())
        };
        if in_expression_context {
            self.expect(SyntaxKind::CloseBraceToken);
        } else {
            let end = self.token_end();
            self.expect_without_advancing(SyntaxKind::CloseBraceToken);
            self.scan_jsx_text();
            return Arc::new(Node::with_loc(
                SyntaxKind::JsxExpression,
                NodeData::JsxExpression(JsxExpressionData {
                    dot_dot_dot_token,
                    expression,
                }),
                TextRange::new(pos, end),
            ));
        }
        Arc::new(Node::with_loc(
            SyntaxKind::JsxExpression,
            NodeData::JsxExpression(JsxExpressionData {
                dot_dot_dot_token,
                expression,
            }),
            TextRange::new(pos, self.token_pos()),
        ))
    }

    fn parse_jsx_children(&mut self) -> Arc<NodeList> {
        let pos = self.token_pos();
        let mut children = Vec::new();
        loop {
            match self.token {
                SyntaxKind::EndOfFile | SyntaxKind::LessThanSlashToken => break,
                SyntaxKind::JsxText | SyntaxKind::JsxTextAllWhiteSpaces => {
                    children.push(self.parse_jsx_text());
                }
                SyntaxKind::OpenBraceToken => {
                    children.push(self.parse_jsx_expression(false));
                }
                SyntaxKind::LessThanToken => {
                    children.push(self.parse_jsx_element_or_fragment(false));
                }
                _ => break,
            }
        }
        Arc::new(NodeList {
            loc: TextRange::new(pos, self.token_pos()),
            nodes: children,
        })
    }

    fn parse_jsx_text(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let text = self.scanner.token_text().to_string();
        let end = self.token_end();
        let is_all_whitespace = self.token == SyntaxKind::JsxTextAllWhiteSpaces;
        self.scan_jsx_text();
        Arc::new(Node::with_loc(
            SyntaxKind::JsxText,
            NodeData::JsxText(JsxTextData {
                text,
                contains_only_trivia_white_spaces: is_all_whitespace,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_jsx_closing_element(&mut self, in_expression_context: bool) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::LessThanSlashToken);
        let tag_name = self.parse_jsx_name();
        let end = self.token_end();
        self.expect_without_advancing(SyntaxKind::GreaterThanToken);

        if in_expression_context {
            self.next_token();
        } else {
            self.scan_jsx_text();
        }
        Arc::new(Node::with_loc(
            SyntaxKind::JsxClosingElement,
            NodeData::JsxClosingElement(JsxClosingElementData { tag_name }),
            TextRange::new(pos, end),
        ))
    }

    fn is_import_meta(&self) -> bool {
        if self.token != SyntaxKind::ImportKeyword {
            return false;
        }
        let mut scanner = self.scanner.clone();
        scanner.scan() == SyntaxKind::DotToken && !scanner.has_preceding_line_break()
    }

    fn parse_import_meta(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        self.next_token();
        let name = self.parse_identifier_name_or_keyword();
        let end = name.end();
        Arc::new(Node::with_loc(
            SyntaxKind::MetaProperty,
            NodeData::MetaProperty(MetaPropertyData {
                keyword_token: SyntaxKind::ImportKeyword,
                name,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn is_async_function_expression(&self) -> bool {
        if self.token != SyntaxKind::AsyncKeyword {
            return false;
        }
        let mut scanner = self.scanner.clone();
        scanner.scan() == SyntaxKind::FunctionKeyword && !scanner.has_preceding_line_break()
    }

    fn parse_async_function_expression(&mut self) -> Arc<Node> {
        let async_modifier = self.create_token_node();
        self.next_token();
        let pos = async_modifier.pos();

        self.next_token();
        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);
        let name = if self.is_identifier() {
            Some(self.parse_identifier())
        } else {
            None
        };
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let is_generator = asterisk_token.is_some();
        let body = self.parse_function_block(is_generator, true);
        let end = body.end();
        let modifiers = self.make_async_modifier_list(async_modifier);
        Arc::new(Node::with_loc(
            SyntaxKind::FunctionExpression,
            NodeData::FunctionExpression(FunctionExpressionData {
                modifiers,
                asterisk_token,
                name,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                body,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_function_expression(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);
        let name = if self.is_identifier() {
            Some(self.parse_identifier())
        } else {
            None
        };
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let body = self.parse_block();
        let end = body.end();
        Arc::new(Node::with_loc(
            SyntaxKind::FunctionExpression,
            NodeData::FunctionExpression(FunctionExpressionData {
                modifiers: None,
                asterisk_token,
                name,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                body,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_class_expression(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();

        let name = if self.is_identifier()
            && !matches!(self.token, SyntaxKind::ExtendsKeyword | SyntaxKind::ImplementsKeyword)
        {
            Some(self.parse_identifier())
        } else {
            None
        };
        let type_parameters = self.parse_optional_type_parameters();
        let heritage_clauses = self.parse_heritage_clauses();
        let members = self.parse_class_members();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ClassExpression,
            NodeData::ClassExpression(ClassExpressionData {
                modifiers: None,
                name,
                type_parameters,
                heritage_clauses,
                members: Arc::new(members),
            }),
            TextRange::new(pos, end),
        ))
    }

    fn modifier_flag(kind: SyntaxKind) -> ModifierFlags {
        match kind {
            SyntaxKind::ExportKeyword => ModifierFlags::Export,
            SyntaxKind::DeclareKeyword => ModifierFlags::Ambient,
            SyntaxKind::DefaultKeyword => ModifierFlags::Default,
            SyntaxKind::AbstractKeyword => ModifierFlags::Abstract,
            SyntaxKind::StaticKeyword => ModifierFlags::Static,
            SyntaxKind::ReadonlyKeyword => ModifierFlags::Readonly,
            SyntaxKind::PublicKeyword => ModifierFlags::Public,
            SyntaxKind::PrivateKeyword => ModifierFlags::Private,
            SyntaxKind::ProtectedKeyword => ModifierFlags::Protected,
            SyntaxKind::AsyncKeyword => ModifierFlags::Async,
            SyntaxKind::ConstKeyword => ModifierFlags::Const,
            SyntaxKind::AccessorKeyword => ModifierFlags::Accessor,
            _ => ModifierFlags::empty(),
        }
    }

    fn make_modifier_list(&self, modifiers: Vec<(SyntaxKind, usize, usize)>) -> Arc<ModifierList> {
        let mut flags = ModifierFlags::empty();
        let nodes = modifiers
            .into_iter()
            .map(|(kind, pos, end)| {
                flags |= Self::modifier_flag(kind);
                Arc::new(Node::with_loc(
                    kind,
                    NodeData::Token,
                    TextRange::new(pos, end),
                ))
            })
            .collect();
        Arc::new(ModifierList::new(nodes, flags))
    }

    fn make_modifier_list_with_decorators(
        &self,
        modifiers: Vec<(SyntaxKind, usize, usize)>,
        decorators: Vec<Arc<Node>>,
    ) -> Arc<ModifierList> {
        let mut flags = ModifierFlags::empty();
        let mut nodes: Vec<Arc<Node>> = Vec::with_capacity(modifiers.len() + decorators.len());
        for (kind, pos, end) in modifiers {
            flags |= Self::modifier_flag(kind);
            nodes.push(Arc::new(Node::with_loc(
                kind,
                NodeData::Token,
                TextRange::new(pos, end),
            )));
        }
        if !decorators.is_empty() {
            flags |= ModifierFlags::Decorator;
            nodes.extend(decorators);
        }
        Arc::new(ModifierList::new(nodes, flags))
    }

    fn parse_decorator(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::AtToken);
        let expression = self.parse_left_hand_side_expression();
        let end = expression.end();
        Arc::new(Node::with_loc(
            SyntaxKind::Decorator,
            NodeData::Decorator(DecoratorData { expression }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_declaration_with_modifiers(
        &mut self,
        mut modifiers: Vec<(SyntaxKind, usize, usize)>,
    ) -> Arc<Node> {
        let mut decorators: Vec<Arc<Node>> = Vec::new();
        loop {
            if self.token == SyntaxKind::AtToken {
                decorators.push(self.parse_decorator());
                continue;
            }
            if !matches!(
                self.token,
                SyntaxKind::ExportKeyword
                    | SyntaxKind::DeclareKeyword
                    | SyntaxKind::DefaultKeyword
                    | SyntaxKind::AbstractKeyword
                    | SyntaxKind::AsyncKeyword
                    | SyntaxKind::ReadonlyKeyword
                    | SyntaxKind::PublicKeyword
                    | SyntaxKind::PrivateKeyword
                    | SyntaxKind::ProtectedKeyword
                    | SyntaxKind::StaticKeyword
                    | SyntaxKind::ConstKeyword
                    | SyntaxKind::AccessorKeyword
                    | SyntaxKind::OverrideKeyword
            ) {
                break;
            }

            let mut s = self.scanner.clone();
            s.scan();
            let can_follow = if self.token == SyntaxKind::ConstKeyword {

                s.token() == SyntaxKind::EnumKeyword
            } else {
                !s.has_preceding_line_break() && Self::token_can_follow_modifier(s.token())
            };
            if !can_follow {
                break;
            }
            let kind = self.token;
            let pos = self.token_pos();
            let end = self.token_end();
            self.next_token();
            modifiers.push((kind, pos, end));
        }

        let modifiers = Some(if decorators.is_empty() {
            self.make_modifier_list(modifiers)
        } else {
            self.make_modifier_list_with_decorators(modifiers, decorators)
        });
        match self.token {
            SyntaxKind::FunctionKeyword => {
                self.parse_function_declaration_with_modifiers(modifiers)
            }
            SyntaxKind::ClassKeyword => self.parse_class_declaration_with_modifiers(modifiers),
            SyntaxKind::InterfaceKeyword => {
                self.parse_interface_declaration_with_modifiers(modifiers)
            }
            SyntaxKind::TypeKeyword => self.parse_type_alias_declaration_with_modifiers(modifiers),
            SyntaxKind::EnumKeyword => self.parse_enum_declaration_with_modifiers(modifiers),
            SyntaxKind::NamespaceKeyword | SyntaxKind::ModuleKeyword => {
                self.parse_namespace_declaration_with_modifiers(modifiers)
            }
            SyntaxKind::GlobalKeyword => {

                self.parse_namespace_declaration_with_modifiers(modifiers)
            }
            SyntaxKind::VarKeyword | SyntaxKind::LetKeyword | SyntaxKind::ConstKeyword => {
                self.parse_variable_statement_with_modifiers(modifiers)
            }
            SyntaxKind::ImportKeyword => {

                self.parse_import_equals_declaration_with_modifiers(modifiers)
            }
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_import_equals_declaration_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let name = self.parse_identifier();
        self.parse_import_equals_tail(pos, modifiers, name, false)
    }

    fn parse_function_declaration(&mut self) -> Arc<Node> {
        self.parse_function_declaration_with_modifiers(None)
    }

    fn parse_function_declaration_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);
        let is_generator = asterisk_token.is_some();
        let is_async = modifiers
            .as_ref()
            .map(|m| m.flags().contains(ModifierFlags::Async))
            .unwrap_or(false);
        let name = if self.is_identifier() {
            Some(self.parse_identifier())
        } else {
            None
        };
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let body = if self.token == SyntaxKind::OpenBraceToken {
            Some(self.parse_function_block(is_generator, is_async))
        } else {
            self.parse_semicolon();
            None
        };
        let end = body.as_ref().map_or(self.token_pos(), |b| b.end());
        Arc::new(Node::with_loc(
            SyntaxKind::FunctionDeclaration,
            NodeData::FunctionDeclaration(FunctionDeclarationData {
                modifiers,
                asterisk_token,
                name,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                body,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_function_block(&mut self, is_generator: bool, is_async: bool) -> Arc<Node> {
        let saved_yield = self.yield_context;
        let saved_await = self.await_context;
        self.yield_context = is_generator;
        self.await_context = is_async;
        let block = self.parse_block();
        self.yield_context = saved_yield;
        self.await_context = saved_await;
        block
    }

    fn parse_class_declaration(&mut self) -> Arc<Node> {
        self.parse_class_declaration_with_modifiers(None)
    }

    fn parse_class_declaration_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let name = if self.is_identifier() {
            Some(self.parse_identifier())
        } else {
            None
        };
        let type_parameters = self.parse_optional_type_parameters();
        let heritage_clauses = self.parse_heritage_clauses();
        let members = self.parse_class_members();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ClassDeclaration,
            NodeData::ClassDeclaration(ClassDeclarationData {
                modifiers,
                name,
                type_parameters,
                heritage_clauses,
                members: Arc::new(members),
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_interface_declaration(&mut self) -> Arc<Node> {
        self.parse_interface_declaration_with_modifiers(None)
    }

    fn parse_interface_declaration_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let name = self.parse_identifier();
        let type_parameters = self.parse_optional_type_parameters();
        let heritage_clauses = self.parse_heritage_clauses();
        self.expect(SyntaxKind::OpenBraceToken);
        let members = self.parse_list(ParsingContext::TypeMembers, Parser::parse_type_member);
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::InterfaceDeclaration,
            NodeData::InterfaceDeclaration(InterfaceDeclarationData {
                modifiers,
                name,
                type_parameters,
                heritage_clauses,
                members: Arc::new(members),
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_type_alias_declaration(&mut self) -> Arc<Node> {
        self.parse_type_alias_declaration_with_modifiers(None)
    }

    fn parse_type_alias_declaration_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let name = self.parse_identifier();
        let type_parameters = self.parse_optional_type_parameters();
        self.expect(SyntaxKind::EqualsToken);
        let type_node = self.parse_type();
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::TypeAliasDeclaration,
            NodeData::TypeAliasDeclaration(TypeAliasDeclarationData {
                modifiers,
                name,
                type_parameters,
                type_node,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_enum_declaration(&mut self) -> Arc<Node> {
        self.parse_enum_declaration_with_modifiers(None)
    }

    fn parse_enum_declaration_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        let name = self.parse_identifier();
        self.expect(SyntaxKind::OpenBraceToken);
        let members =
            self.parse_delimited_list(ParsingContext::EnumMembers, Self::parse_enum_member);
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::EnumDeclaration,
            NodeData::EnumDeclaration(EnumDeclarationData {
                modifiers,
                name,
                members: Arc::new(members),
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_namespace_declaration(&mut self) -> Arc<Node> {
        self.parse_namespace_declaration_with_modifiers(None)
    }

    fn parse_namespace_declaration_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {

        let pos = self.token_pos();
        let keyword = self.token;

        if self.token == SyntaxKind::GlobalKeyword {
            return self.parse_ambient_external_module_declaration(pos, modifiers);
        }

        self.next_token();

        if self.token == SyntaxKind::StringLiteral {
            return self.parse_ambient_external_module_declaration(pos, modifiers);
        }

        let mut segments: Vec<Arc<Node>> = vec![self.parse_identifier()];
        while self.token == SyntaxKind::DotToken {
            self.next_token();
            segments.push(self.parse_identifier());
        }
        let body = if self.token == SyntaxKind::OpenBraceToken {
            let body_pos = self.token_pos();
            self.next_token();
            let statements =
                self.parse_list(ParsingContext::BlockStatements, Parser::parse_statement);
            self.expect(SyntaxKind::CloseBraceToken);
            let end = self.token_pos();
            Some(Arc::new(Node::with_loc(
                SyntaxKind::ModuleBlock,
                NodeData::ModuleBlock(ModuleBlockData {
                    statements: Arc::new(statements),
                }),
                TextRange::new(body_pos, end),
            )))
        } else {
            self.parse_semicolon();
            None
        };
        let end = body.as_ref().map_or(self.token_pos(), |b| b.end());

        let mut name = segments.pop().expect("at least one segment");
        let mut inner_body = body;

        let user_modifiers = modifiers;
        let mut mods = if segments.is_empty() {
            user_modifiers.clone()
        } else {
            None
        };
        let outermost = segments.is_empty();

        let export_only: Option<Arc<ModifierList>> = if segments.is_empty() {
            None
        } else {
            let export_tok = Arc::new(Node::with_loc(
                SyntaxKind::ExportKeyword,
                NodeData::Token,
                TextRange::new(pos, pos + 6),
            ));
            Some(Arc::new(ModifierList::new(
                vec![export_tok],
                ModifierFlags::Export,
            )))
        };
        loop {
            let decl = Arc::new(Node::with_loc(
                SyntaxKind::ModuleDeclaration,
                NodeData::ModuleDeclaration(ModuleDeclarationData {
                    modifiers: mods.clone().or_else(|| export_only.clone()),
                    keyword,
                    name: Arc::clone(&name),
                    body: inner_body,
                }),
                TextRange::new(pos, end),
            ));
            match segments.pop() {
                Some(seg) => {
                    name = seg;
                    inner_body = Some(decl);
                    mods = None;
                }
                None => {

                    if !outermost {
                        let decl_mut = Arc::as_ptr(&decl) as *mut Node;
                        unsafe {
                            if let NodeData::ModuleDeclaration(d) = &mut (*decl_mut).data {
                                d.modifiers = user_modifiers.clone();
                            }
                        }
                    }
                    return decl;
                }
            }
        }
    }

    fn parse_ambient_external_module_declaration(
        &mut self,
        pos: usize,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {

        let keyword = self.token;
        let name = if self.token == SyntaxKind::GlobalKeyword {

            self.parse_identifier()
        } else {

            self.parse_string_literal_name()
        };
        let body = if self.token == SyntaxKind::OpenBraceToken {
            let body_pos = self.token_pos();
            self.next_token();
            let statements =
                self.parse_list(ParsingContext::BlockStatements, Parser::parse_statement);
            self.expect(SyntaxKind::CloseBraceToken);
            let end = self.token_pos();
            Some(Arc::new(Node::with_loc(
                SyntaxKind::ModuleBlock,
                NodeData::ModuleBlock(ModuleBlockData {
                    statements: Arc::new(statements),
                }),
                TextRange::new(body_pos, end),
            )))
        } else {
            self.parse_semicolon();
            None
        };
        let end = body.as_ref().map_or(self.token_pos(), |b| b.end());
        Arc::new(Node::with_loc(
            SyntaxKind::ModuleDeclaration,
            NodeData::ModuleDeclaration(ModuleDeclarationData {
                modifiers,
                keyword,
                name,
                body,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_string_literal_name(&mut self) -> Arc<Node> {
        let text = self.scanner.token_text().to_string();
        let pos = self.token_pos();
        let end = self.token_end();
        self.next_token();
        Arc::new(Node::with_loc(
            SyntaxKind::StringLiteral,
            NodeData::StringLiteral(StringLiteralData {
                text,
                token_flags: 0,
            }),
            TextRange::new(pos, end),
        ))
    }

    #[allow(dead_code)]
    fn parse_namespace_name(&mut self) -> Arc<Node> {
        let name = self.parse_identifier();

        if self.token == SyntaxKind::DotToken {
            let pos = name.pos();
            let mut left = name;
            while self.token == SyntaxKind::DotToken {
                self.next_token();
                let right = self.parse_identifier();
                let end = right.end();
                left = Arc::new(Node::with_loc(
                    SyntaxKind::QualifiedName,
                    NodeData::QualifiedName(QualifiedNameData { left, right }),
                    TextRange::new(pos, end),
                ));
            }
            return left;
        }
        name
    }

    fn parse_import_declaration(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();

        let after_import_pos = self.token_pos();
        let mut identifier = if self.is_identifier() {
            Some(self.parse_identifier())
        } else {
            None
        };

        let mut phase_modifier = None;
        if let Some(id) = identifier.as_ref() {
            if id.text() == "type"
                && (self.token != SyntaxKind::FromKeyword
                    || (self.is_identifier()
                        && matches!(
                            self.look_ahead_token(),
                            SyntaxKind::FromKeyword | SyntaxKind::EqualsToken
                        )))
                && (self.is_identifier()
                    || self.token_after_import_definitely_produces_import_declaration())
            {
                phase_modifier = Some(SyntaxKind::TypeKeyword);
                identifier = if self.is_identifier() {
                    Some(self.parse_identifier())
                } else {
                    None
                };
            } else if id.text() == "defer" {
                let should_parse_as_defer_modifier = if self.token == SyntaxKind::FromKeyword {
                    self.look_ahead_token() != SyntaxKind::StringLiteral
                } else {
                    self.token != SyntaxKind::CommaToken && self.token != SyntaxKind::EqualsToken
                };
                if should_parse_as_defer_modifier {
                    phase_modifier = Some(SyntaxKind::DeferKeyword);
                    identifier = if self.is_identifier() {
                        Some(self.parse_identifier())
                    } else {
                        None
                    };
                }
            }
        }

        if let Some(id) = identifier.as_ref() {
            if !self.token_after_imported_identifier_definitely_produces_import_declaration()
                && phase_modifier != Some(SyntaxKind::DeferKeyword)
            {
                let is_type_only = phase_modifier == Some(SyntaxKind::TypeKeyword);
                return self.parse_import_equals_declaration(pos, id.clone(), is_type_only);
            }
        }

        let import_clause =
            self.try_parse_import_clause(identifier, after_import_pos, phase_modifier);
        let module_specifier = self.parse_module_specifier();
        let attributes = self.try_parse_import_attributes();
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ImportDeclaration,
            NodeData::ImportDeclaration(ImportDeclarationData {
                modifiers: None,
                import_clause,
                module_specifier,
                attributes,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn token_after_import_definitely_produces_import_declaration(&self) -> bool {
        self.token == SyntaxKind::AsteriskToken || self.token == SyntaxKind::OpenBraceToken
    }

    fn token_after_imported_identifier_definitely_produces_import_declaration(&self) -> bool {
        self.token == SyntaxKind::CommaToken || self.token == SyntaxKind::FromKeyword
    }

    fn parse_import_equals_declaration(
        &mut self,
        pos: usize,
        name: Arc<Node>,
        is_type_only: bool,
    ) -> Arc<Node> {
        self.parse_import_equals_tail(pos, None, name, is_type_only)
    }

    fn parse_import_equals_tail(
        &mut self,
        pos: usize,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        is_type_only: bool,
    ) -> Arc<Node> {
        self.expect(SyntaxKind::EqualsToken);
        let module_reference = self.parse_module_reference();
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ImportEqualsDeclaration,
            NodeData::ImportEqualsDeclaration(ImportEqualsDeclarationData {
                modifiers,
                is_type_only,
                name,
                module_reference,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_module_reference(&mut self) -> Arc<Node> {
        if self.token == SyntaxKind::RequireKeyword
            && self.look_ahead_token() == SyntaxKind::OpenParenToken
        {
            return self.parse_external_module_reference();
        }
        self.parse_entity_name()
    }

    fn parse_external_module_reference(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::RequireKeyword);
        self.expect(SyntaxKind::OpenParenToken);
        let expression = self.parse_module_specifier();
        self.expect(SyntaxKind::CloseParenToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ExternalModuleReference,
            NodeData::ExternalModuleReference(ExternalModuleReferenceData { expression }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_module_specifier(&mut self) -> Arc<Node> {
        if self.token == SyntaxKind::StringLiteral {
            return self.parse_string_literal_node();
        }
        self.parse_expression()
    }

    fn try_parse_import_attributes(&mut self) -> Option<Arc<Node>> {
        if self.token == SyntaxKind::WithKeyword
            || (self.token == SyntaxKind::AssertKeyword && !self.has_preceding_line_break())
        {

            let mut probe = self.scanner.clone();
            probe.scan();
            if probe.token() != SyntaxKind::OpenBraceToken {
                self.next_token();
                self.parse_error_at_current_token(
                    crate::diagnostics::X_0_EXPECTED,
                    &["{"],
                );
                return None;
            }
            Some(self.parse_import_attributes(self.token, false))
        } else {
            None
        }
    }

    fn parse_import_attributes(&mut self, token: SyntaxKind, skip_keyword: bool) -> Arc<Node> {
        let pos = self.token_pos();
        if !skip_keyword {
            self.next_token();
        }
        self.expect(SyntaxKind::OpenBraceToken);
        let multi_line = self.has_preceding_line_break();
        let attributes = self.parse_delimited_list(
            ParsingContext::ImportAttributes,
            Parser::parse_import_attribute,
        );
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ImportAttributes,
            NodeData::ImportAttributes(ImportAttributesData {
                token,
                attributes: Arc::new(attributes),
                multi_line,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_import_attribute(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let name = if is_identifier_or_keyword(self.token) {
            self.parse_identifier_name_or_keyword()
        } else if self.token == SyntaxKind::StringLiteral {
            self.parse_string_literal_node()
        } else {
            self.expect(SyntaxKind::Identifier);
            self.parse_identifier_name_or_keyword()
        };
        self.expect(SyntaxKind::ColonToken);
        let value = self.parse_assignment_expression();
        let end = value.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ImportAttribute,
            NodeData::ImportAttribute(ImportAttributeData { name, value }),
            TextRange::new(pos, end),
        ))
    }

    fn try_parse_import_clause(
        &mut self,
        identifier: Option<Arc<Node>>,
        pos: usize,
        phase_modifier: Option<SyntaxKind>,
    ) -> Option<Arc<Node>> {
        if identifier.is_some()
            || self.token == SyntaxKind::AsteriskToken
            || self.token == SyntaxKind::OpenBraceToken
        {
            let import_clause = self.parse_import_clause(identifier, pos, phase_modifier);
            self.expect(SyntaxKind::FromKeyword);
            Some(import_clause)
        } else {
            None
        }
    }

    fn parse_import_clause(
        &mut self,
        identifier: Option<Arc<Node>>,
        pos: usize,
        phase_modifier: Option<SyntaxKind>,
    ) -> Arc<Node> {
        let mut named_bindings = None;
        if identifier.is_none() || self.parse_optional(SyntaxKind::CommaToken) {
            named_bindings = if self.token == SyntaxKind::AsteriskToken {
                Some(self.parse_namespace_import())
            } else {
                Some(self.parse_named_imports())
            };
        }
        let end = named_bindings.as_ref().map_or_else(
            || identifier.as_ref().map_or(pos, |id| id.end()),
            |n| n.end(),
        );
        Arc::new(Node::with_loc(
            SyntaxKind::ImportClause,
            NodeData::ImportClause(ImportClauseData {
                phase_modifier,
                name: identifier,
                named_bindings,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_namespace_import(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token();
        self.expect(SyntaxKind::AsKeyword);
        let name = self.parse_identifier();
        let end = name.end();
        Arc::new(Node::with_loc(
            SyntaxKind::NamespaceImport,
            NodeData::NamespaceImport(NamespaceImportData { name }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_named_imports(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let elements = self.parse_list(
            ParsingContext::ImportOrExportSpecifiers,
            Parser::parse_import_specifier,
        );
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::NamedImports,
            NodeData::NamedImports(NamedImportsData {
                elements: Arc::new(elements),
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_import_specifier(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let (is_type_only, property_name, name) = self.parse_import_or_export_specifier(true);

        let name = if name.kind == SyntaxKind::Identifier {
            name
        } else {
            self.parse_error_at_range(
                TextRange::new(name.pos(), name.end()),
                diagnostics::IDENTIFIER_EXPECTED,
                &[],
            );
            Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData { text: String::new() }),
                TextRange::new(name.pos(), name.pos()),
            ))
        };

        self.parse_optional(SyntaxKind::CommaToken);
        let end = name.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ImportSpecifier,
            NodeData::ImportSpecifier(ImportSpecifierData {
                is_type_only,
                property_name,
                name,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_import_or_export_specifier(
        &mut self,
        is_import: bool,
    ) -> (bool, Option<Arc<Node>>, Arc<Node>) {
        let mut can_parse_as_keyword = true;
        let disallow_keywords = is_import;
        let (mut name, mut name_ok) = self.parse_module_export_name(disallow_keywords);
        let mut is_type_only = false;
        let mut property_name: Option<Arc<Node>> = None;
        if name.kind == SyntaxKind::Identifier && name.text() == "type" {
            if self.token == SyntaxKind::AsKeyword {

                let first_as = self.parse_identifier_name_or_keyword();
                if self.token == SyntaxKind::AsKeyword {

                    let second_as = self.parse_identifier_name_or_keyword();
                    if self.can_parse_module_export_name() {

                        is_type_only = true;
                        property_name = Some(first_as);
                        let (n, ok) = self.parse_module_export_name(disallow_keywords);
                        name = n;
                        name_ok = ok;
                        can_parse_as_keyword = false;
                    } else {

                        property_name = Some(name);
                        name = second_as;
                        can_parse_as_keyword = false;
                    }
                } else if self.can_parse_module_export_name() {

                    property_name = Some(name);
                    let (n, ok) = self.parse_module_export_name(disallow_keywords);
                    name = n;
                    name_ok = ok;
                    can_parse_as_keyword = false;
                } else {

                    is_type_only = true;
                    name = first_as;
                }
            } else if self.can_parse_module_export_name() {

                is_type_only = true;
                let (n, ok) = self.parse_module_export_name(disallow_keywords);
                name = n;
                name_ok = ok;
            }

        }
        if can_parse_as_keyword && self.token == SyntaxKind::AsKeyword {
            property_name = Some(name);
            self.expect(SyntaxKind::AsKeyword);
            let (n, ok) = self.parse_module_export_name(disallow_keywords);
            name = n;
            name_ok = ok;
        }
        if !name_ok {

            self.parse_error_at_range(
                TextRange::new(name.pos(), name.end()),
                diagnostics::IDENTIFIER_EXPECTED,
                &[],
            );
        }
        (is_type_only, property_name, name)
    }

    fn can_parse_module_export_name(&self) -> bool {
        is_identifier_or_keyword(self.token) || self.token == SyntaxKind::StringLiteral
    }

    fn parse_module_export_name(&mut self, disallow_keywords: bool) -> (Arc<Node>, bool) {
        if self.token == SyntaxKind::StringLiteral {
            return (self.parse_string_literal_node(), true);
        }
        let name_ok = !(disallow_keywords
            && is_keyword(self.token)
            && is_reserved_word_kind(self.token));
        (self.parse_identifier_name_or_keyword(), name_ok)
    }

    fn parse_identifier_name_or_keyword(&mut self) -> Arc<Node> {
        if self.is_identifier() {
            self.parse_identifier()
        } else {

            let text = format!("{:?}", self.token)
                .trim_end_matches("Keyword")
                .to_lowercase();
            let pos = self.token_pos();
            let end = self.token_end();
            self.next_token();
            Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData { text }),
                TextRange::new(pos, end),
            ))
        }
    }

    fn parse_string_literal_node(&mut self) -> Arc<Node> {
        let text = self.scanner.token_value();
        let pos = self.token_pos();
        let end = self.token_end();
        self.next_token();
        Arc::new(Node::with_loc(
            SyntaxKind::StringLiteral,
            NodeData::StringLiteral(StringLiteralData {
                text,
                token_flags: 0,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn make_export_modifier(&self, pos: usize, end: usize) -> ModifierList {
        let export_token = Arc::new(Node::with_loc(
            SyntaxKind::ExportKeyword,
            NodeData::Token,
            TextRange::new(pos, end),
        ));
        ModifierList::new(vec![export_token], ModifierFlags::Export)
    }

    fn parse_export_declaration(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let export_end = self.token_end();
        self.next_token();

        if matches!(
            self.token,
            SyntaxKind::DeclareKeyword
                | SyntaxKind::AsyncKeyword
                | SyntaxKind::AbstractKeyword
                | SyntaxKind::ReadonlyKeyword
                | SyntaxKind::PublicKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::ProtectedKeyword
                | SyntaxKind::StaticKeyword
        ) {
            return self.parse_declaration_with_modifiers(vec![(
                SyntaxKind::ExportKeyword,
                pos,
                export_end,
            )]);
        }

        if self.token == SyntaxKind::DefaultKeyword {
            let default_pos = self.token_pos();
            let default_end = self.token_end();
            self.next_token();
            if self.token == SyntaxKind::FunctionKeyword {

                let modifiers = self.make_modifier_list(vec![
                    (SyntaxKind::ExportKeyword, pos, export_end),
                    (SyntaxKind::DefaultKeyword, default_pos, default_end),
                ]);
                return self.parse_function_declaration_with_modifiers(Some(modifiers));
            }
            if self.token == SyntaxKind::ClassKeyword {
                let modifiers = self.make_modifier_list(vec![
                    (SyntaxKind::ExportKeyword, pos, export_end),
                    (SyntaxKind::DefaultKeyword, default_pos, default_end),
                ]);
                return self.parse_class_declaration_with_modifiers(Some(modifiers));
            }

            if self.token == SyntaxKind::InterfaceKeyword {
                return self.parse_declaration_with_modifiers(vec![
                    (SyntaxKind::ExportKeyword, pos, export_end),
                    (SyntaxKind::DefaultKeyword, default_pos, default_end),
                ]);
            }

            let expr = self.parse_assignment_expression();
            self.parse_semicolon();
            let end = self.token_pos();
            return Arc::new(Node::with_loc(
                SyntaxKind::ExportAssignment,
                NodeData::ExportAssignment(ExportAssignmentData {
                    modifiers: None,
                    is_export_equals: false,
                    type_node: expr.clone(),
                    expression: expr,
                }),
                TextRange::new(pos, end),
            ));
        }

        if self.token == SyntaxKind::EqualsToken {
            self.next_token();
            let expr = self.parse_assignment_expression();
            self.parse_semicolon();
            let end = self.token_pos();
            return Arc::new(Node::with_loc(
                SyntaxKind::ExportAssignment,
                NodeData::ExportAssignment(ExportAssignmentData {
                    modifiers: None,
                    is_export_equals: true,
                    type_node: expr.clone(),
                    expression: expr,
                }),
                TextRange::new(pos, end),
            ));
        }

        if self.token == SyntaxKind::AsKeyword {
            self.next_token();
            if self.token == SyntaxKind::NamespaceKeyword {
                self.next_token();
                let name = self.parse_identifier_name_or_keyword();
                self.parse_semicolon();
                let end = self.token_pos();
                return Arc::new(Node::with_loc(
                    SyntaxKind::NamespaceExportDeclaration,
                    NodeData::NamespaceExportDeclaration(NamespaceExportDeclarationData {
                        modifiers: None,
                        name,
                    }),
                    TextRange::new(pos, end),
                ));
            }
        }

        match self.token {
            SyntaxKind::FunctionKeyword
            | SyntaxKind::ClassKeyword
            | SyntaxKind::InterfaceKeyword
            | SyntaxKind::EnumKeyword
            | SyntaxKind::NamespaceKeyword
            | SyntaxKind::ModuleKeyword
            | SyntaxKind::ImportKeyword => {
                return self.parse_declaration_with_modifiers(vec![(
                    SyntaxKind::ExportKeyword,
                    pos,
                    export_end,
                )]);
            }
            SyntaxKind::TypeKeyword => {

                let mut s = self.scanner.clone();
                s.scan();
                if !s.has_preceding_line_break() && Self::token_is_identifier(&s) {
                    return self.parse_declaration_with_modifiers(vec![(
                        SyntaxKind::ExportKeyword,
                        pos,
                        export_end,
                    )]);
                }
                self.next_token();
                return self.parse_export_declaration_tail(pos, true);
            }
            SyntaxKind::ConstKeyword | SyntaxKind::LetKeyword | SyntaxKind::VarKeyword => {

                if self.token == SyntaxKind::ConstKeyword {
                    let mut s = self.scanner.clone();
                    if s.scan() == SyntaxKind::EnumKeyword {
                        return self.parse_declaration_with_modifiers(vec![(
                            SyntaxKind::ExportKeyword,
                            pos,
                            export_end,
                        )]);
                    }
                }

                let export_mod = self.make_export_modifier(pos, export_end);
                let declaration_list = self.parse_variable_declaration_list(false);
                self.parse_semicolon();
                let end = self.token_pos();
                return Arc::new(Node::with_loc(
                    SyntaxKind::VariableStatement,
                    NodeData::VariableStatement(VariableStatementData {
                        modifiers: Some(Arc::new(export_mod)),
                        declaration_list,
                    }),
                    TextRange::new(pos, end),
                ));
            }
            _ => {}
        }

        self.parse_export_declaration_tail(pos, false)
    }

    fn parse_export_declaration_tail(&mut self, pos: usize, is_type_only: bool) -> Arc<Node> {
        let export_clause = if self.parse_optional(SyntaxKind::AsteriskToken) {

            if self.parse_optional(SyntaxKind::AsKeyword) {

                let (name, _) = self.parse_module_export_name(false);
                let end = name.end();
                Some(Arc::new(Node::with_loc(
                    SyntaxKind::NamespaceExport,
                    NodeData::NamespaceExport(NamespaceExportData { name }),
                    TextRange::new(pos, end),
                )))
            } else {
                None
            }
        } else {

            Some(self.parse_named_exports())
        };

        let module_specifier = if self.parse_optional(SyntaxKind::FromKeyword) {
            Some(self.parse_string_literal_node())
        } else {
            None
        };
        let attributes = self.try_parse_import_attributes();
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ExportDeclaration,
            NodeData::ExportDeclaration(ExportDeclarationData {
                modifiers: None,
                is_type_only,
                export_clause,
                module_specifier,
                attributes,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_named_exports(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let elements = self.parse_list(
            ParsingContext::ImportOrExportSpecifiers,
            Parser::parse_export_specifier,
        );
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::NamedExports,
            NodeData::NamedExports(NamedExportsData {
                elements: Arc::new(elements),
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_export_specifier(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let (is_type_only, property_name, name) = self.parse_import_or_export_specifier(false);
        self.parse_optional(SyntaxKind::CommaToken);
        let end = name.end();
        Arc::new(Node::with_loc(
            SyntaxKind::ExportSpecifier,
            NodeData::ExportSpecifier(ExportSpecifierData {
                is_type_only,
                property_name,
                name,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_enum_member(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let name = self.parse_property_name();
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();
            Some(self.parse_assignment_expression())
        } else {
            None
        };

        let end = initializer.as_ref().map_or(name.end(), |i| i.end());
        Arc::new(Node::with_loc(
            SyntaxKind::EnumMember,
            NodeData::EnumMember(EnumMemberData { name, initializer }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_template_expression(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let head = self.create_token_node();
        self.next_token();
        let mut spans = Vec::new();
        loop {
            let expression = self.parse_expression();
            let literal = if self.token == SyntaxKind::CloseBraceToken {
                self.next_template_token();
                self.create_token_node()
            } else {
                break;
            };
            let span_pos = expression.pos();
            let span_end = literal.end();
            spans.push(Arc::new(Node::with_loc(
                SyntaxKind::TemplateSpan,
                NodeData::TemplateSpan(TemplateSpanData {
                    expression,
                    literal,
                }),
                TextRange::new(span_pos, span_end),
            )));
            if self.token == SyntaxKind::NoSubstitutionTemplateLiteral
                || self.token == SyntaxKind::TemplateTail
            {
                self.next_token();
                break;
            }
            self.next_token();
        }
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::TemplateExpression,
            NodeData::TemplateExpression(TemplateExpressionData {
                head,
                template_spans: Arc::new(NodeList {
                    loc: TextRange::new(pos, end),
                    nodes: spans,
                }),
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_optional_type_parameters(&mut self) -> Option<Arc<NodeList>> {
        if self.token != SyntaxKind::LessThanToken {
            return None;
        }
        let pos = self.token_pos();
        self.next_token();
        let params =
            self.parse_delimited_list(ParsingContext::TypeParameters, Parser::parse_type_parameter);

        self.re_scan_greater_than();

        if params.nodes.is_empty() {

            self.parse_error_at_range(
                crate::core::text::TextRange::new(pos, pos + 1),
                crate::diagnostics::messages_generated::TYPE_PARAMETER_LIST_CANNOT_BE_EMPTY,
                &[],
            );
        }
        self.expect(SyntaxKind::GreaterThanToken);
        let end = self.token_pos();
        Some(Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: params.nodes,
        }))
    }

    fn parse_type_parameter_modifiers(&mut self) -> Option<Arc<ModifierList>> {
        let mut modifiers: Vec<(SyntaxKind, usize, usize)> = Vec::new();
        loop {
            if !matches!(
                self.token,
                SyntaxKind::InKeyword | SyntaxKind::OutKeyword | SyntaxKind::ConstKeyword
            ) {
                break;
            }

            let mut s = self.scanner.clone();
            s.scan();
            if s.has_preceding_line_break() || !Self::token_can_follow_modifier(s.token()) {
                break;
            }
            let kind = self.token;
            let pos = self.token_pos();
            let end = self.token_end();
            self.next_token();
            modifiers.push((kind, pos, end));
        }
        if modifiers.is_empty() {
            None
        } else {
            Some(self.make_modifier_list(modifiers))
        }
    }

    fn parse_type_parameter(&mut self) -> Arc<Node> {
        let pos = self.token_pos();

        let modifiers = self.parse_type_parameter_modifiers();
        let name = self.parse_identifier();
        let constraint = if self.parse_optional(SyntaxKind::ExtendsKeyword) {
            Some(self.parse_type())
        } else {
            None
        };
        let default_type = if self.parse_optional(SyntaxKind::EqualsToken) {
            Some(self.parse_type())
        } else {
            None
        };
        let end = default_type.as_ref().map_or_else(
            || constraint.as_ref().map_or(name.end(), |c| c.end()),
            |d| d.end(),
        );
        Arc::new(Node::with_loc(
            SyntaxKind::TypeParameter,
            NodeData::TypeParameterDeclaration(TypeParameterDeclarationData {
                modifiers,
                name,
                constraint,
                expression: None,
                default_type,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_parameter_list(&mut self) -> Arc<NodeList> {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenParenToken);
        let params = self.parse_delimited_list(ParsingContext::Parameters, Parser::parse_parameter);
        self.expect(SyntaxKind::CloseParenToken);
        let end = self.token_pos();
        Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: params.nodes,
        })
    }

    fn token_after_modifier_can_follow(
        &self,
        s: &mut crate::scanner::Scanner,
    ) -> bool {
        match self.token {

            SyntaxKind::ConstKeyword => s.token() == SyntaxKind::EnumKeyword,
            SyntaxKind::ExportKeyword => {
                match s.token() {
                    SyntaxKind::DefaultKeyword => {
                        let t = s.scan();
                        Self::token_can_follow_default_keyword(t, s)
                    }
                    SyntaxKind::TypeKeyword => {
                        let t = s.scan();
                        Self::can_follow_export_modifier(t)
                    }
                    _ => Self::can_follow_export_modifier(s.token()),
                }
            }
            SyntaxKind::DefaultKeyword => Self::token_can_follow_default_keyword(s.token(), s),

            SyntaxKind::StaticKeyword => Self::token_can_follow_modifier(s.token()),
            _ => !s.has_preceding_line_break() && Self::token_can_follow_modifier(s.token()),
        }
    }

    fn parse_parameter_modifiers(&mut self) -> Option<Arc<ModifierList>> {
        enum Entry {
            Mod(SyntaxKind, usize, usize),
            Dec(Arc<Node>),
        }
        let mut entries: Vec<Entry> = Vec::new();
        let mut flags = ModifierFlags::empty();
        let mut has_leading_modifier = false;
        let mut has_trailing_decorator = false;
        let mut has_trailing_modifier = false;
        let mut has_static_modifier = false;
        loop {
            if self.token == SyntaxKind::AtToken && !has_trailing_modifier {
                let dec = self.parse_decorator();
                if has_leading_modifier {
                    has_trailing_decorator = true;
                }
                flags |= ModifierFlags::Decorator;
                entries.push(Entry::Dec(dec));
                continue;
            }

            if has_static_modifier && self.token == SyntaxKind::StaticKeyword {
                break;
            }
            if !is_modifier_kind(self.token) {
                break;
            }
            let mut s = self.scanner.clone();
            s.scan();
            if !self.token_after_modifier_can_follow(&mut s) {
                break;
            }
            let kind = self.token;
            let mpos = self.token_pos();
            let mend = self.token_end();
            self.next_token();
            if kind == SyntaxKind::StaticKeyword {
                has_static_modifier = true;
            }
            flags |= Self::modifier_flag(kind);
            entries.push(Entry::Mod(kind, mpos, mend));
            if has_trailing_decorator {
                has_trailing_modifier = true;
            } else {
                has_leading_modifier = true;
            }
        }
        if entries.is_empty() {
            return None;
        }
        let nodes = entries
            .into_iter()
            .map(|e| match e {
                Entry::Mod(kind, pos, end) => Arc::new(Node::with_loc(
                    kind,
                    NodeData::Token,
                    TextRange::new(pos, end),
                )),
                Entry::Dec(n) => n,
            })
            .collect();
        Some(Arc::new(ModifierList::new(nodes, flags)))
    }

    fn parse_parameter(&mut self) -> Arc<Node> {
        let pos = self.token_pos();

        let modifiers = self.parse_parameter_modifiers();

        let dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);

        let name = self.parse_identifier_or_pattern_with_diagnostic(Some(
            &diagnostics::PRIVATE_IDENTIFIERS_CANNOT_BE_USED_AS_PARAMETERS,
        ));
        let question_token = self.parse_optional_token(SyntaxKind::QuestionToken);
        let type_node = self.parse_optional_type_annotation();
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();
            Some(self.parse_assignment_expression())
        } else {
            None
        };
        let end = initializer.as_ref().map_or_else(
            || type_node.as_ref().map_or(name.end(), |t| t.end()),
            |i| i.end(),
        );
        Arc::new(Node::with_loc(
            SyntaxKind::Parameter,
            NodeData::ParameterDeclaration(ParameterDeclarationData {
                modifiers,
                dot_dot_dot_token,
                name,
                question_token,
                type_node,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_heritage_clauses(&mut self) -> Option<Arc<NodeList>> {
        let mut clauses = Vec::new();
        let mut pos = self.token_pos();
        if self.parse_optional(SyntaxKind::ExtendsKeyword) {
            let types = self.parse_delimited_list(
                ParsingContext::HeritageClauseElement,
                Parser::parse_heritage_clause_element,
            );
            let end = self.token_pos();
            clauses.push(Arc::new(Node::with_loc(
                SyntaxKind::HeritageClause,
                NodeData::HeritageClause(HeritageClauseData {
                    token: SyntaxKind::ExtendsKeyword,
                    types: Arc::new(types),
                }),
                TextRange::new(pos, end),
            )));
        }
        if self.token == SyntaxKind::ImplementsKeyword {
            pos = self.token_pos();
            self.next_token();
            let types = self.parse_delimited_list(
                ParsingContext::HeritageClauseElement,
                Parser::parse_heritage_clause_element,
            );
            let end = self.token_pos();
            clauses.push(Arc::new(Node::with_loc(
                SyntaxKind::HeritageClause,
                NodeData::HeritageClause(HeritageClauseData {
                    token: SyntaxKind::ImplementsKeyword,
                    types: Arc::new(types),
                }),
                TextRange::new(pos, end),
            )));
        }
        if clauses.is_empty() {
            None
        } else {
            let end = clauses.last().unwrap().end();
            Some(Arc::new(NodeList {
                loc: TextRange::new(clauses[0].pos(), end),
                nodes: clauses,
            }))
        }
    }

    fn parse_heritage_clause_element(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let expression = self.parse_left_hand_side_expression();
        let type_arguments = self.parse_optional_type_arguments();
        let end = type_arguments
            .as_ref()
            .map_or(expression.end(), |ta| ta.end());
        Arc::new(Node::with_loc(
            SyntaxKind::ExpressionWithTypeArguments,
            NodeData::ExpressionWithTypeArguments(ExpressionWithTypeArgumentsData {
                expression,
                type_arguments,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_type_member(&mut self) -> Arc<Node> {
        if self.token == SyntaxKind::OpenParenToken || self.token == SyntaxKind::LessThanToken {
            return self.parse_signature_member(SyntaxKind::CallSignature);
        }

        if self.token == SyntaxKind::NewKeyword && {
            let mut s = self.scanner.clone();
            let t = s.scan();
            t == SyntaxKind::OpenParenToken || t == SyntaxKind::LessThanToken
        } {
            return self.parse_signature_member(SyntaxKind::ConstructSignature);
        }

        let pos = self.token_pos();
        let modifiers = self.parse_type_member_modifiers();

        if self.token == SyntaxKind::GetKeyword || self.token == SyntaxKind::SetKeyword {
            let mut s = self.scanner.clone();
            s.scan();
            if Self::token_can_follow_get_or_set(s.token()) {
                let accessor_kind = self.token;
                return self.parse_accessor_declaration(pos, modifiers, accessor_kind);
            }
        }
        if self.is_index_signature_start() {
            return self.parse_index_signature(pos, modifiers);
        }

        let name = self.parse_property_name();
        let postfix_token = self.parse_optional_token(SyntaxKind::QuestionToken);
        let type_parameters = self.parse_optional_type_parameters();
        if self.token == SyntaxKind::OpenParenToken {
            let parameters = self.parse_parameter_list();
            let type_node = self.parse_optional_return_type();
            self.parse_type_member_semicolon();
            let end = self.token_pos();
            return Arc::new(Node::with_loc(
                SyntaxKind::MethodSignature,
                NodeData::MethodSignatureDeclaration(MethodSignatureDeclarationData {
                    modifiers,
                    name,
                    postfix_token,
                    type_parameters,
                    parameters,
                    type_node,
                }),
                TextRange::new(pos, end),
            ));
        }

        let type_node = self
            .parse_optional_type_annotation()
            .unwrap_or_else(|| self.missing_node(self.token_pos()));
        let initializer = if self.parse_optional(SyntaxKind::EqualsToken) {
            self.parse_type()
        } else {
            self.missing_node(self.token_pos())
        };
        self.parse_type_member_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::PropertySignature,
            NodeData::PropertySignatureDeclaration(PropertySignatureDeclarationData {
                modifiers,
                name,
                postfix_token,
                type_node,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_type_member_modifiers(&mut self) -> Option<Arc<ModifierList>> {
        let mut modifiers = Vec::new();

        while matches!(
            self.token,
            SyntaxKind::ReadonlyKeyword
                | SyntaxKind::StaticKeyword
                | SyntaxKind::PublicKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::ProtectedKeyword
        ) {
            let mut s = self.scanner.clone();
            s.scan();
            if s.has_preceding_line_break() || !Self::token_can_follow_modifier(s.token()) {
                break;
            }
            let kind = self.token;
            let pos = self.token_pos();
            let end = self.token_end();
            self.next_token();
            modifiers.push((kind, pos, end));
        }
        if modifiers.is_empty() {
            None
        } else {
            Some(self.make_modifier_list(modifiers))
        }
    }

    fn token_can_follow_modifier(token: SyntaxKind) -> bool {
        token == SyntaxKind::OpenBracketToken
            || token == SyntaxKind::OpenBraceToken
            || token == SyntaxKind::AsteriskToken
            || token == SyntaxKind::DotDotDotToken
            || is_identifier_or_keyword(token)
            || token == SyntaxKind::StringLiteral
            || token == SyntaxKind::NumericLiteral
            || token == SyntaxKind::BigIntLiteral
    }

    fn token_can_follow_get_or_set(token: SyntaxKind) -> bool {
        token == SyntaxKind::OpenBracketToken
            || is_identifier_or_keyword(token)
            || token == SyntaxKind::StringLiteral
            || token == SyntaxKind::NumericLiteral
            || token == SyntaxKind::BigIntLiteral
    }

    fn can_follow_export_modifier(token: SyntaxKind) -> bool {
        token == SyntaxKind::AtToken
            || (token != SyntaxKind::AsteriskToken
                && token != SyntaxKind::AsKeyword
                && token != SyntaxKind::OpenBraceToken
                && Self::token_can_follow_modifier(token))
    }

    fn token_can_follow_default_keyword(
        t: SyntaxKind,
        s: &mut crate::scanner::Scanner,
    ) -> bool {
        match t {
            SyntaxKind::ClassKeyword
            | SyntaxKind::FunctionKeyword
            | SyntaxKind::InterfaceKeyword
            | SyntaxKind::AtToken => true,
            SyntaxKind::AbstractKeyword => {
                s.scan() == SyntaxKind::ClassKeyword && !s.has_preceding_line_break()
            }
            SyntaxKind::AsyncKeyword => {
                s.scan() == SyntaxKind::FunctionKeyword && !s.has_preceding_line_break()
            }
            _ => false,
        }
    }

    fn is_class_member_modifier(token: SyntaxKind) -> bool {
        matches!(
            token,
            SyntaxKind::PublicKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::ProtectedKeyword
                | SyntaxKind::ReadonlyKeyword
                | SyntaxKind::StaticKeyword
                | SyntaxKind::OverrideKeyword
                | SyntaxKind::AccessorKeyword
        )
    }

    fn look_ahead_class_member_start(&self) -> bool {
        if self.token == SyntaxKind::AtToken {
            return true;
        }
        let mut id_token = SyntaxKind::Unknown;
        let mut s = self.scanner.clone();
        let mut t = self.token;

        while is_modifier_kind(t) {
            id_token = t;
            if Self::is_class_member_modifier(id_token) {
                return true;
            }
            t = s.scan();
        }
        if t == SyntaxKind::AsteriskToken {
            return true;
        }

        if is_identifier_or_keyword(t)
            || t == SyntaxKind::PrivateIdentifier
            || t == SyntaxKind::StringLiteral
            || t == SyntaxKind::NumericLiteral
            || t == SyntaxKind::BigIntLiteral
        {
            id_token = t;
            t = s.scan();
        }

        if t == SyntaxKind::OpenBracketToken {
            return true;
        }

        if id_token != SyntaxKind::Unknown {

            if !is_keyword_kind(id_token)
                || id_token == SyntaxKind::SetKeyword
                || id_token == SyntaxKind::GetKeyword
            {
                return true;
            }

            match t {
                SyntaxKind::OpenParenToken
                | SyntaxKind::LessThanToken
                | SyntaxKind::ExclamationToken
                | SyntaxKind::ColonToken
                | SyntaxKind::EqualsToken
                | SyntaxKind::QuestionToken => return true,
                _ => {}
            }

            return t == SyntaxKind::SemicolonToken
                || t == SyntaxKind::CloseBraceToken
                || t == SyntaxKind::EndOfFile
                || s.has_preceding_line_break();
        }
        false
    }

    fn is_index_signature_start(&self) -> bool {

        if self.token != SyntaxKind::OpenBracketToken {
            return false;
        }
        let mut s = self.scanner.clone();
        s.scan();
        let t1 = s.token();

        if t1 == SyntaxKind::DotDotDotToken || t1 == SyntaxKind::CloseBracketToken {
            return true;
        }

        if is_modifier_kind(t1) {
            s.scan();
            return Self::token_is_identifier(&s);
        }

        if !Self::token_is_identifier(&s) {
            return false;
        }

        s.scan();

        let t2 = s.token();
        if t2 == SyntaxKind::ColonToken || t2 == SyntaxKind::CommaToken {
            return true;
        }

        if t2 != SyntaxKind::QuestionToken {
            return false;
        }
        s.scan();
        matches!(
            s.token(),
            SyntaxKind::ColonToken | SyntaxKind::CommaToken | SyntaxKind::CloseBracketToken
        )
    }

    fn token_is_identifier(scanner: &crate::scanner::Scanner) -> bool {
        let t = scanner.token();
        if t == SyntaxKind::Identifier {
            return true;
        }

        (t as i16) > (SyntaxKind::WithKeyword as i16)
    }

    fn parse_index_signature(
        &mut self,
        pos: usize,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let parameters = self.parse_bracketedList(
            ParsingContext::Parameters,
            Parser::parse_parameter,
            SyntaxKind::OpenBracketToken,
            SyntaxKind::CloseBracketToken,
        );
        let type_node = self
            .parse_optional_type_annotation()
            .unwrap_or_else(|| self.missing_node(self.token_pos()));
        self.parse_type_member_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::IndexSignature,
            NodeData::IndexSignatureDeclaration(IndexSignatureDeclarationData {
                modifiers,
                parameters: Arc::new(parameters),
                type_node,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_signature_member(&mut self, kind: SyntaxKind) -> Arc<Node> {
        let pos = self.token_pos();
        if kind == SyntaxKind::ConstructSignature {
            self.expect(SyntaxKind::NewKeyword);
        }
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        self.parse_type_member_semicolon();
        let end = self.token_pos();
        if kind == SyntaxKind::CallSignature {
            Arc::new(Node::with_loc(
                SyntaxKind::CallSignature,
                NodeData::CallSignatureDeclaration(CallSignatureDeclarationData {
                    type_parameters,
                    parameters,
                    type_node,
                }),
                TextRange::new(pos, end),
            ))
        } else {
            Arc::new(Node::with_loc(
                SyntaxKind::ConstructSignature,
                NodeData::ConstructSignatureDeclaration(ConstructSignatureDeclarationData {
                    type_parameters,
                    parameters,
                    type_node,
                }),
                TextRange::new(pos, end),
            ))
        }
    }

    fn parse_type_member_semicolon(&mut self) {
        if !self.parse_optional(SyntaxKind::SemicolonToken) {
            self.parse_optional(SyntaxKind::CommaToken);
        }
    }

    fn parse_class_members(&mut self) -> NodeList {
        let pos = self.token_pos();
        self.expect(SyntaxKind::OpenBraceToken);
        let members = self.parse_list(ParsingContext::ClassMembers, Parser::parse_class_member);
        self.expect(SyntaxKind::CloseBraceToken);
        let end = self.token_pos();
        NodeList {
            loc: TextRange::new(pos, end),
            nodes: members.nodes,
        }
    }

    fn parse_class_member(&mut self) -> Arc<Node> {
        let pos = self.token_pos();

        if self.token == SyntaxKind::SemicolonToken {
            let end = self.token_end();
            self.next_token();
            return Arc::new(Node::with_loc(
                SyntaxKind::SemicolonClassElement,
                NodeData::Token,
                TextRange::new(pos, end),
            ));
        }

        let mut decorators: Vec<Arc<Node>> = Vec::new();
        let mut modifiers: Vec<(SyntaxKind, usize, usize)> = Vec::new();
        loop {
            if self.token == SyntaxKind::AtToken {
                decorators.push(self.parse_decorator());
                continue;
            }
            if !matches!(
                self.token,
                SyntaxKind::PublicKeyword
                    | SyntaxKind::PrivateKeyword
                    | SyntaxKind::ProtectedKeyword
                    | SyntaxKind::StaticKeyword
                    | SyntaxKind::ReadonlyKeyword
                    | SyntaxKind::AbstractKeyword
                    | SyntaxKind::AsyncKeyword
                    | SyntaxKind::OverrideKeyword
                    | SyntaxKind::AccessorKeyword

                    | SyntaxKind::ConstKeyword

                    | SyntaxKind::ExportKeyword
                    | SyntaxKind::DefaultKeyword

                    | SyntaxKind::DeclareKeyword
            ) {
                break;
            }

            let mut s = self.scanner.clone();
            s.scan();
            if s.has_preceding_line_break() {
                break;
            }
            if self.token == SyntaxKind::StaticKeyword && s.token() == SyntaxKind::OpenBraceToken {
                break;
            }

            let can_follow = match self.token {
                SyntaxKind::ExportKeyword => {
                    let next = s.token();
                    match next {
                        SyntaxKind::DefaultKeyword => {
                            let t = s.scan();
                            Self::token_can_follow_default_keyword(t, &mut s)
                        }
                        SyntaxKind::TypeKeyword => {
                            let t = s.scan();
                            Self::can_follow_export_modifier(t)
                        }
                        _ => Self::can_follow_export_modifier(next),
                    }
                }
                SyntaxKind::DefaultKeyword => {
                    Self::token_can_follow_default_keyword(s.token(), &mut s)
                }
                _ => Self::token_can_follow_modifier(s.token()),
            };
            if !can_follow {
                break;
            }
            let kind = self.token;
            let mpos = self.token_pos();
            let mend = self.token_end();
            self.next_token();
            modifiers.push((kind, mpos, mend));
        }
        let modifiers = if decorators.is_empty() && modifiers.is_empty() {
            None
        } else if decorators.is_empty() {
            Some(self.make_modifier_list(modifiers))
        } else {
            Some(self.make_modifier_list_with_decorators(modifiers, decorators))
        };

        if self.token == SyntaxKind::StaticKeyword {

            let mut s = self.scanner.clone();
            s.scan();
            if !s.has_preceding_line_break() && s.token() == SyntaxKind::OpenBraceToken {
                let pos = self.token_pos();
                self.next_token();
                let body = self.parse_block();
                let end = body.end();
                return Arc::new(Node::with_loc(
                    SyntaxKind::ClassStaticBlockDeclaration,
                    NodeData::ClassStaticBlockDeclaration(ClassStaticBlockDeclarationData {
                        modifiers: None,
                        body,
                    }),
                    TextRange::new(pos, end),
                ));
            }
        }

        if self.is_index_signature_start() {
            return self.parse_index_signature(pos, modifiers);
        }

        if self.token == SyntaxKind::GetKeyword || self.token == SyntaxKind::SetKeyword {
            let mut s = self.scanner.clone();
            s.scan();
            let next = s.token();
            let is_accessor = Self::token_can_follow_get_or_set(next);
            if is_accessor {
                let accessor_kind = self.token;
                return self.parse_accessor_declaration(pos, modifiers, accessor_kind);
            }
        }

        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);
        let name = self.parse_property_name();
        let postfix_token = self
            .parse_optional_token(SyntaxKind::QuestionToken)
            .or_else(|| self.parse_optional_token(SyntaxKind::ExclamationToken));

        if self.token == SyntaxKind::OpenParenToken
            || self.token == SyntaxKind::LessThanToken
            || asterisk_token.is_some()
        {

            let is_constructor =
                name.kind == SyntaxKind::Identifier && name.text() == "constructor";
            let type_parameters = self.parse_optional_type_parameters();

            let prev_yield = self.yield_context;
            let prev_await = self.await_context;
            if asterisk_token.is_some() {
                self.yield_context = true;
            }

            let parameters = self.parse_parameter_list();
            let type_node = self.parse_optional_return_type();
            let body = if self.token == SyntaxKind::OpenBraceToken {
                Some(self.parse_block())
            } else {
                self.parse_semicolon();
                None
            };
            self.yield_context = prev_yield;
            self.await_context = prev_await;

            let end = body.as_ref().map_or(self.token_pos(), |b| b.end());
            if is_constructor {
                return Arc::new(Node::with_loc(
                    SyntaxKind::Constructor,
                    NodeData::ConstructorDeclaration(ConstructorDeclarationData {
                        modifiers,
                        type_parameters,
                        parameters,
                        type_node,
                        full_signature: None,
                        body,
                    }),
                    TextRange::new(pos, end),
                ));
            }
            return Arc::new(Node::with_loc(
                SyntaxKind::MethodDeclaration,
                NodeData::MethodDeclaration(MethodDeclarationData {
                    modifiers,
                    asterisk_token,
                    name,
                    postfix_token,
                    type_parameters,
                    parameters,
                    type_node,
                    full_signature: None,
                    body,
                }),
                TextRange::new(pos, end),
            ));
        }

        let type_node = self.parse_optional_type_annotation();
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();
            Some(self.parse_assignment_expression())
        } else {
            None
        };
        self.parse_semicolon_after_property_name(&name, type_node.as_ref(), initializer.as_ref());
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::PropertyDeclaration,
            NodeData::PropertyDeclaration(PropertyDeclarationData {
                modifiers,
                name,
                postfix_token,
                type_node,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_semicolon_after_property_name(
        &mut self,
        name: &Arc<Node>,
        type_node: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) {
        if self.token == SyntaxKind::AtToken && !self.has_preceding_line_break() {
            self.parse_error_at_current_token(
                diagnostics::DECORATORS_MUST_PRECEDE_THE_NAME_AND_ALL_KEYWORDS_OF_PROPERTY_DECLARATIONS,
                &[],
            );
            return;
        }
        if self.token == SyntaxKind::OpenParenToken {
            self.parse_error_at_current_token(
                diagnostics::CANNOT_START_A_FUNCTION_CALL_IN_A_TYPE_ANNOTATION,
                &[],
            );
            self.next_token();
            return;
        }
        if type_node.is_some() && !self.can_parse_semicolon() {
            if initializer.is_some() {
                self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[";"]);
            } else {
                self.parse_error_at_current_token(
                    diagnostics::EXPECTED_FOR_PROPERTY_INITIALIZER,
                    &[],
                );
            }
            return;
        }
        if self.try_parse_semicolon() {
            return;
        }
        if initializer.is_some() {
            self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[";"]);
            return;
        }
        self.parse_error_for_missing_semicolon_after(name);
    }

    fn parse_error_for_missing_semicolon_after(&mut self, node: &Arc<Node>) {

        let expression_text = if node.kind == SyntaxKind::Identifier {
            node.text().to_string()
        } else {
            String::new()
        };
        if expression_text.is_empty() {
            self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[";"]);
            return;
        }

        let pos = node.loc.pos();
        match expression_text.as_str() {
            "const" | "let" | "var" => {
                self.parse_error_at(
                    pos,
                    node.end(),
                    diagnostics::VARIABLE_DECLARATION_NOT_ALLOWED_AT_THIS_LOCATION,
                    &[],
                );
            }

            "declare" => {}
            "interface" => {
                self.parse_error_for_invalid_name(
                    diagnostics::INTERFACE_NAME_CANNOT_BE_0,
                    diagnostics::INTERFACE_MUST_BE_GIVEN_A_NAME,
                );
            }
            "is" => {
                self.parse_error_at(
                    pos,
                    self.token_pos(),
                    diagnostics::A_TYPE_PREDICATE_IS_ONLY_ALLOWED_IN_RETURN_TYPE_POSITION_FOR_FUNCTIONS_AND_METHODS,
                    &[],
                );
            }
            "module" | "namespace" => {
                self.parse_error_for_invalid_name(
                    diagnostics::NAMESPACE_NAME_CANNOT_BE_0,
                    diagnostics::NAMESPACE_MUST_BE_GIVEN_A_NAME,
                );
            }
            "type" => {
                self.parse_error_for_invalid_name(
                    diagnostics::TYPE_ALIAS_NAME_CANNOT_BE_0,
                    diagnostics::TYPE_ALIAS_MUST_BE_GIVEN_A_NAME,
                );
            }
            _ => {

                if self.token == SyntaxKind::Unknown {
                    return;
                }

                let expression_text = if node.kind == SyntaxKind::Identifier {
                    node.text().to_string()
                } else {
                    String::new()
                };
                let followed_by_identifier = {
                    let text = self.scanner.text();
                    let mut i = node.end();
                    let bytes = text.as_bytes();
                    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
                        i += 1;
                    }
                    i < bytes.len()
                        && (bytes[i].is_ascii_alphabetic()
                            || bytes[i] == b'_'
                            || bytes[i] == b'$')
                };
                if !expression_text.is_empty() && followed_by_identifier {
                    let lower = expression_text.to_ascii_lowercase();
                    let mut best: Option<(usize, String)> = None;
                    for kw in KEYWORD_SUGGESTIONS {
                        if kw.len() <= 2 {
                            continue;
                        }
                        let d = crate::checker::edit_distance(
                            &lower,
                            &kw.to_ascii_lowercase(),
                        );
                        let budget = (expression_text.len() as f64 * 0.4).floor() + 0.9;
                        if d as f64 > budget {
                            continue;
                        }
                        if best.as_ref().is_none_or(|(bd, _)| d < *bd) {
                            best = Some((d, kw.to_string()));
                        }
                    }

                    let space_sugg = best.is_none().then(|| {
                        KEYWORD_SUGGESTIONS
                            .iter()
                            .find(|kw| {
                                kw.len() > 2
                                    && expression_text.len() > kw.len() + 2
                                    && expression_text.starts_with(*kw)
                            })
                            .map(|kw| format!("{kw} {}", &expression_text[kw.len()..]))
                    })
                            .flatten();
                    if let Some((_, sugg)) = best {
                        self.parse_error_at(
                            pos,
                            node.end(),
                            diagnostics::UNKNOWN_KEYWORD_OR_IDENTIFIER_DID_YOU_MEAN_0,
                            &[&sugg],
                        );
                        return;
                    }
                    if let Some(sugg) = space_sugg {
                        self.parse_error_at(
                            pos,
                            node.end(),
                            diagnostics::UNKNOWN_KEYWORD_OR_IDENTIFIER_DID_YOU_MEAN_0,
                            &[&sugg],
                        );
                        return;
                    }
                }
                self.parse_error_at(
                    pos,
                    node.end(),
                    diagnostics::UNEXPECTED_KEYWORD_OR_IDENTIFIER,
                    &[],
                );
            }
        }
    }

    fn parse_error_for_invalid_name(
        &mut self,
        name_diagnostic: Message,
        blank_diagnostic: Message,
    ) {
        if self.token == SyntaxKind::OpenBraceToken {
            self.parse_error_at_current_token(blank_diagnostic, &[]);
        } else {
            let arg = self.scanner.token_text().to_string();
            self.parse_error_at_current_token(name_diagnostic, &[&arg]);
        }
    }

    fn parse_accessor_declaration(
        &mut self,
        pos: usize,
        modifiers: Option<Arc<ModifierList>>,
        accessor_kind: SyntaxKind,
    ) -> Arc<Node> {

        self.next_token();
        let name = self.parse_property_name();
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_return_type();
        let body = if self.token == SyntaxKind::OpenBraceToken {
            Some(self.parse_block())
        } else {
            self.parse_semicolon();
            None
        };

        let end = body
            .as_ref()
            .map_or(self.scanner.full_start_pos(), |b| b.end());
        let range = TextRange::new(pos, end);
        match accessor_kind {
            SyntaxKind::GetKeyword => Arc::new(Node::with_loc(
                SyntaxKind::GetAccessor,
                NodeData::GetAccessorDeclaration(GetAccessorDeclarationData {
                    modifiers,
                    name,
                    type_parameters,
                    parameters,
                    type_node,
                    full_signature: None,
                    body,
                }),
                range,
            )),
            _ => Arc::new(Node::with_loc(
                SyntaxKind::SetAccessor,
                NodeData::SetAccessorDeclaration(SetAccessorDeclarationData {
                    modifiers,
                    name,
                    type_parameters,
                    parameters,
                    type_node,
                    full_signature: None,
                    body,
                }),
                range,
            )),
        }
    }

    pub fn diagnostics(&self) -> &[ParserDiagnostic] {
        &self.diagnostics
    }
}

fn binary_precedence(token: SyntaxKind) -> u8 {
    match token {
        SyntaxKind::BarBarToken | SyntaxKind::QuestionQuestionToken => 1,
        SyntaxKind::AmpersandAmpersandToken => 2,
        SyntaxKind::BarToken => 3,
        SyntaxKind::CaretToken => 4,
        SyntaxKind::AmpersandToken => 5,
        SyntaxKind::EqualsEqualsToken
        | SyntaxKind::ExclamationEqualsToken
        | SyntaxKind::EqualsEqualsEqualsToken
        | SyntaxKind::ExclamationEqualsEqualsToken => 6,
        SyntaxKind::LessThanToken
        | SyntaxKind::LessThanEqualsToken
        | SyntaxKind::GreaterThanToken
        | SyntaxKind::GreaterThanEqualsToken
        | SyntaxKind::InstanceOfKeyword
        | SyntaxKind::InKeyword => 7,
        SyntaxKind::LessThanLessThanToken
        | SyntaxKind::GreaterThanGreaterThanToken
        | SyntaxKind::GreaterThanGreaterThanGreaterThanToken => 8,
        SyntaxKind::PlusToken | SyntaxKind::MinusToken => 9,
        SyntaxKind::AsteriskToken | SyntaxKind::SlashToken | SyntaxKind::PercentToken => 10,
        SyntaxKind::AsteriskAsteriskToken => 11,
        _ => 0,
    }
}

fn is_assignment_operator(token: SyntaxKind) -> bool {
    matches!(
        token,
        SyntaxKind::EqualsToken
            | SyntaxKind::PlusEqualsToken
            | SyntaxKind::MinusEqualsToken
            | SyntaxKind::AsteriskEqualsToken
            | SyntaxKind::AsteriskAsteriskEqualsToken
            | SyntaxKind::SlashEqualsToken
            | SyntaxKind::PercentEqualsToken
            | SyntaxKind::LessThanLessThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken
            | SyntaxKind::AmpersandEqualsToken
            | SyntaxKind::BarEqualsToken
            | SyntaxKind::CaretEqualsToken
            | SyntaxKind::BarBarEqualsToken
            | SyntaxKind::AmpersandAmpersandEqualsToken
            | SyntaxKind::QuestionQuestionEqualsToken
    )
}

fn is_keyword(token: SyntaxKind) -> bool {
    crate::ast::node_data_generated::is_keyword_kind(token)
}

fn is_identifier_or_keyword(token: SyntaxKind) -> bool {

    token == SyntaxKind::Identifier
        || token == SyntaxKind::PrivateIdentifier
        || is_keyword(token)
}

fn is_reserved_word_kind(token: SyntaxKind) -> bool {
    matches!(
        token,
        SyntaxKind::BreakKeyword
            | SyntaxKind::CaseKeyword
            | SyntaxKind::CatchKeyword
            | SyntaxKind::ClassKeyword
            | SyntaxKind::ConstKeyword
            | SyntaxKind::ContinueKeyword
            | SyntaxKind::DebuggerKeyword
            | SyntaxKind::DefaultKeyword
            | SyntaxKind::DeleteKeyword
            | SyntaxKind::DoKeyword
            | SyntaxKind::ElseKeyword
            | SyntaxKind::EnumKeyword
            | SyntaxKind::ExportKeyword
            | SyntaxKind::ExtendsKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::FinallyKeyword
            | SyntaxKind::ForKeyword
            | SyntaxKind::FunctionKeyword
            | SyntaxKind::IfKeyword
            | SyntaxKind::ImportKeyword
            | SyntaxKind::InKeyword
            | SyntaxKind::InstanceOfKeyword
            | SyntaxKind::NewKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::ReturnKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::SwitchKeyword
            | SyntaxKind::ThisKeyword
            | SyntaxKind::ThrowKeyword
            | SyntaxKind::TrueKeyword
            | SyntaxKind::TryKeyword
            | SyntaxKind::TypeOfKeyword
            | SyntaxKind::VarKeyword
            | SyntaxKind::VoidKeyword
            | SyntaxKind::WhileKeyword
            | SyntaxKind::WithKeyword
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_identifier() {
        let mut p = Parser::new("foo");
        let node = p.parse_expression();
        assert_eq!(node.kind, SyntaxKind::Identifier);
        assert_eq!(node.text(), "foo");
        assert!(p.diagnostics().is_empty());
    }

    #[test]
    fn parse_numeric_literal() {
        let mut p = Parser::new("42");
        let node = p.parse_expression();
        assert_eq!(node.kind, SyntaxKind::NumericLiteral);
        assert_eq!(node.text(), "42");
    }

    #[test]
    fn parse_string_literal() {
        let mut p = Parser::new("\"hello\"");
        let node = p.parse_expression();
        assert_eq!(node.kind, SyntaxKind::StringLiteral);
    }

    #[test]
    fn parse_private_identifier_class_field() {

        let (_, diags) = Parser::parse_source_file_text_with_diagnostics(
            "a.ts",
            "class C { #name: string; }".to_string(),
        );
        assert!(
            diags.is_empty(),
            "expected no diagnostics, got: {:?}",
            diags
        );
    }

    #[test]
    fn parse_private_identifier_member_access() {

        let mut p = Parser::new("this.#name");
        let node = p.parse_expression();
        assert_eq!(node.kind, SyntaxKind::PropertyAccessExpression);
        assert!(
            p.diagnostics().is_empty(),
            "expected no diagnostics, got: {:?}",
            p.diagnostics()
        );
    }

    #[test]
    fn parse_less_than_is_comparison_not_type_args() {

        let mut p = Parser::new("if (x < 10) { }");
        let _ = p.parse_expression();
        assert!(
            p.diagnostics().iter().all(|d| {
                let msg = format!("{}", d.message);
                !msg.contains("expected")
            }),
            "expected no 'expected' diagnostics, got: {:?}",
            p.diagnostics()
        );
    }

    #[test]
    fn parse_generic_call_keeps_type_arguments() {

        let mut p = Parser::new("f<string>(x)");
        let node = p.parse_expression();
        assert_eq!(node.kind, SyntaxKind::CallExpression);
        assert!(
            p.diagnostics().is_empty(),
            "expected no diagnostics, got: {:?}",
            p.diagnostics()
        );
    }

    #[test]
    fn parse_generic_arrow_function() {

        let mut p = Parser::new("<T>(x: T): T => x");
        let node = p.parse_expression();
        assert_eq!(node.kind, SyntaxKind::ArrowFunction);
        assert!(
            p.diagnostics().is_empty(),
            "expected no diagnostics, got: {:?}",
            p.diagnostics()
        );
    }

    #[test]
    fn parse_async_generic_arrow_function() {

        let mut p = Parser::new("async <T>(value: T): T => value");
        let node = p.parse_expression();
        assert_eq!(node.kind, SyntaxKind::ArrowFunction);
        assert!(
            p.diagnostics().is_empty(),
            "expected no diagnostics, got: {:?}",
            p.diagnostics()
        );
    }

    #[test]
    fn parse_generic_arrow_not_confused_with_comparison() {

        let mut p = Parser::new("let r = a < b;");
        let _ = p.parse_expression();
        assert!(
            p.diagnostics().iter().all(|d| {
                let msg = format!("{}", d.message);
                !msg.contains("expected")
            }),
            "expected no 'expected' diagnostics, got: {:?}",
            p.diagnostics()
        );
    }

    #[test]
    fn parse_for_loop_condition_less_than() {

        let (_, diags) = Parser::parse_source_file_text_with_diagnostics(
            "a.ts",
            "function f() { for (let i = 0; i < n; i++) { } }".to_string(),
        );
        assert!(
            diags.iter().all(|d| {
                let msg = format!("{}", d.message);
                !msg.contains("expected")
            }),
            "expected no 'expected' diagnostics, got: {:?}",
            diags
        );
    }

    #[test]
    fn parse_multi_declarator_variable_list() {

        let (_, diags) = Parser::parse_source_file_text_with_diagnostics(
            "a.ts",
            "let a = 1, b = 2, c = 3;\na; b; c;".to_string(),
        );
        assert!(
            diags.iter().all(|d| {
                let msg = format!("{}", d.message);
                !msg.contains("expected")
            }),
            "expected no parse errors, got: {:?}",
            diags
        );
    }

    #[test]
    fn parse_parenthesized() {
        let mut p = Parser::new("(foo)");
        let node = p.parse_expression();
        assert_eq!(node.kind, SyntaxKind::ParenthesizedExpression);
        assert!(p.diagnostics().is_empty());
    }

    #[test]
    fn parse_unary() {
        let mut p = Parser::new("!foo");
        let node = p.parse_expression();
        assert_eq!(node.kind, SyntaxKind::PrefixUnaryExpression);
    }

    #[test]
    fn parse_binary_precedence() {
        let mut p = Parser::new("a + b * c");
        let node = p.parse_expression();
        assert_eq!(node.kind, SyntaxKind::BinaryExpression);
    }

    #[test]
    fn parse_var_statement() {
        let mut p = Parser::new("var x = 1;");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::VariableStatement);
    }

    #[test]
    fn parse_let_statement() {
        let mut p = Parser::new("let x: number = 42;");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::VariableStatement);
    }

    #[test]
    fn parse_declare_variable_statement() {
        let mut p = Parser::new("declare var x: string;");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::VariableStatement);
        assert!(p.diagnostics().is_empty());
    }

    #[test]
    fn parse_declare_function_statement() {
        let mut p = Parser::new("declare function f(): void;");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::FunctionDeclaration);
        assert!(p.diagnostics().is_empty());
    }

    #[test]
    fn parse_declare_type_alias_statement() {
        let mut p = Parser::new("declare type Name = string;");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::TypeAliasDeclaration);
        assert!(p.diagnostics().is_empty());
    }

    #[test]
    fn parse_export_declare_interface_statement() {
        let mut p = Parser::new("export declare interface Box { value: string; }");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::InterfaceDeclaration);
        assert!(p.diagnostics().is_empty());
    }

    #[test]
    fn parse_if_statement() {
        let mut p = Parser::new("if (x) { y; }");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::IfStatement);
    }

    #[test]
    fn parse_if_else_statement() {
        let mut p = Parser::new("if (x) { y; } else { z; }");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::IfStatement);
    }

    #[test]
    fn parse_return_statement() {
        let mut p = Parser::new("return 42;");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::ReturnStatement);
    }

    #[test]
    fn parse_return_void() {
        let mut p = Parser::new("return;");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::ReturnStatement);
    }

    #[test]
    fn parse_while_statement() {
        let mut p = Parser::new("while (true) { x; }");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::WhileStatement);
    }

    #[test]
    fn parse_for_statement() {
        let mut p = Parser::new("for (let i = 0; i < 10; i++) { x; }");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::ForStatement);
    }

    #[test]
    fn parse_break_statement() {
        let mut p = Parser::new("break;");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::BreakStatement);
    }

    #[test]
    fn parse_continue_statement() {
        let mut p = Parser::new("continue;");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::ContinueStatement);
    }

    #[test]
    fn parse_throw_statement() {
        let mut p = Parser::new("throw new Error();");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::ThrowStatement);
    }

    #[test]
    fn parse_block() {
        let mut p = Parser::new("{ x; y; }");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::Block);
    }

    #[test]
    fn parse_empty_statement() {
        let mut p = Parser::new(";");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::EmptyStatement);
    }

    #[test]
    fn parse_debugger_statement() {
        let mut p = Parser::new("debugger;");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::DebuggerStatement);
    }

    #[test]
    fn parse_member_access() {
        let mut p = Parser::new("a.b.c");
        let node = p.parse_expression();
        assert_eq!(node.kind, SyntaxKind::PropertyAccessExpression);
    }

    #[test]
    fn parse_call_expression() {
        let mut p = Parser::new("foo(1, 2)");
        let node = p.parse_expression();
        assert_eq!(node.kind, SyntaxKind::CallExpression);
    }

    #[test]
    fn parse_array_literal() {
        let mut p = Parser::new("[1, 2, 3]");
        let node = p.parse_expression();
        assert_eq!(node.kind, SyntaxKind::ArrayLiteralExpression);
    }

    #[test]
    fn parse_object_literal() {
        let mut p = Parser::new("{ a: 1, b: 2 }");
        let node = p.parse_expression();
        assert_eq!(node.kind, SyntaxKind::ObjectLiteralExpression);
    }

    #[test]
    fn parse_assignment_expression() {
        let mut p = Parser::new("x = 42");
        let node = p.parse_expression();
        assert_eq!(node.kind, SyntaxKind::BinaryExpression);
    }

    #[test]
    fn script_kind_from_file_name_matches_go_mapping() {
        assert_eq!(script_kind_from_file_name("a.ts"), ScriptKind::Ts);
        assert_eq!(script_kind_from_file_name("a.mts"), ScriptKind::Ts);
        assert_eq!(script_kind_from_file_name("a.cts"), ScriptKind::Ts);
        assert_eq!(script_kind_from_file_name("a.tsx"), ScriptKind::Tsx);
        assert_eq!(script_kind_from_file_name("a.js"), ScriptKind::Js);
        assert_eq!(script_kind_from_file_name("a.mjs"), ScriptKind::Js);
        assert_eq!(script_kind_from_file_name("a.cjs"), ScriptKind::Js);
        assert_eq!(script_kind_from_file_name("a.jsx"), ScriptKind::Jsx);
        assert_eq!(script_kind_from_file_name("a.json"), ScriptKind::Json);
        assert_eq!(script_kind_from_file_name("a.txt"), ScriptKind::Unknown);

        let tsx = Parser::parse_source_file_text("a.tsx", "const x = <div />;".to_string());
        assert_eq!(tsx.script_kind, ScriptKind::Tsx);
        assert_eq!(tsx.language_variant, LanguageVariant::Jsx);

        let jsx = Parser::parse_source_file_text("a.jsx", "const x = <div />;".to_string());
        assert_eq!(jsx.script_kind, ScriptKind::Jsx);
        assert_eq!(jsx.language_variant, LanguageVariant::Jsx);
    }

    #[test]
    fn namespace_import_is_wrapped_in_import_clause() {
        let mut p = Parser::new("import * as ns from \"mod\";");
        let node = p.parse_statement();
        let import = match &node.data {
            NodeData::ImportDeclaration(data) => data,
            other => panic!("expected import declaration, got {other:?}"),
        };
        let clause = import
            .import_clause
            .as_ref()
            .expect("missing import clause");
        let clause_data = match &clause.data {
            NodeData::ImportClause(data) => data,
            other => panic!("expected import clause, got {other:?}"),
        };
        assert!(clause_data.name.is_none());
        let named_bindings = clause_data
            .named_bindings
            .as_ref()
            .expect("missing namespace import");
        assert_eq!(named_bindings.kind, SyntaxKind::NamespaceImport);
    }

    fn first_import_specifier(source: &str) -> (bool, Option<String>, String) {
        let (_file, diags) =
            Parser::parse_source_file_text_with_diagnostics("a.ts", source.to_string());
        assert!(diags.is_empty(), "{source}: {diags:?}");
        let file = &_file;
        let stmt = match &file.node.data {
            NodeData::SourceFile(d) => d.statements.nodes[0].clone(),
            other => panic!("expected source file, got {other:?}"),
        };
        let import = match &stmt.data {
            NodeData::ImportDeclaration(d) => d,
            other => panic!("expected import declaration, got {other:?}"),
        };
        let clause = match &import.import_clause.as_ref().unwrap().data {
            NodeData::ImportClause(d) => d,
            other => panic!("expected import clause, got {other:?}"),
        };
        let named = match &clause.named_bindings.as_ref().unwrap().data {
            NodeData::NamedImports(d) => d,
            other => panic!("expected named imports, got {other:?}"),
        };
        match &named.elements.nodes[0].data {
            NodeData::ImportSpecifier(d) => (
                d.is_type_only,
                d.property_name.as_ref().map(|p| p.text().to_string()),
                d.name.text().to_string(),
            ),
            other => panic!("expected import specifier, got {other:?}"),
        }
    }

    #[test]
    fn specifier_bare_type_is_the_name() {
        assert_eq!(
            first_import_specifier("import { type } from \"mod\";"),
            (false, None, "type".to_string())
        );
        let (_file, diags) = Parser::parse_source_file_text_with_diagnostics(
            "a.ts",
            "export { type };\nexport {};\n".to_string(),
        );
        assert!(diags.is_empty(), "{diags:?}");
    }

    #[test]
    fn specifier_type_as_shapes() {

        assert_eq!(
            first_import_specifier("import { type as } from \"mod\";"),
            (true, None, "as".to_string())
        );

        assert_eq!(
            first_import_specifier("import { type as as } from \"mod\";"),
            (false, Some("type".to_string()), "as".to_string())
        );

        assert_eq!(
            first_import_specifier("import { type as as as } from \"mod\";"),
            (true, Some("as".to_string()), "as".to_string())
        );

        assert_eq!(
            first_import_specifier("import { type x } from \"mod\";"),
            (true, None, "x".to_string())
        );

        assert_eq!(
            first_import_specifier("import { type x as y } from \"mod\";"),
            (true, Some("x".to_string()), "y".to_string())
        );
    }

    #[test]
    fn import_type_named_imports_use_phase_modifier() {
        let mut p = Parser::new("import type { A, B as C } from \"mod\";");
        let node = p.parse_statement();
        assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
        let import = match &node.data {
            NodeData::ImportDeclaration(data) => data,
            other => panic!("expected import declaration, got {other:?}"),
        };
        let clause = import
            .import_clause
            .as_ref()
            .expect("missing import clause");
        let clause_data = match &clause.data {
            NodeData::ImportClause(data) => data,
            other => panic!("expected import clause, got {other:?}"),
        };
        assert_eq!(clause_data.phase_modifier, Some(SyntaxKind::TypeKeyword));
        assert!(clause_data.name.is_none());
        let named_bindings = clause_data
            .named_bindings
            .as_ref()
            .expect("missing named imports");
        assert_eq!(named_bindings.kind, SyntaxKind::NamedImports);
    }

    #[test]
    fn import_type_multiline_named_imports() {
        let source = "import type {\n  A,\n  B,\n} from \"mod\";";
        let (_file, diagnostics) =
            Parser::parse_source_file_text_with_diagnostics("a.ts", source.to_string());
        assert!(diagnostics.is_empty(), "{:?}", diagnostics);
    }

    #[test]
    fn import_default_named_type_is_not_phase_modifier() {
        let mut p = Parser::new("import type from \"mod\";");
        let node = p.parse_statement();
        assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
        let import = match &node.data {
            NodeData::ImportDeclaration(data) => data,
            other => panic!("expected import declaration, got {other:?}"),
        };
        let clause = import
            .import_clause
            .as_ref()
            .expect("missing import clause");
        let clause_data = match &clause.data {
            NodeData::ImportClause(data) => data,
            other => panic!("expected import clause, got {other:?}"),
        };
        assert_eq!(clause_data.phase_modifier, None);
        assert!(clause_data.name.is_some());
    }

    #[test]
    fn import_equals_declaration_matches_go_entry_split() {
        let mut p = Parser::new("import type A = B.C;");
        let node = p.parse_statement();
        assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
        let import_equals = match &node.data {
            NodeData::ImportEqualsDeclaration(data) => data,
            other => panic!("expected import equals declaration, got {other:?}"),
        };
        assert!(import_equals.is_type_only);
        assert_eq!(import_equals.name.text(), "A");
        assert_eq!(
            import_equals.module_reference.kind,
            SyntaxKind::QualifiedName
        );
    }

    #[test]
    fn record_warn4_import_blocks_from_ai_color_toner_parse() {
        let cases = [
            (
                "AiModelField.tsx",
                "import type { AiProfile } from './types'\n",
            ),
            (
                "AiTestControls.tsx",
                "import type {\n  AiProfile,\n  AiProfileOperation,\n  AiTestMode,\n  AiTestResult,\n} from './types'\n",
            ),
            (
                "App.tsx",
                "import { ColorFloatPanel, ModalLayer, TokenFloatPanel } from './AppPanels'\n\
                 import { ComposerPage } from './ComposerPage'\n\
                 import { PalettePage } from './PalettePage'\n\
                 import { SettingsPage } from './SettingsPage'\n\
                 import { ThemesPage } from './ThemesPage'\n\
                 import { useAppController } from './useAppController'\n\
                 import './App.css'\n\
                 import type { Page } from './appTypes'\n",
            ),
            (
                "previewGeneration.ts",
                "import { findToken } from './colorRefs'\n\
                 import { generateAiText } from './aiAdapters'\n\
                 import { canActivateProfile } from './aiProfiles'\n\
                 import { extractAndValidatePreviewHtml } from './previewValidation'\n\
                 import type { AiProfile, AppState, Theme } from './types'\n",
            ),
        ];

        for (file_name, source) in cases {
            let (_file, diagnostics) =
                Parser::parse_source_file_text_with_diagnostics(file_name, source.to_string());
            assert!(
                diagnostics.is_empty(),
                "{file_name} produced diagnostics: {diagnostics:?}"
            );
        }
    }

    #[test]
    fn record_warn6_arrow_and_as_const_fragments_parse() {
        let cases = [
            (
                "AiModelField.test.tsx",
                "describe('AiModelField', () => {\n  it('renders', () => {\n    const profile = {\n      model: 'manual-model',\n      models: [\n        { id: 'model-a', name: 'Model A' },\n        { id: 'model-b', name: 'Model B' },\n      ],\n    }\n  })\n})\n",
            ),
            (
                "AiTestControls.test.tsx",
                "describe('AiTestControls', () => {\n  it('renders', () => {\n    const profile = {\n      lastQuickTest: {\n        status: 'success' as const,\n        latencyMs: 91,\n      },\n    }\n  })\n})\n",
            ),
            (
                "previewGeneration.ts",
                "export function builtinTemplate(state: AppState) {\n  const legend = state.tokenNames\n    .map(\n      (name) => `<div style=\"color:${muted}\"><span style=\"background:var(--${name})\"></span>--${name}</div>`,\n    )\n    .join('')\n\n  return `<body style=\"background:${bg};color:${text}\">\n    ${['Design', 'Build', 'Verify']\n      .map(\n        (title) => `<div>${title}</div>`,\n      )\n      .join('')}\n  </body>`\n}\n",
            ),
            (
                "previewGeneration.ts",
                "export async function aiGenerate(\n  state: AppState,\n  intent: string,\n  deps: PreviewGenerationDeps = {},\n) {\n  const data = await (deps.generateText || generateAiText)(\n    profile,\n    systemPrompt,\n    intent,\n  )\n  return data\n}\n",
            ),
        ];

        for (file_name, source) in cases {
            let (_file, diagnostics) =
                Parser::parse_source_file_text_with_diagnostics(file_name, source.to_string());
            assert!(
                diagnostics.is_empty(),
                "{file_name} produced diagnostics: {diagnostics:?}"
            );
        }
    }

    #[test]
    fn record_warn6_tsx_jsx_fragment_from_app_parse() {
        let source = "function App() {\n  const controller = useAppController()\n\n  return (\n    <div className=\"app-shell\" onMouseDown={() => controller.setFloatPanel(null)}>\n      <nav className=\"top-nav\" onMouseDown={(event) => event.stopPropagation()}>\n        <button\n          className=\"brand\"\n          type=\"button\"\n          onClick={() => controller.navigate('palette')}\n        >\n          <span>COLOR</span>\n          <span>TONER</span>\n        </button>\n        {NAV_ITEMS.map(([itemPage, label]) => (\n          <button\n            className={controller.page === itemPage ? 'active' : ''}\n            key={itemPage}\n            type=\"button\"\n            onClick={() => controller.navigate(itemPage)}\n          >\n            {label}\n          </button>\n        ))}\n      </nav>\n    </div>\n  )\n}\n";
        let (_file, diagnostics) =
            Parser::parse_source_file_text_with_diagnostics("App.tsx", source.to_string());
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn parse_jsx_simple_element() {
        let source = "const x = <div>hello</div>;";
        let (_file, diagnostics) =
            Parser::parse_source_file_text_with_diagnostics("test.tsx", source.to_string());
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn parse_jsx_fragment() {
        let source = "const x = <>fragment text</>;";
        let (_file, diagnostics) =
            Parser::parse_source_file_text_with_diagnostics("test.tsx", source.to_string());
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn parse_jsx_self_closing() {
        let source = "const x = <img src=\"foo.png\" alt=\"bar\" />;";
        let (_file, diagnostics) =
            Parser::parse_source_file_text_with_diagnostics("test.tsx", source.to_string());
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn parse_jsx_dashed_tag_name() {
        let source = "const x = <my-component data-foo=\"bar\">text</my-component>;";
        let (_file, diagnostics) =
            Parser::parse_source_file_text_with_diagnostics("test.tsx", source.to_string());
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn parse_jsx_expression_children() {
        let source = "const x = <div>{items.map(i => <span key={i}>{i}</span>)}</div>;";
        let (_file, diagnostics) =
            Parser::parse_source_file_text_with_diagnostics("test.tsx", source.to_string());
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn parse_jsx_nested_elements() {
        let source = "const x = <div><p><span>deep</span></p></div>;";
        let (_file, diagnostics) =
            Parser::parse_source_file_text_with_diagnostics("test.tsx", source.to_string());
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn parse_jsx_spread_attribute() {
        let source = "const x = <div {...props}>text</div>;";
        let (_file, diagnostics) =
            Parser::parse_source_file_text_with_diagnostics("test.tsx", source.to_string());
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn parse_jsx_member_expression_tag() {
        let source = "const x = <Foo.Bar>text</Foo.Bar>;";
        let (_file, diagnostics) =
            Parser::parse_source_file_text_with_diagnostics("test.tsx", source.to_string());
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn parse_primitive_keyword_type_nodes() {

        for (src, expected_kind) in [
            ("type T = any;", SyntaxKind::AnyKeyword),
            ("type T = unknown;", SyntaxKind::UnknownKeyword),
            ("type T = string;", SyntaxKind::StringKeyword),
            ("type T = number;", SyntaxKind::NumberKeyword),
            ("type T = bigint;", SyntaxKind::BigIntKeyword),
            ("type T = symbol;", SyntaxKind::SymbolKeyword),
            ("type T = boolean;", SyntaxKind::BooleanKeyword),
            ("type T = undefined;", SyntaxKind::UndefinedKeyword),
            ("type T = never;", SyntaxKind::NeverKeyword),
            ("type T = object;", SyntaxKind::ObjectKeyword),
            ("type T = void;", SyntaxKind::VoidKeyword),
        ] {
            let mut p = Parser::new(src);
            let node = p.parse_statement();
            assert_eq!(node.kind, SyntaxKind::TypeAliasDeclaration, "source: {src}");
            let alias = match &node.data {
                NodeData::TypeAliasDeclaration(data) => data,
                other => panic!("expected type alias, got {other:?} for {src}"),
            };
            assert_eq!(alias.type_node.kind, expected_kind, "source: {src}");
            assert!(
                matches!(alias.type_node.data, NodeData::KeywordTypeNode),
                "expected KeywordTypeNode for {src}"
            );
        }
    }

    #[test]
    fn parse_keyword_type_followed_by_dot_is_type_reference() {

        let mut p = Parser::new("type T = String.fromCharCode;");
        let node = p.parse_statement();
        let alias = match &node.data {
            NodeData::TypeAliasDeclaration(data) => data,
            other => panic!("expected type alias, got {other:?}"),
        };
        assert_eq!(alias.type_node.kind, SyntaxKind::TypeReference);
    }

    #[test]
    fn parse_typeof_type_query() {
        let mut p = Parser::new("type T = typeof foo;");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
        let alias = match &node.data {
            NodeData::TypeAliasDeclaration(data) => data,
            other => panic!("expected type alias, got {other:?}"),
        };
        assert_eq!(alias.type_node.kind, SyntaxKind::TypeQuery);
    }

    #[test]
    fn parse_import_type() {
        let mut p = Parser::new("type T = import(\"mod\").Foo;");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
        let alias = match &node.data {
            NodeData::TypeAliasDeclaration(data) => data,
            other => panic!("expected type alias, got {other:?}"),
        };
        assert_eq!(alias.type_node.kind, SyntaxKind::ImportType);
    }

    #[test]
    fn parse_import_type_with_attributes() {

        let mut p = Parser::new(
            "type T = import(\"pkg\", { with: { \"resolution-mode\": \"import\" } }).Foo;",
        );
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
        let alias = match &node.data {
            NodeData::TypeAliasDeclaration(data) => data,
            other => panic!("expected type alias, got {other:?}"),
        };
        assert_eq!(alias.type_node.kind, SyntaxKind::ImportType);
        let import_type = match &alias.type_node.data {
            NodeData::ImportTypeNode(data) => data,
            other => panic!("expected import type, got {other:?}"),
        };
        let attrs = import_type
            .attributes
            .as_ref()
            .expect("attributes clause present");
        match &attrs.data {
            NodeData::ImportAttributes(d) => {
                assert_eq!(d.token, SyntaxKind::WithKeyword);
                assert_eq!(d.attributes.len(), 1);
            }
            other => panic!("expected import attributes, got {other:?}"),
        }
    }

    #[test]
    fn parse_import_type_missing_with_reports_1005() {

        let mut p = Parser::new(
            "type T = import(\"pkg\", {\"resolution-mode\": \"require\"}).Foo;",
        );
        let node = p.parse_statement();
        let diags = p.diagnostics();
        assert!(
            diags.iter().any(|d| d.message.code == 1005),
            "expected TS1005 'with' expected: {diags:?}"
        );
        assert_eq!(node.kind, SyntaxKind::TypeAliasDeclaration);
    }

    #[test]
    fn parse_typeof_import_type() {
        let mut p = Parser::new("type T = typeof import(\"mod\").Foo;");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
        let alias = match &node.data {
            NodeData::TypeAliasDeclaration(data) => data,
            other => panic!("expected type alias, got {other:?}"),
        };
        assert_eq!(alias.type_node.kind, SyntaxKind::ImportType);
        let import_type = match &alias.type_node.data {
            NodeData::ImportTypeNode(data) => data,
            other => panic!("expected import type, got {other:?}"),
        };
        assert!(import_type.is_type_of);
    }

    #[test]
    fn parse_negative_literal_type() {
        let mut p = Parser::new("type T = -1;");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
        let alias = match &node.data {
            NodeData::TypeAliasDeclaration(data) => data,
            other => panic!("expected type alias, got {other:?}"),
        };
        assert_eq!(alias.type_node.kind, SyntaxKind::LiteralType);
    }

    #[test]
    fn parse_this_type() {
        let mut p = Parser::new("type T = this;");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
        let alias = match &node.data {
            NodeData::TypeAliasDeclaration(data) => data,
            other => panic!("expected type alias, got {other:?}"),
        };
        assert_eq!(alias.type_node.kind, SyntaxKind::ThisType);
    }

    #[test]
    fn parse_tuple_types() {

        let mut p = Parser::new("type T = [string, number];");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
        let alias = match &node.data {
            NodeData::TypeAliasDeclaration(data) => data,
            other => panic!("expected type alias, got {other:?}"),
        };
        assert_eq!(alias.type_node.kind, SyntaxKind::TupleType);

        let mut p = Parser::new("type T = readonly [string, number];");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
        let alias = match &node.data {
            NodeData::TypeAliasDeclaration(data) => data,
            other => panic!("expected type alias, got {other:?}"),
        };
        assert_eq!(alias.type_node.kind, SyntaxKind::TypeOperator);

        let mut p = Parser::new("type T = [string, ...number[]];");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

        let mut p = Parser::new("type T = [name: string, age: number];");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
        let alias = match &node.data {
            NodeData::TypeAliasDeclaration(data) => data,
            other => panic!("expected type alias, got {other:?}"),
        };
        assert_eq!(alias.type_node.kind, SyntaxKind::TupleType);

        let mut p = Parser::new("type T = [string?, number?];");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    }

    #[test]
    fn parse_union_intersection_precedence() {

        let mut p = Parser::new("type T = A | B & C;");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
        let alias = match &node.data {
            NodeData::TypeAliasDeclaration(data) => data,
            other => panic!("expected type alias, got {other:?}"),
        };

        assert_eq!(alias.type_node.kind, SyntaxKind::UnionType);
        let union = match &alias.type_node.data {
            NodeData::UnionTypeNode(d) => d,
            other => panic!("expected union, got {other:?}"),
        };
        assert_eq!(union.types.nodes.len(), 2);

        assert_eq!(union.types.nodes[1].kind, SyntaxKind::IntersectionType);

        let mut p = Parser::new("type T = | A | B;");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

        let mut p = Parser::new("type T = & A & B;");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    }

    #[test]
    fn parse_generic_type_params_and_references() {

        let mut p = Parser::new("type T<A, B extends string = \"x\"> = A | B;");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
        let alias = match &node.data {
            NodeData::TypeAliasDeclaration(data) => data,
            other => panic!("expected type alias, got {other:?}"),
        };
        assert!(alias.type_parameters.is_some());
        let tps = alias.type_parameters.as_ref().unwrap();
        assert_eq!(tps.nodes.len(), 2);
        assert_eq!(tps.nodes[0].kind, SyntaxKind::TypeParameter);
        assert_eq!(tps.nodes[1].kind, SyntaxKind::TypeParameter);

        let mut p = Parser::new("type T = Foo<string, number>;");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
        let alias = match &node.data {
            NodeData::TypeAliasDeclaration(data) => data,
            other => panic!("expected type alias, got {other:?}"),
        };
        assert_eq!(alias.type_node.kind, SyntaxKind::TypeReference);
        let tr = match &alias.type_node.data {
            NodeData::TypeReferenceNode(d) => d,
            other => panic!("expected type ref, got {other:?}"),
        };
        assert!(tr.type_arguments.is_some());
        assert_eq!(tr.type_arguments.as_ref().unwrap().nodes.len(), 2);

        let mut p = Parser::new("type T = A.B.C<T>;");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

        let mut p = Parser::new("type T = Map<string, Array<number>>;");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    }

    #[test]
    fn parse_mapped_types() {

        let mut p = Parser::new("type M<T> = { [K in keyof T]: string };");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
        let alias = match &node.data {
            NodeData::TypeAliasDeclaration(data) => data,
            other => panic!("expected type alias, got {other:?}"),
        };
        assert_eq!(alias.type_node.kind, SyntaxKind::MappedType);

        let mut p = Parser::new("type M<T> = { readonly [K in keyof T]: string };");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

        let mut p = Parser::new("type M<T> = { -readonly [K in keyof T]: string };");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

        let mut p = Parser::new("type M<T> = { [K in keyof T]-?: string };");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

        let mut p = Parser::new("type M<T> = { [K in keyof T as `${K}`]: string };");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    }

    #[test]
    fn parse_conditional_types() {

        let mut p = Parser::new("type R<T> = T extends string ? number : boolean;");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
        let alias = match &node.data {
            NodeData::TypeAliasDeclaration(data) => data,
            other => panic!("expected type alias, got {other:?}"),
        };
        assert_eq!(alias.type_node.kind, SyntaxKind::ConditionalType);

        let mut p = Parser::new("type R<T> = T extends A ? X : T extends B ? Y : Z;");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

        let mut p = Parser::new("type R<T> = T extends (infer U)[] ? U : never;");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    }

    #[test]
    fn parse_call_and_construct_signatures() {

        let mut p = Parser::new("type T = { (): string };");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

        let mut p = Parser::new("type T = { new (): Foo };");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

        let mut p = Parser::new("type T = { abstract new (): Foo };");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

        let mut p = Parser::new("type T = () => string;");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
        let alias = match &node.data {
            NodeData::TypeAliasDeclaration(data) => data,
            other => panic!("expected type alias, got {other:?}"),
        };
        assert_eq!(alias.type_node.kind, SyntaxKind::FunctionType);

        let mut p = Parser::new("type T = new () => Foo;");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
        let alias = match &node.data {
            NodeData::TypeAliasDeclaration(data) => data,
            other => panic!("expected type alias, got {other:?}"),
        };
        assert_eq!(alias.type_node.kind, SyntaxKind::ConstructorType);
    }

    #[test]
    fn parse_index_signatures() {

        let mut p = Parser::new("type T = { [key: string]: number };");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

        let mut p = Parser::new("type T = { [index: number]: string };");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

        let mut p = Parser::new("type T = { readonly [key: string]: number };");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    }

    #[test]
    fn parse_satisfies_and_as_const() {

        let mut p = Parser::new("const x = { a: 1 } as const;");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

        let mut p = Parser::new("const x = { a: 1 } satisfies Foo;");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

        let mut p = Parser::new("const x = foo!.bar;");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    }

    #[test]
    fn parse_declare_module_string_literal() {

        let mut p = Parser::new("declare module \"foo\";");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::ModuleDeclaration);
        let mod_decl = match &node.data {
            NodeData::ModuleDeclaration(d) => d,
            other => panic!("expected module decl, got {other:?}"),
        };
        assert_eq!(mod_decl.name.kind, SyntaxKind::StringLiteral);

        let mut p = Parser::new("declare module \"foo\" { export const x: number; }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::ModuleDeclaration);
        let mod_decl = match &node.data {
            NodeData::ModuleDeclaration(d) => d,
            other => panic!("expected module decl, got {other:?}"),
        };
        assert_eq!(mod_decl.name.kind, SyntaxKind::StringLiteral);
        assert!(mod_decl.body.is_some());
    }

    #[test]
    fn parse_declare_namespace_dotted() {

        let mut p = Parser::new("declare namespace A.B.C { export const x: number; }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::ModuleDeclaration);
    }

    #[test]
    fn parse_declare_global() {

        let mut p = Parser::new("declare global { const x: number; }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::ModuleDeclaration);
        let mod_decl = match &node.data {
            NodeData::ModuleDeclaration(d) => d,
            other => panic!("expected module decl, got {other:?}"),
        };
        assert_eq!(mod_decl.name.kind, SyntaxKind::Identifier);
        assert_eq!(mod_decl.name.text(), "global");
        assert!(mod_decl.body.is_some());
    }

    #[test]
    fn parse_declare_class_full_body() {

        let mut p =
            Parser::new("declare class C extends Base { constructor(x: number); foo(): void; }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::ClassDeclaration);
    }

    #[test]
    fn parse_declare_var_and_function() {

        let mut p = Parser::new("declare var x: number;");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::VariableStatement);

        let mut p = Parser::new("declare const y: string;");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::VariableStatement);

        let mut p = Parser::new("declare function f(): void;");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::FunctionDeclaration);
        let fn_decl = match &node.data {
            NodeData::FunctionDeclaration(d) => d,
            other => panic!("expected function decl, got {other:?}"),
        };
        assert!(
            fn_decl.body.is_none(),
            "declare function should have no body"
        );
    }

    #[test]
    fn parse_declare_enum_and_interface() {

        let mut p = Parser::new("declare enum E { A, B }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::EnumDeclaration);

        let mut p = Parser::new("declare interface I { foo(): void; }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::InterfaceDeclaration);

        let mut p = Parser::new("declare type T = string;");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::TypeAliasDeclaration);
    }

    #[test]
    fn parse_asi_basic() {

        let mut p = Parser::new("let x = 1");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::VariableStatement);

        let mut p = Parser::new("let x = 1\nlet y = 2");
        let s1 = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(s1.kind, SyntaxKind::VariableStatement);
        let s2 = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(s2.kind, SyntaxKind::VariableStatement);

        let mut p = Parser::new("let x = 1;\nlet y = 2;");
        let s1 = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        let s2 = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(s1.kind, SyntaxKind::VariableStatement);
        assert_eq!(s2.kind, SyntaxKind::VariableStatement);
    }

    #[test]
    fn parse_asi_postfix_no_line_break() {

        let mut p = Parser::new("let x = 1\n++y");
        let s1 = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(s1.kind, SyntaxKind::VariableStatement);

        let s2 = p.parse_statement();
        assert_eq!(s2.kind, SyntaxKind::ExpressionStatement);
    }

    #[test]
    fn parse_asi_throw_needs_expression() {

        let mut p = Parser::new("throw\nnew Error()");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::ThrowStatement);
        let throw = match &node.data {
            NodeData::ThrowStatement(d) => d,
            other => panic!("expected throw, got {other:?}"),
        };

        assert_eq!(throw.expression.kind, SyntaxKind::Identifier);
        let id = match &throw.expression.data {
            NodeData::Identifier(d) => d,
            other => panic!("expected identifier, got {other:?}"),
        };
        assert!(
            id.text.is_empty(),
            "expected missing identifier, got {:?}",
            id.text
        );
    }

    #[test]
    fn parse_scanner_errors_reach_parser_diagnostics() {

        let (file, diags) =
            Parser::parse_source_file_text_with_diagnostics("test.ts", "·".to_string());
        assert!(
            diags.iter().any(|d| d.message.code == 1127),
            "expected Invalid character diagnostic (TS1127), got: {diags:?}"
        );
        assert_eq!(file.node.kind, SyntaxKind::SourceFile);

        let (_file, diags) = Parser::parse_source_file_text_with_diagnostics(
            "test.ts",
            "\"unterminated".to_string(),
        );
        assert!(
            diags.iter().any(|d| d.message.code == 1002),
            "expected Unterminated string literal diagnostic (TS1002), got: {diags:?}"
        );
    }

    #[test]
    fn parse_regex_flag_diagnostics_reach_parser() {

        let (_file, diags) = Parser::parse_source_file_text_with_diagnostics(
            "test.ts",
            "let x = /foo/z;".to_string(),
        );
        assert!(
            diags.iter().any(|d| d.message.code == 1499),
            "expected TS1499 for unknown regex flag, got: {diags:?}"
        );

        let (_file, diags) = Parser::parse_source_file_text_with_diagnostics(
            "test.ts",
            "let x = /foo/gg;".to_string(),
        );
        assert!(
            diags.iter().any(|d| d.message.code == 1500),
            "expected TS1500 for duplicate regex flag, got: {diags:?}"
        );

        let (_file, diags) = Parser::parse_source_file_text_with_diagnostics(
            "test.ts",
            "let x = /foo/uv;".to_string(),
        );
        assert!(
            diags.iter().any(|d| d.message.code == 1502),
            "expected TS1502 for u+v flags, got: {diags:?}"
        );

        let (_file, diags) = Parser::parse_source_file_text_with_diagnostics(
            "test.ts",
            "let x = /foo/gim;".to_string(),
        );
        assert!(
            !diags
                .iter()
                .any(|d| matches!(d.message.code, 1499 | 1500 | 1501 | 1502)),
            "expected no regex flag diagnostics for valid flags, got: {diags:?}"
        );
    }

    #[test]
    fn parse_import_attributes_with() {

        let mut p = Parser::new(r#"import x from "y" with { type: "json" }"#);
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::ImportDeclaration);
        let imp = match &node.data {
            NodeData::ImportDeclaration(d) => d,
            other => panic!("expected import, got {other:?}"),
        };
        assert!(imp.attributes.is_some(), "expected import attributes");
        let attrs = imp.attributes.as_ref().unwrap();
        assert_eq!(attrs.kind, SyntaxKind::ImportAttributes);
        let attr_data = match &attrs.data {
            NodeData::ImportAttributes(d) => d,
            other => panic!("expected ImportAttributes, got {other:?}"),
        };
        assert_eq!(attr_data.token, SyntaxKind::WithKeyword);
        assert_eq!(attr_data.attributes.nodes.len(), 1);

        let mut p = Parser::new(r#"import { foo } from "y" with { type: "json", other: 42 }"#);
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::ImportDeclaration);
        let imp = match &node.data {
            NodeData::ImportDeclaration(d) => d,
            other => panic!("expected import, got {other:?}"),
        };
        let attrs = imp.attributes.as_ref().unwrap();
        let attr_data = match &attrs.data {
            NodeData::ImportAttributes(d) => d,
            other => panic!("expected ImportAttributes, got {other:?}"),
        };
        assert_eq!(attr_data.attributes.nodes.len(), 2);

        let mut p = Parser::new(r#"export { foo } from "y" with { type: "json" }"#);
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::ExportDeclaration);
        let exp = match &node.data {
            NodeData::ExportDeclaration(d) => d,
            other => panic!("expected export, got {other:?}"),
        };
        assert!(exp.attributes.is_some(), "expected export attributes");
    }

    #[test]
    fn parse_decorators() {

        let mut p = Parser::new("@decorator\nclass Foo {}");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::ClassDeclaration);
        let class = match &node.data {
            NodeData::ClassDeclaration(d) => d,
            other => panic!("expected class, got {other:?}"),
        };
        let mods = class
            .modifiers
            .as_ref()
            .expect("expected modifiers with decorator");
        assert!(mods.modifier_flags.contains(ModifierFlags::Decorator));
        let decorators: Vec<_> = mods
            .iter()
            .filter(|n| n.kind == SyntaxKind::Decorator)
            .collect();
        assert_eq!(decorators.len(), 1);

        let mut p = Parser::new("class Foo { @decorator bar() {} }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::ClassDeclaration);
        let class = match &node.data {
            NodeData::ClassDeclaration(d) => d,
            other => panic!("expected class, got {other:?}"),
        };
        let members = &class.members;
        assert_eq!(members.nodes.len(), 1);
        let method = &members.nodes[0];
        assert_eq!(method.kind, SyntaxKind::MethodDeclaration);
        let method_data = match &method.data {
            NodeData::MethodDeclaration(d) => d,
            other => panic!("expected method, got {other:?}"),
        };
        let mods = method_data
            .modifiers
            .as_ref()
            .expect("method should have decorator modifiers");
        assert!(mods.modifier_flags.contains(ModifierFlags::Decorator));

        let mut p = Parser::new("class Foo { @decorator x: number = 1; }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        let class = match &node.data {
            NodeData::ClassDeclaration(d) => d,
            other => panic!("expected class, got {other:?}"),
        };
        let prop = &class.members.nodes[0];
        assert_eq!(prop.kind, SyntaxKind::PropertyDeclaration);

        let mut p = Parser::new("@Dec({ option: true })\nclass Foo {}");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::ClassDeclaration);

        let mut p = Parser::new("@A @B\nclass Foo {}");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        let class = match &node.data {
            NodeData::ClassDeclaration(d) => d,
            other => panic!("expected class, got {other:?}"),
        };
        let mods = class.modifiers.as_ref().unwrap();
        let decorators: Vec<_> = mods
            .iter()
            .filter(|n| n.kind == SyntaxKind::Decorator)
            .collect();
        assert_eq!(decorators.len(), 2);

        let mut p = Parser::new("@Namespace.Dec\nclass Foo {}");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::ClassDeclaration);
    }

    #[test]
    fn parse_regex_literal() {

        let mut p = Parser::new("let x = /foo/g;");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::VariableStatement);

        let mut p = Parser::new("/foo/g.test(str);");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::ExpressionStatement);

        let mut p = Parser::new("function f() { return /pattern/; }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::FunctionDeclaration);

        let mut p = Parser::new(r"let x = /a\/b/;");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

        let mut p = Parser::new(r"let x = /[\/]/;");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

        let mut p = Parser::new("let x = a / b;");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    }

    #[test]
    fn parse_regex_in_call_expression() {
        let mut p = Parser::new("let r = str.replace(/foo/g, 'bar');");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::VariableStatement);
    }

    #[test]
    fn parse_comment_directives_propagate_to_source_file() {
        use crate::scanner::CommentDirectiveKind;
        let file = Parser::parse_source_file_text(
            "test.ts",
            "// @ts-ignore\nlet x = 1;\n// @ts-expect-error\n".to_string(),
        );
        assert_eq!(file.comment_directives.len(), 2);
        assert_eq!(
            file.comment_directives[0].kind,
            CommentDirectiveKind::Ignore
        );
        assert_eq!(
            file.comment_directives[1].kind,
            CommentDirectiveKind::ExpectError
        );
    }

    #[test]
    fn parse_using_declaration() {
        let mut p = Parser::new("using x = getResource();");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::VariableStatement);

        let stmt = match &node.data {
            NodeData::VariableStatement(d) => d,
            other => panic!("expected variable statement, got {other:?}"),
        };
        assert!(stmt.declaration_list.flags.contains(NodeFlags::Using));

        let mut p = Parser::new("using = 1;");
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::ExpressionStatement);

        let mut p = Parser::new("using\nx = 1;");
        let node = p.parse_statement();
        assert_ne!(node.kind, SyntaxKind::VariableStatement);
    }

    #[test]
    fn parse_await_using_declaration() {
        let mut p = Parser::new("await using x = getResource();");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::VariableStatement);
        let stmt = match &node.data {
            NodeData::VariableStatement(d) => d,
            other => panic!("expected variable statement, got {other:?}"),
        };
        assert!(stmt.declaration_list.flags.contains(NodeFlags::AwaitUsing));
    }

    #[test]
    fn parse_accessor_property() {
        let mut p = Parser::new("class C { accessor x = 1; }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::ClassDeclaration);
        let class = match &node.data {
            NodeData::ClassDeclaration(d) => d,
            other => panic!("expected class, got {other:?}"),
        };
        let prop = &class.members.nodes[0];
        assert_eq!(prop.kind, SyntaxKind::PropertyDeclaration);
        let prop_data = match &prop.data {
            NodeData::PropertyDeclaration(d) => d,
            other => panic!("expected property, got {other:?}"),
        };
        let mods = prop_data.modifiers.as_ref().expect("expected modifiers");
        assert!(mods.modifier_flags.contains(ModifierFlags::Accessor));
    }

    #[test]
    fn parse_type_predicate_in_function_type_return() {
        let mut p = Parser::new("type Predicate = (value: T) => value is S;");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::TypeAliasDeclaration);

        let mut p = Parser::new("type P = (value: T, index: number) => value is S;");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

        let mut p = Parser::new("type P = () => this is T;");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    }

    #[test]
    fn parse_type_predicate_in_method_return_type() {
        let mut p = Parser::new("interface I { isFoo(x: any): x is Foo; }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::InterfaceDeclaration);

        let mut p = Parser::new("function isFoo(x: any): x is Foo { return true; }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::FunctionDeclaration);

        let mut p = Parser::new("const isFoo = (x: any): x is Foo => true;");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    }

    #[test]
    fn parse_computed_property_name_in_type_member() {

        let mut p = Parser::new("interface I { [Symbol.iterator](): Iterator<T>; }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::InterfaceDeclaration);

        let mut p = Parser::new("interface Symbol { [Symbol.toPrimitive](hint: string): symbol; }");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

        let mut p = Parser::new("interface X { readonly [Symbol.toStringTag]: string; }");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

        let mut p = Parser::new("interface X { [key: string]: number; }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        let iface = match &node.data {
            NodeData::InterfaceDeclaration(d) => d,
            other => panic!("expected interface, got {other:?}"),
        };
        assert_eq!(iface.members.nodes[0].kind, SyntaxKind::IndexSignature);
    }

    #[test]
    fn parse_contextual_keyword_as_property_name_in_type_member() {

        let mut p = Parser::new("interface X { readonly static: boolean; }");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

        let mut p = Parser::new("interface X { readonly private: boolean; }");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

        let mut p =
            Parser::new("interface EcdhKeyDeriveParams extends Algorithm { public: CryptoKey; }");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

        let mut p = Parser::new("interface X { readonly x: boolean; }");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    }

    #[test]
    fn parse_heritage_clause_with_tuple_type_arguments() {

        let mut p = Parser::new("interface X extends Array<[number, number] | undefined> {}");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::InterfaceDeclaration);

        let mut p = Parser::new("interface X extends Foo<[number]> {}");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

        let mut p = Parser::new("interface X extends A, Foo<[number, number]> {}");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

        let mut p = Parser::new("class X extends Foo<[number, number]> {}");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    }

    #[test]
    fn parse_contextual_keyword_as_class_member_name() {

        let mut p = Parser::new("class C { static: number = 1; }");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

        let mut p = Parser::new("class C { public: number = 1; }");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

        let mut p = Parser::new("class C { readonly static: boolean; }");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

        let mut p = Parser::new("class C { static x: number = 1; }");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    }

    #[test]
    fn parse_const_enum() {
        let mut p = Parser::new("const enum E { A, B, C }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::EnumDeclaration);
    }

    #[test]
    fn parse_const_variable_not_treated_as_enum() {
        let mut p = Parser::new("const x = 1;");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::VariableStatement);
    }

    #[test]
    fn parse_abstract_class() {
        let mut p = Parser::new("abstract class Animal { abstract makeSound(): void; }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::ClassDeclaration);
    }

    #[test]
    fn parse_async_function() {
        let mut p = Parser::new("async function fetchData(): Promise<void> { return; }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::FunctionDeclaration);
    }

    #[test]
    fn parse_async_generator() {
        let mut p = Parser::new("async function* gen() { yield 1; }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::FunctionDeclaration);
    }

    #[test]
    fn parse_yield_in_generator() {
        let mut p = Parser::new("function* counter() { yield 1; yield* [2, 3]; }");
        let node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
        assert_eq!(node.kind, SyntaxKind::FunctionDeclaration);
    }

    #[test]
    fn parse_yield_await_in_async_generator() {
        let mut p = Parser::new("async function* gen() { yield await fetch('url'); }");
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    }

    #[test]
    fn parse_for_await_of() {
        let mut p = Parser::new(
            "async function process(stream) { for await (const chunk of stream) { console.log(chunk); } }",
        );
        let _node = p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    }

    #[test]
    fn parse_optional_chaining() {
        let mut p = Parser::new("const x = obj?.foo?.bar;");
        p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

        let mut p = Parser::new("const x = obj?.foo?.();");
        p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    }

    #[test]
    fn parse_nullish_coalescing() {
        let mut p = Parser::new("const x = a ?? b;");
        p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    }

    #[test]
    fn parse_variance_annotations() {
        let mut p = Parser::new("interface Box<in T> { value: T; }");
        p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

        let mut p = Parser::new("interface Box<out T> { value: T; }");
        p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

        let mut p = Parser::new("interface Box<in out T> { value: T; }");
        p.parse_statement();
        assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    }
}

#[cfg(test)]
mod batch1100_tests {
    use super::*;

    #[test]
    fn parse_constructor_param_static_modifier() {
        let (_, diags) = Parser::parse_source_file_text_with_diagnostics(
            "a.ts",
            "class foo {\n    constructor (static a: number) {\n    }\n}".to_string(),
        );
        eprintln!("diags: {diags:?}");
        let mut p = Parser::new("constructor (static a: number) {}");
        let _ = p.parse_expression();
    }
}

const KEYWORD_SUGGESTIONS: &[&str] = &[
    "abstract", "accessor", "any", "as", "asserts", "bigint", "boolean", "break", "case",
    "catch", "class", "continue", "const", "constenum", "constructor", "debugger", "declare",
    "default", "delete", "do", "else", "enum", "export", "extends", "false", "finally",
    "for", "from", "function", "get", "global", "if", "implements", "import", "in",
    "infer", "instanceof", "interface", "intrinsic", "is", "keyof", "let", "module",
    "namespace", "never", "new", "null", "number", "object", "package", "private",
    "protected", "public", "override", "out", "readonly", "return", "satisfies", "set",
    "static", "string", "super", "switch", "symbol", "this", "throw", "true", "try",
    "type", "typeof", "undefined", "unique", "unknown", "var", "void", "while", "with",
    "yield", "async", "await", "of",
];
