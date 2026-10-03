#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::nodecopy_builder::NodeBuilderImpl;
use crate::checker::symboltracker::{NodeBuilderFlags, NodeBuilderInternalFlags};
use std::sync::Arc;
use tsox_frontend::ast::{
    get_name_of_declaration, get_source_file_of_node, is_computed_property_name, Node, Symbol,
    SymbolFlags, SyntaxKind,
};

#[path = "r21k9_defs.rs"]
pub mod r21k9_defs;
#[path = "r22k9_defs.rs"]
pub mod r22k9_defs;
pub use r21k9_defs::{
    count_path_components, EmitContextStubExt, NodeFactoryExt21, NodeM2gExt,
    SortedSymbolNamePair,
};
use r22k9_defs::*;

pub fn starts_with_single_or_double_quote(s: &str) -> bool { ::tsox_core::fntrace::enter("starts_with_single_or_double_quote"); 
    s.starts_with('\'') || s.starts_with('"')
}

pub fn starts_with_square_bracket(s: &str) -> bool { ::tsox_core::fntrace::enter("starts_with_square_bracket"); 
    s.starts_with('[')
}

pub fn types_are_same_reference(a: &Arc<Type>, b: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("types_are_same_reference"); 
    Arc::ptr_eq(a, b)
        || a
            .symbol
            .as_ref()
            .is_some_and(|s| b.symbol.as_ref().is_some_and(|o| Arc::ptr_eq(s, o)))
        || a
            .alias
            .as_ref()
            .zip(b.alias.as_ref())
            .is_some_and(|(al, o)| std::ptr::eq(&**al, &**o))
}

impl<'a> NodeBuilderImpl<'a> {
    pub fn set_comment_range(&mut self, node: &Arc<Node>, range: Option<&Arc<Node>>) { ::tsox_core::fntrace::enter("set_comment_range"); 
        if let Some(range) = range {
            let enclosing = self.ctx.borrow().enclosing_file.clone();
            if let Some(enclosing_file) = enclosing {
                let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
                if ch
                    .get_source_file_of_node(range)
                    .is_some_and(|sf| Arc::ptr_eq(&enclosing_file, &sf))
                {
                    self.e.assign_comment_range(node, range);
                }
            }
        }
    }

    pub fn should_expand_type(&mut self, t: &Arc<Type>, is_alias: bool) -> bool { ::tsox_core::fntrace::enter("should_expand_type"); 
        {
            let mut ctx = self.ctx.borrow_mut();
            if ctx.max_expansion_depth < 0 {
                return false;
            }
        }
        if !self.is_expandable_type(t, is_alias) {
            return false;
        }
        if self.is_type_on_stack(t) {
            return false;
        }
        let mut ctx = self.ctx.borrow_mut();
        if ctx.depth < ctx.max_expansion_depth as usize {
            return true;
        }
        ctx.can_increase_expansion_depth = true;
        false
    }

    pub fn type_node_is_equivalent_to_type(
        &mut self,
        annotated_declaration: Option<&Arc<Node>>,
        t: &Arc<Type>,
        type_from_type_node: &Arc<Type>,
    ) -> bool { ::tsox_core::fntrace::enter("type_node_is_equivalent_to_type"); 
        if Arc::ptr_eq(type_from_type_node, t) {
            return true;
        }
        let Some(annotated_declaration) = annotated_declaration else {
            return false;
        };
        if crate::checker::utilities_has_only_expression_initialization::is_optional_declaration(annotated_declaration) {
            let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
            let with_facts = ch.get_type_with_facts(t, TypeFacts::NE_UNDEFINED);
            return Arc::ptr_eq(&with_facts, type_from_type_node);
        }
        false
    }

