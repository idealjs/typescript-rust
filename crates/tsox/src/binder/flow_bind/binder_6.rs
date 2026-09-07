#![allow(unused_imports)]

use super::*;

impl Binder {
    pub(crate) fn bind_assignment_target_flow(&mut self, node: &Arc<Node>) {
        match &node.data {
            NodeData::ArrayLiteralExpression(arr) => {
                for e in &arr.elements.nodes {
                    if e.kind == SyntaxKind::SpreadElement {
                        if let Some(inner) = e.expression() {
                            self.bind_assignment_target_flow(&inner);
                        }
                    } else {
                        self.bind_destructuring_target_flow(e);
                    }
                }
            }
            NodeData::ObjectLiteralExpression(obj) => {
                for p in &obj.properties.nodes {
                    match &p.data {
                        NodeData::PropertyAssignment(pa) => {
                            self.bind_destructuring_target_flow(&pa.initializer);
                        }
                        NodeData::ShorthandPropertyAssignment(sa) => {
                            self.bind_assignment_target_flow(&sa.name);
                        }
                        NodeData::SpreadAssignment(sp) => {
                            self.bind_assignment_target_flow(&sp.expression);
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                if self.is_mutation_tracked_reference(node)
                    && matches!(
                        node.kind,
                        SyntaxKind::Identifier
                            | SyntaxKind::PropertyAccessExpression
                            | SyntaxKind::ElementAccessExpression
                            | SyntaxKind::ParenthesizedExpression
                            | SyntaxKind::NonNullExpression
                            | SyntaxKind::ThisKeyword
                            | SyntaxKind::SuperKeyword
                            | SyntaxKind::MetaProperty
                    )
                {
                    if let Some(current) = self.current_flow.take() {
                        let assign_flow = self.create_flow_assignment(&current, node);
                        self.current_flow = Some(assign_flow);
                    }
                }
            }
        }
    }

    pub(crate) fn bind_destructuring_target_flow(&mut self, node: &Arc<Node>) {
        if let NodeData::BinaryExpression(bin) = &node.data {
            if bin.operator_token.kind == SyntaxKind::EqualsToken {
                self.bind_assignment_target_flow(&bin.left);
                return;
            }
        }
        self.bind_assignment_target_flow(node);
    }

    pub(crate) fn bind_initialized_variable_flow(&mut self, node: &Arc<Node>) {
        let name = match &node.data {
            NodeData::VariableDeclaration(d) => Some(Arc::clone(&d.name)),
            NodeData::BindingElement(d) => d.name.clone(),
            _ => None,
        };
        let Some(name) = name else { return };
        if matches!(
            name.kind,
            SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
        ) {
            if let NodeData::BindingPattern(pattern) = &name.data {
                for child in &pattern.elements.nodes {
                    self.bind_initialized_variable_flow(child);
                }
            }
            return;
        }
        if let Some(current) = self.current_flow.take() {
            let assign_flow = self.create_flow_assignment(&current, node);
            self.symbol_map
                .set_flow_node(node, Arc::clone(&assign_flow));
            self.current_flow = Some(assign_flow);
        }
    }

    pub(crate) fn check_contextual_identifier(&mut self, node: &Arc<Node>) {
        let Some(file) = self.current_source_file.clone() else {
            return;
        };
        if file.has_parse_diagnostics
            || node.flags.contains(NodeFlags::Ambient)
            || node.flags.contains(NodeFlags::JSDoc)
            || is_identifier_name(node)
            || file.is_declaration_file
        {
            return;
        }

        {
            let mut anc = node.parent.as_ref();
            while let Some(a) = anc {
                if a.has_syntactic_modifier(ModifierFlags::Ambient) {
                    return;
                }
                anc = a.parent.as_ref();
            }
        }
        let Some(kind) = crate::scanner::string_to_keyword(node.text()) else {
            return;
        };
        let is_future_reserved = matches!(
            kind,
            SyntaxKind::ImplementsKeyword
                | SyntaxKind::InterfaceKeyword
                | SyntaxKind::LetKeyword
                | SyntaxKind::PackageKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::ProtectedKeyword
                | SyntaxKind::PublicKeyword
                | SyntaxKind::StaticKeyword
                | SyntaxKind::YieldKeyword
        );
        let message = if is_future_reserved {
            if crate::ast::utilities::get_containing_class(node).is_some() {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_CLASS_DEFINITIONS_ARE_AUTOMATICALLY_IN_STRICT_MODE
            } else if file.external_module_indicator.is_some() {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE
            } else {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE
            }
        } else if kind == SyntaxKind::AwaitKeyword {
            if file.external_module_indicator.is_some()
                && crate::ast::utilities::is_in_top_level_context(node)
            {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_AT_THE_TOP_LEVEL_OF_A_MODULE
            } else if node.flags.contains(NodeFlags::AwaitContext) {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE
            } else {
                return;
            }
        } else if kind == SyntaxKind::YieldKeyword && node.flags.contains(NodeFlags::YieldContext) {
            IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE
        } else {
            return;
        };
        self.symbol_map.binder_diagnostics.push(Diagnostic::new(
            Some(file),
            node.loc,
            message,
            vec![node.text().to_string()],
        ));
    }
}
