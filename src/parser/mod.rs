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

mod statements;
mod types;
mod expressions;
mod jsx;
mod declarations;
mod members;

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
