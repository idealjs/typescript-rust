#![allow(unused_imports)]

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::checker::checker::Checker;
use crate::checker::nodecopy_builder::NodeBuilderImpl;
use crate::checker::symboltracker::NodeBuilderContext;
use crate::checker::types::Type;
use tsox_frontend::ast::{
    Node, NodeData, NodeFlags, NodeList, Symbol, SymbolFlags, SyntaxKind,
};

use crate::checker::mig::m2b::r22k6_defs::R22K6NodeFactoryExt;
use crate::checker::mig::m2g::r21k9_defs::NodeFactoryExt21;

thread_local! {
    static TYPE_STACK: RefCell<HashMap<usize, Vec<Option<Arc<Type>>>>> = RefCell::new(HashMap::new());
    static REMAPPED_SYMBOL_REFERENCES: RefCell<HashMap<usize, HashMap<u64, Arc<Symbol>>>> =
        RefCell::new(HashMap::new());
    static TYPE_PARAMETER_SYMBOL_LIST: RefCell<HashMap<usize, HashSet<u64>>> = RefCell::new(HashMap::new());
}

pub fn type_stack_with<R>(
    ctx: &NodeBuilderContext,
    f: impl FnOnce(&mut Vec<Option<Arc<Type>>>) -> R,
) -> R {
    TYPE_STACK.with(|m| {
        f(m.borrow_mut()
            .entry(crate::checker::mig::m2g::r22k9_defs::ctx_side_key(ctx))
            .or_default())
    })
}

pub fn remapped_symbol_references_with<R>(
    ctx: &NodeBuilderContext,
    f: impl FnOnce(&mut HashMap<u64, Arc<Symbol>>) -> R,
) -> R {
    REMAPPED_SYMBOL_REFERENCES.with(|m| {
        f(m.borrow_mut()
            .entry(crate::checker::mig::m2g::r22k9_defs::ctx_side_key(ctx))
            .or_default())
    })
}

pub fn type_parameter_symbol_list_with<R>(
    ctx: &NodeBuilderContext,
    f: impl FnOnce(&mut HashSet<u64>) -> R,
) -> R {
    TYPE_PARAMETER_SYMBOL_LIST.with(|m| {
        f(m.borrow_mut()
            .entry(crate::checker::mig::m2g::r22k9_defs::ctx_side_key(ctx))
            .or_default())
    })
}

pub fn get_type_argument_list_of(node: &Arc<Node>) -> Option<Arc<NodeList>> {
    match &node.data {
        NodeData::TypeReferenceNode(tr) => tr.type_arguments.clone(),
        NodeData::ImportTypeNode(it) => it.type_arguments.clone(),
        NodeData::ExpressionWithTypeArguments(et) => et.type_arguments.clone(),
        _ => None,
    }
}

pub fn has_non_global_augmentation_external_module_symbol(
    _ch: &Checker,
    declaration: &Arc<Node>,
) -> bool {
    tsox_frontend::ast::is_module_with_string_literal_name(declaration)
        || Checker::is_external_or_common_js_module(declaration)
}

pub fn escape_internal_symbol_name(name: &str) -> String {
    match name.strip_prefix('\u{FE}') {
        Some(rest) => format!("__{}", rest),
        None => name.to_string(),
    }
}

pub trait R24K13NodeBuilderExt {
    fn create_access_expression(&mut self, node: &Arc<Node>) -> Arc<Node>;
    fn create_expression_with_type_arguments(
        &mut self,
        expr: Arc<Node>,
        type_arguments: Option<&Arc<NodeList>>,
    ) -> Arc<Node>;
    fn lookup_expression_chain_type_argument_nodes(
        &mut self,
        chain: &[Arc<Symbol>],
        index: usize,
    ) -> Option<Arc<NodeList>>;
    fn lookup_type_parameter_nodes(
        &mut self,
        chain: &[Arc<Symbol>],
        index: usize,
    ) -> Option<Arc<NodeList>>;
}

