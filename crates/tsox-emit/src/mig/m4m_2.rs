#![allow(unused_imports)]
#![allow(dead_code)]

#[path = "r36k22_defs.rs"]
pub mod r36k22_defs;

use std::sync::Arc;

use tsox_core::core::core::find_last;
use tsox_core::core::mig::m3j::last_or_nil;
use tsox_core::core::text::TextRange;
use tsox_core::jsnum::Number;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_source_file::SourceFile;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{
    can_have_modifiers, get_source_file_of_node, is_binding_pattern, is_decorator,
    is_expression_statement, is_identifier, is_keyword_kind, is_method_declaration,
    is_numeric_literal, is_property_declaration, is_string_literal_like, is_super_call,
    is_try_statement, position_is_synthesized, skip_parentheses,
};
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::scanner::TOKEN_FLAGS_NONE;

use crate::mig::m4j::r36k3_defs::R36K3NodeFactoryExt;
use crate::printer::{EmitContext, NodeFactory};

pub fn get_ecma_line_of_position(source_file: &SourceFile, position: usize) -> usize {
    source_file.line_map.line_at(position)
}

static SOURCE_FILE_REGISTRY: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<u64, std::sync::Arc<SourceFile>>>,
> = std::sync::OnceLock::new();

fn source_file_registry() -> &'static std::sync::Mutex<std::collections::HashMap<u64, std::sync::Arc<SourceFile>>> {
    SOURCE_FILE_REGISTRY.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

pub fn register_source_file_line_map(source_file: &std::sync::Arc<SourceFile>) {
    source_file_registry()
        .lock()
        .unwrap()
        .insert(source_file.id(), source_file.clone());
}

pub fn registered_source_file_of_node(source_file_node: &Arc<Node>) -> Option<std::sync::Arc<SourceFile>> {
    source_file_registry()
        .lock()
        .unwrap()
        .get(&source_file_node.id())
        .cloned()
}

pub fn is_original_node_single_line_in_file(
    emit_context: &EmitContext,
    source_file: Option<std::sync::Arc<SourceFile>>,
    node: Option<&Arc<Node>>,
) -> bool {
    let node = match node {
        Some(n) => n,
        None => return false,
    };
    let original = emit_context.most_original(node);
    let source = match source_file {
        Some(s) => s,
        None => return false,
    };
    let start_line = get_ecma_line_of_position(&source, original.loc.pos());
    let end_line = get_ecma_line_of_position(&source, original.loc.end());
    start_line == end_line
}

pub fn is_generated_identifier(emit_context: &EmitContext, name: &Arc<Node>) -> bool {
    emit_context.has_auto_generate_info(name)
}

pub fn is_helper_name(emit_context: &EmitContext, name: &Arc<Node>) -> bool {
    emit_context.emit_flags(name).contains(EmitFlags::HELPER_NAME)
}

pub fn is_local_name(emit_context: &EmitContext, name: &Arc<Node>) -> bool {
    emit_context.emit_flags(name).contains(EmitFlags::LOCAL_NAME)
}

pub fn is_export_name(emit_context: &EmitContext, name: &Arc<Node>) -> bool {
    emit_context.emit_flags(name).contains(EmitFlags::EXPORT_NAME)
}

fn node_matches(child: Option<&Arc<Node>>, name: &Arc<Node>) -> bool {
    child.map_or(false, |c| Arc::ptr_eq(c, name))
}

fn node_in_list(list: &tsox_frontend::ast::NodeList, name: &Arc<Node>) -> bool {
    list.nodes.iter().any(|c| Arc::ptr_eq(c, name))
}

