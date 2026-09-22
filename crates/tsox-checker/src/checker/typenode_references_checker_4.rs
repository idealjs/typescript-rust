#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    fn push_interface_shell(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {
        let key = Arc::as_ptr(symbol) as *const tsox_frontend::ast::Symbol as usize;
        if self.pending_interface_shells.contains_key(&key) {
            return None;
        }
        let shell = Arc::new(Type {
            flags: crate::checker::types::TypeFlags::Object,
            object_flags: crate::checker::types::ObjectFlags::Reference,
            id: crate::checker::types::next_type_id(),
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: crate::checker::types::TypeData::Object(Default::default()),
        });
        self.pending_interface_shells.insert(key, Arc::clone(&shell));
        Some(shell)
    }

    pub fn resolve_interface_type_ex(
        &mut self,
        symbol: &Arc<Symbol>,
        type_args: Option<Vec<Arc<Type>>>,
    ) -> Arc<Type> {
        let merged_symbol = self.get_merged_symbol(symbol);
        let symbol: &Arc<Symbol> = &merged_symbol;
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

        let key = Arc::as_ptr(symbol) as *const tsox_frontend::ast::Symbol;
                if !self.push_type_resolution(
            key,
            crate::checker::checker::TypeResolutionProperty::DeclaredType,
        ) {
            if let Some(shell) = self.pending_interface_shells.get(&(key as usize)) {
                let shell = Arc::clone(shell);
                let args = type_args.unwrap_or_default();
                if args.is_empty() {
                    return shell;
                }
                // 壳让外层构建不完整：标记 degraded 使其不进缓存（重试拿完整版），
                // relater 对 degraded 放行；空壳型自身也标 degraded
                self.heritage_degraded_events += 1;
                let rebuilt = self.rebuild_with_type_arguments(&shell, args);
                self.degraded_type_ptrs.insert(rebuilt.id);
                return rebuilt;
            }
            self.heritage_degraded_events += 1;
            return self.error_type();
        }
        let shell_key = key as usize;
        let shell = self.push_interface_shell(symbol);
        let shell_cleanup = |checker: &mut Checker| {
            if shell.is_some() {
                checker.pending_interface_shells.remove(&shell_key);
            }
        };

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
                // 声明类型（及其实例）的成员按声明自身的类型参数求值，
                // 不受外层进行中的实例化映射影响（对齐 Go 声明类型与实例化解耦）
                let saved_type_argument_stack = std::mem::take(&mut self.type_argument_stack);
                if has_type_args {
                    self.push_interface_type_argument_mapping(
                        &interface_decls,
                        &tp_symbols,
                        &arg_types,
                    );
                }

                let saved_scope_stack = std::mem::take(&mut self.scope_stack);
                for scope_id in crate::checker::checker_resolve_checker::lexical_scope_chain_ids(
                    symbol
                        .declarations
                        .iter()
                        .next()
                        .expect("interface has a declaration"),
                )
                .into_iter()
                .rev()
                {
                    self.scope_stack.push(scope_id);
                }

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
                if has_type_args {
                    // 增强声明的同名 T 是独立符号：实例化标记须覆盖全部声明，
                    // 否则增强成员（如 es2015 Array.find）的类型参数悬空
                    let sym_map = self.program.symbol_map();
                    let mut all_tp_symbols: Vec<Arc<Symbol>> = tp_symbols.clone();
                    let mut all_args: Vec<Arc<Type>> = arg_types.clone();
                    for decl in interface_decls.iter().skip(1) {
                        let NodeData::InterfaceDeclaration(d) = &decl.data else {
                            continue;
                        };
                        let Some(tps) = &d.type_parameters else {
                            continue;
                        };
                        for (i, tp) in tps.iter().enumerate() {
                            let Some(tp_sym) = sym_map.symbol_of(tp) else {
                                continue;
                            };
                            if all_tp_symbols.iter().any(|a| Arc::ptr_eq(a, tp_sym)) {
                                continue;
                            }
                            let Some(arg) = arg_types.get(i) else {
                                continue;
                            };
                            all_tp_symbols.push(Arc::clone(tp_sym));
                            all_args.push(Arc::clone(arg));
                        }
                    }
                    let all_tp_types: Vec<Arc<Type>> = all_tp_symbols
                        .iter()
                        .map(|s| self.get_type_parameter_from_symbol(s))
                        .collect();
                    self.mark_structured_members_instantiated(
                        &own_result,
                        symbol,
                        &all_tp_symbols,
                        &all_tp_types,
                        &all_args,
                    );
                }

                let mut heritage_base_degraded = false;
                let base_types = self
                    .collect_interface_base_types(&interface_decls, &mut heritage_base_degraded);
                // 基类是空壳（解析重入期产物）时合并结果只有自有成员：按 degraded
                // 处理——结果不进缓存、标 degraded，后续引用重建拿完整版
                let base_shell = base_types.iter().any(|(_, bt)| {
                    bt.as_structured().is_some_and(|s| s.members.entries.is_empty())
                });
                if heritage_base_degraded || base_shell {
                    heritage_degraded = true;
                }
                self.scope_stack = saved_scope_stack;
                self.type_argument_stack = saved_type_argument_stack;
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
                    self.report_interface_simultaneous_extends(
                        symbol,
                        &interface_decls,
                        &own_result,
                        &base_types,
                    );
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
        shell_cleanup(self);

        if self.heritage_degraded_events != epoch_at_entry {
            heritage_degraded = true;
        }

        let result = match (
            shell.as_ref(),
            has_type_args,
            crate::checker::utilities::is_type_error(&result),
        ) {
            (Some(shell), false, false) => self.fill_interface_shell(shell, result),
            _ => result,
        };

        let mut degraded_accepted = false;
        if heritage_degraded {
            let sym_key = Arc::as_ptr(symbol) as *const tsox_frontend::ast::Symbol as usize;
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

    fn fill_interface_shell(
        &mut self,
        shell: &Arc<Type>,
        result: Arc<Type>,
    ) -> Arc<Type> {
        let shell_empty = shell
            .as_structured()
            .is_some_and(|s| s.members.entries.is_empty() && s.index_infos.is_empty());
        if !shell_empty {
            return result;
        }
        let sptr = Arc::as_ptr(shell) as *mut crate::checker::types::Type;
        let rptr = Arc::as_ptr(&result) as *mut crate::checker::types::Type;
        unsafe {
            (*sptr).object_flags |= (*rptr).object_flags;
            std::mem::swap(&mut (*sptr).data, &mut (*rptr).data);
        }
        Arc::clone(shell)
    }
}
