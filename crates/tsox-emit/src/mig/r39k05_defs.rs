#![allow(unused_imports)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated as ndg;
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{ModifierList, NodeList};
use tsox_frontend::ast::visitor::NodeVisitor;

use crate::mig::m4h_5::EsDecoratorTransformer;
use crate::printer::{EmitContext, NodeFactory};

impl EsDecoratorTransformer {
    pub fn factory(&self) -> NodeFactory<'_> { ::tsox_core::fntrace::enter("factory"); 
        self.transformer.factory()
    }

    pub fn emit_context(&self) -> EmitContext { ::tsox_core::fntrace::enter("emit_context"); 
        self.transformer.emit_context()
    }

    pub fn visit_property_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_property_declaration"); 
        Some(node.clone())
    }

    pub fn visit_class_static_block_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_class_static_block_declaration"); 
        Some(node.clone())
    }
}

pub trait R39k05NodeVisitorExt {
    fn visit_each_child(&mut self, node: &Arc<Node>) -> Option<Arc<Node>>;

    fn visit_modifiers(&mut self, modifiers: Option<&NodeList>) -> Option<Arc<ModifierList>>;
}

impl R39k05NodeVisitorExt for NodeVisitor {
    fn visit_each_child(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_each_child"); 
        Some(self.visit_node(node))
    }

    fn visit_modifiers(&mut self, modifiers: Option<&NodeList>) -> Option<Arc<ModifierList>> { ::tsox_core::fntrace::enter("visit_modifiers"); 
        let modifiers = modifiers?;
        let kept: Vec<Arc<Node>> = modifiers
            .nodes
            .iter()
            .filter(|n| n.kind == SyntaxKind::StaticKeyword)
            .cloned()
            .collect();
        Some(Arc::new(ModifierList::new(kept, ModifierFlags::empty())))
    }
}

pub fn new_function_call_call_r39k05(
    f: &NodeFactory,
    target: &Arc<Node>,
    this_arg: &Arc<Node>,
    arguments_list: &[Arc<Node>],
) -> Arc<Node> { ::tsox_core::fntrace::enter("new_function_call_call_r39k05"); 
    let mut args = Vec::with_capacity(1 + arguments_list.len());
    args.push(Arc::clone(this_arg));
    args.extend(arguments_list.iter().cloned());
    f.new_call_expression(
        &f.new_property_access_expression(target, None, &f.new_identifier("call"), NodeFlags::empty()),
        None,
        None,
        f.new_node_list(args),
        NodeFlags::empty(),
    )
}

pub fn new_string_literal_from_node_r39k05(f: &NodeFactory, text_source_node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_string_literal_from_node_r39k05"); 
    f.new_string_literal(text_source_node.text(), 0)
}

pub fn new_set_function_name_helper_r39k05(
    f: &NodeFactory,
    constructor: &Arc<Node>,
    name: &Arc<Node>,
    prefix: Option<&str>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("new_set_function_name_helper_r39k05"); 
    let mut args = vec![constructor.clone(), name.clone()];
    if let Some(prefix) = prefix {
        args.push(f.new_string_literal(prefix, 0));
    }
    f.new_call_expression(
        &f.new_identifier("__setFunctionName"),
        None,
        None,
        f.new_node_list(args),
        NodeFlags::empty(),
    )
}

pub fn new_run_initializers_helper_r39k05(
    f: &NodeFactory,
    this_arg: &Arc<Node>,
    initializers: &Arc<Node>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("new_run_initializers_helper_r39k05"); 
    f.new_call_expression(
        &f.new_identifier("__runInitializers"),
        None,
        None,
        f.new_node_list(vec![this_arg.clone(), initializers.clone()]),
        NodeFlags::empty(),
    )
}

pub fn new_get_accessor_declaration_full_r39k05(
    f: &NodeFactory,
    modifiers: Option<Arc<ModifierList>>,
    name: &Arc<Node>,
    type_parameters: Option<Arc<NodeList>>,
    parameters: Arc<NodeList>,
    type_node: Option<Arc<Node>>,
    full_signature: Option<Arc<Node>>,
    body: Arc<Node>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("new_get_accessor_declaration_full_r39k05"); 
    Arc::new(Node::new(
        SyntaxKind::GetAccessor,
        ndg::NodeData::GetAccessorDeclaration(ndg::GetAccessorDeclarationData {
            modifiers,
            name: name.clone(),
            type_parameters,
            parameters,
            type_node,
            full_signature,
            body: Some(body),
        }),
    ))
}
