#![allow(unused_imports)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use tsox_frontend::ast::node_data_generated::{is_call_expression, is_property_access_expression};
use tsox_frontend::ast::Node;

use crate::checker::checker_checker::*;
use crate::checker::checker_checker_checker::Checker;
use crate::checker::types_impl_chunk_3::{CompositeSignature, Signature};

thread_local! {
    static SIGNATURE_COMPOSITES: RefCell<HashMap<usize, Arc<CompositeSignature>>> =
        RefCell::new(HashMap::new());
}

pub(crate) fn signature_composite(sig: &Arc<Signature>) -> Option<Arc<CompositeSignature>> { ::tsox_core::fntrace::enter("signature_composite"); 
    SIGNATURE_COMPOSITES.with(|m| m.borrow().get(&(Arc::as_ptr(sig) as usize)).cloned())
}

pub(crate) fn set_signature_composite(sig: &Arc<Signature>, composite: CompositeSignature) { ::tsox_core::fntrace::enter("set_signature_composite"); 
    SIGNATURE_COMPOSITES.with(|m| {
        m.borrow_mut()
            .insert(Arc::as_ptr(sig) as usize, Arc::new(composite))
    });
}

pub(crate) fn error_node_for_call_node_arc(node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("error_node_for_call_node_arc"); 
    if is_call_expression(node) {
        if let Some(expr) = node.expression() {
            if is_property_access_expression(expr) {
                if let Some(name) = expr.name() {
                    return Arc::clone(name);
                }
            }
            return Arc::clone(expr);
        }
    }
    Arc::clone(node)
}

pub(crate) fn pattern_for_type_of(t: &Arc<crate::checker::types::Type>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("pattern_for_type_of"); 
    PATTERN_FOR_TYPES.with(|m| m.borrow().get(&(Arc::as_ptr(t) as usize)).cloned())
}

pub(crate) fn set_pattern_for_type(t: &Arc<crate::checker::types::Type>, pattern: Arc<Node>) { ::tsox_core::fntrace::enter("set_pattern_for_type"); 
    PATTERN_FOR_TYPES.with(|m| {
        m.borrow_mut()
            .insert(Arc::as_ptr(t) as usize, pattern)
    });
}

thread_local! {
    static PATTERN_FOR_TYPES: RefCell<HashMap<usize, Arc<Node>>> = RefCell::new(HashMap::new());
}

pub(crate) fn is_js_literal_type(checker: &mut Checker, t: &Arc<crate::checker::types::Type>) -> bool { ::tsox_core::fntrace::enter("is_js_literal_type"); 
    use crate::checker::types::{TypeFlags, TYPE_FLAGS_INSTANTIABLE};
    if checker.no_implicit_any {
        return false;
    }
    if t.object_flags.intersects(crate::checker::types::ObjectFlags::JSLiteral) {
        return true;
    }
    if t.flags.intersects(TypeFlags::Union) {
        if let Some(types) = t.types() {
            return types.iter().all(|u| is_js_literal_type(checker, u));
        }
    }
    if t.flags.intersects(TypeFlags::Intersection) {
        if let Some(types) = t.types() {
            return types.iter().any(|u| is_js_literal_type(checker, u));
        }
    }
    if t.flags.intersects(TYPE_FLAGS_INSTANTIABLE) {
        let constraint = checker.get_resolved_base_constraint(t, &[]);
        return !Arc::ptr_eq(&constraint, t) && is_js_literal_type(checker, &constraint);
    }
    false
}

pub trait R24K17FactoryExt {
    fn new_element_access_expression(
        &self,
        expression: &Arc<Node>,
        question_dot_token: Option<&Arc<Node>>,
        argument_expression: &Arc<Node>,
        flags: tsox_frontend::ast::NodeFlags,
    ) -> Arc<Node>;
}

impl R24K17FactoryExt for tsox_frontend::ast::mig::m3c::NodeFactory {
    fn new_element_access_expression(
        &self,
        expression: &Arc<Node>,
        question_dot_token: Option<&Arc<Node>>,
        argument_expression: &Arc<Node>,
        flags: tsox_frontend::ast::NodeFlags,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_element_access_expression"); 
        let mut node = Node::new(
            tsox_frontend::ast::SyntaxKind::ElementAccessExpression,
            tsox_frontend::ast::NodeData::ElementAccessExpression(
                tsox_frontend::ast::node_data_generated::ElementAccessExpressionData {
                    expression: Arc::clone(expression),
                    question_dot_token: question_dot_token.cloned(),
                    argument_expression: Arc::clone(argument_expression),
                },
            ),
        );
        node.flags |= flags & tsox_frontend::ast::NodeFlags::OptionalChain;
        Arc::new(node)
    }
}

pub(crate) fn is_property_initialized_in_constructor(
    checker: &mut Checker,
    prop_name: &Arc<Node>,
    prop_type: &Arc<crate::checker::types::Type>,
    constructor: &Arc<Node>,
) -> bool { ::tsox_core::fntrace::enter("is_property_initialized_in_constructor"); 
    use tsox_frontend::ast::SyntaxKind;
    use crate::checker::mig::m2h::r22k10_defs::{set_flow_node_of, R22K10FactoryExt};
    use tsox_frontend::ast::mig::m3c::NodeFactory;
    use tsox_frontend::ast::node_data_generated::is_computed_property_name;

    let reference = if is_computed_property_name(prop_name) {
        let this_keyword = checker.factory.new_keyword_expression(SyntaxKind::ThisKeyword);
        let prop_expr = prop_name.expression().unwrap();
        R24K17FactoryExt::new_element_access_expression(
            &checker.factory,
            &this_keyword,
            None,
            &prop_expr,
            tsox_frontend::ast::NodeFlags::empty(),
        )
    } else {
        let this_keyword = checker.factory.new_keyword_expression(SyntaxKind::ThisKeyword);
        checker.factory.new_property_access_expression(
            &this_keyword,
            None,
            prop_name,
            tsox_frontend::ast::NodeFlags::empty(),
        )
    };
    reference.expression().unwrap().set_parent(&reference);
    reference.set_parent(constructor);
    set_flow_node_of(
        &reference,
        checker
            .program
            .symbol_map()
            .end_flow_node_of(constructor)
            .cloned(),
    );
    let optional_prop_type = checker.get_optional_type(prop_type.clone());
    let flow_type = checker.get_flow_type_of_reference_ex(&reference, prop_type, Some(&optional_prop_type), None);
    !checker.contains_undefined_type(&flow_type)
}
