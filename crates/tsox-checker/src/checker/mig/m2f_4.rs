#![allow(unused_imports)]
#[path = "r28k4_defs.rs"]
pub mod r28k4_defs;
use r28k4_defs::R28K4NodeBuilderExt;
use crate::checker::mig::m2a::r19k11_defs::*;
use tsox_frontend::ast::LanguageVariant;
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::scanner::mig::m3i::{declaration_name_to_string, is_identifier_text};
use crate::checker::types_impl_chunk::LiteralValue;
use crate::checker::mig::m2c::r19k8_defs::pseudo_big_int_to_string;
use crate::checker::mig::m2g::r21k9_defs::{value_to_string, PseudoLiteralValue};
use crate::checker::mig::m2b::r22k6_defs::R22K6NodeFactoryExt;
use crate::checker::mig::m2g::r21k9_defs::NodeFactoryExt21;
use crate::checker::mig::m2f_3::r24k13_defs;
use r24k13_defs::R24K13NodeBuilderExt;
use tsox_frontend::ast::CheckFlags;
use crate::checker::utilities_is_optional_symbol::{is_late_bound_name, is_numeric_literal_name};

use std::sync::Arc;

use tsox_frontend::ast::{Node, NodeData, NodeFlags, Symbol, SymbolFlags, SyntaxKind};

use crate::checker::nodecopy_builder::NodeBuilderImpl;
use crate::checker::symboltracker::NodeBuilderFlags;
use crate::checker::types::{Type, TypeFlags};

pub fn get_topmost_indexed_access_type(node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("get_topmost_indexed_access_type"); 
    if let NodeData::IndexedAccessTypeNode(iat) = &node.data {
        if tsox_frontend::ast::is_indexed_access_type_node(&iat.object_type) {
            return get_topmost_indexed_access_type(&iat.object_type);
        }
    }
    Arc::clone(node)
}

pub fn can_use_property_access(name: &str) -> bool { ::tsox_core::fntrace::enter("can_use_property_access"); 
    if name.is_empty() {
        return false;
    }
    if let Some(rest) = name.strip_prefix('#') {
        return !rest.is_empty()
            && is_identifier_text(rest, LanguageVariant::Standard);
    }
    is_identifier_text(name, LanguageVariant::Standard)
}

pub fn is_default_binding_context(location: &Node) -> bool { ::tsox_core::fntrace::enter("is_default_binding_context"); 
    location.kind == SyntaxKind::SourceFile || tsox_frontend::ast::is_ambient_module(location)
}

