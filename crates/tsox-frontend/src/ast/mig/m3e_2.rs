use std::sync::Arc;

use crate::ast::node_node::Node;
use crate::ast::node_node_list::NodeList;
use crate::ast::node_source_file::SourceFile;
use crate::ast::syntax_kind_generated::SyntaxKind;
use tsox_core::core::compiler_options::CompilerOptions;

use super::m3g::is_jsx_opening_like_element;
use super::x4ast::{get_implied_node_format_for_emit_worker, SourceFileMetaData};
use crate::ast::subtree_facts::SubtreeFacts;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ExternalModuleIndicatorOptions {
    pub jsx: bool,
    pub force: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct SourceFileParseOptions {
    pub file_name: String,
    pub path: String,
    pub external_module_indicator_options: ExternalModuleIndicatorOptions,
}

pub fn get_external_module_indicator_options(
    file_name: &str,
    options: &CompilerOptions,
    metadata: &SourceFileMetaData,
) -> ExternalModuleIndicatorOptions {
    if tsox_core::tspath::is_declaration_file_name(file_name) {
        return ExternalModuleIndicatorOptions::default();
    }
    match options.get_emit_module_detection_kind() {
        tsox_core::core::compiler_options::ModuleDetectionKind::Force => {
            ExternalModuleIndicatorOptions {
                jsx: false,
                force: true,
            }
        }
        tsox_core::core::compiler_options::ModuleDetectionKind::Legacy => {
            ExternalModuleIndicatorOptions::default()
        }
        tsox_core::core::compiler_options::ModuleDetectionKind::Auto => ExternalModuleIndicatorOptions {
            jsx: options.jsx == tsox_core::core::compiler_options::JsxEmit::ReactJSX
                || options.jsx == tsox_core::core::compiler_options::JsxEmit::ReactJSXDev,
            force: is_file_forced_to_be_module_by_format(file_name, options, metadata),
        },
        _ => ExternalModuleIndicatorOptions::default(),
    }
}

pub const IS_FILE_FORCED_TO_BE_MODULE_BY_FORMAT_EXTENSIONS: [&str; 4] = [
    tsox_core::tspath::EXTENSION_CJS,
    tsox_core::tspath::EXTENSION_CTS,
    tsox_core::tspath::EXTENSION_MJS,
    tsox_core::tspath::EXTENSION_MTS,
];

pub fn is_file_forced_to_be_module_by_format(
    file_name: &str,
    options: &CompilerOptions,
    metadata: &SourceFileMetaData,
) -> bool {
    get_implied_node_format_for_emit_worker(file_name, options.get_emit_module_kind(), metadata.clone())
        == tsox_core::core::compiler_options::ModuleKind::ESNext
        || tsox_core::tspath::file_extension_is_one_of(
            file_name,
            &IS_FILE_FORCED_TO_BE_MODULE_BY_FORMAT_EXTENSIONS,
        )
}

pub fn set_external_module_indicator_with_options(
    file: &mut SourceFile,
    opts: ExternalModuleIndicatorOptions,
) {
    file.external_module_indicator = get_external_module_indicator(file, opts);
}

pub fn get_external_module_indicator(
    file: &SourceFile,
    opts: ExternalModuleIndicatorOptions,
) -> Option<Arc<Node>> {
    if file.script_kind == crate::ast::node_source_file::ScriptKind::Json {
        return None;
    }
    if let Some(node) = is_file_probably_external_module(file) {
        return Some(node);
    }
    if file.is_declaration_file {
        return None;
    }
    if opts.jsx {
        if let Some(node) = is_file_module_from_using_jsx_tag(file) {
            return Some(node);
        }
    }
    if opts.force {
        return Some(file.node.clone());
    }
    None
}

pub fn is_file_probably_external_module(source_file: &SourceFile) -> Option<Arc<Node>> {
    let statements = match &source_file.node.data {
        crate::ast::node_data_generated::NodeData::SourceFile(d) => &d.statements.nodes,
        _ => return None,
    };
    for statement in statements {
        if is_an_external_module_indicator_node(statement) {
            return Some(statement.clone());
        }
    }
    get_import_meta_if_necessary(source_file)
}

pub fn is_an_external_module_indicator_node(node: &Arc<Node>) -> bool {
    if node.has_syntactic_modifier(crate::ast::node_flags::ModifierFlags::Export) {
        return true;
    }
    match &node.data {
        crate::ast::node_data_generated::NodeData::ImportEqualsDeclaration(d) => {
            d.module_reference.kind == SyntaxKind::ExternalModuleReference
        }
        crate::ast::node_data_generated::NodeData::ImportDeclaration(_)
        | crate::ast::node_data_generated::NodeData::ExportAssignment(_)
        | crate::ast::node_data_generated::NodeData::ExportDeclaration(_) => true,
        _ => false,
    }
}

pub fn get_import_meta_if_necessary(source_file: &SourceFile) -> Option<Arc<Node>> {
    if source_file
        .node
        .flags
        .contains(crate::ast::node_flags::NodeFlags::PossiblyContainsImportMeta)
    {
        return find_child_node(&source_file.node, is_import_meta);
    }
    None
}

pub fn is_import_meta(node: &Arc<Node>) -> bool {
    if node.kind != SyntaxKind::MetaProperty {
        return false;
    }
    match &node.data {
        crate::ast::node_data_generated::NodeData::MetaProperty(d) => {
            d.keyword_token == SyntaxKind::ImportKeyword && d.name.text() == "meta"
        }
        _ => false,
    }
}

pub fn find_child_node(root: &Arc<Node>, check: impl Fn(&Arc<Node>) -> bool) -> Option<Arc<Node>> {
    let mut result: Option<Arc<Node>> = None;
    fn visit(node: &Arc<Node>, check: &impl Fn(&Arc<Node>) -> bool, result: &mut Option<Arc<Node>>) -> bool {
        if check(node) {
            *result = Some(node.clone());
            return true;
        }
        crate::ast::node_data_generated::for_each_child(node, |child| {
            visit(child, check, result)
        })
    }
    visit(root, &check, &mut result);
    result
}

pub fn is_file_module_from_using_jsx_tag(file: &SourceFile) -> Option<Arc<Node>> {
    walk_tree_for_jsx_tags(&file.node)
}

pub fn walk_tree_for_jsx_tags(node: &Arc<Node>) -> Option<Arc<Node>> {
    fn visitor(node: &Arc<Node>, found: &mut Option<Arc<Node>>) -> bool {
        if found.is_some() {
            return true;
        }
        if !node.subtree_facts().contains(SubtreeFacts::CONTAINS_JSX) {
            return false;
        }
        if is_jsx_opening_like_element(node) || node.kind == SyntaxKind::JsxFragment {
            *found = Some(node.clone());
            return true;
        }
        crate::ast::node_data_generated::for_each_child(node, |child| visitor(child, found))
    }
    let mut found: Option<Arc<Node>> = None;
    visitor(node, &mut found);
    found
}
