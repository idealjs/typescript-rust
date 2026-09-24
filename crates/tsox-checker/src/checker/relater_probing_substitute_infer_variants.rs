#![allow(unused_imports)]

use crate::checker::relater_probing::*;

impl Checker {
    pub(crate) fn substitute_infer_tuple(
        &mut self,
        t: &Arc<Type>,
        tup: &TupleTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> {
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
        self.create_tuple_type(new_elems)
    }

    pub(crate) fn substitute_infer_indexed_access(
        &mut self,
        t: &Arc<Type>,
        ia: &IndexedAccessTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> {
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
    ) -> Arc<Type> {
        let Some(old_check) = ct.check_type.clone() else {
            return Arc::clone(t);
        };
        let new_check = self.substitute_infer_type_parameters(&old_check, params, substitutions);
        if Arc::ptr_eq(&new_check, &old_check) {
            return Arc::clone(t);
        }
        if type_contains_type_parameter(&new_check) {
            return self.rebuild_deferred_conditional(t, ct, new_check, params, substitutions);
        }
        // Go getConditionalType：分支结果 = instantiateType(branchNode, trueMapper)，
        // 解析出的分支再用 mapper 实例化（T→实参）；嵌套别名引用在实例化中
        // 按 (symbol, args) 缓存递归收敛
        // Go getConditionalType：分支以 trueMapper（本层 params→substitutions）
        // 实例化；帧栈是全局的，嵌套实例化时外层帧会污染本层分支节点的解析，
        // 解析期间把栈替换为本层帧
        let saved_stack = std::mem::take(&mut self.type_argument_stack);
        let mut frame: HashMap<*const tsox_frontend::ast::Symbol, Arc<Type>> = HashMap::new();
        for (i, p) in params.iter().enumerate() {
            if let Some(sym) = &p.symbol {
                frame.insert(
                    Arc::as_ptr(sym) as *const tsox_frontend::ast::Symbol,
                    Arc::clone(&substitutions[i.min(substitutions.len() - 1)]),
                );
            }
        }
        self.type_argument_stack.push(frame);
        let resolved_branch = self.resolve_conditional_type_with_check(t, Some(new_check));
        self.type_argument_stack = saved_stack;
        match resolved_branch {
            Some(branch) => self.substitute_infer_type_parameters(&branch, params, substitutions),
            None => Arc::clone(t),
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
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> {
        let new_extends = ct
            .extends_type
            .as_ref()
            .map(|e| self.substitute_infer_type_parameters(e, params, substitutions));
        let mut creation_stack = ct.creation_type_argument_stack.clone();
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
