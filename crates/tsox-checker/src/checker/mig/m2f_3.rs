#![allow(unused_imports)]
use crate::checker::mig::m2a::r19k11_defs::*;
use crate::checker::mig::m2b::r22k6_defs::R22K6NodeFactoryExt;
use crate::checker::mig::m2g::r21k9_defs::NodeFactoryExt21;
use crate::checker::mig::m2g::types_are_same_reference;

#[path = "r24k13_defs.rs"]
pub mod r24k13_defs;
use r24k13_defs::R24K13NodeBuilderExt;

#[path = "r25k8_defs.rs"]
pub mod r25k8_defs;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use tsox_frontend::ast::{is_identifier, Node, NodeData, NodeFlags, Symbol, SymbolFlags, SyntaxKind};
use tsox_frontend::format::mig::m4o::EmitFlags;

use crate::checker::nodecopy_builder::{EmitContextStub, NodeBuilderImpl, NodeFactoryStub};
use super::r27k_defs::{clone_binding_name_visitor_hook, new_node_visitor, NodeVisitorHooks};
use crate::checker::symboltracker::{
    NodeBuilderContext, NodeBuilderFlags, NodeBuilderInternalFlags, SharedNodeBuilderContext,
    DEFAULT_MAXIMUM_TRUNCATION_LENGTH, NO_TRUNCATION_MAXIMUM_TRUNCATION_LENGTH,
};
use crate::checker::types::{
    ObjectFlags, SignatureFlags, Type, TypeFlags, TypeMapper, StructuredType,
};

pub struct SavedNodeBuilderFlags {
    ctx: SharedNodeBuilderContext,
    flags: NodeBuilderFlags,
    internal_flags: NodeBuilderInternalFlags,
    depth: usize,
}

impl SavedNodeBuilderFlags {
    pub fn restore(&self) {
        let mut ctx = self.ctx.borrow_mut();
        ctx.flags = self.flags;
        ctx.internal_flags = self.internal_flags;
        ctx.depth = self.depth;
    }
}

pub fn new_node_builder_impl<'a>(
    ch: &'a crate::checker::checker::Checker,
    e: &EmitContextStub,
    id_to_symbol: Option<HashMap<u64, Arc<Symbol>>>,
) -> NodeBuilderImpl<'a> {
    let id_to_symbol = id_to_symbol.unwrap_or_default();
    let mut b = NodeBuilderImpl::new(ch, id_to_symbol);
    b.pc = r25k8_defs::new_pseudo_checker(ch.strict_null_checks, ch.exact_optional_property_types);
    b.clone_binding_name_visitor = Some(new_node_visitor(
        clone_binding_name_visitor_hook(&b),
        &b.f,
        NodeVisitorHooks::default(),
    ));
    b
}

impl<'a> NodeBuilderImpl<'a> {
    pub fn save_restore_flags(&mut self) -> SavedNodeBuilderFlags {
        let (flags, internal_flags, depth) = {
            let ctx = self.ctx.borrow();
            (ctx.flags, ctx.internal_flags, ctx.depth)
        };
        SavedNodeBuilderFlags {
            ctx: Rc::clone(&self.ctx),
            flags,
            internal_flags,
            depth,
        }
    }

    pub fn check_truncation_length(&mut self) -> bool {
        {
            let ctx = self.ctx.borrow();
            if ctx.truncating {
                return true;
            }
        }
        let max_length = {
            let ctx = self.ctx.borrow();
            if ctx.flags.contains(NodeBuilderFlags::NoTruncation) {
                NO_TRUNCATION_MAXIMUM_TRUNCATION_LENGTH
            } else if ctx.max_truncation_length > 0 {
                ctx.max_truncation_length
            } else {
                DEFAULT_MAXIMUM_TRUNCATION_LENGTH
            }
        };
        let over = self.ctx.borrow().approximate_length > max_length;
        self.ctx.borrow_mut().truncating = over;
        over
    }

    pub fn check_truncation_length_if_expanding(&mut self) -> bool {
        let expanding = self.ctx.borrow().max_expansion_depth >= 0;
        if expanding && self.check_truncation_length() {
            self.ctx.borrow_mut().expansion_truncated = true;
            return true;
        }
        false
    }

