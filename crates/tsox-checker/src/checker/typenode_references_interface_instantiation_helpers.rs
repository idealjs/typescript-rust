#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    pub(crate) fn push_interface_type_argument_mapping(
        &mut self,
        interface_decls: &[Arc<Node>],
        tp_symbols: &[Arc<Symbol>],
        arg_types: &[Arc<Type>],
    ) {
        let mut mapping = HashMap::new();
        for (i, tp_sym) in tp_symbols.iter().enumerate() {
            if let Some(arg) = arg_types.get(i) {
                let k = Arc::as_ptr(tp_sym) as *const tsox_frontend::ast::Symbol;
                mapping.insert(k, Arc::clone(arg));
            }
        }
        for decl in interface_decls {
            let NodeData::InterfaceDeclaration(d) = &decl.data else {
                continue;
            };
            let Some(tps) = &d.type_parameters else {
                continue;
            };
            let sym_map = self.program.symbol_map();
            for (i, tp) in tps.iter().enumerate() {
                let Some(tp_sym) = sym_map.symbol_of(tp) else {
                    continue;
                };

                let idx = if let Some(first_sym) = tp_symbols.get(i) {
                    if first_sym.name == tp_sym.name {
                        i
                    } else {
                        tp_symbols
                            .iter()
                            .position(|s| s.name == tp_sym.name)
                            .unwrap_or(i)
                    }
                } else {
                    i
                };
                if let Some(arg) = arg_types.get(idx) {
                    let k = Arc::as_ptr(tp_sym) as *const tsox_frontend::ast::Symbol;
                    mapping.insert(k, Arc::clone(arg));
                }
            }
        }
        self.type_argument_stack.push(mapping);
    }

    /// 实例化语境（接口带实参解析成员期，type_argument_stack 非空）中，
    /// 声明的类型参数集合生成新实例：约束经容器实参代入（Go instantiateSymbol
    /// 的约束代入），符号不变（推断按符号等价路由）
    pub(crate) fn instantiate_declaration_type_parameters(
        &mut self,
        tps: Vec<Arc<Type>>,
    ) -> Vec<Arc<Type>> {
        if self.type_argument_stack.is_empty() {
            return tps;
        }
        let mut frame_pairs: Vec<(*const Symbol, Arc<Type>)> = Vec::new();
        for frame in self.type_argument_stack.iter().rev() {
            for (k, v) in frame.iter() {
                frame_pairs.push((*k, Arc::clone(v)));
            }
        }
        tps.into_iter()
            .map(|tp| {
                let constraint = match &tp.data {
                    TypeData::TypeParameter(d) => d.constraint.clone(),
                    _ => None,
                };
                let Some(constraint) = constraint else {
                    return tp;
                };
                let mut free: Vec<Arc<Type>> = Vec::new();
                self.collect_free_type_parameters_deep(&constraint, &mut free);
                let mut mapping_params: Vec<Arc<Type>> = Vec::new();
                let mut mapping_args: Vec<Arc<Type>> = Vec::new();
                for f in &free {
                    if let Some(sym) = &f.symbol {
                        let key = Arc::as_ptr(sym) as *const Symbol;
                        if let Some(v) = frame_pairs
                            .iter()
                            .find(|(k, _)| *k == key)
                            .map(|(_, v)| Arc::clone(v))
                        {
                            mapping_params.push(Arc::clone(f));
                            mapping_args.push(v);
                        }
                    }
                }
                if mapping_params.is_empty() {
                    return tp;
                }
                let substituted = self.substitute_infer_type_parameters(
                    &constraint,
                    &mapping_params,
                    &mapping_args,
                );
                if Arc::ptr_eq(&substituted, &constraint) {
                    return tp;
                }
                Arc::new(Type {
                    flags: tp.flags,
                    object_flags: tp.object_flags,
                    id: crate::checker::types::next_type_id(),
                    symbol: tp.symbol.clone(),
                    alias: None,
                    data: TypeData::TypeParameter(TypeParameterData {
                        constrained: ConstrainedTypeData::default(),
                        constraint: Some(substituted),
                        target: None,
                        mapper: None,
                        is_this_type: false,
                        resolved_default_type: OnceLock::new(),
                    }),
                })
            })
            .collect()
    }

    pub(crate) fn collect_interface_base_types(
        &mut self,
        interface_decls: &[Arc<Node>],
        heritage_degraded: &mut bool,
    ) -> Vec<(Arc<Node>, Arc<Type>)> {
        let mut base_types: Vec<(Arc<Node>, Arc<Type>)> = Vec::new();
        for decl in interface_decls {
            if let NodeData::InterfaceDeclaration(d) = &decl.data {
                if let Some(heritage) = &d.heritage_clauses {
                    for clause in heritage.iter() {
                        if let NodeData::HeritageClause(hc) = &clause.data
                            && hc.token == SyntaxKind::ExtendsKeyword
                        {
                            for type_ref in hc.types.iter() {
                                let bt = self.get_type_from_type_node(type_ref);

                                if crate::checker::utilities::is_type_error(&bt)
                                    && !*heritage_degraded
                                {
                                    *heritage_degraded = true;
                                    self.heritage_degraded_events += 1;
                                }
                                base_types.push((Arc::clone(type_ref), bt));
                            }
                        }
                    }
                }
            }
        }
        base_types
    }
}