impl<'a> NodeBuilderImpl<'a> {
    pub fn create_access_from_symbol_chain(
        &mut self,
        chain: &[Arc<Symbol>],
        index: usize,
        stopper: usize,
        override_type_arguments: Option<&Arc<tsox_frontend::ast::NodeList>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("create_access_from_symbol_chain"); 
        let type_parameter_nodes = if index != chain.len() - 1 {
            self.lookup_type_parameter_nodes(chain, index)
        } else {
            override_type_arguments.cloned()
        };
        let symbol = &chain[index];
        let parent = if index > 0 {
            Some(&chain[index - 1])
        } else {
            None
        };

        let mut symbol_name = String::new();
        if index == 0 {
            self.ctx.borrow_mut().flags |= NodeBuilderFlags::InInitialEntityName;
            symbol_name = self.get_name_of_symbol_as_written(symbol);
            self.ctx.borrow_mut().approximate_length += symbol_name.len() + 1;
            self.ctx.borrow_mut().flags ^= NodeBuilderFlags::InInitialEntityName;
        } else if let Some(parent) = parent {
            let exports = self.ch.get_exports_of_symbol(parent);
            if !exports.is_empty() {
                let res = exports.iter().find(|(_, ex)| ex.name == symbol.name).map(|(_, ex)| Arc::clone(ex));
                let same = res.is_some_and(|r| {
                    symbol.name != "export=" && !is_late_bound_name(&symbol.name) && self
                        .ch
                        .get_symbol_if_same_reference(&r, symbol)
                        .is_some()
                });
                if same {
                    symbol_name = symbol.name.clone();
                } else {
                    let mut results: Vec<(Arc<Symbol>, String)> = Vec::new();
                    for (name, ex) in exports.iter() {
                        if self.ch.get_symbol_if_same_reference(ex, symbol).is_some()
                            && !is_late_bound_name(name)
                            && name != "export="
                        {
                            results.push((Arc::clone(ex), name.clone()));
                        }
                    }
                    if !results.is_empty() {
                        let mut result_symbols: Vec<Arc<Symbol>> =
                            results.iter().map(|(s, _)| Arc::clone(s)).collect();
                        self.ch.sort_symbols(&mut result_symbols);
                        if let Some((_, name)) = results
                            .iter()
                            .find(|(s, _)| Arc::ptr_eq(s, &result_symbols[0]))
                        {
                            symbol_name = name.clone();
                        }
                    }
                }
            }
        }

        if symbol_name.is_empty() {
            let mut name: Option<Arc<Node>> = None;
            for d in &symbol.declarations {
                name = tsox_frontend::ast::get_name_of_declaration(d);
                if name.is_some() {
                    break;
                }
            }
            let is_entity_name_expr = name.as_ref().is_some_and(|n| {
                tsox_frontend::ast::is_computed_property_name(n)
                    && n
                        .expression()
                        .is_some_and(|e| tsox_frontend::ast::is_entity_name(e))
            });
            if let Some(name) = name {
                if is_entity_name_expr {
                    if index == 0 {
                        // Go would recurse with index -1; guarded here since index 0 has no parent
                        symbol_name = self.get_name_of_symbol_as_written(symbol);
                    } else {
                        let lhs = self.create_access_from_symbol_chain(
                            chain,
                            index - 1,
                            stopper,
                            override_type_arguments,
                        );
                        if tsox_frontend::ast::is_entity_name(&lhs) {
                            let object_type = self.f.new_parenthesized_type_node(
                                &self.f.new_type_query_node(&lhs),
                            );
                            let index_type = self.f.new_type_query_node(
                                name.expression().expect("entity name expression"),
                            );
                            return self
                                .f
                                .new_indexed_access_type_node(&object_type, &index_type);
                        }
                        return lhs;
                    }
                } else {
                    symbol_name = self.get_name_of_symbol_as_written(symbol);
                }
            } else {
                symbol_name = self.get_name_of_symbol_as_written(symbol);
            }
        }
        self.ctx.borrow_mut().approximate_length += symbol_name.len() + 1;

        let use_indexed_access = {
            let forbid = self
                .ctx
                .borrow()
                .flags
                .contains(NodeBuilderFlags::ForbidIndexedAccessSymbolReferences);
            !forbid
                && parent.is_some_and(|p| {
                    let members = self.ch.get_members_of_symbol(p);
                    members
                        .iter()
                        .find(|(_, m)| m.name == symbol.name)
                        .is_some_and(|(_, m)| {
                            self.ch.get_symbol_if_same_reference(m, symbol).is_some()
                        })
                })
        };
        if use_indexed_access {
            let lhs =
                self.create_access_from_symbol_chain(chain, index - 1, stopper, override_type_arguments);
            if tsox_frontend::ast::is_indexed_access_type_node(&lhs) {
                let name_literal = self.new_string_literal(&symbol_name);
                let index_type = self.f.new_literal_type_node(name_literal);
                return self.f.new_indexed_access_type_node(&lhs, &index_type);
            }
            let name_literal = self.new_string_literal(&symbol_name);
            let index_type = self.f.new_literal_type_node(name_literal);
            let object_node = self.f.new_type_reference_node(&lhs, type_parameter_nodes);
            return self.f.new_indexed_access_type_node(&object_node, &index_type);
        }

        let identifier = self.new_identifier(&symbol_name, Some(symbol));
        self.e.add_emit_flags(&identifier, EmitFlags::NO_ASCII_ESCAPING.0);

        if index > stopper {
            let lhs = self.create_access_from_symbol_chain(
                chain,
                index - 1,
                stopper,
                override_type_arguments,
            );
            let use_instantiation = self
                .ctx
                .borrow()
                .flags
                .contains(NodeBuilderFlags::UseInstantiationExpressions);
            let has_no_type_params = type_parameter_nodes
                .as_ref()
                .is_none_or(|tp| tp.nodes.is_empty());
            if !use_instantiation
                || (tsox_frontend::ast::is_entity_name(&lhs) && has_no_type_params)
            {
                return self.f.new_qualified_name(&lhs, &identifier);
            }
            let lhs_expression = self.create_access_expression(&lhs);
            return self.create_expression_with_type_arguments(
                self.f.new_property_access_expression(
                    &lhs_expression,
                    None,
                    &identifier,
                    NodeFlags::empty(),
                ),
                type_parameter_nodes.as_ref(),
            );
        }
        identifier
    }

