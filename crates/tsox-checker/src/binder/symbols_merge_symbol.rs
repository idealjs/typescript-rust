#![allow(unused_imports)]

use crate::binder::symbols::*;

impl Binder {
    // Go 模块容器本地/导出分表的合并面：同名不同导出性的声明并入同一
    // 符号（checker 的 TS2395 依赖合并声明集），不做 binder 冲突报告
    pub(crate) fn append_declaration_to_existing_symbol(
        &mut self,
        node: &Arc<Node>,
        existing: &Arc<Symbol>,
        includes: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let existing_mut = Arc::as_ptr(existing) as *mut Symbol;
        unsafe {
            (*existing_mut).declarations.push(Arc::clone(node));
            (*existing_mut).flags |= includes;
            if (*existing_mut).value_declaration.is_none()
                && includes.intersects(SymbolFlags::VALUE)
            {
                (*existing_mut).value_declaration = Some(Arc::clone(node));
            }
        }
        self.symbol_map.set_symbol(node, Arc::clone(existing));
        Some(Arc::clone(existing))
    }

    pub(crate) fn merge_into_existing_symbol(
        &mut self,
        node: &Arc<Node>,
        existing: &Arc<Symbol>,
        includes: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let var_var_merge = Self::declaration_is_var(node)
            && existing.flags == SymbolFlags::BlockScopedVariable
            && existing
                .declarations
                .iter()
                .all(|d| Self::declaration_is_var(d));

        // Go declareSymbol：var 与非实例化 namespace 互相合并（var excludes 不含
        // NamespaceModule；非实例化 namespace excludes=None）
        let ns_var_merge = Self::declaration_is_var(node)
            && existing.flags.contains(SymbolFlags::NamespaceModule)
            && !existing.flags.contains(SymbolFlags::ValueModule);

        let var_ns_merge = node.kind == SyntaxKind::ModuleDeclaration
            && get_module_instance_state(node) == ModuleInstanceState::NonInstantiated
            && existing.flags == SymbolFlags::BlockScopedVariable;

        // Go declareSymbol：类成员只要未触发 excludes 冲突即并入既有符号
        // （get/set 对、static/实例分表合并等），跨 staticness 子集恒并入
        let same_static_flags = self.class_member_same_static_flags(node, existing);
        let staticness_split = same_static_flags
            .map(|(same, _)| same == SymbolFlags::empty())
            .unwrap_or(false);
        let class_member_merge = same_static_flags.is_some();

        let import_export_alias_merge = includes.contains(SymbolFlags::Alias)
            && existing.flags.contains(SymbolFlags::Alias)
            && {
                let node_is_spec = node.kind == SyntaxKind::ExportSpecifier;
                let existing_all_spec = existing
                    .declarations
                    .iter()
                    .all(|d| d.kind == SyntaxKind::ExportSpecifier);
                node_is_spec != existing_all_spec
            };
        let comparison_flags = match same_static_flags {
            Some((same, _)) => same,
            None => existing.flags,
        };
        if staticness_split
            || class_member_merge
            || self.can_merge_symbols(comparison_flags, includes)
            || var_var_merge
            || ns_var_merge
            || var_ns_merge
            || import_export_alias_merge
        {
            let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
            unsafe {
                (*existing_mut).declarations.push(Arc::clone(node));
                (*existing_mut).flags |= includes;

                if (*existing_mut).value_declaration.is_none()
                    && includes.intersects(SymbolFlags::VALUE)
                {
                    (*existing_mut).value_declaration = Some(Arc::clone(node));
                }
            }

            let ns_proto_loc: Option<tsox_core::core::text::TextRange> = (|| {
                let scan = |n: &Arc<Node>| -> Option<tsox_core::core::text::TextRange> {
                    if n.kind != SyntaxKind::ModuleDeclaration {
                        return None;
                    }
                    let NodeData::ModuleDeclaration(md) = &n.data else {
                        return None;
                    };
                    let body = md.body.as_ref()?;
                    let mut hit: Option<tsox_core::core::text::TextRange> = None;
                    tsox_frontend::ast::node_data_generated::for_each_child(body, |stmt| {
                        if stmt.kind == SyntaxKind::VariableStatement {
                            if let NodeData::VariableStatement(vs) = &stmt.data {
                                let NodeData::VariableDeclarationList(vdl) =
                                    &vs.declaration_list.data
                                else {
                                    return false;
                                };
                                for decl in vdl.declarations.iter() {
                                    if decl.name().is_some_and(|n| n.text() == "prototype") {
                                        hit = decl.name().map(|n| n.loc);
                                    }
                                }
                            }
                        }
                        false
                    });
                    hit
                };
                if node.kind == SyntaxKind::ModuleDeclaration
                    && existing
                        .flags
                        .intersects(SymbolFlags::Class | SymbolFlags::Function)
                {
                    return scan(node);
                }
                if matches!(
                    node.kind,
                    SyntaxKind::ClassDeclaration | SyntaxKind::FunctionDeclaration
                ) && existing.flags.contains(SymbolFlags::ValueModule)
                {
                    for d in &existing.declarations {
                        if let Some(loc) = scan(d) {
                            return Some(loc);
                        }
                    }
                }
                None
            })();
            if let Some(loc) = ns_proto_loc {
                let already = self
                    .symbol_map
                    .binder_diagnostics
                    .iter()
                    .any(|dd| dd.code == 2300 && dd.loc == loc);
                if !already {
                    self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                        self.current_source_file.clone(),
                        loc,
                        DUPLICATE_IDENTIFIER_0,
                        vec!["prototype".to_string()],
                    ));
                }
            }

            if node.kind == SyntaxKind::EnumDeclaration
                && existing.flags.intersects(SymbolFlags::ENUM)
            {
                let NodeData::EnumDeclaration(new_ed) = &node.data else {
                    unreachable!()
                };
                let mut new_names: Vec<(String, tsox_core::core::text::TextRange)> = Vec::new();
                for m in new_ed.members.iter() {
                    if let Some(n) = m.name() {
                        new_names.push((n.text().to_string(), n.loc));
                    }
                }
                for d in &existing.declarations {
                    if d.kind != SyntaxKind::EnumDeclaration || Arc::ptr_eq(d, node) {
                        continue;
                    }
                    let NodeData::EnumDeclaration(ed) = &d.data else {
                        continue;
                    };
                    for m in ed.members.iter() {
                        let Some(n) = m.name() else { continue };
                        if let Some((_, new_loc)) =
                            new_names.iter().find(|(name, _)| *name == n.text())
                        {
                            for loc in [*new_loc, n.loc] {
                                let already = self
                                    .symbol_map
                                    .binder_diagnostics
                                    .iter()
                                    .any(|dd| dd.code == 2300 && dd.loc == loc);
                                if !already {
                                    self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                                        self.current_source_file.clone(),
                                        loc,
                                        DUPLICATE_IDENTIFIER_0,
                                        vec![n.text().to_string()],
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            self.symbol_map.set_symbol(node, Arc::clone(&existing));

            if let Some(container) = &self.container {
                let is_module_container = container.kind == SyntaxKind::SourceFile
                    || container.kind == SyntaxKind::ModuleDeclaration;
                if is_module_container
                    && self
                        .get_combined_modifier_flags(node)
                        .contains(ModifierFlags::Export)
                {
                    let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                    unsafe {
                        (*existing_mut).export_symbol = Some(Arc::clone(&existing));
                    }
                }
            }
            return Some(Arc::clone(existing));
        }
        None
    }
}
