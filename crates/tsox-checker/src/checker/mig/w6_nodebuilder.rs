#![allow(unused_imports)]
#![allow(dead_code)]

use crate::checker::checker::*;
use crate::checker::nodecopy_builder::NodeBuilderImpl;
use crate::checker::symboltracker::{NodeBuilderFlags, NodeBuilderInternalFlags, SymbolTracker};
use crate::checker::types::*;
use std::sync::Arc;
use tsox_frontend::ast::{is_js_type_alias_declaration, is_type_alias_declaration, Node, Symbol, SyntaxKind};
use super::wc2_2::get_name_from_index_info;
use crate::checker::checker_es_symbol::walk_up_parenthesized_types;
use super::m2g::NodeFactoryExt21;
use super::m2b::r22k6_defs::R22K6NodeFactoryExt;

#[path = "r23k8_defs.rs"]
pub mod r23k8_defs;
use self::r23k8_defs::R23K8NodeFactoryExt;

pub type NodeBuilder<'a> = NodeBuilderImpl<'a>;

impl<'a> NodeBuilder<'a> {
    pub fn index_info_to_index_signature_declaration(
        &mut self,
        info: &IndexInfo,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        tracker: Option<Box<dyn SymbolTracker>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("index_info_to_index_signature_declaration"); 
        let _ = (enclosing_declaration, flags, internal_flags, tracker);
        self.index_info_to_index_signature_declaration_helper(info, None)
    }
}

impl<'a> NodeBuilderImpl<'a> {
    pub fn index_info_to_object_computed_names_or_signature_declaration(
        &mut self,
        index_info: &IndexInfo,
        type_node: Option<&Arc<Node>>,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("index_info_to_object_computed_names_or_signature_declaration"); 
        if !index_info.components.is_empty() {
            let ch_ptr: *mut Checker = self.ch as *const Checker as *mut Checker;
            let enclosing = self.ctx.borrow().enclosing_declaration.clone();
            let all_component_computed_names_serializable =
                enclosing.is_some() && index_info.components.iter().all(|c| self.is_trivially_serializable_computed_name(Some(c)));
            if all_component_computed_names_serializable {
                let new_components: Vec<Arc<Node>> = index_info
                    .components
                    .iter()
                    .filter(|c| !unsafe { (*ch_ptr).has_late_bindable_name(c) })
                    .cloned()
                    .collect();
                let mut bailed = false;
                let results: Vec<Arc<Node>> = new_components
                    .iter()
                    .filter_map(|e| {
                        let name = self.reuse_node(e.name());
                        let name = match name {
                            Some(n) => n,
                            None => {
                                bailed = true;
                                return None;
                            }
                        };
                        self.track_computed_name(e.name().unwrap().expression().unwrap(), enclosing.as_ref());
                        let mods = if index_info.is_readonly {
                            self.f.new_modifier_list(&[self.f.new_modifier(SyntaxKind::ReadonlyKeyword)])
                        } else {
                            None
                        };
                        let postfix_token = tsox_frontend::ast::mig::m3b::postfix_token(e).map(|t| self.f.clone_node(&t));
                        let current_type_node: Option<Arc<Node>> = match type_node {
                            Some(tn) => Some(self.deep_clone_node(tn)),
                            None => {
                                let value_type = unsafe { (*ch_ptr).get_symbol_of_node(e) }
                                    .map(|s| unsafe { (*ch_ptr).get_type_of_symbol(&s) });
                                value_type.as_ref().and_then(|t| self.type_to_type_node(t))
                            }
                        };
                        let mut sig = self.f.new_property_signature_declaration(mods, name, postfix_token, current_type_node, None);
                        Arc::get_mut(&mut sig).unwrap().loc = e.loc;
                        Some(sig)
                    })
                    .collect();
                if !bailed {
                    return results;
                }
            }
        }
        vec![self.index_info_to_index_signature_declaration_helper(index_info, type_node).unwrap()]
    }

    pub fn index_info_to_index_signature_declaration_helper(
        &mut self,
        index_info: &IndexInfo,
        type_node: Option<&Arc<Node>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("index_info_to_index_signature_declaration_helper"); 
        let name = get_name_from_index_info(index_info);        let indexer_type_node = self.type_to_type_node(index_info.key_type.as_ref()?);
        let indexing_parameter_name = self.new_identifier(&name, None);
        let indexing_parameter = self.f.new_parameter_declaration(
            None,
            None,
            indexing_parameter_name,
            None,
            indexer_type_node,
            None,
        );
        let mut type_node = type_node.map(Arc::clone);
        if type_node.is_none() {
            if index_info.value_type.is_none() {
                type_node = Some(self.f.new_keyword_type_node(SyntaxKind::AnyKeyword));
            } else {
                type_node = self.type_to_type_node(index_info.value_type.as_ref().unwrap());
            }
        }
        if index_info.value_type.is_none() && !self.ctx.borrow().flags.intersects(NodeBuilderFlags::AllowEmptyIndexInfoType) {
            self.ctx.borrow_mut().encountered_error = true;
        }
        self.ctx.borrow_mut().approximate_length += name.len() + 4;
        let mut modifiers = None;
        if index_info.is_readonly {
            self.ctx.borrow_mut().approximate_length += 9;
            modifiers = self.f.new_modifier_list(&[self.f.new_modifier(SyntaxKind::ReadonlyKeyword)]);
        }
        Some(self.f.new_index_signature_declaration(
            modifiers,
            self.f.new_node_list(vec![indexing_parameter]),
            type_node,
        ))
    }
}

pub fn has_type_annotation(declaration: Option<&Arc<Node>>) -> bool { ::tsox_core::fntrace::enter("has_type_annotation"); 
    let Some(declaration) = declaration else { return false };
    if declaration.type_().is_none() {
        return false;
    }
    if is_type_alias_declaration(declaration) || is_js_type_alias_declaration(declaration) {
        return false;
    }
    true
}

pub fn get_type_alias_for_type_literal(c: &Checker, t: &Arc<Type>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_type_alias_for_type_literal"); 
    if let Some(symbol) = t.symbol.as_ref()
        && symbol.flags.intersects(SymbolFlags::TypeLiteral)
        && !symbol.declarations.is_empty()
    {
        let node = walk_up_parenthesized_types(&symbol.declarations[0].parent().unwrap());
        if is_type_alias_declaration(&node) {
            return c.get_symbol_of_declaration(&node);
        }
    }
    None
}
