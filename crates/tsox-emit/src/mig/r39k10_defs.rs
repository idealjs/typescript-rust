#![allow(dead_code, unused_imports, unused_variables)]

use std::cell::OnceCell;
use std::sync::Arc;

use tsox_frontend::ast::mig::m3c;
use tsox_frontend::ast::node::ModifierList;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_data_generated as ndg;
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::node::NodeList;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::visitor::NodeVisitor;

use crate::mig::m4l::LegacyDecoratorsTransformer;
use crate::mig::m4m_4::move_range_past_modifiers;
use crate::printer::{EmitContext, NodeFactory};

thread_local! {
    static FACTORY_EMIT_CONTEXT: OnceCell<&'static EmitContext> = const { OnceCell::new() };
}

fn factory_emit_context() -> &'static EmitContext {
    FACTORY_EMIT_CONTEXT.with(|cell| {
        *cell.get_or_init(|| Box::leak(Box::new(EmitContext::default())))
    })
}

pub struct R39K10Visitor {
    inner: tsox_frontend::ast::mig::m3c::NodeVisitor,
}

impl Default for R39K10Visitor {
    fn default() -> Self {
        Self {
            inner: tsox_frontend::ast::mig::m3c::NodeVisitor {
                factory: tsox_frontend::ast::mig::m3c::NodeFactory {
                    hooks: tsox_frontend::ast::mig::m3c::NodeFactoryHooks::default(),
                    text_count: 0,
                    node_count: 0,
                },
            },
        }
    }
}

impl R39K10Visitor {
    pub fn visit_node(&mut self, node: &Arc<Node>) -> Arc<Node> {
        Arc::clone(node)
    }

    pub fn visit_node_opt(&mut self, node: Option<&Arc<Node>>) -> Option<Arc<Node>> {
        node.cloned()
    }

    pub fn visit_modifiers(
        &mut self,
        modifiers: Option<&Arc<ModifierList>>,
    ) -> Option<Arc<ModifierList>> {
        modifiers.cloned()
    }

    pub fn visit_nodes(&mut self, nodes: Option<&NodeList>) -> NodeList {
        match nodes {
            Some(list) => {
                let mut new_list = NodeList::new(list.nodes.clone());
                new_list.loc = list.loc;
                new_list
            }
            None => NodeList::new(Vec::new()),
        }
    }

    pub fn visit_each_child(&mut self, node: &Arc<Node>) -> Arc<Node> {
        m3c::visit_each_child(node, &mut self.inner)
    }
}

pub trait R39K10LegacyDecoratorsExt {
    fn emit_context(&self) -> &'static mut EmitContext;
    fn factory(&self) -> NodeFactory<'static>;
    fn visitor(&mut self) -> R39K10Visitor;
    fn finish_class_element(&mut self, updated: &Arc<Node>, original: &Arc<Node>) -> Arc<Node>;
}

impl R39K10LegacyDecoratorsExt for LegacyDecoratorsTransformer {
    fn emit_context(&self) -> &'static mut EmitContext {
        Box::leak(Box::new(EmitContext::default()))
    }

    fn factory(&self) -> NodeFactory<'static> {
        NodeFactory::new(factory_emit_context())
    }

    fn visitor(&mut self) -> R39K10Visitor {
        R39K10Visitor::default()
    }

    fn finish_class_element(&mut self, updated: &Arc<Node>, original: &Arc<Node>) -> Arc<Node> {
        if !Arc::ptr_eq(updated, original) {
            self.emit_context_mut().set_comment_range(updated, original.loc);
            let source_map_range = move_range_past_modifiers(original);
            self.emit_context_mut()
                .set_source_map_range(updated, source_map_range);
        }
        Arc::clone(updated)
    }
}

