#![allow(unused_imports)]

use crate::checker::checker_classes::*;

impl Checker {
    pub(crate) fn check_private_name_conflicts(&mut self, node: &Arc<Node>) {
        let class_node = node.parent();
        if let Some(cls) = &class_node
            && matches!(
                cls.kind,
                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
            )
            && let tsox_frontend::ast::NodeData::ClassDeclaration(cd) = &cls.data
        {
            let (my_name, my_loc) = match &node.data {
                tsox_frontend::ast::NodeData::PropertyDeclaration(d) => {
                    (d.name.text().to_string(), d.name.loc)
                }
                tsox_frontend::ast::NodeData::MethodDeclaration(d) => {
                    (d.name.text().to_string(), d.name.loc)
                }
                _ => (String::new(), node.loc),
            };
            if !my_name.is_empty() && my_name.starts_with('#') {
                let i_am_static = node.has_syntactic_modifier(ModifierFlags::Static);
                let conflict = cd.members.iter().any(|m| {
                    if m.loc.pos() >= node.loc.pos() {
                        return false;
                    }
                    let Some(mn) = m.name() else { return false };
                    mn.kind == SyntaxKind::PrivateIdentifier
                        && mn.text() == my_name
                        && m.has_syntactic_modifier(ModifierFlags::Static) != i_am_static
                });
                if conflict {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        my_loc,
                        tsox_core::diagnostics::messages_generated::
                            DUPLICATE_IDENTIFIER_0_STATIC_AND_INSTANCE_ELEMENTS_CANNOT_SHARE_THE_SAME_PRIVATE_NAME,
                        vec![my_name.clone()],
                    ));
                }
            }
        }
    }
}
