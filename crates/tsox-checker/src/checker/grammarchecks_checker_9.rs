#![allow(unused_imports)]

use crate::checker::grammarchecks::*;

impl Checker {
    pub fn check_grammar_name_in_let_or_const_declarations(&mut self, name: &Arc<Node>) -> bool {
        if name.kind == SyntaxKind::Identifier {
            if name.text() == "let" {
                return self.grammar_error_on_node(
                    name,
                    &X_LET_IS_NOT_ALLOWED_TO_BE_USED_AS_A_NAME_IN_LET_OR_CONST_DECLARATIONS,
                );
            }
        } else if let NodeData::BindingPattern(data) = &name.data {
            for element in data.elements.iter() {
                if let NodeData::BindingElement(elem) = &element.data {
                    if let Some(elem_name) = &elem.name {
                        self.check_grammar_name_in_let_or_const_declarations(elem_name);
                    }
                }
            }
        }
        false
    }

    pub fn check_grammar_for_invalid_question_mark(
        &mut self,
        _postfix_token: &Arc<Node>,
        _message: &Message,
    ) -> bool {
        false
    }

    pub fn check_grammar_for_invalid_exclamation_token(
        &mut self,
        _postfix_token: &Arc<Node>,
        _message: &Message,
    ) -> bool {
        false
    }

    pub fn check_grammar_object_literal_expression(
        &mut self,
        _node: &Arc<Node>,
        _in_destructuring: bool,
    ) -> bool {
        false
    }

    pub fn check_grammar_for_in_or_for_of_statement(&mut self, node: &Arc<Node>) -> bool {
        use tsox_core::diagnostics::messages_generated as msg;
        let tsox_frontend::ast::NodeData::ForInOrOfStatement(data) = &node.data else {
            return false;
        };
        let is_for_of = node.kind == SyntaxKind::ForOfStatement;

        if is_for_of && self.check_for_await_out_of_context(node) {
            return true;
        }

        if is_for_of
            && !node
                .flags
                .intersects(tsox_frontend::ast::node_flags::NodeFlags::AwaitContext)
            && tsox_frontend::ast::is_identifier(&data.initializer)
            && data.initializer.text() == "async"
        {
            return self.grammar_error_on_node(
                &data.initializer,
                &msg::THE_LEFT_HAND_SIDE_OF_A_FOR_OF_STATEMENT_MAY_NOT_BE_ASYNC,
            );
        }

        if data.initializer.kind != SyntaxKind::VariableDeclarationList {
            return false;
        }
        let list = Arc::clone(&data.initializer);
        if self.check_grammar_variable_declaration_list(&list) {
            return true;
        }
        let tsox_frontend::ast::NodeData::VariableDeclarationList(list_data) = &list.data else {
            return false;
        };
        let declarations = &list_data.declarations.nodes;
        if declarations.is_empty() {
            return false;
        }
        if declarations.len() > 1 {
            let diagnostic = if is_for_of {
                &msg::ONLY_A_SINGLE_VARIABLE_DECLARATION_IS_ALLOWED_IN_A_FOR_OF_STATEMENT
            } else {
                &msg::ONLY_A_SINGLE_VARIABLE_DECLARATION_IS_ALLOWED_IN_A_FOR_IN_STATEMENT
            };
            return self.grammar_error_on_first_token(&declarations[1], diagnostic);
        }
        let tsox_frontend::ast::NodeData::VariableDeclaration(first) = &declarations[0].data else {
            return false;
        };
        if let Some(initializer) = &first.initializer {
            let _ = initializer;
            let diagnostic = if is_for_of {
                &msg::THE_VARIABLE_DECLARATION_OF_A_FOR_OF_STATEMENT_CANNOT_HAVE_AN_INITIALIZER
            } else {
                &msg::THE_VARIABLE_DECLARATION_OF_A_FOR_IN_STATEMENT_CANNOT_HAVE_AN_INITIALIZER
            };
            let name = first.name.clone();
            return self.grammar_error_on_node(&name, diagnostic);
        }
        if let Some(type_node) = &first.type_node {
            let _ = type_node;
            let diagnostic = if is_for_of {
                &msg::THE_LEFT_HAND_SIDE_OF_A_FOR_OF_STATEMENT_CANNOT_USE_A_TYPE_ANNOTATION
            } else {
                &msg::THE_LEFT_HAND_SIDE_OF_A_FOR_IN_STATEMENT_CANNOT_USE_A_TYPE_ANNOTATION
            };
            let node = declarations[0].clone();
            return self.grammar_error_on_node(&node, diagnostic);
        }
        false
    }

