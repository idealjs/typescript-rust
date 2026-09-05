#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::ast::node_data_generated::NodeData;
use crate::ast::Symbol;

use crate::checker::checker::Checker;
use crate::checker::inference::{InferenceContext, InferenceInfo, InferencePriority};

use super::*;

impl Checker {

    pub fn resolve_conditional_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        let ct = match &t.data {
            TypeData::Conditional(ct) => ct,
            _ => return None,
        };

        if let Some(rt) = ct.resolved_true_type.get() {
            return Some(Arc::clone(rt));
        }
        if let Some(rt) = ct.resolved_false_type.get() {
            return Some(Arc::clone(rt));
        }

        let check_type = ct.check_type.clone()?;
        let _extends_type = ct.extends_type.clone()?;

        if check_type.flags.contains(TypeFlags::Any) && check_type.intrinsic_name() == Some("error")
        {
            return Some(Arc::clone(&check_type));
        }

        let (is_distributive, check_tp_symbol) = match ct.root.as_ref() {
            Some(root) => (
                root.is_distributive,
                root.check_type_parameter_symbol.clone(),
            ),
            None => (false, None),
        };
        if is_distributive && let Some(tp_symbol) = &check_tp_symbol {
            if check_type.flags.contains(TypeFlags::Never) {

                let never = self.never_type();
                if let TypeData::Conditional(ct2) = &t.data {
                    let _ = ct2.resolved_true_type.set(Arc::clone(&never));
                }
                return Some(never);
            }
            if let TypeData::Union(u) = &check_type.data {
                let constituents = u.union_or_intersection.types.clone();
                if std::env::var_os("TSOX_DEBUG_COND").is_some() {
                    eprintln!(
                        "[cond] distributing over {} constituent(s); tp={}",
                        constituents.len(),
                        tp_symbol.name
                    );
                }
                let key = Arc::as_ptr(tp_symbol) as *const crate::ast::Symbol;
                let mut results: Vec<Arc<Type>> = Vec::with_capacity(constituents.len());
                for constituent in constituents {
                    let mut mapping = std::collections::HashMap::new();
                    mapping.insert(key, Arc::clone(&constituent));
                    self.type_argument_stack.push(mapping);
                    let r =
                        self.resolve_conditional_type_with_check(t, Some(Arc::clone(&constituent)));
                    self.type_argument_stack.pop();
                    if std::env::var_os("TSOX_DEBUG_COND").is_some() {
                        eprintln!(
                            "[cond]   constituent {} -> {:?}",
                            self.type_to_string(&constituent),
                            r.as_ref().map(|x| self.type_to_string(x))
                        );
                    }
                    results.push(r?);
                }
                let union = self.get_union_type(results);
                if std::env::var_os("TSOX_DEBUG_COND").is_some() {
                    eprintln!("[cond] result union = {}", self.type_to_string(&union));
                }
                return Some(union);
            }
        }

