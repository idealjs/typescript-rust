#![allow(unused_imports)]

use crate::checker::checker_resolve::*;

pub(crate) const ANCESTRY_CONTAINERS: &[SyntaxKind] = &[
    SyntaxKind::SourceFile,
    SyntaxKind::ModuleDeclaration,
    SyntaxKind::CaseBlock,
    SyntaxKind::Block,
    SyntaxKind::CatchClause,
    SyntaxKind::ForStatement,
    SyntaxKind::ForInStatement,
    SyntaxKind::ForOfStatement,
    SyntaxKind::FunctionDeclaration,
    SyntaxKind::FunctionExpression,
    SyntaxKind::ArrowFunction,
    SyntaxKind::MethodDeclaration,
    SyntaxKind::MethodSignature,
    SyntaxKind::CallSignature,
    SyntaxKind::ConstructSignature,
    SyntaxKind::FunctionType,
    SyntaxKind::ConstructorType,
    SyntaxKind::MappedType,
    SyntaxKind::Constructor,
    SyntaxKind::GetAccessor,
    SyntaxKind::SetAccessor,
    SyntaxKind::InterfaceDeclaration,
    SyntaxKind::ClassDeclaration,
    SyntaxKind::ClassExpression,
    SyntaxKind::TypeAliasDeclaration,
    SyntaxKind::EnumDeclaration,
    SyntaxKind::ConditionalType,
];

/// Go nameresolver.go KindDecorator 分支（binder/nameresolver.go:255-281）：
/// 装饰器表达式内名字的上溯到达装饰器节点时整体跳到其下一个查找容器——
/// 参数装饰器跳到所属方法（类元素），类元素装饰器跳到所属方法，两者下一
/// 查找容器为所属类声明（类 locals=类型参数仍可达）；类装饰器跳到类声明
/// 本身，下一查找容器为类声明之外（类型参数不可达）。返回须整体跳过查找
/// 的容器 id 集（装饰器节点与下一查找容器之间的全部容器）；名字不在装饰器
/// 内时返回 None。
pub(crate) fn decorator_skip_container_ids(
    node: &Arc<Node>,
) -> Option<std::collections::HashSet<u64>> { ::tsox_core::fntrace::enter("decorator_skip_container_ids");
    let decorator = tsox_frontend::ast::find_ancestor_kind(node, SyntaxKind::Decorator)?;
    let mut location = Arc::clone(&decorator);
    if location
        .parent()
        .is_some_and(|p| p.kind == SyntaxKind::Parameter)
    {
        location = location.parent()?;
    }
    if let Some(parent) = location.parent() {
        if tsox_frontend::ast::is_class_element(&parent)
            || parent.kind == SyntaxKind::ClassDeclaration
        {
            location = parent;
        }
    }
    let boundary = location.parent()?;
    let mut skip = std::collections::HashSet::new();
    let mut current = Some(decorator);
    while let Some(a) = current {
        if a.id() == boundary.id() {
            break;
        }
        skip.insert(a.id());
        current = a.parent();
    }
    Some(skip)
}

pub(crate) fn lexical_scope_chain_ids(node: &Arc<Node>) -> Vec<u64> { ::tsox_core::fntrace::enter("lexical_scope_chain_ids"); 
    let mut chain = Vec::new();
    let mut ancestor = node.parent();
    while let Some(a) = ancestor {
        if ANCESTRY_CONTAINERS.contains(&a.kind) {
            chain.push(a.id());
        }
        ancestor = a.parent();
    }
    chain
}

impl Checker {
    pub fn resolve_identifier(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_identifier"); 
        self.resolve_identifier_with_meaning(node, SymbolFlags::all())
    }

    pub fn resolve_identifier_with_meaning(
        &self,
        node: &Arc<Node>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_identifier_with_meaning"); 
        self.resolve_identifier_scope_symbol(node, meaning)
            .and_then(|s| self.follow_alias(&s))
    }

