#![allow(unused_imports)]

use crate::binder::symbols::*;

impl Binder {
    // Go binder 各声明类别的 excludes（bindWorker 各 case 的 includes/excludes 配对）
    fn excludes_for_declaration(node: &Arc<Node>, includes: SymbolFlags) -> SymbolFlags {
        if node.kind == SyntaxKind::Parameter
            || node.kind == SyntaxKind::BindingElement && Self::is_part_of_parameter_declaration(node)
        {
            return SymbolFlags::ParameterExcludes;
        }
        if node.kind == SyntaxKind::MethodDeclaration
            && node.parent().is_some_and(|p| p.kind == SyntaxKind::ObjectLiteralExpression)
        {
            return SymbolFlags::VALUE;
        }
        if includes.contains(SymbolFlags::FunctionScopedVariable) {
            return SymbolFlags::FunctionScopedVariableExcludes;
        }
        if includes.contains(SymbolFlags::BlockScopedVariable) {
            return SymbolFlags::BlockScopedVariableExcludes;
        }
        if includes.contains(SymbolFlags::Function) {
            return SymbolFlags::FunctionExcludes;
        }
        if includes.contains(SymbolFlags::Class) {
            return SymbolFlags::ClassExcludes;
        }
        if includes.contains(SymbolFlags::Interface) {
            return SymbolFlags::InterfaceExcludes;
        }
        if includes.contains(SymbolFlags::ConstEnum) {
            return SymbolFlags::ConstEnumExcludes;
        }
        if includes.contains(SymbolFlags::RegularEnum) {
            return SymbolFlags::RegularEnumExcludes;
        }
        if includes.contains(SymbolFlags::ValueModule) {
            return SymbolFlags::ValueModuleExcludes;
        }
        if includes.contains(SymbolFlags::NamespaceModule) {
            return SymbolFlags::NamespaceModuleExcludes;
        }
        if includes.contains(SymbolFlags::TypeAlias) {
            return SymbolFlags::TypeAliasExcludes;
        }
        if includes.contains(SymbolFlags::TypeParameter) {
            return SymbolFlags::TypeParameterExcludes;
        }
        if includes.contains(SymbolFlags::Method) {
            return SymbolFlags::MethodExcludes;
        }
        if includes.contains(SymbolFlags::Property) {
            return SymbolFlags::PropertyExcludes;
        }
        if includes.contains(SymbolFlags::GetAccessor) {
            return SymbolFlags::GetAccessorExcludes;
        }
        if includes.contains(SymbolFlags::SetAccessor) {
            return SymbolFlags::SetAccessorExcludes;
        }
        if includes.contains(SymbolFlags::EnumMember) {
            return SymbolFlags::EnumMemberExcludes;
        }
        if includes.contains(SymbolFlags::Alias) {
            return SymbolFlags::AliasExcludes;
        }
        SymbolFlags::empty()
    }

