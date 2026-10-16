#![allow(unused_imports)]

use crate::checker::mig::wc3::CallState;
use crate::checker::types_impl_chunk_3::Signature;
use std::sync::{Arc, OnceLock};
use tsox_frontend::ast::{Node, SyntaxKind};

fn clone_resolved_return_type(l: &OnceLock<Arc<crate::checker::types::Type>>) -> OnceLock<Arc<crate::checker::types::Type>> { ::tsox_core::fntrace::enter("clone_resolved_return_type"); 
    let out = OnceLock::new();
    if let Some(v) = l.get() {
        let _ = out.set(Arc::clone(v));
    }
    out
}

impl Clone for Signature {
    fn clone(&self) -> Self { ::tsox_core::fntrace::enter("clone"); 
        Self {
            id: self.id,
            flags: self.flags,
            min_argument_count: self.min_argument_count,
            resolved_min_argument_count: self.resolved_min_argument_count,
            declaration: self.declaration.clone(),
            type_parameters: self.type_parameters.clone(),
            parameters: self.parameters.clone(),
            this_parameter: self.this_parameter.clone(),
            resolved_return_type: clone_resolved_return_type(&self.resolved_return_type),
            resolved_type_predicate: self.resolved_type_predicate.clone(),
            target: self.target.clone(),
            mapper: self.mapper.clone(),
            isolated_signature_type: clone_resolved_return_type(&self.isolated_signature_type),
            instantiated_parameter_types: self.instantiated_parameter_types.clone(),
        }
    }
}

impl Clone for CallState {
    fn clone(&self) -> Self { ::tsox_core::fntrace::enter("clone"); 
        Self {
            node: self.node.clone(),
            args: self.args.clone(),
            type_arguments: self.type_arguments.clone(),
            candidates: self.candidates.clone(),
            is_single_non_generic_candidate: self.is_single_non_generic_candidate,
            arg_check_mode: self.arg_check_mode,
            signature_help_trailing_comma: self.signature_help_trailing_comma,
            candidates_for_argument_error: self.candidates_for_argument_error.clone(),
            candidate_for_argument_arity_error: self.candidate_for_argument_arity_error.clone(),
            candidate_for_type_argument_error: self.candidate_for_type_argument_error.clone(),
        }
    }
}

pub(crate) fn for_each_yield_expression(
    body: Option<&Arc<Node>>,
    visitor: &mut impl FnMut(&Arc<Node>) -> bool,
) -> bool { ::tsox_core::fntrace::enter("for_each_yield_expression"); 
    fn traverse(node: &Arc<Node>, visitor: &mut impl FnMut(&Arc<Node>) -> bool) -> bool { ::tsox_core::fntrace::enter("traverse"); 
        match node.kind {
            SyntaxKind::YieldExpression => {
                if visitor(node) {
                    return true;
                }
                match node.expression() {
                    None => false,
                    Some(operand) => traverse(operand, visitor),
                }
            }
            SyntaxKind::EnumDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::TypeAliasDeclaration => false,
            _ => {
                if tsox_frontend::ast::utilities::is_function_like(node) {
                    match node.name() {
                        Some(name)
                            if tsox_frontend::ast::node_data_generated::is_computed_property_name(name) =>
                        {
                            match name.expression() {
                                Some(expr) => traverse(expr, visitor),
                                None => false,
                            }
                        }
                        _ => false,
                    }
                } else if !tsox_frontend::ast::mig::m3g_3::is_part_of_type_node(node) {
                    tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
                        traverse(child, visitor)
                    })
                } else {
                    false
                }
            }
        }
    }
    match body {
        Some(body) => traverse(body, visitor),
        None => false,
    }
}