impl R24K13NodeBuilderExt for NodeBuilderImpl<'_> {
    fn create_access_expression(&mut self, node: &Arc<Node>) -> Arc<Node> {
        if let NodeData::QualifiedName(entity) = &node.data {
            let left = self.create_access_expression(&entity.left);
            let right = self.deep_clone_node(&entity.right);
            return self
                .f
                .new_property_access_expression(&left, None, &right, NodeFlags::empty());
        }
        if matches!(
            node.kind,
            SyntaxKind::Identifier
                | SyntaxKind::PropertyAccessExpression
                | SyntaxKind::ExpressionWithTypeArguments
        ) {
            return Arc::clone(node);
        }
        panic!("unexpected access node kind: {:?}", node.kind)
    }

    fn create_expression_with_type_arguments(
        &mut self,
        expr: Arc<Node>,
        type_arguments: Option<&Arc<NodeList>>,
    ) -> Arc<Node> {
        match type_arguments {
            Some(ta) if !ta.nodes.is_empty() => {
                Arc::new(Node::new(
                    SyntaxKind::ExpressionWithTypeArguments,
                    NodeData::ExpressionWithTypeArguments(
                        tsox_frontend::ast::ExpressionWithTypeArgumentsData {
                            expression: expr,
                            type_arguments: Some(Arc::clone(ta)),
                        },
                    ),
                ))
            }
            _ => expr,
        }
    }

    fn lookup_expression_chain_type_argument_nodes(
        &mut self,
        chain: &[Arc<Symbol>],
        index: usize,
    ) -> Option<Arc<NodeList>> {
        let symbol_id = tsox_frontend::ast::get_symbol_id(&chain[index]);
        let listed = {
            let ctx = self.ctx.borrow();
            r24k13_type_parameter_symbol_list_has(&ctx, symbol_id)
        };
        if listed {
            return None;
        }
        {
            let ctx = self.ctx.borrow();
            r24k13_type_parameter_symbol_list_add(&ctx, symbol_id);
        }
        let write_type_params = {
            let ctx = self.ctx.borrow();
            ctx.flags
                .contains(crate::checker::symboltracker::NodeBuilderFlags::WriteTypeParametersInQualifiedName)
        };
        if write_type_params && index < chain.len() - 1 {
            let type_parameter_nodes = self.type_parameters_to_type_parameter_declarations(&chain[index]);
            if let Some(nodes) = type_parameter_nodes.filter(|nodes| !nodes.is_empty()) {
                return Some(Arc::new(
                    crate::checker::mig::m2f::r17k8_factory_ext::NodeFactoryExt::new_node_list(
                        &self.f,
                        nodes,
                    ),
                ));
            }
        }
        None
    }

    fn lookup_type_parameter_nodes(
        &mut self,
        chain: &[Arc<Symbol>],
        index: usize,
    ) -> Option<Arc<NodeList>> {
        let symbol_id = tsox_frontend::ast::get_symbol_id(&chain[index]);
        let listed = {
            let ctx = self.ctx.borrow();
            r24k13_type_parameter_symbol_list_has(&ctx, symbol_id)
        };
        if listed {
            return None;
        }
        {
            let ctx = self.ctx.borrow();
            r24k13_type_parameter_symbol_list_add(&ctx, symbol_id);
        }
        let write_type_params = {
            let ctx = self.ctx.borrow();
            ctx.flags
                .contains(crate::checker::symboltracker::NodeBuilderFlags::WriteTypeParametersInQualifiedName)
        };
        if write_type_params && index < chain.len() - 1 {
            let type_parameter_nodes = self.type_parameters_to_type_parameter_declarations(&chain[index]);
            if let Some(nodes) = type_parameter_nodes.filter(|nodes| !nodes.is_empty()) {
                return Some(Arc::new(
                    crate::checker::mig::m2f::r17k8_factory_ext::NodeFactoryExt::new_node_list(
                        &self.f,
                        nodes,
                    ),
                ));
            }
        }
        None
    }
}

fn r24k13_type_parameter_symbol_list_has(ctx: &NodeBuilderContext, symbol_id: u64) -> bool {
    type_parameter_symbol_list_with(ctx, |set| set.contains(&symbol_id))
}

fn r24k13_type_parameter_symbol_list_add(ctx: &NodeBuilderContext, symbol_id: u64) {
    type_parameter_symbol_list_with(ctx, |set| {
        set.insert(symbol_id);
    });
}

pub fn r24k13_find_ancestor_same(
    a: &Option<Arc<Node>>,
    b: &Option<Arc<Node>>,
) -> bool {
    match (a, b) {
        (Some(x), Some(y)) => Arc::ptr_eq(x, y),
        (None, None) => true,
        _ => false,
    }
}
