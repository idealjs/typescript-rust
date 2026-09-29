#![allow(unused_imports)]

use std::sync::Arc;

use tsox_frontend::ast::{
    CheckFlags, NodeList, Node, Symbol, SymbolFlags,
};
use tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS;

use crate::checker::checker_checker_checker::Checker;
use crate::checker::nodecopy_builder::NodeBuilderImpl;
use crate::checker::symboltracker::NodeBuilderFlags;
use crate::checker::mig::m2f_3::r24k13_defs::has_non_global_augmentation_external_module_symbol;
use crate::checker::symbolaccessibility_symbol_table_id::get_qualified_left_meaning;
use crate::checker::mig::m2g::r21k9_defs::count_path_components;
use tsox_core::tspath::get_normalized_absolute_path::path_is_relative;

struct SortedSymbolNamePair {
    sym: Arc<Symbol>,
    name: String,
}

pub trait R28K4NodeBuilderExt {
    fn lookup_instantiated_type_argument_nodes(
        &mut self,
        chain: &[Arc<Symbol>],
        index: usize,
    ) -> Option<Arc<NodeList>>;
    fn get_symbol_chain(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
        end_of_chain: bool,
        yield_module_symbol: bool,
    ) -> Vec<Arc<Symbol>>;
}

fn builder_checker() -> &'static mut Checker {
    unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() }
}

impl R28K4NodeBuilderExt for NodeBuilderImpl<'_> {
    fn lookup_instantiated_type_argument_nodes(
        &mut self,
        chain: &[Arc<Symbol>],
        index: usize,
    ) -> Option<Arc<NodeList>> {
        if !self.should_write_type_parameters_in_qualified_name(chain, index) {
            return None;
        }
        let ch = builder_checker();
        let symbol = &chain[index];
        let next_symbol = &chain[index + 1];
        if !next_symbol.check_flags.intersects(CheckFlags::Instantiated) {
            return None;
        }

        let mut target_symbol = Arc::clone(symbol);
        if symbol.flags.intersects(SymbolFlags::Alias)
            && !ch.can_get_type_parameters_of_class_or_interface(symbol)
        {
            target_symbol = ch.resolve_alias(symbol);
        }

        if !ch.can_get_type_parameters_of_class_or_interface(&target_symbol) {
            return None;
        }

        let mut params = self.get_type_parameters_of_class_or_interface(&target_symbol);
        let target_mapper = ch
            .value_symbol_links
            .get(next_symbol)
            .and_then(|l| l.mapper.clone());
        if let Some(mapper) = target_mapper {
            params = params.iter().map(|t| mapper.map(t)).collect();
        }
        self.map_to_type_nodes(&params, false)
    }

    fn get_symbol_chain(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
        end_of_chain: bool,
        yield_module_symbol: bool,
    ) -> Vec<Arc<Symbol>> {
        let ch = builder_checker();
        let enclosing = self.ctx.borrow().enclosing_declaration.clone();
        let use_only_external_aliasing = self
            .ctx
            .borrow()
            .flags
            .contains(NodeBuilderFlags::UseOnlyExternalAliasing);
        let mut accessible_symbol_chain = ch.get_accessible_symbol_chain(
            symbol,
            enclosing.as_ref(),
            meaning,
            use_only_external_aliasing,
        );
        let mut qualifier_meaning = meaning;
        if accessible_symbol_chain.len() > 1 {
            qualifier_meaning = get_qualified_left_meaning(meaning);
        }
        if accessible_symbol_chain.is_empty()
            || ch.needs_qualification(
                &accessible_symbol_chain[0],
                enclosing.as_ref(),
                qualifier_meaning,
            )
        {
            let root = match accessible_symbol_chain.first() {
                Some(first) => Arc::clone(first),
                None => Arc::clone(symbol),
            };
            let parents = ch.get_containers_of_symbol(&root, enclosing.as_ref(), meaning);
            if !parents.is_empty() {
                let mut parent_specifiers: Vec<SortedSymbolNamePair> = parents
                    .into_iter()
                    .map(|sym| {
                        if sym
                            .declarations
                            .iter()
                            .any(|d| has_non_global_augmentation_external_module_symbol(ch, d))
                        {
                            let specifier = self.get_specifier_for_module_symbol(
                                &sym,
                                tsox_core::core::compiler_options_kinds::ResolutionMode::None
                                    as u32,
                            );
                            SortedSymbolNamePair {
                                sym,
                                name: specifier,
                            }
                        } else {
                            SortedSymbolNamePair {
                                sym,
                                name: String::new(),
                            }
                        }
                    })
                    .collect();
                parent_specifiers.sort_by(|a, b| sort_by_best_name(ch, a, b));
                for pair in &parent_specifiers {
                    let parent = &pair.sym;
                    let parent_chain = self.get_symbol_chain(
                        parent,
                        get_qualified_left_meaning(meaning),
                        false,
                        yield_module_symbol,
                    );
                    if !parent_chain.is_empty() {
                        if let Some(exported) =
                            parent.exports.get(INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
                        {
                            if ch.get_symbol_if_same_reference(exported, symbol).is_some() {
                                accessible_symbol_chain = parent_chain;
                                break;
                            }
                        }
                        let mut next_syms = accessible_symbol_chain.clone();
                        if next_syms.is_empty() {
                            let fallback = ch
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
                    .any(|d| has_non_global_augmentation_external_module_symbol(ch, d))
            {
                return Vec::new();
            }
            return vec![Arc::clone(symbol)];
        }
        Vec::new()
    }
}

fn sort_by_best_name(
    ch: &Checker,
    a: &SortedSymbolNamePair,
    b: &SortedSymbolNamePair,
) -> std::cmp::Ordering {
    let specifier_a = &a.name;
    let specifier_b = &b.name;
    if !specifier_a.is_empty() && !specifier_b.is_empty() {
        let is_b_relative = path_is_relative(specifier_b);
        if path_is_relative(specifier_a) == is_b_relative {
            return count_path_components(specifier_a).cmp(&count_path_components(specifier_b));
        }
        if is_b_relative {
            return std::cmp::Ordering::Less;
        }
        return std::cmp::Ordering::Greater;
    }
    ch.compare_symbols(&a.sym, &b.sym).cmp(&0)
}

use std::cell::Cell;

thread_local! {
    static APPARENT_ARGUMENT_COUNT: Cell<Option<usize>> = const { Cell::new(None) };
}

pub fn apparent_argument_count() -> Option<usize> {
    APPARENT_ARGUMENT_COUNT.with(|c| c.get())
}

pub fn set_apparent_argument_count(value: Option<usize>) {
    APPARENT_ARGUMENT_COUNT.with(|c| c.set(value));
}

pub struct SendCheckerPtr(*mut Checker);
unsafe impl Send for SendCheckerPtr {}

impl SendCheckerPtr {
    pub fn from_checker(ch: &Checker) -> Self {
        Self(ch as *const Checker as *mut Checker)
    }

    pub unsafe fn get(&self) -> &'static mut Checker {
        unsafe { &mut *self.0 }
    }
}
