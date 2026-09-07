#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn check_invalid_initializer_reference(
        &mut self,
        node: &Arc<Node>,
        name: &str,
    ) -> bool {
        if self.emit_standard_class_fields {
            return false;
        }
        let Some(parent) = node.parent.as_ref() else {
            return false;
        };
        let Some(property) = crate::ast::utilities::find_ancestor(parent, |n| {
            n.kind == SyntaxKind::PropertyDeclaration
        }) else {
            return false;
        };

        if let Some(sym) = self.resolve_identifier(node) {
            let binds_in_initializer_fn = sym.declarations.iter().any(|d| {
                let mut cur = d.parent.as_ref();
                while let Some(a) = cur {
                    if Arc::ptr_eq(a, &property) {
                        return false;
                    }
                    if matches!(
                        a.kind,
                        SyntaxKind::FunctionDeclaration
                            | SyntaxKind::FunctionExpression
                            | SyntaxKind::ArrowFunction
                    ) {
                        return true;
                    }
                    cur = a.parent.as_ref();
                }
                false
            });
            if binds_in_initializer_fn {
                return false;
            }
        }
        if property.has_syntactic_modifier(ModifierFlags::Static) {
            return false;
        }
        let Some(class) = property.parent.as_ref() else {
            return false;
        };
        if !matches!(
            class.kind,
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
        ) {
            return false;
        }

        let crate::ast::NodeData::ClassDeclaration(cd) = &class.data else {
            return false;
        };
        let ctor = cd.members.iter().find(|m| {
            m.kind == SyntaxKind::Constructor
                && matches!(&m.data, crate::ast::NodeData::ConstructorDeclaration(d) if d.body.is_some())
        });
        let Some(ctor) = ctor else {
            return false;
        };
        let symbol_map = self.program.symbol_map();
        let ctor_has_name = symbol_map.locals.get(&ctor.id()).is_some_and(|locals| {
            locals
                .get(name)
                .is_some_and(|sym| sym.flags.intersects(SymbolFlags::VALUE))
        });
        if !ctor_has_name {
            return false;
        }
        let file = self.current_file.clone();
        let property_name = property
            .name()
            .map(|n| n.text().to_string())
            .unwrap_or_default();
        self.diagnostics.add(crate::ast::Diagnostic::new(
            file,
            node.loc,
            crate::diagnostics::messages_generated::
                INITIALIZER_OF_INSTANCE_MEMBER_VARIABLE_0_CANNOT_REFERENCE_IDENTIFIER_1_DECLARED_IN_THE_CONSTRUCTOR,
            vec![property_name, name.to_string()],
        ));
        true
    }
}
