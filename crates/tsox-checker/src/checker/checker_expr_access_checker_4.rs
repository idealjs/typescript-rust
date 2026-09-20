#![allow(unused_imports)]

use crate::checker::checker_expr_access::*;
use tsox_frontend::ast::is_function_like_kind;

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
        self.readonly_property_symbol(t, name)
            .is_some_and(|s| self.symbol_is_readonly(&s))
    }

    pub(crate) fn readonly_property_symbol(&self, t: &Arc<Type>, name: &str) -> Option<Arc<Symbol>> {
        if t.flags.intersects(TypeFlags::Union | TypeFlags::Intersection) {
            let parts: Vec<Arc<Type>> = t
                .types()
                .into_iter()
                .flatten()
                .cloned()
                .collect();
            let part_symbols: Vec<Option<Arc<Symbol>>> = parts
                .iter()
                .map(|p| self.readonly_property_symbol(p, name))
                .collect();
            if t.flags.contains(TypeFlags::Intersection) {
                let all_readonly = part_symbols.iter().all(|s| {
                    s.as_ref()
                        .is_some_and(|sym| self.symbol_is_readonly(sym))
                });
                return all_readonly.then(|| part_symbols.into_iter().flatten().next()).flatten();
            }
            return part_symbols
                .into_iter()
                .flatten()
                .find(|s| self.symbol_is_readonly(s));
        }
        if let Some(structured) = t.as_structured()
            && let Some(symbol) = structured.members.get(name)
        {
            return Some(Arc::clone(symbol));
        }
        let constraint = if t.flags.contains(TypeFlags::TypeParameter) {
            self.get_constraint_of_type_parameter(t)
        } else if t
            .flags
            .intersects(TypeFlags::IndexedAccess | TypeFlags::Conditional)
        {
            self.get_base_constraint_of_type(t)
        } else {
            None
        };
        constraint.and_then(|c| self.readonly_property_symbol(&c, name))
    }

    pub(crate) fn symbol_is_readonly(&self, symbol: &Arc<Symbol>) -> bool {
        for decl in &symbol.declarations {
            match &decl.data {
                tsox_frontend::ast::NodeData::PropertyDeclaration(d) => {
                    if let Some(m) = &d.modifiers
                        && m.modifier_flags.contains(ModifierFlags::Readonly)
                    {
                        return true;
                    }
                }
                tsox_frontend::ast::NodeData::PropertySignatureDeclaration(d) => {
                    if let Some(m) = &d.modifiers
                        && m.modifier_flags.contains(ModifierFlags::Readonly)
                    {
                        return true;
                    }
                }
                tsox_frontend::ast::NodeData::ParameterDeclaration(d) => {
                    if let Some(m) = &d.modifiers
                        && m.modifier_flags.contains(ModifierFlags::Readonly)
                    {
                        return true;
                    }
                }
                tsox_frontend::ast::NodeData::GetAccessorDeclaration(_) => {
                    let has_set = symbol
                        .declarations
                        .iter()
                        .any(|d| matches!(&d.data, tsox_frontend::ast::NodeData::SetAccessorDeclaration(_)));
                    if !has_set {
                        return true;
                    }
                }
                _ => continue,
            }
        }
        symbol.check_flags.contains(CheckFlags::Readonly)
    }

    pub(crate) fn is_readonly_property_write(
        &self,
        target: &Arc<Node>,
        t: &Arc<Type>,
        name: &str,
    ) -> bool {
        let Some(symbol) = self.readonly_property_symbol(t, name) else {
            return false;
        };
        if !self.symbol_is_readonly(&symbol) {
            return false;
        }
        let tsox_frontend::ast::NodeData::PropertyAccessExpression(pa) = &target.data else {
            return true;
        };
        let receiver = Self::skip_parentheses(&pa.expression);
        let is_property_decl = symbol.declarations.iter().any(|d| {
            matches!(
                &d.data,
                tsox_frontend::ast::NodeData::PropertyDeclaration(_)
                    | tsox_frontend::ast::NodeData::PropertySignatureDeclaration(_)
                    | tsox_frontend::ast::NodeData::ParameterDeclaration(_)
            )
        });
        if receiver.kind == SyntaxKind::ThisKeyword && is_property_decl {

            let container = get_control_flow_container(target);
            let Some(container) = container else {
                return true;
            };
            if container.kind != SyntaxKind::Constructor {
                return true;
            }
            let decl = symbol
                .value_declaration
                .clone()
                .or_else(|| symbol.declarations.first().cloned());
            if let Some(decl) = decl {
                let decl_parent_same = container
                    .parent()
                    .zip(decl.parent())
                    .is_some_and(|(p, q)| Arc::ptr_eq(&p, &q));
                return !decl_parent_same;
            }
        }
        true
    }
}

fn get_control_flow_container(node: &Arc<Node>) -> Option<Arc<Node>> {
    let mut current = node.parent()?;
    loop {
        let is_container = (is_function_like_kind(current.kind) && !is_immediately_invoked(&current))
            || current.kind == SyntaxKind::ModuleBlock
            || current.kind == SyntaxKind::SourceFile
            || current.kind == SyntaxKind::PropertyDeclaration;
        if is_container {
            return Some(current);
        }
        current = current.parent()?;
    }
}

fn is_immediately_invoked(func: &Arc<Node>) -> bool {
    let mut wrapper = Arc::clone(func);
    while let Some(p) = wrapper.parent()
        && p.kind == SyntaxKind::ParenthesizedExpression
    {
        wrapper = p;
    }
    wrapper.parent().is_some_and(|p| {
        matches!(&p.data, tsox_frontend::ast::NodeData::CallExpression(c) if Arc::ptr_eq(&c.expression, &wrapper))
    })
}
