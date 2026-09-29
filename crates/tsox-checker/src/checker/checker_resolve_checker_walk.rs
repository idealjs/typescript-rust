#![allow(unused_imports)]

use crate::checker::checker_resolve::*;
use std::sync::Arc;

impl Checker {
    pub(crate) fn resolve_identifier_scope_symbol(
        &self,
        node: &Arc<Node>,
        meaning: SymbolFlags,
    ) -> Option<Arc<Symbol>> {
        let name = match &node.data {
            tsox_frontend::ast::NodeData::Identifier(data) => data.text.as_str(),
            _ => return None,
        };
        let symbol_map = self.program.symbol_map();

        let mut chain: std::collections::HashMap<u64, (Arc<Node>, Arc<Node>)> =
            std::collections::HashMap::new();
        {
            let mut child = Arc::clone(node);
            let mut ancestor = node.parent();
            while let Some(a) = ancestor {
                chain.insert(a.id(), (Arc::clone(&a), Arc::clone(&child)));
                child = Arc::clone(&a);
                ancestor = a.parent();
            }
        }
        let module_meaning = meaning & SymbolFlags::MODULE_MEMBER;
        let enum_meaning = meaning & SymbolFlags::EnumMember;
        let type_meaning = meaning & SymbolFlags::TYPE;
        if let Some(sym) = self.scope_stack_lookup(
            &chain,
            name,
            meaning,
            module_meaning,
            enum_meaning,
            type_meaning,
        ) {
            return Some(sym);
        }
        if let Some(sym) = self.ancestry_lookup(
            node,
            &chain,
            name,
            meaning,
            module_meaning,
            enum_meaning,
            type_meaning,
        ) {
            return Some(sym);
        }

        if self.function_scope_count > 0
            && name == "arguments"
            && meaning.intersects(SymbolFlags::VARIABLE)
        {
            if let Some(ref sym) = self.arguments_symbol {
                return Some(Arc::clone(sym));
            }
        }

        if let Some(sym) = self.globals.get(name) {
            if sym
                .flags
                .intersects(meaning.union(SymbolFlags::GlobalLookup))
            {
                return Some(Arc::clone(sym));
            }
        }

        // Go NameResolver：JS 文件中 require 调用的 callee 解析失败时回退
        // 到 requireSymbol（类型 any），避免报 cannot-find-name
        if name == "require" && node_parent_is_require_call(node) {
            if let Some(ref sym) = self.require_symbol {
                return Some(Arc::clone(sym));
            }
        }

        None
    }
}
