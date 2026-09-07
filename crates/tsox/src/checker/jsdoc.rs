use std::sync::Arc;

use crate::ast::{Node, NodeData, SyntaxKind, is_binding_pattern, is_identifier};

use super::checker::Checker;

impl Checker {
    pub fn check_unmatched_jsdoc_parameters(&mut self, node: &Arc<Node>) {
        let jsdoc_parameters = self.get_all_jsdoc_parameter_tags(node);
        if jsdoc_parameters.is_empty() {
            return;
        }

        let is_js = node.flags.contains(crate::ast::NodeFlags::JavaScriptFile);

        let parameters = match &node.data {
            NodeData::FunctionDeclaration(d) => &d.parameters,
            NodeData::FunctionExpression(d) => &d.parameters,
            NodeData::ArrowFunction(d) => &d.parameters,
            NodeData::MethodDeclaration(d) => &d.parameters,
            NodeData::ConstructorDeclaration(d) => &d.parameters,
            NodeData::GetAccessorDeclaration(d) => &d.parameters,
            NodeData::SetAccessorDeclaration(d) => &d.parameters,
            NodeData::CallSignatureDeclaration(d) => &d.parameters,
            NodeData::ConstructSignatureDeclaration(d) => &d.parameters,
            NodeData::MethodSignatureDeclaration(d) => &d.parameters,
            _ => return,
        };

        let mut param_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut excluded: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for (i, param) in parameters.iter().enumerate() {
            let name = match &param.data {
                NodeData::ParameterDeclaration(d) => &d.name,
                _ => continue,
            };
            if is_identifier(name) {
                param_names.insert(name.text().to_string());
            }
            if is_binding_pattern(name) {
                excluded.insert(i);
            }
        }

        if self.contains_arguments_reference(node) {
            if is_js {
                let last_idx = jsdoc_parameters.len() - 1;
                let last_tag = &jsdoc_parameters[last_idx];
                let (last_name, last_type_expr) = match &last_tag.data {
                    NodeData::JSDocParameterOrPropertyTag(d) => (&d.name, &d.type_expression),
                    _ => return,
                };
                if !is_identifier(last_name) {
                    return;
                }
                if excluded.contains(&last_idx) || param_names.contains(last_name.text()) {
                    return;
                }
                let Some(type_expr) = last_type_expr else {
                    return;
                };
                let type_node = match &type_expr.data {
                    NodeData::JSDocTypeExpression(d) => &d.type_node,
                    _ => return,
                };
                let tag_type = self.get_type_from_type_node(type_node);
                if self.is_array_type(&tag_type) {
                    return;
                }
                self.grammar_error_on_node_with_args(
                    last_name,
                    &crate::diagnostics::messages_generated::JSDOC_PARAM_TAG_HAS_NAME_0_BUT_THERE_IS_NO_PARAMETER_WITH_THAT_NAME_IT_WOULD_MATCH_ARGUMENTS_IF_IT_HAD_AN_ARRAY_TYPE,
                    &[last_name.text().to_string()],
                );
            }
        } else {
            for (index, tag) in jsdoc_parameters.iter().enumerate() {
                let (name, is_name_first) = match &tag.data {
                    NodeData::JSDocParameterOrPropertyTag(d) => (&d.name, d.is_name_first),
                    _ => continue,
                };
                if excluded.contains(&index)
                    || (is_identifier(name) && param_names.contains(name.text()))
                {
                    continue;
                }
                if name.kind == SyntaxKind::QualifiedName {
                    if is_js {
                        let full = name.text().to_string();
                        let left = match &name.data {
                            NodeData::QualifiedName(d) => d.left.text().to_string(),
                            _ => String::new(),
                        };
                        self.grammar_error_on_node_with_args(
                            name,
                            &crate::diagnostics::messages_generated::QUALIFIED_NAME_0_IS_NOT_ALLOWED_WITHOUT_A_LEADING_PARAM_OBJECT_1,
                            &[full, left],
                        );
                    }
                } else if !is_name_first {
                    self.grammar_error_on_node_with_args(
                        name,
                        &crate::diagnostics::messages_generated::JSDOC_PARAM_TAG_HAS_NAME_0_BUT_THERE_IS_NO_PARAMETER_WITH_THAT_NAME,
                        &[name.text().to_string()],
                    );
                }
            }
        }
    }

    fn get_all_jsdoc_parameter_tags(&self, _node: &Arc<Node>) -> Vec<Arc<Node>> {
        Vec::new()
    }

    pub fn contains_arguments_reference(&self, node: &Arc<Node>) -> bool {
        let body: Option<Arc<Node>> = match &node.data {
            NodeData::FunctionDeclaration(d) => d.body.as_ref().map(Arc::clone),
            NodeData::FunctionExpression(d) => Some(Arc::clone(&d.body)),
            NodeData::MethodDeclaration(d) => d.body.as_ref().map(Arc::clone),
            NodeData::ConstructorDeclaration(d) => d.body.as_ref().map(Arc::clone),
            NodeData::GetAccessorDeclaration(d) => d.body.as_ref().map(Arc::clone),
            NodeData::SetAccessorDeclaration(d) => d.body.as_ref().map(Arc::clone),
            _ => None,
        };
        let Some(body) = body else { return false };

        let mut found = false;
        self.walk_for_arguments(&body, &mut found);
        found
    }

    fn walk_for_arguments(&self, node: &Arc<Node>, found: &mut bool) {
        if *found {
            return;
        }

        match node.kind {
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::ClassStaticBlockDeclaration => {
                return;
            }
            _ => {}
        }
        if node.kind == SyntaxKind::Identifier {
            if let NodeData::Identifier(d) = &node.data {
                if d.text == "arguments" {
                    *found = true;
                    return;
                }
            }
        }
        crate::ast::node_data_generated::for_each_child(node, |child| {
            self.walk_for_arguments(child, found);
            false
        });
    }
}
