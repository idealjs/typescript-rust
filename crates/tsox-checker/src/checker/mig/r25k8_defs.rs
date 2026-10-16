#![allow(unused_imports)]

use std::sync::Arc;

use tsox_frontend::ast::mig::m3f_2::has_inferred_type;
use tsox_frontend::ast::{
    is_parameter_declaration, is_property_declaration, is_property_signature_declaration,
    node_is_synthesized, Node, NodeData,
};

use crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr;
use crate::checker::mig::m2g::r22k9_defs::NodeBuilderPseudoExt22;
use crate::checker::nodecopy_builder::{NodeBuilderImpl, PseudoCheckerStub};
use crate::checker::types::{Type, TypeFlags, TypeFacts, OBJECT_FLAGS_REQUIRES_WIDENING};
use crate::checker::types::Ternary;
use crate::checker::utilities_has_only_expression_initialization::is_optional_declaration;

pub fn new_pseudo_checker(
    _strict_null_checks: bool,
    _exact_optional_property_types: bool,
) -> PseudoCheckerStub { ::tsox_core::fntrace::enter("new_pseudo_checker");
    PseudoCheckerStub
}

pub fn get_type_of_declaration_direct_annotation(declaration: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("typeFromParameter");
    if !is_parameter_declaration(declaration) {
        return None;
    }
    match &declaration.data {
        NodeData::ParameterDeclaration(p) => {
            if p.initializer.is_some() {
                return None;
            }
            p.type_node.clone()
        }
        _ => None,
    }
}

fn serialize_reuse_gate_open(
    nb: &mut NodeBuilderImpl,
    declaration: &Node,
    t: &Type,
    try_reuse: bool,
) -> bool {
    try_reuse
        && !nb.is_actively_expanding()
        && nb.ctx.borrow().enclosing_declaration.is_some()
        && has_inferred_type(declaration)
        && !node_is_synthesized(declaration)
        && !t.object_flags.intersects(OBJECT_FLAGS_REQUIRES_WIDENING)
}

fn pseudo_direct_equivalent_to_type(
    annotation: &Arc<Node>,
    t: &Arc<Type>,
    is_optional_annotated: bool,
) -> bool { ::tsox_core::fntrace::enter("pseudoTypeEquivalentToType");
    let ch = unsafe { &mut *builder_checker_ptr() };
    if ch.is_error_type(t) {
        return true;
    }
    let type_from_pseudo = ch.get_type_from_type_node(annotation);
    if Arc::ptr_eq(&type_from_pseudo, t) {
        return true;
    }
    let undefined_stripped = if is_optional_annotated {
        ch.get_type_with_facts(t, TypeFacts::NE_UNDEFINED)
    } else {
        Arc::clone(t)
    };
    if is_optional_annotated && Arc::ptr_eq(&undefined_stripped, &type_from_pseudo) {
        return true;
    }
    if is_optional_annotated
        && type_from_pseudo.flags.contains(TypeFlags::Union)
        && undefined_stripped.flags.contains(TypeFlags::Union)
        && ch.compare_types_identical(&type_from_pseudo, &undefined_stripped) == Ternary::True
    {
        return true;
    }
    if Arc::ptr_eq(
        &ch.get_regular_type_of_literal_type(&type_from_pseudo),
        &ch.get_regular_type_of_literal_type(t),
    ) {
        return true;
    }
    type_from_pseudo.flags.contains(TypeFlags::Union)
        && t.flags.contains(TypeFlags::Union)
        && ch.compare_types_identical(&type_from_pseudo, t) == Ternary::True
}

fn can_reuse_existing_js_type_node(
    nb: &mut NodeBuilderImpl,
    existing: &Arc<Node>,
    t: &Arc<Type>,
) -> bool { ::tsox_core::fntrace::enter("canReuseExistingJSTypeNode");
    let ch = unsafe { &mut *builder_checker_ptr() };
    ch.get_intended_type_from_jsdoc_type_reference(existing).is_none()
        && nb.existing_type_node_is_not_reference_or_is_reference_with_compatible_type_argument_count(existing, t)
}

pub fn serialize_type_for_declaration_with_pseudo_reuse(
    nb: &mut NodeBuilderImpl,
    declaration: Option<&Arc<Node>>,
    t: &Arc<Type>,
    try_reuse: bool,
) -> Option<Arc<Node>> {
    let declaration = declaration?;
    if !serialize_reuse_gate_open(nb, declaration, t, try_reuse) {
        return None;
    }
    let annotation = get_type_of_declaration_direct_annotation(declaration)?;
    let is_optional_annotated = (is_parameter_declaration(declaration)
        || is_property_signature_declaration(declaration)
        || is_property_declaration(declaration))
        && is_optional_declaration(declaration);
    if !pseudo_direct_equivalent_to_type(&annotation, t, is_optional_annotated) {
        return None;
    }
    if !can_reuse_existing_js_type_node(nb, &annotation, t) {
        return None;
    }
    let pt = crate::checker::mig::r27k_defs::new_pseudo_type_direct(Arc::clone(&annotation));
    Some(nb.pseudo_type_to_node_with_checker_fallback(Some(&pt), t))
}
