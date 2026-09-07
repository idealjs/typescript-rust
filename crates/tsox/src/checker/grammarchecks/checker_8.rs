#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn check_grammar_jsx_element(&mut self, node: &Arc<Node>) -> bool {
        let tag_name = match crate::checker::jsx::jsx_tag_name(node) {
            Some(t) => t,
            None => return false,
        };

        if self.check_grammar_jsx_name(&tag_name) {
            return true;
        }

        let type_args: Option<Vec<Arc<Node>>> = match &node.data {
            NodeData::JsxOpeningElement(data) => data
                .type_arguments
                .as_ref()
                .map(|l| l.iter().cloned().collect()),
            NodeData::JsxSelfClosingElement(data) => data
                .type_arguments
                .as_ref()
                .map(|l| l.iter().cloned().collect()),
            _ => None,
        };
        if let Some(args) = type_args {
            if !args.is_empty() {
                let count = args.len().to_string();
                return self.grammar_error_on_node_with_args(
                    node,
                    &EXPECTED_0_TYPE_ARGUMENTS_BUT_GOT_1,
                    &["0".to_string(), count],
                );
            }
        }

        let attrs = match crate::checker::jsx::jsx_attributes(node) {
            Some(a) => a,
            None => return false,
        };
        let properties: Vec<Arc<Node>> = match &attrs.data {
            NodeData::JsxAttributes(data) => data.properties.iter().cloned().collect(),
            _ => return false,
        };

        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for attr in &properties {
            if attr.kind == SyntaxKind::JsxSpreadAttribute {
                continue;
            }
            let (name_node, initializer) = match &attr.data {
                NodeData::JsxAttribute(d) => (Arc::clone(&d.name), d.initializer.clone()),
                _ => continue,
            };
            let text = name_node.text().to_string();
            if !seen.insert(text.clone()) {
                return self.grammar_error_on_node(
                    &name_node,
                    &JSX_ELEMENTS_CANNOT_HAVE_MULTIPLE_ATTRIBUTES_WITH_THE_SAME_NAME,
                );
            }
            if let Some(init) = initializer {
                if init.kind == SyntaxKind::JsxExpression {
                    if let NodeData::JsxExpression(d) = &init.data {
                        if d.expression.is_none() {
                            return self.grammar_error_on_node(
                                &init,
                                &JSX_ATTRIBUTES_MUST_ONLY_BE_ASSIGNED_A_NON_EMPTY_EXPRESSION,
                            );
                        }
                    }
                }
            }
        }

        false
    }

    pub fn check_grammar_jsx_name(&mut self, node: &Arc<Node>) -> bool {
        if node.kind == SyntaxKind::PropertyAccessExpression {
            if let NodeData::PropertyAccessExpression(data) = &node.data {
                let expr = &data.expression;
                if is_jsx_namespaced_name(expr) {
                    return self.grammar_error_on_node(
                        expr,
                        &JSX_PROPERTY_ACCESS_EXPRESSIONS_CANNOT_INCLUDE_JSX_NAMESPACE_NAMES,
                    );
                }
            }
        }

        if is_jsx_namespaced_name(node) && self.is_jsx_transform_enabled() {
            let namespace_text = match &node.data {
                NodeData::JsxNamespacedName(data) => data.namespace.text().to_string(),
                _ => String::new(),
            };
            if !crate::checker::jsx::is_intrinsic_jsx_name(&namespace_text) {
                return self.grammar_error_on_node(
                    node,
                    &REACT_COMPONENTS_CANNOT_INCLUDE_JSX_NAMESPACE_NAMES,
                );
            }
        }
        false
    }

    pub fn check_grammar_jsx_expression(&mut self, node: &Arc<Node>) -> bool {
        let expr = match &node.data {
            NodeData::JsxExpression(data) => &data.expression,
            _ => return false,
        };
        let Some(expr) = expr else { return false };

        if is_comma_sequence(expr) {
            return self.grammar_error_on_node(
                expr,
                &JSX_EXPRESSIONS_MAY_NOT_USE_THE_COMMA_OPERATOR_DID_YOU_MEAN_TO_WRITE_AN_ARRAY,
            );
        }
        false
    }

    pub(crate) fn is_jsx_transform_enabled(&self) -> bool {
        self.compiler_options.jsx != crate::core::compiler_options::JsxEmit::None
    }

    pub fn grammar_error_on_node_skipped_on_no_emit(
        &mut self,
        node: &Arc<Node>,
        message: &Message,
    ) -> bool {
        self.grammar_error_on_node(node, message)
    }

    pub fn check_grammar_regular_expression_literal(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_private_identifier_expression(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_mapped_type(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_decorator(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_export_declaration(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_module_element_context(
        &mut self,
        _node: &Arc<Node>,
        _error_message: &Message,
    ) -> bool {
        false
    }

    pub fn report_obvious_modifier_errors(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn find_first_modifier_except(
        &self,
        _node: &Arc<Node>,
        _allowed_modifier: SyntaxKind,
    ) -> Option<Arc<Node>> {
        None
    }

    pub fn find_first_illegal_modifier(&self, _node: &Arc<Node>) -> Option<Arc<Node>> {
        None
    }

    pub fn report_obvious_decorator_errors(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn find_first_illegal_decorator(&self, _node: &Arc<Node>) -> Option<Arc<Node>> {
        None
    }

    pub fn check_grammar_async_modifier(
        &mut self,
        _node: &Arc<Node>,
        _async_modifier: &Arc<Node>,
    ) -> bool {
        false
    }

    pub fn check_grammar_for_disallowed_trailing_comma(
        &mut self,
        _list: &crate::ast::NodeList,
        _diag: &Message,
    ) -> bool {
        false
    }

    pub fn check_grammar_type_parameter_list(
        &mut self,
        _type_parameters: &crate::ast::NodeList,
        _file: &Arc<crate::ast::SourceFile>,
    ) -> bool {
        false
    }

    pub fn check_grammar_for_use_strict_simple_parameter_list(
        &mut self,
        _node: &Arc<Node>,
    ) -> bool {
        false
    }

    pub fn check_grammar_function_like_declaration(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_class_like_declaration(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_arrow_function(
        &mut self,
        _node: &Arc<Node>,
        _file: &Arc<crate::ast::SourceFile>,
    ) -> bool {
        false
    }

    pub fn check_grammar_index_signature_parameters(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_index_signature(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_for_at_least_one_type_argument(
        &mut self,
        _node: &Arc<Node>,
        _type_arguments: &crate::ast::NodeList,
    ) -> bool {
        false
    }

    pub fn check_grammar_type_arguments(
        &mut self,
        _node: &Arc<Node>,
        _type_arguments: &crate::ast::NodeList,
    ) -> bool {
        false
    }

    pub fn check_grammar_tagged_template_chain(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_heritage_clause(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_expression_with_type_arguments(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_class_declaration_heritage_clauses(
        &mut self,
        _node: &Arc<Node>,
        _file: &Arc<crate::ast::SourceFile>,
    ) -> bool {
        false
    }

    pub fn check_grammar_interface_declaration(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_computed_property_name(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

    pub fn check_grammar_for_generator(&mut self, _node: &Arc<Node>) -> bool {
        false
    }

}
