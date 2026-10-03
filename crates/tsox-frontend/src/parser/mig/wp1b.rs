#![allow(unused_imports)]

//! wp1b: parser.go 余量收尾续作批（孪生/架构裁决见文件头注释）

use crate::ast::is_modifier_kind;
use crate::ast::{Node, NodeData, NodeFlags, NodeList, SyntaxKind};
use crate::ast::LanguageVariant;
use crate::parser::parsing_context::{ParsingContext, Parser};
use crate::scanner::token_to_string;
use std::sync::Arc;
use crate::ast::ScriptKind;
use crate::ast::mig::m3b_2::NodeFactory;
use tsox_core::core::text::TextRange;
use tsox_core::diagnostics::{self, Message};

// 孪生/架构裁决（Go 名 → Rust 名/位置）：
// parseCaseClause → statements_parser.rs parse_case_or_default_clause（CaseKeyword 分支内联）
// parseDefaultClause → statements_parser.rs parse_case_or_default_clause（else 分支内联）
// parseDeleteExpression → expressions_parser_4.rs 内联（VoidKeyword|DeleteKeyword 分支）
// parseAwaitExpression → expressions_parser_4.rs 内联（AwaitKeyword 分支）
// parseComputedPropertyName → impl_chunk_parser_5.rs 内联于 parse_property_name（OpenBracketToken 分支）
// parseClassStaticBlockDeclaration → members_parser_4.rs 内联（StaticKeyword + `{` 前瞻分支）
// parseClassStaticBlockBody → members_parser_4.rs 内联（Yield 关/Await 开的 save/restore 直写字段）
// parseConditionalExpressionRest → expressions_parser.rs 内联于 parse_assignment_expression（QuestionToken 分支）
// parseEmptyNodeList → 架构差异：newNodeList 以 NodeList 直构表达
// createMissingList → 架构差异：缺失列表以 Option/missing 标记表达（wp1 已裁决）
// parseExpectedMatchingBrackets 的 lastError.AddRelatedInfo → 架构差异：ParserDiagnostic 无 related-info 结构，本轮仅保留主错误上报
// initializeState 的 scanner.SetOnError(scanError) → 架构差异：drain_scanner_errors 约定承接扫描错误，error_callback 保持 None
// finishSourceFile 的 SetDiagnostics/SetJSDocDiagnostics → 架构差异：诊断由 parse 返回值 (SourceFile, Vec<ParserDiagnostic>) 携带
// finishSourceFile 的 Flags|=sourceFlags / NodeCount/TextCount/IdentifierCount → 架构差异：SourceFile 无对应存储，计数由 NodeFactory 原子计数承接
// finishSourceFile 的 processPragmasIntoFields → wp1_2.rs process_pragmas_into_fields（活体入口 impl_chunk_parser.rs parse_source_file_text_with_diagnostics；CheckJsDirective 无存储未落字段）
// finishSourceFile 的 ReparsedClones → 架构差异：由 m4d.rs REPARSED_CLONES thread_local 承接，SourceFile 无存储
// createJSDocCache → 架构差异：jsdocInfos 未积累，缓存维持空表

