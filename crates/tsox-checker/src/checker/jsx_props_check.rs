#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::jsx_impl_chunk_2::*;
use crate::checker::relater_relation::RelationKind;
use crate::checker::types::*;
use std::sync::Arc;
use tsox_frontend::ast::{Diagnostic, Node, NodeData, Symbol, SymbolFlags};

pub(crate) fn semantic_jsx_children(children: &[Arc<Node>]) -> Vec<Arc<Node>> {
    children
        .iter()
        .filter(|c| match &c.data {
            NodeData::JsxExpression(e) => e.expression.is_some(),
            NodeData::JsxText(t) => !t.contains_only_trivia_white_spaces,
            _ => true,
        })
        .cloned()
        .collect()
}

fn jsx_attribute_name_text(attr: &Arc<Node>) -> Option<String> {
    match &attr.data {
        NodeData::JsxAttribute(d) => Some(d.name.text().to_string()),
        _ => None,
    }
}

fn is_hyphenated_jsx_name(name: &str) -> bool {
    name.contains('-') || name.contains(':')
}

impl Checker {
    pub(crate) fn check_jsx_element_props(&mut self, opening: &Arc<Node>) {
        let attributes = match &opening.data {
            NodeData::JsxOpeningElement(d) => Some(Arc::clone(&d.attributes)),
            NodeData::JsxSelfClosingElement(d) => Some(Arc::clone(&d.attributes)),
            _ => None,
        };
        let props = if opening.kind == SyntaxKind::JsxOpeningFragment {
            self.jsx_fragment_props_type(opening)
        } else {
            let tag_name = match jsx_tag_name(opening) {
                Some(t) => t,
                None => return,
            };
            if is_jsx_intrinsic_tag_name(&tag_name) {
                self.intrinsic_props_type(&tag_name)
            } else {
                self.component_props_type(opening, &tag_name)
            }
        };
        let Some(props) = props else { return };
        let attrs_type = self.create_jsx_attributes_type(opening);
        let error_node = match jsx_tag_name(opening) {
            Some(t) => t,
            None => Arc::clone(opening),
        };
        let expr = if opening.kind == SyntaxKind::JsxOpeningFragment {
            None
        } else {
            attributes
        };
        self.check_type_related_to_and_optionally_elaborate(
            &attrs_type,
            &props,
            RelationKind::Assignable,
            Some(&error_node),
            expr.as_ref(),
            None,
            None,
        );
    }

    fn intrinsic_props_type(&mut self, tag_name: &Arc<Node>) -> Option<Arc<Type>> {
        let intrinsic_elements = self.get_jsx_intrinsic_elements()?;
        let tag_text = tag_name.text().to_string();
        if let Some(member) = intrinsic_elements
            .members
            .get(&tag_text)
            .or_else(|| intrinsic_elements.exports.get(&tag_text))
        {
            return Some(self.get_type_of_symbol(member));
        }
        let elements_type = self.get_type_of_symbol(&intrinsic_elements);
        for info in self.get_index_infos_of_type(&elements_type) {
            if let Some(key) = &info.key_type
                && self.is_type_assignable_to(&self.get_string_type(), key)
                && let Some(value) = &info.value_type
            {
                return Some(Arc::clone(value));
            }
        }
        None
    }

    fn component_props_type(
        &mut self,
        opening: &Arc<Node>,
        tag_name: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        self.check_expression(tag_name);
        let tag_type = self.get_type_of_node(tag_name);
        if tag_type.flags.contains(TypeFlags::Any) {
            return None;
        }
        let apparent = self.get_apparent_type(&tag_type);
        let construct = self.get_signatures_of_type(&apparent, crate::checker::SignatureKind::Construct);
        let (sig, is_class) = if !construct.is_empty() {
            (construct.into_iter().next()?, true)
        } else {
            let call = self.get_signatures_of_type(&apparent, crate::checker::SignatureKind::Call);
            (call.into_iter().next()?, false)
        };
        if is_class {
            self.class_props_type(opening, &sig)
        } else {
            Some(self.get_type_at_position(&sig, 0))
        }
    }

    fn class_props_type(
        &mut self,
        opening: &Arc<Node>,
        sig: &Arc<crate::checker::types::Signature>,
    ) -> Option<Arc<Type>> {
        let ns = self.get_jsx_namespace()?;
        let forced = self.get_name_from_jsx_element_attributes_container(
            crate::checker::jsx_impl_chunk::JsxNames::ELEMENT_ATTRIBUTES_PROPERTY_NAME_CONTAINER,
            &ns,
        );
        let instance = self.get_return_type_of_signature(sig)?;
        match forced {
            None => Some(self.get_type_at_position(sig, 0)),
            Some(name) if name.is_empty() => Some(instance),
            Some(name) => {
                let attr_type = self.get_type_of_property_of_type(&instance, &name);
                match attr_type {
                    Some(t) => Some(t),
                    None => {
                        let has_attrs = match &opening.data {
                            NodeData::JsxOpeningElement(d) => {
                                !matches!(&d.attributes.data, NodeData::JsxAttributes(a) if a.properties.is_empty())
                            }
                            NodeData::JsxSelfClosingElement(d) => {
                                !matches!(&d.attributes.data, NodeData::JsxAttributes(a) if a.properties.is_empty())
                            }
                            _ => false,
                        };
                        if has_attrs {
                            self.grammar_error_on_node_with_args(
                                opening,
                                &tsox_core::diagnostics::messages_generated::
                                    JSX_ELEMENT_CLASS_DOES_NOT_SUPPORT_ATTRIBUTES_BECAUSE_IT_DOES_NOT_HAVE_A_0_PROPERTY,
                                &[name],
                            );
                        }
                        None
                    }
                }
            }
        }
    }

    fn jsx_fragment_props_type(&mut self, opening: &Arc<Node>) -> Option<Arc<Type>> {
        let frag_type = self.get_jsx_fragment_type_for_props(opening)?;
        let apparent = self.get_apparent_type(&frag_type);
        let sigs = self.get_signatures_of_type(&apparent, crate::checker::SignatureKind::Call);
        let sig = sigs.into_iter().next()?;
        Some(self.get_type_at_position(&sig, 0))
    }

    fn get_jsx_fragment_type_for_props(&mut self, opening: &Arc<Node>) -> Option<Arc<Type>> {
        use tsox_core::core::compiler_options::JsxEmit;
        let _ = opening;
        let namespace = self.jsx_mark_namespace(true);
        let should_resolve = matches!(self.compiler_options.jsx, JsxEmit::React)
            || !self.compiler_options.jsx_fragment_factory.is_empty();
        if !(should_resolve && namespace != "null") {
            return Some(self.get_any_type());
        }
        let container = self.jsx_factory_namespace_symbol(&namespace)?;
        let sym = container.exports.get("Fragment").cloned()?;
        Some(self.get_type_of_symbol(&sym))
    }

}
