#![allow(unused_imports)]

use crate::binder::symbols::*;

impl Binder {
    // Go declareSymbol 冲突路径：报所有既有声明名 + 当前声明名的 Duplicate identifier
    pub(crate) fn probe_tag(&self, tag: &str) {
        if std::env::var_os("TSOX_DEBUG_DUP").is_some() {
            eprintln!("[dup] {} file={}", tag, self.current_source_file.as_ref().map(|f| f.file_name.clone()).unwrap_or_default());
        }
    }

    pub(crate) fn report_duplicate_identifier_all(
        &mut self,
        node: &Arc<Node>,
        existing: &Arc<Symbol>,
        name: &str,
    ) {
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
    ) {
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
    fn declaration_name_display(b: &Binder, name_node: &Arc<Node>, fallback: &str) -> String {
        if let Some(sf) = b.current_source_file.as_ref()
            && name_node.loc.end() > name_node.loc.pos()
            && name_node.loc.end() <= sf.text.len()
        {
            return sf.text[name_node.loc.pos()..name_node.loc.end()].to_string();
        }
        crate::checker::property_name_for_display(fallback)
    }

    pub(crate) fn report_symbol_conflict(
        &mut self,
        node: &Arc<Node>,
        existing: &Arc<Symbol>,
        name: &str,
        includes: SymbolFlags,
    ) -> bool {
        let mut conflicted = false;
        let both_block_scoped_var = existing.flags.contains(SymbolFlags::BlockScopedVariable)
            && includes.contains(SymbolFlags::BlockScopedVariable);
        if !name.is_empty() {
            let report_all = |b: &mut Self, message: &'static tsox_core::diagnostics::Message| {
                b.probe_tag(&format!("report_all {}", message.code));
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
        }
        conflicted
    }
}
