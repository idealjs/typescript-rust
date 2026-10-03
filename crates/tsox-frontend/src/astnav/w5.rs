#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_core::core;

use crate::ast::{Node, NodeVisitor, NodeList, SourceFile};

pub fn get_position(
    node: &Arc<Node>,
    source_file: &SourceFile,
    allow_position_in_leading_trivia: bool,
) -> usize { ::tsox_core::fntrace::enter("get_position"); 
    if allow_position_in_leading_trivia {
        return node.pos();
    }
    crate::scanner::get_token_pos_of_node(node, source_file, true)
}

pub fn get_node_visitor(
    visit_node: Option<fn(&Arc<Node>, &NodeVisitor) -> Arc<Node>>,
    visit_nodes: Option<fn(&NodeList, &NodeVisitor) -> NodeList>,
) -> NodeVisitor { ::tsox_core::fntrace::enter("get_node_visitor"); 
    let wrapped_visit_node: Option<fn(&Arc<Node>, &NodeVisitor) -> Arc<Node>> =
        visit_node.map(|visit_node| {
            move |n: &Arc<Node>, v: &NodeVisitor| -> Arc<Node> {
                if crate::ast::is_jsdoc_single_comment_node_comment(n) {
                    return n.clone();
                }
                visit_node(n, v)
            }
        });

    let wrapped_visit_nodes: Option<fn(&NodeList, &NodeVisitor) -> NodeList> =
        visit_nodes.map(|visit_nodes| {
            move |n: &NodeList, v: &NodeVisitor| -> NodeList {
                if crate::ast::is_jsdoc_single_comment_node_list(n) {
                    return n.clone();
                }
                visit_nodes(n, v)
            }
        });

    NodeVisitor::new(
        core::identity,
        None,
        NodeVisitorHooks {
            visit_node: wrapped_visit_node,
            visit_token: wrapped_visit_node,
            visit_nodes: wrapped_visit_nodes,
            visit_modifiers: Some(|modifiers: &crate::ast::ModifierList, visitor: &NodeVisitor| {
                wrapped_visit_nodes.map(|f| f(&modifiers.node_list, visitor));
            }),
        },
    )
}