    pub fn resolve_identifier_use(
        &self,
        node: &Arc<Node>,
        record_meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_identifier_use"); 
        let scope = self.resolve_identifier_scope_symbol(node, SymbolFlags::all());
        let result = scope.as_ref().and_then(|s| self.follow_alias(s));
        if let Some(sym) = &scope
            && access_kind(node) != AccessKind::Write
            && !self.self_reference_location_exempts(node, sym)
        {
            self.record_symbol_reference(sym, record_meaning);
        }
        result
    }

    fn self_reference_location_exempts(&self, node: &Arc<Node>, resolved: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("self_reference_location_exempts"); 
        let name = match &node.data {
            tsox_frontend::ast::NodeData::Identifier(data) => data.text.as_str(),
            _ => return false,
        };
        let symbol_map = self.program.symbol_map();
        let mut last_location: Option<Arc<Node>> = None;
        let mut last_self_reference: Option<Arc<Node>> = None;
        let mut current = node.parent();
        while let Some(location) = current {
            if let Some(locals) = symbol_map.locals.get(&location.id())
                && locals.get(name).is_some()
            {
                break;
            }
            let is_self_reference = match location.kind {
                SyntaxKind::Parameter => last_location
                    .as_ref()
                    .zip(location.name())
                    .is_some_and(|(last, name)| Arc::ptr_eq(last, &name)),
                SyntaxKind::FunctionDeclaration
                | SyntaxKind::ClassDeclaration
                | SyntaxKind::InterfaceDeclaration
                | SyntaxKind::EnumDeclaration
                | SyntaxKind::TypeAliasDeclaration
                | SyntaxKind::ModuleDeclaration => true,
                _ => false,
            };
            if is_self_reference && last_self_reference.is_none() {
                last_self_reference = Some(Arc::clone(&location));
            }
            last_location = Some(Arc::clone(&location));
            current = location.parent();
        }
        last_self_reference.is_some_and(|location| {
            symbol_map
                .symbol_of(&location)
                .is_some_and(|sym| sym.id() == resolved.id())
        })
    }

    pub(crate) fn record_symbol_reference(&self, symbol: &Arc<Symbol>, bits: SymbolFlags) { ::tsox_core::fntrace::enter("record_symbol_reference"); 
        self.symbol_reference_kinds
            .entry(symbol.id())
            .and_modify(|f| *f |= bits)
            .or_insert(bits);
    }

    pub(crate) fn alias_chain_hits_meaning(&self, sym: &Arc<Symbol>, meaning: SymbolFlags) -> bool { ::tsox_core::fntrace::enter("alias_chain_hits_meaning"); 
        if !sym.flags.intersects(SymbolFlags::Alias) {
            return false;
        }
        match self.follow_alias(sym) {
            Some(target) if !Arc::ptr_eq(&target, sym) => target.flags.intersects(meaning),
            _ => true,
        }
    }

