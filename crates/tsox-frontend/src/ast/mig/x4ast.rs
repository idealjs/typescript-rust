#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_core::core::compiler_options::{CompilerOptions, JsxEmit, ModuleKind, ResolutionMode};
use tsox_core::tspath;
use tsox_core::tspath::file_extension_is_one_of;

use crate::ast::*;
use crate::astnav;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceFileMetaData {
    pub package_json_type: String,
    pub package_json_directory: String,
    pub implied_node_format: ResolutionMode,
}

pub fn get_class_like_declaration_of_symbol(symbol: &Symbol) -> Option<Arc<Node>> {
    symbol
        .declarations
        .iter()
        .find(|d| is_class_like(d))
        .cloned()
}

pub fn get_containing_function(node: &Arc<Node>) -> Option<Arc<Node>> {
    let parent = node.parent()?;
    find_ancestor(&parent, is_function_like)
}

pub fn get_elements_of_binding_or_assignment_pattern(name: &Arc<Node>) -> Vec<Arc<Node>> {
    match name.kind {
        SyntaxKind::ObjectBindingPattern
        | SyntaxKind::ArrayBindingPattern
        | SyntaxKind::ArrayLiteralExpression => crate::ast::mig::m3b::elements(name).to_vec(),
        SyntaxKind::ObjectLiteralExpression => crate::ast::mig::m3b::properties(name).to_vec(),
        _ => Vec::new(),
    }
}

pub fn get_emit_module_format_of_file_worker(
    file_name: &str,
    options: &CompilerOptions,
    source_file_meta_data: SourceFileMetaData,
) -> ModuleKind {
    let result = get_implied_node_format_for_emit_worker(
        file_name,
        options.get_emit_module_kind(),
        source_file_meta_data,
    );
    if result != ModuleKind::None {
        return result;
    }
    options.get_emit_module_kind()
}

pub fn get_external_module_import_equals_declaration_expression(node: &Arc<Node>) -> Arc<Node> {
    match &node.data {
        NodeData::ImportEqualsDeclaration(d) => match &d.module_reference.data {
            NodeData::ExternalModuleReference(e) => e.expression.clone(),
            _ => d.module_reference.clone(),
        },
        _ => node.clone(),
    }
}

pub fn get_first_constructor_with_body(node: &Arc<Node>) -> Option<Arc<Node>> {
    for member in crate::ast::mig::m3b::members(node) {
        if is_constructor_declaration(member) {
            if let NodeData::ConstructorDeclaration(d) = &member.data {
                if crate::ast::utilities_synthesized::node_is_present(d.body.as_ref()) {
                    return Some(member.clone());
                }
            }
        }
    }
    None
}

pub fn get_host_signature_from_jsdoc(node: &Arc<Node>) -> Option<Arc<Node>> {
    let host = get_jsdoc_host(node)?;
    if is_property_signature_declaration(&host) {
        if let Some(type_node) = host.type_node() {
            if is_function_like(type_node) {
                return Some(type_node.clone());
            }
        }
    }
    if is_function_like(&host) {
        return Some(host);
    }
    None
}

pub fn get_implied_node_format_for_emit_worker(
    file_name: &str,
    emit_module_kind: ModuleKind,
    source_file_meta_data: SourceFileMetaData,
) -> ResolutionMode {
    if ModuleKind::Node16 <= emit_module_kind && emit_module_kind <= ModuleKind::NodeNext {
        return source_file_meta_data.implied_node_format;
    }
    if source_file_meta_data.implied_node_format == ResolutionMode::CommonJS
        && (source_file_meta_data.package_json_type == "commonjs"
            || file_extension_is_one_of(
                file_name,
                &[tspath::EXTENSION_CJS, tspath::EXTENSION_CTS],
            ))
    {
        return ResolutionMode::CommonJS;
    }
    if source_file_meta_data.implied_node_format == ResolutionMode::ESNext
        && (source_file_meta_data.package_json_type == "module"
            || file_extension_is_one_of(
                file_name,
                &[tspath::EXTENSION_MJS, tspath::EXTENSION_MTS],
            ))
    {
        return ResolutionMode::ESNext;
    }
    ResolutionMode::None
}