pub fn is_identifier_reference(name: &Arc<Node>, parent: &Arc<Node>) -> bool {
    match parent.kind {
        SyntaxKind::BinaryExpression
        | SyntaxKind::PrefixUnaryExpression
        | SyntaxKind::PostfixUnaryExpression
        | SyntaxKind::YieldExpression
        | SyntaxKind::AsExpression
        | SyntaxKind::SatisfiesExpression
        | SyntaxKind::ElementAccessExpression
        | SyntaxKind::NonNullExpression
        | SyntaxKind::SpreadElement
        | SyntaxKind::SpreadAssignment
        | SyntaxKind::ParenthesizedExpression
        | SyntaxKind::ArrayLiteralExpression
        | SyntaxKind::DeleteExpression
        | SyntaxKind::TypeOfExpression
        | SyntaxKind::VoidExpression
        | SyntaxKind::AwaitExpression
        | SyntaxKind::TypeAssertionExpression
        | SyntaxKind::ExpressionWithTypeArguments
        | SyntaxKind::JsxSelfClosingElement
        | SyntaxKind::JsxSpreadAttribute
        | SyntaxKind::JsxExpression
        | SyntaxKind::PartiallyEmittedExpression => true,
        SyntaxKind::ComputedPropertyName
        | SyntaxKind::Decorator
        | SyntaxKind::IfStatement
        | SyntaxKind::DoStatement
        | SyntaxKind::WhileStatement
        | SyntaxKind::WithStatement
        | SyntaxKind::ReturnStatement
        | SyntaxKind::SwitchStatement
        | SyntaxKind::CaseClause
        | SyntaxKind::ThrowStatement
        | SyntaxKind::ExpressionStatement
        | SyntaxKind::ExportAssignment
        | SyntaxKind::PropertyAccessExpression
        | SyntaxKind::TemplateSpan => node_matches(parent.expression(), name),
        SyntaxKind::VariableDeclaration
        | SyntaxKind::Parameter
        | SyntaxKind::BindingElement
        | SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertySignature
        | SyntaxKind::PropertyAssignment
        | SyntaxKind::EnumMember
        | SyntaxKind::JsxAttribute => {
            node_matches(tsox_frontend::ast::mig::m3b::initializer(parent), name)
        }
        SyntaxKind::ShorthandPropertyAssignment => match &parent.data {
            NodeData::ShorthandPropertyAssignment(d) => {
                node_matches(d.object_assignment_initializer.as_ref(), name)
            }
            _ => false,
        },
        SyntaxKind::ForStatement => match &parent.data {
            NodeData::ForStatement(d) => {
                node_matches(d.initializer.as_ref(), name)
                    || node_matches(d.condition.as_ref(), name)
                    || node_matches(d.incrementor.as_ref(), name)
            }
            _ => false,
        },
        SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => match &parent.data {
            NodeData::ForInOrOfStatement(d) => {
                Arc::ptr_eq(&d.initializer, name) || Arc::ptr_eq(&d.expression, name)
            }
            _ => false,
        },
        SyntaxKind::ImportEqualsDeclaration => match &parent.data {
            NodeData::ImportEqualsDeclaration(d) => Arc::ptr_eq(&d.module_reference, name),
            _ => false,
        },
        SyntaxKind::ArrowFunction => match &parent.data {
            NodeData::ArrowFunction(d) => Arc::ptr_eq(&d.body, name),
            _ => false,
        },
        SyntaxKind::ConditionalExpression => match &parent.data {
            NodeData::ConditionalExpression(d) => {
                Arc::ptr_eq(&d.condition, name)
                    || Arc::ptr_eq(&d.when_true, name)
                    || Arc::ptr_eq(&d.when_false, name)
            }
            _ => false,
        },
        SyntaxKind::CallExpression | SyntaxKind::NewExpression => match &parent.data {
            NodeData::CallExpression(d) => {
                Arc::ptr_eq(&d.expression, name) || node_in_list(&d.arguments, name)
            }
            NodeData::NewExpression(d) => {
                Arc::ptr_eq(&d.expression, name)
                    || d.arguments.as_deref().map_or(false, |l| node_in_list(l, name))
            }
            _ => false,
        },
        SyntaxKind::TaggedTemplateExpression => match &parent.data {
            NodeData::TaggedTemplateExpression(d) => Arc::ptr_eq(&d.tag, name),
            _ => false,
        },
        SyntaxKind::ImportAttribute => match &parent.data {
            NodeData::ImportAttribute(d) => Arc::ptr_eq(&d.value, name),
            _ => false,
        },
        SyntaxKind::JsxOpeningElement | SyntaxKind::JsxClosingElement => match &parent.data {
            NodeData::JsxOpeningElement(d) => Arc::ptr_eq(&d.tag_name, name),
            NodeData::JsxClosingElement(d) => Arc::ptr_eq(&d.tag_name, name),
            _ => false,
        },
        _ => false,
    }
}

fn binding_element_data(element: &Arc<Node>) -> &tsox_frontend::ast::node_data_generated::BindingElementData {
    match &element.data {
        NodeData::BindingElement(d) => d,
        _ => panic!("BindingElement expected"),
    }
}

