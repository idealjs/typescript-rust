#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;
use tsox_frontend::ast::node_data_generated::{IndexSignatureDeclarationData, NodeData};
use tsox_frontend::ast::{ModifierList, Node, NodeList, SyntaxKind};

use crate::checker::nodecopy_builder::NodeFactoryStub;
use crate::checker::mig::m2f::r17k8_factory_ext::NodeFactoryExt;

pub trait R23K8NodeFactoryExt {
    fn new_modifier(&self, kind: SyntaxKind) -> Arc<Node>;
    fn new_node_list(&self, nodes: Vec<Arc<Node>>) -> NodeList;
    fn new_index_signature_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        parameters: NodeList,
        type_node: Option<Arc<Node>>,
    ) -> Arc<Node>;
}

impl R23K8NodeFactoryExt for NodeFactoryStub {
    fn new_modifier(&self, kind: SyntaxKind) -> Arc<Node> { ::tsox_core::fntrace::enter("new_modifier"); 
        NodeFactoryExt::new_modifier(self, kind)
    }

    fn new_node_list(&self, nodes: Vec<Arc<Node>>) -> NodeList { ::tsox_core::fntrace::enter("new_node_list"); 
        NodeFactoryExt::new_node_list(self, nodes)
    }

    fn new_index_signature_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        parameters: NodeList,
        type_node: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_index_signature_declaration"); 
        Arc::new(Node::new(
            SyntaxKind::IndexSignature,
            NodeData::IndexSignatureDeclaration(IndexSignatureDeclarationData {
                modifiers,
                parameters: Arc::new(parameters),
                type_node: type_node
                    .unwrap_or_else(|| Arc::new(Node::new(SyntaxKind::UnknownKeyword, NodeData::Token))),
            }),
        ))
    }
}
