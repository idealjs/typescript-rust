#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn module_member_lookup(
        &mut self,
        module_symbol: &Arc<Symbol>,
        name: &str,
    ) -> ModuleMemberLookup {
        use ModuleMemberLookup as M;

        if let Some(export_equals) = module_symbol.exports.get("export=") {
            let target = self.resolve_export_equals_target(export_equals);
            if std::env::var_os("TSOX_DEBUG_MODULE").is_some() {
                eprintln!(
                    "[mod-lookup] export= chain: module={:?} target={:?} exports={} members={}",
                    module_symbol.name,
                    target.name,
                    target.exports.len(),
                    target.members.len()
                );
            }
            if self.module_target_has_member(&target, name)
                || module_symbol.exports.get(name).is_some()
            {
                return M::Found;
            }

            if self.module_star_chain_exports(module_symbol, name)
                || (name == "default" && self.module_can_have_synthetic_default(module_symbol))
            {
                return M::Found;
            }
            return M::Missing;
        }
        if module_symbol.exports.get(name).is_some() {
            return M::Found;
        }
        if std::env::var_os("TSOX_DEBUG_MODULE").is_some() {
            eprintln!(
                "[mod-lookup] plain: name={name} exports={:?} members_with={:?} decls={:?}",
                module_symbol
                    .exports
                    .iter()
                    .take(12)
                    .map(|(k, _)| k.clone())
                    .collect::<Vec<_>>(),
                module_symbol
                    .members
                    .get(name)
                    .map(|s| (s.export_symbol.is_some(), s.flags)),
                module_symbol
                    .declarations
                    .iter()
                    .map(|d| d.kind)
                    .collect::<Vec<_>>()
            );
        }

        if self.module_has_export_clause(module_symbol, name) {
            return M::Found;
        }

        if name == "default" && self.module_has_syntactic_default(module_symbol) {
            return M::Found;
        }
        if let Some(sym) = module_symbol.members.get(name) {
            return if sym.export_symbol.is_some() {
                M::Found
            } else {
                M::LocalNotExported
            };
        }

        if let Some(file_node) = module_symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::SourceFile)
        {
            if let Some(locals) = self.program.symbol_map().locals.get(&file_node.id())
                && let Some(sym) = locals.get(name)
            {
                return if sym.export_symbol.is_some() {
                    M::Found
                } else {
                    M::LocalNotExported
                };
            }
        }

        if self.module_is_ambient_export_context(module_symbol)
            && self.module_ambient_locals_contain(module_symbol, name)
        {
            return M::Found;
        }

        if name != "default" && self.module_star_chain_exports(module_symbol, name) {
            return M::Found;
        }

        if name == "default" && self.module_can_have_synthetic_default(module_symbol) {
            return M::Found;
        }
        M::Missing
    }
}
