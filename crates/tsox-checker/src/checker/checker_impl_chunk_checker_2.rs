#![allow(unused_imports)]

use crate::checker::checker_impl_chunk::*;
use tsox_frontend::ast::SymbolTable;

impl Checker {
    pub(crate) fn merge_global_symbols(&mut self, dst: &Arc<Symbol>, src: &Arc<Symbol>) { ::tsox_core::fntrace::enter("merge_global_symbols"); 
        let dst_mut = Arc::as_ptr(dst) as *mut Symbol;
        unsafe {
            (*dst_mut).flags |= src.flags;
            for d in &src.declarations {
                if !dst
                    .declarations
                    .iter()
                    .any(|existing| Arc::ptr_eq(existing, d))
                {
                    (*dst_mut).declarations.push(Arc::clone(d));
                }
            }
            if dst.value_declaration.is_none() {
                (*dst_mut).value_declaration = src.value_declaration.clone();
            }
            for (name, member) in src.members.entries.iter() {
                match (*dst_mut).members.entries.get(name).cloned() {
                    Some(existing) => {
                        if !self.report_global_merge_conflict(&existing, member) {
                            self.merge_global_symbols(&existing, member);
                            // 成员声明节点绑符号同谱系：binder 对声明节点绑定的
                            // 符号与容器成员表 Arc 可为并行谱系（共享 binder 跨文
                            // 件复用累积实例），Go getSymbolOfDeclaration（checker.
                            // go:15181-15189）对成员声明恒解析到合并体；接口声明
                            // 型成员解析（add_method_signature_member 等）的
                            // record_merged_symbol_if_absent 若先于此记录，会把节
                            // 点绑符号重定向到解析期单声明实例，getTypeOfSymbol
                            // 经合并链只见到部分声明（promisesWithConstraints
                            // >then 单签名函数型）。populate 合并后预置节点绑符
                            // 号→accumulator 记录，解析层 if_absent 随之跳过，
                            // get_merged_symbol 链终点恒为全量合并体
                            for d in &member.declarations {
                                if let Some(node_sym) =
                                    self.program.symbol_map().symbol_of(d).cloned()
                                {
                                    self.record_merged_symbol_if_absent(&existing, &node_sym);
                                }
                            }
                        }
                    }
                    None => {
                        (*dst_mut)
                            .members
                            .entries
                            .insert(name.clone(), Arc::clone(member));
                    }
                }
            }
            for (name, export) in src.exports.entries.iter() {
                match (*dst_mut).exports.entries.get(name).cloned() {
                    Some(existing) => {
                        if !self.report_global_merge_conflict(&existing, export) {
                            self.merge_global_symbols(&existing, export);
                        }
                    }
                    None => {
                        (*dst_mut)
                            .exports
                            .entries
                            .insert(name.clone(), Arc::clone(export));
                    }
                }
            }
            if !Arc::ptr_eq(dst, src) {
                self.record_merged_symbol(dst, src);
            }
        }
    }