pub fn convert_binding_element_to_array_assignment_element(
    emit_context: &EmitContext,
    element: &Arc<Node>,
) -> Arc<Node> {
    let binding = binding_element_data(element);
    let Some(name) = binding.name.clone() else {
        let elision = emit_context.factory().new_omitted_expression();
        emit_context.set_original(&elision, element);
        emit_context.assign_comment_and_source_map_ranges(&elision, element);
        return elision;
    };
    if binding.dot_dot_dot_token.is_some() {
        let spread = emit_context.factory().new_spread_element(name);
        emit_context.set_original(&spread, element);
        emit_context.assign_comment_and_source_map_ranges(&spread, element);
        return spread;
    }
    let expression =
        convert_binding_name_to_assignment_element_target(emit_context, &name);
    if let Some(initializer) = binding.initializer.clone() {
        let assignment = emit_context.factory().new_assignment_expression(
            &expression,
            &initializer,
        );
        emit_context.set_original(&assignment, element);
        emit_context.assign_comment_and_source_map_ranges(&assignment, element);
        return assignment;
    }
    expression
}

pub fn convert_binding_element_to_object_assignment_element(
    emit_context: &EmitContext,
    element: &Arc<Node>,
) -> Arc<Node> {
    let binding = binding_element_data(element);
    let name = binding.name.clone().expect("BindingElement name expected");
    if binding.dot_dot_dot_token.is_some() {
        let spread = emit_context.factory().new_spread_assignment(name);
        emit_context.set_original(&spread, element);
        emit_context.assign_comment_and_source_map_ranges(&spread, element);
        return spread;
    }
    if let Some(property_name) = binding.property_name.clone() {
        let mut expression =
            convert_binding_name_to_assignment_element_target(emit_context, &name);
        if let Some(initializer) = binding.initializer.clone() {
            expression = emit_context.factory().new_assignment_expression(
                &expression,
                &initializer,
            );
        }
        let assignment = emit_context.factory().new_property_assignment(
            None,
            &property_name,
            None,
            None,
            &expression,
        );
        emit_context.set_original(&assignment, element);
        emit_context.assign_comment_and_source_map_ranges(&assignment, element);
        return assignment;
    }
    let equals_token = binding
        .initializer
        .as_ref()
        .map(|_| emit_context.factory().new_token(SyntaxKind::EqualsToken));
    let assignment = emit_context.factory().new_shorthand_property_assignment(
        None,
        &name,
        None,
        None,
        equals_token.as_ref(),
        binding.initializer.clone(),
    );
    emit_context.set_original(&assignment, element);
    emit_context.assign_comment_and_source_map_ranges(&assignment, element);
    assignment
}

pub fn convert_binding_pattern_to_assignment_pattern(
    emit_context: &EmitContext,
    element: &Arc<Node>,
) -> Arc<Node> {
    match element.kind {
        SyntaxKind::ArrayBindingPattern => {
            convert_binding_element_to_array_assignment_pattern(emit_context, element)
        }
        SyntaxKind::ObjectBindingPattern => {
            convert_binding_element_to_object_assignment_pattern(emit_context, element)
        }
        _ => panic!("Unknown binding pattern"),
    }
}

fn binding_pattern_elements(element: &Arc<Node>) -> std::sync::Arc<tsox_frontend::ast::NodeList> {
    match &element.data {
        NodeData::BindingPattern(d) => d.elements.clone(),
        _ => panic!("BindingPattern expected"),
    }
}

pub fn convert_binding_element_to_object_assignment_pattern(
    emit_context: &EmitContext,
    element: &Arc<Node>,
) -> Arc<Node> {
    let mut properties: Vec<Arc<Node>> = vec![];
    let elements = binding_pattern_elements(element);
    for child in elements.nodes.iter() {
        properties.push(convert_binding_element_to_object_assignment_element(
            emit_context,
            child,
        ));
    }
    let property_list = node_list_with_loc(properties, elements.loc);
    let object = emit_context
        .factory()
        .new_object_literal_expression(&property_list, false);
    emit_context.set_original(&object, element);
    emit_context.assign_comment_and_source_map_ranges(&object, element);
    object
}

