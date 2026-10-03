#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::nodecopy_builder::{NodeBuilderImpl, NodeFactoryStub};
use crate::checker::symboltracker::{NodeBuilderFlags, NodeBuilderInternalFlags};
use crate::checker::mig::m2a::is_rest_parameter;
use crate::checker::mig::wc1b::is_type_any;
use crate::checker::mig::m2g::r21k9_defs::*;
use crate::checker::mig::m2g::r22k9_defs::*;
use std::rc::Rc;
use std::sync::Arc;
use tsox_frontend::ast::{
    can_have_modifiers, get_name_of_declaration, get_symbol_id, is_accessor, is_class_like,
    is_in_js_file, node_is_synthesized, Node, Symbol, SymbolFlags, SyntaxKind,
};

impl<'a> NodeBuilderImpl<'a> {
    pub fn should_write_type_parameters_in_qualified_name(
        &self,
        chain: &[Arc<Symbol>],
        index: usize,
    ) -> bool { ::tsox_core::fntrace::enter("should_write_type_parameters_in_qualified_name"); 
        self.ctx
            .borrow()
            .flags
            .contains(NodeBuilderFlags::WriteTypeParametersInQualifiedName)
            && index + 1 < chain.len()
    }

    pub fn track_computed_name(
        &mut self,
        access_expression: &Arc<Node>,
        enclosing_declaration: Option<&Arc<Node>>,
    ) { ::tsox_core::fntrace::enter("track_computed_name"); 
        let first_identifier = tsox_frontend::ast::mig::m3e_4::get_first_identifier(access_expression);
        let location = enclosing_declaration.cloned().unwrap_or(Arc::clone(access_expression));
        let name = self.ch.resolve_name(
            &first_identifier.text(),
            &location,
            SymbolFlags::VALUE | SymbolFlags::ExportValue,
            true,
        );
        if let Some(name) = name {
            self.ctx
                .borrow_mut()
                .track_symbol(&name, enclosing_declaration, SymbolFlags::VALUE);
        } else {
            let fallback = self.ch.resolve_name(
                &first_identifier.text(),
                &first_identifier,
                SymbolFlags::VALUE | SymbolFlags::ExportValue,
                true,
            );
            if let Some(fallback) = fallback {
                self.ctx
                    .borrow_mut()
                    .track_symbol(&fallback, enclosing_declaration, SymbolFlags::VALUE);
            }
        }
    }

    pub fn try_get_this_parameter_declaration(
        &mut self,
        signature: &Signature,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("try_get_this_parameter_declaration"); 
        if let Some(this_parameter) = signature.this_parameter.as_ref() {
            return Some(self.symbol_to_parameter_declaration(this_parameter, false));
        }
        let _ = signature
            .declaration
            .as_ref()
            .filter(|d| is_in_js_file(d));
        None
    }

    pub fn serialize_type_for_expression(&mut self, expr: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("serialize_type_for_expression"); 
        let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
        let regular = ch.get_regular_type_of_expression(expr);
        let widened = ch.get_widened_type(&regular);
        let mapper = side_with(&self.ctx.borrow(), |s| s.mapper.clone());
        let t = ch.instantiate_type(&widened, mapper.as_ref());
        self.type_to_type_node_ex(&t)
    }

    pub fn type_predicate_to_type_predicate_node_helper(
        &mut self,
        type_predicate: &TypePredicate,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("type_predicate_to_type_predicate_node_helper"); 
        let asserts_modifier = if type_predicate.kind == TypePredicateKind::AssertsThis
            || type_predicate.kind == TypePredicateKind::AssertsIdentifier
        {
            Some(self.f.new_token(SyntaxKind::AssertsKeyword))
        } else {
            None
        };
        let parameter_name = if type_predicate.kind == TypePredicateKind::Identifier
            || type_predicate.kind == TypePredicateKind::AssertsIdentifier
        {
            let id = self.new_identifier(&type_predicate.parameter_name.clone(), None);
            self.e.add_emit_flags(&id, EF_NO_ASCII_ESCAPING);
            id
        } else {
            self.f.new_this_type_node()
        };
        let type_node = type_predicate
            .t
            .as_ref()
            .and_then(|t| self.type_to_type_node(t));
        self.f
            .new_type_predicate_node(asserts_modifier, &parameter_name, type_node)
    }

