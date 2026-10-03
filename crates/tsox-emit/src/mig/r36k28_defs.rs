#![allow(unused_imports, dead_code)]

use std::sync::Arc;
use tsox_frontend::ast::node::{ModifierList, Node, NodeList};
use tsox_frontend::ast::node_data_generated::{
    ClassDeclarationData, ClassExpressionData, NodeData,
};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::visitor::NodeVisitor;
use tsox_frontend::ast::{is_prologue_directive, ModifierFlags};

use crate::printer::NodeFactory;

impl<'a> NodeFactory<'a> {
    pub fn update_class_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: Option<&Arc<Node>>,
        type_parameters: Option<Arc<NodeList>>,
        heritage_clauses: Option<&NodeList>,
        members: &NodeList,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_class_declaration"); 
        let mut updated = Node::new(
            SyntaxKind::ClassDeclaration,
            NodeData::ClassDeclaration(ClassDeclarationData {
                modifiers,
                name: name.cloned(),
                type_parameters,
                heritage_clauses: heritage_clauses
                    .map(|l| Arc::new(NodeList {
                        loc: l.loc,
                        nodes: l.nodes.clone(),
                    })),
                members: Arc::new(NodeList {
                    loc: members.loc,
                    nodes: members.nodes.clone(),
                }),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn update_class_expression(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: Option<&Arc<Node>>,
        type_parameters: Option<Arc<NodeList>>,
        heritage_clauses: Option<&NodeList>,
        members: &NodeList,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_class_expression"); 
        let mut updated = Node::new(
            SyntaxKind::ClassExpression,
            NodeData::ClassExpression(ClassExpressionData {
                modifiers,
                name: name.cloned(),
                type_parameters,
                heritage_clauses: heritage_clauses
                    .map(|l| Arc::new(NodeList {
                        loc: l.loc,
                        nodes: l.nodes.clone(),
                    })),
                members: Arc::new(NodeList {
                    loc: members.loc,
                    nodes: members.nodes.clone(),
                }),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    pub fn split_standard_prologue(
        &self,
        source: impl AsRef<[Arc<Node>]>,
    ) -> (Vec<Arc<Node>>, Vec<Arc<Node>>) { ::tsox_core::fntrace::enter("split_standard_prologue"); 
        let source = source.as_ref();
        for (i, statement) in source.iter().enumerate() {
            if !is_prologue_directive(statement) {
                return (source[..i].to_vec(), source[i..].to_vec());
            }
        }
        (source.to_vec(), Vec::new())
    }
}

pub trait R36K28NodeVisitorExt {
    fn visit_modifiers(&mut self, modifiers: Option<&Arc<ModifierList>>) -> Option<Arc<ModifierList>>;
    fn visit_nodes(&mut self, nodes: &Arc<NodeList>) -> Arc<NodeList>;
    fn visit_nodes_opt(&mut self, nodes: Option<&Arc<NodeList>>) -> Option<Arc<NodeList>>;
    fn visit_slice(&mut self, nodes: &[Arc<Node>]) -> (Vec<Arc<Node>>, bool);
}

impl R36K28NodeVisitorExt for NodeVisitor {
    fn visit_modifiers(&mut self, modifiers: Option<&Arc<ModifierList>>) -> Option<Arc<ModifierList>> { ::tsox_core::fntrace::enter("visit_modifiers"); 
        modifiers.map(|m| {
            let visited: Vec<Arc<Node>> = m.list.nodes.iter().map(|n| self.visit_node(n)).collect();
            Arc::new(ModifierList::new(visited, m.modifier_flags))
        })
    }

    fn visit_nodes(&mut self, nodes: &Arc<NodeList>) -> Arc<NodeList> { ::tsox_core::fntrace::enter("visit_nodes"); 
        Arc::new(NodeList::new(
            nodes.nodes.iter().map(|n| self.visit_node(n)).collect(),
        ))
    }

    fn visit_nodes_opt(&mut self, nodes: Option<&Arc<NodeList>>) -> Option<Arc<NodeList>> { ::tsox_core::fntrace::enter("visit_nodes_opt"); 
        nodes.map(|l| self.visit_nodes(l))
    }

    fn visit_slice(&mut self, nodes: &[Arc<Node>]) -> (Vec<Arc<Node>>, bool) { ::tsox_core::fntrace::enter("visit_slice"); 
        let visited: Vec<Arc<Node>> = nodes.iter().map(|n| self.visit_node(n)).collect();
        let changed = visited
            .iter()
            .zip(nodes.iter())
            .any(|(r, o)| !Arc::ptr_eq(r, o));
        (visited, changed)
    }
}
