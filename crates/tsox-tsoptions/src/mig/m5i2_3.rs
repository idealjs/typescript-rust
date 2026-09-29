#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_core::diagnostics::Message;
use tsox_core::json::Value as JsonValue;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::text::TextRange;
use tsox_core::tspath::{self, ComparePathsOptions};
use tsox_frontend::ast::diagnostic::{self, Diagnostic};
use tsox_frontend::ast::node_data_generated::{is_array_literal_expression, is_object_literal_expression, is_string_literal, NodeData};
use tsox_frontend::ast::Node;
use tsox_frontend::ast::SourceFile;

use super::m5h_3::ContentMapper;
use super::m5i2::*;
use super::m5i2_2::*;
use super::m5j::*;
use super::m5j_2::*;
use super::m5i_3::{add_implied_options, filter_same_as_default_include, serialize_compiler_options, TSConfig};
use crate::tsoptions::build_options::ParsedCommandLine;

fn skip_trivia(_text: &str, pos: usize) -> usize {
    pos
}

pub fn create_diagnostic_at_project_reference_property(
    source_file: Option<&TsConfigSourceFile>,
    index: usize,
    property_name: &str,
    message: Message,
    args: Vec<String>,
) -> Diagnostic {
    let node: Option<Arc<Node>> = source_file.and_then(|sf| {
        for_each_ts_config_prop_array(Some(sf.source_file.as_ref()), "references", &mut |property| {
            if is_array_literal_expression(property) {
                let elements = match &property.data {
                    NodeData::ArrayLiteralExpression(data) => &data.elements,
                    _ => return None,
                };
                if let Some(element) = elements.nodes.get(index) {
                    if is_object_literal_expression(element) {
                        if let Some(property_node) = for_each_property_assignment(
                            Some(element),
                            property_name,
                            None,
                            &mut |property| property_initializer(property),
                        ) {
                            return Some(property_node);
                        }
                        return Some(element.clone());
                    }
                }
            }
            None
        })
    });
    create_diagnostic_for_node_in_source_file_or_compiler_diagnostic(
        tsconfig_to_source_file(source_file),
        node.as_deref(),
        message,
        args,
    )
}

pub fn get_content_mapper_syntax(
    source_file: Option<&SourceFile>,
    index: i64,
    sub_key: &str,
) -> Option<Arc<Node>> {
    let source_file = source_file?;
    for_each_ts_config_prop_array(Some(source_file), "contentMappers", &mut |property| {
        let initializer = property_initializer(property)?;
        if !is_array_literal_expression(&initializer) {
            return Some(initializer);
        }
        let elements = match &initializer.data {
            NodeData::ArrayLiteralExpression(data) => &data.elements,
            _ => return Some(initializer),
        };
        let length = elements.nodes.len() as i64;
        if index < 0 || index >= length {
            return Some(initializer);
        }
        let element = &elements.nodes[index as usize];
        if !sub_key.is_empty() && is_object_literal_expression(element) {
            if let Some(node) = for_each_property_assignment(
                Some(element),
                sub_key,
                None,
                &mut |property| property_initializer(property),
            ) {
                return Some(node);
            }
        }
        Some(element.clone())
    })
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MapperDefinition {
    pub extensions: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mapper {
    pub definition: MapperDefinition,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OptionPathSegment {
    pub property: String,
    pub index: usize,
    pub is_index: bool,
}

pub fn get_content_mapper_option_diagnostic_location<'a>(
    config: Option<&'a ParsedCommandLine>,
    mapper: &ContentMapper,
    path: &[OptionPathSegment],
) -> Option<(&'a SourceFile, TextRange)> {
    let config = config?;
    let config_file = config.config_file.as_ref()?;
    let index = config
        .content_mappers()
        .iter()
        .position(|candidate| {
            candidate.definition.package == mapper.definition.package
                && candidate.definition.extensions == mapper.definition.extensions
        })
        .map(|found| found as i64)
        .unwrap_or(-1);
    let mapper_node = get_content_mapper_syntax(Some(config_file.source_file.as_ref()), index, "");
    let mut node =
        get_content_mapper_syntax(Some(config_file.source_file.as_ref()), index, "options").or(mapper_node);
    for segment in path {
        let current = match &node {
            Some(current) => current,
            None => break,
        };
        let next = if segment.is_index {
            if is_array_literal_expression(current) {
                match &current.data {
                    NodeData::ArrayLiteralExpression(data) => data.elements.nodes.get(segment.index).cloned(),
                    _ => None,
                }
            } else {
                None
            }
        } else if is_object_literal_expression(current) {
            for_each_property_assignment(
                Some(current),
                &segment.property,
                None,
                &mut |property| property_initializer(property),
            )
        } else {
            None
        };
        node = match next {
            Some(next) => Some(next),
            None => break,
        };
    }
    let node = node?;
    let file = config_file.source_file.as_ref();
    Some((file, TextRange::new(skip_trivia(&file.text, node.pos()), node.end())))
}

pub fn get_content_mappers_key_syntax(source_file: Option<&SourceFile>) -> Option<Arc<Node>> {
    let source_file = source_file?;
    for_each_ts_config_prop_array(Some(source_file), "contentMappers", &mut |property| {
        property.name().cloned()
    })
}

pub fn get_content_mapper_extension_syntax(
    source_file: Option<&SourceFile>,
    index: i64,
    ext: &str,
) -> Option<Arc<Node>> {
    let node = get_content_mapper_syntax(source_file, index, "extensions")?;
    if is_array_literal_expression(&node) {
        let elements = match &node.data {
            NodeData::ArrayLiteralExpression(data) => &data.elements,
            _ => return Some(node),
        };
        if let Some(element) = elements
            .nodes
            .iter()
            .find(|element| is_string_literal(element) && element.text() == ext)
        {
            return Some(element.clone());
        }
    }
    Some(node)
}
