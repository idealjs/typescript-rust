#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_function_like_body(&mut self, node: &Arc<Node>) {
        self.get_type_of_node(node);

        self.in_ctor_body_stack.push(false);
        let (body, type_node): (Option<Arc<Node>>, Option<Arc<Node>>) = match &node.data {
            crate::ast::NodeData::FunctionExpression(data) => {
                (Some(data.body.clone()), data.type_node.clone())
            }
            crate::ast::NodeData::ArrowFunction(data) => {
                (Some(data.body.clone()), data.type_node.clone())
            }
            _ => (None, None),
        };
        if let Some(body) = body {
            let is_arrow = matches!(node.data, crate::ast::NodeData::ArrowFunction(_));
            if is_arrow {
                self.push_arrow_function_scope(node);
            } else {
                self.push_function_scope(node);
            }

            let is_async = node.has_syntactic_modifier(ModifierFlags::Async);
            let declared_return = type_node
                .as_ref()
                .map(|tn| self.get_type_from_type_node(tn))
                .map(|t| self.unwrap_async_return_type(t, is_async));
            self.return_type_stack.push(declared_return);
            match body.kind {
                SyntaxKind::Block => self.check_statement(&body),
                _ => {
                    self.check_expression(&body);
                    if let Some(expected) =
                        self.return_type_stack.last().and_then(|opt| opt.clone())
                    {
                        let actual = self.get_type_of_node(&body);
                        if !actual.flags.contains(TypeFlags::Any)
                            && !self.is_type_assignable_to(&actual, &expected)
                        {
                            let actual_str = self.type_to_string(&actual);
                            let expected_str = self.type_to_string(&expected);
                            self.diagnostics.add(crate::ast::Diagnostic::new(
                                self.current_file.clone(),
                                body.loc,
                                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                                vec![actual_str, expected_str],
                            ));
                        }
                    }
                }
            }
            self.return_type_stack.pop();
            self.in_ctor_body_stack.pop();
            if is_arrow {
                self.pop_arrow_function_scope();
            } else {
                self.pop_function_scope();
            }
        }
    }

    pub(crate) fn walk_children_for_expressions(&mut self, node: &Arc<Node>) {
        let children: Vec<Arc<Node>> = {
            let mut collected = Vec::new();
            crate::ast::node_data_generated::for_each_child(node, |child| {
                collected.push(Arc::clone(child));
                false
            });
            collected
        };
        for child in &children {
            if is_expression_position_kind(child.kind) {
                self.check_expression(child);
            } else if is_statement_kind(child.kind) {
                self.check_statement(child);
            }
        }
    }

    pub(crate) fn check_jsx_element(&mut self, node: &Arc<Node>) {
        let opening_element: Option<Arc<Node>> = match &node.data {
            crate::ast::NodeData::JsxElement(data) => Some(Arc::clone(&data.opening_element)),
            crate::ast::NodeData::JsxSelfClosingElement(_) => Some(Arc::clone(node)),
            _ => None,
        };
        let children: Vec<Arc<Node>> = match &node.data {
            crate::ast::NodeData::JsxElement(data) => data.children.iter().cloned().collect(),
            crate::ast::NodeData::JsxFragment(data) => data.children.iter().cloned().collect(),
            _ => Vec::new(),
        };

        if let Some(opening) = opening_element {
            let attributes: Option<Arc<Node>> = match &opening.data {
                crate::ast::NodeData::JsxOpeningElement(data) => Some(Arc::clone(&data.attributes)),
                crate::ast::NodeData::JsxSelfClosingElement(data) => {
                    Some(Arc::clone(&data.attributes))
                }
                _ => None,
            };
            if let Some(attrs) = attributes {
                if let crate::ast::NodeData::JsxAttributes(data) = &attrs.data {
                    for attr in data.properties.iter() {
                        self.check_jsx_attribute(attr);
                    }
                }
            }
        }

        for child in &children {
            self.check_jsx_child(child);
        }
    }

    pub(crate) fn check_jsx_attribute(&mut self, node: &Arc<Node>) {
        match &node.data {
            crate::ast::NodeData::JsxAttribute(data) => {
                if let Some(init) = &data.initializer {
                    self.check_expression(init);
                }
            }
            crate::ast::NodeData::JsxSpreadAttribute(data) => {
                self.check_expression(&data.expression);
            }
            _ => {}
        }
    }

    pub(crate) fn check_jsx_child(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::JsxElement
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxFragment => {
                self.check_expression(node);
            }
            SyntaxKind::JsxExpression => {
                self.check_expression(node);
            }

            _ => {}
        }
    }

    pub(crate) fn cannot_find_name_message_for(
        name: &str,
    ) -> Option<&'static crate::diagnostics::Message> {
        use crate::diagnostics::messages_generated as mg;
        match name {
            "document" | "console" => Some(
                &mg::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_CHANGE_YOUR_TARGET_LIBRARY_TRY_CHANGING_THE_LIB_COMPILER_OPTION_TO_INCLUDE_DOM,
            ),
            "process" | "require" | "Buffer" | "module" | "NodeJS" => Some(
                &mg::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_NODE_TRY_NPM_I_SAVE_DEV_TYPES_SLASHNODE_AND_THEN_ADD_NODE_TO_THE_TYPES_FIELD_IN_YOUR_TSCONFIG,
            ),
            _ => None,
        }
    }

    pub(crate) fn check_parameter_default_initializer(&mut self, param: &Arc<Node>) {
        if let crate::ast::NodeData::ParameterDeclaration(pd) = &param.data
            && let Some(init) = &pd.initializer
        {
            self.check_expression(init);
        }
    }
}