        self.resolve_conditional_type_with_check(t, None)
    }

    pub(crate) fn resolve_conditional_type_with_check(
        &mut self,
        t: &Arc<Type>,
        check_override: Option<Arc<Type>>,
    ) -> Option<Arc<Type>> {
        let ct = match &t.data {
            TypeData::Conditional(ct) => ct,
            _ => return None,
        };

        let check_type = match check_override {
            Some(ref c) => Arc::clone(c),
            None => ct.check_type.clone()?,
        };
        let cond_node = ct.root.as_ref().and_then(|r| r.node.clone());
        let extends_type = if check_override.is_some() {

            let extends_node = match cond_node.as_ref().and_then(|n| match &n.data {
                NodeData::ConditionalTypeNode(data) => Some(Arc::clone(&data.extends_type)),
                _ => None,
            }) {
                Some(node) => node,
                None => return None,
            };
            self.get_type_from_type_node(&extends_node)
        } else {
            ct.extends_type.clone()?
        };

        if type_contains_type_parameter(&check_type) {
            return None;
        }

        let infer_params: Vec<Arc<Type>> = ct
            .root
            .as_ref()
            .map(|r| r.infer_type_parameters.clone())
            .unwrap_or_default();
        let inferences: Vec<InferenceInfo> = infer_params
            .iter()
            .map(|p| InferenceInfo::new(Arc::clone(p)))
            .collect();
        let mut context = InferenceContext::new(inferences);

        if !infer_params.is_empty() {

            self.infer_types(
                &mut context.inferences,
                Some(Arc::clone(&check_type)),
                Some(Arc::clone(&extends_type)),
                InferencePriority::NoConstraints | InferencePriority::AlwaysStrict,
                false,
            );

            let inferred = self.get_inferred_types(&context);
            for inf in &inferred {
                if inf.flags.contains(TypeFlags::Any) && inf.intrinsic_name() == Some("error") {
                    return None;
                }
            }
        }

        let inferred_extends = if !infer_params.is_empty() {
            let inferred = self.get_inferred_types(&context);
            self.substitute_infer_type_parameters(&extends_type, &infer_params, &inferred)
        } else {
            Arc::clone(&extends_type)
        };
        let extends_any_or_unknown = inferred_extends
            .flags
            .intersects(TypeFlags::Any | TypeFlags::Unknown);
        let check_is_any = check_type.flags.contains(TypeFlags::Any);
        let definitely_false = if extends_any_or_unknown {
            false
        } else if check_is_any {
            true
        } else {
            let permissive_check = self.get_permissive_instantiation(&check_type);
            let permissive_extends = self.get_permissive_instantiation(&inferred_extends);
            !self.is_type_assignable_to(&permissive_check, &permissive_extends)
        };
        let take_true = if !definitely_false {
            let definitely_true = if extends_any_or_unknown {
                true
            } else {
                let restrictive_check = self.get_restrictive_instantiation(&check_type);
                let restrictive_extends = self.get_restrictive_instantiation(&inferred_extends);
                self.is_type_assignable_to(&restrictive_check, &restrictive_extends)
            };
            if !definitely_true {
                if std::env::var_os("TSOX_DEBUG_COND").is_some() {
                    eprintln!(
                        "[cond]     deferred (neither definite) check={} extends={}",
                        self.type_to_string(&check_type),
                        self.type_to_string(&inferred_extends)
                    );
                }
                return None;
            }
            true
        } else {
            false
        };

        let include_true_branch = take_true == false && check_is_any;
        if std::env::var_os("TSOX_DEBUG_COND").is_some() {
            eprintln!(
                "[cond]     take_true={} check={} extends={}",
                take_true,
                self.type_to_string(&check_type),
                self.type_to_string(&inferred_extends)
            );
        }

        let (cond_node, branch_node) = match ct
            .root
            .as_ref()
            .and_then(|r| r.node.as_ref())
            .and_then(|n| match &n.data {
                NodeData::ConditionalTypeNode(data) => {
                    let branch = if take_true {
                        Arc::clone(&data.true_type)
                    } else {
                        Arc::clone(&data.false_type)
                    };
                    Some((Arc::clone(n), branch))
                }
                _ => None,
            }) {
            Some(pair) => pair,
            None => return None,
        };

        if take_true {
            self.push_scope(&cond_node);
        }

        let creation_scopes: Vec<u64> = ct
            .root
            .as_ref()
            .map(|r| r.creation_scopes.clone())
            .unwrap_or_default();
        let mut common = 0usize;
        while common < creation_scopes.len()
            && common < self.scope_stack.len()
            && creation_scopes[common] == self.scope_stack[common]
        {
            common += 1;
        }
        let scopes_pushed = creation_scopes.len() - common;
        self.scope_stack.extend_from_slice(&creation_scopes[common..]);

        let mut merged_creation: HashMap<usize, Arc<Type>> = HashMap::new();
        for frame in ct.creation_type_argument_stack.iter() {
            for (k, v) in frame {
                merged_creation.insert(*k, Arc::clone(v));
            }
        }
        for map in self.type_argument_stack.iter() {
            for k in map.keys() {
                merged_creation.remove(&(*k as usize));
            }
        }
        let pushes_creation = !merged_creation.is_empty();
        if pushes_creation {
            self.type_argument_stack
                .push(merged_creation.into_iter().map(|(k, v)| ((k as *const Symbol), v)).collect());
        }
        let branch = self.get_type_from_type_node(&branch_node);
        if pushes_creation {
            self.type_argument_stack.pop();
        }
        if scopes_pushed > 0 {
            self.scope_stack.truncate(self.scope_stack.len() - scopes_pushed);
        }
        if take_true {
            self.pop_scope();
        }
        let resolved = if !infer_params.is_empty() {
            let inferred = self.get_inferred_types(&context);
            self.substitute_infer_type_parameters(&branch, &infer_params, &inferred)
        } else {
            Arc::clone(&branch)
        };

        let resolved = if include_true_branch
            && let Some(true_branch) = self.get_forced_branch_type_of_conditional_type(t, true)
        {
            let true_branch = if !infer_params.is_empty() {
                let inferred = self.get_inferred_types(&context);
                self.substitute_infer_type_parameters(&true_branch, &infer_params, &inferred)
            } else {
                true_branch
            };
            self.get_union_type(vec![true_branch, Arc::clone(&resolved)])
        } else {
            resolved
        };

        if let TypeData::Conditional(ct2) = &t.data {
            let cell = if take_true {
                &ct2.resolved_true_type
            } else {
                &ct2.resolved_false_type
            };
            let _ = cell.set(Arc::clone(&resolved));
        }
        Some(resolved)
    }

    pub fn get_permissive_instantiation(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let key = t.id;
        if let Some(cached) = self.probe_cache_permissive.get(&key) {
            return Arc::clone(cached);
        }
        let result = self.instantiate_probing(t, ProbeMode::Permissive);
        self.probe_cache_permissive.insert(key, Arc::clone(&result));
        result
    }

    pub fn get_restrictive_instantiation(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let key = t.id;
        if let Some(cached) = self.probe_cache_restrictive.get(&key) {
            return Arc::clone(cached);
        }
        let result = self.instantiate_probing(t, ProbeMode::Restrictive);
        self.probe_cache_restrictive.insert(key, Arc::clone(&result));
        result
    }

    fn instantiate_probing(&mut self, t: &Arc<Type>, mode: ProbeMode) -> Arc<Type> {
        match &t.data {
            TypeData::TypeParameter(_) => match mode {
                ProbeMode::Permissive => self.any_function_type(),
                ProbeMode::Restrictive => {
                    let tp = match &t.data {
                        TypeData::TypeParameter(tp) => tp,
                        _ => unreachable!(),
                    };
                    if tp.constraint.is_none() {
                        return Arc::clone(t);
                    }
                    let mut rebuilt = Type::new(
                        t.flags,
                        TypeData::TypeParameter(TypeParameterData {
                            constrained: ConstrainedTypeData::default(),
                            constraint: None,
                            target: tp.target.clone(),
                            mapper: tp.mapper.clone(),
                            is_this_type: tp.is_this_type,
                            resolved_default_type: OnceLock::new(),
                        }),
                    );
                    rebuilt.symbol = t.symbol.clone();
                    rebuilt.object_flags = t.object_flags;
                    Arc::new(rebuilt)
                }
            },
            TypeData::Union(u) => {
                let types = u.union_or_intersection.types.clone();
                let new_types: Vec<Arc<Type>> = types
                    .iter()
                    .map(|c| self.instantiate_probing(c, mode))
                    .collect();
                if new_types.iter().zip(types.iter()).all(|(n, o)| Arc::ptr_eq(n, o)) {
                    return Arc::clone(t);
                }
                self.get_union_type(new_types)
            }
            TypeData::Intersection(i) => {
                let types = i.union_or_intersection.types.clone();
                let new_types: Vec<Arc<Type>> = types
                    .iter()
                    .map(|c| self.instantiate_probing(c, mode))
                    .collect();
                if new_types.iter().zip(types.iter()).all(|(n, o)| Arc::ptr_eq(n, o)) {
                    return Arc::clone(t);
                }
                self.get_intersection_type(new_types)
            }
            TypeData::Object(o) => {

                if o.type_arguments.is_empty() {
                    return Arc::clone(t);
                }
                let new_args: Vec<Arc<Type>> = o
                    .type_arguments
                    .iter()
                    .map(|a| self.instantiate_probing(a, mode))
                    .collect();
                if new_args
                    .iter()
                    .zip(o.type_arguments.iter())
                    .all(|(n, old)| Arc::ptr_eq(n, old))
                {
                    return Arc::clone(t);
                }
                if o.target.is_none() && o.type_arguments.len() == 1 && self.is_array_type(t) {
                    return self.create_array_type(Arc::clone(&new_args[0]));
                }
                let mut rebuilt = Type::new(
                    t.flags,
                    TypeData::Object(ObjectTypeData {
                        structured: StructuredTypeData::default(),
                        target: o.target.clone(),
                        mapper: None,
                        type_arguments: new_args,
                    }),
                );
                rebuilt.object_flags = t.object_flags;
                rebuilt.symbol = t.symbol.clone();
                Arc::new(rebuilt)
            }
            TypeData::Tuple(tup) => {
                let args: Vec<Arc<Type>> = tup
                    .element_infos
                    .iter()
                    .filter_map(|ei| ei.type_.clone())
                    .collect();
                if args.is_empty() {
                    return Arc::clone(t);
                }
                let new_elems: Vec<Arc<Type>> =
                    args.iter().map(|e| self.instantiate_probing(e, mode)).collect();
                if new_elems.iter().zip(args.iter()).all(|(n, o)| Arc::ptr_eq(n, o)) {
                    return Arc::clone(t);
                }
                self.create_tuple_type(new_elems)
            }
            TypeData::Conditional(ct) => {

                let (old_check, old_extends) =
                    match (ct.check_type.as_ref(), ct.extends_type.as_ref()) {
                        (Some(c), Some(e)) => (Arc::clone(c), Arc::clone(e)),
                        _ => return Arc::clone(t),
                    };
                let new_check = self.instantiate_probing(&old_check, mode);
                let new_extends = self.instantiate_probing(&old_extends, mode);
                if Arc::ptr_eq(&new_check, &old_check) && Arc::ptr_eq(&new_extends, &old_extends)
                {
                    return Arc::clone(t);
                }
                let mut rebuilt = Type::new(
                    t.flags,
                    TypeData::Conditional(ConditionalTypeData {
                        constrained: ConstrainedTypeData::default(),
                        root: ct.root.as_ref().map(|r| {
                            Box::new(ConditionalRoot {
                                node: r.node.clone(),
                                check_type: r.check_type.clone(),
                                extends_type: r.extends_type.clone(),
                                is_distributive: r.is_distributive,
                                check_type_parameter_symbol: r
                                    .check_type_parameter_symbol
                                    .clone(),
                                infer_type_parameters: r.infer_type_parameters.clone(),
                                outer_type_parameters: r.outer_type_parameters.clone(),
                                alias: None,
                                creation_scopes: r.creation_scopes.clone(),
                            })
                        }),
                        check_type: Some(new_check),
                        extends_type: Some(new_extends),
                        resolved_true_type: OnceLock::new(),
                        resolved_false_type: OnceLock::new(),
                        resolved_inferred_true_type: OnceLock::new(),
                        resolved_default_constraint: OnceLock::new(),
                        resolved_constraint_of_distributive: OnceLock::new(),
                        mapper: None,
                        combined_mapper: None,
                        creation_type_argument_stack: Vec::new(),
                    }),
                );
                rebuilt.symbol = t.symbol.clone();
                rebuilt.object_flags = t.object_flags;
                Arc::new(rebuilt)
            }
            TypeData::IndexedAccess(ia) => {
                let (Some(old_obj), Some(old_idx)) =
                    (ia.object_type.as_ref(), ia.index_type.as_ref())
                else {
                    return Arc::clone(t);
                };
                let new_obj = self.instantiate_probing(old_obj, mode);
                let new_idx = self.instantiate_probing(old_idx, mode);
                if Arc::ptr_eq(&new_obj, old_obj) && Arc::ptr_eq(&new_idx, old_idx) {
                    return Arc::clone(t);
                }
                let mut rebuilt = Type::new(
                    t.flags,
                    TypeData::IndexedAccess(IndexedAccessTypeData {
                        constrained: ConstrainedTypeData::default(),
                        object_type: Some(new_obj),
                        index_type: Some(new_idx),
                        access_flags: ia.access_flags,
                    }),
                );
                rebuilt.symbol = t.symbol.clone();
                rebuilt.object_flags = t.object_flags;
                Arc::new(rebuilt)
            }
            _ => Arc::clone(t),
        }
    }

    pub(crate) fn type_param_symbols_share_container(&self, a: &Arc<Symbol>, b: &Arc<Symbol>) -> bool {
        let symbol_map = self.program.symbol_map();
        let container_of = |s: &Arc<Symbol>| -> Option<usize> {
            let mut node = s.declarations.first()?.parent.as_ref()?;
            for _ in 0..4 {
                if let Some(sym) = symbol_map.symbols.get(&node.id()) {
                    return Some(Arc::as_ptr(sym) as *const Symbol as usize);
                }
                node = node.parent.as_ref()?;
            }
            None
        };
        match (container_of(a), container_of(b)) {
            (Some(x), Some(y)) => x == y,
            _ => false,
        }
    }

    pub(crate) fn substitute_object_properties_deep(
        &mut self,
        t: &Arc<Type>,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> {
        let key = t.id;
        if let Some(cached) = self.subst_object_in_progress.get(&key) {
            return Arc::clone(cached);
        }
        let Some(o) = t.as_object() else {
            return Arc::clone(t);
        };

        let mut old_types: Vec<Option<Arc<Type>>> = Vec::with_capacity(o.structured.properties.len());
        for prop in &o.structured.properties {
            old_types.push(
                self.value_symbol_links
                    .get(prop)
                    .and_then(|l| l.resolved_type.clone()),
            );
        }

        let shell = Arc::new(Type::new(
            t.flags,
            TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: o.structured.members.clone(),
                    properties: o.structured.properties.clone(),
                    signatures: o.structured.signatures.clone(),
                    call_signature_count: o.structured.call_signature_count,
                    index_infos: o.structured.index_infos.clone(),
                    ..Default::default()
                },
                target: o.target.clone(),
                mapper: o.mapper.clone(),
                type_arguments: o.type_arguments.clone(),
            }),
        ));
        {
            let shell_mut = Arc::as_ptr(&shell) as *mut Type;
            unsafe {
                (*shell_mut).object_flags = t.object_flags;
                (*shell_mut).symbol = t.symbol.clone();

            }
        }
        self.subst_object_in_progress.insert(key, Arc::clone(&shell));
        let mut changed = false;
        let mut new_props: Vec<Arc<Symbol>> = Vec::with_capacity(o.structured.properties.len());
        let mut new_members = o.structured.members.clone();
        for (prop, old_t) in o.structured.properties.iter().zip(old_types.iter()) {
            let Some(old_t) = old_t else {
                new_props.push(Arc::clone(prop));
                continue;
            };
            let new_t = self.substitute_infer_type_parameters(old_t, params, substitutions);
            if Arc::ptr_eq(&new_t, old_t) {
                new_props.push(Arc::clone(prop));
                continue;
            }
            changed = true;
            let mut new_sym = Symbol::new(prop.flags, prop.name.clone());
            new_sym.declarations = prop.declarations.clone();
            new_sym.check_flags = prop.check_flags;
            let new_sym = Arc::new(new_sym);
            self.value_symbol_links.insert(
                &new_sym,
                ValueSymbolLinks {
                    resolved_type: Some(new_t),
                    ..Default::default()
                },
            );
            new_members.insert(prop.name.clone(), Arc::clone(&new_sym));
            new_props.push(new_sym);
        }
        if !changed {
            self.subst_object_in_progress.remove(&key);
            return Arc::clone(t);
        }

        {
            let shell_mut = Arc::as_ptr(&shell) as *mut Type;
            unsafe {
                if let TypeData::Object(so) = &mut (*shell_mut).data {
                    so.structured.members = new_members;
                    so.structured.properties = new_props;
                }
            }
        }
        shell
    }

    pub fn substitute_infer_type_parameters(
        &mut self,
        t: &Arc<Type>,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> {

        if params.is_empty() || substitutions.is_empty() {
            return Arc::clone(t);
        }

        for (i, p) in params.iter().enumerate() {
            if Arc::ptr_eq(p, t)
                || (p.is_type_parameter()
                    && t.is_type_parameter()
                    && (p.symbol.as_ref().zip(t.symbol.as_ref()).is_some_and(
                        |(ps, ts)| {
                            Arc::ptr_eq(ps, ts)
                                || (ps.name == ts.name
                                    && self.type_param_symbols_share_container(ps, ts))
                        },
                    )))
            {
                return Arc::clone(&substitutions[i.min(substitutions.len() - 1)]);
            }
        }

        match &t.data {
            TypeData::Union(u) => {
                let new_types: Vec<Arc<Type>> = u
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|inner| {
                        self.substitute_infer_type_parameters(inner, params, substitutions)
                    })
                    .collect();
                self.get_union_type(new_types)
            }
            TypeData::Intersection(i) => {
                let new_types: Vec<Arc<Type>> = i
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|inner| {
                        self.substitute_infer_type_parameters(inner, params, substitutions)
                    })
                    .collect();
                self.get_intersection_type(new_types)
            }
            TypeData::Object(o) => {

                if t.object_flags.contains(ObjectFlags::Reference)
                    && o.target.is_none()
                    && o.type_arguments.len() == 1
                {
                    let new_elem = self.substitute_infer_type_parameters(
                        &o.type_arguments[0],
                        params,
                        substitutions,
                    );

                    if Arc::ptr_eq(&new_elem, &o.type_arguments[0]) {
                        return Arc::clone(t);
                    }
                    return self.create_array_type(new_elem);
                }

                if !o.type_arguments.is_empty() {
                    let new_args: Vec<Arc<Type>> = o
                        .type_arguments
                        .iter()
                        .map(|arg| {
                            self.substitute_infer_type_parameters(arg, params, substitutions)
                        })
                        .collect();
                    let changed = o
                        .type_arguments
                        .iter()
                        .zip(new_args.iter())
                        .any(|(old, new)| !Arc::ptr_eq(old, new));
                    if changed {

                        let new_index_infos: Vec<Arc<crate::checker::IndexInfo>> = o
                            .structured
                            .index_infos
                            .iter()
                            .map(|info| {
                                let new_value = info.value_type.as_ref().map(|v| {
                                    self.substitute_infer_type_parameters(v, params, substitutions)
                                });
                                if new_value.is_some()
                                    && !new_value.as_ref().is_some_and(|nv| {
                                        info.value_type.as_ref().is_some_and(|ov| Arc::ptr_eq(nv, ov))
                                    })
                                {
                                    Arc::new(crate::checker::IndexInfo {
                                        key_type: info.key_type.clone(),
                                        value_type: new_value,
                                        is_readonly: info.is_readonly,
                                        declaration: info.declaration.clone(),
                                        index_symbol: info.index_symbol.clone(),
                                        components: info.components.clone(),
                                    })
                                } else {
                                    Arc::clone(info)
                                }
                            })
                            .collect();
                        let mut rebuilt = Type::new(
                            t.flags,
                            TypeData::Object(ObjectTypeData {
                                structured: StructuredTypeData {
                                    members: o.structured.members.clone(),
                                    properties: o.structured.properties.clone(),
                                    signatures: o.structured.signatures.clone(),
                                    call_signature_count: o.structured.call_signature_count,
                                    index_infos: new_index_infos,
                                    ..Default::default()
                                },
                                target: o.target.clone(),
                                mapper: o.mapper.clone(),
                                type_arguments: new_args,
                            }),
                        );
                        rebuilt.object_flags = t.object_flags;
                        rebuilt.symbol = t.symbol.clone();
                        return Arc::new(rebuilt);
                    }
                    return Arc::clone(t);
                }

                if t.object_flags.contains(ObjectFlags::Anonymous)
                    && !o.structured.signatures.is_empty()
                {
                    let signatures = o.structured.signatures.clone();
                    let call_signature_count = o.structured.call_signature_count;
                    let mut changed = false;
                    let mut new_sigs: Vec<Arc<Signature>> =
                        Vec::with_capacity(signatures.len());
                    for sig in &signatures {
                        let rest_offset = usize::from(sig.has_rest_parameter());
                        let fixed = sig.parameters.len().saturating_sub(rest_offset);
                        let mut new_params: Vec<Arc<Type>> =
                            Vec::with_capacity(sig.parameters.len());
                        let mut old_params: Vec<Arc<Type>> =
                            Vec::with_capacity(sig.parameters.len());
                        for i in 0..fixed {
                            let pt = self
                                .try_get_type_at_position(sig, i)
                                .unwrap_or_else(|| self.any_type());
                            old_params.push(Arc::clone(&pt));
                            new_params.push(
                                self.substitute_infer_type_parameters(&pt, params, substitutions),
                            );
                        }
                        if rest_offset == 1 {
                            if let Some(last) = sig.parameters.last() {
                                let rt = self.get_type_of_symbol(last);
                                old_params.push(Arc::clone(&rt));
                                new_params.push(
                                    self.substitute_infer_type_parameters(&rt, params, substitutions),
                                );
                            }
                        }
                        let new_return = self
                            .get_return_type_of_signature(sig)
                            .map(|rt| {
                                self.substitute_infer_type_parameters(&rt, params, substitutions)
                            });
                        let params_changed = old_params
                            .iter()
                            .zip(new_params.iter())
                            .any(|(old, new)| !Arc::ptr_eq(old, new));
                        let return_changed = new_return.as_ref().is_some_and(|nr| {
                            self.get_return_type_of_signature(sig)
                                .is_some_and(|old| !Arc::ptr_eq(nr, &old))
                        });
                        if !params_changed && !return_changed {
                            new_sigs.push(Arc::clone(sig));
                            continue;
                        }
                        changed = true;
                        let mut inst = Signature::new();
                        inst.flags = sig.flags;
                        inst.min_argument_count = sig.min_argument_count;
                        inst.resolved_min_argument_count = sig.resolved_min_argument_count;
                        inst.declaration = sig.declaration.clone();
                        inst.target = Some(Arc::clone(sig));
                        inst.parameters = sig.parameters.clone();
                        inst.this_parameter = sig.this_parameter.clone();
                        inst.type_parameters = sig.type_parameters.clone();
                        inst.resolved_type_predicate = sig.resolved_type_predicate.clone();
                        inst.instantiated_parameter_types = Some(new_params);
                        if let Some(nr) = new_return {
                            let _ = inst.resolved_return_type.set(nr);
                        }
                        new_sigs.push(Arc::new(inst));
                    }
                    if !changed {
                        return Arc::clone(t);
                    }
                    let is_construct = call_signature_count == 0;
                    return self.create_function_or_constructor_type(new_sigs, is_construct);
                }

                if self.in_return_substitution
                    && t.symbol.is_none()
                    && !o.structured.properties.is_empty()
                {
                    let fresh = self.subst_object_in_progress.is_empty();
                    let result =
                        self.substitute_object_properties_deep(t, params, substitutions);
                    if fresh {
                        self.subst_object_in_progress.clear();
                    }
                    return result;
                }

                Arc::clone(t)
            }
            TypeData::Tuple(tup) => {

                let new_elems: Vec<Arc<Type>> = tup
                    .element_infos
                    .iter()
                    .map(|ei| match &ei.type_ {
                        Some(ty) => {
                            self.substitute_infer_type_parameters(ty, params, substitutions)
                        }
                        None => self.error_type(),
                    })
                    .collect();

                let changed = tup
                    .element_infos
                    .iter()
                    .zip(new_elems.iter())
                    .any(|(ei, new_t)| match &ei.type_ {
                        Some(old_t) => !Arc::ptr_eq(old_t, new_t),
                        None => true,
                    });
                if !changed {
                    return Arc::clone(t);
                }
                self.create_tuple_type(new_elems)
            }

            TypeData::IndexedAccess(ia) => {
                let new_object = ia
                    .object_type
                    .as_ref()
                    .map(|o| self.substitute_infer_type_parameters(o, params, substitutions));
                let new_index = ia
                    .index_type
                    .as_ref()
                    .map(|idx| self.substitute_infer_type_parameters(idx, params, substitutions));
                let object_changed = new_object
                    .as_ref()
                    .zip(ia.object_type.as_ref())
                    .map(|(new, old)| !Arc::ptr_eq(new, old))
                    .unwrap_or(false);
                let index_changed = new_index
                    .as_ref()
                    .zip(ia.index_type.as_ref())
                    .map(|(new, old)| !Arc::ptr_eq(new, old))
                    .unwrap_or(false);
                if !object_changed && !index_changed {
                    return Arc::clone(t);
                }
                let mut rebuilt = Type::new(
                    t.flags,
                    TypeData::IndexedAccess(IndexedAccessTypeData {
                        constrained: ConstrainedTypeData::default(),
                        object_type: new_object.or_else(|| ia.object_type.clone()),
                        index_type: new_index.or_else(|| ia.index_type.clone()),
                        access_flags: ia.access_flags,
                    }),
                );
                rebuilt.object_flags = t.object_flags;
                rebuilt.symbol = t.symbol.clone();
                Arc::new(rebuilt)
            }

            TypeData::Conditional(ct) => {
                let Some(old_check) = ct.check_type.clone() else {
                    return Arc::clone(t);
                };
                let new_check =
                    self.substitute_infer_type_parameters(&old_check, params, substitutions);
                if Arc::ptr_eq(&new_check, &old_check) || type_contains_type_parameter(&new_check)
                {
                    return Arc::clone(t);
                }
                self.resolve_conditional_type_with_check(t, Some(new_check))
                    .unwrap_or_else(|| Arc::clone(t))
            }

            _ => Arc::clone(t),
        }
    }
}