fn node_list_with_loc(nodes: Vec<Arc<Node>>, loc: TextRange) -> tsox_frontend::ast::NodeList {
    let mut list = tsox_frontend::ast::NodeList::new(nodes);
    list.loc = loc;
    list
}

pub fn convert_binding_element_to_array_assignment_pattern(
    emit_context: &EmitContext,
    element: &Arc<Node>,
) -> Arc<Node> {
    let mut elements_out: Vec<Arc<Node>> = vec![];
    let elements = binding_pattern_elements(element);
    for child in elements.nodes.iter() {
        elements_out.push(convert_binding_element_to_array_assignment_element(
            emit_context, child,
        ));
    }
    let element_list = node_list_with_loc(elements_out, elements.loc);
    let object = emit_context
        .factory()
        .new_array_literal_expression(&element_list, false);
    emit_context.set_original(&object, element);
    emit_context.assign_comment_and_source_map_ranges(&object, element);
    object
}

pub fn convert_binding_name_to_assignment_element_target(
    emit_context: &EmitContext,
    element: &Arc<Node>,
) -> Arc<Node> {
    if is_binding_pattern(element) {
        return convert_binding_pattern_to_assignment_pattern(emit_context, element);
    }
    element.clone()
}

pub fn convert_variable_declaration_to_assignment_expression(
    emit_context: &EmitContext,
    element: &Arc<Node>,
) -> Option<Arc<Node>> {
    let declaration = match &element.data {
        NodeData::VariableDeclaration(d) => d,
        _ => panic!("VariableDeclaration expected"),
    };
    let initializer = declaration.initializer.clone()?;
    let expression = convert_binding_name_to_assignment_element_target(
        emit_context,
        &declaration.name,
    );
    let assignment =
        emit_context
            .factory()
            .new_assignment_expression(&expression, &initializer);
    emit_context.set_original(&assignment, element);
    emit_context.assign_comment_and_source_map_ranges(&assignment, element);
    Some(assignment)
}

pub fn single_or_many(nodes: Option<&[Arc<Node>]>, factory: &NodeFactory) -> Option<Arc<Node>> {
    let nodes = nodes?;
    if nodes.len() == 1 {
        return Some(nodes[0].clone());
    }
    Some(factory.new_syntax_list(nodes.to_vec()))
}

pub fn is_simple_copiable_expression(expression: &Arc<Node>) -> bool {
    is_string_literal_like(expression)
        || is_numeric_literal(expression)
        || is_keyword_kind(expression.kind)
        || is_identifier(expression)
}

pub fn is_original_node_single_line(emit_context: &EmitContext, node: Option<&Arc<Node>>) -> bool {
    let node = match node {
        Some(n) => n,
        None => return false,
    };
    let original = emit_context.most_original(node);
    let source = match get_source_file_of_node(&original) {
        Some(s) => s,
        None => return false,
    };
    let source = match registered_source_file_of_node(&source) {
        Some(s) => s,
        None => return false,
    };
    let start_line = get_ecma_line_of_position(&source, original.loc.pos());
    let end_line = get_ecma_line_of_position(&source, original.loc.end());
    start_line == end_line
}

pub fn is_simple_inlineable_expression(expression: &Arc<Node>) -> bool {
    !is_identifier(expression) && is_simple_copiable_expression(expression)
}

pub fn find_super_statement_index_path(statements: &[Arc<Node>], start: usize) -> Vec<usize> {
    let mut indices =
        find_super_statement_index_path_worker(statements, start, Vec::new()).unwrap_or_default();
    indices.reverse();
    indices
}

pub fn find_super_statement_index_path_worker(
    statements: &[Arc<Node>],
    start: usize,
    indices: Vec<usize>,
) -> Option<Vec<usize>> {
    let mut indices = indices;
    for i in start..statements.len() {
        let statement = &statements[i];
        if get_super_call_from_statement(statement).is_some() {
            indices.push(i);
            return Some(indices);
        } else if is_try_statement(statement) {
            let try_block = match &statement.data {
                NodeData::TryStatement(d) => d.try_block.clone(),
                _ => panic!("TryStatement expected"),
            };
            let block_statements = match &try_block.data {
                NodeData::Block(d) => &d.statements.nodes,
                _ => panic!("Block expected"),
            };
            if let Some(mut result) =
                find_super_statement_index_path_worker(block_statements, 0, indices.clone())
            {
                result.push(i);
                return Some(result);
            }
        }
    }
    None
}

