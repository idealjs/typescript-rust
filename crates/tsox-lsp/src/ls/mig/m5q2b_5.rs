#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::ls::lsutil_user_preferences::{QuotePreference, UserPreferences};
use tsox_checker::checker::Checker;
use tsox_core::core;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::astnav;
use tsox_frontend::scanner;

use super::m5q_3::SORT_TEXT_LOCATION_PRIORITY;
use super::m5q2::generate_jsdoc_param_tags_for_destructuring;
use super::m5r::str_ptr_to;
use super::m5q2b_3::lsproto;
use super::m5q2b_6::get_jsdoc_param_annotation;

pub fn get_jsdoc_parameter_completions(
    file: &Arc<SourceFile>,
    position: usize,
    type_checker: &mut Checker,
    options: &core::compiler_options::CompilerOptions,
    preferences: &UserPreferences,
    tag_name_only: bool,
) -> Vec<crate::ls::types_completion::CompletionItem> { ::tsox_core::fntrace::enter("get_jsdoc_parameter_completions"); 
    let Some(current_token) = astnav::get_token_at_position(&file.node, position) else {
        return Vec::new();
    };
    if !ast::is_jsdoc_tag(&current_token) && !tsox_frontend::ast::node_data_generated::is_jsdoc(&current_token) {
        return Vec::new();
    }
    let js_doc = if tsox_frontend::ast::node_data_generated::is_jsdoc(&current_token) {
        Arc::clone(&current_token)
    } else {
        current_token.parent().expect("jsdoc tag should have a jsdoc parent")
    };
    if !ast::node_data_generated::is_jsdoc(&js_doc) {
        return Vec::new();
    }
    let fun = js_doc.parent().expect("jsdoc should have a parent");
    if !ast::is_function_like(&fun) {
        return Vec::new();
    }

    let is_js = ast::is_source_file_js(file);
    let is_snippet = false;
    let mut param_tag_count = 0usize;
    let tags = super::m5u_3::jsdoc_tags(&js_doc).unwrap_or_default();
    for tag in tags.iter() {
        if tag.kind == SyntaxKind::JSDocParameterTag
            && astnav::get_start_of_node(tag, file, false) < position
            && tag.name().is_some_and(|n| ast::is_identifier(n))
        {
            param_tag_count += 1;
        }
    }
    let mut param_index: i64 = -1;
    let mut items = Vec::new();
    for param in ast::mig::m3b::parameters(&fun).iter() {
        param_index += 1;
        if (param_index as usize) < param_tag_count {
            continue;
        }
        if param.name().is_some_and(|n| ast::is_identifier(n)) {
            let mut tabstop_counter = 1i32;
            let param_name = param.name().map(|n| n.text()).unwrap_or("");
            let mut display_text = get_jsdoc_param_annotation(
                &param_name,
                ast::mig::m3b::initializer(&param),
                parameter_dot_dot_dot_token(&param),
                is_js,
                false,
                false,
                type_checker,
                options,
                preferences,
                &mut tabstop_counter,
            );
            let mut snippet_text = String::new();
            if is_snippet {
                snippet_text = get_jsdoc_param_annotation(
                    &param_name,
                    ast::mig::m3b::initializer(&param),
                    parameter_dot_dot_dot_token(&param),
                    is_js,
                    false,
                    true,
                    type_checker,
                    options,
                    preferences,
                    &mut tabstop_counter,
                );
            }
            if tag_name_only {
                display_text.remove(0);
                if !snippet_text.is_empty() {
                    snippet_text.remove(0);
                }
            }
            items.push(crate::ls::types_completion::CompletionItem {
                label: display_text,
                kind: Some(lsproto::CompletionItemKind::Variable as u32),
                sort_text: Some(SORT_TEXT_LOCATION_PRIORITY.to_string()),
                insert_text: str_ptr_to(&snippet_text),
                insert_text_format: if is_snippet {
                    Some(lsproto::InsertTextFormat::Snippet as u32)
                } else {
                    None
                },
                ..Default::default()
            });
        } else if param_index as usize == param_tag_count {
            let param_path = format!("param{}", param_index);
            let display_text_result = generate_jsdoc_param_tags_for_destructuring(
                &param_path,
                param.name().expect("parameter should have a name"),
                ast::mig::m3b::initializer(&param),
                parameter_dot_dot_dot_token(&param),
                is_js,
                false,
                type_checker,
                options,
                preferences,
            );
            let mut snippet_text = String::new();
            if is_snippet {
                let snippet_text_result = generate_jsdoc_param_tags_for_destructuring(
                    &param_path,
                    param.name().expect("parameter should have a name"),
                    ast::mig::m3b::initializer(&param),
                    parameter_dot_dot_dot_token(&param),
                    is_js,
                    true,
                    type_checker,
                    options,
                    preferences,
                );
                snippet_text = snippet_text_result.join(&format!(
                    "{}* ",
                    options.new_line.get_new_line_character()
                ));
            }
            let mut display_text = display_text_result.join(&format!(
                "{}* ",
                options.new_line.get_new_line_character()
            ));
            if tag_name_only {
                display_text = display_text
                    .strip_prefix('@')
                    .unwrap_or(&display_text)
                    .to_string();
                snippet_text = snippet_text
                    .strip_prefix('@')
                    .unwrap_or(&snippet_text)
                    .to_string();
            }
            items.push(crate::ls::types_completion::CompletionItem {
                label: display_text,
                kind: Some(lsproto::CompletionItemKind::Variable as u32),
                sort_text: Some(SORT_TEXT_LOCATION_PRIORITY.to_string()),
                insert_text: str_ptr_to(&snippet_text),
                insert_text_format: if is_snippet {
                    Some(lsproto::InsertTextFormat::Snippet as u32)
                } else {
                    None
                },
                ..Default::default()
            });
        }
    }
    items
}

fn parameter_dot_dot_dot_token(param: &Node) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("parameter_dot_dot_dot_token"); 
    match &param.data {
        ast::NodeData::ParameterDeclaration(d) => d.dot_dot_dot_token.as_ref(),
        _ => None,
    }
}
