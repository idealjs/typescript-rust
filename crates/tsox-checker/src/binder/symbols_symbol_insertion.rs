#![allow(unused_imports)]

use crate::binder::symbols::*;

impl Binder {
    pub(crate) fn module_member_is_exported(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("module_member_is_exported"); 
        node.kind == SyntaxKind::ExportSpecifier
            || self
                .get_combined_modifier_flags(node)
                .contains(ModifierFlags::Export)
            // Go declareModuleMember：ambient 容器的 ExportContext 内隐式导出
            || self
                .container
                .as_ref()
                .is_some_and(|c| c.flags.contains(NodeFlags::ExportContext))
    }

    // Go declareModuleMember（binder.go:394-410）alias 臂分表谓词：
    // export specifier 与带 export 修饰的 import equals 落容器 exports，
    // 其余别名声明落容器 locals——ExportContext 隐式导出不参与 alias 分表
    pub(crate) fn module_member_alias_declaration_exports(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("module_member_alias_declaration_exports"); 
        match node.kind {
            SyntaxKind::ExportSpecifier => true,
            SyntaxKind::ImportEqualsDeclaration => self
                .get_combined_modifier_flags(node)
                .contains(ModifierFlags::Export),
            _ => false,
        }
    }

    /// Go 成员声明语境的 parent 实参一律取 container.Symbol()
    /// （declareModuleMember binder.go:390/411/416/424、bindExportAssignment
    /// binder.go:905-912、bindExportDeclaration binder.go:891-898），Rust 以
    /// parent_symbol 承载；ModuleBlock 等无符号容器进 bind_container 后
    /// parent_symbol 被清空而 container 不前移，此处从 container 节点直取
    /// 符号、parent_symbol 兜底
    pub(crate) fn container_symbol_or_parent(&self) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("container_symbol_or_parent");
        self.container
            .as_ref()
            .and_then(|c| self.symbol_map.symbol_of(c).cloned())
            .or_else(|| self.parent_symbol.clone())
    }

    pub(crate) fn insert_symbol_into_container(
        &mut self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
        name: &str,
        var_hoist_container: &Option<Arc<Node>>,
    ) { ::tsox_core::fntrace::enter("insert_symbol_into_container"); 
        if let Some(container) = &self.container {
            // Go 声明分派两路（bindBlockScopedDeclaration binder.go:1313-1327 /
            // declareSymbolAndAddToSymbolTable binder.go:447-466）：块作用域声明
            // 嵌套块容器内入块自身 locals；var/命名空间/别名等按 container.Kind
            // 恒入模块成员表（嵌套块内仍是模块成员）
            let module_scoped_member = container.kind == SyntaxKind::ModuleDeclaration
                && (Self::declaration_is_var(node)
                    || !Self::is_block_scope_dispatched_declaration(node)
                    || self
                        .block_scope_container
                        .as_ref()
                        .is_none_or(|b| b.id() == container.id()));
            if module_scoped_member {
                // Go declareModuleMember（binder.go:394-410）alias 臂先于
                // ExportContext：exports 仅收 export specifier 与带 export 修饰的
                // import equals，其余 import 别名（含 ambient ExportContext 下
                // 未加 export 的 import X = require(...)）恒落容器 locals；
                // locals/exports 分表是 getAccessibleSymbolChain 表序决胜
                // （someSymbolTableInScope 先 locals 后 exports，
                // symbolaccessibility.go:783-801）的前提
                let alias_exports = self.module_member_alias_declaration_exports(node);
                let alias_declaration = alias_exports
                    || matches!(
                        node.kind,
                        SyntaxKind::ExportSpecifier
                            | SyntaxKind::ImportEqualsDeclaration
                            | SyntaxKind::NamespaceImport
                            | SyntaxKind::ImportSpecifier
                    );
                if alias_declaration {
                    if alias_exports {
                        let module_parent = self.container_symbol_or_parent();
                        if let Some(parent_sym) = &module_parent {
                            let parent_sym_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                            let symbol_mut = Arc::as_ptr(symbol) as *mut Symbol;
                            unsafe {
                                (*parent_sym_mut)
                                    .exports
                                    .insert(name.to_string(), Arc::clone(&symbol));
                                // 命名空间成员挂父链（显示限定名用，对齐 tsc symbol.parent）
                                if (*symbol_mut).parent().is_none() {
                                    (*symbol_mut).set_parent(parent_sym);
                                }
                            }
                        }
                    } else if has_locals(container.kind) {
                        let locals = self
                            .symbol_map
                            .locals
                            .entry(container.id())
                            .or_insert_with(SymbolTable::new);
                        locals.insert(name.to_string(), Arc::clone(&symbol));
                    }
                } else {
                    let has_export = self.module_member_is_exported(node);
                    if has_export {
                        let module_parent = self.container_symbol_or_parent();
                        if let Some(parent_sym) = &module_parent {
                            let parent_sym_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                            let symbol_mut = Arc::as_ptr(symbol) as *mut Symbol;
                            unsafe {
                                (*parent_sym_mut)
                                    .exports
                                    .insert(name.to_string(), Arc::clone(&symbol));
                                // 命名空间成员挂父链（显示限定名用，对齐 tsc symbol.parent）
                                if (*symbol_mut).parent().is_none() {
                                    (*symbol_mut).set_parent(parent_sym);
                                }
                            }
                        }

                        if has_locals(container.kind) {
                            let locals = self
                                .symbol_map
                                .locals
                                .entry(container.id())
                                .or_insert_with(SymbolTable::new);
                            locals.insert(name.to_string(), Arc::clone(&symbol));
                        }
                    } else if has_locals(container.kind) {
                        let locals = self
                            .symbol_map
                            .locals
                            .entry(container.id())
                            .or_insert_with(SymbolTable::new);
                        locals.insert(name.to_string(), Arc::clone(&symbol));
                    }
                }
            } else if var_hoist_container.is_none()
                && let Some(block_container) = &self.block_scope_container
                && self
                    .container
                    .as_ref()
                    .is_none_or(|c| c.id() != block_container.id())
            {
                let locals = self
                    .symbol_map
                    .locals
                    .entry(block_container.id())
                    .or_insert_with(SymbolTable::new);
                locals.insert(name.to_string(), Arc::clone(symbol));
            } else if is_function_like_locals_container(container.kind) {
                // Go declareSymbolAndAddToSymbolTable 的函数类容器分支：
                // 进容器节点 locals、parent 不挂。插进容器符号 members 会让
                // 重载声明的同名类型参数相互覆盖（符号表是按名字索引的）
                let locals = self
                    .symbol_map
                    .locals
                    .entry(container.id())
                    .or_insert_with(SymbolTable::new);
                locals.insert(name.to_string(), Arc::clone(&symbol));
            } else if let Some(parent_sym) = &self.parent_symbol {
                let parent_sym_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                let symbol_mut = Arc::as_ptr(symbol) as *mut Symbol;
                unsafe {
                    // Go declareClassMember：类的 static 成员进 parentSymbol.Exports
                    //（与实例成员分表，同名 static/实例为两个符号）
                    let parent_is_class = parent_sym.declarations.iter().any(|d| {
                        matches!(
                            d.kind,
                            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                        )
                    });
                    let is_static_member = node.has_syntactic_modifier(ModifierFlags::Static)
                        && matches!(
                            node.kind,
                            SyntaxKind::PropertyDeclaration
                                | SyntaxKind::MethodDeclaration
                                | SyntaxKind::GetAccessor
                                | SyntaxKind::SetAccessor
                        );
                    // export 语境的别名进 exports 表（对齐 Go declareModuleMember），
                    // 其余本地声明进 members
                    if matches!(
                        node.kind,
                        SyntaxKind::ExportSpecifier | SyntaxKind::NamespaceExportDeclaration
                    ) || (parent_is_class && is_static_member)
                    {
                        (*parent_sym_mut)
                            .exports
                            .insert(name.to_string(), Arc::clone(symbol));
                    } else {
                        (*parent_sym_mut)
                            .members
                            .insert(name.to_string(), Arc::clone(symbol));
                        // Go declareSourceFileMember：外部模块文件的顶层导出成员
                        // 同时进 exports；default 导出经 declareSymbolEx
                        // （binder.go:162-172，isDefaultExport && parent != nil
                        // 恒名 InternalSymbolNameDefault）以 "default" 入
                        // exports——export { zzz as default } 的
                        // declareSymbol(exports, Alias, AliasExcludes) 非冲突
                        // 合并（Alias 位并入，Specifier 声明挂导出符号）由此
                        // 获得 Interface 位，跨文件 Type 含义解析
                        // （getSymbol getSymbolFlagsEx 别名回退
                        // checker.go:2272-2288）依赖该位
                        let file_is_external_module = self
                            .current_source_file
                            .as_ref()
                            .is_some_and(|f| f.external_module_indicator.is_some());
                        let combined = self.get_combined_modifier_flags(node);
                        if file_is_external_module && combined.contains(ModifierFlags::Export) {
                            let export_name = if combined.contains(ModifierFlags::Default) {
                                tsox_frontend::ast::INTERNAL_SYMBOL_NAME_DEFAULT.to_string()
                            } else {
                                name.to_string()
                            };
                            // Go declareSymbolEx（binder.go:190-194、283-290）：查重、
                            // 合并与插表同用一名——default 导出重写名为 default 而声明
                            // 名不是时（匿名 default 成员声明名为空），前置查重键与
                            // exports 键不一致，既有条目须按 addDeclarationToSymbol
                            //（binder.go:2690）合流（flags 并入、声明追加、值声明补挂、
                            // 节点符号后写覆盖），整表替换会顶掉先落表的 default 符号
                            //（export default interface A + export default function(){}
                            // 的 Function|Interface 合流即赖此）
                            if combined.contains(ModifierFlags::Default)
                                && export_name != name
                                && let Some(existing) =
                                    (*parent_sym_mut).exports.get(&export_name).cloned()
                            {
                                let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                                let new_sym_ptr = Arc::as_ptr(symbol) as *mut Symbol;
                                unsafe {
                                    (*existing_mut).flags |= (*new_sym_ptr).flags;
                                    (*existing_mut)
                                        .declarations
                                        .extend((*new_sym_ptr).declarations.iter().cloned());
                                    if (*existing_mut).value_declaration.is_none()
                                        && (*new_sym_ptr).value_declaration.is_some()
                                    {
                                        (*existing_mut).value_declaration =
                                            (*new_sym_ptr).value_declaration.clone();
                                    }
                                }
                                self.symbol_map.set_symbol(node, Arc::clone(&existing));
                            } else {
                                (*parent_sym_mut)
                                    .exports
                                    .insert(export_name, Arc::clone(symbol));
                            }
                        }
                    }
                    // Go declareSymbolAndAddToSymbolTable（binder.go:442-450）：
                    // SourceFile 容器走 declareSourceFileMember（binder.go:435-441）
                    // → declareModuleMember（binder.go:391/418/428），非导出文件
                    // 顶层声明 declareSymbol(locals, nil /*parent*/) 不挂父链，
                    // 父链只落在导出符号上（单符号架构下以 Export 修饰符区分）；
                    // 类/命名空间成员父链恒挂容器（binder.go:430-433）
                    let container_is_source_file = self
                        .container
                        .as_ref()
                        .is_some_and(|c| c.kind == SyntaxKind::SourceFile);
                    let node_is_exported = self
                        .get_combined_modifier_flags(node)
                        .contains(ModifierFlags::Export);
                    if !container_is_source_file || node_is_exported {
                        (*symbol_mut).set_parent(parent_sym);
                    }
                }
            } else if let Some(hoist) = &var_hoist_container {
                match hoist.kind {
                    SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration => {
                        if let Some(sym) = self.symbol_map.symbol_of(hoist) {
                            let sym_mut = Arc::as_ptr(&sym) as *mut Symbol;
                            unsafe {
                                (*sym_mut)
                                    .members
                                    .insert(name.to_string(), Arc::clone(&symbol));
                            }
                        }
                    }
                    _ => {
                        let locals = self
                            .symbol_map
                            .locals
                            .entry(hoist.id())
                            .or_insert_with(SymbolTable::new);
                        locals.insert(name.to_string(), Arc::clone(&symbol));
                    }
                }
            } else if let Some(block_container) = &self.block_scope_container {
                let container_id = block_container.id();
                let locals = self
                    .symbol_map
                    .locals
                    .entry(container_id)
                    .or_insert_with(SymbolTable::new);
                locals.insert(name.to_string(), Arc::clone(&symbol));
            }
        }
    }
}