pub fn r39k10_elide_modifiers(
    f: &NodeFactory,
    modifiers: Option<&Arc<ModifierList>>,
) -> Option<Arc<ModifierList>> {
    let modifiers = modifiers?;
    if modifiers.list.nodes.is_empty() {
        return Some(Arc::clone(modifiers));
    }
    let mut replacement = f.new_modifier_list(Vec::<Arc<Node>>::new());
    if let Some(replacement_list) = Arc::get_mut(&mut replacement) {
        replacement_list.list.loc = modifiers.list.loc;
    }
    Some(replacement)
}

pub trait R39K10NodeFactoryExt {
    fn new_class_static_block_declaration(
        &self,
        decorators: Option<Arc<ModifierList>>,
        body: Arc<Node>,
    ) -> Arc<Node>;
    fn update_property_access_expression(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
        question_dot_token: Option<&Arc<Node>>,
        name: &Arc<Node>,
        flags: NodeFlags,
    ) -> Arc<Node>;
}

impl R39K10NodeFactoryExt for NodeFactory<'_> {
    fn new_class_static_block_declaration(
        &self,
        decorators: Option<Arc<ModifierList>>,
        body: Arc<Node>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ClassStaticBlockDeclaration,
            NodeData::ClassStaticBlockDeclaration(ndg::ClassStaticBlockDeclarationData {
                modifiers: decorators,
                body,
            }),
        ))
    }

    fn update_property_access_expression(
        &self,
        node: &Arc<Node>,
        expression: &Arc<Node>,
        question_dot_token: Option<&Arc<Node>>,
        name: &Arc<Node>,
        flags: NodeFlags,
    ) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::PropertyAccessExpression,
            NodeData::PropertyAccessExpression(ndg::PropertyAccessExpressionData {
                expression: Arc::clone(expression),
                question_dot_token: question_dot_token.cloned(),
                name: Arc::clone(name),
            }),
        );
        updated.loc = node.loc;
        updated.flags = flags;
        Arc::new(updated)
    }
}

pub trait R39K10NodeExt {
    fn parameters(&self) -> Option<&Arc<NodeList>>;
    fn body(&self) -> Option<&Arc<Node>>;
    fn dot_dot_dot_token(&self) -> Option<&Arc<Node>>;
    fn initializer(&self) -> Option<&Arc<Node>>;
}

impl R39K10NodeExt for Node {
    fn parameters(&self) -> Option<&Arc<NodeList>> {
        match &self.data {
            NodeData::ConstructorDeclaration(d) => Some(&d.parameters),
            NodeData::MethodDeclaration(d) => Some(&d.parameters),
            NodeData::GetAccessorDeclaration(d) => Some(&d.parameters),
            NodeData::SetAccessorDeclaration(d) => Some(&d.parameters),
            NodeData::FunctionDeclaration(d) => Some(&d.parameters),
            NodeData::FunctionExpression(d) => Some(&d.parameters),
            _ => None,
        }
    }

    fn body(&self) -> Option<&Arc<Node>> {
        match &self.data {
            NodeData::ConstructorDeclaration(d) => d.body.as_ref(),
            NodeData::MethodDeclaration(d) => d.body.as_ref(),
            NodeData::GetAccessorDeclaration(d) => d.body.as_ref(),
            NodeData::SetAccessorDeclaration(d) => d.body.as_ref(),
            NodeData::FunctionDeclaration(d) => d.body.as_ref(),
            NodeData::FunctionExpression(d) => Some(&d.body),
            _ => None,
        }
    }

    fn dot_dot_dot_token(&self) -> Option<&Arc<Node>> {
        match &self.data {
            NodeData::ParameterDeclaration(d) => d.dot_dot_dot_token.as_ref(),
            _ => None,
        }
    }

    fn initializer(&self) -> Option<&Arc<Node>> {
        match &self.data {
            NodeData::ParameterDeclaration(d) => d.initializer.as_ref(),
            NodeData::PropertyDeclaration(d) => d.initializer.as_ref(),
            NodeData::VariableDeclaration(d) => d.initializer.as_ref(),
            _ => None,
        }
    }
}
