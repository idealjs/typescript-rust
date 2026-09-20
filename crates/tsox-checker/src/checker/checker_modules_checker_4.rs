#![allow(unused_imports)]

use crate::checker::checker_modules::*;

impl Checker {
    pub(crate) fn module_member_lookup(
        &mut self,
        module_symbol: &Arc<Symbol>,
        name: &str,
    ) -> ModuleMemberLookup {
        use ModuleMemberLookup as M;

        // resolveJsonModule：json 模块 named exports = 顶层对象属性
        if self.module_is_json_source_file(module_symbol) {
            if let Some(t) = self.json_module_exports(module_symbol)
                && t.entries.contains_key(name)
            {
                return M::Found;
            }
        }
        if let Some(export_equals) = module_symbol.exports.get("export=") {
            let target = self.resolve_export_equals_target(export_equals);
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

impl Checker {
    pub(crate) fn module_is_json_source_file(&mut self, module_symbol: &Arc<Symbol>) -> bool {
        let file_node = module_symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::SourceFile);
        let Some(file_node) = file_node else {
            return false;
        };
        self.get_source_file_of_node(file_node)
            .is_some_and(|f| tsox_frontend::ast::is_json_source_file(&f))
    }
}
