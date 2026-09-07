#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn is_simple_type_identical_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> bool {
        match (&source.data, &target.data) {
            (TypeData::Intrinsic(s), TypeData::Intrinsic(t)) => {
                s.intrinsic_name == t.intrinsic_name
            }
            (TypeData::Literal(s), TypeData::Literal(t)) => s.value == t.value,
            (TypeData::TypeParameter(s), TypeData::TypeParameter(t)) => {
                s.is_this_type == t.is_this_type
            }

            (TypeData::IndexedAccess(s), TypeData::IndexedAccess(t)) => {
                match (&s.object_type, &t.object_type, &s.index_type, &t.index_type) {
                    (Some(so), Some(to), Some(si), Some(ti)) => {
                        self.is_type_identical_to(so, to) && self.is_type_identical_to(si, ti)
                    }
                    _ => Arc::ptr_eq(source, target),
                }
            }

            (TypeData::Index(s), TypeData::Index(t)) => match (&s.target, &t.target) {
                (Some(so), Some(to)) => self.is_type_identical_to(so, to),
                _ => Arc::ptr_eq(source, target),
            },
            _ => source.flags == target.flags,
        }
    }

    pub fn is_simple_type_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        let s = source.flags;
        let t = target.flags;

        if t.contains(TypeFlags::Any) || s.contains(TypeFlags::Never) {
            return true;
        }

        if t.contains(TypeFlags::Unknown)
            && !(relation == RelationKind::StrictSubtype && s.contains(TypeFlags::Any))
        {
            return true;
        }

        if t.contains(TypeFlags::Never) {
            return false;
        }

        if s.intersects(TYPE_FLAGS_STRING_LIKE) && t.contains(TypeFlags::String) {
            return true;
        }

        if s.contains(TypeFlags::StringLiteral)
            && s.contains(TypeFlags::EnumLiteral)
            && t.contains(TypeFlags::StringLiteral)
            && !t.contains(TypeFlags::EnumLiteral)
            && self.literal_values_equal(source, target)
        {
            return true;
        }

        if s.intersects(TYPE_FLAGS_LITERAL)
            && t.intersects(TYPE_FLAGS_LITERAL)
            && (s & TYPE_FLAGS_LITERAL) == (t & TYPE_FLAGS_LITERAL)
            && self.literal_values_equal(source, target)
        {
            return true;
        }

        if s.intersects(TYPE_FLAGS_NUMBER_LIKE) && t.contains(TypeFlags::Number) {
            return true;
        }

        if s.contains(TypeFlags::NumberLiteral)
            && s.contains(TypeFlags::EnumLiteral)
            && t.contains(TypeFlags::NumberLiteral)
            && !t.contains(TypeFlags::EnumLiteral)
            && self.literal_values_equal(source, target)
        {
            return true;
        }

        if s.intersects(TYPE_FLAGS_BIG_INT_LIKE) && t.contains(TypeFlags::BigInt) {
            return true;
        }

        if s.intersects(TYPE_FLAGS_BOOLEAN_LIKE) && t.contains(TypeFlags::Boolean) {
            return true;
        }

        if s.intersects(TYPE_FLAGS_ES_SYMBOL_LIKE) && t.contains(TypeFlags::ESSymbol) {
            return true;
        }

        if s.contains(TypeFlags::Enum)
            && t.contains(TypeFlags::Enum)
            && self.is_enum_type_related_to(source, target)
        {
            return true;
        }

        if s.contains(TypeFlags::EnumLiteral)
            && t.contains(TypeFlags::EnumLiteral)
            && s.intersects(TYPE_FLAGS_LITERAL)
            && t.intersects(TYPE_FLAGS_LITERAL)
            && self.literal_values_equal(source, target)
            && self.is_enum_type_related_to(source, target)
        {
            return true;
        }

        if s.contains(TypeFlags::Undefined)
            && (!self.strict_null_checks && !t.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
                || t.intersects(TypeFlags::Undefined | TypeFlags::Void))
        {
            return true;
        }

        if s.contains(TypeFlags::Null)
            && (!self.strict_null_checks && !t.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
                || t.contains(TypeFlags::Null))
        {
            return true;
        }

        if s.contains(TypeFlags::Object)
            && t.contains(TypeFlags::NonPrimitive)
            && !(relation == RelationKind::StrictSubtype)
        {
            return true;
        }

        if s.contains(TypeFlags::NonPrimitive)
            && t.contains(TypeFlags::NonPrimitive)
            && source.intrinsic_name() == target.intrinsic_name()
        {
            return true;
        }

        if relation == RelationKind::Assignable || relation == RelationKind::Comparable {
            if s.contains(TypeFlags::Any) {
                return true;
            }

            if s.contains(TypeFlags::Number)
                && (t.contains(TypeFlags::Enum)
                    || (t.contains(TypeFlags::NumberLiteral) && t.contains(TypeFlags::EnumLiteral)))
            {
                return true;
            }

            if s.contains(TypeFlags::NumberLiteral)
                && !s.contains(TypeFlags::EnumLiteral)
                && (t.contains(TypeFlags::Enum)
                    || (t.contains(TypeFlags::NumberLiteral)
                        && t.contains(TypeFlags::EnumLiteral)
                        && self.literal_values_equal(source, target)))
            {
                return true;
            }

            if self.is_unknown_like_union_type(target) {
                return true;
            }
        }

        false
    }

    pub(crate) fn literal_values_equal(&self, a: &Arc<Type>, b: &Arc<Type>) -> bool {
        match (&a.data, &b.data) {
            (TypeData::Literal(la), TypeData::Literal(lb)) => la.value == lb.value,
            _ => false,
        }
    }

    pub(crate) fn erase_bare_generic_params(
        &mut self,
        owner: &Arc<Type>,
        member_type: &Arc<Type>,
    ) -> Arc<Type> {
        let Some(sym) = owner.symbol.as_ref() else {
            return Arc::clone(member_type);
        };
        if owner
            .as_object()
            .is_some_and(|o| !o.type_arguments.is_empty())
        {
            return Arc::clone(member_type);
        }
        let tps = self.declared_type_parameter_types(sym);
        if tps.is_empty() {
            return Arc::clone(member_type);
        }
        let anys: Vec<Arc<Type>> = std::iter::repeat(self.get_any_type())
            .take(tps.len())
            .collect();
        self.substitute_infer_type_parameters(member_type, &tps, &anys)
    }
}
