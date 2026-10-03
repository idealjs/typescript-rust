#![allow(unused_imports)]

use crate::checker::checker_statements::*;

impl Checker {
    pub fn check_return_statement(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_return_statement"); 
        let container = crate::checker::utilities_get_assignment_target::
            get_containing_function_or_class_static_block(node);
        if container
            .as_ref()
            .is_some_and(|c| c.kind == SyntaxKind::ClassStaticBlockDeclaration)
        {
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                node.loc,
                tsox_core::diagnostics::messages_generated::
                    A_RETURN_STATEMENT_CANNOT_BE_USED_INSIDE_A_CLASS_STATIC_BLOCK,
                Vec::new(),
            ));
            return;
        }
        if self.function_scope_count == 0 && self.arrow_function_scope_count == 0 {
            if self
                .current_file
                .as_ref()
                .is_some_and(|f| f.has_parse_diagnostics)
            {
                // Go checkReturnStatement：表达式先于一切容器检查（错位 return 也要
                // 解析标识符）；hasParseDiagnostics 只抑制 TS1109（grammarErrorOnFirstToken）
                if let tsox_frontend::ast::NodeData::ReturnStatement(data) = &node.data
                    && let Some(expr) = &data.expression
                {
                    self.check_expression(expr);
                }
                return;
            }
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    node.loc,
                    tsox_core::diagnostics::messages_generated::
                        A_RETURN_STATEMENT_CAN_ONLY_BE_USED_WITHIN_A_FUNCTION_BODY,
                    Vec::new(),
                ));
        }
        if let tsox_frontend::ast::NodeData::ReturnStatement(data) = &node.data {
            // Go unwrapReturnType：生成器 return 比对注解返回型的 TReturn
            //（async 生成器再 awaited），而非整个注解类型；容器缺失
            //（accessor/构造器等非生成器容器）时透传期望型
            let unwrap_if_generator = |c: &mut Self, expected: Option<Arc<Type>>| -> Option<Arc<Type>> {
                let expected = expected?;
                match c.yield_container_of(node) {
                    Some(container) if container.is_generator => {
                        Some(c.unwrap_generator_return_type(&expected, container.is_async))
                    }
                    _ => Some(expected),
                }
            };
            if let Some(expr) = &data.expression {
                self.check_expression(expr);

                // Go checkReturnStatement：构造器容器走独立分支，返回型取
                // 构造器签名的实例型（注解栈为空），不可赋值即级联 TS2409
                if container
                    .as_ref()
                    .is_some_and(|c| c.kind == SyntaxKind::Constructor)
                {
                    let expected =
                        self.container_instance_type_of(container.as_ref().unwrap());
                    let actual = self.get_type_of_node(expr);
                    if !self.check_type_related_to_and_optionally_elaborate(
                        &actual,
                        &expected,
                        crate::checker::relater::RelationKind::Assignable,
                        Some(node),
                        Some(expr),
                        None,
                        None,
                    ) {
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            node.loc,
                            tsox_core::diagnostics::messages_generated::
                                RETURN_TYPE_OF_CONSTRUCTOR_SIGNATURE_MUST_BE_ASSIGNABLE_TO_THE_INSTANCE_TYPE_OF_THE_CLASS,
                            Vec::new(),
                        ));
                    }
                } else if let Some(expected) = unwrap_if_generator(
                    self,
                    self.return_type_stack.last().and_then(|opt| opt.clone()),
                ) {
                    // Go checkReturnExpression：返回表达式（剥括号）为条件表达式时
                    // 按分支逐个对返回型比较，错误锚定分支节点，不再整体比较
                    let unwrapped = Checker::skip_parentheses(expr);
                    if matches!(
                        &unwrapped.data,
                        tsox_frontend::ast::NodeData::ConditionalExpression(_)
                    ) {
                        self.check_return_expression_against_type(&expected, node, expr, false, true);
                    } else {
                        // Go checkReturnExpression：async 容器返回表达式先取
                        // awaited 型，thenable 报 TS1058 并按 errorType 参与比对
                        let raw_actual = self.get_type_of_node(expr);
                        let container_async = container
                            .as_ref()
                            .is_some_and(|c| c.has_syntactic_modifier(ModifierFlags::Async));
                        let actual = if container_async {
                            match self.check_awaited_type_no_alias(
                                &raw_actual,
                                Some(node),
                                tsox_core::diagnostics::messages_generated::
                                    THE_RETURN_TYPE_OF_AN_ASYNC_FUNCTION_MUST_EITHER_BE_A_VALID_PROMISE_OR_MUST_NOT_CONTAIN_A_CALLABLE_THEN_MEMBER,
                            ) {
                                Some(t) => t,
                                None => self.get_error_type(),
                            }
                        } else {
                            raw_actual
                        };

                        if !actual.flags.contains(TypeFlags::Any)
                            && !self.is_type_assignable_to(&actual, &expected)
                        {
                            let display_type = if crate::checker::is_literal_type(&actual) {
                                self.get_base_type_of_literal_type(&actual)
                            } else {
                                actual.clone()
                            };
                            let ok = self.check_type_related_to_and_optionally_elaborate(
                                &display_type,
                                &expected,
                                crate::checker::relater::RelationKind::Assignable,
                                Some(node),
                                Some(expr),
                                None,
                                None,
                            );
                            if ok {}
                        }
                    }
                }
            } else {
                let annotated = self.return_type_stack.last().and_then(|opt| opt.clone());
                let expected = unwrap_if_generator(self, annotated.clone());
                self.check_bare_return_statement(node, container.as_ref(), expected);
            }
        }
    }
}
