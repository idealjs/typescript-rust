#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn is_ambient_module(node: &Arc<Node>) -> bool {
        if node.kind != SyntaxKind::ModuleDeclaration {
            return false;
        }
        if let NodeData::ModuleDeclaration(d) = &node.data {
            d.keyword == SyntaxKind::ModuleKeyword || d.keyword == SyntaxKind::NamespaceKeyword
        } else {
            false
        }
    }

    pub(crate) fn is_module_augmentation_external(node: &Arc<Node>) -> bool {
        let parent = match &node.parent {
            Some(p) => p,
            None => return false,
        };
        match parent.kind {
            SyntaxKind::SourceFile => Self::is_external_or_common_js_module(parent),
            SyntaxKind::ModuleBlock => {
                let grandparent = match &parent.parent {
                    Some(gp) => gp,
                    None => return false,
                };
                Self::is_ambient_module(grandparent)
                    && matches!(&grandparent.parent, Some(ggp) if ggp.kind == SyntaxKind::SourceFile)
                    && !Self::is_external_or_common_js_module(grandparent.parent.as_ref().unwrap())
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
            SyntaxKind::ImportClause => node.parent.clone(),
            SyntaxKind::NamespaceImport => node.parent.clone().and_then(|p| p.parent.clone()),
            SyntaxKind::ImportSpecifier => node
                .parent
                .clone()
                .and_then(|p| p.parent.clone())
                .and_then(|gp| gp.parent.clone()),
            _ => None,
        }
    }
}
