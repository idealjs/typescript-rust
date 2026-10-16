#![allow(unused_imports)]

use crate::binder::symbols::*;

impl Binder {
    // Go declareSymbol 冲突路径：报所有既有声明名 + 当前声明名的 Duplicate identifier
    pub(crate) fn report_duplicate_identifier_all(
        &mut self,
        node: &Arc<Node>,
        existing: &Arc<Symbol>,
        name: &str,
    ) { ::tsox_core::fntrace::enter("report_duplicate_identifier_all"); 
        self.report_declaration_conflict_all(node, existing, Some(name), &DUPLICATE_IDENTIFIER_0);
    }

    // Go declareSymbol 冲突路径：报所有既有声明名 + 当前声明名，
    // 消息按既有符号位选择（enum > block-scoped > duplicate identifier）
    pub(crate) fn report_declaration_conflict_all(
        &mut self,
        node: &Arc<Node>,
        existing: &Arc<Symbol>,
        name: Option<&str>,
        message: &'static tsox_core::diagnostics::Message,
    ) { ::tsox_core::fntrace::enter("report_declaration_conflict_all"); 
        let push = |b: &mut Self, n: &Arc<Node>| {
            let name_node = tsox_frontend::ast::utilities::get_name_of_declaration(n)
                .unwrap_or_else(|| Arc::clone(n));
            if b
                .symbol_map
                .binder_diagnostics
                .iter()
                .any(|d| d.loc == name_node.loc && d.code == message.code)
            {
                return;
            }
            let args = name.map(|n| vec![Self::declaration_name_display(b, &name_node, n)]);
            b.symbol_map.binder_diagnostics.push(Diagnostic::new(
                b.current_source_file.clone(),
                name_node.loc,
                *message,
                args.unwrap_or_default(),
            ));
        };
        for d in &existing.declarations {
            push(self, d);
        }
        push(self, node);
    }

    // Go DeclarationNameToString：名字按源文本展示（字符串字面量名带引号）
    fn declaration_name_display(b: &Binder, name_node: &Arc<Node>, fallback: &str) -> String { ::tsox_core::fntrace::enter("declaration_name_display"); 
        if name_node.loc.end() == name_node.loc.pos() {
            return "(Missing)".to_string();
        }
        if let Some(sf) = b.current_source_file.as_ref()
            && name_node.loc.end() > name_node.loc.pos()
            && name_node.loc.end() <= sf.text.len()
        {
            return sf.text[name_node.loc.pos()..name_node.loc.end()].to_string();
        }
        crate::checker::property_name_for_display(fallback)
    }

    // Go declareSymbol 调用方的 parent 实参（declareSymbolEx 尾段 binder.go:299-301
    // 统一挂到符号，含冲突新建符号）：容器符号表语境传容器符号——
    // declareModuleMember exports 臂（binder.go:395/416/424，导出性 =
    // export 修饰 | ExportContext，外部模块文件符号全部成员适用）、
    // declareClassMember（binder.go:434-436）、enum/interface/type-literal
    // 成员臂；locals 容器（块/函数局部，binder.go:397/423/428/444/454+）
    // 传 nil 不挂
    pub(crate) fn conflict_symbol_parent(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        let container = self.container.as_ref()?;
        match container.kind {
            SyntaxKind::SourceFile => self
                .current_source_file
                .as_ref()
                .is_some_and(|f| f.external_module_indicator.is_some())
                .then(|| self.parent_symbol.clone())
                .flatten(),
            SyntaxKind::ModuleDeclaration => {
                // Go declareModuleMember（binder.go:394-410）：alias 声明按
                // alias 臂分表定 parent（exports 位挂容器符号，locals 位传 nil），
                // 非 alias 维持 exports 臂导出性口径；容器符号恒取
                // container.Symbol()（binder.go:390/411/416/424），parent_symbol
                // 在 ModuleBlock 等无符号容器语境被清空不作为来源
                let parent_attached = if matches!(
                    node.kind,
                    SyntaxKind::ExportSpecifier
                        | SyntaxKind::ImportEqualsDeclaration
                        | SyntaxKind::NamespaceImport
                        | SyntaxKind::ImportSpecifier
                ) {
                    self.module_member_alias_declaration_exports(node)
                } else {
                    self.module_member_is_exported(node)
                };
                parent_attached
                    .then(|| self.container_symbol_or_parent())
                    .flatten()
            }
            SyntaxKind::ClassExpression
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::TypeLiteral
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::JsxAttributes => self.parent_symbol.clone(),
            _ => None,
        }
    }

