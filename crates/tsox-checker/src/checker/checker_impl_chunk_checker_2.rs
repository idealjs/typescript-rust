#![allow(unused_imports)]

use crate::checker::checker_impl_chunk::*;
use tsox_frontend::ast::SymbolTable;

impl Checker {
    pub(crate) fn merge_global_symbols(&mut self, dst: &Arc<Symbol>, src: &Arc<Symbol>) {
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
        }
    }

    pub(crate) fn populate_globals(&mut self) {
        let script_files: Vec<Arc<SourceFile>> = self
            .files
            .iter()
            .filter(|f| f.external_module_indicator.is_none())
            .cloned()
            .collect();
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
            // Go createGlobals 只并各文件 exports；别名（import 子句）留在文件
            // 局部表，不参与全局合并
            for (name, sym) in member_entries
                .into_iter()
                .chain(local_entries)
                .filter(|(_, sym)| !sym.flags.contains(SymbolFlags::Alias))
            {
                self.merge_global_entry(&name, &sym);
            }
        }

        // Go initializeTypeChecker：UMD 全局（export as namespace）并入
        // globals（first-in-wins，仅外部模块声明文件可产生）
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

        let mut global_aug_members: Vec<(String, Arc<Symbol>)> = Vec::new();
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
                    global_aug_members.extend(
                        module_sym
                            .exports
                            .iter()
                            .map(|(k, v)| (k.clone(), Arc::clone(v))),
                    );
                }
            }
        }
        for (name, sym) in global_aug_members {
            let existing = self.globals.get(&name).cloned();
            match existing {
                Some(existing) => {
                    if !self.report_global_merge_conflict(&existing, &sym) {
                        self.merge_global_symbols(&existing, &sym);
                    }
                }
                None => {
                    self.globals.insert(name, sym);
                }
            }
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

        self.report_missing_global_types();
    }

    // Go addUndefinedToGlobalsOrErrorOnRedeclaration：globals 已有用户声明的
    // undefined 时逐非类型声明报 2397，否则并入内建 undefined 符号
    fn add_undefined_to_globals_or_error_on_redeclaration(&mut self) {
        let name = self
            .undefined_symbol
            .as_ref()
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "undefined".to_string());
        match self.globals.get(&name).cloned() {
            Some(target) => {
                for d in &target.declarations {
                    let is_type_declaration = matches!(
                        d.kind,
                        tsox_frontend::ast::SyntaxKind::TypeParameter
                            | tsox_frontend::ast::SyntaxKind::ClassDeclaration
                            | tsox_frontend::ast::SyntaxKind::InterfaceDeclaration
                            | tsox_frontend::ast::SyntaxKind::TypeAliasDeclaration
                            | tsox_frontend::ast::SyntaxKind::EnumDeclaration
                    );
                    if !is_type_declaration {
                        let loc = d.name().map(|n| n.loc).unwrap_or(d.loc);
                        let file = self.get_source_file_of_node(d);
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            loc,
                            tsox_core::diagnostics::messages_generated::
                                DECLARATION_NAME_CONFLICTS_WITH_BUILT_IN_GLOBAL_IDENTIFIER_0,
                            vec![name.clone()],
                        ));
                    }
                }
            }
            None => {
                if let Some(undef) = self.undefined_symbol.clone() {
                    self.globals.insert(name, undef);
                }
            }
        }
    }

    /// Go mergeModuleAugmentation（非 global）：增广模块的导出并入目标模块符号；
    /// 目标带 export * 时先并入 re-export 解析出的目标符号（声明合并）
    fn merge_module_augmentations(&mut self) {
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
            // 将 export= 经 resolveAlias 解到目标符号（foo 等）再判 Namespace
            let main_module = self.resolve_external_module_symbol_go(&main_module);
            if !main_module.flags.intersects(SymbolFlags::NAMESPACE) {
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
            if let Some(locals) = symbol_map.locals_of(&module_node) {
                for (k, v) in locals.iter() {
                    if !aug_entries.iter().any(|(ek, _)| ek == k) {
                        aug_entries.push((k.clone(), Arc::clone(v)));
                    }
                }
            }
            if aug_entries.is_empty() {
                continue;
            }
            let mut star_merged: std::collections::HashMap<String, Arc<Symbol>> =
                std::collections::HashMap::new();
            if main_module.exports.get(INTERNAL_SYMBOL_NAME_EXPORT_STAR).is_some() {
                let resolved = self.get_exports_of_module_table(&main_module);
                for (key, value) in &aug_entries {
                    if main_module.exports.get(key).is_none()
                        && let Some(target) = resolved.get(key)
                        && !Arc::ptr_eq(target, value)
                    {
                        self.merge_augmentation_symbols(target, value);
                        star_merged.insert(key.clone(), Arc::clone(target));
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
                        self.merge_augmentation_symbols(&base, value);
                    }
                }
            }
            let m_mut = Arc::as_ptr(&main_module) as *mut Symbol;
            for (key, value) in aug_entries {
                unsafe {
                    match (*m_mut).exports.get(&key) {
                        Some(existing) => {
                            self.merge_augmentation_symbols(existing, &value);
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

    pub(crate) fn report_missing_global_types(&mut self) {
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
    fn merge_augmentation_symbols(&mut self, target: &Arc<Symbol>, source: &Arc<Symbol>) {
        let mut records = Vec::new();
        merge_declarations_into_rec(target, source, &mut records);
        for (t, s) in records {
            self.record_merged_symbol(&t, &s);
        }
    }
}

fn merge_declarations_into(target: &Arc<Symbol>, source: &Arc<Symbol>) {
    let mut records = Vec::new();
    merge_declarations_into_rec(target, source, &mut records);
}

fn merge_declarations_into_rec(
    target: &Arc<Symbol>,
    source: &Arc<Symbol>,
    records: &mut Vec<(Arc<Symbol>, Arc<Symbol>)>,
) {
    let t_mut = Arc::as_ptr(target) as *mut Symbol;
    let s_mut = Arc::as_ptr(source) as *mut Symbol;
    unsafe {
        for d in &(*s_mut).declarations {
            if !(*t_mut).declarations.iter().any(|x| Arc::ptr_eq(x, d)) {
                (*t_mut).declarations.push(Arc::clone(d));
            }
        }
        (*t_mut).flags |= (*s_mut).flags;
        records.push((Arc::clone(target), Arc::clone(source)));
        merge_symbol_tables_into(&mut (*t_mut).members, &(*s_mut).members, records);
        merge_symbol_tables_into(&mut (*t_mut).exports, &(*s_mut).exports, records);
    }
}

fn merge_symbol_tables_into(
    target: &mut SymbolTable,
    source: &SymbolTable,
    records: &mut Vec<(Arc<Symbol>, Arc<Symbol>)>,
) {
    let entries: Vec<(String, Arc<Symbol>)> = source
        .entries
        .iter()
        .map(|(k, v)| (k.clone(), Arc::clone(v)))
        .collect();
    for (k, v) in entries {
        match target.entries.get(&k) {
            Some(existing) => merge_declarations_into_rec(existing, &v, records),
            None => {
                target.entries.insert(k, v);
            }
        }
    }
}
