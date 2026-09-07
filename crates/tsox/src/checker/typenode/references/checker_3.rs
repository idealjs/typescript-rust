#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn resolve_alias_body(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        for decl in &symbol.declarations {
            if let NodeData::TypeAliasDeclaration(data) = &decl.data {
                self.push_scope(decl);
                let result = self.get_type_from_type_node(&data.type_node);
                self.pop_scope();
                return result;
            }
        }
        self.error_type()
    }

    pub(crate) fn collect_alias_type_params_and_body(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> (Vec<Arc<Symbol>>, Arc<Node>) {
        let mut tp_symbols = Vec::new();
        let mut type_node = None;
        for decl in &symbol.declarations {
            if let NodeData::TypeAliasDeclaration(data) = &decl.data {
                type_node = Some(Arc::clone(&data.type_node));
                if let Some(tps) = &data.type_parameters {
                    for tp in tps.iter() {
                        if let Some(tp_sym) = self.program.symbol_map().symbol_of(tp) {
                            tp_symbols.push(Arc::clone(tp_sym));
                        }
                    }
                }
                break;
            }
        }
        (
            tp_symbols,
            type_node.unwrap_or_else(|| Arc::clone(&symbol.declarations[0])),
        )
    }

    pub(crate) fn resolve_interface_type(
        &mut self,
        symbol: &Arc<Symbol>,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Type> {
        let arg_types = type_arguments.map(|nodes| {
            nodes
                .iter()
                .map(|a| self.get_type_from_type_node(a))
                .collect()
        });
        self.resolve_interface_type_ex(symbol, arg_types)
    }
}
