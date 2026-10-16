#![allow(unused_imports)]

use crate::checker::symbolaccessibility::*;
use tsox_frontend::ast::ModifierFlags;

impl Checker {
    pub(crate) fn get_candidate_list_for_symbol(
        &mut self,
        ctx: &AccessibleSymbolChainContext,
        symbol_from_symbol_table: &Arc<Symbol>,
        resolved_imported_symbol: &Arc<Symbol>,
        ignore_qualification: bool,
    ) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_candidate_list_for_symbol"); 
        if self.is_accessible(
            ctx,
            symbol_from_symbol_table,
            Some(resolved_imported_symbol),
            ignore_qualification,
        ) {
            return vec![Arc::clone(symbol_from_symbol_table)];
        }

        // Go getCandidateListForSymbol（symbolaccessibility.go:662）经
        // getExportsOfSymbol（checker.go:16867-16876）：Module 位目标展开
        // export * 后再寻可及链段
        let candidate_table = self.get_exports_of_symbol_resolved(resolved_imported_symbol);
        let candidate_table_id = symbol_table_id_from_resolved_exports(resolved_imported_symbol);
        let accessible_symbols_from_exports = self.get_accessible_symbol_chain_from_symbol_table(
            ctx,
            &candidate_table,
            candidate_table_id,
            true,
            false,
        );
        if accessible_symbols_from_exports.is_empty() {
            return Vec::new();
        }
        if !self.can_qualify_symbol(
            ctx,
            symbol_from_symbol_table,
            get_qualified_left_meaning(ctx.meaning),
        ) {
            return Vec::new();
        }
        let mut result = vec![Arc::clone(symbol_from_symbol_table)];
        result.extend(accessible_symbols_from_exports);
        result
    }

    pub(crate) fn is_accessible(
        &mut self,
        ctx: &AccessibleSymbolChainContext,
        symbol_from_symbol_table: &Arc<Symbol>,
        resolved_alias_symbol: Option<&Arc<Symbol>>,
        ignore_qualification: bool,
    ) -> bool { ::tsox_core::fntrace::enter("is_accessible"); 
        let mut like_symbols = false;
        if let Some(ref resolved) = resolved_alias_symbol {
            if ctx.symbol.id() == resolved.id() {
                like_symbols = true;
            }
        }
        if ctx.symbol.id() == symbol_from_symbol_table.id() {
            like_symbols = true;
        }
        let symbol = self.get_merged_symbol(&ctx.symbol);
        if let Some(ref resolved) = resolved_alias_symbol {
            let merged_resolved = self.get_merged_symbol(resolved);
            if symbol.id() == merged_resolved.id() {
                like_symbols = true;
            }
        }
        let merged_from_table = self.get_merged_symbol(symbol_from_symbol_table);
        if symbol.id() == merged_from_table.id() {
            like_symbols = true;
        }
        if !like_symbols {
            return false;
        }

        !symbol_from_symbol_table
            .declarations
            .iter()
            .any(has_non_global_augmentation_external_module_symbol)
            && (ignore_qualification
                || self.can_qualify_symbol(
                    ctx,
                    &self.get_merged_symbol(symbol_from_symbol_table),
                    ctx.meaning,
                ))
    }

    pub(crate) fn can_qualify_symbol(
        &mut self,
        ctx: &AccessibleSymbolChainContext,
        symbol_from_symbol_table: &Arc<Symbol>,
        meaning: SymbolFlags,
    ) -> bool { ::tsox_core::fntrace::enter("can_qualify_symbol"); 
        if !self.needs_qualification(
            symbol_from_symbol_table,
            ctx.enclosing_declaration.as_ref(),
            meaning,
        ) {
            return true;
        }

        if let Some(ref parent) = symbol_from_symbol_table.parent() {
            let parent_ctx = AccessibleSymbolChainContext {
                symbol: Arc::clone(parent),
                enclosing_declaration: ctx.enclosing_declaration.clone(),
                meaning: get_qualified_left_meaning(meaning),
                use_only_external_aliasing: ctx.use_only_external_aliasing,
                visited_symbol_tables_map: RefCell::new(
                    ctx.visited_symbol_tables_map.borrow().clone(),
                ),
            };
            !self.get_accessible_symbol_chain_ex(parent_ctx).is_empty()
        } else {
            false
        }
    }

    pub(crate) fn needs_qualification(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
    ) -> bool { ::tsox_core::fntrace::enter("needs_qualification"); 
        let mut qualify = false;
        let tables = self.collect_symbol_tables_in_scope(enclosing_declaration);
        for info in &tables {
            let res = match info.table.get(&symbol.name) {
                Some(r) => r,
                None => continue,
            };
            let mut symbol_from_symbol_table = self.get_merged_symbol(res);

            if symbol_from_symbol_table.id() == symbol.id() {
                return false;
            }

            let should_resolve_alias = symbol_from_symbol_table
                .flags
                .intersects(SymbolFlags::Alias)
                && symbol_from_symbol_table
                    .declarations
                    .iter()
                    .all(|d| d.kind != SyntaxKind::ExportSpecifier);
            if should_resolve_alias {
                symbol_from_symbol_table = self.resolve_alias(&symbol_from_symbol_table);
            }
            let mut flags = symbol_from_symbol_table.flags;
            if should_resolve_alias {
                flags = self.get_symbol_flags(&symbol_from_symbol_table);
            }
            if flags.intersects(meaning) {
                qualify = true;
                break;
            }
        }
        qualify
    }

    // Go declareModuleMember（binder.go:406-416）对模块容器顶层 export 声明拆
    // local/export 双符号：类型侧引用（getSymbolAtLocation→getSymbolOfDeclaration
    // →node.Symbol() 最终绑 export 符号，checker.go:33833-33839）持 export 身份，
    // needsQualification（symbolaccessibility.go:729-759）遍历 enclosing 作用域
    // 符号表命中 local 身份（≠）后按 local.flags&meaning 判同名遮蔽，
    // getSymbolChain（nodebuilderimpl.go:1152-1186）据此上爬模块容器，
    // symbolToTypeNode（nodebuilderimpl.go:681-760）链根为模块走 ImportTypeNode。
    // 本仓顶层声明走单符号快路径（bind_walk_binder.rs:189 ClassDeclaration→
    // declare_symbol 合并声明进单符号，无 export 符号），needs_qualification 命中
    // 自身恒 false。本谓词按单符号复算该遮蔽：local 身份位 = 非 export 声明的
    // 原语义位 ∪ export 声明的 ExportValue（原位含 VALUE 时），与 meaning 相交
    // 即需限定。仅限 .types 显示链上爬判定（get_display_symbol_chain）消费：
    // 同文件限定（Go 只在 enclosing 作用域链内命中 local）；纯 export class/
    // interface 的 local 位（ExportValue/空）对 Type 不命中维持裸名（Go 同），
    // export function/enum 的 typeof（Value）同不命中；位映射未覆盖的声明
    //（变量/namespace）保守放弃模拟。local 位只由父容器为源文件的声明构成
    //（Go binder.go:419-426 local 仅 ExportValue、428 非导出成员落 GetLocals；
    // module augmentation 声明按 binder.go:410-413 并入模块符号 exports 侧，
    // 不进文件 locals——跨文件增补 interface 合并不得给 local 位加 Interface）
    pub(crate) fn display_export_shadowing(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
    ) -> bool { ::tsox_core::fntrace::enter("display_export_shadowing");
        let Some(enclosing) = enclosing_declaration else {
            return false;
        };
        let Some(ctx_file) = Self::enclosing_source_file_node(enclosing) else {
            return false;
        };
        let Some(decl_file) = symbol
            .declarations
            .first()
            .and_then(Self::enclosing_source_file_node)
        else {
            return false;
        };
        if ctx_file.id() != decl_file.id()
            || !Checker::is_external_or_common_js_module(&decl_file)
        {
            return false;
        }
        let mut has_export = false;
        let mut local = SymbolFlags::empty();
        for d in &symbol.declarations {
            if !d.parent().is_some_and(|p| p.kind == SyntaxKind::SourceFile) {
                continue;
            }
            let Some(includes) = Self::module_member_declaration_includes(d) else {
                return false;
            };
            if self
                .get_combined_modifier_flags(d)
                .contains(ModifierFlags::Export)
            {
                has_export = true;
                if includes.intersects(SymbolFlags::VALUE) {
                    local |= SymbolFlags::ExportValue;
                }
            } else {
                local |= includes;
            }
        }
        has_export && local.intersects(meaning)
    }

    // bind_walk_binder.rs 顶层声明快路径的 includes 硬编码表同源
    //（ClassDeclaration→Class、InterfaceDeclaration→Interface、EnumDeclaration→
    // Const/RegularEnum、FunctionDeclaration→Function、TypeAliasDeclaration→
    // TypeAlias）；未覆盖 kind 返 None
    fn module_member_declaration_includes(d: &Arc<Node>) -> Option<SymbolFlags> {
        match d.kind {
            SyntaxKind::ClassDeclaration => Some(SymbolFlags::Class),
            SyntaxKind::InterfaceDeclaration => Some(SymbolFlags::Interface),
            SyntaxKind::TypeAliasDeclaration => Some(SymbolFlags::TypeAlias),
            SyntaxKind::EnumDeclaration => Some(if tsox_frontend::ast::mig::x6a::is_enum_const(d) {
                SymbolFlags::ConstEnum
            } else {
                SymbolFlags::RegularEnum
            }),
            SyntaxKind::FunctionDeclaration => Some(SymbolFlags::Function),
            _ => None,
        }
    }

    fn enclosing_source_file_node(n: &Arc<Node>) -> Option<Arc<Node>> {
        let mut cur = Some(Arc::clone(n));
        while let Some(c) = cur {
            if c.kind == SyntaxKind::SourceFile {
                return Some(c);
            }
            cur = c.parent();
        }
        None
    }

    // Go trySymbolTable（symbolaccessibility.go:570-573/588-612）按表序决胜：
    // 首个产出结果的表内，直名命中（symbols[name] 解到目标且 isAccessible——
    // needsQualification 为 false）先于同表别名扫描。本谓词按同一表序判定
    // 「直名命中先于别名链」：var a + import a = M 的 a 可解到命名空间 M，
    // M 的显示链取自身名（typeof M）；constEnumOnlyModuleMerging 的 B 内
    // O 先于 globals 的 Outer 出现，别名链合法（typeof O），顺序敏感不误伤
    pub(crate) fn symbol_accessible_by_own_name(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
    ) -> bool { ::tsox_core::fntrace::enter("symbol_accessible_by_own_name"); 
        // Go isAccessible：canQualifySymbol 失败（直名形态需再限定）时直名
        // 命中不可用，别名链按扫描结果决胜
        if self.needs_qualification(symbol, enclosing_declaration, meaning) {
            return false;
        }
        let merged_id = self.get_merged_symbol(symbol).id();
        for info in self.collect_symbol_tables_in_scope(enclosing_declaration) {
            if let Some(res) = info.table.get(&symbol.name) {
                let res_id = self.get_merged_symbol(res).id();
                if res_id == merged_id
                    || self.merged_symbols.get(&merged_id) == Some(&res_id)
                    || self.merged_symbols.get(&res_id) == Some(&merged_id)
                {
                    return true;
                }
            }
            for alias in self.get_symbol_table_aliases(&info.table, info.table_id) {
                if alias.name == INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
                    || alias.name == INTERNAL_SYMBOL_NAME_DEFAULT
                    || !alias.flags.intersects(SymbolFlags::Alias)
                {
                    continue;
                }
                let resolved = self.resolve_alias(&alias);
                if self.get_merged_symbol(&resolved).id() == merged_id {
                    return false;
                }
            }
        }
        false
    }

    pub(crate) fn collect_symbol_tables_in_scope(
        &mut self,
        enclosing_declaration: Option<&Arc<Node>>,
    ) -> Vec<SymbolTableInScope> { ::tsox_core::fntrace::enter("collect_symbol_tables_in_scope"); 
        let mut result: Vec<SymbolTableInScope> = Vec::new();
        let mut location = enclosing_declaration.cloned();

        while let Some(loc) = location {
            if can_have_locals(loc.kind) && !matches!(
                loc.kind,
                SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration
            ) {
                if let Some(locals) = self.program.symbol_map().locals_of(&loc) {
                    if !locals.is_empty() {
                        result.push(SymbolTableInScope {
                            table: Arc::new(locals.clone()),
                            table_id: symbol_table_id_from_locals(&loc),
                            ignore_qualification: false,
                            is_local_name_lookup: true,
                            scope_node: Some(Arc::clone(&loc)),
                        });
                    }
                }
            }

            match loc.kind {
                SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration => {
                    if loc.kind == SyntaxKind::SourceFile
                        && !Checker::is_external_or_common_js_module(&loc)
                    {
                        // Go 脚本文件跳过容器 exports 臂（symbolaccessibility.go:794-796），
                        // 但 binder 对脚本文件把顶层声明并进 globals
                        //（bindSourceFile→mergeSymbolTable），直名可达由 globals
                        // 表兑现。本仓按同一内容补一份文件级 scope 表（locals+members），
                        // 保住 trySymbolTable 的直名命中先于别名扫描的优先序
                        //（symbolaccessibility.go:570-573）：var a + import a = M
                        // 的别名 a 在作用域表内可解到命名空间 M 时，M 的显示链
                        // 仍取自身名（typeof M），不被别名链顶替
                        if let Some(sym) = self.get_symbol_of_declaration(&loc) {
                            let node_id = loc.id();
                            let scope = if let Some(cached) = self.scope_table_cache.get(&(node_id, 0u8)) {
                                Arc::clone(cached)
                            } else {
                                let mut scope = SymbolTable::new();
                                if let Some(locals) = self.program.symbol_map().locals_of(&loc) {
                                    for (k, v) in locals.iter() {
                                        scope.insert(k.clone(), Arc::clone(v));
                                    }
                                }
                                for (k, v) in sym.members.iter() {
                                    if scope.get(k).is_none() {
                                        scope.insert(k.clone(), Arc::clone(v));
                                    }
                                }
                                let scope = Arc::new(scope);
                                self.scope_table_cache.insert((node_id, 0u8), Arc::clone(&scope));
                                scope
                            };
                            if !scope.is_empty() {
                                result.push(SymbolTableInScope {
                                    table: scope,
                                    table_id: symbol_table_id_from_locals(&loc),
                                    ignore_qualification: false,
                                    is_local_name_lookup: true,
                                    scope_node: Some(Arc::clone(&loc)),
                                });
                            }
                        }
                    } else {
                        if let Some(sym) = self.get_symbol_of_declaration(&loc) {
                            // Go binder：外部模块容器顶层声明（含 namespace
                            // import 别名）统一落容器 locals，本仓 binder 分落
                            // locals（import 默认绑定）与容器符号 members——
                            // 二表合并充当文件 locals 可达面（locals 优先）
                            let node_id = loc.id();
                            let scope = if let Some(cached) = self.scope_table_cache.get(&(node_id, 0u8)) {
                                Arc::clone(cached)
                            } else {
                                let mut scope = SymbolTable::new();
                                if let Some(locals) = self.program.symbol_map().locals_of(&loc) {
                                    for (k, v) in locals.iter() {
                                        scope.insert(k.clone(), Arc::clone(v));
                                    }
                                }
                                for (k, v) in sym.members.iter() {
                                    if scope.get(k).is_none() {
                                        scope.insert(k.clone(), Arc::clone(v));
                                    }
                                }
                                let scope = Arc::new(scope);
                                self.scope_table_cache.insert((node_id, 0u8), Arc::clone(&scope));
                                scope
                            };
                            if !scope.is_empty() {
                                result.push(SymbolTableInScope {
                                    table: scope,
                                    table_id: symbol_table_id_from_locals(&loc),
                                    ignore_qualification: false,
                                    is_local_name_lookup: true,
                                    scope_node: Some(Arc::clone(&loc)),
                                });
                            }
                            if !sym.exports.is_empty() {
                                let exports = if let Some(cached) = self.scope_table_cache.get(&(node_id, 1u8)) {
                                    Arc::clone(cached)
                                } else {
                                    let exports = Arc::new(sym.exports.clone());
                                    self.scope_table_cache.insert((node_id, 1u8), Arc::clone(&exports));
                                    exports
                                };
                                result.push(SymbolTableInScope {
                                    table: exports,
                                    table_id: symbol_table_id_from_exports(&sym),
                                    ignore_qualification: false,
                                    is_local_name_lookup: true,
                                    scope_node: Some(Arc::clone(&loc)),
                                });
                            }
                        }
                    }
                }
                SyntaxKind::ClassDeclaration
                | SyntaxKind::ClassExpression
                | SyntaxKind::InterfaceDeclaration => {
                    if let Some(sym) = self.get_symbol_of_declaration(&loc) {
                        let mut table = SymbolTable::new();
                        for (key, member_symbol) in sym.members.entries.iter() {
                            if member_symbol
                                .flags
                                .intersects(SymbolFlags::TYPE.difference(SymbolFlags::Assignment))
                            {
                                table.insert(key.clone(), Arc::clone(member_symbol));
                            }
                        }
                        if !table.is_empty() {
                            result.push(SymbolTableInScope {
                                table: Arc::new(table),
                                table_id: symbol_table_id_from_members(&sym),
                                ignore_qualification: false,
                                is_local_name_lookup: false,
                                scope_node: Some(Arc::clone(&loc)),
                            });
                        }

                        if loc.kind == SyntaxKind::ClassExpression {
                            if let Some(name_table) = self.get_class_expression_name_table(&loc) {
                                if !name_table.is_empty() {
                                    result.push(SymbolTableInScope {
                                        table: Arc::new(name_table),
                                        table_id: symbol_table_id_from_locals(&loc),
                                        ignore_qualification: false,
                                        is_local_name_lookup: true,
                                        scope_node: Some(Arc::clone(&loc)),
                                    });
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            location = loc.parent();
        }

        if !self.globals.is_empty() {
            let globals = if let Some(cached) = self.scope_table_cache.get(&(0u64, 2u8)) {
                Arc::clone(cached)
            } else {
                let globals = Arc::new(self.globals.clone());
                self.scope_table_cache.insert((0u64, 2u8), Arc::clone(&globals));
                globals
            };
            result.push(SymbolTableInScope {
                table: globals,
                table_id: symbol_table_id_from_globals(),
                ignore_qualification: false,
                is_local_name_lookup: true,
                scope_node: None,
            });
        }

        result
    }

    pub(crate) fn get_class_expression_name_table(
        &mut self,
        location: &Arc<Node>,
    ) -> Option<SymbolTable> { ::tsox_core::fntrace::enter("get_class_expression_name_table"); 
        let node_id = location.id();

        if let Some(table) = self.class_expression_name_tables.get(&node_id) {
            return Some(table.clone());
        }

        let class_symbol = self.get_symbol_of_declaration(location)?;

        let name_text = class_symbol.name.clone();
        if name_text.is_empty() {
            return None;
        }
        let mut table = SymbolTable::new();
        table.insert(name_text, class_symbol);
        self.class_expression_name_tables
            .insert(node_id, table.clone());
        Some(table)
    }

    pub(crate) fn has_visible_declarations_with_aliases(
        &mut self,
        symbol: &Arc<Symbol>,
        _should_compute_aliases_to_make_visible: bool,
    ) -> Option<SymbolAccessibilityResult> { ::tsox_core::fntrace::enter("has_visible_declarations_with_aliases"); 
        self.has_visible_declarations(symbol)
    }
}
