#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_core::diagnostics::Message;
use tsox_core::json::Value as JsonValue;
use tsox_core::core::text::TextRange;
use tsox_core::tspath;
use tsox_frontend::ast::diagnostic::{self, Diagnostic};
use tsox_frontend::ast::mig::m3f_4::is_computed_non_literal_name;
use tsox_frontend::ast::node_data_generated::{
    is_array_literal_expression, is_object_literal_expression, is_property_assignment,
    is_string_literal, NodeData,
};
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node::SourceFile;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use super::m5h_2::NameMap;
use super::m5h_4::get_compiler_option_value_type_string;
use super::m5h_5::create_unknown_option_error;
use super::m5i2::*;
use super::m5j::*;
use super::m5j_2::*;
use crate::tsoptions::option_kind::OptionDecl;

pub type CommandLineOptionNameMap = NameMap;

fn command_line_option_name_map_get<'a>(
    map: &'a CommandLineOptionNameMap,
    name: &str,
) -> Option<&'a OptionDecl> { ::tsox_core::fntrace::enter("command_line_option_name_map_get"); 
    map.get(name).or_else(|| map.get(&name.to_lowercase()))
}

fn new_diagnostic(
    _source_file: &SourceFile,
    loc: TextRange,
    message: Message,
    args: Vec<String>,
) -> Diagnostic { ::tsox_core::fntrace::enter("new_diagnostic"); 
    Diagnostic::new(None, loc, message, args)
}

pub fn convert_array_literal_expression_to_json(
    source_file: &SourceFile,
    elements: &[Arc<Node>],
    element_option: Option<&OptionDecl>,
    return_value: bool,
) -> (Option<JsonValue>, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("convert_array_literal_expression_to_json"); 
    if !return_value {
        for element in elements {
            convert_property_value_to_json(source_file, element, element_option, return_value, None);
        }
        return (None, Vec::new());
    }
    if elements.is_empty() {
        return (Some(JsonValue::Array(Vec::new())), Vec::new());
    }
    let mut errors: Vec<Diagnostic> = Vec::new();
    let mut value: Vec<JsonValue> = Vec::new();
    for element in elements {
        let (converted_value, err) =
            convert_property_value_to_json(source_file, element, element_option, return_value, None);
        errors.extend(err);
        if let Some(converted_value) = converted_value {
            value.push(converted_value);
        }
    }
    (Some(JsonValue::Array(value)), errors)
}

pub fn convert_object_literal_expression_to_json(
    source_file: &SourceFile,
    return_value: bool,
    node: &Node,
    object_option: Option<&OptionDecl>,
    json_conversion_notifier: Option<&JsonConversionNotifier>,
) -> (Option<JsonObject>, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("convert_object_literal_expression_to_json"); 
    let mut result: JsonObject = Vec::new();
    let mut errors: Vec<Diagnostic> = Vec::new();
    let properties = match &node.data {
        NodeData::ObjectLiteralExpression(data) => &data.properties,
        _ => return (if return_value { Some(result) } else { None }, errors),
    };
    for element in &properties.nodes {
        if !is_property_assignment(element) {
            errors.push(new_diagnostic(
                source_file,
                element.loc,
                tsox_core::diagnostics::PROPERTY_ASSIGNMENT_EXPECTED,
                vec![],
            ));
            continue;
        }
        let question_token = match &element.data {
            NodeData::PropertyAssignment(data) => data.postfix_token.clone(),
            _ => None,
        };
        if let Some(question_token) = &question_token {
            errors.push(new_diagnostic(
                source_file,
                question_token.loc,
                tsox_core::diagnostics::THE_0_MODIFIER_CAN_ONLY_BE_USED_IN_TYPESCRIPT_FILES,
                vec!["?".to_string()],
            ));
        }
        let name_node = match &element.data {
            NodeData::PropertyAssignment(data) => &data.name,
            _ => continue,
        };
        let key_text = if !is_computed_non_literal_name(name_node) {
            tsox_frontend::ast::mig::m3h::try_get_text_of_property_name(name_node).unwrap_or_default()
        } else {
            String::new()
        };
        let mut option: Option<&'static OptionDecl> = None;
        if !key_text.is_empty() {
            if let Some(object_option) = object_option {
                if let Some(element_option) = object_option.elements() {
                    if element_option.name == key_text {
                        option = Some(element_option);
                    }
                }
            }
        }
        let initializer = property_initializer(element);
        let (value, err) = match initializer {
            Some(initializer) => convert_property_value_to_json(
                source_file,
                &initializer,
                option,
                return_value,
                json_conversion_notifier,
            ),
            None => (None, Vec::new()),
        };
        errors.extend(err);
        if !key_text.is_empty() {
            if return_value {
                json_object_set(&mut result, &key_text, value.clone().unwrap_or(JsonValue::Null));
            }
            if let Some(json_conversion_notifier) = json_conversion_notifier {
                let (_, err) = json_conversion_notifier.on_property_set(
                    &key_text,
                    value,
                    element,
                    object_option,
                    option,
                );
                errors.extend(err);
            }
        }
    }
    (if return_value { Some(result) } else { None }, errors)
}

