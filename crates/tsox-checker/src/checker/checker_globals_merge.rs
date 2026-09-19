use crate::checker::checker::*;
use tsox_frontend::ast::Symbol;
use tsox_frontend::ast::SymbolFlags;

impl Checker {
    pub(crate) fn merge_global_entry(&mut self, name: &str, sym: &Arc<Symbol>) {
        let existing = self.globals.get(name).cloned();
        match existing {
            Some(existing) => {
                let conflicted = self.report_global_merge_conflict(&existing, sym);
                if !conflicted {
                    self.merge_global_symbols(&existing, sym);
                }
            }
            None => {
                self.globals.insert(name.to_string(), Arc::clone(sym));
            }
        }
    }

    // Go mergeSymbol 冲突面：target.Flags 与 source 的 excludes 相交即冲突；
    // 冲突时不合并符号（target 声明集不增长），对双侧声明各报一条错误并互挂 related
    pub(crate) fn report_global_merge_conflict(
        &mut self,
        target: &Arc<Symbol>,
        source: &Arc<Symbol>,
    ) -> bool {
        if (source.flags | target.flags).intersects(SymbolFlags::Assignment) {
            return false;
        }
        if target.flags & get_excluded_symbol_flags(source.flags) == SymbolFlags::empty() {
            return false;
        }
        let message = if source.flags.intersects(SymbolFlags::ENUM)
            || target.flags.intersects(SymbolFlags::ENUM)
        {
            tsox_core::diagnostics::messages_generated::
                ENUM_DECLARATIONS_CAN_ONLY_MERGE_WITH_NAMESPACE_OR_OTHER_ENUM_DECLARATIONS
        } else if source.flags.contains(SymbolFlags::BlockScopedVariable)
            || target.flags.contains(SymbolFlags::BlockScopedVariable)
        {
            tsox_core::diagnostics::messages_generated::CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0
        } else {
            tsox_core::diagnostics::messages_generated::DUPLICATE_IDENTIFIER_0
        };
        let name = source.name.clone();
        let target_nodes = self.declaration_name_nodes(target);
        let source_nodes = self.declaration_name_nodes(source);
        for (loc, file) in &target_nodes {
            self.push_dup_error_with_related(file, *loc, message, &name, &source_nodes);
        }
        for (loc, file) in &source_nodes {
            self.push_dup_error_with_related(file, *loc, message, &name, &target_nodes);
        }
        true
    }

    fn declaration_name_nodes(
        &mut self,
        sym: &Arc<Symbol>,
    ) -> Vec<(tsox_core::core::text::TextRange, Option<Arc<SourceFile>>)> {
        sym.declarations
            .iter()
            .filter_map(|d| {
                let name_node = tsox_frontend::ast::utilities::get_name_of_declaration(d)?;
                let file = self.source_file_of_root(d);
                Some((name_node.loc, file))
            })
            .collect()
    }

    fn push_dup_error_with_related(
        &mut self,
        file: &Option<Arc<SourceFile>>,
        loc: tsox_core::core::text::TextRange,
        message: tsox_core::diagnostics::Message,
        name: &str,
        related: &[(tsox_core::core::text::TextRange, Option<Arc<SourceFile>>)],
    ) {
        let mut diag = tsox_frontend::ast::Diagnostic::new(
            file.clone(),
            loc,
            message,
            vec![name.to_string()],
        );
        for (i, (rloc, rfile)) in related.iter().enumerate() {
            let (rel_msg, args) = if i == 0 {
                (
                    tsox_core::diagnostics::messages_generated::X_0_WAS_ALSO_DECLARED_HERE,
                    vec![name.to_string()],
                )
            } else {
                (tsox_core::diagnostics::messages_generated::X_AND_HERE, Vec::new())
            };
            diag.related_information
                .push(tsox_frontend::ast::Diagnostic::new(
                    rfile.clone(),
                    *rloc,
                    rel_msg,
                    args,
                ));
        }
        self.diagnostics.add_or_append_related(diag);
    }

    fn source_file_of_root(&self, decl: &Arc<Node>) -> Option<Arc<SourceFile>> {
        let mut root = Arc::clone(decl);
        while let Some(p) = root.parent() {
            root = p;
        }
        self.files.iter().find(|f| Arc::ptr_eq(&f.node, &root)).cloned()
    }
}
