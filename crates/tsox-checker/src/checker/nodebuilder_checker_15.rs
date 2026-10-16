#![allow(unused_imports)]

use std::sync::Arc;

use tsox_frontend::ast::{Node, Symbol, SymbolFlags};
use tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS;

use crate::checker::checker::Checker;
use crate::checker::symbolaccessibility_symbol_table_id::{
    get_qualified_left_meaning, has_non_global_augmentation_external_module_symbol,
};

impl Checker {
    // Go symbolToTypeNode（nodebuilderimpl.go:680-686）链名合成：链段以 `.`
    // 连接；单段链与别名符号交由调用方既有通路（accessible_chain_full_name）
    pub(crate) fn symbol_chain_qualified_name(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
    ) -> Option<String> { ::tsox_core::fntrace::enter("symbol_chain_qualified_name");
        if symbol.flags.intersects(SymbolFlags::Alias) {
            return None;
        }
        let enclosing = self
            .type_render_enclosing
            .clone()
            .or_else(|| self.display_enclosing_node.clone());
        // Go lookupSymbolChainWorker（nodebuilderimpl.go:1105-1118）：无
        // enclosingDeclaration 且未开 UseFullyQualifiedType 时链退化为符号自身
        let enclosing = match enclosing {
            Some(enc) => enc,
            None => return None,
        };
        // .types 渲染 flags 不含 UseAliasDefinedOutsideCurrentScope →
        // yieldModuleSymbol = true（nodebuilderimpl.go:683 symbolToTypeNode）
        let chain = self.render_symbol_chain(symbol, Some(&enclosing), meaning, true, true);
        if chain.len() <= 1 {
            return None;
        }
        Some(
            chain
                .iter()
                .map(|s| s.name.clone())
                .collect::<Vec<_>>()
                .join("."),
        )
    }

    // Go NodeBuilderImpl.getSymbolChain（nodebuilderimpl.go:1127-1177）的
    // Checker 渲染侧移植：可及链（getAccessibleSymbolChain）为空或链头需
    // 限定（needsQualification）时，经 getContainersOfSymbol 上爬父命名空间
    // 链补全限定段——namespace 外引用 typeof A.B 而非 typeof B
    pub(crate) fn render_symbol_chain(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing: Option<&Arc<Node>>,
        meaning: SymbolFlags,
        end_of_chain: bool,
        yield_module_symbol: bool,
    ) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("render_symbol_chain");
        let mut accessible_symbol_chain =
            self.get_accessible_symbol_chain(symbol, enclosing, meaning, false);
        let qualifier_meaning = if accessible_symbol_chain.len() > 1 {
            get_qualified_left_meaning(meaning)
        } else {
            meaning
        };
        if accessible_symbol_chain.is_empty()
            || self.needs_qualification(&accessible_symbol_chain[0], enclosing, qualifier_meaning)
        {
            let root = match accessible_symbol_chain.first() {
                Some(first) => Arc::clone(first),
                None => Arc::clone(symbol),
            };
            let mut parents = self.get_containers_of_symbol(&root, enclosing, meaning, true);
            parents.sort_by(|a, b| self.compare_symbols(a, b).cmp(&0));
            for parent in &parents {
                let parent_chain = self.render_symbol_chain(
                    parent,
                    enclosing,
                    get_qualified_left_meaning(meaning),
                    false,
                    yield_module_symbol,
                );
                if parent_chain.is_empty() {
                    continue;
                }
                if let Some(exported) = parent.exports.get(INTERNAL_SYMBOL_NAME_EXPORT_EQUALS) {
                    if self.get_symbol_if_same_reference(exported, symbol).is_some() {
                        accessible_symbol_chain = parent_chain;
                        break;
                    }
                }
                let mut next_syms = accessible_symbol_chain.clone();
                if next_syms.is_empty() {
                    let fallback = self
                        .get_alias_for_symbol_in_container(parent, symbol)
                        .unwrap_or_else(|| Arc::clone(symbol));
                    next_syms = vec![fallback];
                }
                accessible_symbol_chain = parent_chain;
                accessible_symbol_chain.extend(next_syms);
                break;
            }
        }
        if !accessible_symbol_chain.is_empty() {
            return accessible_symbol_chain;
        }
        if end_of_chain
            || !symbol
                .flags
                .intersects(SymbolFlags::TypeLiteral | SymbolFlags::ObjectLiteral)
        {
            if !end_of_chain
                && !yield_module_symbol
                && symbol
                    .declarations
                    .iter()
                    .any(|d| has_non_global_augmentation_external_module_symbol(d))
            {
                return Vec::new();
            }
            return vec![Arc::clone(symbol)];
        }
        Vec::new()
    }
}
