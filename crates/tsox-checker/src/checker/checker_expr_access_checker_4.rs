#![allow(unused_imports)]

use crate::checker::checker_expr_access::*;

impl Checker {
    pub(crate) fn is_property_readonly(&self, t: &Arc<Type>, name: &str) -> bool {
        let Some(structured) = t.as_structured() else {
            return false;
        };
        let Some(symbol) = structured.members.get(name) else {
            return false;
        };

        for decl in &symbol.declarations {
            let modifiers = match &decl.data {
                tsox_frontend::ast::NodeData::PropertyDeclaration(d) => &d.modifiers,
                tsox_frontend::ast::NodeData::PropertySignatureDeclaration(d) => &d.modifiers,
                tsox_frontend::ast::NodeData::ParameterDeclaration(d) => &d.modifiers,
                _ => continue,
            };
            if let Some(m) = modifiers {
                if m.modifier_flags.contains(ModifierFlags::Readonly) {
                    return true;
                }
            }
        }

        if symbol.check_flags.contains(CheckFlags::Readonly) {
            return true;
        }
        false
    }
}
