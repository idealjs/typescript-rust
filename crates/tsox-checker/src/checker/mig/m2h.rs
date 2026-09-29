#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::relater_relation::*;
use crate::checker::types::*;
use crate::checker::relater_relation::RelationComparisonResult;
use super::r19k5_flags_ext::TypeFlagsExt;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated as msg;
use tsox_frontend::ast::{self, Symbol, SymbolFlags, SyntaxKind};

#[path = "r21k10_defs.rs"]
pub mod r21k10_defs;
pub use r21k10_defs::*;

#[path = "r22k10_defs.rs"]
pub mod r22k10_defs;
pub use r22k10_defs::*;

use super::m1c_3::create_diagnostic_for_node;
use super::wc1b::{every_type, is_literal_type, is_object_literal_type};
use crate::binder::mig::m3h::get_symbol_name_for_private_identifier;
use crate::checker::relater_relation::{error_range_for_node, ChainRelated};
use crate::checker::utilities_token_is_identifier_or_keyword::is_tuple_type;

impl Checker {
    pub fn reset_maybe_stack(
        &mut self,
        maybe_start: usize,
        propagating_variance_flags: RelationComparisonResult,
        mark_all_as_succeeded: bool,
    ) {
        for i in maybe_start..r21k10_defs::maybe_keys_len() {
            let key = r21k10_defs::maybe_keys_get(i).unwrap();
            r21k10_defs::maybe_keys_set_remove(&key);
            if mark_all_as_succeeded {
                r21k10_defs::maybe_result_mark_succeeded(key, propagating_variance_flags);
                self.relation_count = self.relation_count.saturating_sub(1);
            }
        }
        r21k10_defs::maybe_keys_truncate(maybe_start);
    }

    pub fn structured_type_related_to(
        &mut self,
        relation: RelationKind,
        source: &Arc<Type>,
        target: &Arc<Type>,
        report_errors: bool,
        intersection_state: IntersectionState,
    ) -> Ternary {
        let save_error_state = self.get_error_state();
        let mut result =
            self.structured_type_related_to_worker(source, target, report_errors, intersection_state);
        if relation != RelationKind::Identity {
            if result == Ternary::False
                && (source.flags.intersects(TypeFlags::Intersection)
                    || (source.flags.intersects(TypeFlags::TypeParameter)
                        && target.flags.intersects(TypeFlags::Union)))
            {
                let source_types: Vec<Arc<Type>> = if source.flags.intersects(TypeFlags::Intersection)
                {
                    source.types().unwrap_or(&[]).to_vec()
                } else {
                    vec![Arc::clone(source)]
                };
                let constraint = self.get_effective_constraint_of_intersection(
                    &source_types,
                    target.flags.intersects(TypeFlags::Union),
                );
                if let Some(constraint) = constraint {
                    let all_distinct = every_type(&constraint, &|c| !Arc::ptr_eq(c, source));
                    if all_distinct {
                        result = self.is_related_to_ex(
                            &constraint,
                            target,
                            RecursionFlags::Source,
                            false,
                            None,
                            intersection_state,
                        );
                    }
                }
            }
            if result != Ternary::False
                && !intersection_state.intersects(IntersectionState::Target)
                && target.flags.intersects(TypeFlags::Intersection)
                && !self.is_generic_object_type(target)
                && source.flags.intersects(TypeFlags::Object | TypeFlags::Intersection)
            {
                result &= self.properties_related_to(
                    relation,
                    source,
                    target,
                    report_errors,
                    &HashSet::new(),
                    false,
                    IntersectionState::None,
                );
                if result != Ternary::False
                    && is_object_literal_type(source)
                    && source.object_flags.intersects(ObjectFlags::FreshLiteral)
                {
                    result &= self.index_signatures_related_to(
                        source,
                        target,
                        false,
                        relation,
                    );
                }
            }
            if result != Ternary::False
                && self.is_non_generic_object_type(target)
                && !self.is_array_or_tuple_type(target)
                && self.is_source_intersection_needing_extra_check(source, target)
            {
                result &= self.properties_related_to(
                    relation,
                    source,
                    target,
                    report_errors,
                    &HashSet::new(),
                    true,
                    intersection_state,
                );
            }
        }
        if result != Ternary::False {
            self.restore_error_state(save_error_state);
        }
        result
    }

