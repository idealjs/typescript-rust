#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_core::diagnostics::Message;
use tsox_core::json::Value as JsonValue;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::text::TextRange;
use tsox_core::tspath;
use tsox_frontend::ast::diagnostic::{self, Diagnostic};
use tsox_frontend::ast::node_data_generated::{is_array_literal_expression, is_object_literal_expression, is_property_assignment, is_string_literal, NodeData};
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::node_source_file::SourceFile;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use super::m5h_4::*;
use super::m5h_5::create_unknown_option_error;
use super::m5j::*;
use super::m5j_2::*;
use super::m5j_3::parse_json_source_file_config_file_content;
use super::m5i_2::{option_elements, CommandLineOptionNameMap};
use crate::tsoptions::option_kind::OptionDecl;
use crate::tsoptions::build_options::ParsedCommandLine;

pub struct ResolverHost<'a> {
    pub host: &'a ParseConfigHost,
}

impl ResolverHost<'_> {
    pub fn trace(&self, _msg: &str) {}
}

pub struct ExtendedConfigCacheEntry {
    pub extended_result: Option<TsConfigSourceFile>,
    pub extended_config: Option<ParsedTsconfig>,
    pub errors: Vec<Diagnostic>,
}

impl ExtendedConfigCacheEntry {
    pub fn extended_file_names(&self) -> Vec<String> {
        match &self.extended_result {
            Some(extended_result) => extended_result.extended_source_files.clone(),
            None => Vec::new(),
        }
    }
}

pub fn command_line_option_name_map_get(
    map: &'static CommandLineOptionNameMap,
    name: &str,
) -> Option<&'static OptionDecl> {
    map.get(name).or_else(|| map.get(&name.to_lowercase()))
}

pub fn command_line_option_name_map_get_spelling_suggestion(
    map: &'static CommandLineOptionNameMap,
    name: &str,
) -> Option<&'static OptionDecl> {
    tsox_core::core::mig::m3j_2::get_spelling_suggestion(
        name,
        map.options_names.values(),
        |option| option.name.to_string(),
        |a, b| a.name.cmp(b.name),
    )
}

pub fn for_each_property_assignment<T>(
    object_literal: Option<&Node>,
    key: &str,
    key2: Option<&str>,
    callback: &mut dyn FnMut(&Node) -> Option<T>,
) -> Option<T> {
    let object_literal = object_literal?;
    let properties = match &object_literal.data {
        NodeData::ObjectLiteralExpression(data) => &data.properties,
        _ => return None,
    };
    for property in &properties.nodes {
        if !is_property_assignment(property) {
            continue;
        }
        let name_node = match &property.data {
            NodeData::PropertyAssignment(data) => &data.name,
            _ => continue,
        };
        let Some(prop_name) = tsox_frontend::ast::mig::m3h::try_get_text_of_property_name(name_node) else {
            continue;
        };
        if prop_name == key || key2 == Some(prop_name.as_str()) {
            return callback(property);
        }
    }
    None
}

pub fn for_each_ts_config_prop_array<T>(
    ts_config_source_file: Option<&SourceFile>,
    prop_key: &str,
    callback: &mut dyn FnMut(&Node) -> Option<T>,
) -> Option<T> {
    let object_literal = get_ts_config_object_literal_expression(ts_config_source_file)?;
    for_each_property_assignment(Some(object_literal), prop_key, None, callback)
}

pub fn get_callback_for_finding_property_assignment_by_value(
    value: &str,
) -> impl Fn(&Node) -> Option<Arc<Node>> + '_ {
    move |property: &Node| {
        if is_array_literal_expression(property) {
            let elements = match &property.data {
                NodeData::ArrayLiteralExpression(data) => &data.elements,
                _ => return None,
            };
            return elements
                .nodes
                .iter()
                .find(|element| is_string_literal(element) && element.text() == value)
                .cloned();
        }
        None
    }
}

pub fn get_options_syntax_by_array_element_value(
    object_literal: Option<&Node>,
    prop_key: &str,
    element_value: &str,
) -> Option<Arc<Node>> {
    let callback = get_callback_for_finding_property_assignment_by_value(element_value);
    for_each_property_assignment(object_literal, prop_key, None, &mut |property| callback(property))
}

pub fn property_initializer(property: &Node) -> Option<Arc<Node>> {
    match &property.data {
        NodeData::PropertyAssignment(data) => Some(data.initializer.clone()),
        _ => None,
    }
}

fn skip_trivia(_text: &str, pos: usize) -> usize {
    pos
}

pub fn new_tsconfig_source_file_from_file_path(
    config_file_name: &str,
    config_path: &str,
    config_source_text: &str,
) -> TsConfigSourceFile {
    let source_file = tsox_frontend::parser::Parser::parse_source_file_text(
        config_file_name,
        config_source_text.to_string(),
    );
    TsConfigSourceFile {
        extended_source_files: Vec::new(),
        config_file_specs: None,
        source_file: Arc::new(source_file),
    }
}

pub fn parse_config_file_text_to_json(
    file_name: &str,
    path: &str,
    json_text: &str,
) -> (Option<JsonValue>, Vec<Diagnostic>) {
    let (json_source_file, parse_diagnostics) =
        tsox_frontend::parser::Parser::parse_source_file_text_with_diagnostics(
            file_name,
            json_text.to_string(),
        );
    let (config, mut errors) = convert_config_file_to_object(&json_source_file, None);
    if !parse_diagnostics.is_empty() {
        let first = &parse_diagnostics[0];
        errors = vec![Diagnostic::new(
            None,
            first.range,
            first.message,
            first.message_args.clone(),
        )];
    }
    (config, errors)
}
