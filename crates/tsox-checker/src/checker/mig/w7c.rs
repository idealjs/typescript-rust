#![allow(dead_code, unused_imports, unused_variables)]
use crate::checker::relater_relation::RECURSION_FLAGS_BOTH;

use std::sync::Arc;

use super::x3::TracedTypeAdapter;
use crate::checker::checker_checker_checker::Checker;
use crate::checker::mig::m2d_3::FlowState;
use crate::checker::mig::wc2_3::r29k1_defs;
use crate::checker::mig::wc3::NodeAccessExt;
use crate::checker::nodecopy_builder::NodeBuilderImpl;
use super::r27k_defs::{MappedType, PseudoType, PseudoTypeKind};
use crate::checker::*;
use tsox_frontend::ast::{self, *};
use tsox_frontend::ast::is_computed_property_name;
use tsox_frontend::scanner::TOKEN_FLAGS_SINGLE_QUOTE;

pub(crate) type Relater = Checker;

pub fn is_infinity_or_nan_string(name: &str) -> bool {
    name == "Infinity" || name == "-Infinity" || name == "NaN"
}

pub fn is_structural_pseudo_type(t: Option<&PseudoType>) -> bool {
    let Some(t) = t else {
        return false;
    };
    match t.kind {
        PseudoTypeKind::ObjectLiteral | PseudoTypeKind::Tuple | PseudoTypeKind::SingleCallSignature => true,
        PseudoTypeKind::MaybeConstLocation => {
            let d = t.as_pseudo_type_maybe_const_location();
            is_structural_pseudo_type(d.const_type.as_deref())
                || is_structural_pseudo_type(d.regular_type.as_deref())
        }
        _ => false,
    }
}

impl<'a> NodeBuilderImpl<'a> {
    pub fn is_mapped_type_homomorphic(&mut self, mapped: &Arc<Type>) -> bool {
        let ch_ptr = self.ch as *const Checker as *mut Checker;
        unsafe { (*ch_ptr).get_homomorphic_type_variable(mapped) }.is_some()
    }

    pub fn is_homomorphic_mapped_type_with_non_homomorphic_instantiation(
        &mut self,
        mapped: &MappedType,
    ) -> bool {
        mapped.target.is_some()
            && !self.is_mapped_type_homomorphic(&mapped.ty)
            && self
                .mapped_type_target(mapped)
                .is_some_and(|t| self.is_mapped_type_homomorphic(&t))
    }

    pub fn is_string_named(&mut self, d: &Arc<Node>) -> bool {
        let Some(name) = get_name_of_declaration(d) else {
            return false;
        };
        if is_computed_property_name(&name) {
            let Some(expr) = name.expression() else {
                return false;
            };
            let t = self.ch_check_expression_type(&expr);
            return t.flags.intersects(TypeFlags::STRING_LIKE);
        }
        if is_element_access_expression(&name) {
            let argument = name
                .as_element_access_expression()
                .argument_expression
                .clone();
            let t = self.ch_check_expression_type(&argument);
            return t.flags.intersects(TypeFlags::STRING_LIKE);
        }
        is_string_literal(&name)
    }

    pub fn is_single_quoted_string_named(&mut self, d: &Arc<Node>) -> bool {
        get_name_of_declaration(d).is_some_and(|name| {
            match &name.data {
                NodeData::StringLiteral(d) => {
                    d.token_flags & TOKEN_FLAGS_SINGLE_QUOTE != 0
                }
                _ => false,
            }
        })
    }
}

impl<'a> NodeBuilderImpl<'a> {
    fn ch_check_expression_type(&mut self, expr: &Arc<Node>) -> Arc<Type> {
        let ch_ptr = self.ch as *const Checker as *mut Checker;
        unsafe { (*ch_ptr).get_type_of_expression(expr) }
    }
}

impl TracedTypeAdapter {
    pub fn is_tuple(&self) -> bool {
        let t = unsafe { &*self.t };
        t.object_flags.intersects(ObjectFlags::Tuple)
    }
}

