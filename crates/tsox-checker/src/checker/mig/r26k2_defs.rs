#![allow(unused_imports)]
use std::cell::Cell;

use std::sync::Arc;

use crate::checker::checker::Checker;
use crate::checker::types::*;
use tsox_frontend::ast::node_data_generated::{
    NodeData, ParenthesizedExpressionData, StringLiteralData,
};
use tsox_frontend::ast::{Node, SyntaxKind};

pub trait R26K2FactoryExt {
    fn new_string_literal(&self, text: &str, token_flags: i32) -> Arc<Node>;
    fn new_parenthesized_expression(&self, expression: &Arc<Node>) -> Arc<Node>;
}

impl R26K2FactoryExt for tsox_frontend::ast::mig::m3c::NodeFactory {
    fn new_string_literal(&self, text: &str, _token_flags: i32) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::StringLiteral,
            NodeData::StringLiteral(StringLiteralData {
                text: text.to_string(),
                token_flags: Default::default(),
            }),
        ))
    }

    fn new_parenthesized_expression(&self, expression: &Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ParenthesizedExpression,
            NodeData::ParenthesizedExpression(ParenthesizedExpressionData {
                expression: Arc::clone(expression),
            }),
        ))
    }
}

pub(crate) fn with_object_type_structured_mut(
    t: &Arc<Type>,
    f: impl FnOnce(&mut StructuredTypeData),
) {
    let mut t = Arc::clone(t);
    let Some(t) = Arc::get_mut(&mut t) else { return };
    let structured = match &mut t.data {
        TypeData::Object(d) => &mut d.structured,
        TypeData::Interface(d) => &mut d.object.structured,
        TypeData::Tuple(d) => &mut d.interface_data.object.structured,
        TypeData::Mapped(d) => &mut d.object.structured,
        TypeData::ReverseMapped(d) => &mut d.object.structured,
        TypeData::EvolvingArray(d) => &mut d.object.structured,
        TypeData::InstantiationExpression(d) => &mut d.object.structured,
        _ => return,
    };
    f(structured);
}

pub(crate) fn set_interface_resolved_base_types(t: &Arc<Type>, base_types: Vec<Arc<Type>>) {
    let mut t = Arc::clone(t);
    let Some(t) = Arc::get_mut(&mut t) else { return };
    match &mut t.data {
        TypeData::Interface(interface) => interface.resolved_base_types = base_types,
        TypeData::Tuple(tuple) => tuple.interface_data.resolved_base_types = base_types,
        _ => {}
    }
}

impl Checker {
    pub(crate) fn set_mapped_type_contains_error(&self, t: &Arc<Type>, v: bool) {
        if let TypeData::Mapped(d) = &t.data {
            d.contains_error.store(v, std::sync::atomic::Ordering::Relaxed);
        }
    }
}
