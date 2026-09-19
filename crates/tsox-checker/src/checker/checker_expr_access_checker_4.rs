#![allow(unused_imports)]

use crate::checker::checker_expr_access::*;

impl Checker {
    /// Go errorIfWritingToReadonlyIndex 的判定：成员不在属性表，但存在匹配
    /// 键型的 readonly 索引签名
    pub(crate) fn is_readonly_index_write(&self, t: &Arc<Type>, name: &str) -> bool {
        self.is_readonly_index_write_kind(t, name, TypeFlags::String)
    }

    pub(crate) fn is_readonly_index_write_kind(
        &self,
        t: &Arc<Type>,
        name: &str,
        key_flags: TypeFlags,
    ) -> bool {
        let Some(structured) = t.as_structured() else {
            return false;
        };
        if structured.members.get(name).is_some() {
            return false;
        }
        structured.index_infos.iter().any(|info| {
            info.is_readonly
                && info
                    .key_type
                    .as_ref()
                    .is_some_and(|k| k.flags.contains(key_flags))
        })
    }

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
