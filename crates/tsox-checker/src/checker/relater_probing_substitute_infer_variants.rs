#![allow(unused_imports)]

use crate::checker::relater_probing::*;

impl Checker {
    pub(crate) fn substitute_infer_tuple(
        &mut self,
        t: &Arc<Type>,
        tup: &TupleTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("substitute_infer_tuple"); 
        let new_elems: Vec<Arc<Type>> = tup
            .element_infos
            .iter()
            .map(|ei| match &ei.type_ {
                Some(ty) => self.substitute_infer_type_parameters(ty, params, substitutions),
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
        let mut element_types: Vec<Arc<Type>> = Vec::new();
        let mut element_infos: Vec<TupleElementInfo> = Vec::new();
        for (ei, new_t) in tup.element_infos.iter().zip(new_elems.iter()) {
            if ei.flags.contains(ElementFlags::Variadic) {
                self.spread_variadic_tuple_element(
                    new_t,
                    ei,
                    &mut element_types,
                    &mut element_infos,
                );
            } else {
                element_types.push(Arc::clone(new_t));
                element_infos.push(TupleElementInfo {
                    label: ei.label.clone(),
                    flags: ei.flags,
                    labeled_declaration: ei.labeled_declaration.clone(),
                    type_: None,
                });
            }
        }
        self.create_tuple_type_ex(element_types, element_infos, tup.readonly)
    }

    pub(crate) fn substitute_infer_indexed_access(
        &mut self,
        t: &Arc<Type>,
        ia: &IndexedAccessTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("substitute_infer_indexed_access"); 
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
        let object_type = new_object.or_else(|| ia.object_type.clone());
        let index_type = new_index.or_else(|| ia.index_type.clone());
        // Go instantiateIndexedAccessType：替换后对象类型不再是泛型载体
        // （类型参数/索引访问/条件型）时立即归约（T["name"] 代入 Error 后解析为 string）
        let generic_object = |t: &Arc<Type>| {
            t.flags.intersects(
                TypeFlags::TypeParameter | TypeFlags::IndexedAccess | TypeFlags::Conditional,
            ) || matches!(&t.data, TypeData::IndexedAccess(_))
        };
        if let (Some(obj), Some(idx)) = (object_type.as_ref(), index_type.as_ref())
            && !generic_object(obj)
            && !generic_object(idx)
        {
            let resolved = self.get_indexed_access_type(obj, idx);
            if resolved.intrinsic_name() != Some("error") {
                return resolved;
            }
        }
        // 保持延迟的 IndexedAccess 按 (对象, 索引) 驻留，保证类型恒等
        // （推断信息表按恒等匹配，Go 依赖 interning）
        if let (Some(obj), Some(idx)) = (object_type.as_ref(), index_type.as_ref()) {
            let interned = self.deferred_indexed_access(obj, idx);
            if interned.flags.contains(TypeFlags::IndexedAccess) {
                return interned;
            }
        }
        let mut rebuilt = Type::new(
            t.flags,
            TypeData::IndexedAccess(IndexedAccessTypeData {
                constrained: ConstrainedTypeData::default(),
                object_type,
                index_type,
                access_flags: ia.access_flags,
            }),
        );
        rebuilt.object_flags = t.object_flags;
        rebuilt.symbol = t.symbol.clone();
        Arc::new(rebuilt)
    }

    pub(crate) fn substitute_infer_conditional(
        &mut self,
        t: &Arc<Type>,
        ct: &ConditionalTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("substitute_infer_conditional"); 
        let Some(old_check) = ct.check_type.clone() else {
            return Arc::clone(t);
        };
        let old_extends = ct.extends_type.clone();
        let new_check = self.substitute_infer_type_parameters(&old_check, params, substitutions);
        let new_extends = old_extends
            .as_ref()
            .map(|e| self.substitute_infer_type_parameters(e, params, substitutions));
        let check_changed = !Arc::ptr_eq(&new_check, &old_check);
        let extends_changed = new_extends
            .as_ref()
            .zip(old_extends.as_ref())
            .is_some_and(|(n, o)| !Arc::ptr_eq(n, o));
        if !check_changed && !extends_changed {
            return Arc::clone(t);
        }
        let infer_params: Vec<Arc<Type>> = ct
            .root
            .as_ref()
            .map(|r| r.infer_type_parameters.clone())
            .unwrap_or_default();
        if type_contains_type_parameter(&new_check)
            || new_extends
                .as_ref()
                .is_some_and(|e| {
                    crate::checker::relater_type_params::type_contains_type_parameter_skipping(
                        e, &infer_params,
                    )
                })
        {
            return self.rebuild_deferred_conditional(
                t,
                ct,
                new_check,
                new_extends,
                params,
                substitutions,
            );
        }
        // Go getConditionalType：分支结果 = instantiateType(branchNode, trueMapper)，
        // 解析出的分支再用 mapper 实例化（T→实参）；嵌套别名引用在实例化中
        // 按 (symbol, args) 缓存递归收敛
        // Go getConditionalType：分支以 trueMapper（本层 params→substitutions）
        // 实例化；帧栈是全局的，嵌套实例化时外层帧会污染本层分支节点的解析，
        // 解析期间把栈替换为本层帧
        let saved_stack = std::mem::take(&mut self.type_argument_stack);
        let mut frames: Vec<HashMap<*const Symbol, Arc<Type>>> = ct
            .creation_type_argument_stack
            .iter()
            .map(|frame| {
                frame
                    .iter()
                    .map(|(k, v)| {
                        (
                            *k as *const Symbol,
                            self.substitute_infer_type_parameters(v, params, substitutions),
                        )
                    })
                    .collect()
            })
            .collect();
        let mut frame: HashMap<*const Symbol, Arc<Type>> = HashMap::new();
        for (i, p) in params.iter().enumerate() {
            if let Some(sym) = &p.symbol {
                frame.insert(
                    Arc::as_ptr(sym),
                    Arc::clone(&substitutions[i.min(substitutions.len() - 1)]),
                );
            }
        }
        if !frame.is_empty() {
            frames.push(frame);
        }
        self.type_argument_stack = frames;
        // Go getConditionalTypeInstantiation：distributive 条件型的 check 型
        // 被映射为 union 时按成分分发求值（A extends U ? X : Y | B extends U ?
        // X : Y），never 收敛为 never；成分不可判定时保持挂起
        let distribute = ct.root.as_ref().is_some_and(|r| r.is_distributive)
            && (new_check.flags.contains(TypeFlags::Never)
                || matches!(&new_check.data, TypeData::Union(_)));
        let resolved_branch = if new_check.flags.contains(TypeFlags::Never) && distribute {
            Some(self.never_type())
        } else if distribute
            && let TypeData::Union(u) = &new_check.data
            && let Some(check_sym) = ct
                .root
                .as_ref()
                .and_then(|r| r.check_type_parameter_symbol.clone())
        {
            let constituents = u.union_or_intersection.types.clone();
            let key = Arc::as_ptr(&check_sym) as *const tsox_frontend::ast::Symbol;
            let mut results: Vec<Arc<Type>> = Vec::with_capacity(constituents.len());
            let mut all_resolved = true;
            for constituent in constituents {
                let mut mapping = HashMap::new();
                mapping.insert(key, Arc::clone(&constituent));
                self.type_argument_stack.push(mapping);
                let r = self.resolve_conditional_type_with_check(
                    t,
                    Some(Arc::clone(&constituent)),
                    new_extends.clone(),
                );
                self.type_argument_stack.pop();
                match r {
                    Some(r) => results.push(r),
                    None => {
                        all_resolved = false;
                        break;
                    }
                }
            }
            if all_resolved {
                Some(self.get_union_type(results))
            } else {
                None
            }
        } else {
            self.resolve_conditional_type_with_check(
                t,
                Some(Arc::clone(&new_check)),
                new_extends.clone(),
            )
        };
        self.type_argument_stack = saved_stack;
        match resolved_branch {
            Some(branch) => self.substitute_infer_type_parameters(&branch, params, substitutions),
            None => self.rebuild_deferred_conditional(
                t,
                ct,
                new_check,
                new_extends,
                params,
                substitutions,
            ),
        }
    }

    /// 挂起条件型代入后仍含类型参数（别名体 T→TActor 这类外层形参）：Go
    /// instantiateType 对 deferred conditional 实例化 check/extends 并携带
    /// mapper；这里重建携带代入映射的新挂起条件，映射并入
    /// creation_type_argument_stack 供后续分支解析时回放
    fn rebuild_deferred_conditional(
        &mut self,
        t: &Arc<Type>,
        ct: &ConditionalTypeData,
        new_check: Arc<Type>,
        new_extends: Option<Arc<Type>>,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("rebuild_deferred_conditional"); 
        let mut creation_stack: Vec<HashMap<usize, Arc<Type>>> = ct
            .creation_type_argument_stack
            .iter()
            .map(|frame| {
                frame
                    .iter()
                    .map(|(k, v)| {
                        (
                            *k,
                            self.substitute_infer_type_parameters(v, params, substitutions),
                        )
                    })
                    .collect()
            })
            .collect();
        let mut frame: HashMap<usize, Arc<Type>> = HashMap::new();
        for (i, p) in params.iter().enumerate() {
            if let Some(sym) = &p.symbol {
                frame.insert(
                    Arc::as_ptr(sym) as usize,
                    Arc::clone(&substitutions[i.min(substitutions.len() - 1)]),
                );
            }
        }
        if !frame.is_empty() {
            creation_stack.push(frame);
        }
        let mut rebuilt = Type::new(
            t.flags,
            TypeData::Conditional(ConditionalTypeData {
                constrained: ConstrainedTypeData::default(),
                root: ct.root.clone(),
                check_type: Some(new_check),
                extends_type: new_extends,
                resolved_true_type: OnceLock::new(),
                resolved_false_type: OnceLock::new(),
                resolved_inferred_true_type: OnceLock::new(),
                resolved_default_constraint: OnceLock::new(),
                resolved_constraint_of_distributive: OnceLock::new(),
                mapper: ct.mapper.clone(),
                combined_mapper: ct.combined_mapper.clone(),
                creation_type_argument_stack: creation_stack,
            }),
        );
        rebuilt.symbol = t.symbol.clone();
        rebuilt.alias = t.alias.as_ref().map(|a| {
            let new_args: Vec<Arc<Type>> = a
                .type_arguments
                .iter()
                .map(|arg| self.substitute_infer_type_parameters(arg, params, substitutions))
                .collect();
            Box::new(crate::checker::types::TypeAlias::new(
                a.symbol.clone(),
                new_args,
            ))
        });
        rebuilt.object_flags = t.object_flags;
        Arc::new(rebuilt)
    }
}