    pub fn serialize_inferred_return_type_for_signature(
        &mut self,
        signature: &Signature,
        return_type: &Arc<Type>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("serialize_inferred_return_type_for_signature"); 
        let signature = Arc::new(signature.clone());
        let old_suppress = self.ctx.borrow().suppress_report_inference_fallback;
        self.ctx.borrow_mut().suppress_report_inference_fallback = true;
        let type_predicate = self.ch.get_type_predicate_of_signature(&signature);
        let return_type_node = match type_predicate {
            Some(type_predicate) => {
                let mapper = side_with(&self.ctx.borrow(), |s| s.mapper.clone());
                let predicate = match mapper {
                    Some(mapper) => {
                        let ch =
                            unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
                        ch.instantiate_type_predicate(type_predicate, &mapper)
                    }
                    None => Some(Box::new(type_predicate.clone())),
                };
                match predicate.as_deref() {
                    Some(p) => self.type_predicate_to_type_predicate_node_helper(p),
                    None => self.type_to_type_node_ex(return_type),
                }
            }
            None => self.type_to_type_node_ex(return_type),
        };
        self.ctx.borrow_mut().suppress_report_inference_fallback = old_suppress;
        return_type_node
    }

    pub fn serialize_return_type_for_signature(
        &mut self,
        signature: &Signature,
        try_reuse: bool,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("serialize_return_type_for_signature"); 
        let signature = Arc::new(signature.clone());
        let suppress_any = self
            .ctx
            .borrow()
            .flags
            .contains(NodeBuilderFlags::SuppressAnyReturnType);
        let restore_flags = self.save_restore_flags();
        if suppress_any {
            self.ctx
                .borrow_mut()
                .flags
                .remove(NodeBuilderFlags::SuppressAnyReturnType);
        }
        let declaration_is_real = signature
            .declaration
            .as_ref()
            .is_some_and(|d| !node_is_synthesized(d));
        let return_type = if declaration_is_real {
            let declaration = signature.declaration.clone().unwrap();
            let symbol = self.ch.get_symbol_of_declaration(&declaration);
            let symbol_id = symbol.as_ref().map(get_symbol_id).unwrap_or(0) as usize;
            let cached = side_with(&self.ctx.borrow(), |s| {
                s.enclosing_symbol_types.get(&symbol_id).cloned()
            });
            match cached {
                Some(t) => t,
                None => {
                    let mapper = side_with(&self.ctx.borrow(), |s| s.mapper.clone());
                    match self.ch.get_return_type_of_signature(&signature) {
                        Some(rt) => {
                            let ch = unsafe {
                                &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr()
                            };
                            ch.instantiate_type(&rt, mapper.as_ref())
                        }
                        None => self.ch.unknown_type(),
                    }
                }
            }
        } else {
            self.ch
                .get_return_type_of_signature(&signature)
                .unwrap_or_else(|| self.ch.unknown_type())
        };
        let mut return_type_node: Option<Arc<Node>> = None;
        if !(suppress_any && is_type_any(&return_type)) {
            let enclosing_declaration = self.ctx.borrow().enclosing_declaration.clone();
            if !self.is_actively_expanding()
                && try_reuse
                && enclosing_declaration.is_some()
                && declaration_is_real
            {
                let declaration = signature.declaration.clone().unwrap();
                let declaration_symbol = self.ch.get_symbol_of_declaration(&declaration);
                let restore =
                    self.add_symbol_type_to_context(declaration_symbol.as_ref(), &return_type);
                let mut pt = self.pc.get_return_type_of_signature(&declaration);
                let suppress_fallback = self.ctx.borrow().suppress_report_inference_fallback;
                if self.pseudo_type_equivalent_to_type(pt.as_ref(), &return_type, false, !suppress_fallback)
                {
                    let type_predicate = self.ch.get_type_predicate_of_signature(&signature);
                    if let Some(type_predicate) = type_predicate {
                        if !self.pseudo_return_type_matches_predicate(pt.as_ref(), &type_predicate)
                        {
                            if !suppress_fallback {
                                let mut ctx = self.ctx.borrow_mut();
                                if let Some(tracker) = ctx.tracker.as_mut() {
                                    tracker.report_inference_fallback(&declaration);
                                }
                            }
                            pt = None;
                        }
                    }
                    if pt.is_some() {
                        return_type_node =
                            Some(self.pseudo_type_to_node_with_checker_fallback(
                                pt.as_ref(),
                                &return_type,
                            ));
                    }
                }
                restore();
            }
            if return_type_node.is_none() {
                return_type_node =
                    Some(self.serialize_inferred_return_type_for_signature(&signature, &return_type));
            }
        }
        if return_type_node.is_none() && !suppress_any {
            return_type_node = Some(self.f.new_keyword_type_node(SyntaxKind::AnyKeyword));
        }
        restore_flags.restore();
        return_type_node.unwrap()
    }