    pub fn is_expandable_type(&mut self, t: &Arc<Type>, is_alias: bool) -> bool {
        if is_alias {
            if let Some(s) = t.alias.as_ref().and_then(|a| a.symbol.as_ref()) {
                return !self.ch.is_lib_symbol_for_hover_verbosity(s);
            }
            return true;
        }
        if self.ch.is_lib_type_for_hover_verbosity(t) {
            return false;
        }
        let object_flags = t.object_flags;
        if t.flags.intersects(TypeFlags::ENUM_LIKE)
            || object_flags.intersects(ObjectFlags::Reference)
            || object_flags.intersects(ObjectFlags::CLASS_OR_INTERFACE)
        {
            return true;
        }
        if object_flags.intersects(ObjectFlags::Anonymous)
            && t.symbol.is_some()
            && t
                .symbol
                .as_ref()
                .is_some_and(|s| {
                    s.flags.intersects(
                        SymbolFlags::Class
                            | SymbolFlags::ENUM
                            | SymbolFlags::ValueModule
                            | SymbolFlags::Function
                            | SymbolFlags::Method,
                    )
                })
        {
            return true;
        }
        false
    }

    pub fn is_type_on_stack(&mut self, t: &Arc<Type>) -> bool {
        let ctx = self.ctx.borrow();
        r24k13_defs::type_stack_with(&ctx, |stack| {
            if stack.is_empty() {
                return false;
            }
            for item in &stack[..stack.len() - 1] {
                if let Some(item) = item {
                    if Arc::ptr_eq(item, t) {
                        return true;
                    }
                }
            }
            false
        })
    }

    pub fn is_actively_expanding(&mut self) -> bool {
        let ctx = self.ctx.borrow();
        ctx.max_expansion_depth > 0 && ctx.depth < ctx.max_expansion_depth as usize
    }

    pub fn append_reference_to_type(
        &mut self,
        root: &Arc<Node>,
        ref_node: &Arc<Node>,
    ) -> Arc<Node> {
        if tsox_frontend::ast::is_import_type_node(root) {
            if let NodeData::ImportTypeNode(imprt) = &root.data {
                let ids = get_access_stack(ref_node);
                let mut qualifier = imprt.qualifier.clone();
                for id in ids.iter().rev() {
                    if let Some(q) = qualifier {
                        qualifier = Some(self.f.new_qualified_name(&q, id));
                    } else {
                        qualifier = Some(Arc::clone(id));
                    }
                }
                return self.f.update_import_type_node(
                    root,
                    Some(&imprt.argument),
                    qualifier.as_ref(),
                    r24k13_defs::get_type_argument_list_of(ref_node),
                );
            }
        } else if tsox_frontend::ast::is_type_reference_node(root) {
            if let NodeData::TypeReferenceNode(type_ref) = &root.data {
                let use_instantiation = {
                    let ctx = self.ctx.borrow();
                    ctx.flags.contains(NodeBuilderFlags::UseInstantiationExpressions)
                };
                if use_instantiation
                    && type_ref
                        .type_arguments
                        .as_ref()
                        .is_some_and(|ta| !ta.nodes.is_empty())
                {
                    let access = self.create_access_expression(&type_ref.type_name);
                    let mut expr = self.create_expression_with_type_arguments(
                        access,
                        type_ref.type_arguments.as_ref(),
                    );
                    for id in get_access_stack(ref_node) {
                        expr = self.f.new_property_access_expression(
                            &expr,
                            None,
                            &id,
                            NodeFlags::empty(),
                        );
                    }
                    return expr;
                }
                let mut type_name = type_ref.type_name.clone();
                for id in get_access_stack(ref_node) {
                    type_name = self.f.new_qualified_name(&type_name, &id);
                }
                return self.f.update_type_reference_node(
                    root,
                    &type_name,
                    r24k13_defs::get_type_argument_list_of(ref_node),
                );
            }
        }
        let mut expr = self.create_access_expression(root);
        for id in get_access_stack(ref_node) {
            expr = self.f.new_property_access_expression(&expr, None, &id, NodeFlags::empty());
        }
        expr
    }

    pub fn create_elided_information_placeholder(&mut self) -> Arc<Node> {
        self.ctx.borrow_mut().approximate_length += 3;
        let no_truncation = self.ctx.borrow().flags.contains(NodeBuilderFlags::NoTruncation);
        if !no_truncation {
            return self.f.new_type_reference_node(&self.f.new_identifier("..."), None);
        }
        self.f.add_synthetic_leading_comment(
            &self.f.new_keyword_type_node(SyntaxKind::AnyKeyword),
            SyntaxKind::MultiLineCommentTrivia,
            "elided",
        )
    }