impl Checker {
    pub fn is_matching_constructor_reference(&self, f: &FlowState, expr: &Node) -> bool {
        let name = if is_property_access_expression(expr) {
            Some(expr.as_property_access_expression().name.clone())
        } else if is_element_access_expression(expr)
            && is_string_literal_like(
                &expr.as_element_access_expression().argument_expression,
            )
        {
            Some(
                expr.as_element_access_expression()
                    .argument_expression
                    .clone(),
            )
        } else {
            None
        };
        match name {
            Some(name) => {
                name.text() == "constructor"
                    && expr.expression().is_some_and(|e| {
                        f.reference
                            .as_ref()
                            .is_some_and(|r| self.is_matching_reference(r, &e))
                    })
            }
            None => false,
        }
    }

    pub fn is_post_super_flow_node_worker(
        &self,
        f: &mut FlowState,
        flow: Option<Arc<FlowNode>>,
        no_cache_check: bool,
    ) -> bool {
        let mut flow = flow;
        let mut no_cache_check = no_cache_check;
        loop {
            let Some(current) = flow.clone() else {
                return false;
            };
            let flags = current.flags;
            if flags.intersects(FlowFlags::SHARED) {
                if !no_cache_check {
                    if let Some(post_super) = r29k1_defs::flow_node_post_super_get(&current) {
                        return post_super;
                    }
                    let post_super = self.is_post_super_flow_node_worker(f, flow.clone(), true);
                    r29k1_defs::flow_node_post_super_insert(current.clone(), post_super);
                }
                no_cache_check = false;
            }
            if flags.intersects(
                FlowFlags::ASSIGNMENT
                    | FlowFlags::CONDITION
                    | FlowFlags::ARRAY_MUTATION
                    | FlowFlags::SWITCH_CLAUSE,
            ) {
                flow = current.antecedent.clone();
            } else if flags.intersects(FlowFlags::CALL) {
                if current
                    .node
                    .as_ref()
                    .and_then(|n| n.expression())
                    .is_some_and(|e| e.kind == SyntaxKind::SuperKeyword)
                {
                    return true;
                }
                flow = current.antecedent.clone();
            } else if flags.intersects(FlowFlags::BRANCH_LABEL) {
                for a in crate::checker::mig::w6_flow::get_branch_label_antecedents(
                    &current,
                    &f.reduce_labels,
                ) {
                    if !self.is_post_super_flow_node_worker(f, Some(a), false) {
                        return false;
                    }
                }
                return true;
            } else if flags.intersects(FlowFlags::LOOP_LABEL) {
                flow = current.antecedents.first().cloned();
            } else if flags.intersects(FlowFlags::REDUCE_LABEL) {
                if let Some(node) = &current.node {
                    let reduce_data = tsox_frontend::ast::mig::x1a::as_flow_reduce_label_data(node);
                    f.reduce_labels.push(Arc::new(
                        tsox_frontend::ast::mig::m3e::FlowReduceLabelData {
                            target: Arc::clone(&reduce_data.target),
                            antecedents: reduce_data.antecedents.clone(),
                        },
                    ));
                }
                let result =
                    self.is_post_super_flow_node_worker(f, current.antecedent.clone(), false);
                f.reduce_labels.pop();
                return result;
            } else {
                return flags.intersects(FlowFlags::UNREACHABLE);
            }
        }
    }
}

impl Relater {
    fn is_related_to_ex_via_core(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> Ternary {
        if self.is_type_related_to(
            source,
            target,
            crate::checker::relater_relation::RelationKind::Assignable,
        ) {
            Ternary::True
        } else {
            Ternary::False
        }
    }

    pub fn is_related_to_simple(&mut self, source: &Arc<Type>, target: &Arc<Type>) -> Ternary {
        self.is_related_to_ex_via_core(source, target)
    }

    pub fn is_related_to_worker(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        report_errors: bool,
    ) -> Ternary {
        let _ = report_errors;
        self.is_related_to_ex_via_core(source, target)
    }

    pub fn is_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        recursion_flags: RecursionFlags,
        report_errors: bool,
    ) -> Ternary {
        let _ = recursion_flags;
        self.is_related_to_ex_via_core(source, target)
    }
}
