#![allow(unused_imports)]

use super::*;

impl Binder {
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
            let report_all = |b: &mut Self, message: &'static crate::diagnostics::Message| {
                let push = |b: &mut Self, loc: crate::core::text::TextRange| {
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
                        vec![name.to_string()],
                    ));
                };
                for d in &existing.declarations {
                    let name_node = crate::ast::utilities::get_name_of_declaration(d)
                        .unwrap_or_else(|| Arc::clone(d));
                    push(b, name_node.loc);
                }
                let name_node = crate::ast::utilities::get_name_of_declaration(node)
                    .unwrap_or_else(|| Arc::clone(node));
                push(b, name_node.loc);
            };
            if both_block_scoped_var {
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
                        &crate::diagnostics::messages_generated::
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
