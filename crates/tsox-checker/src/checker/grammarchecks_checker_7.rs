#![allow(unused_imports)]

use crate::checker::grammarchecks::*;

impl Checker {
    pub fn check_grammar_break_or_continue_statement(&mut self, node: &Arc<Node>) -> bool {
        let target_label = match &node.data {
            NodeData::BreakStatement(data) => data.label.as_ref(),
            NodeData::ContinueStatement(data) => data.label.as_ref(),
            _ => None,
        };
        let is_break = node.kind == SyntaxKind::BreakStatement;

        let mut current = Some(Arc::clone(node));
        while let Some(cur) = current {
            if tsox_frontend::ast::is_function_like_or_class_static_block_declaration(&cur) {
                return self
                    .grammar_error_on_node(node, &JUMP_TARGET_CANNOT_CROSS_FUNCTION_BOUNDARY);
            }
            match cur.kind {
                SyntaxKind::LabeledStatement => {
                    if let Some(label) = target_label
                        && let NodeData::LabeledStatement(data) = &cur.data
                        && data.label.text() == label.text()
                    {
                        let misplaced_continue = !is_break
                            && !tsox_frontend::ast::is_iteration_statement(&data.statement, true);
                        if misplaced_continue {
                            return self.grammar_error_on_node(
                                node,
                                &A_CONTINUE_STATEMENT_CAN_ONLY_JUMP_TO_A_LABEL_OF_AN_ENCLOSING_ITERATION_STATEMENT,
                            );
                        }
                        return false;
                    }
                }
                SyntaxKind::SwitchStatement => {
                    if is_break && target_label.is_none() {
                        return false;
                    }
                }
                _ => {
                    if target_label.is_none()
                        && tsox_frontend::ast::is_iteration_statement(&cur, false)
                    {
                        return false;
                    }
                }
            }
            current = cur.parent();
        }

        let message = if target_label.is_some() {
            if is_break {
                &A_BREAK_STATEMENT_CAN_ONLY_JUMP_TO_A_LABEL_OF_AN_ENCLOSING_STATEMENT
            } else {
                &A_CONTINUE_STATEMENT_CAN_ONLY_JUMP_TO_A_LABEL_OF_AN_ENCLOSING_ITERATION_STATEMENT
            }
        } else if is_break {
            &A_BREAK_STATEMENT_CAN_ONLY_BE_USED_WITHIN_AN_ENCLOSING_ITERATION_OR_SWITCH_STATEMENT
        } else {
            &A_CONTINUE_STATEMENT_CAN_ONLY_BE_USED_WITHIN_AN_ENCLOSING_ITERATION_STATEMENT
        };
        self.grammar_error_on_node(node, message)
    }

    pub fn check_grammar_variable_declaration_list(&mut self, node: &Arc<Node>) -> bool {
        let data = match &node.data {
            NodeData::VariableDeclarationList(data) => data,
            _ => return false,
        };

        let declarations = &data.declarations;
        if declarations.is_empty() {
            return self.grammar_error_at_pos(
                node,
                declarations.pos(),
                declarations.end() - declarations.pos(),
                &VARIABLE_DECLARATION_LIST_CANNOT_BE_EMPTY,
            );
        }

        let block_scope_flags = node.flags & NodeFlags::BlockScoped;
        if block_scope_flags == NodeFlags::Using || block_scope_flags == NodeFlags::AwaitUsing {
            if let Some(parent) = node.parent() {
                if parent.kind == SyntaxKind::ForInStatement {
                    let message = if block_scope_flags == NodeFlags::Using {
                        &THE_LEFT_HAND_SIDE_OF_A_FOR_IN_STATEMENT_CANNOT_BE_A_USING_DECLARATION
                    } else {
                        &THE_LEFT_HAND_SIDE_OF_A_FOR_IN_STATEMENT_CANNOT_BE_AN_AWAIT_USING_DECLARATION
                    };
                    return self.grammar_error_on_node(node, message);
                }
            }

            if node.flags.contains(NodeFlags::Ambient) {
                let message = if block_scope_flags == NodeFlags::Using {
                    &X_USING_DECLARATIONS_ARE_NOT_ALLOWED_IN_AMBIENT_CONTEXTS
                } else {
                    &X_AWAIT_USING_DECLARATIONS_ARE_NOT_ALLOWED_IN_AMBIENT_CONTEXTS
                };
                return self.grammar_error_on_node(node, message);
            }
        }

        for decl in declarations.iter() {
            if self.check_grammar_variable_declaration(decl) {
                return true;
            }
        }

        false
    }

