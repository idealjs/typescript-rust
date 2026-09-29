#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use crate::printer::{EmitContext, GeneratedIdentifierFlags};
use crate::mig::m4n_2::r39k12_defs::R39K12EmitContextMapsExt;
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::format::mig::m4o_2::EmitHelper;

pub type EmitNodeFlags = u32;

pub const HAS_COMMENT_RANGE: EmitNodeFlags = 1 << 0;
pub const HAS_SOURCE_MAP_RANGE: EmitNodeFlags = 1 << 1;

#[derive(Clone)]
pub struct SnippetElement {
    pub kind: SnippetKind,
    pub order: usize,
}

pub type SnippetKind = u32;

pub const SNIPPET_KIND_TAB_STOP: SnippetKind = 0;

#[derive(Clone)]
pub struct SynthesizedComment {
    pub kind: SyntaxKind,
    pub loc: TextRange,
    pub has_leading_new_line: bool,
    pub has_trailing_new_line: bool,
    pub text: String,
}

#[derive(Default, Clone)]
pub struct EmitNode {
    pub flags: EmitNodeFlags,
    pub emit_flags: EmitFlags,
    pub comment_range: TextRange,
    pub source_map_range: TextRange,
    pub token_source_map_ranges: HashMap<SyntaxKind, TextRange>,
    pub helpers: Vec<Arc<EmitHelper>>,
    pub external_helpers_module_name: Option<Arc<Node>>,
    pub leading_comments: Vec<SynthesizedComment>,
    pub trailing_comments: Vec<SynthesizedComment>,
    pub type_node: Option<Arc<Node>>,
    pub snippet_element: Option<SnippetElement>,
}

impl EmitNode {
    fn copy_from(&mut self, source: &EmitNode) {
        self.flags = source.flags;
        self.emit_flags = source.emit_flags;
        self.comment_range = source.comment_range;
        self.source_map_range = source.source_map_range;
        self.token_source_map_ranges = source.token_source_map_ranges.clone();
        self.helpers = source.helpers.clone();
        self.external_helpers_module_name = source.external_helpers_module_name.clone();
        if let Some(snippet_element) = &source.snippet_element {
            self.snippet_element = Some(snippet_element.clone());
        }
    }
}

impl EmitContext {
    pub fn emit_flags(&self, node: &Arc<Node>) -> EmitFlags {
        if let Some(emit_node) = self.emit_nodes_try_get(node) {
            return emit_node.emit_flags;
        }
        EmitFlags::NONE
    }

    pub fn add_emit_flags(&self, node: &Arc<Node>, flags: EmitFlags) {
        self.emit_nodes_get_mut(node).emit_flags |= flags;
    }

    pub fn comment_range(&self, node: &Arc<Node>) -> TextRange {
        if let Some(emit_node) = self.emit_nodes_try_get(node) {
            if emit_node.flags & HAS_COMMENT_RANGE != 0 {
                return emit_node.comment_range;
            }
        }
        node.loc
    }

    pub fn assign_comment_range(&mut self, to: &Arc<Node>, from: &Arc<Node>) {
        let range = self.comment_range(from);
        self.set_comment_range(to, range);
    }

    pub fn assign_source_map_range(&mut self, to: &Arc<Node>, from: &Arc<Node>) {
        let range = self.source_map_range(from);
        self.set_source_map_range(to, range);
    }

    pub fn assign_comment_and_source_map_ranges(&self, to: &Arc<Node>, from: &Arc<Node>) {
        let comment_range = self.comment_range(from);
        let source_map_range = self.source_map_range(from);
        let mut emit_node = self.emit_nodes_get_mut(to);
        emit_node.comment_range = comment_range;
        emit_node.source_map_range = source_map_range;
        emit_node.flags |= HAS_COMMENT_RANGE | HAS_SOURCE_MAP_RANGE;
    }

    pub fn assigned_name(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        self.r39k12_assigned_name_get(node)
    }

    pub fn class_this(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        self.r39k12_class_this_get(node)
    }

    pub fn add_synthetic_leading_comment(
        &mut self,
        node: &Arc<Node>,
        kind: SyntaxKind,
        text: &str,
        has_trailing_new_line: bool,
    ) -> Arc<Node> {
        let comment = SynthesizedComment {
            kind,
            loc: TextRange::undefined(),
            has_leading_new_line: false,
            has_trailing_new_line,
            text: text.to_string(),
        };
        self.emit_nodes_get_mut(node).leading_comments.push(comment);
        node.clone()
    }

    pub fn add_synthetic_trailing_comment(
        &mut self,
        node: &Arc<Node>,
        kind: SyntaxKind,
        text: &str,
        has_trailing_new_line: bool,
    ) -> Arc<Node> {
        let comment = SynthesizedComment {
            kind,
            loc: TextRange::undefined(),
            has_leading_new_line: false,
            has_trailing_new_line,
            text: text.to_string(),
        };
        self.emit_nodes_get_mut(node).trailing_comments.push(comment);
        node.clone()
    }

    pub fn get_emit_helpers(&self, node: &Arc<Node>) -> Vec<Arc<EmitHelper>> {
        match self.emit_nodes_try_get(node) {
            Some(emit_node) => emit_node.helpers.clone(),
            None => vec![],
        }
    }

    pub fn get_external_helpers_module_name(&self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if let Some(parse_node) = self.parse_node(node) {
            if let Some(emit_node) = self.emit_nodes_try_get(&parse_node) {
                return emit_node.external_helpers_module_name.clone();
            }
        }
        None
    }
}
