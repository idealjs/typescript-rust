#![allow(unused_imports)]

use crate::checker::relater_compare::*;

impl Checker {
    pub fn check_type_related_to_and_optionally_elaborate(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        error_node: Option<&Arc<tsox_frontend::ast::Node>>,
        expr: Option<&Arc<tsox_frontend::ast::Node>>,
        head_message: Option<&tsox_core::diagnostics::Message>,
        mut diagnostic_output: Option<&mut Vec<tsox_frontend::ast::Diagnostic>>,
    ) -> bool {
        {
            let sp = source.id;
            let tp = target.id;
            if source.flags.contains(TypeFlags::Object)
                && target.flags.contains(TypeFlags::Object)
                // 任一方是「空成员且带符号」的壳型（lib 解析重入期的未完成实例）
                // 即放行：带符号才免于误放 `{}` 字面量；有成员的完整实例照常
                // 结构比较（递归由 relation_in_progress 兜底）。
                // 注意：`&&…||…` 必须括号——无括号时 `|| target_shell` 会脱离
                // Object 守卫与 pending 门，target 空壳即无条件放行吞诊断
                && !self.pending_interface_shells.is_empty()
                && (self.side_is_incomplete_shell(source, sp)
                    || self.side_is_incomplete_shell(target, tp))
            {
                return true;
            }
        }

        if self.speculation_depth > 0 {
            return self.is_type_related_to(source, target, relation);
        }
        self.relater_excess_error_node = None;
        self.relater_error_node = error_node.cloned();
        let saved_chain = std::mem::take(&mut self.relater_error_chain);
        let was_active = self.relater_chain_active;
        self.relater_chain_active = true;
        let ok = self.is_type_related_to(source, target, relation);
        if ok {
            self.relater_chain_active = was_active;
            self.relater_error_chain = saved_chain;
            return true;
        }

        if let Some(expr) = expr
            && self.elaborate_error_with_head(
                expr,
                source,
                target,
                relation,
                head_message,
                diagnostic_output.as_deref_mut(),
            )
        {
            self.relater_chain_active = was_active;
            self.relater_error_chain = saved_chain;
            return false;
        }

        self.try_elaborate_primitive_and_object(source, target);

        // Go reportErrorResults：源为全局 Object 接口型时在链上补 TS2696
        //（object→primitive 分支被 switch 前置，不再叠加）
        if source.flags.contains(TypeFlags::Object)
            && !target
                .flags
                .intersects(crate::checker::types_type_id::TYPE_FLAGS_PRIMITIVE)
            && source.symbol.as_ref().is_some_and(|sym| {
                self.globals
                    .get("Object")
                    .is_some_and(|g| Arc::ptr_eq(g, sym))
            })
        {
            use tsox_core::diagnostics::messages_generated as msg;
            self.relater_report_error(
                msg::THE_OBJECT_TYPE_IS_ASSIGNABLE_TO_VERY_FEW_OTHER_TYPES_DID_YOU_MEAN_TO_USE_THE_ANY_TYPE_INSTEAD,
                Vec::new(),
            );
        }

        // Go isRelatedToEx 目标重绑定同样作用于错误报告：gate 与兜底显示
        // 都基于剔除可空成分后的目标（display override 仍优先）；identity
        // 关系不走该路径
        let rebound_target = if relation == RelationKind::Identity {
            Arc::clone(target)
        } else {
            self.rebind_non_nullable_union_target(source, target)
                .unwrap_or_else(|| Arc::clone(target))
        };
        let displayed_target = self
            .display_target_override
            .clone()
            .unwrap_or_else(|| Arc::clone(&rebound_target));
        let source_str = self.type_to_string(source);
        let target_str = self.type_to_string(&displayed_target);
        let (mut head_source, mut head_target) = if self
            .type_could_have_top_level_singleton_types(&rebound_target)
        {
            (source_str.clone(), target_str.clone())
        } else if !target.flags.contains(TypeFlags::Never)
            && (crate::checker::is_fresh_literal_type(source)
                || source.flags.intersects(TYPE_FLAGS_LITERAL))
        {
            let base = self.get_base_type_of_literal_type_for_display(source);
            (self.type_to_string(&base), target_str.clone())
        } else if source
            .object_flags
            .contains(crate::checker::types::ObjectFlags::ObjectLiteral)
            && source.symbol.is_none()
        {
            let widened = self.widen_object_literal_type(source);
            (self.type_to_string(&widened), target_str.clone())
        } else {
            (source_str.clone(), target_str.clone())
        };
        let head = match head_message {
            Some(m) => *m,
            None if head_source == head_target => {
                // Go getTypeNamesForErrorDisplay：同名不同型改用全限定显示
                let fq_source = self.fully_qualified_type_string(source);
                let fq_target = self.fully_qualified_type_string(&displayed_target);
                if fq_source != fq_target {
                    head_source = fq_source;
                    head_target = fq_target;
                    tsox_core::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1
                } else if relation == RelationKind::Comparable {
                    tsox_core::diagnostics::messages_generated::TYPE_0_IS_NOT_COMPARABLE_TO_TYPE_1
                } else {
                    tsox_core::diagnostics::messages_generated::
                        TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1_TWO_DIFFERENT_TYPES_WITH_THIS_NAME_EXIST_BUT_THEY_ARE_UNRELATED
                }
            }
            None if relation == RelationKind::Comparable => {
                tsox_core::diagnostics::messages_generated::TYPE_0_IS_NOT_COMPARABLE_TO_TYPE_1
            }
            None => tsox_core::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
        };

        use tsox_core::diagnostics::messages_generated as msg;
        let head_is_conversion_or_impl = head == msg::CLASS_0_INCORRECTLY_IMPLEMENTS_INTERFACE_1
            || head == msg::CLASS_0_INCORRECTLY_IMPLEMENTS_CLASS_1_DID_YOU_MEAN_TO_EXTEND_1_AND_INHERIT_ITS_MEMBERS_AS_A_SUBCLASS
            || head == msg::CONVERSION_OF_TYPE_0_TO_TYPE_1_MAY_BE_A_MISTAKE_BECAUSE_NEITHER_TYPE_SUFFICIENTLY_OVERLAPS_WITH_THE_OTHER_IF_THIS_WAS_INTENTIONAL_CONVERT_THE_EXPRESSION_TO_UNKNOWN_FIRST
            || head == msg::ITS_INSTANCE_TYPE_0_IS_NOT_A_VALID_JSX_ELEMENT
            || head == msg::ITS_RETURN_TYPE_0_IS_NOT_A_VALID_JSX_ELEMENT
            || head == msg::ITS_ELEMENT_TYPE_0_IS_NOT_A_VALID_JSX_ELEMENT;

        let mut suppress_head = false;
        if !head_is_conversion_or_impl
            && let Some(entry) = self.relater_error_chain.last()
        {
            let m = entry.message;
            let a = &entry.args;
            suppress_head = if m
                == tsox_core::diagnostics::messages_generated::
                    PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2
            {
                a.len() == 3 && a[1] == head_source && a[2] == head_target
            } else if m
                == tsox_core::diagnostics::messages_generated::
                    TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2
                || m
                    == tsox_core::diagnostics::messages_generated::
                        TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2_AND_3_MORE
            {
                a.len() >= 2 && a[0] == head_source && a[1] == head_target
            } else if m
                == tsox_core::diagnostics::messages_generated::
                    OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1
                || m
                    == tsox_core::diagnostics::messages_generated::
                        OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_BUT_0_DOES_NOT_EXIST_IN_TYPE_1_DID_YOU_MEAN_TO_WRITE_2
            {
                true
            } else if m
                == tsox_core::diagnostics::messages_generated::
                    THE_TYPE_0_IS_READONLY_AND_CANNOT_BE_ASSIGNED_TO_THE_MUTABLE_TYPE_1
            {
                a.len() == 2 && a[0] == head_source && a[1] == head_target
            } else {
                false
            };
        }
        if !suppress_head {
            self.push_relation_head_with_tp_note(
                source,
                &displayed_target,
                head,
                vec![head_source, head_target],
            );
        }

        let Some(error_node) = error_node else {
            self.relater_chain_active = was_active;
            self.relater_error_chain = saved_chain;
            return false;
        };
        let (pos_node, file) = match self.relater_excess_error_node.clone() {
            Some(n) => (
                n.loc,
                self.get_source_file_of_node(&n).or_else(|| self.current_file.clone()),
            ),
            None => (
                error_node.loc,
                self
                    .get_source_file_of_node(error_node)
                    .or_else(|| self.current_file.clone()),
            ),
        };
        let mut diagnostic: Option<tsox_frontend::ast::Diagnostic> = None;
        for entry in self.relater_error_chain.iter() {
            if entry.message.elided_in_compatibility_pyramid {
                continue;
            }
            let mut d = tsox_frontend::ast::Diagnostic::new(
                file.clone(),
                pos_node,
                entry.message,
                entry.args.clone(),
            );
            if let Some(rel) = &entry.related {
                d.related_information.push(tsox_frontend::ast::Diagnostic::new(
                    rel.file.clone(),
                    rel.loc,
                    rel.message,
                    rel.args.clone(),
                ));
            }
            if let Some(child) = diagnostic.take() {
                d.message_chain = vec![child];
            }
            diagnostic = Some(d);
        }
        if let Some(d) = diagnostic {
            match diagnostic_output {
                Some(out) => out.push(d),
                None => self.diagnostics.add(d),
            }
        }
        self.relater_chain_active = was_active;
        self.relater_error_chain = saved_chain;
        false
    }

