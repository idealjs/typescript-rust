#![allow(unused_imports)]
use std::sync::Arc;
use tsox_frontend::ast::{for_each_child, is_function_like, is_super_call, Node, NodeData, SyntaxKind};
use tsox_frontend::ast::node_data_generated::IfStatementData;
use tsox_core::core::compiler_options::ModuleKind;
use crate::checker::checker_checker::*;
use crate::checker::types_impl_chunk_3::IndexInfo;

pub trait R24K5NodeExt {
    fn as_if_statement(&self) -> &IfStatementData;
}

impl R24K5NodeExt for Node {
    fn as_if_statement(&self) -> &IfStatementData {
        match &self.data {
            NodeData::IfStatement(d) => d,
            _ => panic!("Expected IfStatement node"),
        }
    }
}

impl Checker {
    pub fn get_node_check_flags(&mut self, node: &Arc<Node>) -> NodeCheckFlags {
        self.node_links.get_or_default(node).flags
    }

    pub fn index_info_is_any_base_type(&self, info: &IndexInfo) -> bool {
        info.key_type
            .as_ref()
            .is_some_and(|k| Arc::ptr_eq(k, &self.any_type()))
    }
}

// Go core.ModuleKind String（modulekind_stringer_generated.go）
pub fn module_kind_string(kind: ModuleKind) -> String {
    match kind {
        ModuleKind::None => "None",
        ModuleKind::CommonJS => "CommonJS",
        ModuleKind::AMD => "AMD",
        ModuleKind::UMD => "UMD",
        ModuleKind::System => "System",
        ModuleKind::ES2015 => "ES2015",
        ModuleKind::ES2020 => "ES2020",
        ModuleKind::ES2022 => "ES2022",
        ModuleKind::ESNext => "ESNext",
        ModuleKind::Node16 => "Node16",
        ModuleKind::Node18 => "Node18",
        ModuleKind::Node20 => "Node20",
        ModuleKind::NodeNext => "NodeNext",
        ModuleKind::Preserve => "Preserve",
    }
    .to_string()
}

// Go findFirstSuperCall（checker.go:2932）：不进入嵌套函数体
pub fn find_first_super_call(node: &Arc<Node>) -> Option<Arc<Node>> {
    if is_super_call(node) {
        return Some(Arc::clone(node));
    }
    if is_function_like(node) || node.kind == SyntaxKind::ClassStaticBlockDeclaration {
        return None;
    }
    let mut found: Option<Arc<Node>> = None;
    for_each_child(node, |child| {
        if found.is_none() {
            found = find_first_super_call(child);
        }
        found.is_some()
    });
    found
}

// Go nodeImmediatelyReferencesSuperOrThis（checker_classes_ctor_super_calls.go）
pub fn node_immediately_references_super_or_this(node: &Arc<Node>) -> bool {
    match node.kind {
        SyntaxKind::SuperKeyword | SyntaxKind::ThisKeyword => true,
        SyntaxKind::ArrowFunction
        | SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::PropertyDeclaration => false,
        SyntaxKind::Block => {
            if let Some(parent) = node.parent()
                && matches!(
                    parent.kind,
                    SyntaxKind::Constructor
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor
                )
            {
                return false;
            }
            node_immediately_references_children(node)
        }
        _ => node_immediately_references_children(node),
    }
}

fn node_immediately_references_children(node: &Arc<Node>) -> bool {
    let mut found = false;
    for_each_child(node, |child| {
        if !found {
            found = node_immediately_references_super_or_this(child);
        }
        found
    });
    found
}
