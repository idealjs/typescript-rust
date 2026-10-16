#![allow(unused_imports)]
use std::sync::Arc;
#[allow(unused_imports)]
use tsox_frontend::ast::node_source_file::ScriptKind;
#[allow(unused_imports)]
use tsox_frontend::ast::node_data_generated::{
    BindingElementData, ExportAssignmentData, PropertyAssignmentData,
    PropertyDeclarationData, ShorthandPropertyAssignmentData, VariableDeclarationData,
};
use tsox_frontend::ast::{is_no_substitution_template_literal, Node, SyntaxKind};
use tsox_frontend::scanner::TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE;
use crate::printer::EmitContext;
use crate::mig::m4n_5::r36k29_defs::R36K29NodeExt;
use super::m4h_2::{
    finish_transform_named_evaluation, get_assigned_name_of_identifier,
    get_assigned_name_of_property_name,
};

pub fn has_invalid_escape(template: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_invalid_escape"); 
    if is_no_substitution_template_literal(template) {
        return template
            .template_literal_flags()
            .is_some_and(|flags| flags & TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE != 0);
    }
    let te = template.as_template_expression();
    if te
        .head
        .template_literal_flags()
        .is_some_and(|flags| flags & TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE != 0)
    {
        return true;
    }
    for span in &te.template_spans.nodes {
        if span
            .as_template_span()
            .literal
            .template_literal_flags()
            .is_some_and(|flags| flags & TOKEN_FLAGS_CONTAINS_INVALID_ESCAPE != 0)
        {
            return true;
        }
    }
    false
}

pub fn transform_named_evaluation_of_property_assignment(
    context: &EmitContext,
    node: &Arc<PropertyAssignmentData>,
    ignore_empty_string_literal: bool,
    assigned_name_text: &str,
) -> Arc<Node> { ::tsox_core::fntrace::enter("transform_named_evaluation_of_property_assignment"); 
    let factory = context.factory();
    let (assigned_name, name) =
        get_assigned_name_of_property_name(context, &node.name, assigned_name_text);
    let initializer = finish_transform_named_evaluation(
        context,
        &node.initializer,
        &assigned_name,
        ignore_empty_string_literal,
    );
    factory.update_property_assignment(node, None, &name, None, None, Some(&initializer))
}

pub fn transform_named_evaluation_of_shorthand_assignment_property(
    context: &EmitContext,
    node: &Arc<ShorthandPropertyAssignmentData>,
    ignore_empty_string_literal: bool,
    assigned_name_text: &str,
) -> Arc<Node> { ::tsox_core::fntrace::enter("transform_named_evaluation_of_shorthand_assignment_property"); 
    let factory = context.factory();
    let assigned_name = if !assigned_name_text.is_empty() {
        factory.new_string_literal(assigned_name_text, 0)
    } else {
        get_assigned_name_of_identifier(
            context,
            &node.name,
            node.object_assignment_initializer.as_ref().unwrap(),
        )
    };
    let object_assignment_initializer = finish_transform_named_evaluation(
        context,
        node.object_assignment_initializer.as_ref().unwrap(),
        &assigned_name,
        ignore_empty_string_literal,
    );
    factory.update_shorthand_property_assignment(
        node,
        None,
        &node.name,
        None,
        None,
        node.equals_token.as_ref(),
        Some(&object_assignment_initializer),
    )
}

pub fn transform_named_evaluation_of_variable_declaration(
    context: &EmitContext,
    node: &Arc<VariableDeclarationData>,
    ignore_empty_string_literal: bool,
    assigned_name_text: &str,
) -> Arc<Node> { ::tsox_core::fntrace::enter("transform_named_evaluation_of_variable_declaration"); 
    let factory = context.factory();
    let assigned_name = if !assigned_name_text.is_empty() {
        factory.new_string_literal(assigned_name_text, 0)
    } else {
        get_assigned_name_of_identifier(context, &node.name, node.initializer.as_ref().unwrap())
    };
    let initializer = finish_transform_named_evaluation(
        context,
        node.initializer.as_ref().unwrap(),
        &assigned_name,
        ignore_empty_string_literal,
    );
    factory.update_variable_declaration(node, &node.name, None, None, Some(&initializer))
}