    pub(crate) fn populate_globals(&mut self) { ::tsox_core::fntrace::enter("populate_globals"); 
        let script_files: Vec<Arc<SourceFile>> = self
            .files
            .iter()
            .filter(|f| f.external_module_indicator.is_none())
            .cloned()
            .collect();
        // Go initializeTypeChecker（checker.go:1348-1356）：UMD 全局（export
        // as namespace）并入 globals（first-in-wins）。段序对齐 Go 文件循环：
        // 外部模块文件（含 .d.ts）在 files 序中先于脚本文件时，其 UMD 别名
        // 先进 globals，随后脚本文件顶层同名符号经 merge_global_entry 与之
        // mergeSymbol 合并——UMD 段后置会让别名被 first-in-wins 挡在合并之外
        for file in &self.files {
            if file.external_module_indicator.is_none() {
                continue;
            }
            let symbol_map = self.program.symbol_map();
            let Some(file_sym) = symbol_map.symbol_of(&file.node).cloned() else {
                continue;
            };
            for (name, sym) in file_sym.exports.entries.iter() {
                let is_umd_alias = sym.declarations.iter().any(|d| {
                    d.kind == tsox_frontend::ast::SyntaxKind::NamespaceExportDeclaration
                });
                if !is_umd_alias {
                    continue;
                }
                match self.globals.get(name) {
                    Some(_) => {}
                    None => {
                        self.globals.insert(name.clone(), Arc::clone(sym));
                    }
                }
            }
        }
        for file in &script_files {
            let symbol_map = self.program.symbol_map();
            // Go initializeChecker：脚本文件自声明 globalThis 即与内建全局冲突
            let file_global_this = symbol_map
                .symbol_of(&file.node)
                .and_then(|fs| fs.members.get("globalThis").cloned())
                .or_else(|| {
                    symbol_map
                        .locals_of(&file.node)
                        .and_then(|l| l.get("globalThis").cloned())
                });
            if let Some(gt) = file_global_this {
                for d in &gt.declarations {
                    let loc = d.name().map(|n| n.loc).unwrap_or(d.loc);
                    let file = self.get_source_file_of_node(d);
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        file,
                        loc,
                        tsox_core::diagnostics::messages_generated::
                            DECLARATION_NAME_CONFLICTS_WITH_BUILT_IN_GLOBAL_IDENTIFIER_0,
                        vec!["globalThis".to_string()],
                    ));
                }
            }
            let mut member_entries: Vec<(String, Arc<Symbol>)> = Vec::new();
            let mut local_entries: Vec<(String, Arc<Symbol>)> = Vec::new();
            if let Some(file_sym) = symbol_map.symbol_of(&file.node) {
                for (k, v) in file_sym.members.iter() {
                    member_entries.push((k.clone(), Arc::clone(v)));
                }
                if let Some(locals) = symbol_map.locals_of(&file.node) {
                    for (k, v) in locals.iter() {
                        local_entries.push((k.clone(), Arc::clone(v)));
                    }
                }
            }
            for (name, sym) in member_entries.into_iter().chain(local_entries) {
                self.merge_global_entry(&name, &sym);
            }
        }

        let mut global_aug_symbols: Vec<Arc<Symbol>> = Vec::new();
        for file in &self.files {
            for aug_name in &file.module_augmentations {
                let Some(module_node) = aug_name.parent() else {
                    continue;
                };
                if !tsox_frontend::ast::is_global_scope_augmentation(&module_node) {
                    continue;
                }
                let symbol_map = self.program.symbol_map();
                if let Some(module_sym) = symbol_map.symbol_of(&module_node) {
                    if module_sym
                        .declarations
                        .first()
                        .is_some_and(|d| d.id() != module_node.id())
                    {
                        continue;
                    }
                    global_aug_symbols.push(Arc::clone(module_sym));
                }
            }
        }
        for aug_sym in global_aug_symbols {
            let exports = aug_sym.exports.clone();
            let mut globals = std::mem::take(&mut self.globals);
            self.merge_symbol_table(&mut globals, &exports, false, None);
            self.globals = globals;
        }

        // Go 模型：脚本文件顶层符号即全局符号（无按文件遮蔽）。全局合并后
        // 把合并符号回写各脚本文件符号表，文件内解析命中同一合并符号
        for file in &script_files {
            let Some(file_sym) = self.program.symbol_map().symbol_of(&file.node).cloned() else {
                continue;
            };
            let member_keys: Vec<String> = file_sym.members.iter().map(|(k, _)| k.clone()).collect();
            let local_keys: Vec<String> = self
                .program
                .symbol_map()
                .locals_of(&file.node)
                .map(|l| l.iter().map(|(k, _)| k.clone()).collect())
                .unwrap_or_default();
            let file_sym_mut = Arc::as_ptr(&file_sym) as *mut tsox_frontend::ast::Symbol;
            unsafe {
                for k in member_keys {
                    if let Some(merged) = self.globals.get(&k) {
                        (*file_sym_mut).members.insert(k, Arc::clone(merged));
                    }
                }
            }
            if let Some(locals) = self.program.symbol_map().locals_of(&file.node) {
                let locals_mut = locals as *const _ as *mut tsox_frontend::ast::SymbolTable;
                unsafe {
                    for k in local_keys {
                        if let Some(merged) = self.globals.get(&k) {
                            (*locals_mut).insert(k, Arc::clone(merged));
                        }
                    }
                }
            }
        }

        self.merge_module_augmentations();

        self.add_undefined_to_globals_or_error_on_redeclaration();

        let args_symbol = self.arguments_symbol.clone();
        if let Some(sym) = args_symbol {
            let t = self.get_global_type_by_name("IArguments");
            if let Some(t) = t {
                self.value_symbol_links.get_or_default(&sym).resolved_type = Some(t);
            }
        }

        self.report_missing_global_types();
    }

    /// Go mergeModuleAugmentation（非 global）：增广模块的导出并入目标模块符号；
    /// 目标带 export * 时先并入 re-export 解析出的目标符号（声明合并）
    fn merge_module_augmentations(&mut self) { ::tsox_core::fntrace::enter("merge_module_augmentations"); 
        use tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_STAR;
        let mut augs: Vec<(Arc<Node>, Arc<Node>)> = Vec::new();
        for file in &self.files {
            for name in &file.module_augmentations {
                let Some(module_node) = name.parent() else {
                    continue;
                };
                if tsox_frontend::ast::is_global_scope_augmentation(&module_node) {
                    continue;
                }
                augs.push((Arc::clone(name), module_node));
            }
        }
        for (aug_name, module_node) in augs {
            let symbol_map = self.program.symbol_map();
            let Some(aug_sym) = symbol_map.symbol_of(&module_node).cloned() else {
                continue;
            };
            let Some(file) = self.get_source_file_of_node(&aug_name) else {
                continue;
            };
            let Some(file_module) = symbol_map.symbol_of(&file.node).cloned() else {
                continue;
            };
            let spec = aug_name.text().trim_matches(['"', '\'', '`']).to_string();
            let main_module = self.resolve_module_spec_from(&file_module, &spec);
            let Some(main_module) = main_module else {
                continue;
            };
            // Go mergeModuleAugmentation：resolveExternalModuleSymbol(m, false)
            // 将 export= 经 resolveAlias 解到目标符号（foo 等）再判 Namespace；
            // resolveExternalModule 返回的是 getMergedSymbol 后的合并符号，
            // global 增强先行克隆的合并结果在此续并（z/a2.y 同一符号视图）
            let main_module = self.resolve_external_module_symbol_go(&main_module);
            let main_module = self.get_merged_symbol(&main_module);
            if !main_module.flags.intersects(SymbolFlags::NAMESPACE) {
                // Go mergeModuleAugmentation（checker.go:1508-1511）：export= 穿透
                // 后目标非 namespace 的增广报 TS2671 且不合并
                let file = self.get_source_file_of_node(&aug_name);
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    file,
                    aug_name.loc,
                    tsox_core::diagnostics::messages_generated::
                        CANNOT_AUGMENT_MODULE_0_BECAUSE_IT_RESOLVES_TO_A_NON_MODULE_ENTITY,
                    vec![spec],
                ));
                continue;
            }
            if self.module_augmentation_merge_conflict(&main_module, &aug_sym) {
                continue;
            }
            let mut aug_entries: Vec<(String, Arc<Symbol>)> = Vec::new();
            for (k, v) in aug_sym.exports.iter() {
                aug_entries.push((k.clone(), Arc::clone(v)));
            }
            for (k, v) in aug_sym.members.iter() {
                if !aug_entries.iter().any(|(ek, _)| ek == k) {
                    aug_entries.push((k.clone(), Arc::clone(v)));
                }
            }
            if aug_entries.is_empty() {
                continue;
            }
            {
                // Go mergeSymbol（checker.go:14928-14968）非 pattern 分支：flags
                // 并集、augmentation 声明并入目标、recordMergedSymbol(target,
                // source)——getSymbolOfDeclaration(augmentation 名节点) 由此取到
                // 目标模块符号（ValueModule 位），声明名类型走模块实例型
                let m_mut = Arc::as_ptr(&main_module) as *mut Symbol;
                unsafe {
                    (*m_mut).flags |= aug_sym.flags;
                    for d in &aug_sym.declarations {
                        if !(*m_mut).declarations.iter().any(|x| Arc::ptr_eq(x, d)) {
                            (*m_mut).declarations.push(Arc::clone(d));
                        }
                    }
                }
                self.record_merged_symbol(&main_module, &aug_sym);
            }
            let m_mut = Arc::as_ptr(&main_module) as *mut Symbol;
            let mut star_merged: std::collections::HashMap<String, Arc<Symbol>> =
                std::collections::HashMap::new();
            if main_module.exports.get(INTERNAL_SYMBOL_NAME_EXPORT_STAR).is_some() {
                let resolved = self.get_exports_of_module_table(&main_module);
                for (key, value) in &aug_entries {
                    if main_module.exports.get(key).is_none()
                        && let Some(target) = resolved.get(key)
                        && !Arc::ptr_eq(target, value)
                    {
                        let merged = self.merge_augmentation_symbols(target, value);
                        star_merged.insert(key.clone(), merged);
                    }
                }
            }
            for (key, value) in &aug_entries {
                if value
                    .declarations
                    .iter()
                    .any(|d| d.has_syntactic_modifier(ModifierFlags::Default))
                    && let Some(default_sym) = main_module.exports.get("default")
                    && !Arc::ptr_eq(default_sym, value)
                {
                    let base = self.resolve_alias_base(Arc::clone(default_sym));
                    if !Arc::ptr_eq(&base, value) {
                        let merged = self.merge_augmentation_symbols(&base, value);
                        if !Arc::ptr_eq(&merged, &base) {
                            unsafe {
                                if let Some(slot) =
                                    (*m_mut).exports.entries.get_mut("default")
                                {
                                    *slot = merged;
                                }
                            }
                        }
                    }
                }
            }
            for (key, value) in aug_entries {
                unsafe {
                    match (*m_mut).exports.get(&key) {
                        Some(existing) => {
                            let merged = self.merge_augmentation_symbols(existing, &value);
                            (*m_mut).exports.entries.insert(key.clone(), merged);
                        }
                        None => {
                            let insert = star_merged.get(&key).unwrap_or(&value);
                            (*m_mut)
                                .exports
                                .entries
                                .insert(key.clone(), Arc::clone(insert));
                        }
                    }
                    if (*m_mut).members.get(&key).is_none() {
                        let insert = star_merged.get(&key).unwrap_or(&value);
                        (*m_mut)
                            .members
                            .entries
                            .insert(key.clone(), Arc::clone(insert));
                    }
                }
            }
        }
    }

    pub(crate) fn report_missing_global_types(&mut self) { ::tsox_core::fntrace::enter("report_missing_global_types"); 
        const GLOBAL_TYPE_NAMES: &[&str] = &[
            "Array",
            "Boolean",
            "CallableFunction",
            "Function",
            "IArguments",
            "NewableFunction",
            "Number",
            "Object",
            "RegExp",
            "String",
        ];
        for name in GLOBAL_TYPE_NAMES {
            if self.globals.get(*name).is_none() {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    None,
                    tsox_core::core::text::TextRange::default(),
                    tsox_core::diagnostics::messages_generated::CANNOT_FIND_GLOBAL_TYPE_0,
                    vec![(*name).to_string()],
                ));
            }
        }
    }
}