    pub fn create_expression_from_symbol_chain(
        &mut self,
        chain: &[Arc<Symbol>],
        index: usize,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("create_expression_from_symbol_chain"); 
        let type_parameter_nodes = self.lookup_expression_chain_type_argument_nodes(chain, index);
        let symbol = &chain[index];

        if index == 0 {
            self.ctx.borrow_mut().flags |= NodeBuilderFlags::InInitialEntityName;
        }
        let mut symbol_name = self.get_name_of_symbol_as_written(symbol);
        if index == 0 {
            self.ctx.borrow_mut().flags ^= NodeBuilderFlags::InInitialEntityName;
        }

        let ch = self.ch;
        if starts_with_single_or_double_quote(&symbol_name)
            && symbol
                .declarations
                .iter()
                .any(|d| r24k13_defs::has_non_global_augmentation_external_module_symbol(ch, d))
        {
            let specifier = self.get_specifier_for_module_symbol(symbol, 0);
            self.ctx.borrow_mut().approximate_length += 2 + specifier.len();
            return self.new_string_literal(&specifier);
        }

        if index == 0 || can_use_property_access(&symbol_name) {
            let identifier = self.new_identifier(&symbol_name, Some(symbol));
            self.e.add_emit_flags(&identifier, EmitFlags::NO_ASCII_ESCAPING.0);
            self.ctx.borrow_mut().approximate_length += 1 + symbol_name.len();
            if index > 0 {
                let inner = self.create_expression_from_symbol_chain(chain, index - 1);
                let result = self.f.new_property_access_expression(
                    &inner,
                    None,
                    &identifier,
                    NodeFlags::empty(),
                );
                self.e.add_emit_flags(&result, EmitFlags::NO_INDENTATION.0);
                return self.create_expression_with_type_arguments(
                    result,
                    type_parameter_nodes.as_ref(),
                );
            }
            return self.create_expression_with_type_arguments(
                identifier,
                type_parameter_nodes.as_ref(),
            );
        }

        if symbol_name.starts_with('[') {
            symbol_name = symbol_name[1..symbol_name.len() - 1].to_string();
        }

        let mut expression: Option<Arc<Node>> = None;
        if starts_with_single_or_double_quote(&symbol_name)
            && !symbol.flags.intersects(SymbolFlags::EnumMember)
        {
            let literal_text = tsox_core::stringutil::mig::m3m_2::unquote_string(&symbol_name);
            self.ctx.borrow_mut().approximate_length += literal_text.len() + 2;
            expression = Some(self.new_string_literal_ex(
                &literal_text,
                symbol_name.starts_with('\''),
            ));
        } else if tsox_core::jsnum::Number::from_string(&symbol_name).0.to_string() == symbol_name {
            self.ctx.borrow_mut().approximate_length += symbol_name.len();
            expression = Some(
                crate::checker::mig::m2f::r17k8_factory_ext::NodeFactoryExt::new_numeric_literal(
                    &self.f,
                    &symbol_name,
                    0,
                ),
            );
        }
        let expression = match expression {
            Some(e) => e,
            None => {
                self.ctx.borrow_mut().approximate_length += symbol_name.len();
                let e = self.new_identifier(&symbol_name, Some(symbol));
                self.e.add_emit_flags(&e, EmitFlags::NO_ASCII_ESCAPING.0);
                e
            }
        };
        self.ctx.borrow_mut().approximate_length += 2;
        let target = self.create_expression_from_symbol_chain(chain, index - 1);
        self.create_expression_with_type_arguments(
            self.f.new_element_access_expression(&target, None, &expression),
            type_parameter_nodes.as_ref(),
        )
    }

