#![allow(unused_imports)]

use crate::checker::checker_statements::*;

impl Checker {
    // Go checkTryStatement / checkCatchClause：try 块、catch 参数、catch 块、
    // finally 块逐段检查；catch 参数类型注解限定 any/unknown（TS1196）、
    // 初始化器非法（TS1197）、catch 参与块内块级变量同名（TS2483）
    pub(crate) fn check_try_statement(&mut self, node: &Arc<Node>, ambient_reported: bool) {
        let tsox_frontend::ast::NodeData::TryStatement(data) = &node.data else {
            return;
        };
        self.check_statement(&data.try_block);
        if let Some(catch_clause) = &data.catch_clause {
            self.check_catch_clause(catch_clause);
        }
        if let Some(finally_block) = &data.finally_block {
            self.check_statement(finally_block);
        }
        let _ = ambient_reported;
    }

    fn check_catch_clause(&mut self, node: &Arc<Node>) {
        let tsox_frontend::ast::NodeData::CatchClause(data) = &node.data else {
            return;
        };
        if let Some(declaration) = &data.variable_declaration {
            self.check_variable_declaration(declaration);
            let type_node = match &declaration.data {
                tsox_frontend::ast::NodeData::VariableDeclaration(d) => d.type_node.as_ref(),
                _ => None,
            };
            if let Some(type_node) = type_node {
                let t = self.get_type_from_type_node(type_node);
                if !t.flags.intersects(TYPE_FLAGS_ANY_OR_UNKNOWN) {
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        first_token_loc(type_node),
                        tsox_core::diagnostics::messages_generated::
                            CATCH_CLAUSE_VARIABLE_TYPE_ANNOTATION_MUST_BE_ANY_OR_UNKNOWN_IF_SPECIFIED,
                        Vec::new(),
                    ));
                }
            } else if let tsox_frontend::ast::NodeData::VariableDeclaration(d) = &declaration.data
                && d.initializer.is_some()
            {
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    first_token_loc(d.initializer.as_ref().unwrap()),
                    tsox_core::diagnostics::messages_generated::
                        CATCH_CLAUSE_VARIABLE_CANNOT_HAVE_AN_INITIALIZER,
                    Vec::new(),
                ));
            } else {
                self.check_catch_clause_redeclarations(node, &data.block);
            }
        }
        self.check_statement(&data.block);
    }

    // Go checkCatchClause 末段：catch 参数名与块内 let/const 同名报 TS2483
    fn check_catch_clause_redeclarations(&mut self, catch_clause: &Arc<Node>, block: &Arc<Node>) {
        let sym_map = self.program.symbol_map();
        let Some(catch_locals) = sym_map.locals.get(&catch_clause.id()) else {
            return;
        };
        let Some(block_locals) = sym_map.locals.get(&block.id()) else {
            return;
        };
        for (name, catch_sym) in catch_locals.entries.iter() {
            if let Some(block_sym) = block_locals.entries.get(name)
                && let Some(value_decl) = &block_sym.value_declaration
                && value_decl.kind == SyntaxKind::VariableDeclaration
                && block_sym
                    .flags
                    .contains(tsox_frontend::ast::SymbolFlags::BlockScopedVariable)
            {
                let decl_name = value_decl.name();
                let display = catch_sym.name.clone();
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    value_decl.loc,
                    tsox_core::diagnostics::messages_generated::
                        CANNOT_REDECLARE_IDENTIFIER_0_IN_CATCH_CLAUSE,
                    vec![decl_name.map(|n| n.text().to_string()).unwrap_or(display)],
                ));
            }
        }
    }
}

fn first_token_loc(node: &Arc<Node>) -> tsox_core::core::text::TextRange {
    tsox_core::core::text::TextRange::new(node.loc.pos(), node.loc.pos())
}
