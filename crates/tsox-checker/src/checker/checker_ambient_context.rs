use crate::checker::checker::*;
use tsox_core::diagnostics::messages_generated::{
    AN_IMPLEMENTATION_CANNOT_BE_DECLARED_IN_AMBIENT_CONTEXTS,
    STATEMENTS_ARE_NOT_ALLOWED_IN_AMBIENT_CONTEXTS,
    TOP_LEVEL_DECLARATIONS_IN_D_TS_FILES_MUST_START_WITH_EITHER_A_DECLARE_OR_EXPORT_MODIFIER,
};

impl Checker {
    pub(crate) fn node_in_ambient_context(&self, node: &Arc<Node>) -> bool {
        let mut current = Some(Arc::clone(node));
        while let Some(n) = current {
            if n.kind == SyntaxKind::SourceFile {
                return self
                    .get_source_file_of_node(&n)
                    .is_some_and(|f| f.is_declaration_file);
            }

            if n.has_syntactic_modifier(ModifierFlags::Ambient) {
                return true;
            }
            current = n.parent();
        }
        false
    }

    pub(crate) fn check_grammar_statement_in_ambient_context(&mut self, node: &Arc<Node>) -> bool {
        if !self.node_in_ambient_context(node) {
            return false;
        }
        let Some(parent) = node.parent() else {
            return false;
        };
        if tsox_frontend::ast::utilities::is_function_like(&parent)
            || tsox_frontend::ast::utilities::is_accessor(&parent)
        {
            let key = node.id();
            if !self.ambient_statement_reported.contains(&key) {
                let reported = self.grammar_error_on_first_token(
                    node,
                    &AN_IMPLEMENTATION_CANNOT_BE_DECLARED_IN_AMBIENT_CONTEXTS,
                );
                if reported {
                    self.ambient_statement_reported.insert(key);
                }
                return reported;
            }
            return false;
        }
        if matches!(
            parent.kind,
            SyntaxKind::Block | SyntaxKind::ModuleBlock | SyntaxKind::SourceFile
        ) {
            let key = parent.id();
            if !self.ambient_statement_reported.contains(&key) {
                let reported = self.grammar_error_on_first_token(
                    node,
                    &STATEMENTS_ARE_NOT_ALLOWED_IN_AMBIENT_CONTEXTS,
                );
                if reported {
                    self.ambient_statement_reported.insert(key);
                }
                return reported;
            }
        }
        false
    }

    pub(crate) fn statement_kind_takes_ambient_check(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::Block
                | SyntaxKind::IfStatement
                | SyntaxKind::DoStatement
                | SyntaxKind::WhileStatement
                | SyntaxKind::ForStatement
                | SyntaxKind::ForInStatement
                | SyntaxKind::ForOfStatement
                | SyntaxKind::BreakStatement
                | SyntaxKind::ContinueStatement
                | SyntaxKind::ReturnStatement
                | SyntaxKind::WithStatement
                | SyntaxKind::SwitchStatement
                | SyntaxKind::LabeledStatement
                | SyntaxKind::ThrowStatement
                | SyntaxKind::TryStatement
                | SyntaxKind::EmptyStatement
                | SyntaxKind::DebuggerStatement
                | SyntaxKind::ExpressionStatement
        )
    }

    fn check_grammar_top_level_element_for_required_declare_modifier(
        &mut self,
        node: &Arc<Node>,
    ) -> bool {
        let exempt_kind = matches!(
            node.kind,
            SyntaxKind::InterfaceDeclaration
                | SyntaxKind::TypeAliasDeclaration
                | SyntaxKind::ImportDeclaration
                | SyntaxKind::ImportEqualsDeclaration
                | SyntaxKind::ExportDeclaration
                | SyntaxKind::ExportAssignment
                | SyntaxKind::NamespaceExportDeclaration
        );
        let exempt_modifier = node.has_syntactic_modifier(
            ModifierFlags::Ambient
                .union(ModifierFlags::Export)
                .union(ModifierFlags::Default),
        );
        if exempt_kind || exempt_modifier {
            return false;
        }
        self.grammar_error_on_first_token(
            node,
            &TOP_LEVEL_DECLARATIONS_IN_D_TS_FILES_MUST_START_WITH_EITHER_A_DECLARE_OR_EXPORT_MODIFIER,
        )
    }

    pub fn check_grammar_source_file(&mut self, file: &Arc<SourceFile>) -> bool {
        if !file.is_declaration_file {
            return false;
        }
        if let tsox_frontend::ast::NodeData::SourceFile(data) = &file.node.data {
            for decl in data.statements.iter() {
                if tsox_frontend::ast::utilities::is_declaration_node(decl)
                    || decl.kind == SyntaxKind::VariableStatement
                {
                    if self.check_grammar_top_level_element_for_required_declare_modifier(decl) {
                        return true;
                    }
                }
            }
        }
        false
    }
}
