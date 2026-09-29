#![allow(unused_imports)]

#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
#[allow(unused_imports)]
use std::sync::Arc;

impl Checker {
    pub fn get_target_of_binary_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        let NodeData::BinaryExpression(d) = &node.data else {
            return None;
        };
        if d.operator_token.kind == SyntaxKind::EqualsToken {
            let _left_type = self.check_expression_cached(&d.left);
            return self.resolve_entity_name(
                &d.right,
                SymbolFlags::VALUE | SymbolFlags::TYPE | SymbolFlags::NAMESPACE,
                true,
                true,
                Some(node),
            );
        }
        if node.kind == SyntaxKind::ElementAccessExpression
            || node.kind == SyntaxKind::PropertyAccessExpression
        {
            return self.get_target_of_access_expression(node);
        }
        None
    }

    pub fn get_export_of_module(
        &mut self,
        module_symbol: &Arc<Symbol>,
        name: &str,
        _location: &Arc<Node>,
        _dont_resolve_alias: bool,
    ) -> Option<Arc<Symbol>> {
        module_symbol.exports.get(name).cloned()
    }

    pub fn resolve_es_module_symbol(
        &mut self,
        imported_symbol: Option<&Arc<Symbol>>,
        _location: &Arc<Node>,
        _module_specifier: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        imported_symbol.cloned()
    }

    pub fn can_have_synthetic_default(
        &mut self,
        _location: Option<&Arc<Node>>,
        module_symbol: &Arc<Symbol>,
        dont_resolve_alias: bool,
        _specifier: &Arc<Node>,
    ) -> bool {
        let resolved = self.resolve_external_module_symbol(module_symbol, true);
        (!(dont_resolve_alias || !Arc::ptr_eq(&resolved, module_symbol)))
            || self.module_can_have_synthetic_default(module_symbol)
    }
}