    pub fn is_source_intersection_needing_extra_check(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> bool {
        source.flags.intersects(TypeFlags::Intersection)
            && self
                .get_apparent_type(source)
                .flags
                .intersects(TypeFlags::StructuredType)
            && !source
                .types()
                .unwrap_or(&[])
                .iter()
                .any(|t| Arc::ptr_eq(t, target) || t.object_flags.intersects(ObjectFlags::NonInferrableType))
    }

    pub fn report_unmatched_property(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        unmatched_property: &Arc<Symbol>,
        require_optional_properties: bool,
    ) {
        if let Some(decl) = unmatched_property.value_declaration.clone() {
            if let Some(name) = decl.name() {
                if ast::is_private_identifier(&name) {
                    if let Some(source_symbol) = &source.symbol {
                        if source_symbol.flags.intersects(SymbolFlags::Class) {
                            let private_identifier_description = name.text();
                            let symbol_table_key = get_symbol_name_for_private_identifier(
                                source_symbol,
                                &private_identifier_description.to_string(),
                            );
                            if self
                                .get_property_of_type(source, &symbol_table_key)
                                .is_some()
                            {
                                let source_symbol_name = self.symbol_to_string(source_symbol);
                                let target_symbol_name =
                                    self.symbol_to_string(target.symbol.as_ref().unwrap());
                                self.relater_report_error(
                                    msg::PROPERTY_0_IN_TYPE_1_REFERS_TO_A_DIFFERENT_MEMBER_THAT_CANNOT_BE_ACCESSED_FROM_WITHIN_TYPE_2,
                                    vec![
                                        private_identifier_description.to_string(),
                                        source_symbol_name,
                                        target_symbol_name,
                                    ],
                                );
                                return;
                            }
                        }
                    }
                }
            }
        }
        let props = self.get_unmatched_properties(source, target, require_optional_properties, false);
        if props.len() == 1 {
            let (source_type, target_type) = self.get_type_names_for_error_display(source, target);
            let prop_name = self.symbol_to_string(&props[0]);
            self.relater_report_error(
                msg::PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2,
                vec![prop_name.clone(), source_type, target_type],
            );
            if let Some(first_decl) = unmatched_property.declarations.first() {
                let related = ChainRelated {
                    file: self.get_source_file_of_node(first_decl),
                    loc: error_range_for_node(first_decl),
                    message: msg::X_0_IS_DECLARED_HERE,
                    args: vec![prop_name],
                };
                r21k10_defs::pending_related_push(related);
            }
        } else if self.try_elaborate_array_like_errors(source, target, false) {
            let (source_type, target_type) = self.get_type_names_for_error_display(source, target);
            if props.len() > 5 {
                let prop_names = props[..4]
                    .iter()
                    .map(|p| self.symbol_to_string(p))
                    .collect::<Vec<_>>()
                    .join(", ");
                self.relater_report_error(
                    msg::TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2_AND_3_MORE,
                    vec![
                        source_type,
                        target_type,
                        prop_names,
                        (props.len() - 4).to_string(),
                    ],
                );
            } else {
                let prop_names = props
                    .iter()
                    .map(|p| self.symbol_to_string(p))
                    .collect::<Vec<_>>()
                    .join(", ");
                self.relater_report_error(
                    msg::TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2,
                    vec![source_type, target_type, prop_names],
                );
            }
        }
    }

    pub fn try_elaborate_array_like_errors(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        report_errors: bool,
    ) -> bool {
        if is_tuple_type(source) {
            if source
                .target_tuple_type()
                .is_some_and(|d| d.readonly)
                && self.is_mutable_array_or_tuple(target)
            {
                if report_errors {
                    let source_name = self.type_to_string(source);
                    let target_name = self.type_to_string(target);
                    self.relater_report_error(
                        msg::THE_TYPE_0_IS_READONLY_AND_CANNOT_BE_ASSIGNED_TO_THE_MUTABLE_TYPE_1,
                        vec![source_name, target_name],
                    );
                }
                return false;
            }
            return self.is_array_or_tuple_type(target);
        }
        if self.is_readonly_array_type(source) && self.is_mutable_array_or_tuple(target) {
            if report_errors {
                let source_name = self.type_to_string(source);
                let target_name = self.type_to_string(target);
                self.relater_report_error(
                    msg::THE_TYPE_0_IS_READONLY_AND_CANNOT_BE_ASSIGNED_TO_THE_MUTABLE_TYPE_1,
                    vec![source_name, target_name],
                );
            }
            return false;
        }
        if is_tuple_type(target) {
            return self.is_array_type(source);
        }
        true
    }

    pub fn try_elaborate_errors_for_primitives_and_objects(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) {
        let global_string = self.global_string_type();
        let string_type = self.string_type();
        let global_number = self.global_number_type();
        let number_type = self.number_type();
        let global_boolean = self.global_boolean_type();
        let boolean_type = self.boolean_type();
        let global_es_symbol = self.get_global_es_symbol_type();
        let es_symbol_type = self.es_symbol_type();
        if (Arc::ptr_eq(source, &global_string) && Arc::ptr_eq(target, &string_type))
            || (Arc::ptr_eq(source, &global_number) && Arc::ptr_eq(target, &number_type))
            || (Arc::ptr_eq(source, &global_boolean) && Arc::ptr_eq(target, &boolean_type))
            || (Arc::ptr_eq(source, &global_es_symbol) && Arc::ptr_eq(target, &es_symbol_type))
        {
            let target_name = self.type_to_string(target);
            let source_name = self.type_to_string(source);
            self.relater_report_error(
                msg::X_0_IS_A_PRIMITIVE_BUT_1_IS_A_WRAPPER_OBJECT_PREFER_USING_0_WHEN_POSSIBLE,
                vec![target_name, source_name],
            );
        }
    }
}

impl Checker {
    pub fn structured_type_related_to_worker(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        report_errors: bool,
        intersection_state: IntersectionState,
    ) -> Ternary {
        let save_error_state = self.get_error_state();
        let _ = save_error_state;
        let _ = report_errors;
        let _ = intersection_state;
        todo!(
            "Go relater.go:3300 structuredTypeRelatedToWorker: relateVariances 闭包(relater.go:3302-3339)+ identity 分派(relater.go:3341-3402: union/intersection 3342-3402、index 3346、indexedAccess 3348、conditional 3356、substitution 3372、templateLiteral 3380、stringMapping 3392)+ unionOrIntersectionRelatedTo(relater.go:3404)+ alias variance 探测(relater.go:3423-3441)+ 单元素泛型元组(relater.go:3442-3455)+ TypeParameter/Object 结构分派(relater.go:3456-3680);依赖 helper union_or_intersection_related_to/members_related_to 等未移植,语义接管留语料轮"
        )
    }

    pub fn properties_related_to(
        &mut self,
        relation: RelationKind,
        source: &Arc<Type>,
        target: &Arc<Type>,
        report_errors: bool,
        excluded_properties: &HashSet<String>,
        optionals_only: bool,
        intersection_state: IntersectionState,
    ) -> Ternary {
        let _ = relation;
        let _ = source;
        let _ = target;
        let _ = report_errors;
        let _ = excluded_properties;
        let _ = optionals_only;
        let _ = intersection_state;
        todo!(
            "Go relater.go:4132 propertiesRelatedTo: identity 走 propertiesIdenticalTo(relater.go:4134)、元组分支(relater.go:4137-4248: readonly/arity/minLength/rest/variadic 全套诊断)、属性循环(relater.go:4249 起: members_related_to 4376 附近)与 propertiesIdenticalTo(4430 附近);依赖 members_related_to 等未移植,语义接管留语料轮"
        )
    }
}
