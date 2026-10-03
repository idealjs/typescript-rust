#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::relater_relation::*;
use crate::checker::types::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated as msg;
use tsox_frontend::ast::{self, Symbol, SymbolFlags, SyntaxKind};

use super::m2h::r21k10_defs;
use super::m2a::r19k11_defs::R19K11CheckerExt;
use super::wc1b::is_literal_type;
use crate::checker::mig::w6_relater::is_conversion_or_interface_implementation_message;
use crate::checker::types_type_id::TYPE_FLAGS_UNION_OR_INTERSECTION;

impl Checker {
    pub fn signature_related_to(
        &mut self,
        relation: RelationKind,
        source: &Arc<Signature>,
        target: &Arc<Signature>,
        erase: bool,
        report_errors: bool,
        intersection_state: IntersectionState,
    ) -> Ternary { ::tsox_core::fntrace::enter("signature_related_to"); 
        let mut check_mode = SignatureCheckMode::None;
        if relation == RelationKind::Subtype {
            check_mode = SignatureCheckMode::StrictTopSignature;
        } else if relation == RelationKind::StrictSubtype {
            check_mode = SignatureCheckMode::StrictTopSignature | SignatureCheckMode::StrictArity;
        }
        let source = if erase {
            self.get_erased_signature(source)
        } else {
            Arc::clone(source)
        };
        let target = if erase {
            self.get_erased_signature(&target)
        } else {
            Arc::clone(target)
        };
        self.compare_signatures_related(&source, &target, check_mode, relation)
    }