    // Go declareClassMember：类容器内 static 成员入 exports 表、实例成员入
    // members 表，同名不同 staticness 永不相交；单表架构下以“同 staticness
    // 子集的声明标志”参与冲突判定，None 表示非类容器（用整符号标志）
    pub(crate) fn class_member_same_static_flags(
        &self,
        node: &Arc<Node>,
        existing: &Arc<Symbol>,
    ) -> Option<(SymbolFlags, SymbolFlags)> {
        let container = self.container.as_ref()?;
        if !matches!(
            container.kind,
            SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
        ) {
            return None;
        }
        let node_static = node.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Static);
        let mut same = SymbolFlags::empty();
        let mut other = SymbolFlags::empty();
        for d in existing.declarations.iter() {
            let fold = if d.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Static)
                == node_static
            {
                &mut same
            } else {
                &mut other
            };
            *fold |= Self::member_includes_flags(d.kind);
        }
        Some((same, other))
    }

    // Go IsPartOfParameterDeclaration：沿父链先遇到 Parameter 即参数解构内
    fn is_part_of_parameter_declaration(node: &Arc<Node>) -> bool {
        let mut cur = node.parent();
        while let Some(p) = cur {
            match p.kind {
                SyntaxKind::Parameter => return true,
                SyntaxKind::SourceFile
                | SyntaxKind::Block
                | SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ArrowFunction
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::Constructor
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor => return false,
                _ => cur = p.parent(),
            }
        }
        false
    }

    fn member_includes_flags(kind: SyntaxKind) -> SymbolFlags {
        match kind {
            SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::PropertyAssignment => SymbolFlags::Property,
            SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature => SymbolFlags::Method,
            SyntaxKind::GetAccessor => SymbolFlags::GetAccessor,
            SyntaxKind::SetAccessor => SymbolFlags::SetAccessor,
            _ => SymbolFlags::empty(),
        }
    }

    // Go HasDynamicName：计算名且表达式非字符串/数值字面量（well-known
    // Symbol.x 亦算动态名，但本仓模型以 __@x 键保持合并语义，故排除）
    fn is_dynamic_nonliteral_computed_member(node: &Arc<Node>) -> bool {
        if !matches!(
            node.kind,
            SyntaxKind::PropertyDeclaration
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor
                | SyntaxKind::PropertySignature
                | SyntaxKind::MethodSignature
                | SyntaxKind::PropertyAssignment
        ) {
            return false;
        }
        let Some(name) = node.name() else {
            return false;
        };
        if name.kind != SyntaxKind::ComputedPropertyName {
            return false;
        }
        if let tsox_frontend::ast::NodeData::ComputedPropertyName(cd) = &name.data {
            if crate::binder::symbols_binder_4::well_known_symbol_member_name(&cd.expression)
                .is_some()
            {
                return false;
            }
            !matches!(
                cd.expression.kind,
                SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral
            )
        } else {
            false
        }
    }

    pub(crate) fn declare_symbol(
        &mut self,
        node: &Arc<Node>,
        includes: SymbolFlags,
        _excludes: SymbolFlags,
    ) -> Arc<Symbol> {
        // Go bindPropertyOrMethodOrAccessor 的 HasDynamicName 分支：非字面量
        // 计算名成员走 bindAnonymousDeclaration（__computed 匿名符号，不入表、
        // 不合并、不冲突）
        if Self::is_dynamic_nonliteral_computed_member(node) {
            let symbol = self.new_symbol(includes, "__computed");
            let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
            unsafe {
                (*symbol_mut).declarations.push(Arc::clone(node));
                if (*symbol_mut).value_declaration.is_none()
                    && includes.intersects(SymbolFlags::VALUE)
                {
                    (*symbol_mut).value_declaration = Some(Arc::clone(node));
                }
            }
            self.symbol_map.set_symbol(node, Arc::clone(&symbol));
            return symbol;
        }

        let name = self.get_declaration_name(node);

        let var_hoist_container: Option<Arc<Node>> =
            if Self::declaration_is_var(node) && self.parent_symbol.is_none() {
                self.container
                    .as_ref()
                    .filter(|c| is_var_container_kind(c.kind))
                    .cloned()
            } else {
                None
            };

        let is_module_member_container = self
            .container
            .as_ref()
            .is_some_and(|c| c.kind == SyntaxKind::ModuleDeclaration);

        let existing: Option<Arc<Symbol>> =
            if is_module_member_container && let Some(parent_sym) = &self.parent_symbol {
                let has_export = self.module_member_is_exported(node);
                let container_id = self.container.as_ref().unwrap().id();
                let locals_hit = || {
                    self.symbol_map
                        .locals
                        .get(&container_id)
                        .and_then(|l| l.get(&name).cloned())
                };
                if includes.contains(SymbolFlags::Alias) {
                    if has_export {
                        parent_sym.exports.get(&name).cloned()
                    } else {
                        locals_hit()
                    }
                } else if has_export {
                    parent_sym.exports.get(&name).cloned().or_else(locals_hit)
                } else {
                    locals_hit()
                }
            } else if var_hoist_container.is_none()
                && let Some(block_container) = &self.block_scope_container
                && self
                    .container
                    .as_ref()
                    .is_none_or(|c| c.id() != block_container.id())
            {
                let container_id = block_container.id();
                self.symbol_map
                    .locals
                    .get(&container_id)
                    .and_then(|l| l.get(&name).cloned())
            } else if self
                .container
                .as_ref()
                .is_some_and(|c| is_function_like_locals_container(c.kind))
            {
                let container_id = self.container.as_ref().unwrap().id();
                self.symbol_map
                    .locals
                    .get(&container_id)
                    .and_then(|l| l.get(&name).cloned())
            } else if let Some(parent_sym) = &self.parent_symbol {
                // Go declareClassMember 按 staticness 分表：实例成员查 members、
                // static 成员查 exports，同名 static/实例不合并
                let node_is_static_member = node.has_syntactic_modifier(ModifierFlags::Static)
                    && matches!(
                        node.kind,
                        SyntaxKind::PropertyDeclaration
                            | SyntaxKind::MethodDeclaration
                            | SyntaxKind::GetAccessor
                            | SyntaxKind::SetAccessor
                    );
                let parent_is_class = parent_sym.declarations.iter().any(|d| {
                    matches!(
                        d.kind,
                        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                    )
                });
                if parent_is_class && node_is_static_member {
                    parent_sym.exports.get(&name).cloned()
                } else if parent_is_class {
                    parent_sym.members.get(&name).cloned()
                } else {
                    parent_sym
                        .members
                        .get(&name)
                        .cloned()
                        .or_else(|| parent_sym.exports.get(&name).cloned())
                }
            } else if let Some(hoist) = &var_hoist_container {
                match hoist.kind {
                    SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration => self
                        .symbol_map
                        .symbol_of(hoist)
                        .and_then(|sym| sym.members.get(&name).cloned()),

                    _ => {
                        let container_id = hoist.id();
                        self.symbol_map
                            .locals
                            .get(&container_id)
                            .and_then(|locals| locals.get(&name).cloned())
                    }
                }
            } else if let Some(block_container) = &self.block_scope_container {
                let container_id = block_container.id();
                self.symbol_map
                    .locals
                    .get(&container_id)
                    .and_then(|locals| locals.get(&name).cloned())
            } else {
                None
            };

        let mut conflicted = false;
        // Go declareModuleMember/declareSourceFileMember：模块容器内本地与导出
        // 声明分属 locals/exports 两表，同名不同导出性不构成 binder 冲突，
        // 后续类型一致性/导出性检查（TS2403/TS2395）由 checker 报
        let mixed_exportness_module_merge = self
            .container
            .as_ref()
            .is_some_and(|c| {
                matches!(
                    c.kind,
                    SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration
                )
            })
            && !name.is_empty()
            && existing.as_ref().is_some_and(|e| {
                let node_exported = self.module_member_is_exported(node);
                e.declarations
                    .iter()
                    .map(|d| self.module_member_is_exported(d))
                    .any(|d_exported| d_exported != node_exported)
            });

        if let Some(existing) = existing {
            // Go declareSymbol 冲突路径：excludes 互斥表（bindWorker 配对），
            // 冲突时报所有既有声明 + 当前声明，且不合并符号、不替换表内既有符号
            let excludes = Self::excludes_for_declaration(node, includes);
            // Go declareClassMember 按 staticness 分表：冲突判定只看同 staticness
            // 子集的声明；无同 staticness 声明（跨表）则永无冲突
            let same_static_flags = self.class_member_same_static_flags(node, &existing);
            let staticness_split = same_static_flags
                .map(|(same, _)| same == SymbolFlags::empty())
                .unwrap_or(false);
            // Go 以 symbol.Flags 判冲突（含访问器满标记位）：剔除跨 staticness
            // 子集贡献后使用整符号标志
            let comparison_flags = match same_static_flags {
                Some((_, other)) => existing.flags & !other,
                None => existing.flags,
            };
            let assignment_merge_exception = (includes.contains(SymbolFlags::FunctionScopedVariable)
                && existing.flags.contains(SymbolFlags::Assignment))
                || (includes.contains(SymbolFlags::Assignment)
                    && existing
                        .flags
                        .contains(SymbolFlags::FunctionScopedVariable));
            if !staticness_split
                && !mixed_exportness_module_merge
                && !excludes.is_empty()
                && !name.is_empty()
                && comparison_flags.intersects(excludes)
                && !assignment_merge_exception
            {
                if comparison_flags.intersects(SymbolFlags::ENUM)
                    || includes.intersects(SymbolFlags::ENUM)
                {
                    self.report_declaration_conflict_all(
                        node,
                        &existing,
                        None,
                        &tsox_core::diagnostics::messages_generated::ENUM_DECLARATIONS_CAN_ONLY_MERGE_WITH_NAMESPACE_OR_OTHER_ENUM_DECLARATIONS,
                    );
                } else if comparison_flags.contains(SymbolFlags::BlockScopedVariable) {
                    self.report_declaration_conflict_all(
                        node,
                        &existing,
                        Some(&name),
                        &CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0,
                    );
                } else {
                    self.report_duplicate_identifier_all(node, &existing, &name);
                }
                if existing.flags.intersects(SymbolFlags::ACCESSOR)
                    && (existing.flags & SymbolFlags::ACCESSOR) != (includes & SymbolFlags::ACCESSOR)
                {
                    let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                    unsafe {
                        (*existing_mut).flags |= SymbolFlags::ACCESSOR;
                    }
                }
                let symbol = self.new_symbol(includes, name.clone());
                {
                    let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                    unsafe {
                        (*symbol_mut).declarations.push(Arc::clone(node));
                        if (*symbol_mut).value_declaration.is_none()
                            && includes.intersects(SymbolFlags::VALUE)
                        {
                            (*symbol_mut).value_declaration = Some(Arc::clone(node));
                        }
                    }
                }
                self.symbol_map.set_symbol(node, Arc::clone(&symbol));
                return symbol;
            }
            if let Some(merged) =
                self.merge_into_existing_symbol(node, &existing, includes)
                    .or_else(|| {
                        if mixed_exportness_module_merge {
                            self.append_declaration_to_existing_symbol(node, &existing, includes)
                        } else {
                            None
                        }
                    })
            {
                return merged;
            }

            conflicted = !mixed_exportness_module_merge
                && self.report_symbol_conflict(node, &existing, &name, includes);
        }

        let symbol = self.new_symbol(includes, name.clone());

        {
            let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
            unsafe {
                (*symbol_mut).declarations.push(Arc::clone(node));

                if (*symbol_mut).value_declaration.is_none()
                    && includes.intersects(SymbolFlags::VALUE)
                {
                    (*symbol_mut).value_declaration = Some(Arc::clone(node));
                }
            }
        }

        if !conflicted {
            self.insert_symbol_into_container(node, &symbol, &name, &var_hoist_container);
        }

        if let Some(container) = &self.container {
            let is_module_container = container.kind == SyntaxKind::SourceFile
                || container.kind == SyntaxKind::ModuleDeclaration;
            if is_module_container
                && self
                    .get_combined_modifier_flags(node)
                    .contains(ModifierFlags::Export)
            {
                let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                unsafe {
                    (*symbol_mut).export_symbol = Some(Arc::clone(&symbol));
                }
            }
        }

        self.symbol_map.set_symbol(node, Arc::clone(&symbol));

        symbol
    }
}