impl Checker {
    fn merge_augmentation_symbols(
        &mut self,
        target: &Arc<Symbol>,
        source: &Arc<Symbol>,
    ) -> Arc<Symbol> { ::tsox_core::fntrace::enter("merge_augmentation_symbols");
        let mut records = Vec::new();
        let mut conflicts = Vec::new();
        let merged = self.merge_declarations_into_rec(target, source, &mut records, &mut conflicts);
        for (t, s) in records {
            self.record_merged_symbol(&t, &s);
        }
        for (t, s) in conflicts {
            self.report_merge_symbol_error(&t, &s);
        }
        merged
    }

    fn merge_declarations_into_rec(
        &self,
        target: &Arc<Symbol>,
        source: &Arc<Symbol>,
        records: &mut Vec<(Arc<Symbol>, Arc<Symbol>)>,
        conflicts: &mut Vec<(Arc<Symbol>, Arc<Symbol>)>,
    ) -> Arc<Symbol> { ::tsox_core::fntrace::enter("merge_declarations_into_rec");
        if merge_symbol_flags_conflict(&target.flags, &source.flags) {
            conflicts.push((Arc::clone(target), Arc::clone(source)));
            return Arc::clone(source);
        }
        // Go mergeSymbol（checker.go:14940-14949）非 transient 分支：alias 目标
        // 先 resolveSymbol 解到真身、cloneSymbol 真身后代入合并，产物由调用方
        // 回填表条目——specifier/import 节点符号保持绑定期的纯 Alias 旗标
        //（checkAliasSymbol excludedMeanings 投影 checker.go:7197-7203 以该
        // 绑定符号为界，增广并旗标不得倒灌别名符号）
        if target.flags.intersects(SymbolFlags::Alias)
            && !target.flags.contains(SymbolFlags::Transient)
        {
            let resolved = self.resolve_symbol(target);
            if Arc::ptr_eq(&resolved, target) {
                // follow 失败（别名环）按 Go resolvedTarget == unknownSymbol 处置
                return Arc::clone(source);
            }
            if merge_symbol_flags_conflict(&resolved.flags, &source.flags) {
                conflicts.push((Arc::clone(target), Arc::clone(source)));
                return Arc::clone(source);
            }
            let working = self
                .clone_symbol(&resolved)
                .unwrap_or_else(|| Arc::clone(&resolved));
            return self.merge_into_working(working, source, records, conflicts);
        }
        let working = Arc::clone(target);
        self.merge_into_working(working, source, records, conflicts)
    }