    pub fn follow_alias(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("follow_alias");
        if !symbol.flags.intersects(SymbolFlags::Alias) {
            return Some(Arc::clone(symbol));
        }

        let is_pure_alias = symbol.flags == SymbolFlags::Alias
            || (symbol.flags.intersects(SymbolFlags::Alias)
                && symbol.flags.intersects(SymbolFlags::Assignment));
        if !is_pure_alias {
            return Some(Arc::clone(symbol));
        }

        let mut current = Arc::clone(symbol);
        let mut seen: Vec<*const Symbol> = vec![Arc::as_ptr(symbol)];
        let mut took_decl_hop = false;
        loop {
            let export_symbol_self_linked = current
                .export_symbol
                .as_ref()
                .is_some_and(|t| {
                    std::ptr::eq(
                        std::sync::Arc::as_ptr(t) as *const u8,
                        Arc::as_ptr(&current) as *const u8,
                    )
                });
            let linked = current.export_symbol.is_some() && !export_symbol_self_linked;
            if linked || (took_decl_hop && current.flags.intersects(SymbolFlags::Alias)) {
                // Go getSymbolFlagsEx（checker.go:17337-17357）：链按 while
                // Alias 以 getExportSymbolOfValueSymbolIfExported ∘ resolveAlias
                // 取跳——resolveAlias（checker.go:17361-17364）按 FindLast 别名
                // 类声明取目标，不读 ExportSymbol；default 导出 local+export 对
                // 的本地符号（local→export 链接非自连）与 import 合流出 Alias
                // 位后按 export_symbol 直连会把含义链劫持到导出符号。
                // took_decl_hop 延续链中段无链接的别名（EA → 值符号）
                match self.merged_default_local_alias_hop(&current, &seen) {
                    Some(target) => {
                        let target_ptr = Arc::as_ptr(&target);
                        if seen.contains(&target_ptr) {
                            return Some(Arc::clone(&current));
                        }
                        seen.push(target_ptr);
                        current = target;
                        took_decl_hop = true;
                        continue;
                    }
                    None => return Some(Arc::clone(&current)),
                }
            }
            if let Some(ref target) = current.export_symbol {
                let target_ptr = Arc::as_ptr(target);
                if seen.contains(&target_ptr) {
                    return Some(Arc::clone(&current));
                }
                let is_pure = target.flags == SymbolFlags::Alias
                    || (target.flags.intersects(SymbolFlags::Alias)
                        && target.flags.intersects(SymbolFlags::Assignment));
                if is_pure {
                    seen.push(target_ptr);
                    current = Arc::clone(target);
                    continue;
                }
                return Some(Arc::clone(target));
            } else {
                return Some(Arc::clone(&current));
            }
        }
    }

    // Go resolveAlias（checker.go:17361-17364）声明驱动跳的 &self 子集，仅
    // 服 default 导出 local+export 对与 import 合并出的别名符号：别名类声明
    // 自后向前逐个取跳（Go getDeclarationOfAliasSymbol 取 FindLast；本仓
    // 无 from specifier 的绑定元素挂在目标符号声明表尾，与同表 ImportClause
    // 并存时自跳须回退到次后别名声明继续链）。ImportClause 取模块 default
    // 导出符号（相对说明符按导入声明所在文件目录+程序解析兜底，Go
    // resolveExternalModuleName checker.go:15942 的 containingFile 语义）；
    // ExportSpecifier 无 from 按 property_name 文件作用域直名解析
    //（getTargetOfExportSpecifier checker.go:15783-15797）；ExportAssignment
    // 取表达式标识符的文件作用域名解析（resolveEntityName 的
    // locals→members→exports 直名序）
    fn merged_default_local_alias_hop(
        &self,
        symbol: &Arc<Symbol>,
        seen: &[*const Symbol],
    ) -> Option<Arc<Symbol>> {
        for decl in symbol.declarations.iter().rev() {
            if !crate::checker::checker_get_excluded_symbol_flags::is_alias_symbol_declaration(decl)
            {
                continue;
            }
            let target = match decl.kind {
                SyntaxKind::ImportClause => {
                    let Some(import_decl) = decl
                        .parent()
                        .filter(|p| p.kind == SyntaxKind::ImportDeclaration)
                    else {
                        continue;
                    };
                    let NodeData::ImportDeclaration(d) = &import_decl.data else {
                        continue;
                    };
                    let spec = d.module_specifier.text().trim_matches(['"', '\'']).to_string();
                    let module_sym = if spec.starts_with('.') {
                        let Some(dir) = self.get_source_file_of_node(&import_decl).map(|f| {
                            match f.file_name.rfind('/') {
                                Some(i) => f.file_name[..i].to_string(),
                                None => String::new(),
                            }
                        }) else {
                            continue;
                        };
                        self.resolve_module_file_symbol_in(&dir, &spec).or_else(|| {
                            self.resolve_module_file_by_program_resolution(
                                &import_decl,
                                &spec,
                            )
                        })
                    } else {
                        self.resolve_module_file_symbol(&spec)
                    };
                    let Some(module_sym) = module_sym else {
                        continue;
                    };
                    module_sym
                        .exports
                        .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_DEFAULT)
                        .cloned()
                }
                SyntaxKind::ExportSpecifier => {
                    let no_from = decl.parent().and_then(|c| c.parent()).is_some_and(
                        |export_decl| {
                            matches!(
                                &export_decl.data,
                                NodeData::ExportDeclaration(d)
                                    if d.module_specifier.is_none()
                            )
                        },
                    );
                    if !no_from {
                        continue;
                    }
                    let NodeData::ExportSpecifier(es) = &decl.data else {
                        continue;
                    };
                    let target_name = es
                        .property_name
                        .as_ref()
                        .unwrap_or(&es.name)
                        .text()
                        .trim_matches(['"', '\'', '`'])
                        .to_string();
                    let Some(scope) = decl
                        .parent()
                        .and_then(|c| c.parent())
                        .filter(|p| p.kind == SyntaxKind::SourceFile)
                    else {
                        continue;
                    };
                    self.export_specifier_file_scope_target(&scope, &target_name)
                }
                SyntaxKind::ExportAssignment => {
                    let NodeData::ExportAssignment(ea) = &decl.data else {
                        continue;
                    };
                    if ea.is_export_equals || ea.expression.kind != SyntaxKind::Identifier {
                        continue;
                    }
                    let name = ea.expression.text();
                    let Some(scope) = decl
                        .parent()
                        .filter(|p| p.kind == SyntaxKind::SourceFile)
                    else {
                        continue;
                    };
                    let symbol_map = self.program.symbol_map();
                    symbol_map
                        .locals
                        .get(&scope.id())
                        .and_then(|l| l.get(name))
                        .or_else(|| {
                            symbol_map
                                .symbol_of(&scope)
                                .and_then(|sym| sym.members.get(name))
                        })
                        .or_else(|| {
                            symbol_map
                                .symbol_of(&scope)
                                .and_then(|sym| sym.exports.get(name))
                        })
                        .cloned()
                }
                _ => continue,
            };
            if let Some(t) = target
                && !seen.contains(&(Arc::as_ptr(&t) as *const Symbol))
                && !std::ptr::eq(
                    Arc::as_ptr(&t) as *const u8,
                    Arc::as_ptr(symbol) as *const u8,
                )
            {
                return Some(t);
            }
        }
        None
    }
}