    pub fn get_name_of_symbol_from_name_type(&mut self, symbol: &Arc<Symbol>) -> String { ::tsox_core::fntrace::enter("get_name_of_symbol_from_name_type"); 
        let name_type = self
            .ch
            .value_symbol_links
            .get(symbol)
            .and_then(|l| l.name_type.clone());
        if let Some(name_type) = name_type {
            if name_type.flags.intersects(TypeFlags::STRING_OR_NUMBER_LITERAL) {
                let name = match &name_type.as_literal_type().unwrap().value {
                    LiteralValue::String(v) => v.clone(),
                    LiteralValue::Number(v) => v.to_string(),
                    _ => String::new(),
                };
                if !is_identifier_text(&name, LanguageVariant::Standard)
                    && !is_numeric_literal_name(&name)
                {
                    let literal_value = &name_type.as_literal_type().unwrap().value;
                    let pseudo = match literal_value {
                        LiteralValue::String(s) => PseudoLiteralValue::Str(s.clone()),
                        LiteralValue::Number(n) => PseudoLiteralValue::Number(n.0),
                        LiteralValue::Boolean(b) => PseudoLiteralValue::Bool(*b),
                        LiteralValue::BigInt(b) => {
                            PseudoLiteralValue::PseudoBigInt(pseudo_big_int_to_string(b))
                        }
                        LiteralValue::None => PseudoLiteralValue::Str(String::new()),
                    };
                    return crate::checker::mig::m2g::r21k9_defs::value_to_string(&pseudo);
                }
                if is_numeric_literal_name(&name) && name.starts_with('-') {
                    return format!("[{}]", name);
                }
                return name;
            }
            if name_type.flags.intersects(TypeFlags::UniqueESSymbol) {
                let text = &name_type
                    .as_unique_es_symbol_type()
                    .expect("unique symbol type")
                    .name;
                return format!("[{}]", text);
            }
        }
        String::new()
    }

    pub fn get_name_of_symbol_as_written(&mut self, symbol: &Arc<Symbol>) -> String { ::tsox_core::fntrace::enter("get_name_of_symbol_as_written"); 
        let remapped = {
            let ctx = self.ctx.borrow();
            r24k13_defs::remapped_symbol_references_with(&ctx, |m| {
                m.get(&tsox_frontend::ast::get_symbol_id(symbol)).cloned()
            })
        };
        let symbol = remapped.as_ref().unwrap_or(symbol);
        let use_alias_outside = {
            let ctx = self.ctx.borrow();
            !ctx.flags.contains(NodeBuilderFlags::UseAliasDefinedOutsideCurrentScope)
        };
        let in_initial_entity_name = {
            let ctx = self.ctx.borrow();
            !ctx.flags.contains(NodeBuilderFlags::InInitialEntityName)
        };
        if symbol.name == "default"
            && use_alias_outside
            && (in_initial_entity_name
                || symbol.declarations.is_empty()
                || {
                    let decl_loc = tsox_frontend::ast::find_ancestor(
                        &symbol.declarations[0],
                        is_default_binding_context,
                    );
                    let enc_loc = self
                        .ctx
                        .borrow()
                        .enclosing_declaration
                        .as_ref()
                        .and_then(|enc| tsox_frontend::ast::find_ancestor(enc, is_default_binding_context));
                    !r24k13_defs::r24k13_find_ancestor_same(&decl_loc, &enc_loc)
                })
        {
            return "default".to_string();
        }
        if !symbol.declarations.is_empty() {
            let name = symbol
                .declarations
                .iter()
                .filter_map(tsox_frontend::ast::get_name_of_declaration)
                .next();
            if let Some(name) = name {
                let is_late = tsox_frontend::ast::is_computed_property_name(&name)
                    && !symbol.check_flags.intersects(CheckFlags::Late);
                if is_late {
                    let name_type = self
                        .ch
                        .value_symbol_links
                        .get(symbol)
                        .and_then(|l| l.name_type.clone());
                    if name_type
                        .as_ref()
                        .is_some_and(|nt| nt.flags.intersects(TypeFlags::STRING_OR_NUMBER_LITERAL))
                    {
                        let result = self.get_name_of_symbol_from_name_type(symbol);
                        if !result.is_empty() {
                            return result;
                        }
                    }
                }
                return declaration_name_to_string(Some(&name));
            }
            let declaration = &symbol.declarations[0];
            if declaration.parent().is_some_and(|p| p.kind == SyntaxKind::VariableDeclaration)
            {
                if let NodeData::VariableDeclaration(vd) = &declaration.parent().unwrap().data {
                    return declaration_name_to_string(Some(&vd.name));
                }
            }
            if tsox_frontend::ast::is_class_expression(declaration)
                || tsox_frontend::ast::is_function_expression(declaration)
                || tsox_frontend::ast::is_arrow_function(declaration)
            {
                {
                    let mut ctx = self.ctx.borrow_mut();
                    if !ctx.encountered_error
                        && !ctx.flags.contains(NodeBuilderFlags::AllowAnonymousIdentifier)
                    {
                        ctx.encountered_error = true;
                    }
                }
                return match declaration.kind {
                    SyntaxKind::ClassExpression => "(Anonymous class)".to_string(),
                    _ => "(Anonymous function)".to_string(),
                };
            }
        }
        let name = self.get_name_of_symbol_from_name_type(symbol);
        if !name.is_empty() {
            return name;
        }
        r24k13_defs::escape_internal_symbol_name(&symbol.name)
    }

