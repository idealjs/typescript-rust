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
                || self.target_type_has_property(&target, name)
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
    // Go checkExportDeclaration 的 export * / export * as ns 分支：解析模块
    // 命中且 exports 表含 export= 条目（hasExportAssignmentSymbol）时在
    // module specifier 上报 TS2498
    pub(crate) fn check_export_star_export_equals(&mut self, spec: &Arc<Node>) {
        if spec.kind != SyntaxKind::StringLiteral {
            return;
        }
        let Some(file) = self.current_file.clone() else {
            return;
        };
        let spec_text = spec.text().trim_matches(['"', '\'', '`']).to_string();
        let file_symbol = |checker: &Self| {
            checker
                .program
                .resolve_external_module_path(
                    &spec_text,
                    &file.file_name,
                    tsox_core::core::compiler_options::ModuleKind::None,
                )
                .and_then(|path| {
                    let sf = checker.program.get_source_file(&path)?;
                    checker.program.symbol_map().symbol_of(&sf.node).cloned()
                })
        };
        let module_symbol = if !spec_text.starts_with('.') && !spec_text.starts_with("..") {
            self.resolve_module_file_symbol(&spec_text)
                .or_else(|| file_symbol(self))
        } else {
            file_symbol(self)
        };
        let Some(module_symbol) = module_symbol else {
            return;
        };
        if module_symbol.exports.get("export=").is_some() {
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                Some(file),
                spec.loc,
                tsox_core::diagnostics::messages_generated::
                    MODULE_0_USES_EXPORT_AND_CANNOT_BE_USED_WITH_EXPORT_ASTERISK,
                vec![format!("\"{spec_text}\"")],
            ));
        }
    }

    // Go getExternalModuleMember：export= 模块的具名成员解析优先取
    // resolved 目标符号类型上的属性（var Foo: {a,b}; export = Foo）
    fn target_type_has_property(&mut self, target: &Arc<Symbol>, name: &str) -> bool {
        let t = self.get_type_of_symbol(target);
        self.get_property_of_type(&t, name).is_some()
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
