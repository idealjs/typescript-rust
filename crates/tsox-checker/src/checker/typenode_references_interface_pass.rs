#![allow(unused_imports)]

use crate::checker::typenode_references::*;

pub(crate) struct InterfacePassOutcome {
    pub result: Arc<crate::checker::types::Type>,
    pub base_degraded: bool,
    pub base_shell: bool,
    pub live_shell_base: bool,
    pub lineage_degraded: bool,
}

impl Checker {
    pub(crate) fn resolve_interface_pass(
        &mut self,
        symbol: &Arc<Symbol>,
        interface_decls: &[Arc<Node>],
        has_type_args: bool,
        arg_types: Vec<Arc<crate::checker::types::Type>>,
    ) -> InterfacePassOutcome { ::tsox_core::fntrace::enter("resolve_interface_pass"); 
        match interface_decls.first() {
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
                let own_result =
                    self.build_interface_type_from_members_with_symbol(&merged_list, None, Some(symbol));
                self.in_static_member_type = saved_static;
                if has_type_args {
                    // 增强声明的同名 T 是独立符号：实例化标记须覆盖全部声明，
                    // 否则增强成员（如 es2015 Array.find）的类型参数悬空
                    let sym_map = self.program.symbol_map();
                    let mut all_tp_symbols: Vec<Arc<Symbol>> = tp_symbols.clone();
                    let mut all_args: Vec<Arc<crate::checker::types::Type>> = arg_types.clone();
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
                    let all_tp_types: Vec<Arc<crate::checker::types::Type>> = all_tp_symbols
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
                let mut circular_base_decls: Vec<Arc<Node>> = Vec::new();
                let base_types = self.collect_interface_base_types(
                    symbol,
                    &interface_decls,
                    &mut heritage_base_degraded,
                    &mut circular_base_decls,
                );
                // 基类是空壳（解析重入期产物）时合并结果只有自有成员：按 degraded
                // 处理——结果不进缓存、标 degraded，后续引用重建拿完整版
                let mut base_shell = false;
                let mut live_shell_base = false;
                for (_, bt) in &base_types {
                    let Some(st) = bt.as_structured() else {
                        continue;
                    };
                    if !st.members.entries.is_empty() {
                        continue;
                    }
                    base_shell = true;
                    if let Some(bsym) = bt.symbol.as_ref()
                        && self.pending_interface_shells.contains_key(
                            &(Arc::as_ptr(bsym) as *const tsox_frontend::ast::Symbol as usize),
                        )
                    {
                        live_shell_base = true;
                    }
                }
                let lineage_degraded = base_types
                    .iter()
                    .any(|(_, bt)| self.degraded_type_ptrs.contains(&bt.id));
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
                    let result_mut =
                        Arc::as_ptr(&result) as *mut crate::checker::types::Type;
                    unsafe {
                        (*result_mut).symbol = Some(Arc::clone(symbol));
                        if has_type_args && let TypeData::Object(o) = &mut (*result_mut).data {
                            o.type_arguments = arg_types.clone();
                        }
                    }
                }
                for declaration in &circular_base_decls {
                    let error_node =
                        crate::checker::mig::m1d_2::get_adjusted_node_for_error(declaration);
                    self.report_circular_base_type(&error_node, &result);
                }
                InterfacePassOutcome {
                    result,
                    base_degraded: heritage_base_degraded,
                    base_shell,
                    live_shell_base,
                    lineage_degraded,
                }
            }
            None => InterfacePassOutcome {
                result: self.error_type(),
                base_degraded: false,
                base_shell: false,
                live_shell_base: false,
                lineage_degraded: false,
            },
        }
    }
}
