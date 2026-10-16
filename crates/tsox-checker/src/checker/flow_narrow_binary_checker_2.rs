#![allow(unused_imports)]

use crate::checker::flow_narrow_binary::*;

impl Checker {
    pub(crate) fn replace_primitives_with_literals(
        &mut self,
        type_: &Arc<Type>,
        value_type: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("replace_primitives_with_literals"); 
        let has_primitives = type_
            .flags
            .intersects(TypeFlags::String | TypeFlags::Number | TypeFlags::BigInt);
        let has_literals = value_type
            .flags
            .intersects(TYPE_FLAGS_LITERAL | TypeFlags::TemplateLiteral | TypeFlags::StringMapping);
        if !has_primitives || !has_literals {
            return Arc::clone(type_);
        }

        let value_constituents = self.constituent_types(value_type);
        let string_literals: Vec<Arc<Type>> = value_constituents
            .iter()
            .filter(|t| {
                t.flags.intersects(
                    TypeFlags::StringLiteral
                        | TypeFlags::TemplateLiteral
                        | TypeFlags::StringMapping,
                )
            })
            .cloned()
            .collect();
        let number_literals: Vec<Arc<Type>> = value_constituents
            .iter()
            .filter(|t| t.flags.contains(TypeFlags::NumberLiteral))
            .cloned()
            .collect();
        let bigint_literals: Vec<Arc<Type>> = value_constituents
            .iter()
            .filter(|t| t.flags.contains(TypeFlags::BigIntLiteral))
            .cloned()
            .collect();
        let constituents = self.constituent_types(type_);
        let mut result: Vec<Arc<Type>> = Vec::new();
        for t in constituents {
            if t.flags.contains(TypeFlags::String) {
                let has_string_value =
                    value_type.flags.contains(TypeFlags::String) || string_literals.is_empty();
                if has_string_value {
                    result.push(t);
                } else {
                    result.extend(string_literals.iter().cloned());
                }
            } else if t.flags.contains(TypeFlags::Number) {
                let has_number_value =
                    value_type.flags.contains(TypeFlags::Number) || number_literals.is_empty();
                if has_number_value {
                    result.push(t);
                } else {
                    result.extend(number_literals.iter().cloned());
                }
            } else if t.flags.contains(TypeFlags::BigInt) {
                let has_bigint_value =
                    value_type.flags.contains(TypeFlags::BigInt) || bigint_literals.is_empty();
                if has_bigint_value {
                    result.push(t);
                } else {
                    result.extend(bigint_literals.iter().cloned());
                }
            } else {
                result.push(t);
            }
        }
        self.rebuild_union_or_never(type_, result)
    }

    pub(crate) fn is_coercible_under_double_equals(source: &Arc<Type>, target: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_coercible_under_double_equals"); 
        source
            .flags
            .intersects(TypeFlags::Number | TypeFlags::String | TypeFlags::BooleanLiteral)
            && target
                .flags
                .intersects(TypeFlags::Number | TypeFlags::String | TypeFlags::Boolean)
    }

    pub(crate) fn narrow_type_by_instanceof(
        &mut self,
        type_: &Arc<Type>,
        expr: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_type_by_instanceof");
        let NodeData::BinaryExpression(bin) = &expr.data else {
            return Arc::clone(type_);
        };
        let assume_true = matches!(kind, NarrowKind::TrueBranch);
        let left = self.get_reference_candidate(&bin.left);
        if !self.expr_matches_target(&left, target) {
            if assume_true
                && self.strict_null_checks
                && self.optional_chain_contains_target(&left, target)
            {
                return self.get_adjusted_type_with_facts(type_, TypeFacts::NE_UNDEFINED_OR_NULL);
            }
            return Arc::clone(type_);
        }
        let right_type = self.get_type_of_node(&bin.right);
        if let Some(global_object) = self.get_global_type_by_name("Object") {
            if !self.is_type_derived_from(&right_type, &global_object) {
                return Arc::clone(type_);
            }
        }
        if let Some(signature) = self.get_effects_signature(expr) {
            if let Some(predicate) = self.get_type_predicate_of_signature(&signature) {
                if predicate.kind == TypePredicateKind::Identifier
                    && predicate.parameter_index == 0
                    && let Some(predicate_t) = &predicate.t
                {
                    return self.get_narrowed_type(type_, predicate_t, assume_true, true);
                }
            }
        }
        if let Some(global_function) = self.get_global_type_by_name("Function") {
            if !self.is_type_derived_from(&right_type, &global_function) {
                return Arc::clone(type_);
            }
        }
        let Some(instance_type) = self.get_instance_type_of_constructor(&right_type) else {
            return Arc::clone(type_);
        };
        let is_global_object = self
            .get_global_type_by_name("Object")
            .is_some_and(|g| Arc::ptr_eq(&instance_type, &g));
        let is_global_function = self
            .get_global_type_by_name("Function")
            .is_some_and(|g| Arc::ptr_eq(&instance_type, &g));
        if type_.flags.intersects(TypeFlags::Any) && (is_global_object || is_global_function)
            || !assume_true
                && !(instance_type.flags.contains(TypeFlags::Object)
                    && !self.is_empty_anonymous_object_type(&instance_type))
        {
            return Arc::clone(type_);
        }
        self.get_narrowed_type(type_, &instance_type, assume_true, true)
    }

    pub(crate) fn narrow_by_in_keyword(
        &mut self,
        type_: &Arc<Type>,
        left: &Arc<Node>,
        right: &Arc<Node>,
        target: &FlowRef,
        kind: NarrowKind,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("narrow_by_in_keyword"); 
        if type_.flags.contains(TypeFlags::Any) {
            return Arc::clone(type_);
        }
        if !self.expr_matches_target(right, target) {
            return Arc::clone(type_);
        }
        let Some(prop_name) = Self::get_accessed_property_name_from_node(left) else {
            return Arc::clone(type_);
        };
        let keep_present = match kind {
            NarrowKind::TrueBranch => true,
            NarrowKind::FalseBranch => false,
        };

        let constituents = self.constituent_types(type_);
        let filtered: Vec<Arc<Type>> = constituents
            .into_iter()
            .filter(|t| {
                let has_prop = self.type_has_property(t, &prop_name);
                if keep_present {
                    !has_prop.is_definitely_not()
                } else {
                    !has_prop.is_definitely()
                }
            })
            .collect();
        self.rebuild_union_or_never(type_, filtered)
    }
}