    pub fn map_to_type_nodes(
        &mut self,
        list: &[Arc<Type>],
        is_bare_list: bool,
    ) -> Option<Arc<tsox_frontend::ast::NodeList>> {
        if list.is_empty() {
            return None;
        }

        if self.check_truncation_length() {
            let no_truncation = self.ctx.borrow().flags.contains(NodeBuilderFlags::NoTruncation);
            if !is_bare_list {
                let node = if no_truncation {
                    self.f.add_synthetic_leading_comment(
                        &self.f.new_keyword_type_node(SyntaxKind::AnyKeyword),
                        SyntaxKind::MultiLineCommentTrivia,
                        "elided",
                    )
                } else {
                    self.f
                        .new_type_reference_node(&self.f.new_identifier("..."), None)
                };
                return Some(Arc::new(crate::checker::mig::m2f::r17k8_factory_ext::NodeFactoryExt::new_node_list(&self.f, vec![node])));
            } else if list.len() > 2 {
                let mut nodes: Vec<Option<Arc<Node>>> = vec![
                    Some(self.type_to_type_node(&list[0])?),
                    None,
                    Some(self.type_to_type_node(&list[list.len() - 1])?),
                ];
                if no_truncation {
                    nodes[1] = Some(self.f.add_synthetic_leading_comment(
                        &self.f.new_keyword_type_node(SyntaxKind::AnyKeyword),
                        SyntaxKind::MultiLineCommentTrivia,
                        &format!("... {} more elided ...", list.len() - 2),
                    ));
                } else {
                    let text = format!("... {} more ...", list.len() - 2);
                    nodes[1] =
                        Some(self.f.new_type_reference_node(&self.f.new_identifier(&text), None));
                }
                let flattened: Vec<Arc<Node>> = nodes.into_iter().flatten().collect();
                return Some(Arc::new(crate::checker::mig::m2f::r17k8_factory_ext::NodeFactoryExt::new_node_list(&self.f, flattened)));
            }
        }

        let may_have_name_collisions =
            !self.ctx.borrow().flags.contains(NodeBuilderFlags::UseFullyQualifiedType);
        let mut seen_names: Option<HashMap<String, Vec<(Arc<Type>, usize)>>> =
            if may_have_name_collisions {
                Some(HashMap::new())
            } else {
                None
            };

        let mut result: Vec<Arc<Node>> = Vec::with_capacity(list.len());

        for (i, t) in list.iter().enumerate() {
            let display_index = i + 1;
            if self.check_truncation_length() && display_index + 2 < list.len() - 1 {
                let no_truncation =
                    self.ctx.borrow().flags.contains(NodeBuilderFlags::NoTruncation);
                if no_truncation {
                    result.push(self.f.add_synthetic_leading_comment(
                        &self.f.new_keyword_type_node(SyntaxKind::AnyKeyword),
                        SyntaxKind::MultiLineCommentTrivia,
                        &format!("... {} more elided ...", list.len() - display_index),
                    ));
                } else {
                    let text = format!("... {} more ...", list.len() - display_index);
                    result.push(
                        self.f
                            .new_type_reference_node(&self.f.new_identifier(&text), None),
                    );
                }
                if let Some(type_node) = self.type_to_type_node(&list[list.len() - 1]) {
                    result.push(type_node);
                }
                break;
            }
            self.ctx.borrow_mut().approximate_length += 2;
            if let Some(type_node) = self.type_to_type_node(t) {
                if let Some(seen) = seen_names.as_mut() {
                    if is_identifier_type_reference(&type_node) {
                        if let NodeData::TypeReferenceNode(tr) = &type_node.data {
                            let text = tr.type_name.text().to_string();
                            seen.entry(text)
                                .or_default()
                                .push((Arc::clone(t), result.len()));
                        }
                    }
                }
                result.push(type_node);
            }
        }

        if let Some(seen) = seen_names {
            let restore_flags = self.save_restore_flags();
            self.ctx.borrow_mut().flags |= NodeBuilderFlags::UseFullyQualifiedType;
            for (_, types) in seen {
                if !array_is_homogeneous(&types, |a, b| types_are_same_reference(&a.0, &b.0)) {
                    for (t, _) in &types {
                        if let Some(node) = self.type_to_type_node(t) {
                            result.push(node);
                        }
                    }
                }
            }
            restore_flags.restore();
        }

        Some(Arc::new(crate::checker::mig::m2f::r17k8_factory_ext::NodeFactoryExt::new_node_list(&self.f, result)))
    }