    pub fn try_reuse_existing_non_parameter_type_node(
        &mut self,
        existing: &Arc<Node>,
        t: &Arc<Type>,
        host: Option<&Arc<Node>>,
        annotation_type: Option<&Arc<Type>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("try_reuse_existing_non_parameter_type_node"); 
        let host = host
            .cloned()
            .or_else(|| self.ctx.borrow().enclosing_declaration.clone());
        let resolved_annotation = match annotation_type {
            Some(at) => Some(Arc::clone(at)),
            None => self.get_type_from_type_node(existing, true),
        };
        if let Some(annotation_type) = resolved_annotation {
            if self.type_node_is_equivalent_to_type(host.as_ref(), t, &annotation_type)
                && self.can_reuse_existing_js_type_node(existing, Some(t))
            {
                if let Some(result) = self.try_reuse_existing_node_helper(existing) {
                    return Some(result);
                }
            }
        }
        None
    }

    pub fn symbol_to_node(&mut self, symbol: &Arc<Symbol>, meaning: SymbolFlags) -> Arc<Node> { ::tsox_core::fntrace::enter("symbol_to_node"); 
        let write_computed = self
            .ctx
            .borrow()
            .internal_flags
            .contains(NodeBuilderInternalFlags::WriteComputedProps);
        if write_computed {
            if let Some(value_declaration) = symbol.value_declaration.as_ref() {
                if let Some(name) = get_name_of_declaration(value_declaration) {
                    if is_computed_property_name(&name) {
                        return name;
                    }
                }
            }
            if let Some(links) = self.ch.value_symbol_links.get(symbol) {
                if let Some(name_type) = links.name_type.as_ref() {
                    if name_type.flags.intersects(
                        TypeFlags::ENUM_LITERAL | TypeFlags::UNIQUE_ES_SYMBOL,
                    ) {
                        let old_enclosing =
                            self.ctx.borrow().enclosing_declaration.clone();
                        let name_symbol = name_type.symbol.clone().unwrap();
                        let decl = name_symbol.value_declaration.clone();
                        self.ctx.borrow_mut().enclosing_declaration = decl;
                        let mut expression = self.symbol_to_expression(&name_symbol, meaning);
                        let result = self.f.new_computed_property_name(&mut expression);
                        self.ctx.borrow_mut().enclosing_declaration = old_enclosing;
                        return result;
                    }
                }
            }
        }
        self.symbol_to_expression(symbol, meaning)
    }

    pub fn symbol_to_name(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
        expects_identifier: bool,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("symbol_to_name"); 
        let chain = self.lookup_symbol_chain(symbol, meaning, false);
        let allow_qualified = {
            let ctx = self.ctx.borrow();
            expects_identifier
                && chain.len() != 1
                && !ctx.encountered_error
                && ctx
                    .flags
                    .contains(NodeBuilderFlags::AllowQualifiedNameInPlaceOfIdentifier)
        };
        if allow_qualified {
            self.ctx.borrow_mut().encountered_error = true;
        }
        let index = chain.len().saturating_sub(1);
        self.create_entity_name_from_symbol_chain(&chain, index)
    }

    pub fn symbol_to_entity_name_node(&mut self, symbol: &Arc<Symbol>) -> Arc<Node> { ::tsox_core::fntrace::enter("symbol_to_entity_name_node"); 
        let identifier = self.new_identifier(&symbol.name.clone(), Some(symbol));
        if let Some(parent) = symbol.parent() {
            let left = self.symbol_to_entity_name_node(&parent);
            return self.f.new_qualified_name(&left, &identifier);
        }
        identifier
    }

    pub fn symbol_to_expression(&mut self, symbol: &Arc<Symbol>, meaning: SymbolFlags) -> Arc<Node> { ::tsox_core::fntrace::enter("symbol_to_expression"); 
        let chain = self.lookup_symbol_chain(symbol, meaning, false);
        let index = chain.len().saturating_sub(1);
        self.create_expression_from_symbol_chain(&chain, index)
    }

    pub fn sort_by_best_name(&self, a: &SortedSymbolNamePair, b: &SortedSymbolNamePair) -> i32 { ::tsox_core::fntrace::enter("sort_by_best_name"); 
        let specifier_a = a.name.as_str();
        let specifier_b = b.name.as_str();
        if !specifier_a.is_empty() && !specifier_b.is_empty() {
            let is_b_relative = tsox_core::tspath::path_is_relative(specifier_b);
            if tsox_core::tspath::path_is_relative(specifier_a) == is_b_relative {
                return count_path_components(specifier_a) - count_path_components(specifier_b);
            }
            if is_b_relative {
                return -1;
            }
            return 1;
        }
        self.ch.compare_symbols(&a.sym, &b.sym)
    }