impl Checker {
    /// Go getVisibleSymbolInAugmentationScope：declare module "foo"（augmentation）
    /// 块内的裸名查找延伸到被增强模块的 exports
    pub(crate) fn augmentation_target_member(
        &self,
        container_sym: &Arc<Symbol>,
        name: &str,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("augmentation_target_member"); 
        let decl = container_sym
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::ModuleDeclaration)?;
        let tsox_frontend::ast::NodeData::ModuleDeclaration(md) = &decl.data else {
            return None;
        };
        if md.name.kind != SyntaxKind::StringLiteral {
            return None;
        }
        let spec = md.name.text().trim_matches(['"', '\'']).to_string();
        let resolved = self.resolve_module_file_symbol(&spec)?;
        let target = if Arc::ptr_eq(&resolved, container_sym) {
            return None;
        } else {
            resolved
        };
        target
            .exports
            .get(name)
            .cloned()
            .or_else(|| target.members.get(name).cloned())
    }
}

pub(crate) fn node_parent_is_require_call(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("node_parent_is_require_call"); 
    let Some(parent) = node.parent() else {
        return false;
    };
    if let tsox_frontend::ast::NodeData::CallExpression(call) = &parent.data
        && call.expression.kind == SyntaxKind::Identifier
        && call.expression.text() == "require"
        && call.arguments.len() == 1
    {} else {
        return false;
    }
    let mut ancestor = Some(Arc::clone(node));
    while let Some(a) = ancestor {
        if a.kind == SyntaxKind::SourceFile {
            return a
                .flags
                .contains(tsox_frontend::ast::NodeFlags::JavaScriptFile);
        }
        ancestor = a.parent();
    }
    false
}
