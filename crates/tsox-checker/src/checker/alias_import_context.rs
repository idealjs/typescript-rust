#![allow(unused_imports)]

use crate::checker::checker_resolve::*;
use tsox_frontend::ast::{Node, NodeData, Symbol, SyntaxKind};
use std::sync::Arc;

impl Checker {
    /// import 声明上下文：(目标模块符号, 说明符文本)
    pub(crate) fn import_declaration_context(
        &mut self,
        decl: &Arc<Node>,
    ) -> Option<(Arc<Symbol>, String)> {
        let import_decl = self
            .ancestor_of_kind(decl, SyntaxKind::ImportDeclaration)?;
        let NodeData::ImportDeclaration(d) = &import_decl.data else {
            return None;
        };
        let spec = d
            .module_specifier
            .text()
            .trim_matches(['"', '\'', '`'])
            .to_string();
        let file_module = self.module_symbol_of_containing_file(&import_decl)?;
        let module = self.resolve_module_spec_from(&file_module, &spec)?;
        Some((module, spec))
    }

    pub(crate) fn module_symbol_of_containing_file(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        let file = self.get_source_file_of_node(node)?;
        let program = self.program.symbol_map();
        program.symbol_of(&file.node).map(Arc::clone)
    }

    pub(crate) fn ancestor_of_kind<'a>(
        &self,
        node: &'a Arc<Node>,
        kind: SyntaxKind,
    ) -> Option<Arc<Node>> {
        let mut cur = node.parent();
        while let Some(n) = cur {
            if n.kind == kind {
                return Some(n);
            }
            cur = n.parent();
        }
        None
    }
}

pub(crate) fn is_alias_declaration(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ImportClause
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::NamespaceImport
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::NamespaceExport
            | SyntaxKind::ExportAssignment
    )
}
