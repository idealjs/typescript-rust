#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn check_expression(&mut self, node: &Arc<Node>) {
        self.current_node = Some(Arc::clone(node));

        self.type_instantiation_count = 0;
        match node.kind {
            SyntaxKind::Identifier => {
                self.check_identifier_reference(node);
            }
            SyntaxKind::NumericLiteral
            | SyntaxKind::StringLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::ThisKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::RegularExpressionLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral => {}
            SyntaxKind::MetaProperty => {
                let _ = self.get_type_of_node(node);
            }
            SyntaxKind::BinaryExpression => {
                self.check_binary_expression(node);
            }
            SyntaxKind::PrefixUnaryExpression => {
                if let crate::ast::NodeData::PrefixUnaryExpression(data) = &node.data {
                    self.check_expression(&data.operand);

                    if data.operator == SyntaxKind::ExclamationToken {
                        self.check_truthiness_of_type(&data.operand);
                    }

                    if matches!(
                        data.operator,
                        SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                    ) {
                        self.check_const_assignment_target(&data.operand);
                    }
                }
            }
            SyntaxKind::PostfixUnaryExpression => {
                if let crate::ast::NodeData::PostfixUnaryExpression(data) = &node.data {
                    self.check_expression(&data.operand);
                    if matches!(
                        data.operator,
                        SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                    ) {
                        self.check_const_assignment_target(&data.operand);
                    }
                }
            }
            SyntaxKind::ParenthesizedExpression => {
                if let crate::ast::NodeData::ParenthesizedExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::ClassExpression => {
                if let crate::ast::NodeData::ClassExpression(data) = &node.data {
                    self.enclosing_class_stack.push(Arc::clone(node));

                    self.push_scope(node);

                    let this_type = self.build_class_instance_type_with_base(node);
                    self.this_type_stack.push(this_type);
                    for member in data.members.iter() {
                        self.check_class_member(member);
                    }
                    self.this_type_stack.pop();
                    self.pop_scope();
                    self.enclosing_class_stack.pop();
                }
            }
            SyntaxKind::CallExpression => {
                if let crate::ast::NodeData::CallExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    for (i, arg) in data.arguments.iter().enumerate() {
                        self.check_call_arg_with_context(&data.expression, i, arg);
                    }
                }
                self.check_call_arguments(node, false);
            }
            SyntaxKind::NewExpression => {
                self.check_new_expression(node);
            }
            SyntaxKind::PropertyAccessExpression => {
                if let crate::ast::NodeData::PropertyAccessExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
                self.check_property_access(node);
            }
            SyntaxKind::ElementAccessExpression => {
                if let crate::ast::NodeData::ElementAccessExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_expression(&data.argument_expression);

                    if data.question_dot_token.is_none() {
                        let obj_type = self.get_type_of_node(&data.expression);
                        self.report_possibly_null_or_undefined(&data.expression, &obj_type, false);
                    }
                }
            }
            SyntaxKind::ConditionalExpression => {
                if let crate::ast::NodeData::ConditionalExpression(data) = &node.data {
                    self.check_expression(&data.condition);
                    self.check_truthiness_of_type(&data.condition);
                    self.check_expression(&data.when_true);
                    self.check_expression(&data.when_false);
                }
            }
            SyntaxKind::ArrayLiteralExpression => {
                if let crate::ast::NodeData::ArrayLiteralExpression(data) = &node.data {
                    for elem in data.elements.iter() {
                        self.check_expression(elem);
                    }
                }
            }
            SyntaxKind::ObjectLiteralExpression => {
                self.check_object_literal_expression(node);
            }
            SyntaxKind::ArrowFunction | SyntaxKind::FunctionExpression => {
                self.check_function_like_expression(node);
            }
            SyntaxKind::TemplateExpression => {
                if let crate::ast::NodeData::TemplateExpression(data) = &node.data {
                    for span in data.template_spans.iter() {
                        if let crate::ast::NodeData::TemplateSpan(span_data) = &span.data {
                            self.check_expression(&span_data.expression);
                        }
                    }
                }
            }
            SyntaxKind::AwaitExpression => {
                if let crate::ast::NodeData::AwaitExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::YieldExpression => {
                if let crate::ast::NodeData::YieldExpression(data) = &node.data {
                    if !self.enclosing_function_is_generator(node) {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            node.loc,
                            crate::diagnostics::messages_generated::
                                A_YIELD_EXPRESSION_IS_ONLY_ALLOWED_IN_A_GENERATOR_BODY,
                            vec![],
                        ));
                    }
                    if let Some(expr) = &data.expression {
                        self.check_expression(expr);
                    }
                }
            }
            SyntaxKind::SpreadElement => {
                if let crate::ast::NodeData::SpreadElement(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::AsExpression => {
                if let crate::ast::NodeData::AsExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_assertion_overlap(node, &data.expression, &data.type_node);

                    if Self::is_const_type_node(&data.type_node)
                        && !self.is_valid_const_assertion_argument(&data.expression)
                    {
                        let file = self.current_file.clone();
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            file,
                            data.expression.loc,
                            crate::diagnostics::messages_generated::
                                A_CONST_ASSERTION_CAN_ONLY_BE_APPLIED_TO_REFERENCES_TO_ENUM_MEMBERS_OR_STRING_NUMBER_BOOLEAN_ARRAY_OR_OBJECT_LITERALS,
                            vec![],
                        ));
                    }
                }
            }
            SyntaxKind::TypeAssertionExpression => {
                if let crate::ast::NodeData::TypeAssertion(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_assertion_overlap(node, &data.expression, &data.type_node);
                }
            }
            SyntaxKind::NonNullExpression => {
                if let crate::ast::NodeData::NonNullExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::SatisfiesExpression => {
                if let crate::ast::NodeData::SatisfiesExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::TypeOfExpression => {
                if let crate::ast::NodeData::TypeOfExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::DeleteExpression => {
                if let crate::ast::NodeData::DeleteExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                    self.check_delete_operand(&data.expression);
                }
            }
            SyntaxKind::VoidExpression => {
                if let crate::ast::NodeData::VoidExpression(data) = &node.data {
                    self.check_expression(&data.expression);
                }
            }
            SyntaxKind::TaggedTemplateExpression => {
                if let crate::ast::NodeData::TaggedTemplateExpression(data) = &node.data {
                    self.check_expression(&data.tag);
                    self.check_expression(&data.template);
                }
            }
            SyntaxKind::JsxElement
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxFragment => {
                let opening = match node.kind {
                    SyntaxKind::JsxElement => match &node.data {
                        crate::ast::NodeData::JsxElement(d) => Some(Arc::clone(&d.opening_element)),
                        _ => None,
                    },
                    SyntaxKind::JsxSelfClosingElement => Some(Arc::clone(node)),
                    SyntaxKind::JsxFragment => match &node.data {
                        crate::ast::NodeData::JsxFragment(d) => {
                            Some(Arc::clone(&d.opening_fragment))
                        }
                        _ => None,
                    },
                    _ => None,
                };
                if let Some(opening) = opening {
                    self.check_jsx_opening_like_element(&opening);
                }

                if node.kind == SyntaxKind::JsxElement {
                    if let crate::ast::NodeData::JsxElement(d) = &node.data
                        && crate::checker::jsx::is_jsx_intrinsic_tag_name(
                            &crate::checker::jsx::jsx_tag_name(&d.closing_element)
                                .unwrap_or_else(|| d.closing_element.clone()),
                        )
                    {
                        self.check_jsx_intrinsic_element(&d.closing_element);
                    }
                }
                self.check_jsx_element(node);
            }
            SyntaxKind::JsxExpression => {
                if let crate::ast::NodeData::JsxExpression(data) = &node.data {
                    self.check_grammar_jsx_expression(node);
                    if let Some(expr) = &data.expression {
                        self.check_expression(expr);
                    }
                }
            }
            _ => {
                self.walk_children_for_expressions(node);
            }
        }
        self.current_node = None;
    }
}
