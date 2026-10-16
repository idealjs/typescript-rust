#![allow(unused_imports)]
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use bitflags::bitflags;
use crate::ast::mig::m3b::is_array_literal_or_object_literal_destructuring_pattern;
use crate::ast::node::{ModifierList, Node, NodeList, SourceFile};
use crate::ast::node_data_generated::*;
use crate::ast::node_flags::{ModifierFlags, NodeFlags};
use crate::ast::syntax_kind_generated::SyntaxKind;
use crate::ast::utilities::*;
use tsox_core::core::text::TextRange;

pub use crate::ast::subtree_facts::{
    propagate_binding_element_subtree_facts, propagate_object_binding_element_subtree_facts,
    propagate_subtree_facts, SubtreeFacts, SubtreeFactsNone, SUBTREE_EXCLUSIONS_ACCESSOR,
    SUBTREE_EXCLUSIONS_ARROW_FUNCTION, SUBTREE_EXCLUSIONS_ARRAY_LITERAL,
    SUBTREE_EXCLUSIONS_BINDING_PATTERN, SUBTREE_EXCLUSIONS_CALL, SUBTREE_EXCLUSIONS_CATCH_CLAUSE,
    SUBTREE_EXCLUSIONS_CLASS, SUBTREE_EXCLUSIONS_CONSTRUCTOR, SUBTREE_EXCLUSIONS_ELEMENT_ACCESS,
    SUBTREE_EXCLUSIONS_FUNCTION, SUBTREE_EXCLUSIONS_METHOD, SUBTREE_EXCLUSIONS_MODULE,
    SUBTREE_EXCLUSIONS_NEW, SUBTREE_EXCLUSIONS_NODE, SUBTREE_EXCLUSIONS_OBJECT_LITERAL,
    SUBTREE_EXCLUSIONS_OUTER_EXPRESSION, SUBTREE_EXCLUSIONS_PARAMETER,
    SUBTREE_EXCLUSIONS_PROPERTY, SUBTREE_EXCLUSIONS_PROPERTY_ACCESS,
    SUBTREE_EXCLUSIONS_VARIABLE_DECLARATION_LIST, SUBTREE_FACTS_NONE,
};

#[derive(Default)]
pub struct NodeFactoryHooks {
    pub on_create: Option<Box<dyn Fn(&Node)>>,
    pub on_update: Option<Box<dyn Fn(&mut Node, &Node)>>,
    pub on_clone: Option<Box<dyn Fn(&Arc<Node>, &Arc<Node>)>>,
}

pub struct NodeFactory {
    pub node_count: usize,
    pub hooks: NodeFactoryHooks,
}

pub struct NodeVisitorHooks {
    pub visit_nodes:
        Option<Box<dyn Fn(Option<&Arc<NodeList>>, &NodeVisitor) -> Option<Arc<NodeList>>>>,
    pub visit_modifiers:
        Option<Box<dyn Fn(Option<&Arc<ModifierList>>, &NodeVisitor) -> Option<Arc<ModifierList>>>>,
}

impl Default for NodeVisitorHooks {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        NodeVisitorHooks { visit_nodes: None, visit_modifiers: None }
    }
}

pub struct NodeVisitor {
    callback: Box<dyn Fn(&Arc<Node>) -> Arc<Node>>,
    hooks: NodeVisitorHooks,
}

impl NodeVisitor {
    pub fn new(
        callback: impl Fn(&Arc<Node>) -> Arc<Node> + 'static,
        _factory: &NodeFactory,
        hooks: NodeVisitorHooks,
    ) -> NodeVisitor { ::tsox_core::fntrace::enter("new"); 
        NodeVisitor { callback: Box::new(callback), hooks }
    }

    pub fn visit_node(&self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_node"); 
        (self.callback)(node)
    }

    pub fn visit_each_child_node(&self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_each_child_node"); 
        (self.callback)(node)
    }

    pub fn visit_nodes(&self, nodes: &Arc<NodeList>) -> Arc<NodeList> { ::tsox_core::fntrace::enter("visit_nodes"); 
        if let Some(hook) = &self.hooks.visit_nodes {
            return hook(Some(nodes), self).unwrap_or_else(|| nodes.clone());
        }
        nodes.clone()
    }

    pub fn visit_modifiers(&self, nodes: &Arc<ModifierList>) -> Arc<ModifierList> { ::tsox_core::fntrace::enter("visit_modifiers"); 
        if let Some(hook) = &self.hooks.visit_modifiers {
            return hook(Some(nodes), self).unwrap_or_else(|| nodes.clone());
        }
        nodes.clone()
    }
}

pub fn visit(v: &mut dyn FnMut(&Node) -> bool, node: Option<&Node>) -> bool { ::tsox_core::fntrace::enter("visit"); 
    if let Some(node) = node {
        return v(node);
    }
    false
}

pub fn visit_nodes(v: &mut dyn FnMut(&Node) -> bool, nodes: &[Arc<Node>]) -> bool { ::tsox_core::fntrace::enter("visit_nodes"); 
    for node in nodes {
        if v(node) {
            return true;
        }
    }
    false
}