    pub fn check_grammar_variable_declaration(&mut self, node: &Arc<Node>) -> bool {
        let data = match &node.data {
            NodeData::VariableDeclaration(data) => data,
            _ => return false,
        };

        let node_flags = tsox_frontend::ast::utilities::get_combined_node_flags(node);
        let block_scope_kind = node_flags & NodeFlags::BlockScoped;

        if is_binding_pattern(&data.name) {
            match block_scope_kind {
                NodeFlags::AwaitUsing => {
                    return self.grammar_error_on_node_with_args(
                        node,
                        &X_0_DECLARATIONS_MAY_NOT_HAVE_BINDING_PATTERNS,
                        &["await using".to_string()],
                    );
                }
                NodeFlags::Using => {
                    return self.grammar_error_on_node_with_args(
                        node,
                        &X_0_DECLARATIONS_MAY_NOT_HAVE_BINDING_PATTERNS,
                        &["using".to_string()],
                    );
                }
                _ => {}
            }
        }

        let in_for_in_or_of = node
            .parent()
            .as_ref()
            .and_then(|p| p.parent())
            .map(|grandparent| {
                grandparent.kind == SyntaxKind::ForInStatement
                    || grandparent.kind == SyntaxKind::ForOfStatement
            })
            .unwrap_or(false);

        if !in_for_in_or_of {
            let mut anc_ambient = false;
            let mut anc = node.parent();
            while let Some(a) = anc {
                if a.has_syntactic_modifier(ModifierFlags::Ambient) {
                    anc_ambient = true;
                    break;
                }
                if matches!(
                    a.kind,
                    SyntaxKind::SourceFile
                        | SyntaxKind::Block
                        | SyntaxKind::ClassDeclaration
                        | SyntaxKind::FunctionDeclaration
                ) {
                    break;
                }
                anc = a.parent();
            }
            let in_dts = self
                .current_file
                .as_ref()
                .is_some_and(|f| f.is_declaration_file);
            if node_flags.contains(NodeFlags::Ambient) || anc_ambient || in_dts {
                self.check_ambient_initializer(node);
            } else if data.initializer.is_none() {
                if is_binding_pattern(&data.name) {
                    let parent_is_binding_pattern = node
                        .parent()
                        .as_ref()
                        .map(|p| is_binding_pattern(p))
                        .unwrap_or(false);
                    if !parent_is_binding_pattern {
                        let reported = self.grammar_error_on_node(
                            node,
                            &A_DESTRUCTURING_DECLARATION_MUST_HAVE_AN_INITIALIZER,
                        );
                        if data.type_node.is_none() {
                            // Go getTypeForBindingElement：无初始化式且无注解的
                            // 解构声明，各绑定元素经 widenTypeForVariableLikeDeclaration
                            // 报 TS7031（按位置序在 TS1182 之后）
                            self.report_implicit_any_binding_elements(&data.name);
                        }
                        return reported;
                    }
                }

                match block_scope_kind {
                    NodeFlags::AwaitUsing => {
                        return self.grammar_error_on_node_with_args(
                            node,
                            &X_0_DECLARATIONS_MUST_BE_INITIALIZED,
                            &["await using".to_string()],
                        );
                    }
                    NodeFlags::Using => {
                        return self.grammar_error_on_node_with_args(
                            node,
                            &X_0_DECLARATIONS_MUST_BE_INITIALIZED,
                            &["using".to_string()],
                        );
                    }
                    NodeFlags::Const => {
                        return self.grammar_error_on_node_with_args(
                            node,
                            &X_0_DECLARATIONS_MUST_BE_INITIALIZED,
                            &["const".to_string()],
                        );
                    }
                    _ => {}
                }
            }
        }

        if let Some(excl_token) = &data.exclamation_token {
            let parent_kind = node
                .parent()
                .as_ref()
                .and_then(|p| p.parent())
                .map(|gp| gp.kind);
            let in_variable_statement = parent_kind == Some(SyntaxKind::VariableStatement);
            let has_type = data.type_node.is_some();
            let has_initializer = data.initializer.is_some();
            let is_ambient = node_flags.contains(NodeFlags::Ambient);

            if !in_variable_statement || !has_type || has_initializer || is_ambient {
                let message = if has_initializer {
                    &DECLARATIONS_WITH_INITIALIZERS_CANNOT_ALSO_HAVE_DEFINITE_ASSIGNMENT_ASSERTIONS
                } else if !has_type {
                    &DECLARATIONS_WITH_DEFINITE_ASSIGNMENT_ASSERTIONS_MUST_ALSO_HAVE_TYPE_ANNOTATIONS
                } else {
                    &A_DEFINITE_ASSIGNMENT_ASSERTION_IS_NOT_PERMITTED_IN_THIS_CONTEXT
                };
                return self.grammar_error_on_node(excl_token, message);
            }
        }

        if !block_scope_kind.is_empty() {
            return self.check_grammar_name_in_let_or_const_declarations(&data.name);
        }

        false
    }