    pub fn type_parameter_to_declaration_with_constraint(
        &mut self,
        type_parameter: &Arc<Type>,
        constraint_node: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("type_parameter_to_declaration_with_constraint"); 
        let restore_flags = self.save_restore_flags();
        self.ctx
            .borrow_mut()
            .flags
            .remove(NodeBuilderFlags::WriteTypeParametersInQualifiedName);
        let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
        let modifiers = self
            .f
            .create_modifiers_from_modifier_flags(ch.get_type_parameter_modifiers(type_parameter));
        let modifiers_list = if !modifiers.is_empty() {
            self.f.new_modifier_list(&modifiers)
        } else {
            None
        };
        let name = self.type_parameter_to_name(type_parameter);
        let default_parameter = self.ch.get_default_from_type_parameter(type_parameter);
        let default_parameter_declaration_node = default_parameter
            .as_ref()
            .and_then(|default| self.type_to_type_node(default));
        restore_flags.restore();
        self.f.new_type_parameter_declaration(
            modifiers_list,
            &name,
            constraint_node,
            None,
            default_parameter_declaration_node,
        )
    }

    pub fn type_parameter_shadows_other_type_parameter_in_scope(
        &mut self,
        name: &str,
        type_parameter: &Arc<Type>,
    ) -> bool { ::tsox_core::fntrace::enter("type_parameter_shadows_other_type_parameter_in_scope"); 
        let enclosing = self.ctx.borrow().enclosing_declaration.clone();
        let result = match enclosing.as_ref() {
            Some(location) => self.ch.resolve_name(name, location, SymbolFlags::TYPE, false),
            None => None,
        };
        if let Some(result) = result {
            if result.flags.contains(SymbolFlags::TypeParameter) {
                let shadowed = match (&result, &type_parameter.symbol) {
                    (r, Some(tp)) => !Arc::ptr_eq(r, tp),
                    (_, None) => true,
                };
                return shadowed;
            }
        }
        false
    }

    pub fn type_predicate_to_type_predicate_node(
        &mut self,
        predicate: &TypePredicate,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("type_predicate_to_type_predicate_node"); 
        let asserts_modifier = if predicate.kind == TypePredicateKind::AssertsIdentifier
            || predicate.kind == TypePredicateKind::AssertsThis
        {
            Some(self.f.new_token(SyntaxKind::AssertsKeyword))
        } else {
            None
        };
        let parameter_name = if predicate.kind == TypePredicateKind::Identifier
            || predicate.kind == TypePredicateKind::AssertsIdentifier
        {
            let id = self.f.new_identifier(&predicate.parameter_name.clone());
            self.e
                .add_emit_flags(&id, crate::checker::mig::m2g::r21k9_defs::EF_NO_ASCII_ESCAPING);
            id
        } else {
            self.f.new_this_type_node()
        };
        let type_node = predicate.t.as_ref().and_then(|t| self.type_to_type_node(t));
        self.f.new_type_predicate_node(asserts_modifier, &parameter_name, type_node)
    }

    pub fn type_to_type_node_helper_with_possible_reusable_type_node(
        &mut self,
        t: Option<&Arc<Type>>,
        type_node: Option<&Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("type_to_type_node_helper_with_possible_reusable_type_node"); 
        let Some(t) = t else {
            return self.f.new_keyword_type_node(SyntaxKind::AnyKeyword);
        };
        if !self.is_actively_expanding() {
            if let Some(type_node) = type_node {
                let existing_t = self.get_type_from_type_node(type_node, false);
                if existing_t.as_ref().is_some_and(|et| Arc::ptr_eq(et, t)) {
                    if let Some(reused) = self.try_reuse_existing_node_helper(type_node) {
                        self.check_type_expandability(t);
                        return reused;
                    }
                }
            }
        }
        self.type_to_type_node_ex(t)
    }

