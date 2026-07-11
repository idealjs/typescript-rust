//! Syntax parser, ported from `internal/parser/parser.go`.
//!
//! This port covers statements, declarations, and expressions.
//! The full parser (6800+ lines in Go) is being ported incrementally.

use crate::ast::*;
use crate::core::text::TextRange;
use crate::scanner::Scanner;
use std::sync::Arc;

/// Parsing context, tracking what kind of list we're currently parsing.
///
/// Mirrors `parser.ParsingContext` in Go.
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

/// The parser.
///
/// Mirrors `parser.Parser` in Go.
pub struct Parser {
    scanner: Scanner,
    token: SyntaxKind,
    diagnostics: Vec<ParserDiagnostic>,
}

/// A parser diagnostic.
#[derive(Debug, Clone)]
pub struct ParserDiagnostic {
    pub message: String,
    pub range: TextRange,
}

/// Determine the `ScriptKind` from a file name's extension.
///
/// Mirrors `core.GetScriptKindFromFileName` in Go.
pub fn script_kind_from_file_name(file_name: &str) -> ScriptKind {
    let ext = file_name.rfind('.').map(|i| &file_name[i..]).unwrap_or("");
    match ext {
        ".ts" | ".mts" | ".cts" => ScriptKind::Ts,
        ".tsx" => ScriptKind::Ts,
        ".js" | ".mjs" | ".cjs" => ScriptKind::Js,
        ".jsx" => ScriptKind::Js,
        ".json" => ScriptKind::Json,
        _ => ScriptKind::Ts,
    }
}

impl Parser {
    pub fn new(source_text: impl Into<String>) -> Self {
        let text = source_text.into();
        let mut scanner = Scanner::new(text);
        let token = scanner.scan();
        Self {
            scanner,
            token,
            diagnostics: Vec::new(),
        }
    }

    /// Parse a full source file.
    pub fn parse_source_file(file_name: impl Into<String>) -> SourceFile {
        let file_name = file_name.into();
        let text = std::fs::read_to_string(&file_name).unwrap_or_default();
        Self::parse_source_file_text(&file_name, text)
    }

    /// Parse source text into a source file (for testing and API use).
    pub fn parse_source_file_text(file_name: &str, text: String) -> SourceFile {
        Self::parse_source_file_text_with_diagnostics(file_name, text).0
    }