    pub fn check_grammar_parameter_list(
        &mut self,
        parameters: &tsox_frontend::ast::NodeList,
    ) -> bool {
        let mut seen_optional = false;
        let count = parameters.nodes.len();

        for (i, param_node) in parameters.nodes.iter().enumerate() {
            let param = match &param_node.data {
                NodeData::ParameterDeclaration(data) => data,
                _ => continue,
            };

            if param.dot_dot_dot_token.is_some() {
                if i != count - 1 {
                    if let Some(rest_token) = &param.dot_dot_dot_token {
                        let _ = self.grammar_error_on_node(
                            rest_token,
                            &A_REST_PARAMETER_MUST_BE_LAST_IN_A_PARAMETER_LIST,
                        );
                        return true;
                    }
                }
                if param.question_token.is_some() {
                    if let Some(q) = &param.question_token {
                        let _ = self.grammar_error_on_node(q, &A_REST_PARAMETER_CANNOT_BE_OPTIONAL);
                        return true;
                    }
                }
                if param.initializer.is_some() {
                    if let Some(name) = param_node.name() {
                        let _ = self.grammar_error_on_node(
                            name,
                            &A_REST_PARAMETER_CANNOT_HAVE_AN_INITIALIZER,
                        );
                        return true;
                    }
                }
            } else if is_optional_declaration(param_node) {
                seen_optional = true;

                if param.question_token.is_some()
                    && !param
                        .question_token
                        .as_ref()
                        .map(|q| q.flags.contains(NodeFlags::Reparsed))
                        .unwrap_or(false)
                    && param.initializer.is_some()
                {
                    if let Some(name) = param_node.name() {
                        let _ = self.grammar_error_on_node(
                            name,
                            &PARAMETER_CANNOT_HAVE_QUESTION_MARK_AND_INITIALIZER,
                        );
                        return true;
                    }
                }
            } else if seen_optional && param.initializer.is_none() {
                if let Some(name) = param_node.name() {
                    let _ = self.grammar_error_on_node(
                        name,
                        &A_REQUIRED_PARAMETER_CANNOT_FOLLOW_AN_OPTIONAL_PARAMETER,
                    );
                    return true;
                }
            }
        }

        false
    }
}
