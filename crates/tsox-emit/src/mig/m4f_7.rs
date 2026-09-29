#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::ModifierList;

use crate::mig::wt1b::r39k05_defs::new_get_accessor_declaration_full_r39k05;
use crate::printer::{AutoGenerateOptions, NodeFactory};

pub(crate) fn create_accessor_property_get_redirector_m4f5(
    factory: &NodeFactory<'_>,
    node: &Arc<Node>,
    modifiers: Option<Arc<ModifierList>>,
    name: &Arc<Node>,
    receiver: &Arc<Node>,
) -> Arc<Node> {
    let backing_field_name = factory.generated_name_node(
        &factory.new_generated_private_name_for_node_ex(
            node.name().expect("property declaration requires a name"),
            AutoGenerateOptions {
                suffix: "_accessor_storage".to_string(),
                ..Default::default()
            },
        ),
    );
    let return_expression = factory.new_property_access_expression(
        receiver,
        None,
        &backing_field_name,
        NodeFlags::empty(),
    );
    let return_statement = factory.new_return_statement(Some(&return_expression));
    let body = factory.new_block(&factory.new_node_list(vec![return_statement]), false);
    let parameters = factory.new_node_list(vec![]);
    new_get_accessor_declaration_full_r39k05(
        factory,
        modifiers,
        name,
        None,
        parameters,
        None,
        None,
        body,
    )
}

pub(crate) fn create_accessor_property_set_redirector_m4f5(
    factory: &NodeFactory<'_>,
    node: &Arc<Node>,
    modifiers: Option<Arc<ModifierList>>,
    name: &Arc<Node>,
    receiver: &Arc<Node>,
) -> Arc<Node> {
    let backing_field_name = factory.generated_name_node(
        &factory.new_generated_private_name_for_node_ex(
            node.name().expect("property declaration requires a name"),
            AutoGenerateOptions {
                suffix: "_accessor_storage".to_string(),
                ..Default::default()
            },
        ),
    );
    let value_param = factory.new_parameter_declaration(
        None,
        None,
        &factory.new_identifier("value"),
        None,
        None,
        None,
    );
    let assignment = factory.new_assignment_expression(
        &factory.new_property_access_expression(
            receiver,
            None,
            &backing_field_name,
            NodeFlags::empty(),
        ),
        &factory.new_identifier("value"),
    );
    let expression_statement = factory.new_expression_statement(&assignment);
    let body = factory.new_block(&factory.new_node_list(vec![expression_statement]), false);
    let parameters = factory.new_node_list(vec![value_param]);
    factory.new_set_accessor_declaration(modifiers, name, None, parameters, None, None, body)
}