    /// Parse source text into a source file, returning the file and any
    /// diagnostics produced during parsing.
    pub fn parse_source_file_text_with_diagnostics(
        file_name: &str,
        text: String,
    ) -> (SourceFile, Vec<ParserDiagnostic>) {
        let line_map = LineMap::from_text(&text);
        let mut parser = Parser::new(text.clone());
        let statements = parser.parse_list(ParsingContext::SourceElements, Parser::parse_statement);
        let end_of_file = parser.create_token_node();
        let pos = 0usize;
        let end = end_of_file.end();
        let node = Arc::new(Node::with_loc(
            SyntaxKind::SourceFile,
            NodeData::SourceFile(SourceFileData {
                statements: Arc::new(statements),
                end_of_file_token: end_of_file,
            }),
            TextRange::new(pos, end),
        ));
        let script_kind = script_kind_from_file_name(file_name);
        let language_variant = if script_kind == ScriptKind::Ts || script_kind == ScriptKind::Js {
            LanguageVariant::Standard
        } else {
            LanguageVariant::Standard
        };
        let file = SourceFile {
            node,
            file_name: file_name.to_string(),
            text,
            line_map,
            language_variant,
            script_kind,
        };
        (file, parser.diagnostics)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Token helpers
    // ─────────────────────────────────────────────────────────────────────

    /// Advance to the next token.
    fn next_token(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan();
        self.token
    }

    /// The current token position.
    fn token_pos(&self) -> usize {
        self.scanner.token_pos()
    }

    /// The current token end.
    fn token_end(&self) -> usize {
        self.scanner.token_end()
    }

    /// Whether the current token is preceded by a line break.
    fn has_preceding_line_break(&self) -> bool {
        self.scanner.has_preceding_line_break()
    }

    /// Expect a specific token, advancing past it. Reports an error if
    /// the current token doesn't match.
    fn expect(&mut self, expected: SyntaxKind) {
        if self.token == expected {
            self.next_token();
        } else {
            self.diagnostics.push(ParserDiagnostic {
                message: format!("Expected {:?} but got {:?}", expected, self.token),
                range: TextRange::new(self.token_pos(), self.token_end()),
            });
        }
    }

    /// If the current token matches, consume it and return true.
    fn parse_optional(&mut self, kind: SyntaxKind) -> bool {
        if self.token == kind {
            self.next_token();
            true
        } else {
            false
        }
    }

    /// If the current token matches, consume and return a token node.
    fn parse_optional_token(&mut self, kind: SyntaxKind) -> Option<Arc<Node>> {
        if self.token == kind {
            let node = self.create_token_node();
            self.next_token();
            Some(node)
        } else {
            None
        }
    }

    /// Create a token node for the current token.
    fn create_token_node(&self) -> Arc<Node> {
        Arc::new(Node::with_loc(
            self.token,
            NodeData::Token,
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

    // ─────────────────────────────────────────────────────────────────────
    // Semicolon handling (ASI support)
    // ─────────────────────────────────────────────────────────────────────

    /// Whether a semicolon can be parsed (explicit or via ASI).
    fn can_parse_semicolon(&self) -> bool {
        self.token == SyntaxKind::SemicolonToken
            || self.token == SyntaxKind::CloseBraceToken
            || self.token == SyntaxKind::EndOfFile
            || self.has_preceding_line_break()
    }

    /// Try to parse a semicolon (explicit or ASI). Returns true if consumed.
    fn try_parse_semicolon(&mut self) -> bool {
        if !self.can_parse_semicolon() {
            return false;
        }
        if self.token == SyntaxKind::SemicolonToken {
            self.next_token();
        }
        true
    }

    /// Parse a semicolon, reporting an error if missing.
    fn parse_semicolon(&mut self) -> bool {
        self.try_parse_semicolon() || {
            self.expect(SyntaxKind::SemicolonToken);
            false
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // List parsing
    // ─────────────────────────────────────────────────────────────────────

    /// Parse a list of elements until a terminator for the given context.
    fn parse_list(
        &mut self,
        context: ParsingContext,
        parse_element: fn(&mut Self) -> Arc<Node>,
    ) -> NodeList {
        let pos = self.token_pos();
        let mut nodes = Vec::new();
        while !self.is_list_terminator(context) {
            if self.is_list_element(context) {
                let element = parse_element(self);
                nodes.push(element);
            } else {
                // Error recovery: skip token if not in a valid context
                if self.is_in_some_parsing_context() {
                    break;
                }
                self.next_token();
            }
        }
        let end = self.token_pos();
        NodeList {
            loc: TextRange::new(pos, end),
            nodes,
        }
    }

    /// Parse a comma-delimited list of elements.
    fn parse_delimited_list(
        &mut self,
        context: ParsingContext,
        parse_element: fn(&mut Self) -> Arc<Node>,
    ) -> NodeList {
        let pos = self.token_pos();
        let mut nodes = Vec::new();
        loop {
            if self.is_list_element(context) {
                let element = parse_element(self);
                nodes.push(element);
                if self.parse_optional(SyntaxKind::CommaToken) {
                    continue;
                }
                if self.is_list_terminator(context) {
                    break;
                }
                // Expected comma but didn't find one
                self.expect(SyntaxKind::CommaToken);
                continue;
            }
            if self.is_list_terminator(context) {
                break;
            }
            // Error recovery
            if self.is_in_some_parsing_context() {
                break;
            }
            self.next_token();
        }
        let end = self.token_pos();
        NodeList {
            loc: TextRange::new(pos, end),
            nodes,
        }
    }

    /// Parse a bracket-enclosed delimited list.
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

    /// Check if the current token is a terminator for the given context.
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
                // If we can consume a semicolon (either explicitly, or with ASI), then
                // consider us done with parsing the list of variable declarators.
                // In a for-in/of, 'in'/'of' also terminates the list.
                // '=>' is for error recovery (arrow function).
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
            ParsingContext::TypeArguments => self.token == SyntaxKind::GreaterThanToken,
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

    /// Check if the current token starts an element in the given context.
    fn is_list_element(&self, context: ParsingContext) -> bool {
        match context {
            ParsingContext::SourceElements
            | ParsingContext::BlockStatements
            | ParsingContext::SwitchClauseStatements => self.is_start_of_statement(),
            ParsingContext::SwitchClauses => {
                self.token == SyntaxKind::CaseKeyword || self.token == SyntaxKind::DefaultKeyword
            }
            ParsingContext::VariableDeclarations => self.is_binding_identifier_or_pattern(),
            ParsingContext::Parameters => self.is_start_of_parameter(),
            ParsingContext::TypeMembers => !self.is_list_terminator(context),
            ParsingContext::TypeArguments | ParsingContext::TupleElementTypes => {
                self.is_start_of_type()
            }
            ParsingContext::ClassMembers
            | ParsingContext::EnumMembers
            | ParsingContext::ObjectLiteralMembers => !self.is_list_terminator(context),
            _ => false,
        }
    }

    /// True if positioned at element or terminator of the current list or any enclosing list.
    fn is_in_some_parsing_context(&self) -> bool {
        // Simplified: check for common terminators
        matches!(
            self.token,
            SyntaxKind::CloseBraceToken
                | SyntaxKind::CloseParenToken
                | SyntaxKind::CloseBracketToken
                | SyntaxKind::EndOfFile
        )
    }

    // ─────────────────────────────────────────────────────────────────────
    // Statement parsing
    // ─────────────────────────────────────────────────────────────────────

    /// Check if the current token starts a statement.
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

    /// Whether the current token starts an expression.
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

    /// Whether the current token starts a left-hand-side expression.
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

    /// Whether the current token is a literal.
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

    /// Parse a statement.
    pub fn parse_statement(&mut self) -> Arc<Node> {
        match self.token {
            SyntaxKind::SemicolonToken => self.parse_empty_statement(),
            SyntaxKind::OpenBraceToken => self.parse_block(),
            SyntaxKind::VarKeyword | SyntaxKind::LetKeyword | SyntaxKind::ConstKeyword => {
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
            SyntaxKind::InterfaceKeyword => self.parse_interface_declaration(),
            SyntaxKind::TypeKeyword => self.parse_type_alias_declaration(),
            SyntaxKind::EnumKeyword => self.parse_enum_declaration(),
            SyntaxKind::NamespaceKeyword | SyntaxKind::ModuleKeyword => {
                self.parse_namespace_declaration()
            }
            SyntaxKind::DeclareKeyword => self.parse_declaration_with_modifiers(Vec::new()),
            SyntaxKind::ImportKeyword => self.parse_import_declaration(),
            SyntaxKind::ExportKeyword => self.parse_export_declaration(),
            SyntaxKind::DebuggerKeyword => self.parse_debugger_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_empty_statement(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token(); // consume ';'
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
            _ => NodeFlags::empty(),
        };
        self.next_token(); // consume var/let/const/using
        let declarations = self.parse_delimited_list(
            ParsingContext::VariableDeclarations,
            Parser::parse_variable_declaration,
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
        let pos = self.token_pos();
        let name = self.parse_identifier_or_pattern();
        let type_node = self.parse_optional_type_annotation();
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();
            Some(self.parse_expression())
        } else {
            None
        };
        let end = initializer.as_ref().map_or(name.end(), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::VariableDeclaration,
            NodeData::VariableDeclaration(VariableDeclarationData {
                name,
                exclamation_token: None,
                type_node,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_identifier_or_pattern(&mut self) -> Arc<Node> {
        if self.token == SyntaxKind::OpenBracketToken {
            self.parse_array_binding_pattern()
        } else if self.token == SyntaxKind::OpenBraceToken {
            self.parse_object_binding_pattern()
        } else {
            self.parse_identifier()
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
            Some(self.parse_expression())
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
            Some(self.parse_expression())
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

    /// Parse an optional type annotation (`: Type`).
    fn parse_optional_type_annotation(&mut self) -> Option<Arc<Node>> {
        if self.token == SyntaxKind::ColonToken {
            self.next_token();
            Some(self.parse_type())
        } else {
            None
        }
    }

    /// Parse a TypeScript type node.
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
            SyntaxKind::StringKeyword
            | SyntaxKind::NumberKeyword
            | SyntaxKind::BooleanKeyword
            | SyntaxKind::AnyKeyword
            | SyntaxKind::UnknownKeyword
            | SyntaxKind::NeverKeyword
            | SyntaxKind::VoidKeyword
            | SyntaxKind::UndefinedKeyword
            | SyntaxKind::ObjectKeyword
            | SyntaxKind::BigIntKeyword
            | SyntaxKind::SymbolKeyword => {
                let pos = self.token_pos();
                let end = self.token_end();
                let node = self.create_token_node();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::TypeReference,
                    NodeData::TypeReferenceNode(TypeReferenceNodeData {
                        type_name: node,
                        type_arguments: None,
                    }),
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
            SyntaxKind::ThisKeyword => {
                let pos = self.token_pos();
                let end = self.token_end();
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::ThisType,
                    NodeData::ThisTypeNode,
                    TextRange::new(pos, end),
                ))
            }
            SyntaxKind::OpenBraceToken => self.parse_type_literal(),
            SyntaxKind::OpenBracketToken => self.parse_tuple_type(),
            SyntaxKind::OpenParenToken => self.parse_parenthesized_or_function_type(),
            SyntaxKind::LessThanToken => self.parse_function_type(),
            SyntaxKind::NewKeyword | SyntaxKind::AbstractKeyword => self.parse_constructor_type(),
            _ => self.parse_type_reference(),
        }
    }

    fn parse_literal_type_node(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
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

    fn parse_entity_name(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
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
        if self.parse_optional(SyntaxKind::DotDotDotToken) {
            let pos = self.token_pos();
            let type_node = self.parse_type();
            let end = type_node.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::RestType,
                NodeData::RestTypeNode(RestTypeNodeData { type_node }),
                TextRange::new(pos, end),
            ));
        }
        self.parse_type()
    }

    fn parse_parenthesized_or_function_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let parameters = self.parse_parameter_list();
        if self.parse_optional(SyntaxKind::EqualsGreaterThanToken) {
            let type_node = if self.is_start_of_type() {
                Some(self.parse_type())
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

        let type_node = if parameters.nodes.len() == 1 {
            parameters.nodes[0].clone()
        } else {
            self.missing_node(pos)
        };
        Arc::new(Node::with_loc(
            SyntaxKind::ParenthesizedType,
            NodeData::ParenthesizedTypeNode(ParenthesizedTypeNodeData { type_node }),
            TextRange::new(pos, self.token_pos()),
        ))
    }

    fn parse_function_type(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let type_node = if self.is_start_of_type() {
            Some(self.parse_type())
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
            Some(self.parse_type())
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

        // for-in / for-of
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
                    await_modifier: None,
                    initializer: initializer.unwrap(),
                    expression,
                    statement,
                }),
                TextRange::new(pos, end),
            ));
        }

        // Regular for loop
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
            // ASI prevented expression on same line
            Arc::new(Node::with_loc(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData {
                    text: String::new(),
                }),
                TextRange::new(self.token_pos(), self.token_pos()),
            ))
        };
        self.parse_semicolon();
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
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ExpressionStatement,
            NodeData::ExpressionStatement(ExpressionStatementData { expression }),
            TextRange::new(pos, end),
        ))
    }

    // ─────────────────────────────────────────────────────────────────────
    // Identifier and property name parsing
    // ─────────────────────────────────────────────────────────────────────

    fn is_identifier(&self) -> bool {
        self.token == SyntaxKind::Identifier || is_keyword(self.token)
    }

    fn is_binding_identifier_or_pattern(&self) -> bool {
        self.is_identifier()
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
                | SyntaxKind::SymbolKeyword
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
        ) || is_keyword(self.token)
    }

    fn is_literal_property_name(&self) -> bool {
        is_identifier_or_keyword(self.token)
            || self.token == SyntaxKind::StringLiteral
            || self.token == SyntaxKind::NumericLiteral
            || self.token == SyntaxKind::BigIntLiteral
    }

    fn parse_identifier(&mut self) -> Arc<Node> {
        if !self.is_identifier() {
            self.diagnostics.push(ParserDiagnostic {
                message: format!("Expected identifier but got {:?}", self.token),
                range: TextRange::new(self.token_pos(), self.token_end()),
            });
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
                self.next_token(); // consume '['
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

    // ─────────────────────────────────────────────────────────────────────
    // Expression parsing
    // ─────────────────────────────────────────────────────────────────────

    /// Parse an expression (entry point).
    pub fn parse_expression(&mut self) -> Arc<Node> {
        let expr = self.parse_assignment_expression();
        // Comma operator
        if self.token == SyntaxKind::CommaToken {
            let pos = expr.pos();
            let mut left = expr;
            while self.parse_optional(SyntaxKind::CommaToken) {
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
                            TextRange::new(0, 0),
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
        // Conditional expression (ternary)
        let expr = self.parse_binary_expression(0);
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
            SyntaxKind::TypeOfKeyword | SyntaxKind::VoidKeyword | SyntaxKind::DeleteKeyword => {
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
            _ => self.parse_postfix_expression(),
        }
    }

    fn parse_postfix_expression(&mut self) -> Arc<Node> {
        let operand = self.parse_left_hand_side_expression();
        // Postfix ++/--
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
        self.parse_call_and_member_chain(expr)
    }

    fn parse_new_expression(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token(); // consume 'new'
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
            self.parse_left_hand_side_expression()
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

    fn parse_call_and_member_chain(&mut self, expr: Arc<Node>) -> Arc<Node> {
        let mut expr = expr;
        loop {
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
                    self.next_token();
                    if self.token == SyntaxKind::OpenParenToken {
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
                    } else {
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
        self.expect(SyntaxKind::GreaterThanToken);
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
            SyntaxKind::ThisKeyword => self.parse_keyword_expression(SyntaxKind::ThisKeyword),
            SyntaxKind::SuperKeyword => self.parse_keyword_expression(SyntaxKind::SuperKeyword),
            SyntaxKind::OpenParenToken => self.parse_parenthesized_or_arrow(),
            SyntaxKind::OpenBracketToken => self.parse_array_literal(),
            SyntaxKind::OpenBraceToken => self.parse_object_literal(),
            SyntaxKind::FunctionKeyword => self.parse_function_expression(),
            SyntaxKind::ClassKeyword => self.parse_class_expression(),
            SyntaxKind::TemplateHead => self.parse_template_expression(),
            _ => {
                // Error recovery
                let pos = self.token_pos();
                let end = self.token_end();
                self.diagnostics.push(ParserDiagnostic {
                    message: format!("Unexpected token: {:?}", self.token),
                    range: TextRange::new(pos, end),
                });
                self.next_token();
                Arc::new(Node::with_loc(
                    SyntaxKind::Unknown,
                    NodeData::Token,
                    TextRange::new(pos, end),
                ))
            }
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
        let pos = self.token_pos();
        self.next_token(); // consume '('
        // Simplified: parse as parenthesized expression
        let expr = self.parse_expression();
        self.expect(SyntaxKind::CloseParenToken);
        let end = self.token_pos();

        // Arrow function: (params) => body
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
        // Simplified: property assignment
        let name = self.parse_property_name();
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
            // Shorthand property
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

    fn parse_function_expression(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token(); // consume 'function'
        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);
        let name = if self.is_identifier() {
            Some(self.parse_identifier())
        } else {
            None
        };
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_type_annotation();
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
        self.next_token(); // consume 'class'
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

    // ─────────────────────────────────────────────────────────────────────
    // Declaration parsing
    // ─────────────────────────────────────────────────────────────────────

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

    fn parse_declaration_with_modifiers(
        &mut self,
        mut modifiers: Vec<(SyntaxKind, usize, usize)>,
    ) -> Arc<Node> {
        while matches!(
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
        ) {
            let kind = self.token;
            let pos = self.token_pos();
            let end = self.token_end();
            self.next_token();
            modifiers.push((kind, pos, end));
        }

        let modifiers = Some(self.make_modifier_list(modifiers));
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
            SyntaxKind::VarKeyword | SyntaxKind::LetKeyword | SyntaxKind::ConstKeyword => {
                self.parse_variable_statement_with_modifiers(modifiers)
            }
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_function_declaration(&mut self) -> Arc<Node> {
        self.parse_function_declaration_with_modifiers(None)
    }

    fn parse_function_declaration_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token(); // consume 'function'
        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);
        let name = if self.is_identifier() {
            Some(self.parse_identifier())
        } else {
            None
        };
        let type_parameters = self.parse_optional_type_parameters();
        let parameters = self.parse_parameter_list();
        let type_node = self.parse_optional_type_annotation();
        let body = if self.token == SyntaxKind::OpenBraceToken {
            Some(self.parse_block())
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

    fn parse_class_declaration(&mut self) -> Arc<Node> {
        self.parse_class_declaration_with_modifiers(None)
    }

    fn parse_class_declaration_with_modifiers(
        &mut self,
        modifiers: Option<Arc<ModifierList>>,
    ) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token(); // consume 'class'
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
        self.next_token(); // consume 'interface'
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
        self.next_token(); // consume 'type'
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
        self.next_token(); // consume 'enum'
        let name = self.parse_identifier();
        self.expect(SyntaxKind::OpenBraceToken);
        let members = self.parse_list(ParsingContext::EnumMembers, Self::parse_enum_member);
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
        self.next_token(); // consume 'namespace' or 'module'
        let name = self.parse_namespace_name();
        let body = if self.token == SyntaxKind::OpenBraceToken {
            let body_pos = self.token_pos();
            self.next_token(); // consume '{'
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

    fn parse_namespace_name(&mut self) -> Arc<Node> {
        let name = self.parse_identifier();
        // Handle dotted namespace names: namespace A.B.C { }
        if self.token == SyntaxKind::DotToken {
            let pos = name.pos();
            let mut left = name;
            while self.token == SyntaxKind::DotToken {
                self.next_token(); // consume '.'
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
        self.next_token(); // consume 'import'
        let import_clause = if self.token == SyntaxKind::StringLiteral {
            None // import "module" (side-effect only)
        } else {
            Some(self.parse_import_clause())
        };
        let module_specifier = if import_clause.is_some() {
            self.expect(SyntaxKind::FromKeyword);
            self.parse_string_literal_node()
        } else {
            self.parse_string_literal_node()
        };
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ImportDeclaration,
            NodeData::ImportDeclaration(ImportDeclarationData {
                modifiers: None,
                import_clause,
                module_specifier,
                attributes: None,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_import_clause(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        if self.token == SyntaxKind::AsteriskToken {
            self.next_token(); // consume '*'
            self.expect(SyntaxKind::AsKeyword);
            let name = self.parse_identifier();
            let end = name.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::NamespaceImport,
                NodeData::NamespaceImport(NamespaceImportData { name }),
                TextRange::new(pos, end),
            ));
        }
        if self.token == SyntaxKind::OpenBraceToken {
            let named = self.parse_named_imports();
            let end = named.end();
            return Arc::new(Node::with_loc(
                SyntaxKind::ImportClause,
                NodeData::ImportClause(ImportClauseData {
                    phase_modifier: None,
                    name: None,
                    named_bindings: Some(named),
                }),
                TextRange::new(pos, end),
            ));
        }
        // default import: import foo from '...'
        let name = self.parse_identifier();
        let named_bindings = if self.parse_optional(SyntaxKind::CommaToken) {
            if self.token == SyntaxKind::AsteriskToken {
                Some(self.parse_namespace_import_after_comma())
            } else {
                Some(self.parse_named_imports())
            }
        } else {
            None
        };
        let end = named_bindings.as_ref().map_or(name.end(), |n| n.end());
        Arc::new(Node::with_loc(
            SyntaxKind::ImportClause,
            NodeData::ImportClause(ImportClauseData {
                phase_modifier: None,
                name: Some(name),
                named_bindings,
            }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_namespace_import_after_comma(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        self.next_token(); // consume '*'
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
        let is_type_only = self.parse_optional(SyntaxKind::TypeKeyword);
        let first = self.parse_identifier_name_or_keyword();
        let (property_name, name) = if self.parse_optional(SyntaxKind::AsKeyword) {
            let local = self.parse_identifier_name_or_keyword();
            (Some(first), local)
        } else {
            (None, first)
        };
        // optional comma
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

    fn parse_identifier_name_or_keyword(&mut self) -> Arc<Node> {
        if self.is_identifier() {
            self.parse_identifier()
        } else {
            // keyword used as identifier
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

    /// Create a `ModifierList` containing a single `ExportKeyword` token,
    /// used for `export const/let/var` statements.
    fn make_export_modifier(&self, pos: usize) -> ModifierList {
        let export_token = Arc::new(Node::with_loc(
            SyntaxKind::ExportKeyword,
            NodeData::Token,
            TextRange::new(pos, pos),
        ));
        ModifierList::new(vec![export_token], ModifierFlags::Export)
    }

    fn parse_export_declaration(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let export_end = self.token_end();
        self.next_token(); // consume 'export'

        if self.token == SyntaxKind::DeclareKeyword {
            return self.parse_declaration_with_modifiers(vec![(
                SyntaxKind::ExportKeyword,
                pos,
                export_end,
            )]);
        }

        // export default ...
        if self.token == SyntaxKind::DefaultKeyword {
            self.next_token(); // consume 'default'
            if self.token == SyntaxKind::FunctionKeyword {
                let func = self.parse_function_declaration();
                return func; // export default function → FunctionDeclaration
            }
            if self.token == SyntaxKind::ClassKeyword {
                let class = self.parse_class_declaration();
                return class; // export default class → ClassDeclaration
            }
            // export default <expression>
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

        // export = ...
        if self.token == SyntaxKind::EqualsToken {
            self.next_token(); // consume '='
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

        // export function/class/interface/type/enum/namespace declarations
        match self.token {
            SyntaxKind::FunctionKeyword => return self.parse_function_declaration(),
            SyntaxKind::ClassKeyword => return self.parse_class_declaration(),
            SyntaxKind::InterfaceKeyword => return self.parse_interface_declaration(),
            SyntaxKind::TypeKeyword => return self.parse_type_alias_declaration(),
            SyntaxKind::EnumKeyword => return self.parse_enum_declaration(),
            SyntaxKind::NamespaceKeyword | SyntaxKind::ModuleKeyword => {
                return self.parse_namespace_declaration();
            }
            SyntaxKind::ConstKeyword | SyntaxKind::LetKeyword | SyntaxKind::VarKeyword => {
                // export const/let/var x = ...
                let export_mod = self.make_export_modifier(pos);
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

        // export * as foo from '...' | export * from '...' | export { ... } from '...' | export { ... }
        let export_clause = if self.parse_optional(SyntaxKind::AsteriskToken) {
            // export * [as name] from '...'
            if self.parse_optional(SyntaxKind::AsKeyword) {
                let name = self.parse_identifier_name_or_keyword();
                let end = name.end();
                Some(Arc::new(Node::with_loc(
                    SyntaxKind::NamespaceExport,
                    NodeData::NamespaceExport(NamespaceExportData { name }),
                    TextRange::new(pos, end),
                )))
            } else {
                None // export * from '...' — will be handled via NamedExports with a star
            }
        } else {
            // export { ... }
            Some(self.parse_named_exports())
        };

        let module_specifier = if self.parse_optional(SyntaxKind::FromKeyword) {
            Some(self.parse_string_literal_node())
        } else {
            None
        };
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::ExportDeclaration,
            NodeData::ExportDeclaration(ExportDeclarationData {
                modifiers: None,
                is_type_only: false,
                export_clause,
                module_specifier,
                attributes: None,
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
        let is_type_only = self.parse_optional(SyntaxKind::TypeKeyword);
        let first = self.parse_identifier_name_or_keyword();
        let (property_name, name) = if self.parse_optional(SyntaxKind::AsKeyword) {
            let local = self.parse_identifier_name_or_keyword();
            (Some(first), local)
        } else {
            (None, first)
        };
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
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::EnumMember,
            NodeData::EnumMember(EnumMemberData { name, initializer }),
            TextRange::new(pos, end),
        ))
    }

    fn parse_template_expression(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let head = self.create_token_node();
        self.next_token(); // consume template head
        let mut spans = Vec::new();
        loop {
            let expression = self.parse_expression();
            let literal = if self.token == SyntaxKind::CloseBraceToken {
                // Continue template
                let lit = self.scanner.scan();
                let _ = lit;
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
                break;
            }
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

    // ─────────────────────────────────────────────────────────────────────
    // Type parameter and parameter parsing
    // ─────────────────────────────────────────────────────────────────────

    fn parse_optional_type_parameters(&mut self) -> Option<Arc<NodeList>> {
        if self.token != SyntaxKind::LessThanToken {
            return None;
        }
        let pos = self.token_pos();
        self.next_token();
        let params =
            self.parse_delimited_list(ParsingContext::TypeParameters, Parser::parse_type_parameter);
        self.expect(SyntaxKind::GreaterThanToken);
        let end = self.token_pos();
        Some(Arc::new(NodeList {
            loc: TextRange::new(pos, end),
            nodes: params.nodes,
        }))
    }

    fn parse_type_parameter(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
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
                modifiers: None,
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

    fn parse_parameter(&mut self) -> Arc<Node> {
        let pos = self.token_pos();
        let dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);
        let name = self.parse_identifier_or_pattern();
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
                modifiers: None,
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
        if self.token == SyntaxKind::NewKeyword {
            return self.parse_signature_member(SyntaxKind::ConstructSignature);
        }

        let pos = self.token_pos();
        let modifiers = self.parse_type_member_modifiers();
        if self.is_index_signature_start() {
            return self.parse_index_signature(pos, modifiers);
        }

        let name = self.parse_property_name();
        let postfix_token = self.parse_optional_token(SyntaxKind::QuestionToken);
        let type_parameters = self.parse_optional_type_parameters();
        if self.token == SyntaxKind::OpenParenToken {
            let parameters = self.parse_parameter_list();
            let type_node = self.parse_optional_type_annotation();
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

    fn is_index_signature_start(&self) -> bool {
        self.token == SyntaxKind::OpenBracketToken
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
        let type_node = self.parse_optional_type_annotation();
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
        // Simplified: parse as method or property
        let pos = self.token_pos();
        let name = self.parse_property_name();
        let postfix_token = self
            .parse_optional_token(SyntaxKind::QuestionToken)
            .or_else(|| self.parse_optional_token(SyntaxKind::ExclamationToken));

        if self.token == SyntaxKind::OpenParenToken {
            // Method
            let type_parameters = self.parse_optional_type_parameters();
            let parameters = self.parse_parameter_list();
            let type_node = self.parse_optional_type_annotation();
            let body = if self.token == SyntaxKind::OpenBraceToken {
                Some(self.parse_block())
            } else {
                self.parse_semicolon();
                None
            };
            let end = body.as_ref().map_or(self.token_pos(), |b| b.end());
            return Arc::new(Node::with_loc(
                SyntaxKind::MethodDeclaration,
                NodeData::MethodDeclaration(MethodDeclarationData {
                    modifiers: None,
                    asterisk_token: None,
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

        // Property
        let type_node = self.parse_optional_type_annotation();
        let initializer = if self.token == SyntaxKind::EqualsToken {
            self.next_token();
            Some(self.parse_assignment_expression())
        } else {
            None
        };
        self.parse_semicolon();
        let end = self.token_pos();
        Arc::new(Node::with_loc(
            SyntaxKind::PropertyDeclaration,
            NodeData::PropertyDeclaration(PropertyDeclarationData {
                modifiers: None,
                name,
                postfix_token,
                type_node,
                initializer,
            }),
            TextRange::new(pos, end),
        ))
    }

    // ─────────────────────────────────────────────────────────────────────
    // Diagnostics
    // ─────────────────────────────────────────────────────────────────────

    /// Get the parser diagnostics.
    pub fn diagnostics(&self) -> &[ParserDiagnostic] {
        &self.diagnostics
    }
}

// ─────────────────────────────────────────────────────────────────────
// Helper functions
// ─────────────────────────────────────────────────────────────────────

/// Binary operator precedence (higher = binds tighter).
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
    token == SyntaxKind::Identifier || is_keyword(token)
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
}