pub fn convert_property_value_to_json(
    source_file: &SourceFile,
    value_expression: &Node,
    option: Option<&OptionDecl>,
    return_value: bool,
    json_conversion_notifier: Option<&JsonConversionNotifier>,
) -> (Option<JsonValue>, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("convert_property_value_to_json"); 
    let not_expected_format = || -> (Option<JsonValue>, Vec<Diagnostic>) {
        if let Some(option) = option {
            (
                None,
                vec![new_diagnostic(
                    source_file,
                    value_expression.loc,
                    tsox_core::diagnostics::COMPILER_OPTION_0_REQUIRES_A_VALUE_OF_TYPE_1,
                    vec![
                        option.name.to_string(),
                        get_compiler_option_value_type_string(option),
                    ],
                )],
            )
        } else {
            (
                None,
                vec![new_diagnostic(
                    source_file,
                    value_expression.loc,
                    tsox_core::diagnostics::PROPERTY_VALUE_CAN_ONLY_BE_STRING_LITERAL_NUMERIC_LITERAL_TRUE_FALSE_NULL_OBJECT_LITERAL_OR_ARRAY_LITERAL,
                    vec![],
                )],
            )
        }
    };
    match value_expression.kind {
        SyntaxKind::TrueKeyword => (Some(JsonValue::Bool(true)), Vec::new()),
        SyntaxKind::FalseKeyword => (Some(JsonValue::Bool(false)), Vec::new()),
        SyntaxKind::NullKeyword => (None, Vec::new()),
        SyntaxKind::StringLiteral => {
            if !is_double_quoted_string(value_expression) {
                return (
                    Some(JsonValue::String(value_expression.text().to_string())),
                    vec![new_diagnostic(
                        source_file,
                        value_expression.loc,
                        tsox_core::diagnostics::STRING_LITERAL_WITH_DOUBLE_QUOTES_EXPECTED,
                        vec![],
                    )],
                );
            }
            (
                Some(JsonValue::String(value_expression.text().to_string())),
                Vec::new(),
            )
        }
        SyntaxKind::NumericLiteral => (
            Some(JsonValue::from(
                tsox_core::jsnum::Number::from_string(value_expression.text()).0,
            )),
            Vec::new(),
        ),
        SyntaxKind::PrefixUnaryExpression => match &value_expression.data {
            NodeData::PrefixUnaryExpression(data)
                if data.operator == SyntaxKind::MinusToken
                    && data.operand.kind == SyntaxKind::NumericLiteral =>
            {
                (
                    Some(JsonValue::from(
                        -tsox_core::jsnum::Number::from_string(data.operand.text()).0,
                    )),
                    Vec::new(),
                )
            }
            _ => not_expected_format(),
        },
        SyntaxKind::ObjectLiteralExpression => {
            let (value, errors) = convert_object_literal_expression_to_json(
                source_file,
                return_value,
                value_expression,
                option,
                json_conversion_notifier,
            );
            (value.map(|obj| object_to_json_value(&obj)), errors)
        }
        SyntaxKind::ArrayLiteralExpression => {
            let elements = match &value_expression.data {
                NodeData::ArrayLiteralExpression(data) => &data.elements,
                _ => return not_expected_format(),
            };
            convert_array_literal_expression_to_json(source_file, &elements.nodes, option, return_value)
        }
        _ => not_expected_format(),
    }
}

pub fn convert_to_object(source_file: &SourceFile) -> (Option<JsonValue>, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("convert_to_object"); 
    convert_config_file_to_object(source_file, None)
}

pub trait OptionsParserLike {
    fn parse_option(&mut self, key: &str, value: &JsonValue) -> Vec<Diagnostic>;
    fn unknown_option_diagnostic(&self) -> Message;
    fn unknown_did_you_mean_diagnostic(&self) -> Message;
}

pub fn convert_map_to_options<O: OptionsParserLike>(
    compiler_options: &JsonObject,
    mut result: O,
) -> O { ::tsox_core::fntrace::enter("convert_map_to_options"); 
    for (key, value) in compiler_options {
        result.parse_option(key, value);
    }
    result
}

pub fn convert_options_from_json<O: OptionsParserLike>(
    options_name_map: &CommandLineOptionNameMap,
    json_options: Option<&JsonObject>,
    base_path: &str,
    mut result: O,
) -> (O, Vec<Diagnostic>) { ::tsox_core::fntrace::enter("convert_options_from_json"); 
    let Some(json_map) = json_options else {
        return (result, Vec::new());
    };
    let mut errors: Vec<Diagnostic> = Vec::new();
    for (key, value) in json_map {
        let opt = match command_line_option_name_map_get(options_name_map, key) {
            Some(opt) if opt.name != key => {
                errors.push(create_diagnostic_for_node_in_source_file_or_compiler_diagnostic(
                    None,
                    None,
                    result.unknown_did_you_mean_diagnostic(),
                    vec![key.clone(), opt.name.to_string()],
                ));
                continue;
            }
            Some(opt) => opt,
            None => {
                errors.push(create_unknown_option_error(
                    key,
                    Some(result.unknown_option_diagnostic()),
                    Some(""),
                    None,
                    None,
                    None,
                    Some(result.unknown_did_you_mean_diagnostic()),
                    Some(options_name_map),
                ));
                continue;
            }
        };
        let (converted, err) = convert_json_option(opt, value, base_path, None, None, None);
        errors.extend(err);
        errors.extend(result.parse_option(key, &converted.unwrap_or(JsonValue::Null)));
    }
    (result, errors)
}
