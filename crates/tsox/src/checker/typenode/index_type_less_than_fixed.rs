use std::sync::Arc;

use crate::ast::node_data_generated::NodeData;
use crate::ast::{ModifierFlags, ModifierList, Node};

use super::*;

pub(crate) fn index_type_less_than_fixed(index_type: &Arc<Type>, limit: usize) -> bool {
    let constituents: Vec<Arc<Type>> = if index_type.flags.contains(TypeFlags::Union) {
        index_type.types().map(|ts| ts.to_vec()).unwrap_or_default()
    } else {
        vec![Arc::clone(index_type)]
    };
    if constituents.is_empty() {
        return false;
    }
    constituents.iter().all(|c| {
        if let Some(LiteralValue::Number(n)) = c.literal_value() {
            let text = n.to_string();
            if let Ok(index) = text.parse::<f64>() {
                return index >= 0.0 && index < limit as f64;
            }
        }
        false
    })
}

pub(crate) fn is_static_modifier(modifiers: &Option<Arc<ModifierList>>) -> bool {
    modifiers
        .as_ref()
        .map(|m| m.modifier_flags.contains(ModifierFlags::Static))
        .unwrap_or(false)
}

pub(crate) fn template_token_text(node: &Arc<Node>) -> String {
    match &node.data {
        NodeData::TemplateHead(d) => d.text.clone(),
        NodeData::TemplateMiddle(d) => d.text.clone(),
        NodeData::TemplateTail(d) => d.text.clone(),
        _ => String::new(),
    }
}