    pub fn check_grammar_accessor(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn does_accessor_have_correct_parameter_count(&mut self, _accessor: &Arc<Node>) -> bool {
        true
    }

    pub fn check_grammar_type_operator_node(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_for_invalid_dynamic_name(
        &mut self,
        node: &Arc<Node>,
        message: &Message,
    ) -> bool {
        if !self.is_non_bindable_dynamic_name(node) {
            return false;
        }
        let expression = match &node.data {
            tsox_frontend::ast::NodeData::ElementAccessExpression(eae) => {
                Self::skip_parentheses(&eae.argument_expression)
            }
            tsox_frontend::ast::NodeData::ComputedPropertyName(d) => Arc::clone(&d.expression),
            _ => return false,
        };

        if !tsox_frontend::ast::is_entity_name_expression(&expression) {
            return self.grammar_error_on_node(node, message);
        }

        false
    }

    // Go isNonBindableDynamicName：动态名且不可迟绑定
    pub fn is_non_bindable_dynamic_name(&mut self, node: &Arc<Node>) -> bool {
        self.is_dynamic_name(node) && !self.is_late_bindable_name(node)
    }

    // Go IsDynamicName：计算名/元素访问，表达式非字面量且非有符号数字字面量
    fn is_dynamic_name(&self, name: &Arc<Node>) -> bool {
        let expr = match &name.data {
            tsox_frontend::ast::NodeData::ComputedPropertyName(d) => Arc::clone(&d.expression),
            tsox_frontend::ast::NodeData::ElementAccessExpression(d) => {
                Self::skip_parentheses(&d.argument_expression)
            }
            _ => return false,
        };
        !tsox_frontend::ast::is_string_or_numeric_literal_like(&expr)
            && !is_signed_numeric_literal(&expr)
    }

    // Go isLateBindableName：实体名表达式且其类型可作属性名
    //（字面量型或 unique symbol）
    fn is_late_bindable_name(&mut self, node: &Arc<Node>) -> bool {
        let expr = match &node.data {
            tsox_frontend::ast::NodeData::ComputedPropertyName(d) => Arc::clone(&d.expression),
            tsox_frontend::ast::NodeData::ElementAccessExpression(d) => {
                Arc::clone(&d.argument_expression)
            }
            _ => return false,
        };
        if !tsox_frontend::ast::is_entity_name_expression(&expr) {
            return false;
        }
        let t = if node.kind == SyntaxKind::ComputedPropertyName {
            self.check_computed_property_name_type(node)
        } else {
            self.get_type_of_node(&expr)
        };
        crate::checker::utilities::is_type_usable_as_property_name(&t)
    }

    pub fn check_grammar_method(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_binding_element(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_for_es_module_marker_in_binding_name(
        &mut self,
        _name: &Arc<Node>,
    ) -> bool {
        false
    }

    pub fn check_grammar_await_or_await_using(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_yield_expression(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_for_disallowed_block_scoped_variable_statement(
        &mut self,
        node: &Arc<Node>,
    ) -> bool {
        let Some(parent) = node.parent() else {
            return false;
        };
        if self.container_allows_block_scoped_variable(&parent) {
            return false;
        }
        let tsox_frontend::ast::NodeData::VariableStatement(data) = &node.data else {
            return false;
        };
        let flags = self.get_combined_node_flags(&data.declaration_list)
            & NodeFlags::BlockScoped;
        if flags.is_empty() {
            return false;
        }
        let keyword = if flags == NodeFlags::AwaitUsing {
            "await using"
        } else if flags.contains(NodeFlags::Using) {
            "using"
        } else if flags.contains(NodeFlags::Const) {
            "const"
        } else if flags.contains(NodeFlags::Let) {
            "let"
        } else {
            return false;
        };
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            self.current_file.clone(),
            node.loc,
            tsox_core::diagnostics::messages_generated::
                X_0_DECLARATIONS_CAN_ONLY_BE_DECLARED_INSIDE_A_BLOCK,
            vec![keyword.to_string()],
        ));
        true
    }

    pub fn container_allows_block_scoped_variable(&self, parent: &Arc<Node>) -> bool {
        match parent.kind {
            SyntaxKind::IfStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::WhileStatement
            | SyntaxKind::WithStatement
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement => false,
            SyntaxKind::LabeledStatement => parent
                .parent()
                .as_ref()
                .is_some_and(|p| self.container_allows_block_scoped_variable(p)),
            _ => true,
        }
    }

    pub fn check_grammar_meta_property(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_constructor_type_parameters(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_constructor_type_annotation(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_property(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_ambient_initializer(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn is_initializer_simple_literal_enum_reference(&mut self, _expr: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_numeric_literal(&mut self, _node: &Arc<Node>) {}

    pub fn check_grammar_big_int_literal(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_import_clause(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_type_only_named_imports_or_exports(
        &mut self,
        _named_bindings: &Arc<Node>,
    ) -> bool {
        false
    }

    pub fn check_grammar_import_call_expression(&mut self, _node: &Arc<Node>) -> bool {
        false
    }
}

pub(crate) fn is_comma_sequence(node: &Arc<Node>) -> bool {
    if node.kind != SyntaxKind::BinaryExpression {
        return false;
    }
    match &node.data {
        NodeData::BinaryExpression(data) => data.operator_token.kind == SyntaxKind::CommaToken,
        _ => false,
    }
}

// Go isSignedNumericLiteral：+/- 一元后随数字字面量
fn is_signed_numeric_literal(node: &Arc<Node>) -> bool {
    matches!(
        &node.data,
        NodeData::PrefixUnaryExpression(p)
            if matches!(p.operator, SyntaxKind::PlusToken | SyntaxKind::MinusToken)
                && p.operand.kind == SyntaxKind::NumericLiteral
    )
}



