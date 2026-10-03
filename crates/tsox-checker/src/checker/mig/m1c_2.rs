#![allow(unused_imports)]

#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
#[allow(unused_imports)]
use tsox_core::diagnostics::messages_generated::*;

pub(crate) use crate::checker::checker::*;
pub(crate) use crate::checker::mig::m1c::*;
#[allow(unused_imports)]
use crate::checker::mig::m1c_3::create_diagnostic_for_node_message;
#[allow(unused_imports)]
use tsox_frontend::scanner::mig::m3i::declaration_name_to_string;
#[allow(unused_imports)]
use tsox_frontend::ast::mig::x4ast::{get_containing_function};
#[allow(unused_imports)]
use tsox_frontend::ast::mig::m3e_4::{get_enclosing_block_scope_container};
#[allow(unused_imports)]
use tsox_frontend::ast::mig::m3g_2::is_part_of_parameter_declaration;
#[allow(unused_imports)]
use tsox_frontend::ast::mig::m3f_2::{node_initializer, node_parameters, node_type_parameters};
#[allow(unused_imports)]
use crate::checker::mig::r18k6_defs::PRIVATE_IDENTIFIERS_CANNOT_BE_USED_IN_DESTRUCTURING_PATTERNS;
#[allow(unused_imports)]
use crate::checker::mig::m1f_5::r24k10_defs::intrinsic_type_kinds;
use std::sync::Arc;

use crate::checker::types::*;
use tsox_frontend::ast::{Diagnostic, Node, NodeData, SyntaxKind};

impl Checker {
    pub fn check_type_parameter_lists_identical(&mut self, symbol: &Arc<Symbol>) { ::tsox_core::fntrace::enter("check_type_parameter_lists_identical"); 
        if symbol.declarations.len() == 1 {
            return;
        }
        let links = self.declared_type_links.get_or_default(symbol);
        if links.type_parameters_checked {
            return;
        }
        self.declared_type_links.get_or_default(symbol).type_parameters_checked = true;
        let declarations = self.get_class_or_interface_declarations_of_symbol(symbol);
        if declarations.len() <= 1 {
            return;
        }
        let t = self.get_declared_type_of_symbol(symbol);
        let local_type_parameters = match &t.data {
            TypeData::Interface(data) => data.local_type_parameters().to_vec(),
            _ => vec![],
        };
        if !self.are_type_parameters_identical(&declarations, &local_type_parameters, |d| {
            node_type_parameters(d)
                .map(|l| l.nodes.clone())
                .unwrap_or_default()
        }) {
            let name = self.symbol_to_string(symbol);
            for declaration in declarations.iter() {
                let decl_name = get_name_of_declaration(declaration).unwrap();
                self.error_message(
                    &decl_name,ALL_DECLARATIONS_OF_0_MUST_HAVE_IDENTICAL_TYPE_PARAMETERS,
                    &[name.clone()],
                );
            }
        }
    }

    pub fn check_type_for_duplicate_index_signatures(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_type_for_duplicate_index_signatures"); 
        let node_symbol = match self.get_symbol_of_declaration(node) {
            Some(s) => s,
            None => return,
        };
        let index_symbol = match self.get_index_symbol(&node_symbol) {
            Some(s) => s,
            None => return,
        };
        if index_symbol.declarations.len() <= 1 {
            return;
        }
        let mut index_signature_map: HashMap<TypeId, Vec<Arc<Node>>> = HashMap::new();
        for declaration in index_symbol.declarations.iter() {
            if is_index_signature_declaration(declaration) {
                if let Some(parameters) = node_parameters(declaration) {
                    if parameters.nodes.len() == 1 {
                        if let Some(param_type) = parameters.nodes[0].type_() {
                            let t = self.get_type_from_type_node(&param_type);
                            for dt in t.distributed() {
                                index_signature_map.entry(dt.id).or_default().push(Arc::clone(declaration));
                            }
                        }
                    }
                }
            }
        }
        for (_t, declarations) in index_signature_map {
            if declarations.len() > 1 {
                for declaration in declarations {
                    let name = get_name_of_declaration(&declaration).unwrap();
                    self.error_message(&name,DUPLICATE_IDENTIFIER_0, &[]);
                }
            }
        }
    }