    pub fn should_write_type_of_function_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        type_id: TypeId,
    ) -> (bool, Arc<Symbol>) { ::tsox_core::fntrace::enter("should_write_type_of_function_symbol"); 
        let is_static_method_symbol = symbol.flags.contains(SymbolFlags::Method)
            && symbol.declarations.iter().any(|declaration| {
                tsox_frontend::ast::is_static(declaration)
                    && !unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() }
                        .is_late_bindable_index_signature(declaration)
            });
        let mut is_non_local_function_symbol = false;
        let mut is_function_expression_symbol = false;
        if symbol.flags.contains(SymbolFlags::Function) {
            if symbol.parent().is_some() {
                is_non_local_function_symbol = true;
            } else {
                for declaration in &symbol.declarations {
                    let parent = declaration.parent();
                    if let Some(parent) = parent {
                        if parent.kind == SyntaxKind::SourceFile
                            || parent.kind == SyntaxKind::ModuleBlock
                        {
                            is_non_local_function_symbol = true;
                            break;
                        }
                        let grand = parent.parent();
                        let great = grand.as_ref().and_then(|g| g.parent());
                        let great2 = great.as_ref().and_then(|g| g.parent());
                        if tsox_frontend::ast::is_function_expression_or_arrow_function(declaration)
                            && parent.kind == SyntaxKind::VariableDeclaration
                            && grand
                                .as_ref()
                                .is_some_and(|g| g.kind == SyntaxKind::VariableDeclarationList)
                            && great
                                .as_ref()
                                .is_some_and(|g| g.kind == SyntaxKind::VariableStatement)
                            && great2.is_some()
                            && (great2.as_ref().unwrap().kind == SyntaxKind::SourceFile
                                || great2.as_ref().unwrap().kind == SyntaxKind::ModuleBlock)
                        {
                            is_non_local_function_symbol = true;
                            is_function_expression_symbol = true;
                            break;
                        }
                    }
                }
            }
        }
        let mut symbol = Arc::clone(symbol);
        if is_static_method_symbol || is_non_local_function_symbol {
            if is_function_expression_symbol {
                if let Some(value_declaration) = symbol.value_declaration.as_ref() {
                    if let Some(parent) = value_declaration.parent() {
                        let enclosing = self.ctx.borrow().enclosing_declaration.clone();
                        if !enclosing.as_ref().is_some_and(|e| Arc::ptr_eq(e, &parent)) {
                            if let Some(parent_symbol) = self.ch.get_symbol_of_declaration(&parent) {
                                symbol = self.ch.get_merged_symbol(&parent_symbol);
                            }
                        }
                    }
                }
            }
            let enclosing_declaration = self.ctx.borrow().enclosing_declaration.clone();
            let use_type_of_allowed = {
                let ctx = self.ctx.borrow();
                ctx.flags.contains(NodeBuilderFlags::UseTypeOfFunction)
                    || side_with(&ctx, |s| s.visited_types.has(type_id))
            };
            let structural_fallback_ok = {
                let ctx = self.ctx.borrow();
                let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
                !ctx.flags.contains(NodeBuilderFlags::UseStructuralFallback)
                    || ch.is_value_symbol_accessible(&symbol, enclosing_declaration.as_ref())
            };
            return (use_type_of_allowed && structural_fallback_ok, symbol);
        }
        (false, symbol)
    }

    pub fn should_emit_type_of_symbol(
        &mut self,
        force_expansion: bool,
        force_class_expansion: bool,
        is_instance_type: SymbolFlags,
        symbol: &Arc<Symbol>,
        type_id: TypeId,
    ) -> (bool, Arc<Symbol>) { ::tsox_core::fntrace::enter("should_emit_type_of_symbol"); 
        if force_expansion {
            return (false, Arc::clone(symbol));
        }
        let enclosing_declaration = self.ctx.borrow().enclosing_declaration.clone();
        let write_class_expression = self
            .ctx
            .borrow()
            .flags
            .contains(NodeBuilderFlags::WriteClassExpressionAsTypeLiteral);
        let class_result = symbol.flags.contains(SymbolFlags::Class)
            && !force_class_expansion
            && (unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() })
                .get_base_type_variable_of_class(symbol)
                .is_none()
            && !(symbol.value_declaration.as_ref().is_some_and(|vd| is_class_like(vd))
                && write_class_expression
                && (!tsox_frontend::ast::is_class_declaration(symbol.value_declaration.as_ref().unwrap())
                    || (unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() })
                        .is_symbol_accessible(
                            symbol,
                            enclosing_declaration.as_ref(),
                            is_instance_type,
                            false,
                        )
                        .accessibility
                        != SymbolAccessibility::Accessible));
        let non_function_result = class_result
            || symbol
                .flags
                .intersects(SymbolFlags::ENUM | SymbolFlags::ValueModule);
        if non_function_result {
            return (true, Arc::clone(symbol));
        }
        self.should_write_type_of_function_symbol(symbol, type_id)
    }

    pub fn type_to_type_node_or_circularity_elision(&mut self, t: &Arc<Type>) -> Arc<Node> { ::tsox_core::fntrace::enter("type_to_type_node_or_circularity_elision"); 
        if t.flags.contains(TypeFlags::UNION) {
            let visited = side_with(&self.ctx.borrow(), |s| s.visited_types.has(t.id));
            if visited {
                let allow_anonymous = self
                    .ctx
                    .borrow()
                    .flags
                    .contains(NodeBuilderFlags::AllowAnonymousIdentifier);
                if !allow_anonymous {
                    self.ctx.borrow_mut().encountered_error = true;
                    let mut ctx = self.ctx.borrow_mut();
                    if let Some(tracker) = ctx.tracker.as_mut() {
                        tracker.report_cyclic_structure_error();
                    }
                }
                return self.create_elided_information_placeholder();
            }
            return self.visit_and_transform_type(t, Self::type_to_type_node);
        }
        self.type_to_type_node_ex(t)
    }

    pub fn visit_and_transform_type(
        &mut self,
        t: &Arc<Type>,
        transform: fn(&mut Self, &Arc<Type>) -> Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_and_transform_type"); 
        let type_id = t.id;
        let is_constructor_object = t.object_flags.contains(ObjectFlags::ANONYMOUS)
            && t.symbol
                .as_ref()
                .is_some_and(|s| s.flags.contains(SymbolFlags::Class));
        let id: Option<CompositeSymbolIdentity> = if t.object_flags.contains(ObjectFlags::Reference)
            && t.as_instantiation_expression_type()
                .is_some_and(|r| r.node.is_some())
        {
            let ref_node = t
                .as_instantiation_expression_type()
                .and_then(|r| r.node.as_ref())
                .unwrap();
            Some(CompositeSymbolIdentity {
                is_constructor_object: false,
                symbol_id: 0,
                node_id: tsox_frontend::ast::mig::m3f::get_node_id(ref_node) as usize,
            })
        } else if t.flags.contains(TypeFlags::Conditional) {
            let cond_node = t
                .as_conditional_type()
                .and_then(|d| d.root.as_ref())
                .and_then(|r| r.node.as_ref())
                .unwrap();
            Some(CompositeSymbolIdentity {
                is_constructor_object: false,
                symbol_id: 0,
                node_id: tsox_frontend::ast::mig::m3f::get_node_id(cond_node) as usize,
            })
        } else if let Some(symbol) = t.symbol.as_ref() {
            Some(CompositeSymbolIdentity {
                is_constructor_object,
                symbol_id: get_symbol_id(symbol) as usize,
                node_id: 0,
            })
        } else {
            None
        };
        let (flags, internal_flags) = {
            let ctx = self.ctx.borrow();
            (ctx.flags, ctx.internal_flags)
        };
        let key = CompositeTypeCacheIdentity {
            type_id,
            flags_bits: flags.bits(),
            internal_flags_bits: internal_flags.bits() as u32,
        };
        let can_use_cache = self.ctx.borrow().max_expansion_depth < 0;
        if can_use_cache {
            let enclosing = self.ctx.borrow().enclosing_declaration.clone();
            if let Some(enclosing) = enclosing {
                if links_has(&enclosing) {
                    let cached_result = links_with(&enclosing, |m| m.get(&key).cloned());
                    if let Some(cached_result) = cached_result {
                        for arg in &cached_result.tracked_symbols {
                            let mut ctx = self.ctx.borrow_mut();
                            ctx.track_symbol(&arg.symbol, arg.enclosing_declaration.as_ref(), arg.meaning);
                        }
                        if cached_result.truncating {
                            self.ctx.borrow_mut().truncating = true;
                        }
                        self.ctx.borrow_mut().approximate_length += cached_result.added_length;
                        return self.deep_clone_node(&cached_result.node);
                    }
                }
            }
        }
        let depth = id
            .as_ref()
            .map(|id| side_with(&self.ctx.borrow_mut(), |s| *s.symbol_depth.entry(*id).or_insert(0)))
            .unwrap_or(0);
        if id.is_some() {
            if depth > 10 {
                return self.create_elided_information_placeholder();
            }
            let id_val = *id.as_ref().unwrap();
            let depth_next = depth + 1;
            side_with(&self.ctx.borrow_mut(), |s| {
                s.symbol_depth.insert(id_val, depth_next)
            });
        }
        side_with(&self.ctx.borrow_mut(), |s| s.visited_types.add(type_id));
        let prev_tracked_symbols = std::mem::take(&mut self.ctx.borrow_mut().tracked_symbols);
        let start_length = self.ctx.borrow().approximate_length;
        let transformed = transform(self, t);
        let result = transformed
            .unwrap_or_else(|| NodeFactoryStub.new_keyword_type_node(SyntaxKind::UnknownKeyword));
        let added_length = self.ctx.borrow().approximate_length - start_length;
        if can_use_cache {
            let ctx = self.ctx.borrow();
            if !ctx.reported_diagnostic && !ctx.encountered_error {
                drop(ctx);
                let enclosing = self.ctx.borrow().enclosing_declaration.clone();
                if let Some(enclosing) = enclosing {
                    let entry = SerializedTypeEntry {
                        node: Arc::clone(&result),
                        truncating: self.ctx.borrow().truncating,
                        added_length,
                        tracked_symbols: self.ctx.borrow().tracked_symbols.clone(),
                    };
                    links_with(&enclosing, |m| m.insert(key, entry));
                }
            }
        }
        side_with(&self.ctx.borrow_mut(), |s| s.visited_types.delete(type_id));
        if let Some(id) = id.as_ref() {
            let id_val = *id;
            side_with(&self.ctx.borrow_mut(), |s| s.symbol_depth.insert(id_val, depth));
        }
        self.ctx.borrow_mut().tracked_symbols = prev_tracked_symbols;
        result
    }

    pub fn add_symbol_type_to_context(
        &mut self,
        symbol: Option<&Arc<Symbol>>,
        t: &Arc<Type>,
    ) -> Box<dyn FnOnce()> { ::tsox_core::fntrace::enter("add_symbol_type_to_context"); 
        let id = symbol.map(get_symbol_id).unwrap_or(0) as usize;
        let old = side_with(&self.ctx.borrow_mut(), |s| {
            s.enclosing_symbol_types.insert(id, Arc::clone(t))
        });
        let ctx = Rc::clone(&self.ctx);
        Box::new(move || {
            let old = old;
            side_with(&ctx.borrow(), |s| match old {
                Some(old_type) => {
                    s.enclosing_symbol_types.insert(id, old_type);
                }
                None => {
                    s.enclosing_symbol_types.remove(&id);
                }
            });
        })
    }

    pub fn clone_node_builder_context(&mut self) -> Box<dyn FnOnce()> { ::tsox_core::fntrace::enter("clone_node_builder_context"); 
        let restore_names = side_with(&self.ctx.borrow_mut(), |s| s.type_parameter_names.enter_scope());
        let restore_names_by_text =
            side_with(&self.ctx.borrow_mut(), |s| s.type_parameter_names_by_text.enter_scope());
        let restore_names_by_text_next_name_count = side_with(&self.ctx.borrow_mut(), |s| {
            s.type_parameter_names_by_text_next_name_count.enter_scope()
        });
        let restore_symbol_list =
            side_with(&self.ctx.borrow_mut(), |s| s.type_parameter_symbol_list.enter_scope());
        let ctx = Rc::clone(&self.ctx);
        Box::new(move || {
            restore_names(&mut ctx.borrow_mut());
            restore_names_by_text(&mut ctx.borrow_mut());
            restore_names_by_text_next_name_count(&mut ctx.borrow_mut());
            restore_symbol_list(&mut ctx.borrow_mut());
        })
    }

    pub fn enter_signature_scope(
        &mut self,
        signature: &Signature,
    ) -> (Vec<Arc<Symbol>>, Box<dyn FnOnce()>) { ::tsox_core::fntrace::enter("enter_signature_scope"); 
        let signature = Arc::new(signature.clone());
        let expanded_params = self
            .ch
            .get_expanded_parameters(&signature, true)
            .into_iter()
            .next()
            .unwrap_or_default();
        let cleanup = match signature.declaration.as_ref() {
            Some(declaration) => {
                let restore = self.enter_new_scope(
                    declaration,
                    Some(expanded_params.clone()),
                    Some(signature.type_parameters.clone()),
                    None,
                    None,
                );
                restore as Box<dyn FnOnce()>
            }
            None => Box::new(|| {}) as Box<dyn FnOnce()>,
        };
        (expanded_params, cleanup)
    }

    pub fn symbol_to_parameter_declaration(
        &mut self,
        parameter_symbol: &Arc<Symbol>,
        preserve_modifier_flags: bool,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("symbol_to_parameter_declaration"); 
        let parameter_declaration = get_effective_parameter_declaration(parameter_symbol);
        let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
        let parameter_type = ch.get_type_of_symbol(parameter_symbol);
        let parameter_type_node = self.serialize_type_for_declaration(
            parameter_declaration.as_ref(),
            &parameter_type,
            Some(parameter_symbol),
            true,
        );
        let omit_modifiers = self
            .ctx
            .borrow()
            .flags
            .contains(NodeBuilderFlags::OmitParameterModifiers);
        let mut modifiers = None;
        if !omit_modifiers
            && preserve_modifier_flags
            && parameter_declaration
                .as_ref()
                .is_some_and(|d| can_have_modifiers(d))
        {
            let declaration = parameter_declaration.clone().unwrap();
            let clones: Vec<Arc<Node>> = declaration
                .modifier_nodes()
                .into_iter()
                .filter(|node| tsox_frontend::ast::mig::m3g::is_modifier(node))
                .map(|node| self.f.clone_node(&node))
                .collect();
            if !clones.is_empty() {
                modifiers = self.f.new_modifier_list(&clones);
            }
        }
        let is_rest = parameter_declaration
            .as_ref()
            .is_some_and(|d| is_rest_parameter(d))
            || parameter_symbol
                .check_flags
                .contains(CheckFlags::RestParameter);
        let dot_dot_dot_token = if is_rest {
            Some(self.f.new_token(SyntaxKind::DotDotDotToken))
        } else {
            None
        };
        let name = self
            .parameter_to_parameter_declaration_name(
                parameter_symbol,
                parameter_declaration.as_ref(),
            )
            .unwrap_or_else(|| {
                self.f
                    .new_identifier(&parameter_symbol.name)
            });
        let is_optional = parameter_declaration
            .as_ref()
            .is_some_and(|d| self.ch.is_optional_parameter(d))
            || parameter_symbol
                .check_flags
                .contains(CheckFlags::OptionalParameter);
        let question_token = if is_optional {
            Some(self.f.new_token(SyntaxKind::QuestionToken))
        } else {
            None
        };
        let parameter_node = self.f.new_parameter_declaration(
            modifiers,
            dot_dot_dot_token,
            name,
            question_token,
            Some(parameter_type_node),
            None,
        );
        self.ctx.borrow_mut().approximate_length += parameter_symbol.name.len() + 3;
        parameter_node
    }
}

pub fn get_effective_parameter_declaration(symbol: &Arc<Symbol>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_effective_parameter_declaration"); 
    if let Some(parameter_declaration) =
        tsox_frontend::ast::mig::m3e_4::get_declaration_of_kind(symbol, SyntaxKind::Parameter)
    {
        return Some(parameter_declaration);
    }
    if !symbol.flags.contains(SymbolFlags::Transient) {
        return tsox_frontend::ast::mig::m3e_4::get_declaration_of_kind(symbol, SyntaxKind::JSDocParameterTag);
    }
    None
}