pub fn get_implied_node_format_for_file(path: &str, package_json_type: &str) -> ModuleKind {
    let mut implied_node_format = ResolutionMode::None;
    if file_extension_is_one_of(
        path,
        &[
            tspath::EXTENSION_DMTS,
            tspath::EXTENSION_MTS,
            tspath::EXTENSION_MJS,
        ],
    ) {
        implied_node_format = ResolutionMode::ESNext;
    } else if file_extension_is_one_of(
        path,
        &[
            tspath::EXTENSION_DCTS,
            tspath::EXTENSION_CTS,
            tspath::EXTENSION_CJS,
        ],
    ) {
        implied_node_format = ResolutionMode::CommonJS;
    } else if file_extension_is_one_of(
        path,
        &[
            tspath::EXTENSION_DTS,
            tspath::EXTENSION_TS,
            tspath::EXTENSION_TSX,
            tspath::EXTENSION_JS,
            tspath::EXTENSION_JSX,
        ],
    ) {
        implied_node_format = if package_json_type == "module" {
            ResolutionMode::ESNext
        } else {
            ResolutionMode::CommonJS
        };
    }

    implied_node_format
}

pub fn get_invoked_expression(node: &Arc<Node>) -> Arc<Node> {
    match node.kind {
        SyntaxKind::TaggedTemplateExpression => {
            if let NodeData::TaggedTemplateExpression(d) = &node.data {
                return d.tag.clone();
            }
        }
        SyntaxKind::JsxOpeningElement | SyntaxKind::JsxSelfClosingElement => {
            return crate::ast::mig::m3c::tag_name(node).clone();
        }
        SyntaxKind::BinaryExpression => {
            if let NodeData::BinaryExpression(d) = &node.data {
                return d.right.clone();
            }
        }
        SyntaxKind::JsxOpeningFragment => return node.clone(),
        _ => {}
    }
    node.expression().cloned().unwrap_or_else(|| node.clone())
}

pub fn get_jsdoc_deprecated_tag(node: &Arc<Node>, source_file: &SourceFile) -> Option<Arc<Node>> {
    for jsdoc in node.jsdoc(source_file) {
        if let NodeData::JSDoc(d) = &jsdoc.data {
            if let Some(tags) = &d.tags {
                for tag in &tags.nodes {
                    if is_jsdoc_deprecated_tag(tag) {
                        return Some(tag.clone());
                    }
                }
            }
        }
    }
    None
}

pub fn get_jsdoc_host(node: &Arc<Node>) -> Option<Arc<Node>> {
    let js_doc = get_jsdoc_root(node)?;
    js_doc.parent()
}

pub fn get_jsdoc_root(node: &Arc<Node>) -> Option<Arc<Node>> {
    let parent = node.parent()?;
    find_ancestor(&parent, |n| n.kind == SyntaxKind::JSDoc)
}

pub fn get_jsx_implicit_import_base(
    compiler_options: &CompilerOptions,
    file: &SourceFile,
) -> String {
    let jsx_import_source_pragma =
        crate::ast::mig::m3f::get_pragma_from_source_file(Some(file), "jsximportsource");
    let jsx_runtime_pragma =
        crate::ast::mig::m3f::get_pragma_from_source_file(Some(file), "jsxruntime");
    if crate::ast::mig::m3f::get_pragma_argument(jsx_runtime_pragma, "factory") == "classic" {
        return String::new();
    }
    if compiler_options.jsx == JsxEmit::ReactJSX
        || compiler_options.jsx == JsxEmit::ReactJSXDev
        || !compiler_options.jsx_import_source.is_empty()
        || jsx_import_source_pragma.is_some()
        || crate::ast::mig::m3f::get_pragma_argument(jsx_runtime_pragma, "factory") == "automatic"
    {
        let mut result =
            crate::ast::mig::m3f::get_pragma_argument(jsx_import_source_pragma, "factory");
        if result.is_empty() {
            result = compiler_options.jsx_import_source.clone();
        }
        if result.is_empty() {
            result = "react".to_string();
        }
        return result;
    }
    String::new()
}

pub fn get_jsx_runtime_import(base: &str, options: &CompilerOptions) -> String {
    if base.is_empty() {
        return base.to_string();
    }
    format!(
        "{}/{}",
        base,
        if options.jsx == JsxEmit::ReactJSXDev {
            "jsx-dev-runtime"
        } else {
            "jsx-runtime"
        }
    )
}
