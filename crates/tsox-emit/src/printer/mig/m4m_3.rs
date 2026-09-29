#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_core::collections::ordered_set::OrderedSet;
use tsox_core::collections::set::Set;
use tsox_core::core::core::append_if_unique;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::{is_member_name, is_prologue_directive};
use tsox_frontend::ast::{is_block, is_function_declaration, is_variable_statement};
use tsox_frontend::ast::mig::m3g_2::is_parse_tree_node;

use crate::printer::mig::m4m_2::EmitNode;
use crate::printer::{AutoGenerateInfo, EmitContext, NodeFactory};

pub use tsox_frontend::format::mig::m4o::EmitFlags;
pub use tsox_frontend::format::mig::m4o_2::EmitHelper;

pub type EnvironmentFlags = u32;

pub const ENVIRONMENT_FLAGS_NONE: EnvironmentFlags = 0;
pub const ENVIRONMENT_FLAGS_IN_PARAMETERS: EnvironmentFlags = 1 << 0;
pub const ENVIRONMENT_FLAGS_VARIABLES_HOISTED_IN_PARAMETERS: EnvironmentFlags = 1 << 1;

#[derive(Default)]
pub struct VarScope {
    pub variables: Vec<Arc<Node>>,
    pub functions: Vec<Arc<Node>>,
    pub flags: EnvironmentFlags,
    pub initialization_statements: Vec<Arc<Node>>,
}

pub fn get_emit_context() -> (EmitContext, impl FnOnce(&mut EmitContext)) {
    let mut context = EmitContext::new();
    let reset = |c: &mut EmitContext| c.reset();
    let _ = &mut context;
    (context, reset)
}

impl EmitContext {
    pub fn factory(&self) -> NodeFactory<'_> {
        NodeFactory::new(self)
    }

    pub fn emit_nodes_try_get(&self, node: &Arc<Node>) -> Option<std::cell::Ref<'_, EmitNode>> {
        std::cell::Ref::filter_map(self.emit_nodes.borrow(), |m| {
            m.get(&(Arc::as_ptr(node) as *const Node))
        })
        .ok()
    }

    pub fn emit_nodes_get_mut(&self, node: &Arc<Node>) -> std::cell::RefMut<'_, EmitNode> {
        let ptr = Arc::as_ptr(node) as *const Node;
        if !self.emit_nodes.borrow().contains_key(&ptr) {
            self.emit_nodes.borrow_mut().insert(ptr, EmitNode::default());
        }
        std::cell::RefMut::map(self.emit_nodes.borrow_mut(), |m| m.get_mut(&ptr).unwrap())
    }

    pub fn get_auto_generate_info(&self, name: &Arc<Node>) -> Option<&AutoGenerateInfo> {
        self.auto_generate
            .get(&(Arc::as_ptr(name) as *const Node))
    }

    pub fn get_node_for_generated_name(&self, name: &Arc<Node>) -> Arc<Node> {
        if let Some(auto_generate) = self.get_auto_generate_info(name) {
            if auto_generate.flags.is_node() {
                if let Some(node) = auto_generate.node.clone() {
                    return self.get_node_for_generated_name_worker(&node, auto_generate.id);
                }
            }
        }
        name.clone()
    }
}
