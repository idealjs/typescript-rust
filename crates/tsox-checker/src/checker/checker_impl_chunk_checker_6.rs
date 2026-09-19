#![allow(unused_imports)]

use crate::checker::checker_impl_chunk::*;

impl Checker {
    // Go ast.IsAmbientModule：名字为字符串字面量或 global 的模块声明
    pub(crate) fn is_ambient_module(node: &Arc<Node>) -> bool {
        tsox_frontend::ast::is_ambient_module(node)
    }

    pub(crate) fn is_module_augmentation_external(node: &Arc<Node>) -> bool {
        let parent = match node.parent() {
            Some(p) => p,
            None => return false,
        };
        match parent.kind {
            SyntaxKind::SourceFile => Self::is_external_or_common_js_module(&parent),
            SyntaxKind::ModuleBlock => {
                let grandparent = match parent.parent() {
                    Some(gp) => gp,
                    None => return false,
                };
                Self::is_ambient_module(&grandparent)
                    && matches!(&grandparent.parent(), Some(ggp) if ggp.kind == SyntaxKind::SourceFile)
                    && !Self::is_external_or_common_js_module(grandparent.parent().as_ref().unwrap())
            }
            _ => false,
        }
    }

    pub fn is_late_visibility_painted_statement(node: &Arc<Node>) -> bool {
        matches!(
            node.kind,
            SyntaxKind::ImportDeclaration
                | SyntaxKind::ImportEqualsDeclaration
                | SyntaxKind::VariableStatement
                | SyntaxKind::ClassDeclaration
                | SyntaxKind::FunctionDeclaration
                | SyntaxKind::ModuleDeclaration
                | SyntaxKind::TypeAliasDeclaration
                | SyntaxKind::InterfaceDeclaration
                | SyntaxKind::EnumDeclaration
        )
    }

    pub fn get_any_import_syntax(node: &Arc<Node>) -> Option<Arc<Node>> {
        match node.kind {
            SyntaxKind::ImportEqualsDeclaration => Some(Arc::clone(node)),
            SyntaxKind::ImportClause => node.parent(),
            SyntaxKind::NamespaceImport => node.parent().and_then(|p| p.parent()),
            SyntaxKind::ImportSpecifier => node
                .parent()
                .clone()
                .and_then(|p| p.parent())
                .and_then(|gp| gp.parent()),
            _ => None,
        }
    }
}
