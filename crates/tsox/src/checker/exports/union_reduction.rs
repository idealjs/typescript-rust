#![allow(unused_imports)]

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnionReduction {
    #[default]
    None,
    Literal,
    Subtype,
}

pub fn get_declaration_modifier_flags_from_symbol(s: &Symbol) -> ModifierFlags {
    get_declaration_modifier_flags_from_symbol_ex(s, false)
}

pub fn get_declaration_modifier_flags_from_symbol_ex(s: &Symbol, is_write: bool) -> ModifierFlags {
    let base_decl = s
        .value_declaration
        .as_ref()
        .or_else(|| s.declarations.first());
    if let Some(value_declaration) = base_decl {
        let mut declaration: Option<&Arc<Node>> = None;
        if is_write {
            declaration = s
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::SetAccessor);
        }
        if declaration.is_none() && s.flags.contains(SymbolFlags::GetAccessor) {
            declaration = s
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::GetAccessor);
        }
        let declaration = declaration
            .map(Arc::clone)
            .unwrap_or_else(|| Arc::clone(value_declaration));
        let flags = get_combined_modifier_flags(&declaration);

        if let Some(parent) = &s.parent {
            if !parent.flags.contains(SymbolFlags::Class) {
                return flags.difference(ModifierFlags::AccessibilityModifier);
            }
        }
        return flags;
    }
    if s.check_flags.contains(CheckFlags::SYNTHETIC) {
        let access_modifier = if s.check_flags.contains(CheckFlags::ContainsPrivate) {
            ModifierFlags::Private
        } else if s.check_flags.contains(CheckFlags::ContainsPublic) {
            ModifierFlags::Public
        } else {
            ModifierFlags::Protected
        };
        let static_modifier = if s.check_flags.contains(CheckFlags::ContainsStatic) {
            ModifierFlags::Static
        } else {
            ModifierFlags::empty()
        };
        return access_modifier.union(static_modifier);
    }
    if s.flags.contains(SymbolFlags::Prototype) {
        return ModifierFlags::Public.union(ModifierFlags::Static);
    }
    ModifierFlags::empty()
}