    pub fn check_variable_statement(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_variable_statement"); 
        let declaration_list = match &node.data {
            NodeData::VariableStatement(data) => Arc::clone(&data.declaration_list),
            _ => return,
        };
        if !self.check_grammar_modifiers(node)
            && !self.check_grammar_variable_declaration_list(&declaration_list)
        {
            self.check_grammar_for_disallowed_block_scoped_variable_statement(node);
        }
        self.check_variable_declaration_list(&declaration_list);
    }

    pub fn check_variable_like_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_variable_like_declaration"); 
        self.check_decorators(node);
        let name = match tsox_frontend::ast::node_data_generated::node_name(node) {
            Some(n) => n,
            None => return,
        };
        let type_node = node_type(node).cloned();
        let initializer = node_initializer(node);
        if !is_binding_element(node) {
            if let Some(type_node) = &type_node {
                self.check_source_element(type_node);
            }
        }
        if tsox_frontend::ast::node_data_generated::is_computed_property_name(name) {
            self.check_computed_property_name(name);
            if let Some(initializer) = &initializer {
                self.check_expression_cached(initializer);
            }
        }
        if is_binding_element(node) {
            let prop_name = match &node.data {
                NodeData::BindingElement(d) => d.property_name.clone(),
                _ => None,
            };
            if let Some(prop_name) = &prop_name {
                if is_private_identifier(prop_name) {
                    self.grammar_error_on_node(
                        prop_name,
                        &PRIVATE_IDENTIFIERS_CANNOT_BE_USED_IN_DESTRUCTURING_PATTERNS,
                    );
                }
            }
            if prop_name.is_some()
                && is_identifier(name)
                && is_part_of_parameter_declaration(node)
                && get_containing_function(node)
                    .and_then(|f| f.body().cloned())
                    .map(|b| node_is_missing(Some(&b)))
                    .unwrap_or(false)
            {
                return;
            }
        }
        self.check_variable_like_declaration_not_binding_element_tail(node, name, type_node.as_ref(), initializer);
    }

    pub fn check_variable_like_declaration_not_binding_element_tail(
        &mut self,
        node: &Arc<Node>,
        name: &Arc<Node>,
        _type_node: Option<&Arc<Node>>,
        initializer: Option<&Arc<Node>>,
    ) { ::tsox_core::fntrace::enter("check_variable_like_declaration_not_binding_element_tail"); 
        if is_binding_pattern(name) {
            for element in tsox_frontend::ast::mig::m3b::elements(name) {
                self.check_source_element(element);
            }
        }
        if initializer.is_some()
            && is_part_of_parameter_declaration(node)
            && get_containing_function(node)
                .and_then(|f| f.body().cloned())
                .map(|b| node_is_missing(Some(&b)))
                .unwrap_or(false)
        {
            self.error_message(
                node,A_PARAMETER_INITIALIZER_IS_ONLY_ALLOWED_IN_A_FUNCTION_OR_CONSTRUCTOR_IMPLEMENTATION,
                &[],
            );
            return;
        }
        if is_binding_pattern(name) {
            if self.is_in_ambient_or_type_node(node) {
                return;
            }
            let need_check_initializer = initializer.is_some()
                && node
                    .parent()
                    .and_then(|p| p.parent())
                    .map(|p| p.kind != SyntaxKind::ForInStatement)
                    .unwrap_or(true);
            let need_check_widened_type = !tsox_frontend::ast::mig::m3b::elements(name)
                .iter()
                .any(|n| node_name(n).is_some());
            if need_check_initializer || need_check_widened_type {
                let widened_type = self
                    .get_widened_type_for_variable_like_declaration(node, false)
                    .unwrap_or_else(|| self.error_type());
                if let Some(initializer) = initializer
                    && need_check_initializer
                {
                    let initializer_type = self.check_expression_cached(initializer);
                    if self.strict_null_checks && need_check_widened_type {
                        self.check_non_null_non_void_type(&initializer_type, node);
                    } else {
                        let widened = self
                            .get_widened_type_for_variable_like_declaration(node, false)
                            .unwrap_or_else(|| self.error_type());
                        self.check_type_assignable_to_and_optionally_elaborate(
                            &initializer_type,
                            &widened,
                            Some(node),
                            Some(initializer),
                            None,
                            None,
                        );
                    }
                }
                if need_check_widened_type {
                    if is_array_binding_pattern(name) {
                        self.check_iterated_type_or_element_type(
                            crate::checker::checker_iteration::IterationUse::Destructuring,
                            &widened_type,
                            Some(node),
                        );
                    } else if self.strict_null_checks {
                        self.check_non_null_non_void_type(&widened_type, node);
                    }
                }
            }
            return;
        }
        let symbol = match self.get_symbol_of_declaration(node) {
            Some(s) => s,
            None => return,
        };
        if symbol.flags.contains(SymbolFlags::Alias)
            && is_variable_declaration_initialized_to_require(node)
        {
            self.check_alias_symbol(node);
            return;
        }
        if name.kind == SyntaxKind::BigIntLiteral {
            self.error_message(
                name,A_BIGINT_LITERAL_CANNOT_BE_USED_AS_A_PROPERTY_NAME,
                &[],
            );
        }
        let symbol_type = self.get_type_of_symbol(&symbol);
        let t = self.convert_auto_to_any(&symbol_type);
        if symbol
            .value_declaration
            .as_ref()
            .is_some_and(|vd| Arc::ptr_eq(vd, node))
        {
            if let Some(initializer) = initializer {
                let parent_is_not_for_in = node
                    .parent()
                    .and_then(|p| p.parent())
                    .map(|p| !is_for_in_statement(&p))
                    .unwrap_or(true);
                if parent_is_not_for_in {
                    let initializer_type = self.check_expression_cached(initializer);
                    self.check_type_assignable_to_and_optionally_elaborate(
                        &initializer_type,
                        &t,
                        Some(node),
                        Some(initializer),
                        None,
                        None,
                    );
                    let block_scope_kind = self.get_combined_node_flags_cached(node)
                        & (NodeFlags::Let
                            .union(NodeFlags::Const)
                            .union(NodeFlags::Using)
                            .union(NodeFlags::AwaitUsing));
                    if block_scope_kind == NodeFlags::AwaitUsing {
                        let global_async_disposable_type =
                            self.get_global_type("AsyncDisposable", 0, true);
                        let global_disposable_type = self.get_global_type("Disposable", 0, true);
                        let empty_object_type = self.empty_object_type();
                        if global_async_disposable_type.id != empty_object_type.id
                            && global_disposable_type.id != empty_object_type.id
                        {
                            let optional_disposable_type = self.get_union_type(vec![
                                global_async_disposable_type,
                                global_disposable_type,
                                self.null_type(),
                                self.undefined_type(),
                            ]);
                            let widened = self.widen_type_for_variable_like_declaration(
                                Some(Arc::clone(&initializer_type)),
                                node,
                                false,
                            );
                            self.check_type_assignable_to(
                                &widened,
                                &optional_disposable_type,
                                Some(initializer),
                                Some(&THE_INITIALIZER_OF_AN_AWAIT_USING_DECLARATION_MUST_BE_EITHER_AN_OBJECT_WITH_A_SYMBOL_ASYNCDISPOSE_OR_SYMBOL_DISPOSE_METHOD_OR_BE_NULL_OR_UNDEFINED),
                            );
                        }
                    } else if block_scope_kind == NodeFlags::Using {
                        let global_disposable_type = self.get_global_type("Disposable", 0, true);
                        let empty_object_type = self.empty_object_type();
                        if global_disposable_type.id != empty_object_type.id {
                            let optional_disposable_type = self.get_union_type(vec![
                                global_disposable_type,
                                self.null_type(),
                                self.undefined_type(),
                            ]);
                            let widened = self.widen_type_for_variable_like_declaration(
                                Some(Arc::clone(&initializer_type)),
                                node,
                                false,
                            );
                            self.check_type_assignable_to(
                                &widened,
                                &optional_disposable_type,
                                Some(initializer),
                                Some(&THE_INITIALIZER_OF_A_USING_DECLARATION_MUST_BE_EITHER_AN_OBJECT_WITH_A_SYMBOL_DISPOSE_METHOD_OR_BE_NULL_OR_UNDEFINED),
                            );
                        }
                    }
                }
            }
            if symbol.declarations.len() > 1
                && symbol.declarations.iter().any(|d| {
                    !Arc::ptr_eq(d, node)
                        && tsox_frontend::ast::mig::m3g_3::is_variable_like(d)
                        && !self.are_declaration_flags_identical(d, node)
                })
            {
                self.error_message(
                    name,ALL_DECLARATIONS_OF_0_MUST_HAVE_IDENTICAL_MODIFIERS,
                    &[declaration_name_to_string(Some(name))],
                );
            }
        } else {
            let declaration_type = self
                .get_widened_type_for_variable_like_declaration(node, false)
                .unwrap_or_else(|| self.error_type());
            let declaration_type = self.convert_auto_to_any(&declaration_type);
            if !self.is_error_type(&t)
                && !self.is_error_type(&declaration_type)
                && !self.is_type_identical_to(&t, &declaration_type)
                && !symbol.flags.contains(SymbolFlags::Assignment)
            {
                self.error_next_variable_or_property_declaration_must_have_same_type(
                    symbol.value_declaration.as_ref(),
                    &t,
                    node,
                    &declaration_type,
                );
            }
            if let Some(initializer) = initializer {
                let initializer_type = self.check_expression_cached(initializer);
                self.check_type_assignable_to_and_optionally_elaborate(
                    &initializer_type,
                    &declaration_type,
                    Some(node),
                    Some(initializer),
                    None,
                    None,
                );
            }
            if let Some(value_declaration) = &symbol.value_declaration
                && !self.are_declaration_flags_identical(node, value_declaration)
            {
                self.error_message(
                    name,ALL_DECLARATIONS_OF_0_MUST_HAVE_IDENTICAL_MODIFIERS,
                    &[declaration_name_to_string(Some(name))],
                );
            }
        }
        if !is_property_declaration(node) && !is_property_signature_declaration(node) {
            self.check_exports_on_merged_declarations(node);
            if is_variable_declaration(node) || is_binding_element(node) {
                self.check_var_declared_names_not_shadowed(node);
            }
            self.check_collisions_for_declaration_name(node, Some(name));
        }
    }

    pub fn error_next_variable_or_property_declaration_must_have_same_type(
        &mut self,
        first_declaration: Option<&Arc<Node>>,
        first_type: &Arc<Type>,
        next_declaration: &Arc<Node>,
        next_type: &Arc<Type>,
    ) { ::tsox_core::fntrace::enter("error_next_variable_or_property_declaration_must_have_same_type"); 
        let next_declaration_name = get_name_of_declaration(next_declaration).unwrap();
        let message = if is_property_declaration(next_declaration)
            || is_property_signature_declaration(next_declaration)
        {
            SUBSEQUENT_PROPERTY_DECLARATIONS_MUST_HAVE_THE_SAME_TYPE_PROPERTY_0_MUST_BE_OF_TYPE_1_BUT_HERE_HAS_TYPE_2
        } else {
            SUBSEQUENT_VARIABLE_DECLARATIONS_MUST_HAVE_THE_SAME_TYPE_VARIABLE_0_MUST_BE_OF_TYPE_1_BUT_HERE_HAS_TYPE_2
        };
        let decl_name = declaration_name_to_string(Some(&next_declaration_name));
        let first_str = self.type_to_string(first_type);
        let next_str = self.type_to_string(next_type);
        let err = self.error_message(&next_declaration_name, message, &[decl_name.clone(), first_str, next_str]);
        if let Some(first_declaration) = first_declaration {
            if let Some(mut err) = err {
                let related =
                    create_diagnostic_for_node_message(first_declaration,X_0_WAS_ALSO_DECLARED_HERE, &[decl_name]);
                Arc::make_mut(&mut err).related_information.push((*related).clone());
            }
        }
    }

    pub fn check_var_declared_names_not_shadowed(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_var_declared_names_not_shadowed"); 
        let name = match tsox_frontend::ast::node_data_generated::node_name(node) {
            Some(n) => n,
            None => return,
        };
        if !is_identifier(name) {
            return;
        }
        let text = node_text(name);
        if text != "await" && text != "yield" {
            return;
        }
        let enclosing_block_scope = get_enclosing_block_scope_container(node);
        let local_symbol = self.get_symbol_of_declaration(node);
    }

    pub fn check_type_alias_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_type_alias_declaration"); 
        self.check_grammar_modifiers(node);
        let name = tsox_frontend::ast::node_data_generated::node_name(node).unwrap();
        self.check_type_name_is_reserved(name, TYPE_ALIAS_NAME_CANNOT_BE_0);
        if let Some(parent) = node.parent() {
            if !self.container_allows_block_scoped_variable(&parent) {
                self.grammar_error_on_node_with_args(
                    node,
                    &X_0_DECLARATIONS_CAN_ONLY_BE_DECLARED_INSIDE_A_BLOCK,
                    &["type".to_string()],
                );
            }
        }
        self.check_exports_on_merged_declarations(node);
        let type_node = node_type(node).cloned();
        let type_parameters = node_type_parameters(node);
        self.check_type_parameters(type_parameters.map(|l| &**l));
        if let Some(type_node) = &type_node {
            if type_node.kind == SyntaxKind::IntrinsicKeyword {
                let type_parameter_count = type_parameters.map(|l| l.nodes.len()).unwrap_or(0);
                let is_builtin_iterator_return =
                    type_parameter_count == 0 && node_text(name) == "BuiltinIteratorReturn";
                let is_known_intrinsic =
                    type_parameter_count == 1 && intrinsic_type_kinds(node_text(name)).is_some();
                if !(is_builtin_iterator_return || is_known_intrinsic) {
                    self.error_message(
                        type_node,THE_INTRINSIC_KEYWORD_CAN_ONLY_BE_USED_TO_DECLARE_COMPILER_PROVIDED_INTRINSIC_TYPES,
                        &[],
                    );
                }
                return;
            }
            self.check_source_element(type_node);
        }
        self.register_for_unused_identifiers_check(node);
    }

    pub fn check_type_name_is_reserved(&mut self, name: &Arc<Node>, message: tsox_core::diagnostics::Message) { ::tsox_core::fntrace::enter("check_type_name_is_reserved"); 
        let text = node_text(name);
        match text {
            "any" | "unknown" | "never" | "number" | "bigint" | "boolean" | "string" | "symbol"
            | "void" | "object" | "undefined" => {
                self.error_message(name, message, &[text.to_string()]);
            }
            _ => {}
        }
    }

    pub fn check_weak_map_set_collision(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_weak_map_set_collision"); 
        let Some(enclosing_block_scope) = get_enclosing_block_scope_container(node) else {
            return;
        };
        let flags = self.node_links.get_or_default(&enclosing_block_scope).flags;
        if flags.contains(NodeCheckFlags::ContainsClassWithPrivateIdentifiers) {
            if let Some(name) = tsox_frontend::ast::node_data_generated::node_name(node) {
                if is_identifier(name) {
                    self.error_skipped_on_no_emit_message(
                        node,COMPILER_RESERVES_NAME_0_WHEN_EMITTING_PRIVATE_IDENTIFIER_DOWNLEVEL,
                        &[node_text(name).to_string()],
                    );
                }
            }
        }
    }

    pub fn check_type_of_expression(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("check_type_of_expression"); 
        let expression = node_expression(node).unwrap();
        self.check_expression(expression);
        self.typeof_type.get().cloned().unwrap()
    }

    pub fn check_void_expression(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("check_void_expression"); 
        self.check_node_deferred(node);
        self.undefined_widening_type.clone()
    }
}

fn is_variable_declaration_initialized_to_require(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_variable_declaration_initialized_to_require"); 
    let node = if is_binding_element(node) {
        match node.parent().and_then(|p| p.parent()) {
            Some(gp) => gp,
            None => return false,
        }
    } else {
        Arc::clone(node)
    };
    if !crate::checker::mig::m1e::r20k2_defs::is_in_js_file(&node) {
        return false;
    }
    let NodeData::VariableDeclaration(d) = &node.data else {
        return false;
    };
    let Some(init) = &d.initializer else {
        return false;
    };
    match node.parent().and_then(|p| p.parent()) {
        Some(gp) if !gp.syntactic_modifier_flags().contains(ModifierFlags::Export) => {
            tsox_frontend::ast::dynamic_imports::is_require_call(init, true)
        }
        _ => false,
    }
}