    pub fn type_parameter_to_declaration(&mut self, parameter: &Arc<Type>) -> Arc<Node> { ::tsox_core::fntrace::enter("type_parameter_to_declaration"); 
        let constraint = self.ch.get_constraint_of_type_parameter(parameter);
        let constraint_node = constraint.as_ref().map(|constraint| {
            let reusable = self.ch.get_constraint_declaration(parameter);
            self.type_to_type_node_helper_with_possible_reusable_type_node(
                Some(constraint),
                reusable.as_ref(),
            )
        });
        self.type_parameter_to_declaration_with_constraint(parameter, constraint_node)
    }

    pub fn symbol_to_type_parameter_declarations(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<Vec<Arc<Node>>> { ::tsox_core::fntrace::enter("symbol_to_type_parameter_declarations"); 
        self.type_parameters_to_type_parameter_declarations(symbol)
    }

    pub fn type_parameters_to_type_parameter_declarations(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<Vec<Arc<Node>>> { ::tsox_core::fntrace::enter("type_parameters_to_type_parameter_declarations"); 
        let target_symbol = self.ch.get_target_symbol(symbol);
        if target_symbol
            .flags
            .intersects(SymbolFlags::Class | SymbolFlags::Interface | SymbolFlags::Alias)
        {
            let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
            let params =
                ch.get_local_type_parameters_of_class_or_interface_or_type_alias(symbol);
            let mut results = Vec::with_capacity(params.len());
            for param in &params {
                results.push(self.type_parameter_to_declaration(param));
            }
            return Some(results);
        } else if target_symbol.flags.intersects(SymbolFlags::Function) {
            let value_declaration = symbol.value_declaration.clone()?;
            let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
            let params = ch.get_type_parameters_from_declaration(&value_declaration);
            let mut results = Vec::with_capacity(params.len());
            for param in &params {
                results.push(self.type_parameter_to_declaration(param));
            }
            return Some(results);
        }
        None
    }
}

pub fn try_get_module_specifier_from_declaration_worker(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("try_get_module_specifier_from_declaration_worker"); 
    match node.kind {
        SyntaxKind::VariableDeclaration | SyntaxKind::BindingElement => {
            let module_call = tsox_frontend::ast::find_ancestor(
                node.initializer()?,
                |n: &Node| tsox_frontend::ast::is_require_call(n, true)
                    || tsox_frontend::ast::is_import_call(n),
            )?;
            module_call
                .arguments()
                .and_then(|l| l.nodes.first().cloned())
        }
        SyntaxKind::ImportDeclaration
        | SyntaxKind::ExportDeclaration
        | SyntaxKind::JSDocImportTag => node.module_specifier().cloned(),
        SyntaxKind::ImportEqualsDeclaration => {
            let reference = node.as_import_equals_declaration()?.module_reference.clone();
            if reference.kind != SyntaxKind::ExternalModuleReference {
                return None;
            }
            reference.expression().cloned()
        }
        SyntaxKind::ImportClause => node.parent()?.module_specifier().cloned(),
        SyntaxKind::NamespaceExport => node.parent()?.module_specifier().cloned(),
        SyntaxKind::NamespaceImport => node.parent()?.parent()?.module_specifier().cloned(),
        SyntaxKind::ExportSpecifier => node.parent()?.parent()?.module_specifier().cloned(),
        SyntaxKind::ImportSpecifier => {
            node.parent()?.parent()?.parent()?.module_specifier().cloned()
        }
        SyntaxKind::ImportType => {
            if !crate::checker::mig::m1a::r19k2_defs::is_literal_import_type_node(node) {
                return None;
            }
            let d = node.as_import_type_node()?;
            let literal = match &d.argument.data {
                tsox_frontend::ast::NodeData::LiteralTypeNode(l) => l.literal.clone(),
                _ => return None,
            };
            Some(literal)
        }
        _ => None,
    }
}
