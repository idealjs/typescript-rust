#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn resolve_interface_type_ex(
        &mut self,
        symbol: &Arc<Symbol>,
        type_args: Option<Vec<Arc<Type>>>,
    ) -> Arc<Type> {
        let has_type_args = type_args.is_some();
        if !has_type_args {
            if let Some(cached) = self
                .type_alias_links
                .get(symbol)
                .and_then(|l| l.declared_type.clone())
            {
                if !crate::checker::utilities::is_type_error(&cached) {
                    return cached;
                }
            }
        }

        let instantiation_key: Option<Vec<usize>> = type_args.as_ref().map(|args| {
            let mut key = Vec::with_capacity(args.len() + 1);
            key.push(Arc::as_ptr(symbol) as *const Symbol as usize);
            key.extend(
                args.iter()
                    .map(|t| Arc::as_ptr(t) as *const crate::checker::types::Type as usize),
            );
            key
        });

        let pinned_args: Option<Vec<Arc<Type>>> = type_args.clone();
        if let Some(key) = &instantiation_key
            && let Some(cached) = self.interface_instantiation_cache.get(key)
        {
            return Arc::clone(&cached.1);
        }

        let key = Arc::as_ptr(symbol) as *const crate::ast::Symbol;
        if !self.push_type_resolution(
            key,
            crate::checker::checker::TypeResolutionProperty::DeclaredType,
        ) {
            self.heritage_degraded_events += 1;
            return self.error_type();
        }

        let interface_decls: Vec<Arc<Node>> = symbol
            .declarations
            .iter()
            .filter(|d| matches!(d.data, NodeData::InterfaceDeclaration(_)))
            .cloned()
            .collect();

        let epoch_at_entry = self.heritage_degraded_events;
        let mut heritage_degraded = false;
        let result = match interface_decls.first() {
            Some(first) => {
                let data = match &first.data {
                    NodeData::InterfaceDeclaration(d) => d,
                    _ => unreachable!(),
                };

                let tp_symbols = match &data.type_parameters {
                    Some(tps) => {
                        let sym_map = self.program.symbol_map();
                        let collected: Vec<Arc<Symbol>> = tps
                            .iter()
                            .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                            .collect();
                        collected
                    }
                    None => Vec::new(),
                };

                let arg_types: Vec<Arc<Type>> = type_args.unwrap_or_default();
                if has_type_args {
                    self.push_interface_type_argument_mapping(
                        &interface_decls,
                        &tp_symbols,
                        &arg_types,
                    );
                }

                self.push_scope(
                    symbol
                        .declarations
                        .iter()
                        .next()
                        .expect("interface has a declaration"),
                );

                let merged_members: Vec<Arc<Node>> = interface_decls
                    .iter()
                    .flat_map(|decl| match &decl.data {
                        NodeData::InterfaceDeclaration(d) => d.members.iter().cloned(),
                        _ => unreachable!(),
                    })
                    .collect();
                let merged_list = Arc::new(NodeList::new(merged_members));

                let saved_static = self.in_static_member_type;
                self.in_static_member_type = false;
                let own_result = self.build_interface_type_from_members(&merged_list);
                self.in_static_member_type = saved_static;

                let mut heritage_base_degraded = false;
                let base_types = self
                    .collect_interface_base_types(&interface_decls, &mut heritage_base_degraded);
                if heritage_base_degraded {
                    heritage_degraded = true;
                }
                self.pop_scope();
                if has_type_args {
                    self.type_argument_stack.pop();
                }
                let result = if base_types.is_empty() {
                    own_result.clone()
                } else {
                    let mut merged = own_result.clone();
                    for (_, base) in &base_types {
                        merged = self.merge_interface_type_with_base(&merged, base);
                    }
                    merged
                };

                if !has_type_args && !base_types.is_empty() {
                    self.report_interface_extends_incompatibilities(
                        symbol,
                        &interface_decls,
                        &own_result,
                        &base_types,
                    );
                }

                {
                    let result_mut = Arc::as_ptr(&result) as *mut crate::checker::types::Type;
                    unsafe {
                        (*result_mut).symbol = Some(Arc::clone(symbol));
                        if has_type_args && let TypeData::Object(o) = &mut (*result_mut).data {
                            o.type_arguments = arg_types.clone();
                        }
                    }
                }
                result
            }
            None => self.error_type(),
        };
        self.pop_type_resolution();

        if self.heritage_degraded_events != epoch_at_entry {
            heritage_degraded = true;
        }

        let mut degraded_accepted = false;
        if heritage_degraded {
            let sym_key = Arc::as_ptr(symbol) as *const crate::ast::Symbol as usize;
            let retries = self.heritage_retry_counts.entry(sym_key).or_insert(0);
            *retries += 1;
            degraded_accepted = *retries > crate::checker::checker::HERITAGE_RETRY_LIMIT;
        }
        let cache_result = !heritage_degraded || degraded_accepted;
        if degraded_accepted && self.heritage_degraded_events != epoch_at_entry {
            self.heritage_degraded_events = epoch_at_entry;
        }
        if heritage_degraded {
            self.degraded_type_ptrs.insert(result.id);
        }
        if !has_type_args && cache_result {
            self.type_alias_links.get_or_default(symbol).declared_type = Some(result.clone());
        }
        if let Some(key) = instantiation_key {
            if cache_result {
                let pin = pinned_args.clone().unwrap_or_default();
                self.interface_instantiation_cache
                    .insert(key, (pin, Arc::clone(&result)));
            }
        }
        result
    }
}
