#![allow(unused_imports)]

use crate::checker::checker_resolve::*;

impl Checker {
    pub fn merge_symbol_table(
        &mut self,
        target: &mut SymbolTable,
        source: &SymbolTable,
        unidirectional: bool,
        merged_parent: Option<u64>,
    ) { ::tsox_core::fntrace::enter("merge_symbol_table"); 
        let entries: Vec<(String, Arc<Symbol>)> = source
            .iter()
            .map(|(k, v)| (k.clone(), Arc::clone(v)))
            .collect();
        for (name, source_symbol) in entries {
            if let Some(target_symbol) = target.entries.get_mut(&name) {
                let merged = self.merge_symbol(target_symbol, &source_symbol, unidirectional);
                let is_transient = merged.flags.intersects(SymbolFlags::Transient);
                *target_symbol = merged;
                if let Some(_parent_id) = merged_parent {
                    if is_transient {}
                }
            } else {
                let merged = self.get_merged_symbol(&source_symbol);
                target.insert(name, merged);
            }
        }
    }

    pub fn merge_symbol(
        &mut self,
        target: &Arc<Symbol>,
        source: &Arc<Symbol>,
        unidirectional: bool,
    ) -> Arc<Symbol> { ::tsox_core::fntrace::enter("merge_symbol"); 
        let excluded = get_excluded_symbol_flags(source.flags);
        if target.flags.intersects(excluded) == false
            || (source.flags | target.flags).intersects(SymbolFlags::Assignment)
        {
            if Arc::ptr_eq(target, source) {
                return Arc::clone(target);
            }

            let (effective_target, cloned_from) = if !target.flags.intersects(SymbolFlags::Transient) {
                // Go resolveSymbolEx：非局部别名先解析真身再比对 excludes；
                // UMD（export as namespace）别名无 export_symbol 直连目标，
                // 同样经完整别名解析揭示真身模块符号后再克隆
                let alias_excludes = SymbolFlags::VALUE | SymbolFlags::TYPE | SymbolFlags::NAMESPACE;
                let is_non_local = target.flags
                    & (SymbolFlags::Alias | alias_excludes)
                    == SymbolFlags::Alias
                    || (target.flags.intersects(SymbolFlags::Alias)
                        && target.flags.intersects(SymbolFlags::Assignment));
                let resolved_target = if is_non_local
                    || target.declarations.iter().any(|d| {
                        d.kind == tsox_frontend::ast::SyntaxKind::NamespaceExportDeclaration
                    }) {
                    let resolved = self.resolve_alias_base(Arc::clone(target));
                    // Go mergeSymbol：resolveSymbol 解析结果为 unknownSymbol
                    //（别名环，resolveAlias 环检测失败）时直接返回 source，
                    // 不做克隆合并
                    if self.alias_circular_reported.contains(&resolved.id()) {
                        return Arc::clone(source);
                    }
                    resolved
                } else {
                    self.resolve_symbol(target)
                };
                if resolved_target
                    .flags
                    .intersects(get_excluded_symbol_flags(source.flags))
                    == false
                    || (source.flags | resolved_target.flags).intersects(SymbolFlags::Assignment)
                {
                    match self.clone_symbol(&resolved_target) {
                        Some(cloned) => (cloned, Some(resolved_target)),
                        None => return Arc::clone(source),
                    }
                } else {
                    self.report_merge_symbol_error(target, source);
                    return Arc::clone(source);
                }
            } else {
                (Arc::clone(target), None)
            };

            let mut source_flags = source.flags;
            if !effective_target
                .flags
                .intersects(SymbolFlags::ConstEnumOnlyModule)
            {
                source_flags.remove(SymbolFlags::ConstEnumOnlyModule);
            }
            let merged_flags = effective_target.flags | source_flags;

            let mut merged = Symbol::new(merged_flags, &effective_target.name);

            merged.value_declaration = source
                .value_declaration
                .clone()
                .or_else(|| effective_target.value_declaration.clone());

            merged.declarations = effective_target.declarations.clone();
            for d in &source.declarations {
                if !merged.declarations.iter().any(|e| Arc::ptr_eq(e, d)) {
                    merged.declarations.push(Arc::clone(d));
                }
            }

            // Go mergeSymbol 克隆路径的最终符号即 cloneSymbol 产物
            //（checker.go:14942），父链 = 真身父链（clone.Parent =
            // resolvedTarget.Parent）；transient 路径父链不变。产物独立于
            // 克隆符号时父链须同样取真身父而非克隆——取克隆会让
            // getContainersOfSymbol 把合并血统当容器上溯，显示链混入克隆段
            if let Some(parent) = effective_target.parent() {
                merged.set_parent(&parent);
            }

            merged.members = SymbolTable {
                entries: effective_target.members.entries.clone(),
            };
            merged.exports = SymbolTable {
                entries: effective_target.exports.entries.clone(),
            };

            let result = Arc::new(merged);

            let mut result_mut = Symbol::new(result.flags, &result.name);
            result_mut.value_declaration = result.value_declaration.clone();
            result_mut.declarations = result.declarations.clone();
            // 终版符号父链与中间产物一致（真身父链），不再指向中间产物
            if let Some(parent) = result.parent() {
                result_mut.set_parent(&parent);
            }
            result_mut.members = SymbolTable {
                entries: result.members.entries.clone(),
            };
            result_mut.exports = SymbolTable {
                entries: result.exports.entries.clone(),
            };

            let source_members: Vec<(String, Arc<Symbol>)> = source
                .members
                .iter()
                .map(|(k, v)| (k.clone(), Arc::clone(v)))
                .collect();
            for (name, source_sym) in source_members {
                if let Some(target_sym) = result_mut.members.entries.get_mut(&name) {
                    let merged = self.merge_symbol(target_sym, &source_sym, unidirectional);
                    *target_sym = merged;
                } else {
                    let merged = self.get_merged_symbol(&source_sym);
                    result_mut.members.insert(name, merged);
                }
            }

            let source_exports: Vec<(String, Arc<Symbol>)> = source
                .exports
                .iter()
                .map(|(k, v)| (k.clone(), Arc::clone(v)))
                .collect();
            for (name, source_sym) in source_exports {
                if let Some(target_sym) = result_mut.exports.entries.get_mut(&name) {
                    let merged = self.merge_symbol(target_sym, &source_sym, unidirectional);
                    *target_sym = merged;
                } else {
                    let merged = self.get_merged_symbol(&source_sym);
                    result_mut.exports.insert(name, merged);
                }
            }

            let final_result = Arc::new(result_mut);

            if !unidirectional {
                self.record_merged_symbol(&final_result, source);
            }
            if let Some(orig) = &cloned_from {
                self.record_merged_symbol(&final_result, orig);
            }

            final_result
        } else if target.flags.intersects(SymbolFlags::NamespaceModule) {
            self.report_cannot_augment_with_value_exports(target, source);
            Arc::clone(target)
        } else {
            self.report_merge_symbol_error(target, source);
            Arc::clone(target)
        }
    }

