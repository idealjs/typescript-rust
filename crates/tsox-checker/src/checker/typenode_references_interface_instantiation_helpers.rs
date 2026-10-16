#![allow(unused_imports)]

use crate::checker::mig::m2a::r20k6_defs::R20K6CheckerExt;
use crate::checker::typenode_references::*;
use std::collections::HashSet;

impl Checker {
    pub(crate) fn push_interface_type_argument_mapping(
        &mut self,
        interface_decls: &[Arc<Node>],
        tp_symbols: &[Arc<Symbol>],
        arg_types: &[Arc<Type>],
    ) { ::tsox_core::fntrace::enter("push_interface_type_argument_mapping"); 
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
    /// 的约束代入），符号不变（推断按符号等价路由）；被替换型参的兄弟约束
    /// 引用随替换改指新实例——Go 实例化签名约束沿 target+mapper 惰性解析
    /// （getConstraintFromTypeParameter checker.go:18078-18080），V extends U[]
    /// 代入后内层 U 与签名声明位同对象，显示按名不落阴影
    /// （typeParameterConstraintInstantiation 的 >m.map : <U extends string,
    /// V extends U[]>）
    pub(crate) fn instantiate_declaration_type_parameters(
        &mut self,
        tps: Vec<Arc<Type>>,
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("instantiate_declaration_type_parameters"); 
        if self.type_argument_stack.is_empty() {
            return tps;
        }
        let mut frame_pairs: Vec<(*const Symbol, Arc<Type>)> = Vec::new();
        for frame in self.type_argument_stack.iter().rev() {
            for (k, v) in frame.iter() {
                frame_pairs.push((*k, Arc::clone(v)));
            }
        }
        let mut replaced: Vec<(*const Symbol, Arc<Type>)> = Vec::new();
        let mut result: Vec<Arc<Type>> = Vec::with_capacity(tps.len());
        let tp_count = tps.len();
        for tp in tps {
            let constraint = match &tp.data {
                TypeData::TypeParameter(d) => d.constraint.clone(),
                _ => None,
            };
            let new_tp = match constraint {
                None => tp,
                Some(constraint) => {
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
                        tp
                    } else {
                        let substituted = self.substitute_infer_type_parameters(
                            &constraint,
                            &mapping_params,
                            &mapping_args,
                        );
                        if Arc::ptr_eq(&substituted, &constraint) {
                            tp
                        } else {
                            let new_tp = Arc::new(Type {
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
                                    resolved_default_type: std::sync::RwLock::new(None),
                                }),
                            });
                            if let Some(sym) = &tp.symbol {
                                replaced.push((Arc::as_ptr(sym) as *const Symbol, Arc::clone(&new_tp)));
                            }
                            new_tp
                        }
                    }
                }
            };
            result.push(new_tp);
        }
        if replaced.is_empty() {
            return result;
        }
        for _ in 0..=tp_count {
            let mut changed = false;
            let mut next: Vec<Arc<Type>> = Vec::with_capacity(result.len());
            for tp in result {
                let constraint = match &tp.data {
                    TypeData::TypeParameter(d) => d.constraint.clone(),
                    _ => None,
                };
                let Some(constraint) = constraint else {
                    next.push(tp);
                    continue;
                };
                let mut free: Vec<Arc<Type>> = Vec::new();
                self.collect_free_type_parameters_deep(&constraint, &mut free);
                let mut mapping_params: Vec<Arc<Type>> = Vec::new();
                let mut mapping_args: Vec<Arc<Type>> = Vec::new();
                for f in &free {
                    if let Some(sym) = &f.symbol {
                        let key = Arc::as_ptr(sym) as *const Symbol;
                        if let Some(v) = replaced
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
                    next.push(tp);
                    continue;
                }
                let substituted =
                    self.substitute_infer_type_parameters(&constraint, &mapping_params, &mapping_args);
                if Arc::ptr_eq(&substituted, &constraint) {
                    next.push(tp);
                    continue;
                }
                let new_tp = Arc::new(Type {
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
                        resolved_default_type: std::sync::RwLock::new(None),
                    }),
                });
                if let Some(sym) = &tp.symbol {
                    replaced.push((Arc::as_ptr(sym) as *const Symbol, Arc::clone(&new_tp)));
                }
                changed = true;
                next.push(new_tp);
            }
            result = next;
            if !changed {
                break;
            }
        }
        result
    }

    pub(crate) fn collect_interface_base_types(
        &mut self,
        symbol: &Arc<Symbol>,
        interface_decls: &[Arc<Node>],
        heritage_degraded: &mut bool,
        circular_decls: &mut Vec<Arc<Node>>,
    ) -> Vec<(Arc<Node>, Arc<Type>)> { ::tsox_core::fntrace::enter("collect_interface_base_types");
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
                                if self.interface_base_type_is_circular(symbol, &bt) {
                                    circular_decls.push(Arc::clone(decl));
                                    continue;
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

    pub(crate) fn interface_base_type_is_circular(
        &mut self,
        symbol: &Arc<Symbol>,
        base: &Arc<Type>,
    ) -> bool { ::tsox_core::fntrace::enter("interface_base_type_is_circular");
        let Some(base_symbol) = base.symbol.as_ref() else {
            return false;
        };
        if Arc::ptr_eq(base_symbol, symbol) {
            return true;
        }
        let mut visited: HashSet<usize> = HashSet::new();
        self.interface_heritage_chain_reaches(base_symbol, symbol, &mut visited)
    }

    pub(crate) fn interface_heritage_chain_reaches(
        &mut self,
        from: &Arc<Symbol>,
        target: &Arc<Symbol>,
        visited: &mut HashSet<usize>,
    ) -> bool { ::tsox_core::fntrace::enter("interface_heritage_chain_reaches");
        let key = Arc::as_ptr(from) as *const tsox_frontend::ast::Symbol as usize;
        if !visited.insert(key) {
            return false;
        }
        for decl in from.declarations.iter() {
            for element in tsox_frontend::ast::get_extends_heritage_clause_elements(decl) {
                let Some(next) =
                    self.resolve_type_reference_name(&element, tsox_frontend::ast::SymbolFlags::TYPE, true)
                else {
                    continue;
                };
                if Arc::ptr_eq(&next, &self.unknown_symbol()) {
                    continue;
                }
                if Arc::ptr_eq(&next, target)
                    || self.interface_heritage_chain_reaches(&next, target, visited)
                {
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn instantiate_interface_member_this_to_instance(
        &mut self,
        instance: &Arc<Type>,
        iface_symbol: &Arc<Symbol>,
    ) {
        let Some(structured) = instance.as_structured() else {
            return;
        };
        let mut member_syms: Vec<Arc<Symbol>> = Vec::new();
        for sym in structured
            .members
            .entries
            .values()
            .chain(structured.properties.iter())
        {
            if !member_syms.iter().any(|s| Arc::ptr_eq(s, sym)) {
                member_syms.push(Arc::clone(sym));
            }
        }
        if member_syms.is_empty() {
            return;
        }

        let mut singles: Vec<Arc<Type>> = Vec::new();
        for sym in &member_syms {
            let member_types: Vec<Arc<Type>> = self
                .value_symbol_links
                .get(sym)
                .into_iter()
                .flat_map(|l| [l.resolved_type.clone(), l.write_type.clone()])
                .flatten()
                .collect();
            for t in member_types {
                let mut visited: Vec<crate::checker::types_type_id::TypeId> = Vec::new();
                self.collect_this_singles_of_iface(&t, iface_symbol, &mut singles, &mut visited);
            }
        }
        if singles.is_empty() {
            return;
        }

        let targets: Vec<Arc<Type>> = singles.iter().map(|_| Arc::clone(instance)).collect();
        let mapper = Arc::new(crate::checker::mapper::new_array_type_mapper(singles, targets));
        for sym in &member_syms {
            let is_own_member = sym
                .parent()
                .is_some_and(|p| self.merged_symbol_eq(&p, iface_symbol));
            if !is_own_member {
                continue;
            }
            let existing = self
                .value_symbol_links
                .get(sym)
                .and_then(|l| l.resolved_type.clone());
            if let Some(t) = existing
                && let Some(new_t) = self.rewrite_member_type_this(&t, &mapper)
            {
                self.value_symbol_links
                    .get_or_default(sym)
                    .resolved_type = Some(new_t);
            }
            let existing_write = self
                .value_symbol_links
                .get(sym)
                .and_then(|l| l.write_type.clone());
            if let Some(wt) = existing_write
                && let Some(new_wt) = self.rewrite_member_type_this(&wt, &mapper)
            {
                self.value_symbol_links
                    .get_or_default(sym)
                    .write_type = Some(new_wt);
            }
        }
    }

    fn collect_this_singles_of_iface(
        &self,
        t: &Arc<Type>,
        iface_symbol: &Arc<Symbol>,
        singles: &mut Vec<Arc<Type>>,
        visited: &mut Vec<crate::checker::types_type_id::TypeId>,
    ) {
        if !visited.contains(&t.id) {
            visited.push(t.id);
        } else {
            return;
        }
        if t.flags.intersects(TypeFlags::TypeParameter) {
            let is_container_this = match &t.data {
                TypeData::TypeParameter(tp) => {
                    tp.is_this_type
                        && t.symbol
                            .as_ref()
                            .is_some_and(|s| self.merged_symbol_eq(s, iface_symbol))
                }
                _ => false,
            };
            if is_container_this && !singles.iter().any(|s| Arc::ptr_eq(s, t)) {
                singles.push(Arc::clone(t));
            }
            return;
        }
        if t.flags.intersects(TypeFlags::Object) {
            if !t.object_flags.intersects(ObjectFlags::Anonymous) {
                return;
            }
            if let Some(structured) = t.as_structured() {
                for sig in structured.signatures.iter() {
                    if let Some(rt) = sig.resolved_return_type.get() {
                        self.collect_this_singles_of_iface(rt, iface_symbol, singles, visited);
                    }
                }
            }
            return;
        }
        match &t.data {
            TypeData::Union(u) => {
                for m in u.union_or_intersection.types.iter() {
                    self.collect_this_singles_of_iface(m, iface_symbol, singles, visited);
                }
            }
            TypeData::Intersection(i) => {
                for m in i.union_or_intersection.types.iter() {
                    self.collect_this_singles_of_iface(m, iface_symbol, singles, visited);
                }
            }
            TypeData::Tuple(tuple) => {
                for info in tuple.element_infos.iter() {
                    if let Some(et) = &info.type_ {
                        self.collect_this_singles_of_iface(et, iface_symbol, singles, visited);
                    }
                }
            }
            _ => {}
        }
    }

    fn merged_symbol_eq(&self, a: &Arc<Symbol>, b: &Arc<Symbol>) -> bool {
        if Arc::ptr_eq(a, b) {
            return true;
        }
        Arc::ptr_eq(&self.get_merged_symbol(a), &self.get_merged_symbol(b))
    }

    fn rewrite_member_type_this(
        &mut self,
        t: &Arc<Type>,
        mapper: &Arc<crate::checker::TypeMapper>,
    ) -> Option<Arc<Type>> {
        if t.flags.intersects(TypeFlags::Object) {
            if !t.object_flags.intersects(ObjectFlags::Anonymous) {
                return None;
            }
            let structured = t.as_structured()?;
            if structured.signatures.is_empty() {
                return None;
            }
            let is_construct = structured.call_signature_count == 0;
            let mut changed = false;
            let mut new_sigs: Vec<Arc<Signature>> =
                Vec::with_capacity(structured.signatures.len());
            for sig in structured.signatures.iter() {
                let Some(rt) = sig.resolved_return_type.get() else {
                    new_sigs.push(Arc::clone(sig));
                    continue;
                };
                let new_rt = self.instantiate_type(rt, Some(mapper));
                if Arc::ptr_eq(&new_rt, rt) {
                    new_sigs.push(Arc::clone(sig));
                } else {
                    changed = true;
                    new_sigs.push(self.clone_signature_with_return_type(sig, &new_rt));
                }
            }
            if !changed {
                return None;
            }
            return Some(self.create_function_or_constructor_type_ex(
                new_sigs,
                is_construct,
                t.symbol.clone(),
            ));
        }
        if t.flags.intersects(
            TypeFlags::TypeParameter
                | TypeFlags::Union
                | TypeFlags::Intersection
                | TypeFlags::Index
                | TypeFlags::IndexedAccess
                | TypeFlags::TemplateLiteral
                | TypeFlags::StringMapping
                | TypeFlags::Conditional
                | TypeFlags::Substitution,
        ) {
            let new_t = self.instantiate_type(t, Some(mapper));
            if Arc::ptr_eq(&new_t, t) {
                return None;
            }
            return Some(new_t);
        }
        None
    }

    fn clone_signature_with_return_type(
        &self,
        sig: &Arc<Signature>,
        rt: &Arc<Type>,
    ) -> Arc<Signature> {
        let mut s = Signature {
            id: sig.id,
            flags: sig.flags,
            min_argument_count: sig.min_argument_count,
            resolved_min_argument_count: sig.resolved_min_argument_count,
            declaration: sig.declaration.clone(),
            type_parameters: sig.type_parameters.clone(),
            parameters: sig.parameters.clone(),
            this_parameter: sig.this_parameter.clone(),
            resolved_return_type: std::sync::OnceLock::from(Arc::clone(rt)),
            resolved_type_predicate: sig.resolved_type_predicate.clone(),
            target: sig.target.clone(),
            mapper: sig.mapper.clone(),
            isolated_signature_type: std::sync::OnceLock::new(),
            instantiated_parameter_types: sig.instantiated_parameter_types.clone(),
        };
        if let Some(it) = sig.isolated_signature_type.get() {
            let _ = s.isolated_signature_type.set(Arc::clone(it));
        }
        Arc::new(s)
    }
}