    pub fn existing_type_node_is_not_reference_or_is_reference_with_compatible_type_argument_count(
        &mut self,
        existing: &Arc<Node>,
        t: &Arc<Type>,
    ) -> bool {
        let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
        if !t.object_flags.intersects(ObjectFlags::Reference) {
            return true;
        }
        if !tsox_frontend::ast::is_type_reference_node(existing) {
            return true;
        }

        let symbol = self.try_get_resolved_symbol_from_type_node(existing);
        let Some(symbol) = symbol else {
            return true;
        };

        let existing_target = ch.get_declared_type_of_symbol(&symbol);
        let Some(target) = t.as_type_reference().and_then(|d| d.target.clone()) else {
            return true;
        };
        if !Arc::ptr_eq(&existing_target, &target) {
            return true;
        }
        let existing_count = match &existing.data {
            NodeData::TypeReferenceNode(tr) => {
                tr.type_arguments.as_ref().map_or(0, |l| l.nodes.len())
            }
            _ => 0,
        };
        let Some(interface_target) = target.as_interface_type() else {
            return true;
        };
        existing_count
            >= self.ch.get_min_type_argument_count(
                &interface_target.all_type_parameters[..interface_target.outer_type_parameter_count],
            )
    }

    pub fn get_resolved_type_without_abstract_construct_signatures(
        &mut self,
        t: &StructuredType,
    ) -> Arc<Type> {
        let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
        if t.construct_signatures().is_empty() {
            return t.as_type();
        }
        if let Some(cached) = t.object_type_without_abstract_construct_signatures.get() {
            return Arc::clone(cached);
        }
        let all_construct = t.construct_signatures();
        let construct_signatures: Vec<_> = all_construct
            .iter()
            .filter(|sig| !sig.flags.intersects(SignatureFlags::Abstract))
            .cloned()
            .collect();
        if construct_signatures.len() == all_construct.len() {
            let as_type = t.as_type();
            let _ = t.object_type_without_abstract_construct_signatures.set(Arc::clone(&as_type));
            return as_type;
        }
        let type_copy = ch.new_anonymous_type(
            t.typ.symbol.as_ref().expect("structured type symbol"),
            t.members.clone(),
            t.call_signatures().to_vec(),
            construct_signatures,
            t.index_infos.to_vec(),
        );
        let _ = t.object_type_without_abstract_construct_signatures.set(Arc::clone(&type_copy));
        if let Some(st) = type_copy.as_structured_type() {
            let _ = st
                .object_type_without_abstract_construct_signatures
                .set(Arc::clone(&type_copy));
        }
        type_copy
    }

    pub fn create_entity_name_from_symbol_chain(
        &mut self,
        chain: &[Arc<Symbol>],
        index: usize,
    ) -> Arc<Node> {
        let symbol = &chain[index];

        if index == 0 {
            self.ctx.borrow_mut().flags |= NodeBuilderFlags::InInitialEntityName;
        }
        let symbol_name = self.get_name_of_symbol_as_written(symbol);
        if index == 0 {
            self.ctx.borrow_mut().flags ^= NodeBuilderFlags::InInitialEntityName;
        }

        let identifier = self.new_identifier(&symbol_name, Some(symbol));
        self.e
            .add_emit_flags(&identifier, EmitFlags::NO_ASCII_ESCAPING.0);
        if index > 0 {
            let left = self.create_entity_name_from_symbol_chain(chain, index - 1);
            return self.f.new_qualified_name(&left, &identifier);
        }
        identifier
    }
}

pub fn get_access_stack(ref_node: &Arc<Node>) -> Vec<Arc<Node>> {
    let mut state = match &ref_node.data {
        NodeData::TypeReferenceNode(tr) => tr.type_name.clone(),
        _ => Arc::clone(ref_node),
    };
    let mut ids: Vec<Arc<Node>> = Vec::new();
    while !is_identifier(&state) {
        if let NodeData::QualifiedName(entity) = &state.data {
            ids.insert(0, Arc::clone(&entity.right));
            state = Arc::clone(&entity.left);
        } else {
            break;
        }
    }
    ids.insert(0, state);
    ids
}

pub fn is_class_instance_side(
    c: &mut crate::checker::checker::Checker,
    t: &Arc<Type>,
) -> bool {
    t.symbol.as_ref().is_some_and(|s| {
        s.flags.intersects(SymbolFlags::Class)
            && (Arc::ptr_eq(t, &c.get_declared_type_of_class_or_interface(s))
                || (t.flags.intersects(TypeFlags::Object)
                    && t.object_flags.intersects(ObjectFlags::IsClassInstanceClone)))
    })
}

pub fn is_identifier_type_reference(node: &Arc<Node>) -> bool {
    tsox_frontend::ast::is_type_reference_node(node)
        && match &node.data {
            NodeData::TypeReferenceNode(tr) => is_identifier(&tr.type_name),
            _ => false,
        }
}

pub fn array_is_homogeneous<T>(array: &[T], comparer: impl Fn(&T, &T) -> bool) -> bool {
    if array.len() < 2 {
        return true;
    }
    let first = &array[0];
    for item in &array[1..] {
        if !comparer(first, item) {
            return false;
        }
    }
    true
}