pub fn get_super_call_from_statement(statement: &Arc<Node>) -> Option<Arc<Node>> {
    if !is_expression_statement(statement) {
        return None;
    }
    let expression = skip_parentheses(&statement.expression().clone().unwrap());
    if is_super_call(&expression) {
        return Some(expression);
    }
    None
}

pub fn move_range_past_modifiers(node: &Arc<Node>) -> TextRange {
    if is_property_declaration(node) || is_method_declaration(node) {
        return TextRange::new(node.name().unwrap().pos(), node.end());
    }

    let mut last_modifier: Option<Arc<Node>> = None;
    if can_have_modifiers(node) {
        last_modifier = last_or_nil(node.modifier_nodes());
    }

    if let Some(last_modifier) = last_modifier {
        if !position_is_synthesized(last_modifier.end()) {
            return TextRange::new(last_modifier.end(), node.end());
        }
    }
    move_range_past_decorators(node)
}

pub fn move_range_past_decorators(node: &Arc<Node>) -> TextRange {
    let mut last_decorator: Option<Arc<Node>> = None;
    if can_have_modifiers(node) {
        let nodes = node.modifier_nodes();
        if !nodes.is_empty() {
            last_decorator = find_last(nodes, |n: &Arc<Node>| is_decorator(n)).cloned();
        }
    }

    if let Some(last_decorator) = last_decorator {
        if !position_is_synthesized(last_decorator.end()) {
            return TextRange::new(last_decorator.end(), node.end());
        }
    }
    node.loc
}

pub fn get_non_assignment_operator_for_compound_assignment(kind: SyntaxKind) -> SyntaxKind {
    match kind {
        SyntaxKind::PlusEqualsToken => SyntaxKind::PlusToken,
        SyntaxKind::MinusEqualsToken => SyntaxKind::MinusToken,
        SyntaxKind::AsteriskEqualsToken => SyntaxKind::AsteriskToken,
        SyntaxKind::AsteriskAsteriskEqualsToken => SyntaxKind::AsteriskAsteriskToken,
        SyntaxKind::SlashEqualsToken => SyntaxKind::SlashToken,
        SyntaxKind::PercentEqualsToken => SyntaxKind::PercentToken,
        SyntaxKind::LessThanLessThanEqualsToken => SyntaxKind::LessThanLessThanToken,
        SyntaxKind::GreaterThanGreaterThanEqualsToken => SyntaxKind::GreaterThanGreaterThanToken,
        SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken => {
            SyntaxKind::GreaterThanGreaterThanGreaterThanToken
        }
        SyntaxKind::AmpersandEqualsToken => SyntaxKind::AmpersandToken,
        SyntaxKind::BarEqualsToken => SyntaxKind::BarToken,
        SyntaxKind::CaretEqualsToken => SyntaxKind::CaretToken,
        SyntaxKind::BarBarEqualsToken => SyntaxKind::BarBarToken,
        SyntaxKind::AmpersandAmpersandEqualsToken => SyntaxKind::AmpersandAmpersandToken,
        SyntaxKind::QuestionQuestionEqualsToken => SyntaxKind::QuestionQuestionToken,
        _ => kind,
    }
}

pub fn constant_expression(value: &ConstantValue, factory: &NodeFactory) -> Option<Arc<Node>> {
    match value {
        ConstantValue::String(s) => Some(factory.new_string_literal(s, TOKEN_FLAGS_NONE)),
        ConstantValue::Number(n) => {
            if n.is_inf() {
                if *n > Number::from(0.0_f64) {
                    return Some(factory.new_identifier("Infinity"));
                }
                return Some(factory.new_prefix_unary_expression(
                    SyntaxKind::MinusToken,
                    &factory.new_identifier("Infinity"),
                ));
            }
            if n.is_nan() {
                return Some(factory.new_identifier("NaN"));
            }
            if *n < Number::from(0.0_f64) {
                return Some(factory.new_prefix_unary_expression(
                    SyntaxKind::MinusToken,
                    &constant_expression(&ConstantValue::Number(-*n), factory)?,
                ));
            }
            Some(factory.new_numeric_literal(&n.to_string(), TOKEN_FLAGS_NONE))
        }
    }
}

pub enum ConstantValue {
    String(String),
    Number(Number),
}
