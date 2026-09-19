#![allow(unused_imports)]

use crate::checker::checker_imports_namespace::*;

impl Checker {
    pub(crate) fn namespace_has_value_side(&mut self, namespace: &Arc<Symbol>) -> bool {
        let value_flags = SymbolFlags::Function
            | SymbolFlags::Class
            | SymbolFlags::FunctionScopedVariable
            | SymbolFlags::BlockScopedVariable
            | SymbolFlags::RegularEnum
            | SymbolFlags::ConstEnum
            | SymbolFlags::Method;
        let has_value_member = |table: &tsox_frontend::ast::SymbolTable| {
            table.iter().any(|(name, s)| {
                name != "export="
                    && s.flags.intersects(value_flags)
                    && s.declarations.iter().any(|d| {
                        !matches!(d.kind, SyntaxKind::Parameter | SyntaxKind::MethodSignature)
                    })
            })
        };
        if has_value_member(&namespace.exports) || has_value_member(&namespace.members) {
            return true;
        }

        for d in &namespace.declarations {
            if d.kind != SyntaxKind::ModuleDeclaration {
                continue;
            }
            let entries: Vec<(String, Arc<Symbol>)> = self
                .program
                .symbol_map()
                .locals
                .get(&d.id())
                .map(|table| {
                    table
                        .iter()
                        .map(|(k, v)| (k.clone(), Arc::clone(v)))
                        .collect()
                })
                .unwrap_or_default();
            if entries.iter().any(|(name, s)| {
                name != "export="
                    && (s.flags.intersects(value_flags)
                        || (s.flags.contains(SymbolFlags::ValueModule)
                            && self.namespace_has_value_side(s)))
            }) {
                return true;
            }
        }

        if self.namespace_value_depth < 4 {
            self.namespace_value_depth += 1;
            let nested = namespace
                .exports
                .iter()
                .chain(namespace.members.iter())
                .any(|(name, s)| {
                    name != "export="
                        && s.flags.contains(SymbolFlags::ValueModule)
                        && self.namespace_has_value_side(s)
                });
            self.namespace_value_depth -= 1;
            if nested {
                return true;
            }
        }

        for decl in &namespace.declarations {
            if decl.kind == SyntaxKind::ModuleDeclaration
                && let Some(locals) = self.program.symbol_map().locals.get(&decl.id())
                && locals
                    .iter()
                    .any(|(name, s)| name != "export=" && s.flags.intersects(value_flags))
            {
                return true;
            }
        }

        if let Some(export_equals) = namespace.exports.get("export=") {
            for decl in &export_equals.declarations {
                if let tsox_frontend::ast::NodeData::ExportAssignment(ea) = &decl.data
                    && ea.is_export_equals
                    && matches!(
                        ea.expression.kind,
                        SyntaxKind::Identifier | SyntaxKind::QualifiedName
                    )
                {
                    let scope_decl = namespace
                        .declarations
                        .iter()
                        .find(|d| d.kind == SyntaxKind::ModuleDeclaration)
                        .cloned();
                    if let Some(scope_decl) = scope_decl {
                        self.push_scope(&scope_decl);
                        let target = self.resolve_qualified_symbol(&ea.expression);
                        self.pop_scope();
                        if let Some(target) = target {
                            if target.flags.intersects(value_flags) {
                                return true;
                            }
                            if target.flags.contains(SymbolFlags::ValueModule) {
                                return self.namespace_has_value_side(&target);
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Go getTargetOfModuleDefault：default 导出符号解析；export default <entity>
    /// 的 Alias 跟随其实体目标，export= 模块回落合成 default（resolveExternalModuleSymbol）
    pub(crate) fn resolve_default_export_target(
        &mut self,
        module_sym: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> {
        let mut target = self.resolve_module_member_symbol(module_sym, "default", 8);
        if target.is_none() {
            let export_eq = module_sym
                .exports
                .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
                .cloned();
            if let Some(ee) = export_eq {
                let resolved = self.resolve_export_equals_target(&ee);
                if resolved.flags.intersects(SymbolFlags::VALUE) {
                    target = Some(resolved);
                }
            }
        }
        for _ in 0..4 {
            let cur = target.clone()?;
            let is_default_alias = cur.flags.contains(SymbolFlags::Alias)
                && cur.declarations.iter().any(|d| {
                    matches!(&d.data, NodeData::ExportAssignment(ea) if !ea.is_export_equals)
                });
            if !is_default_alias {
                break;
            }
            match self.resolve_export_assignment_target(&cur) {
                Some(next) if !Arc::ptr_eq(&next, &cur) => target = Some(next),
                _ => break,
            }
        }
        target
    }

    pub(crate) fn resolve_module_member_symbol(
        &mut self,
        module_sym: &Arc<Symbol>,
        name: &str,
        depth: usize,
    ) -> Option<Arc<Symbol>> {
        if depth == 0 {
            return None;
        }
        let sym = self.namespace_member_recursive(module_sym, name);
        if let Some(sym) = &sym {
            if let Some(target) = &sym.export_symbol
                && !Arc::ptr_eq(target, &sym)
            {
                return Some(Arc::clone(target));
            }
        }
        // export= 模块（module.exports = foo）的具名导入：目标在 export= 值的
        // 属性上（Go tryGetMemberInModuleExportsAndProperties）
        if sym.is_none()
            && let Some(target) = self.try_get_member_in_module_exports_and_properties(name, module_sym)
        {
            return Some(target);
        }

        let mut clause_hits: Vec<(String, Option<String>)> = Vec::new();
        let mut ns_export_target: Option<Arc<Symbol>> = None;
        let mut star_specs: Vec<String> = Vec::new();
        self.for_each_module_statement(module_sym, |stmt| {
            if let tsox_frontend::ast::NodeData::ExportDeclaration(d) = &stmt.data
                && d.export_clause.is_none()
                && let Some(spec) = d.module_specifier.as_ref()
            {
                star_specs.push(spec.text().trim_matches(['"', '\'', '`']).to_string());
            }
            if let tsox_frontend::ast::NodeData::ExportDeclaration(d) = &stmt.data
                && let Some(clause) = &d.export_clause
                && clause.kind == SyntaxKind::NamespaceExport
                && tsox_frontend::ast::node_data_generated::node_name(clause)
                    .is_some_and(|n| n.text() == name)
                && let Some(spec) = d.module_specifier.as_ref()
            {
                // export * as a from '...'：成员名映射目标模块符号（命名空间对象）
                if let Some(text) = self.resolve_module_spec_from(
                    module_sym,
                    spec.text().trim_matches(['"', '\'', '`']),
                ) {
                    ns_export_target = Some(text);
                }
                return true;
            }
            if let tsox_frontend::ast::NodeData::ExportDeclaration(d) = &stmt.data
                && let Some(clause) = &d.export_clause
                && let tsox_frontend::ast::NodeData::NamedExports(ne) = &clause.data
            {
                for el in ne.elements.iter() {
                    if let tsox_frontend::ast::NodeData::ExportSpecifier(spec) = &el.data
                        && spec.name.text().trim_matches(['"', '\'', '`']) == name
                    {
                        let imported = spec
                            .property_name
                            .as_ref()
                            .unwrap_or(&spec.name)
                            .text()
                            .trim_matches(['"', '\'', '`'])
                            .to_string();
                        let module_text = d.module_specifier.as_ref().map(|module_spec| {
                            module_spec
                                .text()
                                .trim_matches(['"', '\'', '`'])
                                .to_string()
                        });
                        clause_hits.push((imported, module_text));
                        return true;
                    }
                }
            }
            false
        });
        if let Some(target) = ns_export_target {
            return Some(target);
        }
        // 普通 `export * from '...'`：递归星号目标查名（具名/本地优先已保证）
        for spec in star_specs {
            let Some(target) = self
                .resolve_module_spec_from(module_sym, &spec)
                .or_else(|| self.resolve_module_file_symbol(&spec))
            else {
                continue;
            };
            if let Some(found) =
                self.resolve_module_member_symbol(&target, name, depth - 1)
            {
                return Some(found);
            }
        }
        for (imported, module_text) in clause_hits {
            let target_module = match module_text {
                None => Arc::clone(module_sym),
                Some(text) => match self.resolve_module_spec_from(module_sym, &text) {
                    Some(m) => m,
                    None => continue,
                },
            };
            if let Some(target) =
                self.resolve_module_member_symbol(&target_module, &imported, depth - 1)
            {
                return Some(target);
            }
        }
        sym
    }

    pub(crate) fn resolve_module_spec_from(
        &self,
        base_module: &Arc<Symbol>,
        specifier: &str,
    ) -> Option<Arc<Symbol>> {
        // tspath isExternalModuleNameRelative 语义：仅 ./ ../ 开头是相对路径，
        // `.prisma/client` 这类点前缀包名走 node_modules 解析
        if !specifier.starts_with("./") && !specifier.starts_with("../") {
            return self.resolve_module_file_symbol(specifier);
        }
        let dir = base_module
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::SourceFile)
            .and_then(|d| self.get_source_file_of_node(d))
            .map(|f| {
                f.file_name
                    .rfind('/')
                    .map(|i| f.file_name[..i].to_string())
                    .unwrap_or_default()
            })?;
        self.resolve_module_file_symbol_in(&dir, specifier)
    }

    pub(crate) fn type_of_dynamic_import(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> {
        let spec = self.spec_of_dynamic_import_call(node)?;
        if spec.is_empty() {
            return None;
        }
        let cur = self.current_file.clone()?;

        let module_sym = match self.resolve_module_file_symbol(&spec) {
            Some(s) => s,
            None => {
                let path = self.program.resolve_external_module_path(
                    &spec,
                    &cur.file_name,
                    tsox_core::core::compiler_options::ModuleKind::ESNext,
                )?;
                let sf = self.program.get_source_file(&path)?;
                self.program.symbol_map().symbol_of(&sf.node).cloned()?
            }
        };
        Some(self.resolve_namespace_type(&module_sym))
    }

    pub(crate) fn spec_of_dynamic_import_call(&self, node: &Arc<Node>) -> Option<String> {
        if node.kind != SyntaxKind::CallExpression {
            return None;
        }
        let (callee, args) = match &node.data {
            NodeData::CallExpression(d) => (&d.expression, &d.arguments),
            _ => return None,
        };
        if callee.kind != SyntaxKind::ImportKeyword {
            return None;
        }
        let spec_node = args.iter().next()?;
        if spec_node.kind != SyntaxKind::StringLiteral {
            return None;
        }
        Some(spec_node.text().trim_matches(['"', '\'', '`']).to_string())
    }
}
