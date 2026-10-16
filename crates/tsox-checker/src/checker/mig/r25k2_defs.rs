#![allow(unused_imports)]

use crate::checker::checker_checker::*;
use std::sync::Arc;
use tsox_frontend::ast::{Node, NodeData, Symbol};

pub fn create_instantiated_symbol_table_opt(
    checker: &mut Checker,
    symbols: &[Arc<Symbol>],
    m: Option<&Arc<crate::checker::types_impl_chunk::TypeMapper>>,
) -> SymbolTable { ::tsox_core::fntrace::enter("create_instantiated_symbol_table_opt"); 
    if symbols.is_empty() {
        return SymbolTable::new();
    }
    let mut result = SymbolTable::new();
    for symbol in symbols {
        if let Some(instantiated) = checker.instantiate_symbol(symbol, m) {
            let name = instantiated.name.clone();
            result.insert(name, instantiated);
        }
    }
    result
}

pub fn r25k2_as_heritage_clause(n: &Node) -> &tsox_frontend::ast::node_data_generated::HeritageClauseData { ::tsox_core::fntrace::enter("r25k2_as_heritage_clause"); 
    match &n.data {
        NodeData::HeritageClause(d) => d,
        _ => panic!("AsHeritageClause on wrong node kind"),
    }
}

impl Checker {
    pub(crate) fn add_undefined_to_globals_or_error_on_redeclaration(&mut self) { ::tsox_core::fntrace::enter("add_undefined_to_globals_or_error_on_redeclaration"); 
        let name = self
            .undefined_symbol
            .as_ref()
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "undefined".to_string());
        match self.globals.get(&name).cloned() {
            Some(target) => {
                for d in &target.declarations {
                    let is_type_declaration = matches!(
                        d.kind,
                        tsox_frontend::ast::SyntaxKind::TypeParameter
                            | tsox_frontend::ast::SyntaxKind::ClassDeclaration
                            | tsox_frontend::ast::SyntaxKind::InterfaceDeclaration
                            | tsox_frontend::ast::SyntaxKind::TypeAliasDeclaration
                            | tsox_frontend::ast::SyntaxKind::EnumDeclaration
                    );
                    if !is_type_declaration {
                        let loc = d.name().map(|n| n.loc).unwrap_or(d.loc);
                        let file = self.get_source_file_of_node(d);
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            file,
                            loc,
                            tsox_core::diagnostics::messages_generated::
                                DECLARATION_NAME_CONFLICTS_WITH_BUILT_IN_GLOBAL_IDENTIFIER_0,
                            vec![name.clone()],
                        ));
                    }
                }
            }
            None => {
                if let Some(undef) = self.undefined_symbol.clone() {
                    self.globals.insert(name, undef);
                }
            }
        }
    }
}
