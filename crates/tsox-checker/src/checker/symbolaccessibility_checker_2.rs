#![allow(unused_imports)]

use crate::checker::symbolaccessibility::*;
use tsox_frontend::ast::NodeData;

impl Checker {
    pub(crate) fn get_with_alternative_containers(
        &mut self,
        container: &Arc<Symbol>,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
    ) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_with_alternative_containers"); 
        let additional_containers: Vec<Arc<Symbol>> = container
            .declarations
            .iter()
            .filter_map(|d| {
                self.get_file_symbol_if_file_symbol_export_equals_container(d, container)
            })
            .collect();

        let reexport_containers = if enclosing_declaration.is_some() {
            self.get_alternative_containing_modules(symbol, enclosing_declaration)
        } else {
            Vec::new()
        };

        let object_literal_container =
            self.get_variable_declaration_of_object_literal(container, meaning);
        let left_meaning = get_qualified_left_meaning(meaning);

        if enclosing_declaration.is_some()
            && container.flags.intersects(left_meaning)
            && !self
                .get_accessible_symbol_chain(
                    container,
                    enclosing_declaration,
                    SymbolFlags::NAMESPACE,
                    false,
                )
                .is_empty()
        {
            let mut res = vec![Arc::clone(container)];
            res.extend(additional_containers.iter().cloned());
            res.extend(reexport_containers.iter().cloned());
            if let Some(olc) = object_literal_container {
                res.push(olc);
            }
            return res;
        }

        let mut variable_matches: Vec<Arc<Symbol>> = Vec::new();
        if meaning == SymbolFlags::VALUE
            && !container.flags.intersects(left_meaning)
            && container.flags.intersects(SymbolFlags::TYPE)
            && self
                .get_declared_type_of_symbol(container)
                .flags
                .intersects(TypeFlags::Object)
        {
            let tables = self.collect_symbol_tables_in_scope(enclosing_declaration);
            for info in &tables {
                let mut found = false;
                for s in info.table.entries.values() {
                    if s.flags.intersects(left_meaning)
                        && Arc::ptr_eq(
                            &self.get_type_of_symbol(s),
                            &self.get_declared_type_of_symbol(container),
                        )
                    {
                        variable_matches.push(Arc::clone(s));
                        found = true;
                    }
                }
                if found {
                    break;
                }
            }
            self.sort_symbols(&mut variable_matches);
        }

