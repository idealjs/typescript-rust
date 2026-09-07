#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_class_member(&mut self, node: &Arc<Node>) {
        self.check_grammar_modifiers(node);

        if node.kind == SyntaxKind::Constructor {
            self.check_multiple_constructor_implementations(node);
        }

        self.check_private_name_conflicts(node);

        match node.kind {
            SyntaxKind::PropertyDeclaration => {
                if let crate::ast::NodeData::PropertyDeclaration(data) = &node.data {
                    self.check_computed_property_name(&data.name);

                    if node.has_syntactic_modifier(ModifierFlags::Abstract)
                        && data.initializer.is_some()
                    {
                        self.diagnostics.add(crate::ast::Diagnostic::new(
                            self.current_file.clone(),
                            data.name.loc,
                            crate::diagnostics::messages_generated::
                                PROPERTY_0_CANNOT_HAVE_AN_INITIALIZER_BECAUSE_IT_IS_MARKED_ABSTRACT,
                            vec![data.name.text().to_string()],
                        ));
                    }

                    if node.has_syntactic_modifier(ModifierFlags::Static) {
                        if let Some(type_node) = &data.type_node {
                            let prev = self.in_static_member_type;
                            self.in_static_member_type = true;
                            let _ = self.get_type_from_type_node(type_node);
                            self.in_static_member_type = prev;
                        }
                    }
                    if let Some(init) = &data.initializer {
                        let is_static = node.has_syntactic_modifier(ModifierFlags::Static);
                        self.this_container_stack.push(if is_static {
                            ThisContainerKind::StaticMember
                        } else {
                            ThisContainerKind::InstanceMember
                        });
                        self.check_expression(init);
                        self.this_container_stack.pop();

                        if let Some(tn) = &data.type_node {
                            let target = self.get_type_from_type_node(tn);
                            let anchor = data.name.loc;
                            self.check_contextual_elements(init, &target, anchor);
                        }
                    }
                }
            }
            SyntaxKind::PropertySignature => {
                if let crate::ast::NodeData::PropertySignatureDeclaration(data) = &node.data {
                    self.check_computed_property_name(&data.name);
                }
            }
            SyntaxKind::ClassStaticBlockDeclaration => {
                if let crate::ast::NodeData::ClassStaticBlockDeclaration(data) = &node.data {
                    self.this_container_stack
                        .push(ThisContainerKind::StaticMember);
                    self.check_statement(&data.body);
                    self.this_container_stack.pop();
                }
            }
            SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => {
                self.check_class_accessor_member(node);
            }
            _ => {}
        }
    }
}
