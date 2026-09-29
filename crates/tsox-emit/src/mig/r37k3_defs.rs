#![allow(dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::{Node, NodeList};
use tsox_frontend::ast::node_data_generated::{NodeData, SourceFileData};
use tsox_frontend::ast::SyntaxKind;

use super::{CjsVisitorKind, CommonJsModuleTransformer};
use crate::mig::m4e::r37k1_defs::R37K1DataExt;

pub fn update_source_file_node(
    node: &Arc<Node>,
    statements: Arc<NodeList>,
    end_of_file_token: Arc<Node>,
) -> Arc<Node> {
    match &node.data {
        NodeData::SourceFile(d) => {
            let mut updated = Node::new(
                SyntaxKind::SourceFile,
                NodeData::SourceFile(SourceFileData {
                    statements,
                    end_of_file_token,
                    global_exports: d.global_exports.clone(),
                }),
            );
            updated.flags = node.flags;
            updated.loc = node.loc;
            Arc::new(updated)
        }
        _ => node.clone(),
    }
}

pub struct CjsNodeVisitor<'a> {
    pub tx: &'a mut CommonJsModuleTransformer,
    pub kind: CjsVisitorKind,
}

impl<'a> CjsNodeVisitor<'a> {
    pub fn visit_node(&mut self, node: Option<&Arc<Node>>) -> Option<Arc<Node>> {
        let node = node?;
        match self.kind {
            CjsVisitorKind::TopLevel => self.tx.visit(node),
            CjsVisitorKind::TopLevelNested => self.tx.visit_no_stack(node, false),
            CjsVisitorKind::DiscardedValue => self.tx.visit_discarded_value(node),
            CjsVisitorKind::AssignmentPattern => self.tx.visit_assignment_pattern(node),
        }
    }

    pub fn visit_slice(&mut self, nodes: &[Arc<Node>]) -> (Vec<Arc<Node>>, bool) {
        let mut changed = false;
        let mut result = Vec::with_capacity(nodes.len());
        for node in nodes {
            match self.visit_node(Some(node)) {
                Some(visited) => {
                    changed |= !Arc::ptr_eq(&visited, node);
                    result.push(visited);
                }
                None => changed = true,
            }
        }
        (result, changed)
    }

    pub fn visit_each_child(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if node.kind == SyntaxKind::SourceFile {
            let data = node.as_source_file();
            let (statements, statements_changed) = self.visit_slice(&data.statements.nodes);
            let eof = data.end_of_file_token.clone();
            let eof_visited = self.visit_node(Some(&eof));
            let eof_changed = eof_visited.as_ref().map_or(true, |v| !Arc::ptr_eq(v, &eof));
            let eof = eof_visited.unwrap_or(eof);
            if !statements_changed && !eof_changed {
                return Some(node.clone());
            }
            return Some(update_source_file_node(
                node,
                Arc::new(NodeList::new(statements)),
                eof,
            ));
        }

        let mut changed = false;
        let mut visited_children: Vec<Option<Arc<Node>>> = Vec::new();
        tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
            let visited = self.visit_node(Some(child));
            changed |= visited.as_ref().map_or(true, |v| !Arc::ptr_eq(v, child));
            visited_children.push(visited);
            true
        });
        if !changed {
            return Some(node.clone());
        }
        panic!(
            "CjsNodeVisitor visit_each_child rebuild for kind {:?} pending Go factory update-function port (progress_notes_r37k3.md)",
            node.kind
        )
    }
}