    fn merge_into_working(
        &self,
        working: Arc<Symbol>,
        source: &Arc<Symbol>,
        records: &mut Vec<(Arc<Symbol>, Arc<Symbol>)>,
        conflicts: &mut Vec<(Arc<Symbol>, Arc<Symbol>)>,
    ) -> Arc<Symbol> {
        let w_mut = Arc::as_ptr(&working) as *mut Symbol;
        let s_mut = Arc::as_ptr(source) as *mut Symbol;
        unsafe {
            for d in &(*s_mut).declarations {
                if !(*w_mut).declarations.iter().any(|x| Arc::ptr_eq(x, d)) {
                    (*w_mut).declarations.push(Arc::clone(d));
                }
            }
            (*w_mut).flags |= (*s_mut).flags;
            records.push((Arc::clone(&working), Arc::clone(source)));
            merge_symbol_tables_into(
                self,
                &mut (*w_mut).members,
                &(*s_mut).members,
                records,
                conflicts,
            );
            merge_symbol_tables_into(
                self,
                &mut (*w_mut).exports,
                &(*s_mut).exports,
                records,
                conflicts,
            );
        }
        working
    }
}

fn merge_symbol_flags_conflict(target: &SymbolFlags, source: &SymbolFlags) -> bool { ::tsox_core::fntrace::enter("merge_symbol_flags_conflict");
    (*target & get_excluded_symbol_flags(*source)) != SymbolFlags::empty()
        && !(*source | *target).intersects(SymbolFlags::Assignment)
}

fn merge_symbol_tables_into(
    checker: &Checker,
    target: &mut SymbolTable,
    source: &SymbolTable,
    records: &mut Vec<(Arc<Symbol>, Arc<Symbol>)>,
    conflicts: &mut Vec<(Arc<Symbol>, Arc<Symbol>)>,
) { ::tsox_core::fntrace::enter("merge_symbol_tables_into");
    let entries: Vec<(String, Arc<Symbol>)> = source
        .entries
        .iter()
        .map(|(k, v)| (k.clone(), Arc::clone(v)))
        .collect();
    for (k, v) in entries {
        match target.entries.get(&k) {
            Some(existing) => {
                let merged =
                    checker.merge_declarations_into_rec(existing, &v, records, conflicts);
                target.entries.insert(k, merged);
            }
            None => {
                target.entries.insert(k, v);
            }
        }
    }
}