    // Go isRelatedToEx 前段：definitely non-nullable 源 + union 目标仅含一个
    // 非 nullable 成员时目标重绑定为该成员（关系判定与错误报告共用）
    pub(crate) fn rebind_non_nullable_union_target(
        &self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        if !source.flags.intersects(TYPE_FLAGS_DEFINITELY_NON_NULLABLE)
            || !target.flags.contains(TypeFlags::Union)
        {
            return None;
        }
        let types = target.types()?;
        if types.len() != 2 && types.len() != 3 {
            return None;
        }
        let mut candidate: Option<Arc<Type>> = None;
        for m in types {
            if m.flags.intersects(TYPE_FLAGS_NULLABLE) {
                continue;
            }
            if candidate.is_some() {
                return None;
            }
            candidate = Some(Arc::clone(m));
        }
        candidate
    }

    pub(crate) fn get_base_type_of_literal_type_for_display(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(TypeFlags::StringLiteral) || t.flags.contains(TypeFlags::StringMapping)
        {
            self.string_type()
        } else if t.flags.contains(TypeFlags::NumberLiteral) {
            self.number_type()
        } else if t.flags.contains(TypeFlags::BigIntLiteral) {
            self.bigint_type()
        } else if t.flags.contains(TypeFlags::BooleanLiteral) {
            self.boolean_type()
        } else {
            Arc::clone(t)
        }
    }

