#![allow(unused_imports)]
#![allow(dead_code)]

#[path = "r36k31_defs.rs"]
pub mod r36k31_defs;

#[path = "r37k19_defs.rs"]
pub mod r37k19_defs;

use self::r37k19_defs::R37K19ArcNodeExt;

use std::collections::HashMap;
use std::sync::Arc;

use super::m4q::r33k12_defs::Set;
use super::m4q::r33k12_defs::{concatenate, every, is_parse_tree_node, splice};
use tsox_core::collections::ordered_set::OrderedSet;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    is_call_expression, is_identifier, is_private_identifier,
};
use tsox_frontend::ast::is_member_name;
use tsox_frontend::ast::node_flags::NodeFlags;

use tsox_frontend::ast::visitor::{NodeVisitor, NodeVisitorHooks};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::is_prologue_directive;
use super::m4q::r33k12_defs::append_if_unique_vec;

use crate::printer::{AutoGenerateId, AutoGenerateInfo, EmitContext, GeneratedIdentifierFlags};
use tsox_frontend::format::mig::m4o_2::EmitHelper;
use crate::printer::mig::m4m_2::SynthesizedComment;
use super::m4q::r33k12_defs::{EmitFlags, NodeFactory};
use crate::printer::mig::m4m_3::{
    VarScope, ENVIRONMENT_FLAGS_IN_PARAMETERS, ENVIRONMENT_FLAGS_VARIABLES_HOISTED_IN_PARAMETERS,
};

pub type EmitNodeFlags = u32;

pub const HAS_COMMENT_RANGE: EmitNodeFlags = 1 << 0;
pub const HAS_SOURCE_MAP_RANGE: EmitNodeFlags = 1 << 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnippetKind {
    TabStop,
}

impl EmitContext {
    pub fn get_synthetic_leading_comments(&self, node: &Arc<Node>) -> Vec<SynthesizedComment> { ::tsox_core::fntrace::enter("get_synthetic_leading_comments"); 
        self.emit_nodes_try_get(node)
            .map(|emit_node| emit_node.leading_comments.clone())
            .unwrap_or_default()
    }

    pub fn get_synthetic_trailing_comments(&self, node: &Arc<Node>) -> Vec<SynthesizedComment> { ::tsox_core::fntrace::enter("get_synthetic_trailing_comments"); 
        self.emit_nodes_try_get(node)
            .map(|emit_node| emit_node.trailing_comments.clone())
            .unwrap_or_default()
    }

    pub fn get_type_node(&self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_type_node"); 
        self.emit_nodes_try_get(node)
            .and_then(|emit_node| emit_node.type_node.clone())
    }

    pub fn has_auto_generate_info(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_auto_generate_info"); 
        self.get_auto_generate_info(node).is_some()
    }

    pub fn has_recorded_external_helpers(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_recorded_external_helpers"); 
        if let Some(parse_node) = self.parse_node(node) {
            if let Some(emit_node) = self.emit_nodes_try_get(&parse_node) {
                return emit_node.external_helpers_module_name.is_some()
                    || emit_node
                        .emit_flags
                        .contains(EmitFlags::EXTERNAL_HELPERS);
            }
        }
        false
    }

    pub fn is_call_to_helper(&self, first_segment: &Arc<Node>, helper_name: &str) -> bool { ::tsox_core::fntrace::enter("is_call_to_helper"); 
        is_call_expression(first_segment)
            && first_segment.expression().is_some_and(|expression| {
                is_identifier(expression)
                    && self
                        .emit_flags(expression)
                        .contains(EmitFlags::HELPER_NAME)
                    && expression.text() == helper_name
            })
    }

    pub fn is_file_level_unique_name(
        &self,
        source_file: &Arc<Node>,
        name: &str,
        has_global_name: Option<&dyn Fn(&str) -> bool>,
    ) -> bool { ::tsox_core::fntrace::enter("is_file_level_unique_name"); 
        if let Some(has_global_name) = has_global_name {
            if has_global_name(name) {
                return false;
            }
        }
        let source_file = self.most_original(source_file);
        let _ = (&source_file, name);
        true
    }

