#![allow(unused_imports)]

use super::*;

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
        if self.is_simple_type_related_to(source, target, relation) {
            return true;
        }

        let s = source.flags;
        let t = target.flags;

        if s.contains(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_constraint_of_type_parameter(source) {
                if self.is_type_related_to(&constraint, target, relation) {
                    return true;
                }
            }
        }

        let source_is_indexed_access = s.contains(TypeFlags::IndexedAccess)
            || matches!(source.data, TypeData::IndexedAccess(_));
        if source_is_indexed_access && !t.contains(TypeFlags::IndexedAccess) {
            if let Some(constraint) = self.constraint_of_indexed_access(source)
                && self.is_type_related_to(&constraint, target, relation)
            {
                return true;
            }
        }

        if s.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
            || t.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
        {
            return self.is_union_or_intersection_related_to(source, target, relation);
        }

        if t.contains(TypeFlags::Object)
            && !s.contains(TypeFlags::Object)
            && relation != RelationKind::Identity
            && let Some(boxed) = self.boxed_apparent_type_of_primitive(source)
        {
            let saved_chain_active = self.relater_chain_active;
            self.relater_chain_active = false;
            let r = self.is_type_related_to(&boxed, target, relation);
            self.relater_chain_active = saved_chain_active;
            return r;
        }

        if s.contains(TypeFlags::Object) && t.contains(TypeFlags::Object) {
            if let (Some(ss), Some(ts)) = (&source.symbol, &target.symbol)
                && ss.id() == ts.id()
                && ss
                    .flags
                    .intersects(SymbolFlags::Interface | SymbolFlags::Class)
            {
                let source_args = self.get_type_arguments(source);
                let target_args = self.get_type_arguments(target);
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

            if self.is_array_type(source) && self.is_array_type(target) {
                return self.is_array_type_related_to(source, target, relation);
            }

            if self.is_tuple_type(source) && self.is_tuple_type(target) {
                return self.is_tuple_type_related_to(source, target, relation);
            }

            if let Some(result) = self.generic_type_reference_related_to(source, target, relation) {
                if result.is_true() {
                    return true;
                }
                if result.is_false() {
                    return false;
                }
            }
            return self.is_object_type_related_to(source, target, relation);
        }

        if s.contains(TypeFlags::TypeParameter)
            && t.contains(TypeFlags::TypeParameter)
            && let (Some(ss), Some(ts)) = (&source.symbol, &target.symbol)
            && Arc::ptr_eq(ss, ts)
        {
            return true;
        }

        if t.contains(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_constraint_of_type_parameter(target) {
                if self.is_type_related_to(source, &constraint, relation) {
                    return true;
                }
            }
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
        }

        if s.contains(TypeFlags::Conditional) {
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
        if t.contains(TypeFlags::Conditional) {
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

        if s.contains(TypeFlags::Object) && source.object_flags.contains(ObjectFlags::Mapped) {
            if let Some(constraint) = self.get_constraint_of_mapped_type(source) {
                if self.is_type_related_to(&constraint, target, relation) {
                    return true;
                }
            }
        }
        if t.contains(TypeFlags::Object) && target.object_flags.contains(ObjectFlags::Mapped) {
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

        false
    }

    pub(crate) fn is_array_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let source_args = self.get_type_arguments(source);
        let target_args = self.get_type_arguments(target);

        if source_args.is_empty() || target_args.is_empty() {
            return self.is_object_type_related_to(source, target, relation);
        }

        let source_elem = &source_args[0];
        let target_elem = &target_args[0];
        self.is_type_related_to(source_elem, target_elem, relation)
    }
}