        let mut res: Vec<Arc<Symbol>> = Vec::new();
        res.extend(variable_matches);
        res.extend(additional_containers);
        res.push(Arc::clone(container));
        if let Some(olc) = object_literal_container {
            res.push(olc);
        }
        res.extend(reexport_containers);
        res
    }

    pub(crate) fn get_alternative_containing_modules(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
    ) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_alternative_containing_modules"); 
        let enclosing_declaration = match enclosing_declaration {
            Some(enc) => enc,
            None => return Vec::new(),
        };

        let containing_file = self.get_source_file_of_node(enclosing_declaration);
        let id = containing_file.as_ref().map(|f| f.id()).unwrap_or(0);

        if let Some(links) = self.symbol_container_links.get(symbol) {
            if let Some(existing) = links.extended_containers_by_file.get(&id) {
                return existing.clone();
            }
        }

        // Go getAlternativeContainingModules 先经宿主文件已有 import 臂
        //（symbolaccessibility.go:192-213）：宿主文件每个模块说明符解析到
        // 的模块，其（解析）导出含目标符号时该模块符号即再导出容器，命中
        // 即按文件缓存返回（keys.ts 的 @raymondfeng/pkg2 由此成为 IdType
        // 显示链根，链根说明符走现有 import 快路径呈非相对形态）；未命中
        // 才落全程序展开
        if let Some(file) = containing_file.clone()
            && let Some(file_symbol) = self.get_symbol_of_declaration(&file.node)
        {
            let specs = self.file_module_specifiers(&file);
            if !specs.is_empty() {
                let mut results: Vec<Arc<Symbol>> = Vec::new();
                for spec in &specs {
                    let Some(resolved) = self.resolve_module_spec_from(&file_symbol, spec) else {
                        continue;
                    };
                    if self
                        .get_alias_for_symbol_in_container(&resolved, symbol)
                        .is_some()
                    {
                        results.push(resolved);
                    }
                }
                if !results.is_empty() {
                    self.symbol_container_links
                        .get_or_default(symbol)
                        .extended_containers_by_file
                        .insert(id, results.clone());
                    return results;
                }
            }
        }

        let mut results: Vec<Arc<Symbol>> = Vec::new();

        if let Some(links) = self.symbol_container_links.get(symbol) {
            if let Some(ref extended) = links.extended_containers {
                return extended.clone();
            }
        }

        let other_files: Vec<Arc<SourceFile>> = self.files.clone();
        for file in &other_files {
            let sym = self.get_symbol_of_declaration(&file.node);
            if let Some(ref sym) = sym {
                let ref_sym = self.get_alias_for_symbol_in_container(sym, symbol);
                if ref_sym.is_some() {
                    results.push(Arc::clone(sym));
                }
            }
        }

        self.symbol_container_links
            .get_or_default(symbol)
            .extended_containers = Some(results.clone());
        results
    }

    /// Go SourceFile.Imports()：文件内全部模块说明符（import/export 声明
    /// 的字符串说明符），re-export 容器臂的解析对象
    fn file_module_specifiers(&self, file: &Arc<SourceFile>) -> Vec<String> {
        let NodeData::SourceFile(sfd) = &file.node.data else {
            return Vec::new();
        };
        let mut specs: Vec<String> = Vec::new();
        for stmt in sfd.statements.iter() {
            let spec = match &stmt.data {
                NodeData::ImportDeclaration(d) => Some(d.module_specifier.text()),
                NodeData::ExportDeclaration(d) => {
                    d.module_specifier.as_ref().map(|s| s.text())
                }
                _ => None,
            };
            if let Some(text) = spec {
                let text = text.trim_matches(['"', '\'', '`']).to_string();
                if !text.is_empty() {
                    specs.push(text);
                }
            }
        }
        specs
    }

    pub(crate) fn get_variable_declaration_of_object_literal(
        &self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_variable_declaration_of_object_literal"); 
        if !meaning.intersects(SymbolFlags::VALUE) {
            return None;
        }
        if symbol.declarations.is_empty() {
            return None;
        }
        let first_decl = &symbol.declarations[0];
        let _parent = first_decl.parent().as_ref()?;

        None
    }

    pub(crate) fn get_external_module_container(
        &self,
        declaration: &Arc<Node>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_external_module_container"); 
        if has_external_module_symbol(declaration) {
            return self.get_symbol_of_declaration(declaration);
        }

        let mut node = declaration.parent();
        while let Some(n) = node {
            if has_external_module_symbol(&n) {
                return self.get_symbol_of_declaration(&n);
            }
            node = n.parent();
        }
        None
    }

    pub(crate) fn get_external_module_container_of_symbol(
        &self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_external_module_container_of_symbol"); 
        for d in &symbol.declarations {
            if let Some(sym) = self.get_external_module_container(d) {
                return Some(sym);
            }
        }
        None
    }

    pub(crate) fn get_file_symbol_if_file_symbol_export_equals_container(
        &mut self,
        d: &Arc<Node>,
        container: &Arc<Symbol>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_file_symbol_if_file_symbol_export_equals_container");
        let file_symbol = self.get_external_module_container(d)?;
        let exported = file_symbol
            .exports
            .get(INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)?;
        if self
            .get_symbol_if_same_reference(exported, container)
            .is_some()
        {
            Some(file_symbol)
        } else {
            None
        }
    }

    pub(crate) fn get_containers_of_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
        include_synthetic_containers: bool,
    ) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_containers_of_symbol");
        // Go getContainersOfSymbol（symbolaccessibility.go:296-299）只认
        // getParentOfSymbol = symbol.Parent：模块文件的非导出顶层声明在 Go
        // binder（binder.go:426-427 declareModuleMember 传 nil /*parent*/）落
        // 容器 locals 且 parent=nil，无 parent 即落下方 candidates 分支并经
        // getAliasForSymbolInContainer 的 exports-only 过滤
        //（symbolaccessibility.go:356-386，跨模块非导出名被剔除）。树回溯按
        // exports∪members 合成容器是 Go 无的私设臂，会把跨模块非导出符号
        // 误挂模块容器，is_any_symbol_accessible 随经 allow_modules 臂
        //（symbolaccessibility.go:49-63）直返 Accessible，nodebuilder 别名
        // 门（nodebuilderimpl.go:3552）误开呈 import("…").Alias；Go 同位
        // isSymbolAccessibleWorker 落异模块臂 CannotBeNamed（:889-904），
        // 别名按目标展开——故可及性路径恒 parent-only（include_synthetic_
        // containers=false）；合成臂仅由显示链调用方启用，兜底函数局部类等
        // parent 链缺失符号的限定名（见下方 Block 认领臂注释）
        let container = if include_synthetic_containers {
            self.get_parent_of_symbol(symbol)
                .or_else(|| {
                    // Go oracle：宿主限定段仅呈于 Type 意（实例型引用，base.
                    // Base / Mixin1.(Anonymous class)），Value 意（typeof 显示）
                    // 保持裸名（baseExpressionTypeParameters 行21/22 对照）
                    if meaning.intersects(SymbolFlags::TYPE) {
                        self.container_of_anonymous_class_symbol(symbol)
                    } else {
                        None
                    }
                })
                .or_else(|| {
            let symbol_map = self.program.symbol_map();
            symbol.declarations.first().and_then(|decl| {
                let merged_id = self.get_merged_symbol(symbol).id();
                let mut cur = decl.parent();
                while let Some(n) = cur {
                    let container_node = match n.kind {
                        SyntaxKind::ModuleDeclaration => Arc::clone(&n),
                        SyntaxKind::SourceFile => {
                            if Checker::is_external_or_common_js_module(&n) {
                                Arc::clone(&n)
                            } else {
                                return None;
                            }
                        }
                        // 函数体块透明穿越：函数局部声明（局部类等）的显示
                        // 容器是宿主函数符号，限定链补该段（Go oracle 对
                        // extends base<T>() 的类实例型呈 base.Base，
                        // baseExpressionTypeParameters .types 首分歧）。
                        // 认领表按 Go binder 双源核对：函数体/静态块体 Block
                        // 为 ContainerFlagsNone（binder.go:2762-2767），体内
                        // 声明的 blockScopeContainer 即宿主函数节点，
                        // declareSymbol 落 GetLocals(函数)（binder.go:1257）；
                        // 非函数体的普通块自身即块作用域容器，声明落
                        // GetLocals(块)（binder.go:1600-1602）
                        SyntaxKind::Block => {
                            let mut owner = None;
                            let mut fn_cur = n.parent();
                            while let Some(cur) = fn_cur {
                                if tsox_frontend::ast::is_function_like_kind(cur.kind) {
                                    owner = Some(Arc::clone(&cur));
                                    break;
                                }
                                fn_cur = cur.parent();
                            }
                            let fn_locals_hit = owner
                                .as_ref()
                                .and_then(|f| symbol_map.locals_of(f))
                                .and_then(|table| table.get(&symbol.name))
                                .is_some_and(|s| {
                                    Arc::ptr_eq(s, symbol)
                                        || self.get_merged_symbol(s).id() == merged_id
                                });
                            if !fn_locals_hit {
                                let block_hit = symbol_map
                                    .locals_of(&n)
                                    .and_then(|table| table.get(&symbol.name))
                                    .is_some_and(|s| {
                                        Arc::ptr_eq(s, symbol)
                                            || self.get_merged_symbol(s).id() == merged_id
                                    });
                                if !block_hit {
                                    return None;
                                }
                            }
                            let Some(f) = owner else {
                                return None;
                            };
                            let Some(container_sym) =
                                symbol_map.symbol_of(&f).map(Arc::clone)
                            else {
                                return None;
                            };
                            return Some(container_sym);
                        }
                        SyntaxKind::CaseClause | SyntaxKind::DefaultClause => return None,
                        _ if tsox_frontend::ast::is_function_like_kind(n.kind) => return None,
                        _ => {
                            cur = n.parent();
                            continue;
                        }
                    };
                    let Some(container_sym) =
                        symbol_map.symbol_of(&container_node).map(Arc::clone)
                    else {
                        return None;
                    };
                    // 容器认领显示链（include_synthetic_containers=true）查
                    // exports∪locals 双表：Go 双符号 binder（binder.go
                    // declareModuleMember 389-427）export 成员拆 local（含
                    // 非导出声明合并的语义位，parent=nil）+ export 符号
                    // （parent=模块），显示链容器上爬由 export 符号 parent
                    // 承载；本仓单符号世界显示位渲染的是合并 local 符号
                    // （decl 含非导出+导出双声明），模块容器在文件符号
                    // locals 表——mergedDeclarationExports 的 >d 呈
                    // import("./mergedDeclarationExports").d 依赖该认领。
                    // 可及性路径（false）维持 exports-only：members 镜像表
                    // 不构成可及性主张，跨模块非导出别名误挂模块容器会使
                    // is_any_symbol_accessible 经 allow_modules 臂直返
                    // Accessible，nodebuilder 别名门（nodebuilderimpl.go:3552）
                    // 误开呈 import("…").Alias（Go 同位 isSymbolAccessible-
                    // Worker 落异模块臂 CannotBeNamed :889-904，别名按目标
                    // 展开）
                    let declared = if include_synthetic_containers {
                        container_sym
                            .exports
                            .get(&symbol.name)
                            .or_else(|| container_sym.members.get(&symbol.name))
                    } else {
                        container_sym.exports.get(&symbol.name)
                    };
                    if let Some(declared) = declared {
                        if Arc::ptr_eq(declared, symbol)
                            || self.get_merged_symbol(declared).id() == merged_id
                        {
                            return Some(container_sym);
                        }
                    }
                    return None;
                }
                None
            })
            })
        } else {
            self.get_parent_of_symbol(symbol)
        };

        if let Some(ref container) = container {
            if !symbol.flags.intersects(SymbolFlags::TypeParameter) {
                return self.get_with_alternative_containers(
                    container,
                    symbol,
                    enclosing_declaration,
                    meaning,
                );
            }
        }

        let mut candidates: Vec<Arc<Symbol>> = Vec::new();
        for d in &symbol.declarations {
            if let Some(ref parent) = d.parent() {
                if has_non_global_augmentation_external_module_symbol(parent) {
                    if let Some(sym) = self.get_symbol_of_declaration(parent) {
                        if !candidates.iter().any(|c| c.id() == sym.id()) {
                            candidates.push(sym);
                        }
                    }
                    continue;
                }
            }
        }

        if candidates.is_empty() {
            return Vec::new();
        }

        let mut best_containers: Vec<Arc<Symbol>> = Vec::new();
        let mut alternative_containers: Vec<Arc<Symbol>> = Vec::new();
        for container in &candidates {
            if self
                .get_alias_for_symbol_in_container(container, symbol)
                .is_none()
            {
                continue;
            }
            let all_alts = self.get_with_alternative_containers(
                container,
                symbol,
                enclosing_declaration,
                meaning,
            );
            if all_alts.is_empty() {
                continue;
            }
            best_containers.push(Arc::clone(&all_alts[0]));
            if all_alts.len() > 1 {
                alternative_containers.extend(all_alts[1..].iter().cloned());
            }
        }
        best_containers.extend(alternative_containers);
        best_containers
    }
}
