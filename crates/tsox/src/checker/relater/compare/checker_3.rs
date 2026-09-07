#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn check_type_related_to_and_optionally_elaborate(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        error_node: Option<&Arc<crate::ast::Node>>,
        expr: Option<&Arc<crate::ast::Node>>,
        head_message: Option<&crate::diagnostics::Message>,
        mut diagnostic_output: Option<&mut Vec<crate::ast::Diagnostic>>,
    ) -> bool {
        {
            let sp = source.id;
            let tp = target.id;
            if source.flags.contains(TypeFlags::Object)
                && target.flags.contains(TypeFlags::Object)
                && (self.degraded_type_ptrs.contains(&sp) || self.degraded_type_ptrs.contains(&tp))
            {
                return true;
            }
        }

        if self.speculation_depth > 0 {
            return self.is_type_related_to(source, target, relation);
        }
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
            && self.elaborate_error(
                expr,
                source,
                target,
                relation,
                diagnostic_output.as_deref_mut(),
            )
        {
            self.relater_chain_active = was_active;
            self.relater_error_chain = saved_chain;
            return false;
        }

        self.try_elaborate_primitive_and_object(source, target);

        let displayed_target = self
            .display_target_override
            .clone()
            .unwrap_or_else(|| Arc::clone(target));
        let source_str = self.type_to_string(source);
        let target_str = self.type_to_string(&displayed_target);
        let (head_source, head_target) = if self.type_could_have_top_level_singleton_types(target) {
            (source_str.clone(), target_str.clone())
        } else if crate::checker::is_fresh_literal_type(source)
            || source.flags.intersects(TYPE_FLAGS_LITERAL)
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
                crate::diagnostics::messages_generated::
                    TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1_TWO_DIFFERENT_TYPES_WITH_THIS_NAME_EXIST_BUT_THEY_ARE_UNRELATED
            }
            None => crate::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
        };

        let mut suppress_head = false;
        if head_message.is_none()
            && let Some(entry) = self.relater_error_chain.last()
        {
            let m = entry.message;
            let a = &entry.args;
            suppress_head = if m
                == crate::diagnostics::messages_generated::
                    PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2
            {
                a.len() == 3 && a[1] == head_source && a[2] == head_target
            } else if m
                == crate::diagnostics::messages_generated::
                    TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2
                || m
                    == crate::diagnostics::messages_generated::
                        TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2_AND_3_MORE
            {
                a.len() >= 2 && a[0] == head_source && a[1] == head_target
            } else if m
                == crate::diagnostics::messages_generated::
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
        let file = self
            .get_source_file_of_node(error_node)
            .or_else(|| self.current_file.clone());
        let mut diagnostic: Option<crate::ast::Diagnostic> = None;
        for entry in self.relater_error_chain.iter() {
            if entry.message.elided_in_compatibility_pyramid {
                continue;
            }
            let mut d = crate::ast::Diagnostic::new(
                file.clone(),
                error_node.loc,
                entry.message,
                entry.args.clone(),
            );
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
            if self.is_known_property(target, &p.name, false) {
                return true;
            }
        }
        false
    }

    pub fn is_known_property(
        &mut self,
        target_type: &Arc<Type>,
        name: &str,
        _is_comparing_jsx_attributes: bool,
    ) -> bool {
        if let Some(structured) = target_type.as_structured() {
            if structured.members.get(name).is_some() {
                return true;
            }
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