    pub(crate) fn report_merge_symbol_error(&mut self, target: &Arc<Symbol>, source: &Arc<Symbol>) { ::tsox_core::fntrace::enter("report_merge_symbol_error"); 
        let is_either_enum =
            target.flags.contains(SymbolFlags::ENUM) || source.flags.contains(SymbolFlags::ENUM);
        let is_either_block_scoped = target.flags.intersects(SymbolFlags::BlockScopedVariable)
            || source.flags.intersects(SymbolFlags::BlockScopedVariable);
        let message = if is_either_enum {
            tsox_core::diagnostics::messages_generated::
                ENUM_DECLARATIONS_CAN_ONLY_MERGE_WITH_NAMESPACE_OR_OTHER_ENUM_DECLARATIONS
        } else if is_either_block_scoped {
            tsox_core::diagnostics::messages_generated::CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0
        } else {
            tsox_core::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0
        };
        let name = source.name.clone();
        for (err_sym, rel_sym) in [(source, target), (target, source)] {
            for decl in &err_sym.declarations {
                let error_node = tsox_frontend::ast::utilities::get_name_of_declaration(decl)
                    .unwrap_or_else(|| Arc::clone(decl));
                let file = self.get_source_file_of_node(&error_node);
                let mut diag = tsox_frontend::ast::Diagnostic::new(
                    file,
                    error_node.loc,
                    message,
                    vec![name.clone()],
                );
                for rel_decl in &rel_sym.declarations {
                    let rel_node = tsox_frontend::ast::utilities::get_name_of_declaration(rel_decl)
                        .unwrap_or_else(|| Arc::clone(rel_decl));
                    if Arc::ptr_eq(&rel_node, &error_node) || diag.related_information.len() >= 5 {
                        continue;
                    }
                    let rel_file = self.get_source_file_of_node(&rel_node);
                    if diag.related_information.iter().any(|d| {
                        d.loc == rel_node.loc
                            && d.file.as_ref().map(|f| f.file_name.as_str())
                                == rel_file.as_ref().map(|f| f.file_name.as_str())
                    }) {
                        continue;
                    }
                    let (rel_msg, args) = if diag.related_information.is_empty() {
                        (
                            tsox_core::diagnostics::messages_generated::X_0_WAS_ALSO_DECLARED_HERE,
                            vec![name.clone()],
                        )
                    } else {
                        (
                            tsox_core::diagnostics::messages_generated::X_AND_HERE,
                            Vec::new(),
                        )
                    };
                    diag.related_information
                        .push(tsox_frontend::ast::Diagnostic::new(
                            rel_file,
                            rel_node.loc,
                            rel_msg,
                            args,
                        ));
                }
                self.diagnostics.add_or_append_related(diag);
            }
        }
    }