pub fn visit_node_list(v: &mut dyn FnMut(&Node) -> bool, node_list: Option<&NodeList>) -> bool { ::tsox_core::fntrace::enter("visit_node_list"); 
    if let Some(node_list) = node_list {
        return visit_nodes(v, &node_list.nodes);
    }
    false
}

pub fn visit_modifiers(v: &mut dyn FnMut(&Node) -> bool, modifiers: Option<&ModifierList>) -> bool { ::tsox_core::fntrace::enter("visit_modifiers"); 
    if let Some(modifiers) = modifiers {
        return visit_nodes(v, &modifiers.nodes);
    }
    false
}

pub fn new_node(kind: SyntaxKind, data: NodeData, hooks: &NodeFactoryHooks) -> Node { ::tsox_core::fntrace::enter("new_node"); 
    let mut n = Node::new(kind, data);
    n.loc = TextRange::undefined();
    if let Some(on_create) = &hooks.on_create {
        on_create(&n);
    }
    n
}

impl NodeFactory {
    pub fn new_node(&mut self, kind: SyntaxKind, data: NodeData) -> Node { ::tsox_core::fntrace::enter("new_node"); 
        self.node_count += 1;
        new_node(kind, data, &self.hooks)
    }
}

pub fn update_node<'a>(
    updated: &'a mut Node,
    original: &Node,
    hooks: &NodeFactoryHooks,
) -> &'a mut Node { ::tsox_core::fntrace::enter("update_node"); 
    if !std::ptr::eq(updated as *const Node, original as *const Node) {
        updated.flags = original.flags;
        updated.loc = original.loc;
        if let Some(on_update) = &hooks.on_update {
            on_update(updated, original);
        }
    }
    updated
}

pub enum AccessKind {
    Read,
    Write,
    ReadWrite,
}

pub fn reverse_access_kind(a: AccessKind) -> AccessKind { ::tsox_core::fntrace::enter("reverse_access_kind"); 
    match a {
        AccessKind::Read => AccessKind::Write,
        AccessKind::Write => AccessKind::Read,
        AccessKind::ReadWrite => AccessKind::ReadWrite,
    }
}

pub fn declaration_is_write_access(decl: Option<&Node>) -> bool { ::tsox_core::fntrace::enter("declaration_is_write_access"); 
    let Some(decl) = decl else {
        return false;
    };
    if decl.flags.intersects(NodeFlags::JavaScriptFile) && decl.flags.intersects(NodeFlags::Ambient) {
        return true;
    }
    if decl.flags.intersects(NodeFlags::Ambient) {
        return true;
    }
    match decl.kind {
        SyntaxKind::BinaryExpression
        | SyntaxKind::BindingElement
        | SyntaxKind::ClassDeclaration
        | SyntaxKind::ClassExpression
        | SyntaxKind::DefaultKeyword
        | SyntaxKind::EnumDeclaration
        | SyntaxKind::EnumMember
        | SyntaxKind::ExportSpecifier
        | SyntaxKind::ImportClause
        | SyntaxKind::ImportEqualsDeclaration
        | SyntaxKind::ImportSpecifier
        | SyntaxKind::InterfaceDeclaration
        | SyntaxKind::JSDocCallbackTag
        | SyntaxKind::JSDocTypedefTag
        | SyntaxKind::JsxAttribute
        | SyntaxKind::ModuleDeclaration
        | SyntaxKind::NamespaceExportDeclaration
        | SyntaxKind::NamespaceImport
        | SyntaxKind::NamespaceExport
        | SyntaxKind::Parameter
        | SyntaxKind::ShorthandPropertyAssignment
        | SyntaxKind::TypeAliasDeclaration
        | SyntaxKind::JSTypeAliasDeclaration
        | SyntaxKind::TypeParameter => true,
        SyntaxKind::PropertyAssignment => {
            !decl
                .parent()
                .is_some_and(|p| is_array_literal_or_object_literal_destructuring_pattern(&p))
        }
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::Constructor
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor => {
            let body = match &decl.data {
                NodeData::FunctionDeclaration(d) => d.body.as_ref(),
                NodeData::FunctionExpression(d) => Some(&d.body),
                NodeData::ConstructorDeclaration(d) => d.body.as_ref(),
                NodeData::MethodDeclaration(d) => d.body.as_ref(),
                NodeData::GetAccessorDeclaration(d) => d.body.as_ref(),
                NodeData::SetAccessorDeclaration(d) => d.body.as_ref(),
                _ => None,
            };
            body.is_some()
        }
        SyntaxKind::VariableDeclaration | SyntaxKind::PropertyDeclaration => {
            let has_init = match &decl.data {
                NodeData::VariableDeclaration(d) => d.initializer.is_some(),
                NodeData::PropertyDeclaration(d) => d.initializer.is_some(),
                _ => false,
            };
            has_init
                || decl
                    .parent()
                    .as_deref()
                    .is_some_and(|p| is_catch_clause(p))
        }
        SyntaxKind::MethodSignature
        | SyntaxKind::PropertySignature
        | SyntaxKind::JSDocPropertyTag
        | SyntaxKind::JSDocParameterTag => false,
        _ => panic!("Unhandled case in declarationIsWriteAccess"),
    }
}