    pub fn report_relation_error(
        &mut self,
        message: Option<tsox_core::diagnostics::Message>,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) { ::tsox_core::fntrace::enter("report_relation_error"); 
        let (source_type, target_type) = self.get_type_names_for_error_display(source, target);
        let mut generalized_source = Arc::clone(source);
        let mut generalized_source_type = source_type.clone();
        if !target.flags.intersects(TypeFlags::Never)
            && is_literal_type(source)
            && !self.type_could_have_top_level_singleton_types(target)
        {
            generalized_source = self.get_base_type_of_literal_type(source);
            generalized_source_type = self.get_type_name_for_error_display(&generalized_source);
        }
        let target_flags = if target.flags.intersects(TypeFlags::IndexedAccess)
            && !source.flags.intersects(TypeFlags::IndexedAccess)
        {
            target
                .as_indexed_access_type()
                .map(|d| d.object_type.as_ref().map(|t| t.flags).unwrap_or(target.flags))
                .unwrap_or(target.flags)
        } else {
            target.flags
        };
        if target_flags.intersects(TypeFlags::TypeParameter)
            && !Arc::ptr_eq(target, &self.marker_super_type())
            && !Arc::ptr_eq(target, &self.marker_sub_type())
        {
            let constraint = self.get_base_constraint_of_type(target);
            if let Some(constraint) = &constraint {
                if self.is_type_assignable_to(&generalized_source, constraint) {
                    let constraint_name = self.type_to_string(constraint);
                    self.relater_report_error(
                        msg::X_0_IS_ASSIGNABLE_TO_THE_CONSTRAINT_OF_TYPE_1_BUT_1_COULD_BE_INSTANTIATED_WITH_A_DIFFERENT_SUBTYPE_OF_CONSTRAINT_2,
                        vec![
                            generalized_source_type.clone(),
                            target_type.clone(),
                            constraint_name,
                        ],
                    );
                } else if self.is_type_assignable_to(source, constraint) {
                    let constraint_name = self.type_to_string(constraint);
                    self.relater_report_error(
                        msg::X_0_IS_ASSIGNABLE_TO_THE_CONSTRAINT_OF_TYPE_1_BUT_1_COULD_BE_INSTANTIATED_WITH_A_DIFFERENT_SUBTYPE_OF_CONSTRAINT_2,
                        vec![
                            source_type.clone(),
                            target_type.clone(),
                            constraint_name,
                        ],
                    );
                } else {
                    self.clear_error_chain();
                    self.relater_report_error(
                        msg::X_0_COULD_BE_INSTANTIATED_WITH_AN_ARBITRARY_TYPE_WHICH_COULD_BE_UNRELATED_TO_1,
                        vec![target_type.clone(), generalized_source_type.clone()],
                    );
                }
            } else {
                self.clear_error_chain();
                self.relater_report_error(
                    msg::X_0_COULD_BE_INSTANTIATED_WITH_AN_ARBITRARY_TYPE_WHICH_COULD_BE_UNRELATED_TO_1,
                    vec![target_type.clone(), generalized_source_type.clone()],
                );
            }
        }
        let mut message = message;
        if message.is_none() {
            if r21k10_defs::current_relation_kind() == RelationKind::Comparable {
                message = Some(msg::TYPE_0_IS_NOT_COMPARABLE_TO_TYPE_1);
            } else if source_type == target_type {
                message = Some(msg::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1_TWO_DIFFERENT_TYPES_WITH_THIS_NAME_EXIST_BUT_THEY_ARE_UNRELATED);
            } else if self.exact_optional_property_types
                && !self
                    .get_exact_optional_unassignable_properties(source, target)
                    .is_empty()
            {
                message = Some(msg::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1_WITH_EXACTOPTIONALPROPERTYTYPES_COLON_TRUE_CONSIDER_ADDING_UNDEFINED_TO_THE_TYPES_OF_THE_TARGET_S_PROPERTIES);
            } else {
                if source.flags.intersects(TypeFlags::StringLiteral)
                    && target.flags.intersects(TypeFlags::Union)
                {
                    if let Some(suggested_type) = self
                        .get_suggested_type_for_nonexistent_string_literal_type(source, target)
                    {
                        let suggested_name = self.type_to_string(&suggested_type);
                        self.relater_report_error(
                            msg::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1_DID_YOU_MEAN_2,
                            vec![
                                generalized_source_type.clone(),
                                target_type.clone(),
                                suggested_name,
                            ],
                        );
                        return;
                    }
                }
                message = Some(msg::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1);
            }
        } else if message == Some(msg::ARGUMENT_OF_TYPE_0_IS_NOT_ASSIGNABLE_TO_PARAMETER_OF_TYPE_1)
            && self.exact_optional_property_types
            && !self
                .get_exact_optional_unassignable_properties(source, target)
                .is_empty()
        {
            message = Some(msg::ARGUMENT_OF_TYPE_0_IS_NOT_ASSIGNABLE_TO_PARAMETER_OF_TYPE_1_WITH_EXACTOPTIONALPROPERTYTYPES_COLON_TRUE_CONSIDER_ADDING_UNDEFINED_TO_THE_TYPES_OF_THE_TARGET_S_PROPERTIES);
        }
        match self.get_chain_message(0) {
            m if m == Some(msg::OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1)
                || m == Some(msg::OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_BUT_0_DOES_NOT_EXIST_IN_TYPE_1_DID_YOU_MEAN_TO_WRITE_2) =>
            {
                return;
            }
            m if m == Some(msg::EXCESSIVE_COMPLEXITY_COMPARING_TYPES_0_AND_1)
                || m == Some(msg::THE_TYPE_0_IS_READONLY_AND_CANNOT_BE_ASSIGNED_TO_THE_MUTABLE_TYPE_1) =>
            {
                if self.chain_args_match(&[generalized_source_type.clone(), target_type.clone()]) {
                    return;
                }
            }
            m if m == Some(msg::PROPERTY_0_IS_MISSING_IN_TYPE_1_BUT_REQUIRED_IN_TYPE_2) => {
                if !message
                    .is_some_and(|m| is_conversion_or_interface_implementation_message(&m))
                    && self.chain_args_match(&[generalized_source_type.clone(), target_type.clone()])
                {
                    return;
                }
            }
            m if m == Some(msg::TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2_AND_3_MORE)
                || m == Some(msg::TYPE_0_IS_MISSING_THE_FOLLOWING_PROPERTIES_FROM_TYPE_1_COLON_2) =>
            {
                if !message
                    .is_some_and(|m| is_conversion_or_interface_implementation_message(&m))
                    && self.chain_args_match(&[generalized_source_type.clone(), target_type.clone()])
                {
                    return;
                }
            }
            _ => {}
        }
        self.relater_report_error(message.unwrap(), vec![generalized_source_type, target_type]);
    }

    pub fn trace_unions_or_intersections_too_large(&mut self, source: &Arc<Type>, target: &Arc<Type>) { ::tsox_core::fntrace::enter("trace_unions_or_intersections_too_large"); 
        let tr = self.tracer.clone();
        if !tr.is_enabled() {
            return;
        }
        if source.flags.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
            && target.flags.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
        {
            if source
                .object_flags
                .intersection(target.object_flags)
                .intersects(ObjectFlags::PrimitiveUnion)
            {
                return;
            }
            let source_size = source.types().map(|t| t.len()).unwrap_or(0);
            let target_size = target.types().map(|t| t.len()).unwrap_or(0);
            if source_size * target_size > 1_000_000 {
                let _instant = tr.start("traceUnionsOrIntersectionsTooLarge_DepthLimit");
            }
        }
    }
}
