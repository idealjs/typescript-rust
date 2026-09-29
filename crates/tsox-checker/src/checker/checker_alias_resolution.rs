use std::sync::Arc;

use tsox_core::diagnostics::messages_generated::CIRCULAR_DEFINITION_OF_IMPORT_ALIAS_0;
use tsox_frontend::ast::{Node, NodeData, Symbol, SymbolFlags, SyntaxKind};

use crate::checker::checker::Checker;

const ALIAS_DECLARATION_KINDS: &[SyntaxKind] = &[
    SyntaxKind::ImportEqualsDeclaration,
    SyntaxKind::ImportClause,
    SyntaxKind::NamespaceImport,
    SyntaxKind::NamespaceExport,
    SyntaxKind::ImportSpecifier,
    SyntaxKind::BindingElement,
    SyntaxKind::ExportSpecifier,
    SyntaxKind::ExportAssignment,
    SyntaxKind::NamespaceExportDeclaration,
    SyntaxKind::ShorthandPropertyAssignment,
    SyntaxKind::PropertyAssignment,
    SyntaxKind::BinaryExpression,
];

impl Checker {
    pub(crate) fn resolve_alias_base(&mut self, symbol: Arc<Symbol>) -> Arc<Symbol> {
        if !symbol.flags.intersects(SymbolFlags::Alias) {
            return symbol;
        }
        if self.alias_circular_reported.contains(&symbol.id()) {
            return Arc::clone(&symbol);
        }
        let sym_ptr = Arc::as_ptr(&symbol);
        if let Some(pos) = self
            .alias_resolution_stack
            .iter()
            .position(|s| Arc::as_ptr(s) == sym_ptr)
        {
            for frame in &self.alias_resolution_stack[pos..] {
                self.alias_circular_frames.insert(Arc::as_ptr(frame));
            }
            return Arc::clone(&symbol);
        }
        self.alias_resolution_stack.push(Arc::clone(&symbol));
        let target = self.resolve_alias_target(Arc::clone(&symbol));
        let mut resolved = target.clone();
        if !self.alias_circular_frames.contains(&sym_ptr)
            && let Some(t) = target.as_ref()
            && is_pure_alias(t)
            && !Arc::ptr_eq(t, &symbol)
        {
            resolved = Some(self.resolve_alias_base(Arc::clone(t)));
        }
        self.alias_resolution_stack.pop();
        if self.alias_circular_frames.remove(&sym_ptr) {
            self.alias_circular_reported.insert(symbol.id());
            self.report_circular_import_alias(&symbol);
            return Arc::clone(&symbol);
        }
        resolved.unwrap_or_else(|| Arc::clone(&symbol))
    }

    fn report_circular_import_alias(&mut self, symbol: &Arc<Symbol>) {
        let Some(decl) = symbol
            .declarations
            .iter()
            .rev()
            .find(|d| ALIAS_DECLARATION_KINDS.contains(&d.kind))
        else {
            return;
        };
        let name = if decl.kind == SyntaxKind::ExportAssignment {
            match &decl.data {
                NodeData::ExportAssignment(ea) => qualified_entity_text(&ea.expression),
                _ => symbol.name.clone(),
            }
        } else {
            symbol.name.clone()
        };
        let file = self
            .get_source_file_of_node(decl)
            .or_else(|| self.current_file.clone());
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            file,
            decl.loc,
            CIRCULAR_DEFINITION_OF_IMPORT_ALIAS_0,
            vec![name],
        ));
    }
}

fn is_pure_alias(symbol: &Arc<Symbol>) -> bool {
    symbol.flags == SymbolFlags::Alias
        || (symbol.flags.intersects(SymbolFlags::Alias)
            && symbol.flags.intersects(SymbolFlags::Assignment))
}

fn qualified_entity_text(node: &Arc<Node>) -> String {
    match &node.data {
        NodeData::Identifier(data) => data.text.clone(),
        NodeData::QualifiedName(data) => {
            format!("{}.{}", qualified_entity_text(&data.left), data.right.text())
        }
        NodeData::PropertyAccessExpression(data) => {
            format!(
                "{}.{}",
                qualified_entity_text(&data.expression),
                data.name.text()
            )
        }
        _ => String::new(),
    }
}
