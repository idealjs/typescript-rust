use std::sync::Arc;

use tsox_frontend::ast::ModifierFlags;
use tsox_frontend::ast::ModifierList;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_data_generated::is_indexed_access_type_node;

use crate::checker::checker::Checker;
use crate::checker::mig::m1f_4::get_total_fixed_element_count;
use crate::checker::types::ElementFlags;
use crate::checker::types::TypeData;
use crate::checker::typenode::*;

pub(crate) fn index_type_less_than_fixed(index_type: &Arc<Type>, limit: usize) -> bool { ::tsox_core::fntrace::enter("index_type_less_than_fixed"); 
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

impl Checker {
    pub(crate) fn should_defer_indexed_access_type(
        &mut self,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
        access_node: Option<&Node>,
    ) -> bool { ::tsox_core::fntrace::enter("should_defer_indexed_access_type");
        if self.type_flags_is_generic_index_type(index_type) {
            return true;
        }
        if access_node.is_some_and(|n| !is_indexed_access_type_node(n)) {
            if let TypeData::Tuple(tup) = &object_type.data {
                return tup.combined_flags.intersects(ElementFlags::Variadic)
                    && !index_type_less_than_fixed(
                        index_type,
                        get_total_fixed_element_count(object_type),
                    );
            }
            return false;
        }
        if self.type_flags_is_generic_object_type(object_type) {
            if let TypeData::Tuple(tup) = &object_type.data {
                if index_type_less_than_fixed(index_type, tup.fixed_length) {
                    return false;
                }
            }
            return true;
        }
        self.is_generic_reducible_type(object_type)
    }
}

pub(crate) fn is_static_modifier(modifiers: &Option<Arc<ModifierList>>) -> bool { ::tsox_core::fntrace::enter("is_static_modifier");
    modifiers
        .as_ref()
        .map(|m| m.modifier_flags.contains(ModifierFlags::Static))
        .unwrap_or(false)
}

pub(crate) fn template_token_text(node: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("template_token_text"); 
    match &node.data {
        NodeData::TemplateHead(d) => d.text.clone(),
        NodeData::TemplateMiddle(d) => d.text.clone(),
        NodeData::TemplateTail(d) => d.text.clone(),
        _ => String::new(),
    }
}