    pub fn is_weak_type(&mut self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Object) {
            if t.flags.contains(TypeFlags::Any) {
                return false;
            }
            let Some(structured) = t.as_structured() else {
                return false;
            };
            if !structured.index_infos.is_empty() {
                return false;
            }
            if !structured.call_signatures().is_empty()
                || !structured.construct_signatures().is_empty()
            {
                return false;
            }
            if structured.properties.is_empty() {
                return false;
            }
            return structured
                .properties
                .iter()
                .all(|p| p.flags.contains(SymbolFlags::Optional));
        } else if t.flags.contains(TypeFlags::Substitution) {
            if let TypeData::Substitution(s) = &t.data {
                s.base_type
                    .as_ref()
                    .map(|bt| self.is_weak_type(bt))
                    .unwrap_or(false)
            } else {
                false
            }
        } else if t.flags.contains(TypeFlags::Intersection) {
            if let Some(types) = t.types() {
                types.iter().all(|ty| self.is_weak_type(ty))
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn has_common_properties(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _is_comparing_jsx_attributes: bool,
    ) -> bool {
        let Some(source_struct) = source.as_structured() else {
            return false;
        };
        for p in &source_struct.properties {
            if self.is_known_property(target, &p.name, _is_comparing_jsx_attributes) {
                return true;
            }
        }
        // Go isKnownProperty 经 resolveStructuredType 合并基类型成员;
        // 实例化泛型接口(HTMLProps<T> extends 多接口)的成员合并未落盘时,
        // JSX 属性源的弱类型检查按存在公共属性处理,避免误报 TS2322
        if _is_comparing_jsx_attributes
            && target.flags.contains(TypeFlags::Object)
            && target
                .symbol
                .as_ref()
                .is_some_and(|s| s.flags.contains(SymbolFlags::Interface))
        {
            return true;
        }
        if _is_comparing_jsx_attributes
            && target.flags.contains(TypeFlags::Intersection)
            && target.types().is_some_and(|types| {
                types.iter().any(|t| {
                    t.flags.contains(TypeFlags::Object)
                        && t
                            .symbol
                            .as_ref()
                            .is_some_and(|s| s.flags.contains(SymbolFlags::Interface))
                })
            })
        {
            return true;
        }
        false
    }

    pub fn is_known_property(
        &mut self,
        target_type: &Arc<Type>,
        name: &str,
        is_comparing_jsx_attributes: bool,
    ) -> bool {
        if target_type.flags.contains(TypeFlags::Object) {
            if self.get_property_of_type(target_type, name).is_some()
                || self.target_index_covers_name(target_type, name)
                || is_comparing_jsx_attributes
                    && crate::checker::relater_predicates::is_hyphenated_jsx_name(name)
            {
                return true;
            }
        }
        if let TypeData::Substitution(s) = &target_type.data
            && let Some(base) = &s.base_type
        {
            return self.is_known_property(base, name, is_comparing_jsx_attributes);
        }
        if target_type.flags.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
            && crate::checker::relater_predicates::is_excess_property_check_target(target_type)
            && let Some(types) = target_type.types()
        {
            return types
                .iter()
                .any(|t| self.is_known_property(t, name, is_comparing_jsx_attributes));
        }
        false
    }

    fn target_index_covers_name(&self, target_type: &Arc<Type>, name: &str) -> bool {
        let Some(structured) = target_type.as_structured() else {
            return false;
        };
        for info in &structured.index_infos {
            if let Some(key) = &info.key_type {
                if key.flags.contains(TypeFlags::String) {
                    return true;
                }
                if key.flags.contains(TypeFlags::Number) && name.parse::<f64>().is_ok() {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_mapped_target_with_symbol(&self, t: &Arc<Type>) -> Arc<Type> {
        Arc::clone(t)
    }

    pub fn has_matching_recursion_identity(&self, t: &Arc<Type>, identity: &Arc<Type>) -> bool {
        Arc::ptr_eq(t, identity)
    }

    pub fn get_best_matching_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        _is_related_to: &dyn Fn(&Arc<Type>, &Arc<Type>) -> Ternary,
    ) -> Option<Arc<Type>> {
        let _ = (source, target);
        None
    }

    pub fn find_matching_type_reference_or_type_alias_reference(
        &mut self,
        source: &Arc<Type>,
        union_target: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let _ = (source, union_target);
        None
    }

    pub fn find_best_type_for_invokable(
        &mut self,
        source: &Arc<Type>,
        union_target: &Arc<Type>,
        _kind: SignatureKind,
    ) -> Option<Arc<Type>> {
        let _ = (source, union_target);
        None
    }
}
