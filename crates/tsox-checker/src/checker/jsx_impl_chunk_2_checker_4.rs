#![allow(unused_imports)]

use crate::checker::jsx_impl_chunk_2::*;
use crate::checker::types::*;
use std::sync::Arc;
use tsox_frontend::ast::{Node, Symbol, SymbolFlags};

impl Checker {
    pub fn get_uninstantiated_jsx_signatures_of_type(
        &mut self,
        element_type: &Arc<Type>,
        caller: &Arc<Node>,
    ) -> Vec<Arc<Signature>> {
        if element_type.flags.contains(TypeFlags::String) {
            let mut sig = Signature::new();
            let _ = sig.resolved_return_type.set(self.get_any_type());
            return vec![Arc::new(sig)];
        }
        if element_type.flags.contains(TypeFlags::StringLiteral) {
            return match self.get_intrinsic_attributes_type_from_string_literal_type(
                element_type,
                caller,
            ) {
                Some(props) => vec![self.create_signature_for_jsx_intrinsic(caller, &props)],
                None => {
                    let name = self
                        .string_literal_values(element_type)
                        .into_iter()
                        .next()
                        .unwrap_or_default();
                    self.grammar_error_on_node_with_args(
                        caller,
                        &tsox_core::diagnostics::messages_generated::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                        &[name, format!("JSX.{}", JsxNames::INTRINSIC_ELEMENTS)],
                    );
                    Vec::new()
                }
            };
        }
        let apparent = if element_type.flags.intersects(TypeFlags::TypeParameter) {
            self.get_constraint_of_type_parameter(element_type)
                .unwrap_or_else(|| self.get_apparent_type(element_type))
        } else {
            self.get_apparent_type(element_type)
        };
        let mut signatures =
            self.get_signatures_of_type(&apparent, crate::checker::SignatureKind::Construct);
        if signatures.is_empty() {
            signatures = self.get_signatures_of_type(&apparent, crate::checker::SignatureKind::Call);
        }
        if signatures.is_empty()
            && apparent.is_union()
            && let Some(types) = apparent.types()
        {
            let lists: Vec<Vec<Arc<Signature>>> = types
                .iter()
                .map(|t| self.get_uninstantiated_jsx_signatures_of_type(t, caller))
                .collect();
            return self.get_union_signatures(&lists);
        }
        signatures
    }

    pub fn get_intrinsic_attributes_type_from_string_literal_type(
        &mut self,
        t: &Arc<Type>,
        _location: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        let intrinsic_elements = self.get_jsx_intrinsic_elements()?;
        let elements_type = self.get_type_of_symbol(&intrinsic_elements);
        if self.is_error_type(&elements_type) {
            return Some(self.get_any_type());
        }
        let Some(name) = self.string_literal_values(t).into_iter().next() else {
            return None;
        };
        if let Some(prop) = intrinsic_elements
            .members
            .get(&name)
            .or_else(|| intrinsic_elements.exports.get(&name))
        {
            return Some(self.get_type_of_symbol(prop));
        }
        self.get_index_type_of_type(&elements_type, crate::checker::services_checker_7::IndexKind::String)
    }

    pub fn create_signature_for_jsx_intrinsic(
        &mut self,
        _node: &Arc<Node>,
        result: &Arc<Type>,
    ) -> Arc<Signature> {
        let mut element_type = self.get_error_type();
        if let Some(ns) = self.get_jsx_namespace()
            && let Some(sym) = ns
                .exports
                .get(JsxNames::ELEMENT)
                .or_else(|| ns.members.get(JsxNames::ELEMENT))
                .filter(|s| s.flags.intersects(SymbolFlags::TYPE))
        {
            element_type = self.get_declared_type_of_symbol(sym);
        }
        let props_sym = Arc::new(Symbol::new(
            SymbolFlags::FunctionScopedVariable,
            "props".to_string(),
        ));
        self.value_symbol_links.insert(
            &props_sym,
            ValueSymbolLinks {
                resolved_type: Some(Arc::clone(result)),
                ..Default::default()
            },
        );
        let mut sig = Signature::new();
        sig.min_argument_count = 1;
        sig.parameters = vec![props_sym];
        let _ = sig.resolved_return_type.set(element_type);
        Arc::new(sig)
    }
}
