#![allow(unused_imports)]

use crate::checker::checker_resolve::*;
use std::sync::Arc;

impl Checker {
    pub(crate) fn ancestry_legacy_lookup(
        &self,
        a_sym: &Arc<Symbol>,
        name: &str,
        meaning: SymbolFlags,
        type_meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {

                                if !a_sym.flags.intersects(SymbolFlags::Class) {
                                    if let Some(sym) = a_sym.members.get(name) {
                                        let interface_member_visible = !a_sym
                                            .flags
                                            .intersects(SymbolFlags::Interface)
                                            || sym.flags.contains(SymbolFlags::TypeParameter);
                                        if interface_member_visible
                                            && self.meaning_hit(sym, meaning)
                                        {
                                            return Some(Arc::clone(sym));
                                        }
                                    }
                                    if a_sym
                                        .flags
                                        .intersects(SymbolFlags::MODULE | SymbolFlags::ENUM)
                                        && let Some(sym) = a_sym.exports.get(name)
                                    {
                                        let is_export_specifier = sym.flags == SymbolFlags::Alias
                                            && sym.declarations.iter().any(|d| {
                                                d.kind == SyntaxKind::ExportSpecifier
                                                    || d.kind == SyntaxKind::NamespaceExport
                                            });
                                        if !is_export_specifier && self.meaning_hit(sym, meaning)
                                        {
                                            return Some(Arc::clone(sym));
                                        }
                                    }

                                    if a_sym.flags.intersects(SymbolFlags::MODULE)
                                        && let Some(merged) = self.globals.get(a_sym.name.as_str())
                                        && !Arc::ptr_eq(merged, a_sym)
                                        && merged.flags.intersects(SymbolFlags::MODULE)
                                    {
                                        if let Some(sym) = merged.exports.get(name)
                                            && self.meaning_hit(sym, meaning)
                                        {
                                            return Some(Arc::clone(sym));
                                        }
                                        if let Some(sym) =
                                            self.ambient_namespace_local(merged, name)
                                            && self.meaning_hit(&sym, meaning)
                                        {
                                            return Some(sym);
                                        }
                                    }
                                }

                                if let Some(sym) = a_sym.members.get(name)
                                    && self.meaning_hit(sym, type_meaning)
                                {
                                    return Some(Arc::clone(sym));
                                }
        None
    }
}