    pub(crate) fn report_symbol_conflict(
        &mut self,
        node: &Arc<Node>,
        existing: &Arc<Symbol>,
        name: &str,
        includes: SymbolFlags,
    ) -> bool { ::tsox_core::fntrace::enter("report_symbol_conflict"); 
        let mut conflicted = false;
        let both_block_scoped_var = existing.flags.contains(SymbolFlags::BlockScopedVariable)
            && includes.contains(SymbolFlags::BlockScopedVariable);
        let report_all = |b: &mut Self, message: &'static tsox_core::diagnostics::Message| {
                let push = |b: &mut Self, loc: tsox_core::core::text::TextRange, display: String| {
                    if b.symbol_map
                        .binder_diagnostics
                        .iter()
                        .any(|d| d.loc == loc && d.code == message.code)
                    {
                        return;
                    }
                    b.symbol_map.binder_diagnostics.push(Diagnostic::new(
                        b.current_source_file.clone(),
                        loc,
                        *message,
                        vec![display],
                    ));
                };
                for d in &existing.declarations {
                    let name_node = tsox_frontend::ast::utilities::get_name_of_declaration(d)
                        .unwrap_or_else(|| Arc::clone(d));
                    push(b, name_node.loc, Self::declaration_name_display(b, &name_node, name));
                }
                let name_node = tsox_frontend::ast::utilities::get_name_of_declaration(node)
                    .unwrap_or_else(|| Arc::clone(node));
                push(b, name_node.loc, Self::declaration_name_display(b, &name_node, name));
            };
            // Go declareSymbol 冲突路径：existing 带 BlockScoped → 2451；
            // 变量对变量的其余冲突（var-then-let）→ 2300
            let var_vs_var = existing
                .flags
                .intersects(SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable)
                && includes.intersects(
                    SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable,
                );
            let one_side_block_scoped = existing.flags.contains(SymbolFlags::BlockScopedVariable)
                || includes.contains(SymbolFlags::BlockScopedVariable);
            if var_vs_var
                && one_side_block_scoped
                && existing.flags.contains(SymbolFlags::BlockScopedVariable)
            {
                report_all(self, &CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0);
                conflicted = true;
            } else if var_vs_var && one_side_block_scoped {
                report_all(self, &DUPLICATE_IDENTIFIER_0);
                conflicted = true;
            } else if both_block_scoped_var {
                if Self::is_let_or_const_declaration(node) {
                    report_all(self, &CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0);

                    conflicted = true;
                }
            } else {
                let member_flags = SymbolFlags::Property
                    .union(SymbolFlags::Method)
                    .union(SymbolFlags::GetAccessor)
                    .union(SymbolFlags::SetAccessor)
                    .union(SymbolFlags::EnumMember)
                    .union(SymbolFlags::FunctionScopedVariable)
                    .union(SymbolFlags::TypeParameter)
                    .union(SymbolFlags::Constructor)
                    .union(SymbolFlags::Signature);

                let involves_namespace_export = node.kind == SyntaxKind::NamespaceExportDeclaration
                    || existing
                        .declarations
                        .iter()
                        .any(|d| d.kind == SyntaxKind::NamespaceExportDeclaration);
                if involves_namespace_export
                    || existing.flags.intersects(member_flags)
                    || includes.intersects(member_flags)
                {
                } else if existing.flags.intersects(SymbolFlags::ENUM)
                    != includes.intersects(SymbolFlags::ENUM)
                    && (existing
                        .flags
                        .intersects(SymbolFlags::ENUM | SymbolFlags::Class)
                        || includes.intersects(SymbolFlags::ENUM | SymbolFlags::Class))
                {
                    report_all(
                        self,
                        &tsox_core::diagnostics::messages_generated::
                            ENUM_DECLARATIONS_CAN_ONLY_MERGE_WITH_NAMESPACE_OR_OTHER_ENUM_DECLARATIONS,
                    );
                    conflicted = true;
                } else {
                    report_all(self, &DUPLICATE_IDENTIFIER_0);
                    conflicted = true;
                }
            }
        conflicted
    }
}
