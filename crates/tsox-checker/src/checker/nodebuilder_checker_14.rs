#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::{Node, Symbol, SymbolFlags};
use tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS;

use crate::checker::checker::Checker;
use crate::checker::symbolaccessibility_symbol_table_id::get_qualified_left_meaning;
use crate::checker::mig::m2f_3::r24k13_defs::has_non_global_augmentation_external_module_symbol;

struct ChainParentNamePair {
    sym: Arc<Symbol>,
    name: String,
}

impl Checker {
    /// Go NodeBuilderImpl.getSymbolChain（nodebuilderimpl.go:1127-1192）在
    /// .types 字符串化通道的等价实现：实体名链——链头就地可及且无需限定时
    /// 按链呈现（import 别名链头按别名，getSymbolChain 尾段）；链空或链头
    /// 需限定（needsQualification，容器内同名遮蔽）时经 getContainersOfSymbol
    /// 上爬补容器段。enclosing 取 .types 基线传入的 node.Parent
    /// （type_render_enclosing，writeTypeOrSymbol 的 TypeToTypeNode(t,
    /// node.Parent, …)）。
    ///
    /// 本仓 binder 偏差的两处对齐（ts 缺省事实：脚本顶层符号 parent=nil，
    /// binder.go declareSourceFileMember→GetLocals(file)+nil parent）：
    /// 一、本仓 binder 给脚本文件造了以路径为名的 ValueModule 文件符号并设为
    /// 顶层符号 parent（binder.rs bind_source_file），容器上爬须剔除脚本文件
    /// 符号段（tsc getSymbolChain 尾段对外部模块 root 的跳过语义在脚本域的
    /// 对应物）；二、链空时仅「非纯别名遮蔽」（容器内同名 interface+import
    /// 合并符号，如 namespace A.M 内 interface M + import M = Z.M）才触发
    /// 容器限定——纯别名遮蔽（import M = Z.M 的别名就地可见）时取裸名
    /// （corpus_go_types: moduleSharesNameWithImportDeclarationInsideIt4 行17
    /// typeof A.M 对照 it6 行17 typeof M 的分野）。
    pub(crate) fn get_symbol_chain_for_display(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
        end_of_chain: bool,
        yield_module_symbol: bool,
    ) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_symbol_chain_for_display");
        let enclosing = self
            .type_render_enclosing
            .clone()
            .or_else(|| self.display_enclosing_node.clone());
        // .types 通道不带 UseOnlyExternalAliasing（type_symbol_baseline.go:408
        // 的 flags 无该位）→ useOnlyExternalAliasing = false
        let mut accessible_symbol_chain =
            self.get_accessible_symbol_chain(symbol, enclosing.as_ref(), meaning, false);
        let mut qualifier_meaning = meaning;
        if accessible_symbol_chain.len() > 1 {
            qualifier_meaning = get_qualified_left_meaning(meaning);
        }
        let chain_head_needs_qualification = !accessible_symbol_chain.is_empty()
            && self.needs_qualification(
                &accessible_symbol_chain[0],
                enclosing.as_ref(),
                qualifier_meaning,
            );
        let empty_chain_shadowed_by_merged_symbol = accessible_symbol_chain.is_empty()
            && self.needs_qualification(symbol, enclosing.as_ref(), meaning)
            && !self.shadowed_by_pure_alias(symbol, enclosing.as_ref());
        if chain_head_needs_qualification || empty_chain_shadowed_by_merged_symbol {
            let root = match accessible_symbol_chain.first() {
                Some(first) => Arc::clone(first),
                None => Arc::clone(symbol),
            };
            let mut parents =
                self.get_containers_of_symbol(&root, enclosing.as_ref(), meaning, true);
            parents.retain(|p| !Self::is_script_file_module_symbol(p));
            if !parents.is_empty() {
                let mut parent_specifiers: Vec<ChainParentNamePair> = parents
                    .into_iter()
                    .map(|sym| {
                        // NodeBuilder 通道 getSpecifierForModuleSymbol 为桩（空
                        // 说明符）——父段比较统一落 compareSymbols
                        ChainParentNamePair {
                            sym,
                            name: String::new(),
                        }
                    })
                    .collect();
                parent_specifiers.sort_by(|a, b| {
                    self.compare_symbols(&a.sym, &b.sym).cmp(&0)
                });
                for pair in &parent_specifiers {
                    let parent = &pair.sym;
                    let parent_chain = self.get_symbol_chain_for_display(
                        parent,
                        get_qualified_left_meaning(meaning),
                        false,
                        yield_module_symbol,
                    );
                    if !parent_chain.is_empty() {
                        if let Some(exported) = parent.exports.get(INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
                        {
                            if self.get_symbol_if_same_reference(exported, symbol).is_some() {
                                // 链根即 export= 隔代自身：链退化为父链
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
                    .any(|d| has_non_global_augmentation_external_module_symbol(self, d))
            {
                return Vec::new();
            }
            return vec![Arc::clone(symbol)];
        }
        Vec::new()
    }

    /// 脚本文件（非外部模块）的文件符号：本仓 binder 为脚本 SourceFile 也建
    /// ValueModule 路径名符号（binder.rs bind_source_file），tsc 中脚本无文件
    /// 符号、顶层符号无 parent——该符号不得充当容器链段
    pub(crate) fn is_script_file_module_symbol(symbol: &Arc<Symbol>) -> bool {
        symbol.declarations.iter().any(|d| {
            d.kind == tsox_frontend::ast::SyntaxKind::SourceFile
                && !Checker::is_external_or_common_js_module(d)
        })
    }

    /// 就地同名首表项是否为「纯别名」（flags 恰为 Alias，如 import M = Z.M 的
    /// 别名、目标异于 symbol）：纯别名遮蔽不触发容器限定（it6 行17 裸名），
    /// interface+import 合并符号遮蔽才触发（it4 行17 A.M）
    fn shadowed_by_pure_alias(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
    ) -> bool { ::tsox_core::fntrace::enter("shadowed_by_pure_alias");
        let tables = self.collect_symbol_tables_in_scope(enclosing_declaration);
        for info in &tables {
            let Some(res) = info.table.get(&symbol.name) else {
                continue;
            };
            let shadow = self.get_merged_symbol(res);
            if shadow.id() == symbol.id() {
                return false;
            }
            let pure_alias = shadow.flags == SymbolFlags::Alias
                || (shadow.flags.intersects(SymbolFlags::Alias)
                    && shadow.flags.intersects(SymbolFlags::Assignment));
            if pure_alias {
                let resolved = self.resolve_alias(&shadow);
                return resolved.id() != symbol.id();
            }
            return false;
        }
        false
    }
}
