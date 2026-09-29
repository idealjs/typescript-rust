use std::sync::Arc;
use tsox_frontend::ast::node_data_generated::{
    EnumDeclarationData, EnumMemberData, NodeData, NumericLiteralData, StringLiteralData,
};
use tsox_frontend::ast::{modifiers_to_flags, ModifierList, Node, NodeList, SyntaxKind};

use crate::checker::nodecopy_builder::NodeFactoryStub;

pub trait NodeFactoryExt {
    fn new_identifier(&self, text: &str) -> Arc<Node>;
    fn new_string_literal(&self, text: &str, token_flags: i32) -> Arc<Node>;
    fn new_numeric_literal(&self, text: &str, token_flags: i32) -> Arc<Node>;
    fn new_modifier(&self, kind: SyntaxKind) -> Arc<Node>;
    fn new_modifier_list(&self, nodes: Vec<Arc<Node>>) -> Option<Arc<ModifierList>>;
    fn new_node_list(&self, nodes: Vec<Arc<Node>>) -> NodeList;
    fn new_enum_member(&self, name: Arc<Node>, initializer: Option<Arc<Node>>) -> Arc<Node>;
    fn new_enum_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        members: NodeList,
    ) -> Arc<Node>;
}

impl NodeFactoryExt for NodeFactoryStub {
    fn new_identifier(&self, text: &str) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::Identifier,
            NodeData::Identifier(tsox_frontend::ast::node_data_generated::IdentifierData {
                text: text.to_string(),
            }),
        ))
    }

    fn new_string_literal(&self, text: &str, _token_flags: i32) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::StringLiteral,
            NodeData::StringLiteral(StringLiteralData {
                text: text.to_string(),
                token_flags: Default::default(),
            }),
        ))
    }

    fn new_numeric_literal(&self, text: &str, _token_flags: i32) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::NumericLiteral,
            NodeData::NumericLiteral(NumericLiteralData {
                text: text.to_string(),
                token_flags: Default::default(),
            }),
        ))
    }

    fn new_modifier(&self, kind: SyntaxKind) -> Arc<Node> {
        Arc::new(Node::new(kind, NodeData::Token))
    }

    fn new_modifier_list(&self, nodes: Vec<Arc<Node>>) -> Option<Arc<ModifierList>> {
        let flags = modifiers_to_flags(&nodes);
        Some(Arc::new(ModifierList::new(nodes, flags)))
    }

    fn new_node_list(&self, nodes: Vec<Arc<Node>>) -> NodeList {
        NodeList::new(nodes)
    }

    fn new_enum_member(&self, name: Arc<Node>, initializer: Option<Arc<Node>>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::EnumMember,
            NodeData::EnumMember(EnumMemberData { name, initializer }),
        ))
    }

    fn new_enum_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        members: NodeList,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::EnumDeclaration,
            NodeData::EnumDeclaration(EnumDeclarationData {
                modifiers,
                name,
                members: Arc::new(members),
            }),
        ))
    }
}

pub fn replace_modifiers(
    _factory: &NodeFactoryStub,
    node: &Arc<Node>,
    modifier_array: Option<Arc<ModifierList>>,
) -> Arc<Node> {
    let frontend_factory = tsox_frontend::ast::mig::m3b_2::NodeFactory::new();
    tsox_frontend::ast::mig::m3g_3::replace_modifiers(&frontend_factory, node, modifier_array)
}