thread_local! {
    static OPTS: std::cell::RefCell<Option<crate::ast::mig::m3e_2::SourceFileParseOptions>> =
        const { std::cell::RefCell::new(None) };
    static SCRIPT_KIND: std::cell::Cell<ScriptKind> = const { std::cell::Cell::new(ScriptKind::Unknown) };
    static FILE_PRAGMAS: std::cell::RefCell<Vec<super::wp1_2::Pragma>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static IDENTIFIER_COUNT: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

fn get_language_variant(script_kind: ScriptKind) -> LanguageVariant { ::tsox_core::fntrace::enter("get_language_variant"); 
    match script_kind {
        ScriptKind::Tsx | ScriptKind::Jsx | ScriptKind::Js | ScriptKind::Json => {
            LanguageVariant::Jsx
        }
        _ => LanguageVariant::Standard,
    }
}

impl Parser {
    pub(crate) fn initialize_state(
        &mut self,
        opts: crate::ast::mig::m3e_2::SourceFileParseOptions,
        source_text: String,
        script_kind: ScriptKind,
    ) { ::tsox_core::fntrace::enter("initialize_state"); 
        if script_kind == ScriptKind::Unknown {
            panic!("ScriptKind must be specified when parsing source file: {}", opts.file_name);
        }
        self.scanner.reset();
        OPTS.with(|c| *c.borrow_mut() = Some(opts));
        SCRIPT_KIND.with(|c| c.set(script_kind));
        self.language_variant = get_language_variant(script_kind);
        self.javascript_file = matches!(
            script_kind,
            ScriptKind::Js | ScriptKind::Jsx | ScriptKind::Json
        );
        self.scanner.set_text(source_text);
        self.scanner.set_language_variant(self.language_variant);
    }

    pub(crate) fn finish_source_file(
        &mut self,
        result: &mut crate::ast::SourceFile,
        is_declaration_file: bool,
    ) { ::tsox_core::fntrace::enter("finish_source_file"); 
        result.comment_directives = self.scanner.comment_directives().to_vec();
        FILE_PRAGMAS.with(|c| *c.borrow_mut() = super::wp1_2::get_comment_pragmas(self.source_text()));
        self.process_pragmas_into_fields(result);
        result.is_declaration_file = is_declaration_file;
        result.language_variant = self.language_variant;
        result.script_kind = SCRIPT_KIND.with(|c| c.get());
        result.set_jsdoc_cache(self.create_jsdoc_cache());
        if !self.is_javascript() {
            result.set_has_lazy_jsdoc(true);
        }
        let external_module_indicator_options = OPTS
            .with(|c| c.borrow().as_ref().map(|o| o.external_module_indicator_options))
            .unwrap_or_default();
        crate::ast::mig::m3e_2::set_external_module_indicator_with_options(
            result,
            external_module_indicator_options,
        );
    }

    pub(crate) fn create_jsdoc_cache(
        &self,
    ) -> std::collections::HashMap<u64, Vec<Arc<Node>>> { ::tsox_core::fntrace::enter("create_jsdoc_cache"); 
        std::collections::HashMap::new()
    }

    pub(crate) fn new_identifier(&self, text: &str) -> Node { ::tsox_core::fntrace::enter("new_identifier"); 
        IDENTIFIER_COUNT.with(|c| c.set(c.get() + 1));
        Node::new(
            SyntaxKind::Identifier,
            NodeData::Identifier(crate::ast::node_data_generated::IdentifierData {
                text: text.to_string(),
            }),
        )
    }

    pub(crate) fn parse_call_expression_rest(&mut self, pos: usize, expression: Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_call_expression_rest"); 
        let mut expression = expression;
        loop {
            expression = self.parse_member_expression_rest(pos, expression, true);
            let mut type_arguments: Option<Arc<NodeList>> = None;
            let question_dot_token = self.parse_optional_token(SyntaxKind::QuestionDotToken);
            if question_dot_token.is_some() {
                type_arguments = self.try_parse_type_arguments_in_expression();
                if self.is_template_start_of_tagged_template() {
                    expression = self.parse_tagged_template_rest(pos, expression, question_dot_token.clone(), type_arguments);
                    continue;
                }
            }
            if type_arguments.is_some() || self.token == SyntaxKind::OpenParenToken {
                if question_dot_token.is_none() && expression.kind == SyntaxKind::ExpressionWithTypeArguments {
                    let (inner_expression, inner_type_arguments) = match &expression.data {
                        NodeData::ExpressionWithTypeArguments(d) => {
                            (d.expression.clone(), d.type_arguments.clone())
                        }
                        _ => (expression.clone(), None),
                    };
                    type_arguments = inner_type_arguments;
                    expression = inner_expression;
                }
                let argument_list = self.parse_argument_list();
                let is_optional_chain = question_dot_token.is_some() || self.try_reparse_optional_chain(&expression);
                let flags = if is_optional_chain { NodeFlags::OptionalChain } else { NodeFlags::empty() };
                let mut n = Node::new(
                    SyntaxKind::CallExpression,
                    NodeData::CallExpression(crate::ast::node_data_generated::CallExpressionData {
                        expression: expression.clone(),
                        question_dot_token: question_dot_token.clone(),
                        type_arguments: type_arguments.clone(),
                        arguments: argument_list,
                    }),
                );
                n.flags = flags;
                let call = Arc::new(self.finish_node(&mut n, pos));
                self.check_js_syntax(&call);
                self.unparse_expression_with_type_arguments(None, type_arguments.as_deref(), &call);
                expression = call;
                continue;
            }
            if let Some(question_dot_token) = question_dot_token {
                self.parse_error_at_current_token(diagnostics::IDENTIFIER_EXPECTED, &[]);
                let name = self.missing_identifier_expression();
                let mut n = Node::new(
                    SyntaxKind::PropertyAccessExpression,
                    NodeData::PropertyAccessExpression(
                        crate::ast::node_data_generated::PropertyAccessExpressionData {
                            expression,
                            question_dot_token: Some(question_dot_token),
                            name,
                        },
                    ),
                );
                n.flags = NodeFlags::OptionalChain;
                expression = Arc::new(self.finish_node(&mut n, pos));
            }
            break;
        }
        expression
    }

    pub(crate) fn parse_member_expression_rest(
        &mut self,
        pos: usize,
        expression: Arc<Node>,
        allow_optional_chain: bool,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_member_expression_rest"); 
        let mut expression = expression;
        loop {
            let mut question_dot_token: Option<Arc<Node>> = None;
            let is_property_access;
            if allow_optional_chain && self.is_start_of_optional_property_or_element_access_chain() {
                question_dot_token = Some(self.parse_expected_token(SyntaxKind::QuestionDotToken));
                is_property_access = super::wp1::token_is_identifier_or_keyword(self.token);
            } else {
                is_property_access = self.parse_optional(SyntaxKind::DotToken);
            }
            if is_property_access {
                expression = self.parse_property_access_expression_rest(pos, expression, question_dot_token);
                continue;
            }
            if (question_dot_token.is_some() || !self.in_decorator_context())
                && self.parse_optional(SyntaxKind::OpenBracketToken)
            {
                expression = self.parse_element_access_expression_rest(pos, expression, question_dot_token);
                continue;
            }
            if self.is_template_start_of_tagged_template() {
                if question_dot_token.is_none() && expression.kind == SyntaxKind::ExpressionWithTypeArguments {
                    let (inner_expression, inner_type_arguments) = match &expression.data {
                        NodeData::ExpressionWithTypeArguments(d) => {
                            (d.expression.clone(), d.type_arguments.clone())
                        }
                        _ => (expression.clone(), None),
                    };
                    expression = self.parse_tagged_template_rest(pos, inner_expression, question_dot_token, inner_type_arguments);
                } else {
                    expression = self.parse_tagged_template_rest(pos, expression, question_dot_token, None);
                }
                continue;
            }
            if question_dot_token.is_none() {
                if self.token == SyntaxKind::ExclamationToken && !self.has_preceding_line_break() {
                    self.next_token();
                    let mut n = Node::new(
                        SyntaxKind::NonNullExpression,
                        NodeData::NonNullExpression(
                            crate::ast::node_data_generated::NonNullExpressionData {
                                expression,
                            },
                        ),
                    );
                    n.flags = NodeFlags::empty();
                    let non_null = Arc::new(self.finish_node(&mut n, pos));
                    self.check_js_syntax(&non_null);
                    expression = non_null;
                    continue;
                }
                if let Some(type_arguments) = self.try_parse_type_arguments_in_expression() {
                    let mut n = Node::new(
                        SyntaxKind::ExpressionWithTypeArguments,
                        NodeData::ExpressionWithTypeArguments(
                            crate::ast::node_data_generated::ExpressionWithTypeArgumentsData {
                                expression,
                                type_arguments: Some(type_arguments),
                            },
                        ),
                    );
                    n.flags = NodeFlags::empty();
                    expression = Arc::new(self.finish_node(&mut n, pos));
                    continue;
                }
            }
            return expression;
        }
    }

    pub(crate) fn is_start_of_optional_property_or_element_access_chain(&mut self) -> bool { ::tsox_core::fntrace::enter("is_start_of_optional_property_or_element_access_chain"); 
        self.token == SyntaxKind::QuestionDotToken
            && self.look_ahead(Parser::next_token_is_identifier_or_keyword_or_open_bracket_or_template)
    }

    pub(crate) fn parse_expected_token(&mut self, kind: SyntaxKind) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_expected_token"); 
        match self.parse_optional_token(kind) {
            Some(t) => t,
            None => {
                self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[token_to_string(kind)]);
                let mut n = Node::new(kind, NodeData::Token);
                Arc::new(self.finish_node(&mut n, self.node_pos()))
            }
        }
    }

    pub(crate) fn parse_property_access_expression_rest(
        &mut self,
        pos: usize,
        expression: Arc<Node>,
        question_dot_token: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_property_access_expression_rest"); 
        let name = self.parse_right_side_of_dot();
        let is_optional_chain = question_dot_token.is_some() || self.try_reparse_optional_chain(&expression);
        let flags = if is_optional_chain { NodeFlags::OptionalChain } else { NodeFlags::empty() };
        let mut n = Node::new(
            SyntaxKind::PropertyAccessExpression,
            NodeData::PropertyAccessExpression(
                crate::ast::node_data_generated::PropertyAccessExpressionData {
                    expression: expression.clone(),
                    question_dot_token: question_dot_token.clone(),
                    name: name.clone(),
                },
            ),
        );
        n.flags = flags;
        self.finish_node_arc(&mut n, pos)
    }

    pub(crate) fn parse_element_access_expression_rest(
        &mut self,
        pos: usize,
        expression: Arc<Node>,
        question_dot_token: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_element_access_expression_rest"); 
        let mut argument_expression = self.missing_identifier_expression();
        if self.token == SyntaxKind::CloseBracketToken {
            let p = self.node_pos();
            self.parse_error_at(p, p, diagnostics::AN_ELEMENT_ACCESS_EXPRESSION_SHOULD_TAKE_AN_ARGUMENT, &[]);
        } else {
            argument_expression = self.parse_expression_allow_in();
        }
        self.expect(SyntaxKind::CloseBracketToken);
        let is_optional_chain = question_dot_token.is_some() || self.try_reparse_optional_chain(&expression);
        let flags = if is_optional_chain { NodeFlags::OptionalChain } else { NodeFlags::empty() };
        let mut n = Node::new(
            SyntaxKind::ElementAccessExpression,
            NodeData::ElementAccessExpression(
                crate::ast::node_data_generated::ElementAccessExpressionData {
                    expression,
                    question_dot_token,
                    argument_expression,
                },
            ),
        );
        n.flags = flags;
        self.finish_node_arc(&mut n, pos)
    }

    pub(crate) fn parse_expression_allow_in(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_expression_allow_in"); 
        let save = self.disallow_in_context;
        self.disallow_in_context = false;
        let result = self.parse_assignment_expression();
        self.disallow_in_context = save;
        result
    }

    pub(crate) fn parse_tagged_template_rest(
        &mut self,
        pos: usize,
        tag: Arc<Node>,
        question_dot_token: Option<Arc<Node>>,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_tagged_template_rest"); 
        let template = if self.token == SyntaxKind::NoSubstitutionTemplateLiteral {
            self.scanner.re_scan_template_token();
            self.parse_literal_expression()
        } else {
            self.parse_template_expression_ex(true)
        };
        let is_optional_chain = question_dot_token.is_some() || tag.flags.intersects(NodeFlags::OptionalChain);
        let flags = if is_optional_chain { NodeFlags::OptionalChain } else { NodeFlags::empty() };
        let mut n = Node::new(
            SyntaxKind::TaggedTemplateExpression,
            NodeData::TaggedTemplateExpression(
                crate::ast::node_data_generated::TaggedTemplateExpressionData {
                    tag,
                    question_dot_token,
                    type_arguments,
                    template,
                },
            ),
        );
        n.flags = flags;
        let result = self.finish_node_arc(&mut n, pos);
        self.check_js_syntax(&result);
        result
    }

    pub(crate) fn finish_node_arc(&mut self, node: &mut Node, pos: usize) -> Arc<Node> { ::tsox_core::fntrace::enter("finish_node_arc"); 
        Arc::new(self.finish_node(node, pos))
    }

    pub(crate) fn parse_literal_expression(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_literal_expression"); 
        let pos = self.node_pos();
        let text = self.scanner.token_value();
        let token_flags = self.scanner.token_flags();
        let kind = self.token;
        let mut n = match kind {
            SyntaxKind::StringLiteral => Node::new(
                kind,
                NodeData::StringLiteral(crate::ast::node_data_generated::StringLiteralData { text, token_flags }),
            ),
            SyntaxKind::NumericLiteral => Node::new(
                kind,
                NodeData::NumericLiteral(crate::ast::node_data_generated::NumericLiteralData { text, token_flags }),
            ),
            SyntaxKind::BigIntLiteral => Node::new(
                kind,
                NodeData::BigIntLiteral(crate::ast::node_data_generated::BigIntLiteralData { text, token_flags }),
            ),
            SyntaxKind::RegularExpressionLiteral => Node::new(
                kind,
                NodeData::RegularExpressionLiteral(
                    crate::ast::node_data_generated::RegularExpressionLiteralData { text, token_flags },
                ),
            ),
            SyntaxKind::NoSubstitutionTemplateLiteral => Node::new(
                kind,
                NodeData::NoSubstitutionTemplateLiteral(
                    crate::ast::node_data_generated::NoSubstitutionTemplateLiteralData {
                        text,
                        template_flags: token_flags,
                    },
                ),
            ),
            _ => panic!("Unhandled case in parseLiteralExpression"),
        };
        self.next_token();
        self.finish_node_arc(&mut n, pos)
    }

    pub(crate) fn parse_arrow_function_expression_body(
        &mut self,
        is_async: bool,
        allow_return_type_in_arrow_function: bool,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_arrow_function_expression_body"); 
        if self.token == SyntaxKind::OpenBraceToken {
            return self.parse_function_block(false, is_async);
        }
        if self.token != SyntaxKind::SemicolonToken
            && self.token != SyntaxKind::FunctionKeyword
            && self.token != SyntaxKind::ClassKeyword
            && self.is_start_of_statement()
            && !self.is_start_of_expression_statement()
        {
            return self.parse_function_block_ex(false, is_async, true);
        }
        let save_await_context = self.await_context;
        let save_yield_context = self.yield_context;
        self.await_context = is_async;
        self.yield_context = false;
        let node = self.parse_assignment_expression_or_higher_worker(allow_return_type_in_arrow_function);
        self.await_context = save_await_context;
        self.yield_context = save_yield_context;
        node
    }

    pub(crate) fn parse_assignment_expression_or_higher_worker(
        &mut self,
        _allow_return_type_in_arrow_function: bool,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_assignment_expression_or_higher_worker"); 
        self.parse_assignment_expression()
    }

    pub(crate) fn next_token_can_follow_modifier(&mut self) -> bool { ::tsox_core::fntrace::enter("next_token_can_follow_modifier"); 
        match self.token {
            SyntaxKind::ConstKeyword => self.next_token() == SyntaxKind::EnumKeyword,
            SyntaxKind::ExportKeyword => {
                self.next_token();
                if self.token == SyntaxKind::DefaultKeyword {
                    return self.look_ahead(Parser::next_token_can_follow_default_keyword);
                }
                if self.token == SyntaxKind::TypeKeyword {
                    return self.look_ahead(Parser::next_token_can_follow_export_modifier);
                }
                Parser::can_follow_export_modifier(self.token)
            }
            SyntaxKind::DefaultKeyword => self.next_token_can_follow_default_keyword(),
            SyntaxKind::StaticKeyword => {
                self.next_token();
                Parser::token_can_follow_modifier(self.token)
            }
            SyntaxKind::GetKeyword | SyntaxKind::SetKeyword => {
                self.next_token();
                Parser::token_can_follow_get_or_set(self.token)
            }
            _ => self.next_token_is_on_same_line_and_can_follow_modifier(),
        }
    }

    pub(crate) fn next_token_can_follow_default_keyword(&mut self) -> bool { ::tsox_core::fntrace::enter("next_token_can_follow_default_keyword"); 
        match self.next_token() {
            SyntaxKind::ClassKeyword
            | SyntaxKind::FunctionKeyword
            | SyntaxKind::InterfaceKeyword
            | SyntaxKind::AtToken => true,
            SyntaxKind::AbstractKeyword => {
                self.look_ahead(Parser::next_token_is_class_keyword_on_same_line)
            }
            SyntaxKind::AsyncKeyword => {
                self.look_ahead(Parser::next_token_is_function_keyword_on_same_line)
            }
            _ => false,
        }
    }

    pub(crate) fn next_token_can_follow_export_modifier(&mut self) -> bool { ::tsox_core::fntrace::enter("next_token_can_follow_export_modifier"); 
        self.next_token();
        Parser::can_follow_export_modifier(self.token)
    }

    pub(crate) fn parse_contextual_modifier(&mut self, t: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("parse_contextual_modifier"); 
        let state = self.mark();
        if self.token == t && self.next_token_can_follow_modifier() {
            return true;
        }
        self.rewind(state);
        false
    }

    pub(crate) fn parse_any_contextual_modifier(&mut self) -> bool { ::tsox_core::fntrace::enter("parse_any_contextual_modifier"); 
        let state = self.mark();
        if is_modifier_kind(self.token) && self.next_token_can_follow_modifier() {
            return true;
        }
        self.rewind(state);
        false
    }

    pub(crate) fn parse_expected_matching_brackets(
        &mut self,
        open_kind: SyntaxKind,
        close_kind: SyntaxKind,
        open_parsed: bool,
        open_position: usize,
    ) { ::tsox_core::fntrace::enter("parse_expected_matching_brackets"); 
        if self.token == close_kind {
            self.next_token();
            return;
        }
        self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[token_to_string(close_kind)]);
        if !open_parsed {
            return;
        }
    }

    pub(crate) fn parse_entity_name_of_type_reference(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_entity_name_of_type_reference"); 
        self.parse_entity_name_ex(true, false, Some(diagnostics::TYPE_EXPECTED))
    }

    pub(crate) fn parse_entity_name_ex(
        &mut self,
        _in_type_context: bool,
        _allow_invalid_identifiers: bool,
        _diagnostic: Option<Message>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("parse_entity_name_ex"); 
        self.parse_entity_name()
    }

    pub(crate) fn in_disallow_conditional_types_context(&self) -> bool { ::tsox_core::fntrace::enter("in_disallow_conditional_types_context"); 
        self.context_flags().intersects(NodeFlags::DisallowConditionalTypesContext)
    }

    pub(crate) fn create_identifier(&mut self, is_identifier: bool) -> Arc<Node> { ::tsox_core::fntrace::enter("create_identifier"); 
        self.create_identifier_with_diagnostic(is_identifier, None, None)
    }

    pub(crate) fn create_identifier_with_diagnostic(
        &mut self,
        is_identifier: bool,
        diagnostic_message: Option<Message>,
        private_identifier_diagnostic_message: Option<Message>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("create_identifier_with_diagnostic"); 
        if is_identifier {
            let pos = if self.scanner.has_preceding_jsdoc_leading_asterisks() {
                self.scanner.token_pos()
            } else {
                self.node_pos()
            };
            let text = self.scanner.token_value();
            self.next_token_without_check();
            let mut id = self.new_identifier(&text);
            return self.finish_node_arc(&mut id, pos);
        }
        if self.token == SyntaxKind::PrivateIdentifier {
            if let Some(message) = private_identifier_diagnostic_message {
                self.parse_error_at_current_token(message, &[]);
            } else {
                self.parse_error_at_current_token(
                    diagnostics::PRIVATE_IDENTIFIERS_ARE_NOT_ALLOWED_OUTSIDE_CLASS_BODIES,
                    &[],
                );
            }
            return self.create_identifier(true);
        }
        let report_at_current_position = self.token == SyntaxKind::EndOfFile;
        if let Some(message) = diagnostic_message {
            if report_at_current_position {
                let pos = self.scanner.full_start_pos();
                self.parse_error_at(pos, pos, message, &[]);
            } else {
                self.parse_error_at_current_token(message, &[]);
            }
        } else if crate::parser::binary_precedence::is_reserved_word_kind(self.token) {
            let text = self.scanner.token_text().to_string();
            if report_at_current_position {
                let pos = self.scanner.full_start_pos();
                self.parse_error_at(
                    pos,
                    pos,
                    diagnostics::IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE,
                    &[&text],
                );
            } else {
                self.parse_error_at_current_token(
                    diagnostics::IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE,
                    &[&text],
                );
            }
        } else if report_at_current_position {
            let pos = self.scanner.full_start_pos();
            self.parse_error_at(pos, pos, diagnostics::IDENTIFIER_EXPECTED, &[]);
        } else {
            self.parse_error_at_current_token(diagnostics::IDENTIFIER_EXPECTED, &[]);
        }
        self.missing_identifier_expression()
    }
}

pub(crate) fn create_union_or_intersection_type_node(
    p: &mut Parser,
    operator: SyntaxKind,
    types: Arc<NodeList>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("create_union_or_intersection_type_node"); 
    match operator {
        SyntaxKind::BarToken => {
            Arc::new(Node::new(
                SyntaxKind::UnionType,
                NodeData::UnionTypeNode(crate::ast::node_data_generated::UnionTypeNodeData { types }),
            ))
        }
        SyntaxKind::AmpersandToken => {
            Arc::new(Node::new(
                SyntaxKind::IntersectionType,
                NodeData::IntersectionTypeNode(
                    crate::ast::node_data_generated::IntersectionTypeNodeData { types },
                ),
            ))
        }
        _ => panic!("Unhandled case in createUnionOrIntersectionType"),
    }
}