pub fn transform_named_evaluation_of_parameter_declaration(
    context: &EmitContext,
    node: &Arc<Node>,
    ignore_empty_string_literal: bool,
    assigned_name_text: &str,
) -> Arc<Node> { ::tsox_core::fntrace::enter("transform_named_evaluation_of_parameter_declaration"); 
    use crate::mig::m4m::r36k5_defs::NodeDataExt;
    let factory = context.factory();
    let data = node.as_parameter_declaration();
    let assigned_name = if !assigned_name_text.is_empty() {
        factory.new_string_literal(assigned_name_text, 0)
    } else {
        get_assigned_name_of_identifier(context, &data.name, data.initializer.as_ref().unwrap())
    };
    let initializer = finish_transform_named_evaluation(
        context,
        data.initializer.as_ref().unwrap(),
        &assigned_name,
        ignore_empty_string_literal,
    );
    factory.update_parameter_declaration(
        node,
        None,
        data.dot_dot_dot_token.as_ref(),
        &data.name,
        None,
        None,
        Some(&initializer),
    )
}

pub fn transform_named_evaluation_of_binding_element(
    context: &EmitContext,
    node: &Arc<BindingElementData>,
    ignore_empty_string_literal: bool,
    assigned_name_text: &str,
) -> Arc<Node> { ::tsox_core::fntrace::enter("transform_named_evaluation_of_binding_element"); 
    let factory = context.factory();
    let assigned_name = if !assigned_name_text.is_empty() {
        factory.new_string_literal(assigned_name_text, 0)
    } else {
        get_assigned_name_of_identifier(
            context,
            node.name.as_ref().unwrap(),
            node.initializer.as_ref().unwrap(),
        )
    };
    let initializer = finish_transform_named_evaluation(
        context,
        node.initializer.as_ref().unwrap(),
        &assigned_name,
        ignore_empty_string_literal,
    );
    factory.update_binding_element(
        node,
        node.dot_dot_dot_token.as_ref(),
        node.property_name.as_ref(),
        node.name.as_ref(),
        Some(&initializer),
    )
}

pub fn transform_named_evaluation_of_property_declaration(
    context: &EmitContext,
    node: &Arc<PropertyDeclarationData>,
    ignore_empty_string_literal: bool,
    assigned_name_text: &str,
) -> Arc<Node> { ::tsox_core::fntrace::enter("transform_named_evaluation_of_property_declaration"); 
    let factory = context.factory();
    let (assigned_name, name) =
        get_assigned_name_of_property_name(context, &node.name, assigned_name_text);
    let initializer = finish_transform_named_evaluation(
        context,
        node.initializer.as_ref().unwrap(),
        &assigned_name,
        ignore_empty_string_literal,
    );
    factory.update_property_declaration_data(node, node.modifiers.clone(), &name, None, None, Some(&initializer))
}

pub fn transform_named_evaluation_of_export_assignment(
    context: &EmitContext,
    node: &Arc<ExportAssignmentData>,
    ignore_empty_string_literal: bool,
    assigned_name_text: &str,
) -> Arc<Node> { ::tsox_core::fntrace::enter("transform_named_evaluation_of_export_assignment"); 
    let factory = context.factory();
    let assigned_name = if !assigned_name_text.is_empty() {
        factory.new_string_literal(assigned_name_text, 0)
    } else if node.is_export_equals {
        factory.new_string_literal("", 0)
    } else {
        factory.new_string_literal("default", 0)
    };
    let expression = finish_transform_named_evaluation(
        context,
        &node.expression,
        &assigned_name,
        ignore_empty_string_literal,
    );
    factory.update_export_assignment(node, None, node.is_export_equals, None, &expression)
}
