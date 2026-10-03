#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::{Arc, OnceLock};

use tsox_checker::checker::mig::m2d::EmitResolver;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated as ndg;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::node::ModifierList;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use crate::mig::m4l_6::TypeEraserTransformer;
use crate::mig::m4h_8::r36k26_defs::EmitContextCommentRangeR36k26;
use crate::printer::{EmitContext, NodeFactory};

fn k3_shared_emit_context() -> &'static EmitContext { ::tsox_core::fntrace::enter("k3_shared_emit_context"); 
    std::thread_local! {
        static CTX: std::cell::OnceCell<&'static EmitContext> = const { std::cell::OnceCell::new() };
    }
    CTX.with(|cell| *cell.get_or_init(|| Box::leak(Box::new(EmitContext::default()))))
}

pub trait TypeEraserK3Ext {
    fn factory(&self) -> NodeFactory<'static>;
    fn visitor(&mut self) -> K3Visitor;
    fn emit_context(&self) -> Arc<EmitContext>;
}

impl TypeEraserK3Ext for TypeEraserTransformer {
    fn factory(&self) -> NodeFactory<'static> { ::tsox_core::fntrace::enter("factory"); 
        NodeFactory::new(k3_shared_emit_context())
    }

    fn visitor(&mut self) -> K3Visitor { ::tsox_core::fntrace::enter("visitor"); 
        K3Visitor
    }

    fn emit_context(&self) -> Arc<EmitContext> { ::tsox_core::fntrace::enter("emit_context"); 
        Arc::new(EmitContext::default())
    }
}

pub struct K3Visitor;

impl K3Visitor {
    pub fn visit_node(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_node"); 
        Arc::clone(node)
    }

    pub fn visit_node_opt(&mut self, node: Option<&Arc<Node>>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_node_opt"); 
        node.cloned()
    }

    pub fn visit_modifiers(
        &mut self,
        modifiers: Option<&Arc<ModifierList>>,
    ) -> Option<Arc<ModifierList>> { ::tsox_core::fntrace::enter("visit_modifiers"); 
        modifiers.cloned()
    }

    pub fn visit_nodes(&mut self, nodes: Option<&Arc<NodeList>>) -> Arc<NodeList> { ::tsox_core::fntrace::enter("visit_nodes"); 
        match nodes {
            Some(list) => Arc::clone(list),
            None => Arc::new(NodeList::new(Vec::new())),
        }
    }

    pub fn visit_slice(&mut self, nodes: Vec<Arc<Node>>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("visit_slice"); 
        nodes
    }

    pub fn visit_each_child(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_each_child"); 
        Arc::clone(node)
    }
}

pub trait K3NodeAccessExt {
    fn asterisk_token(&self) -> Option<&Arc<Node>>;
    fn token(&self) -> SyntaxKind;
    fn types(&self) -> Option<&Arc<NodeList>>;
    fn heritage_clauses(&self) -> Option<&Arc<NodeList>>;
    fn members(&self) -> Option<&Arc<NodeList>>;
    fn tag_name(&self) -> &Arc<Node>;
    fn attributes(&self) -> &Arc<Node>;
    fn decorators(&self) -> Vec<Arc<Node>>;
    fn symbol_declarations_k3(&self) -> Vec<Arc<Node>>;
}

impl K3NodeAccessExt for Node {
    fn asterisk_token(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("asterisk_token"); 
        match &self.data {
            NodeData::MethodDeclaration(d) => d.asterisk_token.as_ref(),
            NodeData::FunctionDeclaration(d) => d.asterisk_token.as_ref(),
            NodeData::FunctionExpression(d) => d.asterisk_token.as_ref(),
            _ => None,
        }
    }

    fn token(&self) -> SyntaxKind { ::tsox_core::fntrace::enter("token"); 
        match &self.data {
            NodeData::HeritageClause(d) => d.token,
            _ => SyntaxKind::Unknown,
        }
    }

    fn types(&self) -> Option<&Arc<NodeList>> { ::tsox_core::fntrace::enter("types"); 
        match &self.data {
            NodeData::HeritageClause(d) => Some(&d.types),
            _ => None,
        }
    }

    fn heritage_clauses(&self) -> Option<&Arc<NodeList>> { ::tsox_core::fntrace::enter("heritage_clauses"); 
        match &self.data {
            NodeData::ClassDeclaration(d) => d.heritage_clauses.as_ref(),
            NodeData::ClassExpression(d) => d.heritage_clauses.as_ref(),
            _ => None,
        }
    }

    fn members(&self) -> Option<&Arc<NodeList>> { ::tsox_core::fntrace::enter("members"); 
        match &self.data {
            NodeData::ClassDeclaration(d) => Some(&d.members),
            NodeData::ClassExpression(d) => Some(&d.members),
            _ => None,
        }
    }

    fn tag_name(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("tag_name"); 
        match &self.data {
            NodeData::JsxOpeningElement(d) => &d.tag_name,
            NodeData::JsxSelfClosingElement(d) => &d.tag_name,
            _ => panic!("tag_name() on {:?}", self.kind),
        }
    }

    fn attributes(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("attributes"); 
        match &self.data {
            NodeData::JsxOpeningElement(d) => &d.attributes,
            NodeData::JsxSelfClosingElement(d) => &d.attributes,
            _ => panic!("attributes() on {:?}", self.kind),
        }
    }

    fn decorators(&self) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("decorators"); 
        self.modifier_nodes()
            .iter()
            .filter(|m| m.kind == SyntaxKind::Decorator)
            .cloned()
            .collect()
    }

    fn symbol_declarations_k3(&self) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("symbol_declarations_k3"); 
        Vec::new()
    }
}

pub trait K3EmitResolverExt {
    fn is_declaration_visible(&self, node: &Arc<Node>) -> bool;
}

impl K3EmitResolverExt for EmitResolver {
    fn is_declaration_visible(&self, _node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_declaration_visible"); 
        false
    }
}
