#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    pub(crate) fn resolve_alias_body(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> {
        for decl in &symbol.declarations {
            if let NodeData::TypeAliasDeclaration(data) = &decl.data {
                // body 按别名声明的词法作用域解析（Go 节点级查找）；
                // 外层栈可能是推断中构造的无关作用域，不得泄漏进来
                let saved_scopes = std::mem::take(&mut self.scope_stack);
                let mut scope_chain: Vec<u64> = Vec::new();
                let mut cur = decl.parent();
                while let Some(c) = cur {
                    scope_chain.push(c.id());
                    cur = c.parent();
                }
                scope_chain.reverse();
                self.scope_stack = scope_chain;
                self.push_scope(decl);
                let result = self.get_type_from_type_node(&data.type_node);
                self.pop_scope();
                self.scope_stack = saved_scopes;
                // 环窗口内的 error（别名自引用经预缓存环断路）不得驻留节点
                // 缓存：窗口外的解析会拿到同一份 error 而永不重算
                if crate::checker::utilities::is_type_error(&result) {
                    self.uncache_type_node(&data.type_node);
                }
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
