use super::m4b::{is_double_quoted_string, JSDocScannerInfo};
use super::wp1_2::get_error_span_for_node;
use crate::ast::mig::m3b_2::NodeFactory;
use crate::ast::mig::m3e_3::OperatorPrecedence;
use crate::ast::{
    is_identifier, is_non_null_expression, Node, NodeData, NodeFlags, NodeList, SourceFile,
    SyntaxKind,
};
use crate::parser::parsing_context::{ParsingContext, Parser};
use crate::scanner::mig::m4d_2::is_valid_identifier;
use crate::scanner::skip_trivia;
use std::cell::Cell;
use std::sync::Arc;
use tsox_core::core::text::TextRange;
use tsox_core::core::tristate::Tristate;
use tsox_core::diagnostics;

thread_local! {
    static CONTEXT_FLAGS: Cell<u32> = const { Cell::new(0) };
    static REPARSED_CLONES: std::cell::RefCell<Vec<Arc<Node>>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

fn set_thread_context_flag(flag: NodeFlags, value: bool) {
    CONTEXT_FLAGS.with(|c| {
        let mut bits = c.get();
        if value {
            bits |= flag.bits();
        } else {
            bits &= !flag.bits();
        }
        c.set(bits);
    });
}

fn set_flag_in_arc(node: Arc<Node>, f: impl FnOnce(&mut Node)) -> Arc<Node> {
    match Arc::try_unwrap(node) {
        Ok(mut owned) => {
            f(&mut owned);
            Arc::new(owned)
        }
        Err(shared) => shared,
    }
}

impl Parser {
    pub(crate) fn source_text(&self) -> &str {
        self.scanner.text()
    }

    pub(crate) fn context_flags(&self) -> NodeFlags {
        let mut flags = NodeFlags::empty();
        if self.disallow_in_context {
            flags |= NodeFlags::DisallowInContext;
        }
        if self.yield_context {
            flags |= NodeFlags::YieldContext;
        }
        if self.decorator_context {
            flags |= NodeFlags::DecoratorContext;
        }
        if self.await_context {
            flags |= NodeFlags::AwaitContext;
        }
        if self.javascript_file {
            flags |= NodeFlags::JavaScriptFile;
        }
        flags |= NodeFlags::from_bits_truncate(CONTEXT_FLAGS.with(|c| c.get()));
        flags
    }

    pub(crate) fn do_in_context<T>(
        &mut self,
        flag: NodeFlags,
        value: bool,
        f: impl FnOnce(&mut Parser) -> T,
    ) -> T {
        let saved = CONTEXT_FLAGS.with(|c| c.get());
        set_thread_context_flag(flag, value);
        let result = f(self);
        CONTEXT_FLAGS.with(|c| c.set(saved));
        result
    }

    pub(crate) fn override_parent_in_immediate_children(&self, node: &Arc<Node>) {
        let children = [node.expression(), node.type_node(), node.name()];
        for child in children.into_iter().flatten() {
            child.set_parent(node);
        }
    }

    pub(crate) fn try_parse_async_simple_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<Arc<Node>> {
        if self.token == SyntaxKind::AsyncKeyword
            && self.look_ahead(Parser::next_is_un_parenthesized_async_arrow_function)
        {
            let pos = self.node_pos();
            let jsdoc = self.jsdoc_scanner_info();
            let async_modifier = self.parse_modifiers_for_arrow_function();
            let expr = self.parse_binary_expression_or_higher(OperatorPrecedence::Comma);
            return Some(self.parse_simple_arrow_function_expression(
                pos,
                expr,
                allow_return_type_in_arrow_function,
                jsdoc,
                async_modifier,
            ));
        }
        None
    }

    pub(crate) fn try_parse_constraint_of_infer_type(&mut self) -> Option<Arc<Node>> {
        let state = self.mark();
        if self.parse_optional(SyntaxKind::ExtendsKeyword) {
            let constraint = self.do_in_context(
                NodeFlags::DisallowConditionalTypesContext,
                true,
                Parser::parse_type,
            );
            if self.in_disallow_conditional_types_context() || self.token != SyntaxKind::QuestionToken
            {
                return Some(constraint);
            }
        }
        self.rewind(state);
        None
    }

    pub(crate) fn try_parse_constructor_declaration(
        &mut self,
        pos: usize,
        jsdoc: JSDocScannerInfo,
        modifiers: Option<Arc<crate::ast::node_node_list::ModifierList>>,
    ) -> Option<Arc<Node>> {
        let state = self.mark();
        if self.token == SyntaxKind::ConstructorKeyword
            || self.token == SyntaxKind::StringLiteral
                && self.scanner.token_value() == "constructor"
                && self.look_ahead(Parser::next_token_is_open_paren)
        {
            self.next_token();
            let type_parameters = self.parse_type_parameters();
            let parameters = self.parse_parameters();
            let return_type = self.parse_return_type(SyntaxKind::ColonToken, false);
            let body = self.parse_function_block_or_semicolon(false, false, false);
            let mut n = Node::with_loc(
                SyntaxKind::Constructor,
                NodeData::ConstructorDeclaration(crate::ast::node_data_generated::ConstructorDeclarationData {
                    modifiers,
                    type_parameters,
                    parameters,
                    type_node: return_type,
                    full_signature: None,
                    body,
                }),
                TextRange::undefined(),
            );
            let result = Arc::new(self.finish_node(&mut n, pos));
            self.with_jsdoc(&result, jsdoc);
            self.check_js_syntax(&result);
            return Some(result);
        }
        self.rewind(state);
        None
    }

    pub(crate) fn parse_modifiers_for_arrow_function(
        &mut self,
    ) -> Option<Arc<crate::ast::node_node_list::ModifierList>> {
        if self.token == SyntaxKind::AsyncKeyword {
            let pos = self.node_pos();
            self.next_token();
            let mut m = Node::new(SyntaxKind::AsyncKeyword, NodeData::Token);
            let m = Arc::new(self.finish_node(&mut m, pos));
            let nodes = vec![m];
            let flags = crate::ast::utilities_modifiers::modifiers_to_flags(&nodes);
            return Some(Arc::new(crate::ast::node_node_list::ModifierList::new(
                nodes, flags,
            )));
        }
        None
    }

    pub(crate) fn parse_binary_expression_or_higher(
        &mut self,
        precedence: OperatorPrecedence,
    ) -> Arc<Node> {
        let min_precedence = match precedence {
            OperatorPrecedence::Comma
            | OperatorPrecedence::Spread
            | OperatorPrecedence::Yield
            | OperatorPrecedence::Assignment
            | OperatorPrecedence::Conditional => 0,
            other => (other as i32 - OperatorPrecedence::LogicalOr as i32 + 1) as u8,
        };
        self.parse_binary_expression(min_precedence)
    }

    pub(crate) fn parse_simple_arrow_function_expression(
        &mut self,
        pos: usize,
        identifier: Arc<Node>,
        allow_return_type_in_arrow_function: bool,
        jsdoc: JSDocScannerInfo,
        async_modifier: Option<Arc<crate::ast::node_node_list::ModifierList>>,
    ) -> Arc<Node> {
        let identifier_pos = identifier.pos();
        let mut parameter = Node::with_loc(
            SyntaxKind::Parameter,
            NodeData::ParameterDeclaration(crate::ast::node_data_generated::ParameterDeclarationData {
                modifiers: None,
                dot_dot_dot_token: None,
                name: identifier,
                question_token: None,
                type_node: None,
                initializer: None,
            }),
            TextRange::new(identifier_pos, identifier_pos),
        );
        let parameter = Arc::new(self.finish_node(&mut parameter, identifier_pos));
        let parameters = Arc::new(NodeList {
            loc: parameter.loc,
            nodes: vec![parameter],
        });
        let equals_greater_than_token = self.create_token_node();
        self.expect(SyntaxKind::EqualsGreaterThanToken);
        let body = self.parse_arrow_function_expression_body(
            async_modifier.is_some(),
            allow_return_type_in_arrow_function,
        );
        let mut n = Node::with_loc(
            SyntaxKind::ArrowFunction,
            NodeData::ArrowFunction(crate::ast::node_data_generated::ArrowFunctionData {
                modifiers: async_modifier,
                type_parameters: None,
                parameters,
                type_node: None,
                full_signature: None,
                equals_greater_than_token,
                body,
            }),
            TextRange::undefined(),
        );
        let result = Arc::new(self.finish_node(&mut n, pos));
        self.with_jsdoc(&result, jsdoc);
        result
    }

    pub(crate) fn parse_type_parameters(&mut self) -> Option<Arc<NodeList>> {
        if self.token != SyntaxKind::LessThanToken {
            return None;
        }
        let pos = self.node_pos();
        self.next_token();
        let parsed = self.parse_delimited_list(ParsingContext::TypeParameters, Parser::parse_type_parameter);
        self.expect(SyntaxKind::GreaterThanToken);
        Some(Arc::new(NodeList {
            loc: TextRange::new(pos, self.node_pos()),
            nodes: parsed.nodes,
        }))
    }

    pub(crate) fn parse_parameters(&mut self) -> Arc<NodeList> {
        self.parse_parameter_list()
    }

    pub(crate) fn parse_return_type(
        &mut self,
        return_token: SyntaxKind,
        is_type: bool,
    ) -> Option<Arc<Node>> {
        if return_token == SyntaxKind::EqualsGreaterThanToken {
            self.expect(SyntaxKind::EqualsGreaterThanToken);
            return Some(self.parse_type_or_type_predicate());
        } else if self.parse_optional(SyntaxKind::ColonToken) {
            return Some(self.parse_type_or_type_predicate());
        } else if is_type && self.token == SyntaxKind::EqualsGreaterThanToken {
            self.parse_error_at_current_token(
                diagnostics::X_0_EXPECTED,
                &[crate::scanner::token_to_string(SyntaxKind::ColonToken)],
            );
            self.next_token();
            return Some(self.parse_type_or_type_predicate());
        }
        None
    }

    pub(crate) fn with_jsdoc(&self, node: &Arc<Node>, jsdoc: JSDocScannerInfo) {
        if jsdoc.has_jsdoc() {
            let owned = node.clone();
            let updated = set_flag_in_arc(owned, |n| n.flags |= NodeFlags::HasJSDoc);
            if Arc::ptr_eq(&updated, node) {
                return;
            }
        }
    }

    pub(crate) fn check_js_syntax(&mut self, node: &Arc<Node>) {
        if !node.flags.contains(NodeFlags::JavaScriptFile)
            || node.flags.intersects(NodeFlags::JSDoc | NodeFlags::Reparsed)
        {
            return;
        }
        if matches!(
            node.kind,
            SyntaxKind::Parameter
                | SyntaxKind::PropertyDeclaration
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::MethodSignature
                | SyntaxKind::Constructor
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor
                | SyntaxKind::FunctionExpression
                | SyntaxKind::FunctionDeclaration
                | SyntaxKind::ArrowFunction
                | SyntaxKind::VariableDeclaration
                | SyntaxKind::IndexSignature
        ) {
            if let Some(t) = node.type_node() {
                if !t.flags.intersects(NodeFlags::Reparsed) {
                    self.parse_error_at_range(
                        t.loc,
                        diagnostics::TYPE_ANNOTATIONS_CAN_ONLY_BE_USED_IN_TYPESCRIPT_FILES,
                        &[],
                    );
                }
            }
        }
    }

    pub(crate) fn try_parse_modifier(
        &mut self,
        has_seen_static_modifier: bool,
        permit_const_as_modifier: bool,
        stop_on_start_of_class_static_block: bool,
    ) -> Option<Arc<Node>> {
        let pos = self.node_pos();
        let kind = self.token;
        if self.token == SyntaxKind::ConstKeyword && permit_const_as_modifier {
            if !self
                .look_ahead(Parser::next_token_is_on_same_line_and_can_follow_modifier)
            {
                return None;
            }
            self.next_token();
        } else if stop_on_start_of_class_static_block
            && self.token == SyntaxKind::StaticKeyword
            && self.look_ahead(Parser::next_token_is_open_brace)
        {
            return None;
        } else if has_seen_static_modifier && self.token == SyntaxKind::StaticKeyword {
            return None;
        } else if !self.parse_any_contextual_modifier() {
            return None;
        }
        let mut n = Node::new(kind, NodeData::Token);
        Some(Arc::new(self.finish_node(&mut n, pos)))
    }

    pub(crate) fn try_parse_parenthesized_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<Arc<Node>> {
        let tristate = self.is_parenthesized_arrow_function_expression();
        if tristate == Tristate::False {
            return None;
        }
        if tristate == Tristate::True {
            return Some(
                self.parse_parenthesized_arrow_function_expression(true, true),
            );
        }
        let state = self.mark();
        let result =
            self.parse_possible_parenthesized_arrow_function_expression(allow_return_type_in_arrow_function);
        if result.is_none() {
            self.rewind(state);
        }
        result
    }

    pub(crate) fn parse_parenthesized_arrow_function_expression(
        &mut self,
        _allow_ambiguity: bool,
        _allow_return_type_in_arrow_function: bool,
    ) -> Arc<Node> {
        self.parse_parenthesized_arrow_function()
    }

    pub(crate) fn parse_possible_parenthesized_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<Arc<Node>> {
        let tristate = self.is_parenthesized_arrow_function_expression();
        if tristate == Tristate::True {
            return Some(self.parse_parenthesized_arrow_function_expression(
                true,
                allow_return_type_in_arrow_function,
            ));
        }
        None
    }

    pub(crate) fn try_reparse_optional_chain(&self, node: &Arc<Node>) -> bool {
        if node.flags.intersects(NodeFlags::OptionalChain) {
            return true;
        }
        if is_non_null_expression(node) {
            let mut expr = node.expression().cloned();
            while let Some(e) = expr.clone() {
                if !is_non_null_expression(&e) || e.flags.intersects(NodeFlags::OptionalChain) {
                    break;
                }
                expr = e.expression().cloned();
            }
            if let Some(e) = expr {
                if e.flags.intersects(NodeFlags::OptionalChain) {
                    let mut cur = node.clone();
                    while is_non_null_expression(&cur) {
                        cur = set_flag_in_arc(cur, |n| n.flags |= NodeFlags::OptionalChain);
                        cur = match cur.expression().cloned() {
                            Some(next) => next,
                            None => break,
                        };
                    }
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn unparse_expression_with_type_arguments(
        &self,
        expression: Option<&Arc<Node>>,
        type_arguments: Option<&NodeList>,
        result: &Arc<Node>,
    ) {
        if let Some(expression) = expression {
            expression.set_parent(result);
        }
        if let Some(type_arguments) = type_arguments {
            for a in type_arguments.nodes.iter() {
                a.set_parent(result);
            }
        }
    }

    pub(crate) fn validate_json_object_literal(
        &mut self,
        source_file: &SourceFile,
        node: &Arc<Node>,
    ) {
        for element in crate::ast::mig::m3b::properties(node).iter() {
            if element.kind != SyntaxKind::PropertyAssignment {
                let span = get_error_span_for_node(&self.source_text(), element);
                self.parse_error_at_range(span, diagnostics::PROPERTY_ASSIGNMENT_EXPECTED, &[]);
                continue;
            }
            if let Some(name) = element.name() {
                if !is_double_quoted_string(name) {
                    let span = get_error_span_for_node(&self.source_text(), name);
                    self.parse_error_at_range(
                        span,
                        diagnostics::STRING_LITERAL_WITH_DOUBLE_QUOTES_EXPECTED,
                        &[],
                    );
                }
            }
            self.validate_json_value(source_file, element.initializer());
        }
    }

    pub(crate) fn validate_json_value(
        &mut self,
        source_file: &SourceFile,
        value_expression: Option<&Arc<Node>>,
    ) {
        let Some(value_expression) = value_expression else {
            return;
        };
        match value_expression.kind {
            SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::NumericLiteral => {}
            SyntaxKind::StringLiteral => {
                if !is_double_quoted_string(value_expression) {
                    let span = get_error_span_for_node(&self.source_text(), value_expression);
                    self.parse_error_at_range(
                        span,
                        diagnostics::STRING_LITERAL_WITH_DOUBLE_QUOTES_EXPECTED,
                        &[],
                    );
                }
            }
            SyntaxKind::PrefixUnaryExpression => {
                let unary_ok = match &value_expression.data {
                    crate::ast::node_data_generated::NodeData::PrefixUnaryExpression(d) => {
                        d.operator == SyntaxKind::MinusToken
                            && d.operand.kind == SyntaxKind::NumericLiteral
                    }
                    _ => false,
                };
                if !unary_ok {
                    let span = get_error_span_for_node(&self.source_text(), value_expression);
                    self.parse_error_at_range(
                        span,
                        diagnostics::PROPERTY_VALUE_CAN_ONLY_BE_STRING_LITERAL_NUMERIC_LITERAL_TRUE_FALSE_NULL_OBJECT_LITERAL_OR_ARRAY_LITERAL,
                        &[],
                    );
                }
            }
            SyntaxKind::ObjectLiteralExpression => {
                self.validate_json_object_literal(source_file, value_expression);
            }
            SyntaxKind::ArrayLiteralExpression => {
                for element in crate::ast::mig::m3b::elements(value_expression).iter() {
                    self.validate_json_value(source_file, Some(element));
                }
            }
            _ => {
                let span = get_error_span_for_node(&self.source_text(), value_expression);
                self.parse_error_at_range(
                    span,
                    diagnostics::PROPERTY_VALUE_CAN_ONLY_BE_STRING_LITERAL_NUMERIC_LITERAL_TRUE_FALSE_NULL_OBJECT_LITERAL_OR_ARRAY_LITERAL,
                    &[],
                );
            }
        }
    }

    pub(crate) fn finish_reparsed_node(
        &self,
        node: Arc<Node>,
        location_node: &Arc<Node>,
    ) -> Arc<Node> {
        let flags = self.context_flags() | NodeFlags::Reparsed;
        let loc = location_node.loc;
        let node = set_flag_in_arc(node, |n| {
            n.flags = flags;
            n.loc = loc;
        });
        self.override_parent_in_immediate_children(&node);
        node
    }

    pub(crate) fn finish_mutated_node(&self, node: &Arc<Node>) {
        self.override_parent_in_immediate_children(node);
    }

    pub(crate) fn add_deep_clone_reparse(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let clone = crate::ast::mig::m3d::NodeFactory {
            node_count: 0,
            hooks: crate::ast::mig::m3d::NodeFactoryHooks::default(),
        }
        .deep_clone_reparse(Some(node.clone()));
        if let Some(clone) = &clone {
            REPARSED_CLONES.with(|c| c.borrow_mut().push(clone.clone()));
        }
        clone
    }

    pub(crate) fn add_transformed_reparse(
        &mut self,
        new_node: Arc<Node>,
        old: &Arc<Node>,
    ) -> Arc<Node> {
        let new_node = self.finish_reparsed_node(new_node, old);
        let new_node = set_flag_in_arc(new_node, |n| n.flags |= NodeFlags::ReparserTransformedLiteral);
        REPARSED_CLONES.with(|c| c.borrow_mut().push(new_node.clone()));
        new_node
    }

    pub(crate) fn check_non_identifier_name(&mut self, name: Option<&Arc<Node>>) -> Option<Arc<Node>> {
        let name = name?;
        let text = match &name.data {
            NodeData::Identifier(d) => d.text.as_str(),
            _ => "",
        };
        if is_identifier(name) && !is_valid_identifier(text) {
            let mut err_loc = name.loc;
            if err_loc.len() == 0 {
                err_loc = TextRange::new(name.loc.pos().saturating_sub(1), name.loc.pos());
            }
            self.parse_error_at_range(err_loc, diagnostics::IDENTIFIER_EXPECTED, &[]);
        }
        Some(name.clone())
    }

}
