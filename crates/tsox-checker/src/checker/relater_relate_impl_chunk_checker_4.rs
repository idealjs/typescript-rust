#![allow(unused_imports)]

use crate::checker::relater_relate_impl_chunk::*;

impl Checker {
    pub(crate) fn is_type_related_to_inner(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        if relation == RelationKind::Comparable
            && !target.flags.contains(TypeFlags::Never)
            && self.is_simple_type_related_to(target, source, relation)
        {
            return true;
        }
        // Go isTypeRelatedTo：identity 关系跳过简单类型快捷通道，
        // 结构化/可实例化类型走完整结构比较
        if relation != RelationKind::Identity
            && self.is_simple_type_related_to(source, target, relation)
        {
            return true;
        }

        let s = source.flags;
        let t = target.flags;

        // Go isRelatedToEx identity 分支：归一化后 flags 必须一致，
        // singleton 型直接通过，其余跳过全部快捷通道
        if relation == RelationKind::Identity {
            if s != t {
                return false;
            }
            if s.contains(TYPE_FLAGS_SINGLETON) {
                return true;
            }
        }

        // 可比性 carve-out：两个裸类型参数互比仅当一方约束是类型参数
        // （Go relater target TypeParameter 分支的 comparable 特例）
        if relation == RelationKind::Comparable
            && s.contains(TypeFlags::TypeParameter)
            && t.contains(TypeFlags::TypeParameter)
        {
            if let Some(constraint) = self.get_constraint_of_type_parameter(source)
                && some_type_is_type_parameter(&constraint)
            {
                return self.is_type_related_to(&constraint, target, relation);
            }
            return false;
        }

        if relation != RelationKind::Identity && s.contains(TypeFlags::TypeParameter) {
            let constraint = self
                .get_constraint_of_type_parameter(source)
                .unwrap_or_else(|| self.unknown_type());
            if self.is_type_related_to(&constraint, target, relation) {
                return true;
            }
        }

        if t.contains(TypeFlags::TypeParameter)
            && let Some(true) =
                self.mapped_source_related_to_type_param_target(&source, &target, relation)
        {
            return true;
        }

        let source_is_indexed_access = s.contains(TypeFlags::IndexedAccess)
            || matches!(source.data, TypeData::IndexedAccess(_));
        if relation != RelationKind::Identity
            && source_is_indexed_access
            && !t.contains(TypeFlags::IndexedAccess)
        {
            if let Some(constraint) = self.constraint_of_indexed_access(source)
                && self.is_type_related_to(&constraint, target, relation)
            {
                return true;
            }
        }

        if s.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
            || t.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
        {
            // Go structuredTypeRelatedToWorker identity 分支：union/intersection
            // 双向 eachTypeRelatedToSomeType（成分对成分，非成分对整体）
            if relation == RelationKind::Identity {
                return self.each_type_related_to_some_type(source, target, relation)
                    && self.each_type_related_to_some_type(target, source, relation);
            }
            return self.is_union_or_intersection_related_to(source, target, relation);
        }

        if t.contains(TypeFlags::Object)
            && !s.contains(TypeFlags::Object)
            && relation != RelationKind::Identity
            && let Some(boxed) = self.boxed_apparent_type_of_primitive(source)
        {
            let saved_chain_active = self.relater_chain_active;
            self.relater_chain_active = false;
            let saved_primitive = std::mem::replace(&mut self.relater_pending_primitive_source, true);
            let r = self.is_type_related_to(&boxed, target, relation);
            self.relater_pending_primitive_source = saved_primitive;
            self.relater_chain_active = saved_chain_active;
            return r;
        }

        if s.contains(TypeFlags::Object) && t.contains(TypeFlags::Object) {
            // ReadonlyArray<T> 接口实例归一化为 readonly 标志数组（与 readonly T[]
            // 同表示）：数组关系走元素协变快路径，避免逐成员结构比较在
            // 递归泛型（every/flatMap/concat 互相引用）上实例漂移失效
            let source = self
                .normalize_readonly_array_instance(source)
                .unwrap_or_else(|| Arc::clone(source));
            let target = self
                .normalize_readonly_array_instance(target)
                .unwrap_or_else(|| Arc::clone(target));

            if let (Some(ss), Some(ts)) = (&source.symbol, &target.symbol)
                && ss.id() == ts.id()
                && ss
                    .flags
                    .intersects(SymbolFlags::Interface | SymbolFlags::Class)
            {
                let source_args = self.get_type_arguments(&source);
                let target_args = self.get_type_arguments(&target);
                if source_args.is_empty() && target_args.is_empty() {
                    return true;
                }
                if source_args.len() == target_args.len()
                    && !source_args.is_empty()
                    && source_args.iter().zip(target_args.iter()).all(|(a, b)| {
                        self.is_type_related_to(a, b, relation)
                            && self.is_type_related_to(b, a, relation)
                    })
                {
                    return true;
                }
            }

            if self.is_array_type(&source) && self.is_array_type(&target) {
                let source_is_primitive = std::mem::take(&mut self.relater_pending_primitive_source);
                let r = self.is_array_type_related_to(&source, &target, relation, source_is_primitive);
                self.relater_pending_primitive_source = source_is_primitive;
                return r;
            }

            if self.is_tuple_type(&source) && self.is_tuple_type(&target) {
                return self.is_tuple_type_related_to(&source, &target, relation);
            }

            // Go propertiesRelatedTo 元组目标分支：源非数组/元组/演进数组时，
            // 变长（rest/variadic）目标直接拒绝；固定长度目标要求源 length
            // 可赋给长度字面量（接口的 length: number 不可赋给 N）
            if self.is_tuple_type(&target)
                && !self.is_array_type(&source)
                && !self.is_tuple_type(&source)
                && !source
                    .object_flags
                    .contains(crate::checker::types::ObjectFlags::EvolvingArray)
                && !source.flags.contains(TypeFlags::Any)
                && let crate::checker::types::TypeData::Tuple(tt) = &target.data
            {
                if tt
                    .combined_flags
                    .intersects(ElementFlags::Rest | ElementFlags::Variadic)
                {
                    return false;
                }
                let Some(length_sym) = self.get_property_of_type(&source, "length") else {
                    return false;
                };
                let source_length = self.get_type_of_symbol(&length_sym);
                if source_length.flags.contains(TypeFlags::Any) {
                    // 落结构比较
                } else if let crate::checker::types::TypeData::Literal(lit) = &source_length.data
                    && let crate::checker::types::LiteralValue::Number(n) = lit.value
                {
                    if tt.fixed_length as f64 > n.0 {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            let chain_len_before = self.relater_error_chain.len();
            if let Some(result) = self.generic_type_reference_related_to(&source, &target, relation) {
                if result.is_false() {
                    let was_active = self.relater_chain_active;
                    self.relater_chain_active = false;
                    let source_is_primitive = std::mem::take(&mut self.relater_pending_primitive_source);
                    let structural =
                        self.is_object_type_related_to(&source, &target, relation, source_is_primitive);
                    self.relater_pending_primitive_source = source_is_primitive;
                    self.relater_chain_active = was_active;
                    if structural {
                        self.relater_error_chain.truncate(chain_len_before);
                    }
                    return structural;
                }
            }
            let source_is_primitive = std::mem::take(&mut self.relater_pending_primitive_source);
            let r = self.is_object_type_related_to(&source, &target, relation, source_is_primitive);
            self.relater_pending_primitive_source = source_is_primitive;
            return r;
        }

        if relation != RelationKind::Identity
            && s.contains(TypeFlags::TypeParameter)
            && t.contains(TypeFlags::TypeParameter)
            && let (Some(ss), Some(ts)) = (&source.symbol, &target.symbol)
            && Arc::ptr_eq(ss, ts)
        {
            return true;
        }

        if t.contains(TypeFlags::IndexedAccess) {
            if let TypeData::IndexedAccess(target_access) = &target.data {
                if s.contains(TypeFlags::IndexedAccess)
                    && let TypeData::IndexedAccess(source_access) = &source.data
                    && let (Some(source_object), Some(source_index)) =
                        (&source_access.object_type, &source_access.index_type)
                    && let (Some(target_object), Some(target_index)) =
                        (&target_access.object_type, &target_access.index_type)
                {
                    let objects_related =
                        self.is_type_related_to(source_object, target_object, relation);
                    if objects_related {
                        let indexes_related =
                            self.is_type_related_to(source_index, target_index, relation);
                        if indexes_related {
                            return true;
                        }
                    }
                }
                if relation == RelationKind::Assignable || relation == RelationKind::Comparable {
                    if let (Some(object_type), Some(index_type)) =
                        (&target_access.object_type, &target_access.index_type)
                    {
                        let base_object = self.get_base_constraint_or_type(object_type);
                        let base_index = self.get_base_constraint_or_type(index_type);
                        let object_changed = !Arc::ptr_eq(&base_object, object_type);
                        if !self.type_flags_is_generic_object_type(&base_object)
                            && !self.type_flags_is_generic_index_type(&base_index)
                        {
                            let mut access_flags = AccessFlags::Writing;
                            if object_changed {
                                access_flags |= AccessFlags::NoIndexSignatures;
                            }
                            if let Some(constraint) = self.try_get_indexed_access_type(
                                &base_object,
                                &base_index,
                                access_flags,
                            ) {
                                if self.is_type_related_to(source, &constraint, relation) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }

        if t.contains(TypeFlags::Index)
            && let TypeData::Index(target_index) = &target.data
            && let Some(target_of) = &target_index.target
        {
            if s.contains(TypeFlags::Index)
                && let TypeData::Index(source_index) = &source.data
                && let Some(source_of) = &source_index.target
            {
                if self.is_type_related_to(target_of, source_of, relation) {
                    return true;
                }
            }
            // Go relater.go keyof 分支：S 可赋给 keyof C 即通过，C 为目标型的
            // 简化型或约束（无约束类型参数隐含 unknown，keyof unknown =
            // string | number | symbol）；identity 仅直比 target
            if relation != RelationKind::Identity {
                if self.is_tuple_type(target_of) {
                    if let Some(known) = self.get_known_keys_of_tuple_type(target_of)
                        && self.is_type_related_to(source, &known, relation)
                    {
                        return true;
                    }
                } else if let Some(constraint) = self.get_simplified_type_or_constraint(target_of) {
                    // Go getIndexTypeEx：keyof unknown = never、keyof any =
                    // string | number | symbol；never 走下方守卫跳过
                    let keys = if constraint.flags.contains(TypeFlags::Any) {
                        let parts = vec![
                            self.string_type(),
                            self.number_type(),
                            self.es_symbol_type(),
                        ];
                        self.get_union_type(parts)
                    } else {
                        self.get_index_type(&constraint)
                    };
                    if !keys.flags.contains(TypeFlags::Never)
                        && self.is_type_related_to(source, &keys, relation)
                    {
                        return true;
                    }
                }
            }
        }

        // Go structuredTypeRelatedToWorker identity 分支：条件类型按四元组
        // 直比（check/extends/true/false），且要求 distributivity 一致
        if relation == RelationKind::Identity
            && s.contains(TypeFlags::Conditional)
            && t.contains(TypeFlags::Conditional)
            && let (TypeData::Conditional(sc), TypeData::Conditional(tc)) = (&source.data, &target.data)
        {
            let distributive_match = sc
                .root
                .as_ref()
                .is_some_and(|r| r.is_distributive)
                == tc.root.as_ref().is_some_and(|r| r.is_distributive);
            if !distributive_match {
                return false;
            }
            let (Some(sc_check), Some(tc_check)) = (
                sc.root.as_ref().and_then(|r| r.check_type.clone()),
                tc.root.as_ref().and_then(|r| r.check_type.clone()),
            ) else {
                return false;
            };
            let (Some(sc_extends), Some(tc_extends)) = (
                sc.root.as_ref().and_then(|r| r.extends_type.clone()),
                tc.root.as_ref().and_then(|r| r.extends_type.clone()),
            ) else {
                return false;
            };
            if !self.is_type_related_to(&sc_check, &tc_check, relation)
                || !self.is_type_related_to(&sc_extends, &tc_extends, relation)
            {
                return false;
            }
            let (Some(st), Some(tt)) = (
                self.get_true_type_of_conditional_type(&source),
                self.get_true_type_of_conditional_type(&target),
            ) else {
                return false;
            };
            if !self.is_type_related_to(&st, &tt, relation) {
                return false;
            }
            let (Some(sf), Some(tf)) = (
                self.get_false_type_of_conditional_type(&source),
                self.get_false_type_of_conditional_type(&target),
            ) else {
                return false;
            };
            return self.is_type_related_to(&sf, &tf, relation);
        }

        if relation != RelationKind::Identity && s.contains(TypeFlags::Conditional) {
            let resolved = match self.get_resolved_type_of_conditional_type(source) {
                Some(resolved) => Some(resolved),

                None => self.resolve_conditional_type(source),
            };
            if let Some(resolved) = resolved {
                if self.is_type_related_to(&resolved, target, relation) {
                    return true;
                }
            }
        }
        if relation != RelationKind::Identity && t.contains(TypeFlags::Conditional) {
            let resolved = match self.get_resolved_type_of_conditional_type(target) {
                Some(resolved) => Some(resolved),
                None => self.resolve_conditional_type(target),
            };
            if let Some(resolved) = resolved {
                if self.is_type_related_to(source, &resolved, relation) {
                    return true;
                }

                if !type_contains_type_parameter(&resolved) {
                    return false;
                }
            }

            if let Some(result) = self.conditional_type_related_to(source, target, relation) {
                if result.is_true() {
                    return true;
                }
                if result.is_false() {
                    return false;
                }
            }
        }

        if relation != RelationKind::Identity
            && s.contains(TypeFlags::Object)
            && source.object_flags.contains(ObjectFlags::Mapped)
        {
            if let Some(constraint) = self.get_constraint_of_mapped_type(source) {
                if self.is_type_related_to(&constraint, target, relation) {
                    return true;
                }
            }
        }
        if relation != RelationKind::Identity
            && t.contains(TypeFlags::Object)
            && target.object_flags.contains(ObjectFlags::Mapped)
        {
            if let Some(constraint) = self.get_constraint_of_mapped_type(target) {
                if self.is_type_related_to(source, &constraint, relation) {
                    return true;
                }
            }

            if s.contains(TypeFlags::Object) && source.object_flags.contains(ObjectFlags::Mapped) {
                if let Some(result) = self.mapped_type_related_to(source, target, relation) {
                    if result.is_true() {
                        return true;
                    }
                    if result.is_false() {
                        return false;
                    }
                }
            }
        }

        if s.contains(TypeFlags::TypeParameter)
            && !t.contains(TypeFlags::TypeParameter)
            && self.relater_chain_active
            && self.speculation_depth == 0
            && relation != RelationKind::Identity
            && let Some(constraint) = self.get_constraint_of_type_parameter(source)
            && !constraint.flags.contains(TypeFlags::Unknown)
        {
            let cs = self.type_to_string(&constraint);
            let ts = self.type_to_string(target);
            self.relater_report_error(
                tsox_core::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                vec![cs, ts],
            );
        }

        false
    }

    pub(crate) fn is_array_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        source_is_primitive: bool,
    ) -> bool {
        let source_args = self.get_type_arguments(source);
        let target_args = self.get_type_arguments(target);

        if source_args.is_empty() || target_args.is_empty() {
            return self.is_object_type_related_to(source, target, relation, source_is_primitive);
        }

        // readonly → 可变按赋值/子类型关系拒绝（可变 → readonly 放行，
        // 元素协变照常）
        let source_ro = source.object_flags.contains(ObjectFlags::IsReadonlyArray);
        let target_ro = target.object_flags.contains(ObjectFlags::IsReadonlyArray);
        if source_ro
            && !target_ro
            && matches!(
                relation,
                RelationKind::Assignable | RelationKind::Subtype | RelationKind::StrictSubtype
            )
        {
            if self.relater_chain_active {
                let source_str = self.type_to_string(source);
                let target_str = self.type_to_string(target);
                self.relater_report_error(
                    tsox_core::diagnostics::messages_generated::
                        THE_TYPE_0_IS_READONLY_AND_CANNOT_BE_ASSIGNED_TO_THE_MUTABLE_TYPE_1,
                    vec![source_str, target_str],
                );
            }
            return false;
        }

        let source_elem = &source_args[0];
        let target_elem = &target_args[0];
        let related = self.is_type_related_to(source_elem, target_elem, relation);
        if !related && self.relater_chain_active {
            let elem_source_str = self.type_to_string(source_elem);
            let elem_target_str = self.type_to_string(target_elem);
            self.relater_report_error(
                tsox_core::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                vec![elem_source_str, elem_target_str],
            );
        }
        related
    }

    /// Array/ReadonlyArray 接口实例（含关系判定中途惰性解析的 Anonymous 形态）
    /// → 驻留数组实例（readonly 语义由 IsReadonlyArray 标志承载）；
    /// 其余类型原样返回。符号比对带名字等价兜底：lib 文件的符号存在双副本
    /// 实例漂移（globals 与类型引用解析不同 Arc），ptr_eq 会漏
    pub(crate) fn normalize_readonly_array_instance(
        &mut self,
        t: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        if !t.flags.contains(TypeFlags::Object) {
            return None;
        }
        let args_len = t.as_object()?.type_arguments.len();
        if args_len != 1 {
            return None;
        }
        let symbol = t.symbol.as_ref()?;
        let array_sym = self.globals.get("Array")?;
        let readonly_sym = self.globals.get("ReadonlyArray")?;
        let is_ro_symbol = Arc::ptr_eq(symbol, readonly_sym)
            || (symbol.name == readonly_sym.name
                && symbol.flags.contains(SymbolFlags::Interface));
        let is_array_symbol = Arc::ptr_eq(symbol, array_sym)
            || (symbol.name == array_sym.name
                && symbol.flags.contains(SymbolFlags::Interface));
        let readonly = if is_ro_symbol {
            true
        } else if is_array_symbol {
            t.object_flags
                .contains(crate::checker::types::ObjectFlags::IsReadonlyArray)
        } else {
            return None;
        };
        let element = Arc::clone(&t.as_object()?.type_arguments[0]);
        Some(self.create_array_type_ex(element, readonly))
    }
}

fn some_type_is_type_parameter(t: &Arc<Type>) -> bool {
    if let TypeData::Union(u) = &t.data {
        return u
            .union_or_intersection
            .types
            .iter()
            .any(|m| m.flags.contains(TypeFlags::TypeParameter));
    }
    t.flags.contains(TypeFlags::TypeParameter)
}