    pub fn get_type_parameters_of_class_or_interface(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_parameters_of_class_or_interface"); 
        let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
        let mut result: Vec<Arc<Type>> = Vec::new();
        result.extend(ch.get_outer_type_parameters_of_class_or_interface(symbol));
        result.extend(
            ch.get_local_type_parameters_of_class_or_interface_or_type_alias(symbol),
        );
        result
    }

    pub fn lookup_type_parameter_nodes(
        &mut self,
        chain: &[Arc<Symbol>],
        index: usize,
    ) -> Option<Arc<tsox_frontend::ast::NodeList>> { ::tsox_core::fntrace::enter("lookup_type_parameter_nodes"); 
        let symbol = &chain[index];
        let symbol_id = tsox_frontend::ast::get_symbol_id(symbol);
        let listed = {
            let ctx = self.ctx.borrow();
            r24k13_defs::type_parameter_symbol_list_with(&ctx, |set| set.contains(&symbol_id))
        };
        if listed {
            return None;
        }
        {
            let ctx = self.ctx.borrow();
            r24k13_defs::type_parameter_symbol_list_with(&ctx, |set| {
                set.insert(symbol_id);
            });
        }

        let write_type_params = {
            let ctx = self.ctx.borrow();
            ctx.flags.contains(NodeBuilderFlags::WriteTypeParametersInQualifiedName)
        };
        if write_type_params && index < chain.len() - 1 {
            if let Some(type_argument_nodes) =
                self.lookup_instantiated_type_argument_nodes(chain, index)
            {
                return Some(type_argument_nodes);
            }
            if let Some(type_parameter_nodes) =
                self.type_parameters_to_type_parameter_declarations(symbol)
            {
                if !type_parameter_nodes.is_empty() {
                    return Some(Arc::new(
                        crate::checker::mig::m2f::r17k8_factory_ext::NodeFactoryExt::new_node_list(
                            &self.f,
                            type_parameter_nodes,
                        ),
                    ));
                }
            }
            return None;
        }

        None
    }

    pub fn lookup_symbol_chain_worker(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
        yield_module_symbol: bool,
    ) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("lookup_symbol_chain_worker"); 
        let mut chain: Vec<Arc<Symbol>> = Vec::new();
        let is_type_parameter = symbol.flags.intersects(SymbolFlags::TypeParameter);
        let (has_enclosing, use_fully_qualified, skip_chain) = {
            let ctx = self.ctx.borrow();
            (
                ctx.enclosing_declaration.is_some(),
                ctx.flags.contains(NodeBuilderFlags::UseFullyQualifiedType),
                ctx.internal_flags
                    .contains(crate::checker::symboltracker::NodeBuilderInternalFlags::DoNotIncludeSymbolChain),
            )
        };
        if !is_type_parameter && (has_enclosing || use_fully_qualified) && !skip_chain {
            let res = self.get_symbol_chain(symbol, meaning, true, yield_module_symbol);
            res
        } else {
            chain.push(Arc::clone(symbol));
            chain
        }
    }
}

fn starts_with_single_or_double_quote(s: &str) -> bool { ::tsox_core::fntrace::enter("starts_with_single_or_double_quote"); 
    s.starts_with('\'') || s.starts_with('"')
}