    pub fn merge_environment(
        &mut self,
        statements: Vec<Arc<Node>>,
        declarations: Vec<Arc<Node>>,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("merge_environment"); 
        let (result, _) = self.merge_environment_inner(&statements, &declarations);
        result
    }

    pub fn merge_environment_list(
        &mut self,
        statements: &Arc<tsox_frontend::ast::NodeList>,
        declarations: Vec<Arc<Node>>,
    ) -> Arc<tsox_frontend::ast::NodeList> { ::tsox_core::fntrace::enter("merge_environment_list"); 
        let (result, changed) = self.merge_environment_inner(&statements.nodes, &declarations);
        if changed {
            let mut list = tsox_frontend::ast::NodeList::new(result);
            list.loc = statements.loc;
            return Arc::new(list);
        }
        Arc::clone(statements)
    }

    pub fn most_original(&self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("most_original"); 
        let mut node = Arc::clone(node);
        while let Some(original) = self.original(&node) {
            node = original;
        }
        node
    }

    pub fn move_emit_helpers(
        &mut self,
        source: &Arc<Node>,
        target: &Arc<Node>,
        predicate: impl Fn(&EmitHelper) -> bool,
    ) { ::tsox_core::fntrace::enter("move_emit_helpers"); 
        let source_helpers = match self.emit_nodes_try_get(source) {
            Some(source_emit_node) => source_emit_node.helpers.clone(),
            None => return,
        };
        if source_helpers.is_empty() {
            return;
        }

        let mut helpers_removed = 0usize;
        for i in 0..source_helpers.len() {
            let helper = Arc::clone(&source_helpers[i]);
            let already_present = self
                .emit_nodes_try_get(target)
                .is_some_and(|t| t.helpers.iter().any(|h| Arc::ptr_eq(h, &helper)));
            if predicate(&helper) && !already_present {
                helpers_removed += 1;
                let mut target_emit_node = self.emit_nodes_get_mut(target);
                target_emit_node.helpers.push(helper);
            } else if helpers_removed > 0 {
                self.emit_nodes_get_mut(source).helpers[i - helpers_removed] = helper;
            }
        }

        if helpers_removed > 0 {
            let new_len = source_helpers.len() - helpers_removed;
            self.emit_nodes_get_mut(source).helpers.truncate(new_len);
        }
    }

    pub fn new_emit_context() -> Box<EmitContext> { ::tsox_core::fntrace::enter("new_emit_context"); 
        Box::new(EmitContext::default())
    }

    pub fn new_node_visitor<T>(
        &self,
        visit: impl Fn(&mut T, Arc<Node>) -> Option<Arc<Node>>,
    ) -> NodeVisitor { ::tsox_core::fntrace::enter("new_node_visitor"); 
        let _ = visit;
        NodeVisitor::default()
    }

    pub fn new_not_emitted_statement(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_not_emitted_statement"); 
        let mut statement = self.factory().new_not_emitted_statement();
        statement.set_loc(node.loc);
        self.set_original(&statement, node);
        self.assign_comment_range(&statement, node);
        statement
    }

    pub fn original(&self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("original"); 
        let _ = node;
        None
    }

    pub fn parse_node(&self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("parse_node"); 
        let node = self.most_original(node);
        if is_parse_tree_node(&node) {
            Some(node)
        } else {
            None
        }
    }

    pub fn read_emit_helpers(&mut self) -> Vec<Arc<EmitHelper>> { ::tsox_core::fntrace::enter("read_emit_helpers"); 
        Vec::new()
    }

    pub fn request_emit_helper(&mut self, helper: &'static Arc<EmitHelper>) { ::tsox_core::fntrace::enter("request_emit_helper"); 
        let _ = helper;
    }

    pub fn reset(&mut self) { ::tsox_core::fntrace::enter("reset"); }
}
