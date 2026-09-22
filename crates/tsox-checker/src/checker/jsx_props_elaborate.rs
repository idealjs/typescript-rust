#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::jsx_impl_chunk_2::*;
use crate::checker::jsx_props_check::semantic_jsx_children;
use crate::checker::relater_relation::RelationKind;
use crate::checker::types::*;
use std::sync::Arc;
use tsox_frontend::ast::{Diagnostic, Node, NodeData};

fn is_hyphenated_jsx_name(name: &str) -> bool {
    name.contains('-') || name.contains(':')
}

impl Checker {
    pub(crate) fn elaborate_jsx_components_impl(
        &mut self,
        node: &Arc<Node>,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        mut out: Option<&mut Vec<Diagnostic>>,
    ) -> bool {
        let properties = match &node.data {
            NodeData::JsxAttributes(d) => &d.properties,
            _ => return false,
        };
        let mut reported = false;
        for prop in properties.iter() {
            let NodeData::JsxAttribute(_) = &prop.data else {
                continue;
            };
            let Some(name_node) = prop.name() else { continue };
            let name = name_node.text().to_string();
            if is_hyphenated_jsx_name(&name) {
                continue;
            }
            let Some(target_prop_type) = self.get_type_of_property_of_type(target, &name) else {
                continue;
            };
            let Some(source_prop_type) = self.get_type_of_property_of_type(source, &name) else {
                continue;
            };
            if self.is_type_related_to(&source_prop_type, &target_prop_type, relation) {
                continue;
            }
            let initializer = match &prop.data {
                NodeData::JsxAttribute(a) => a.initializer.as_ref().and_then(|i| match &i.data {
                    NodeData::JsxExpression(e) => e.expression.clone(),
                    _ => Some(Arc::clone(i)),
                }),
                _ => None,
            };
            let elaborated = initializer.is_some_and(|init| {
                self.elaborate_error(&init, &source_prop_type, &target_prop_type, relation, out.as_deref_mut())
            });
            reported = true;
            if !elaborated {
                self.check_type_related_to_and_optionally_elaborate(
                    &source_prop_type,
                    &target_prop_type,
                    relation,
                    Some(&name_node),
                    None,
                    None,
                    out.as_deref_mut(),
                );
            }
        }
        let (children, opening_parent): (Option<Arc<Node>>, Option<Arc<Node>>) =
            match node.parent() {
                Some(p) if p.kind == SyntaxKind::JsxOpeningElement || p.kind == SyntaxKind::JsxSelfClosingElement => {
                    (p.parent(), Some(p))
                }
                _ => (None, None),
            };
        let Some(containing) = children else { return reported };
        let Some(opening_parent) = opening_parent else { return reported };
        let valid_children = match &containing.data {
            NodeData::JsxElement(d) => semantic_jsx_children(&d.children.nodes),
            _ => return reported,
        };
        if valid_children.is_empty() || containing.kind != SyntaxKind::JsxElement {
            return reported;
        }
        let children_name = self
            .get_jsx_namespace()
            .and_then(|ns| self.get_jsx_element_children_property_name(&ns))
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| "children".to_string());
        let children_name_type = self.get_string_literal_type(&children_name);
        let children_target_type = self.get_indexed_access_type(target, &children_name_type);
        let source_child = self.get_indexed_access_type(source, &children_name_type);
        let more_than_one = valid_children.len() > 1;
        if !more_than_one {
            let child = &valid_children[0];
            let (error_node, inner, inner_type) = match &child.data {
                NodeData::JsxExpression(e) => (
                    Arc::clone(child),
                    e.expression.clone(),
                    e.expression.as_ref().map(|x| self.get_type_of_node(x)),
                ),
                NodeData::JsxText(_) => (Arc::clone(child), None, Some(self.get_string_type())),
                _ => (
                    Arc::clone(child),
                    Some(Arc::clone(child)),
                    Some(self.get_type_of_node(child)),
                ),
            };
            if self.is_type_related_to(&source_child, &children_target_type, relation) {
                return reported;
            }
            if let Some(inner) = &inner
                && let Some(it) = &inner_type
                && self.elaborate_error(
                    inner,
                    it,
                    &children_target_type,
                    relation,
                    out.as_deref_mut(),
                )
            {
                return true;
            }
            let specific = inner_type.clone().unwrap_or(source_child);
            self.check_type_related_to_and_optionally_elaborate(
                &specific,
                &children_target_type,
                relation,
                Some(&error_node),
                None,
                None,
                out.as_deref_mut(),
            );
            return true;
        }
        reported
    }
}
