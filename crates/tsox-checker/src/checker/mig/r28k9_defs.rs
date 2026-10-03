use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use tsox_frontend::ast::{CheckFlags, Node, Symbol, SymbolFlags};

use crate::checker::checker::Checker;
use crate::checker::types::{IterationTypeKind, IterationTypes, IterationUse, Type, TypeFlags};
use crate::checker::types_impl_chunk_3::ValueSymbolLinks;

thread_local! {
    static ITERATION_TYPES_CACHE: RefCell<HashMap<(u32, u8), IterationTypes>> =
        RefCell::new(HashMap::new());
}

fn iteration_types_cache_get(key: &(u32, u8)) -> Option<IterationTypes> { ::tsox_core::fntrace::enter("iteration_types_cache_get"); 
    ITERATION_TYPES_CACHE.with(|c| c.borrow().get(key).cloned())
}

fn iteration_types_cache_insert(key: (u32, u8), result: IterationTypes) { ::tsox_core::fntrace::enter("iteration_types_cache_insert"); 
    ITERATION_TYPES_CACHE.with(|c| {
        c.borrow_mut().insert(key, result);
    });
}

fn iteration_use_cache_discriminant(use_: IterationUse) -> u8 { ::tsox_core::fntrace::enter("iteration_use_cache_discriminant"); 
    match use_ {
        IterationUse::ForOf { for_await } => {
            if for_await {
                2
            } else {
                1
            }
        }
        IterationUse::Spread => 3,
        IterationUse::Destructuring => 4,
        IterationUse::Element => 5,
        IterationUse::YieldStar { is_async } => {
            if is_async {
                7
            } else {
                6
            }
        }
        IterationUse::GeneratorReturnType { is_async } => {
            if is_async {
                9
            } else {
                8
            }
        }
    }
}

impl IterationTypes {
    pub(crate) fn get_type(&self, type_kind: IterationTypeKind) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_type"); 
        match type_kind {
            IterationTypeKind::YIELD => self.yield_type.clone(),
            IterationTypeKind::RETURN => self.return_type.clone(),
            IterationTypeKind::NEXT => self.next_type.clone(),
        }
    }
}

impl Checker {
    pub(crate) fn get_iteration_types_of_generator_function_return_type(
        &mut self,
        t: &Arc<Type>,
        is_async_generator: bool,
    ) -> IterationTypes { ::tsox_core::fntrace::enter("get_iteration_types_of_generator_function_return_type"); 
        if t.flags.contains(TypeFlags::Any) {
            let any = self.get_any_type();
            return IterationTypes {
                yield_type: Some(Arc::clone(&any)),
                return_type: Some(Arc::clone(&any)),
                next_type: Some(any),
            };
        }
        let result = self.get_iteration_types_of_iterable(
            t,
            IterationUse::GeneratorReturnType {
                is_async: is_async_generator,
            },
            None,
        );
        if result.has_types() {
            return result;
        }
        let fast: &[&str] = if is_async_generator {
            &["AsyncGenerator", "AsyncIterator"]
        } else {
            &["Generator", "Iterator", "IterableIterator"]
        };
        let mut pending = Vec::new();
        self.iteration_types_of_iterator(t, fast, None, is_async_generator, &mut pending)
    }

    pub(crate) fn get_iteration_types_of_iterable(
        &mut self,
        t: &Arc<Type>,
        use_: IterationUse,
        error_node: Option<&Node>,
    ) -> IterationTypes { ::tsox_core::fntrace::enter("get_iteration_types_of_iterable"); 
        let reduced = self.get_reduced_type(t);
        if reduced.flags.contains(TypeFlags::Any) {
            let any = self.get_any_type();
            return IterationTypes {
                yield_type: Some(Arc::clone(&any)),
                return_type: Some(Arc::clone(&any)),
                next_type: Some(any),
            };
        }
        let key = (reduced.id, iteration_use_cache_discriminant(use_));
        let mut no_cache = false;
        if let Some(cached) = iteration_types_cache_get(&key) {
            if error_node.is_none() || cached.has_types() {
                return cached;
            }
            no_cache = true;
        }
        let result = self.iteration_types_of_iterable(use_, &reduced, None);
        if !result.has_types() {
            let allows_async = matches!(
                use_,
                IterationUse::ForOf { for_await: true }
                    | IterationUse::YieldStar { is_async: true }
                    | IterationUse::GeneratorReturnType { is_async: true }
            );
            self.report_type_not_iterable_node(error_node, &reduced, allows_async);
        }
        if !no_cache {
            iteration_types_cache_insert(key, result.clone());
        }
        result
    }

    fn report_type_not_iterable_node(
        &mut self,
        error_node: Option<&Node>,
        t: &Arc<Type>,
        allow_async: bool,
    ) { ::tsox_core::fntrace::enter("report_type_not_iterable_node"); 
        let Some(node) = error_node else { return };
        let type_str = self.type_to_string(t);
        let message = if allow_async {
            tsox_core::diagnostics::messages_generated::
                TYPE_0_MUST_HAVE_A_SYMBOL_ASYNCITERATOR_METHOD_THAT_RETURNS_AN_ASYNC_ITERATOR
        } else {
            tsox_core::diagnostics::messages_generated::
                TYPE_0_MUST_HAVE_A_SYMBOL_ITERATOR_METHOD_THAT_RETURNS_AN_ITERATOR
        };
        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
            self.current_file.clone(),
            node.loc,
            message,
            vec![type_str],
        ));
    }
}

pub(crate) fn get_spread_symbol(c: &mut Checker, prop: &Arc<Symbol>, readonly: bool) -> Arc<Symbol> { ::tsox_core::fntrace::enter("get_spread_symbol"); 
    let is_setonly_accessor = prop.flags.contains(SymbolFlags::SetAccessor)
        && !prop.flags.contains(SymbolFlags::GetAccessor);
    if !is_setonly_accessor && readonly == c.is_readonly_symbol_for_identity(prop) {
        return Arc::clone(prop);
    }
    let mut sym = Symbol::new(
        SymbolFlags::Property.union(prop.flags.intersection(SymbolFlags::Optional)),
        prop.name.clone(),
    );
    if readonly {
        sym.check_flags |= CheckFlags::Readonly;
    }
    sym.declarations = prop.declarations.clone();
    let symbol = Arc::new(sym);
    let resolved = if is_setonly_accessor {
        c.undefined_type()
    } else {
        c.get_type_of_symbol(prop)
    };
    c.value_symbol_links.insert(
        &symbol,
        ValueSymbolLinks {
            resolved_type: Some(resolved),
            name_type: c
                .value_symbol_links
                .get(prop)
                .and_then(|l| l.name_type.clone()),
            ..Default::default()
        },
    );
    symbol
}
