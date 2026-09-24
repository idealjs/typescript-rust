#![allow(unused_imports)]

use crate::checker::checker_classes::*;

impl Checker {
    pub(crate) fn check_type_predicate(&mut self, node: &Arc<Node>) {
        let NodeData::TypePredicateNode(data) = &node.data else {
            return;
        };
        if let Some(t) = &data.type_node {
            self.check_type_annotation(t);
        }
        let Some(parent) = Self::type_predicate_parent(node) else {
            return;
        };
        let parameter_name_node = Arc::clone(&data.parameter_name);
        if Self::type_predicate_parameter_is_this(&parameter_name_node) {
            return;
        }
        let NodeData::Identifier(id) = &parameter_name_node.data else {
            return;
        };
        let parameter_name = id.text.clone();
        let parameters = Self::predicate_parent_parameters(&parent);
        if parameters.iter().any(|p| {
            p.name()
                .is_some_and(|n| n.kind == SyntaxKind::Identifier && n.text() == parameter_name)
        }) {
            return;
        }
        let mut reported_in_binding_pattern = false;
        for param in parameters.iter() {
            let NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };
            if Self::is_binding_pattern(&pd.name)
                && self.binding_pattern_declares_name(&pd.name, &parameter_name_node, &parameter_name)
            {
                reported_in_binding_pattern = true;
                break;
            }
        }
        if !reported_in_binding_pattern {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                parameter_name_node.loc,
                tsox_core::diagnostics::messages_generated::CANNOT_FIND_PARAMETER_0,
                vec![parameter_name],
            ));
        }
    }

    fn type_predicate_parent(node: &Arc<Node>) -> Option<Arc<Node>> {
        let parent = node.parent()?;
        if !matches!(
            parent.kind,
            SyntaxKind::ArrowFunction
                | SyntaxKind::CallSignature
                | SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::FunctionType
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::MethodSignature
        ) {
            return None;
        }
        if parent.type_node().is_some_and(|t| Arc::ptr_eq(t, node)) {
            Some(parent)
        } else {
            None
        }
    }

    fn type_predicate_parameter_is_this(name: &Arc<Node>) -> bool {
        matches!(name.kind, SyntaxKind::ThisKeyword | SyntaxKind::ThisType)
            || (name.kind == SyntaxKind::Identifier && name.text() == "this")
    }

    fn predicate_parent_parameters(parent: &Arc<Node>) -> &NodeList {
        match &parent.data {
            NodeData::FunctionDeclaration(d) => &d.parameters,
            NodeData::FunctionExpression(d) => &d.parameters,
            NodeData::ArrowFunction(d) => &d.parameters,
            NodeData::MethodDeclaration(d) => &d.parameters,
            NodeData::MethodSignatureDeclaration(d) => &d.parameters,
            NodeData::CallSignatureDeclaration(d) => &d.parameters,
            NodeData::FunctionTypeNode(d) => &d.parameters,
            _ => unreachable!("type predicate parent is a signature declaration"),
        }
    }

    fn binding_pattern_declares_name(
        &mut self,
        pattern: &Arc<Node>,
        predicate_node: &Arc<Node>,
        name: &str,
    ) -> bool {
        let NodeData::BindingPattern(data) = &pattern.data else {
            return false;
        };
        for element in data.elements.iter() {
            let Some(element_name) = element.name() else {
                continue;
            };
            if element_name.kind == SyntaxKind::Identifier && element_name.text() == name {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    predicate_node.loc,
                    tsox_core::diagnostics::messages_generated::
                        A_TYPE_PREDICATE_CANNOT_REFERENCE_ELEMENT_0_IN_A_BINDING_PATTERN,
                    vec![name.to_string()],
                ));
                return true;
            }
            if Self::is_binding_pattern(element_name)
                && self.binding_pattern_declares_name(element_name, predicate_node, name)
            {
                return true;
            }
        }
        false
    }
}