    pub fn record_merged_symbol(&mut self, target: &Arc<Symbol>, source: &Arc<Symbol>) { ::tsox_core::fntrace::enter("record_merged_symbol"); 
        self.merged_symbols.insert(source.id(), target.id());
        self.merged_symbol_targets
            .insert(target.id(), Arc::clone(target));
    }

    pub fn merged_symbol_by_id(&self, id: u64) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("merged_symbol_by_id"); 
        self.merged_symbol_targets.get(&id).cloned()
    }

    pub(crate) fn record_merged_symbol_if_absent(&mut self, target: &Arc<Symbol>, source: &Arc<Symbol>) { ::tsox_core::fntrace::enter("record_merged_symbol_if_absent"); 
        if !self.merged_symbols.contains_key(&source.id()) {
            self.record_merged_symbol(target, source);
        }
    }

    pub fn clone_symbol(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("clone_symbol"); 
        let mut cloned = Symbol::new(symbol.flags | SymbolFlags::Transient, &symbol.name);
        cloned.declarations = symbol.declarations.clone();
        // Go cloneSymbol（checker.go:15133）result.Parent = symbol.Parent：
        // 克隆继承原符号的父链（文件符号克隆父为 None），而非指回原符号——
        // 指回自身/原符号会让 getContainersOfSymbol 的容器攀爬沿合并血统
        // 上溯，显示链多出克隆段
        if let Some(parent) = symbol.parent() {
            cloned.set_parent(&parent);
        }
        cloned.value_declaration = symbol.value_declaration.clone();
        cloned.members = SymbolTable {
            entries: symbol.members.entries.clone(),
        };
        cloned.exports = SymbolTable {
            entries: symbol.exports.entries.clone(),
        };
        let result = Arc::new(cloned);
        Some(result)
    }

    pub fn resolve_symbol(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> { ::tsox_core::fntrace::enter("resolve_symbol"); 
        if let Some(result) = self.follow_alias(symbol) {
            result
        } else {
            Arc::clone(symbol)
        }
    }

    /// 声明名归并记录未命中时的回落：脚本文件顶层声明的节点侧符号若与
    /// populate_globals 写回后的文件符号表条目（合并符号）不一致，取表中
    /// 声明集不小于节点侧符号者。仅命中顶层声明（声明宿主直接挂脚本
    /// SourceFile），嵌套容器内同名声明不入文件表、不受此回落影响
    pub(crate) fn script_top_level_table_symbol_of(
        &self,
        sym: &Arc<Symbol>,
        name_node: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        let decl = name_node.parent()?;
        let mut file_node = Arc::clone(&decl);
        let mut current = decl.parent();
        while let Some(a) = current {
            file_node = Arc::clone(&a);
            current = a.parent();
        }
        if file_node.kind != SyntaxKind::SourceFile || !Self::is_global_source_file(&file_node) {
            return None;
        }
        if decl.parent().is_none_or(|p| p.id() != file_node.id()) {
            return None;
        }
        let name = sym.name.as_str();
        let symbol_map = self.program.symbol_map();
        // 回落源优先级：globals（解析回退同源，populate_globals 合并产物）→
        // 文件 locals/members（写回循环同步的合并符号）
        let table_sym = self
            .globals
            .get(name)
            .or_else(|| symbol_map.locals_of(&file_node).and_then(|l| l.get(name)))
            .or_else(|| {
                symbol_map
                    .symbol_of(&file_node)
                    .and_then(|fs| fs.members.get(name))
            })?;
        if Arc::ptr_eq(&table_sym, sym) || table_sym.name != sym.name {
            return None;
        }
        if table_sym.declarations.len() < sym.declarations.len() {
            return None;
        }
        Some(Arc::clone(table_sym))
    }

    pub fn get_symbol_at_location(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_symbol_at_location");
        if tsox_frontend::ast::mig::m3f_4::is_declaration_name_or_import_property_name(node) {
            // Go getSymbolAtLocation（checker.go:33452-33458）声明名经
            // getSymbolOfDeclaration(parent)（checker.go:15181-15188）恒归并：
            // getMergedSymbol(getLateBoundSymbol(symbol))——script 文件顶层声明
            // 与 lib 全局符（lib.dom alert + 本文件 declare function alert）合并
            // 后节点侧符号缺 lib 声明集，未归并则声明名型呈单声明函数型而非
            // 聚合过载型；晚绑定步对本臂符号（非计算属性名成员）恒等，此处按
            // &self 入口仅施 getMergedSymbol
            if let Some(sym) = node
                .parent()
                .and_then(|parent| self.get_symbol_of_declaration(&parent))
            {
                let merged = self.get_merged_symbol(&sym);
                // 容器内成员声明：归并链终点可为解析期单声明实例（成员节点
                // 绑符号的 if_absent 记录先于合并体落链），终点不持有本声明
                // 或声明集小于宿主容器合并体成员表条目时取后者——Go 声明名
                // 恒解析到合并体，getTypeOfSymbol 聚合全部声明签名
                if let Some(table_sym) = self.container_member_table_symbol_of(&sym, node)
                    && node.parent().is_some_and(|decl| {
                        merged.declarations.len() < table_sym.declarations.len()
                            || !merged.declarations.iter().any(|d| Arc::ptr_eq(d, &decl))
                    })
                {
                    return Some(table_sym);
                }
                if !Arc::ptr_eq(&merged, &sym) {
                    return Some(merged);
                }
                // 归并记录未命中（get_merged_symbol 恒等）时，按 populate_globals
                // 的「脚本文件顶层符号即全局符号」写回模型回落文件符号表：合并
                // 符号已回写 members/locals，取声明集不小于节点侧符号者
                if let Some(table_sym) = self.script_top_level_table_symbol_of(&sym, node) {
                    return Some(table_sym);
                }
                return Some(sym);
            }
            return None;
        }
        if let Some(sym) = self.program.symbol_map().symbol_of(node) {
            return Some(Arc::clone(sym));
        }

        // Go getSymbolAtLocation（checker.go:33445-33452）：声明名回溯父声明
        // 取 getSymbolOfDeclaration(parent)（接口成员名/类型参数名等符号挂在
        // 成员节点而非名字节点）——同上恒归并
        if tsox_frontend::ast::mig::m3f_4::is_declaration_name_or_import_property_name(node) {
            if let Some(parent) = node.parent()
                && let Some(sym) = self.get_symbol_of_declaration(&parent)
            {
                return Some(self.get_merged_symbol(&sym));
            }
        }

        if node.kind == tsox_frontend::ast::SyntaxKind::PropertyAccessExpression {
            if let tsox_frontend::ast::NodeData::PropertyAccessExpression(data) = &node.data {
                if let Some(links) = self.type_node_links.get(&data.expression) {
                    if let Some(ref t) = links.resolved_type {
                        return self.get_property_of_type_cached(t, data.name.text());
                    }
                }
            }
        }

        None
    }
}
