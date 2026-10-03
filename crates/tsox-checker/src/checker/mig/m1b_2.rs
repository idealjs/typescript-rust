#![allow(unused_imports)]

use crate::checker::checker::*;
use std::sync::Arc;
use tsox_frontend::ast::NodeData;
use tsox_frontend::ast::NodeFlags;
use tsox_frontend::ast::SyntaxKind;
use tsox_frontend::ast::SymbolFlags;
use tsox_frontend::ast::{is_array_binding_pattern, is_binding_pattern, is_identifier, is_in_js_file, is_object_binding_pattern, is_string_literal, is_string_literal_like};
use tsox_frontend::scanner::mig::m3i::declaration_name_to_string;
use crate::checker::utilities_has_only_expression_initialization::get_declarations_of_kind;
use crate::checker::mig::m2d_3::get_flow_node_of_node;
use tsox_frontend::ast::mig::m3e::get_function_flags;
use crate::checker::mig::m1g_3::has_type_json_import_attribute;
use crate::checker::utilities_get_assignment_target::is_in_type_query;
use tsox_frontend::ast::is_object_literal_expression;
use crate::checker::utilities_is_private_within_ambient::is_private_within_ambient;
use crate::checker::mig::m2a::is_rest_parameter;
use crate::checker::mig::m3a_2::new_diagnostic_for_node;
use crate::checker::mig::m2c_3::should_mark_identifier_alias_referenced;
use tsox_frontend::ast::skip_parentheses;
use tsox_core::diagnostics::messages_generated::*;
use tsox_frontend::ast::mig::m3e::FunctionFlags;
use crate::checker::mig::wc3::ReferenceHint;
use super::m1b::{PredicateSemantics, find_ancestor_node, import_attributes_list, import_attribute_value, object_literal_properties, variable_declaration_exclamation_token, function_like_data_full_signature, InternalSymbolName, get_external_module_name};
use tsox_core::core::compiler_options_kinds::ResolutionMode;
use tsox_frontend::ast::mig::m3e_4::{get_import_attributes, get_first_identifier};
use tsox_frontend::ast::mig::x6a::is_exclusively_type_only_import_or_export;
use tsox_frontend::ast::mig::m3b::{elements as node_elements, import_clause, is_type_only as node_is_type_only, parameters as node_parameters, property_name_or_name, module_specifier as node_module_specifier};
use tsox_frontend::ast::mig::m3g_2::is_object_literal_or_class_expression_method_or_accessor;
use tsox_frontend::ast::{FlowFlags, FlowNode};
use tsox_frontend::ast::mig::m3g_3::module_export_name_is_default;
use crate::checker::types_type_id::TYPE_FLAGS_NULLABLE;
use crate::checker::mig::wc3_3::is_internal_module_import_equals_declaration;
use crate::checker::services_checker_7::IndexKind;
use super::m1b::InheritanceInfo;
use super::m1b::r20k1_ext::symbol_option_ptr_eq;
use tsox_frontend::ast::mig::m3d_2::new_diagnostic_chain;

impl Checker {
    pub fn check_function_or_method_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_function_or_method_declaration"); 
        self.check_decorators(node);
        self.check_signature_declaration(node);
        let function_flags = get_function_flags(Some(node));
        let name = node.name();
        if name.is_some() {
            let name = name.unwrap();
            if tsox_frontend::ast::is_computed_property_name(&name) {
                self.check_computed_property_name(&name);
            }
        }
        if self.has_bindable_name(node) {
            let symbol = self.get_symbol_of_declaration(node);
            let local_symbol = self.program.symbol_map().symbol_of(node).cloned().unwrap_or_else(|| symbol.clone().expect("bindable name symbol"));
            if !node.flags.contains(NodeFlags::JavaScriptFile) {
                self.check_function_or_constructor_symbol(&local_symbol);
            }
            if symbol.as_ref().and_then(|s| s.parent()).is_some() {
                if let Some(sym) = symbol.as_ref() {
                    self.check_function_or_constructor_symbol(sym);
                }
            }
        }
        let body = node.body();
        if let Some(body) = &body {
            self.check_source_element(body);
        }
        let return_type = self.get_return_type_from_annotation(node);
        self.check_all_code_paths_in_non_void_function_return_or_throw(node, Some(&return_type));
        if let Some(full_signature) = function_like_data_full_signature(node) {
            let signature_type = self.get_type_from_type_node(&full_signature);
            if self.get_contextual_call_signature(&signature_type, node).is_none() {
                self.error_message(&full_signature,A_JSDOC_TYPE_TAG_ON_A_FUNCTION_MUST_HAVE_A_SIGNATURE_WITH_THE_CORRECT_NUMBER_OF_ARGUMENTS, &[]);
            }
        }
        if node.type_node().is_none() {
            let body_missing = tsox_frontend::ast::node_is_missing(body);
            if body_missing && !is_private_within_ambient(node) {
                let any_type = self.any_type();
                self.report_implicit_any(node, &any_type, WideningKind::Normal);
            }
            if function_flags.contains(FunctionFlags::GENERATOR) && tsox_frontend::ast::node_is_present(body) {
                if let Some(signature) = self.get_signature_from_declaration(node) {
                    self.get_return_type_of_signature(&signature);
                }
            }
        }
    }

    pub fn check_generator_instantiation_assignability_to_return_type(&mut self, return_type: &Arc<Type>, function_flags: FunctionFlags, error_node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("check_generator_instantiation_assignability_to_return_type"); 
        let is_async = function_flags.contains(FunctionFlags::ASYNC);
        let generator_yield_type = self
            .get_iteration_type_of_generator_function_return_type(IterationTypeKind::YIELD, return_type, is_async)
            .unwrap_or_else(|| self.any_type());
        let generator_return_type = self
            .get_iteration_type_of_generator_function_return_type(IterationTypeKind::RETURN, return_type, is_async)
            .unwrap_or_else(|| Arc::clone(&generator_yield_type));
        let generator_next_type = self
            .get_iteration_type_of_generator_function_return_type(IterationTypeKind::NEXT, return_type, is_async)
            .unwrap_or_else(|| self.unknown_type());
        let generator_instantiation = self.create_generator_type(&generator_yield_type, &generator_return_type, &generator_next_type, is_async);
        self.check_type_assignable_to(&generator_instantiation, return_type, Some(error_node), None)
    }

    pub fn check_identifier(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> { ::tsox_core::fntrace::enter("check_identifier");
        if tsox_frontend::ast::is_this_in_type_query(node) {
            return self.check_this_expression(node);
        }
        let symbol = match self.get_resolved_symbol(node) {
            Some(symbol) if !symbol_option_ptr_eq(&Some(Arc::clone(&symbol)), &self.unknown_symbol) => symbol,
            // Go checkIdentifier 的符号由 resolveName 即时解析并驻留
            // nodeLinks；本仓普通读引用的类型走 get_type_of_identifier
            //（不经 symbol_node_links），本函数被直接调用时（如 extends
            // 基类值位求值）链接为空，按 Go resolveName 语义节点锚定
            // 按需解析（与当前检查到哪个文件无关）并回填链接
            _ => match self.get_resolved_symbol_on_demand(node) {
                Some(symbol) if !symbol_option_ptr_eq(&Some(Arc::clone(&symbol)), &self.unknown_symbol) => symbol,
                _ => return self.error_type(),
            },
        };
        if symbol_option_ptr_eq(&Some(Arc::clone(&symbol)), &self.arguments_symbol) {
            if super::m1b::r25k1_defs::is_in_property_initializer_or_class_static_block_ex(node, true) {
                self.error_message(node,X_ARGUMENTS_CANNOT_BE_REFERENCED_IN_PROPERTY_INITIALIZERS_OR_CLASS_STATIC_INITIALIZATION_BLOCKS, &[]);
                return self.error_type();
            }
            return self.get_type_of_symbol(&symbol);
        }
        if should_mark_identifier_alias_referenced(node) {
            self.mark_linked_references(node, ReferenceHint::Identifier, None, None);
        }
        let local_or_export_symbol = self.get_export_symbol_of_value_symbol_if_exported(&symbol);
        let target_symbol = self.resolve_alias_with_deprecation_check(&local_or_export_symbol, node);
        if !target_symbol.declarations.is_empty() && self.is_deprecated_symbol(&target_symbol) && self.is_uncalled_function_reference(node, &target_symbol) {
            let declarations = target_symbol.declarations.clone();
            let text = node.text();
            self.add_deprecated_suggestion(node, &declarations, text);
        }
        let mut declaration = local_or_export_symbol.value_declaration.clone();
        let immediate_declaration = declaration.clone();
        if let Some(decl) = &declaration {
            if decl.kind == SyntaxKind::BindingElement
                && super::m1b::r23k1_defs::contextual_binding_patterns_contains_declaration(decl)
                && find_ancestor_node(node, |parent| decl.parent().map(|dp| Arc::ptr_eq(&dp, parent)).unwrap_or(false)).is_some()
            {
                return self.non_inferrable_any_type();
            }
        }
        let mut t = self.get_narrowed_type_of_symbol(&local_or_export_symbol, None, Some(node));
        let assignment_kind = get_assignment_target_kind(node);
        if assignment_kind != AssignmentKind::None {
            if !local_or_export_symbol.flags.contains(SymbolFlags::VARIABLE)
                && !(is_in_js_file(node) && local_or_export_symbol.flags.contains(SymbolFlags::ValueModule))
            {
                let assignment_error = if local_or_export_symbol.flags.contains(SymbolFlags::ENUM) {
                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_AN_ENUM
                } else if local_or_export_symbol.flags.contains(SymbolFlags::Class) {
                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_CLASS
                } else if local_or_export_symbol.flags.contains(SymbolFlags::MODULE) {
                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_NAMESPACE
                } else if local_or_export_symbol.flags.contains(SymbolFlags::Function) {
                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_FUNCTION
                } else if local_or_export_symbol.flags.contains(SymbolFlags::Alias) {
                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_AN_IMPORT
                } else {
                    CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_NOT_A_VARIABLE
                };
                let name = self.symbol_to_string(&symbol);
                self.error_message(node, assignment_error, &[name]);
                return self.error_type();
            }
            if self.is_readonly_symbol(&local_or_export_symbol) {
                let name = self.symbol_to_string(&symbol);
                if local_or_export_symbol.flags.contains(SymbolFlags::VARIABLE) {
                    self.error_message(node,CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_CONSTANT, &[name]);
                } else {
                    self.error_message(node,CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY, &[name]);
                }
                return self.error_type();
            }
        }
        let is_alias = local_or_export_symbol.flags.contains(SymbolFlags::Alias);
        if local_or_export_symbol.flags.contains(SymbolFlags::VARIABLE) {
            if assignment_kind == AssignmentKind::Definite {
                if is_in_compound_like_assignment(node) {
                    return self.get_base_type_of_literal_type(&t);
                }
                return t;
            }
        } else if is_alias {
            declaration = self.get_declaration_of_alias_symbol(&symbol);
        } else {
            return t;
        }
        let Some(declaration) = declaration else {
            return t;
        };
        t = self.get_narrowable_type_for_reference(&t, node);
        let is_parameter = tsox_frontend::ast::get_root_declaration(&declaration).kind == SyntaxKind::Parameter;
        let declaration_container = self.get_control_flow_container(&declaration);
        let mut flow_container = self.get_control_flow_container(node);
        let is_outer_variable = !Arc::ptr_eq(&flow_container, &declaration_container);
        let is_spread_destructuring_assignment_target = node
            .parent()
            .map(|parent| {
                parent
                    .parent()
                    .map(|grandparent| tsox_frontend::ast::is_spread_assignment(&parent) && self.is_destructuring_assignment_target(&grandparent))
                    .unwrap_or(false)
            })
            .unwrap_or(false);
        let is_module_exports = symbol.flags.contains(SymbolFlags::ModuleExports);
        let type_is_automatic = Arc::ptr_eq(&t, &self.auto_type()) || Arc::ptr_eq(&t, &self.auto_array_type());
        let is_automatic_type_in_non_null =
            type_is_automatic && node.parent().map(|p| p.kind == SyntaxKind::NonNullExpression).unwrap_or(false);
        while !Arc::ptr_eq(&flow_container, &declaration_container)
            && (tsox_frontend::ast::is_function_expression_or_arrow_function(&flow_container)
                || is_object_literal_or_class_expression_method_or_accessor(&flow_container))
            && (self.is_constant_variable(&local_or_export_symbol) && !Arc::ptr_eq(&t, &self.auto_array_type())
                || self.is_parameter_or_mutable_local_variable(&local_or_export_symbol) && self.is_past_last_assignment(&local_or_export_symbol, Some(node)))
        {
            flow_container = self.get_control_flow_container(&flow_container);
        }
        let is_never_initialized = immediate_declaration
            .as_ref()
            .map(|decl| {
                tsox_frontend::ast::is_variable_declaration(decl)
                    && !decl
                        .parent()
                        .and_then(|p| p.parent())
                        .map(|gp| tsox_frontend::ast::is_for_in_or_of_statement(&gp))
                        .unwrap_or(false)
                    && decl.initializer().is_none()
                    && variable_declaration_exclamation_token(decl).is_none()
                    && self.is_mutable_local_variable_declaration(decl)
                    && !self.is_symbol_assigned_definitely(&symbol)
            })
            .unwrap_or(false);
        let assume_initialized = is_parameter
            || is_alias
            || (is_outer_variable && !is_never_initialized)
            || is_spread_destructuring_assignment_target
            || is_module_exports
            || self.is_same_scoped_binding_element(node, &declaration)
            || (!Arc::ptr_eq(&t, &self.auto_type())
                && !Arc::ptr_eq(&t, &self.auto_array_type())
                && (!self.strict_null_checks
                    || t.flags.intersects(TypeFlags::ANY_OR_UNKNOWN | TypeFlags::Void)
                    || is_in_type_query(node)
                    || self.is_in_ambient_or_type_node(node)
                    || node.parent().map(|p| p.kind == SyntaxKind::ExportSpecifier).unwrap_or(false)))
            || node.parent().map(|p| tsox_frontend::ast::is_non_null_expression(&p)).unwrap_or(false)
            || (tsox_frontend::ast::is_variable_declaration(&declaration) && variable_declaration_exclamation_token(&declaration).is_some())
            || declaration.flags.contains(NodeFlags::Ambient);
        let initial_type;
        if is_automatic_type_in_non_null {
            initial_type = Some(self.undefined_type());
        } else if assume_initialized && is_parameter {
            initial_type = Some(self.remove_optionality_from_declared_type(&t, &declaration));
        } else if assume_initialized {
            initial_type = Some(Arc::clone(&t));
        } else if type_is_automatic {
            initial_type = Some(self.undefined_type());
        } else {
            initial_type = Some(self.get_optional_type(Arc::clone(&t)));
        }
        let initial_type = initial_type.unwrap();
        let flow_type;
        if is_automatic_type_in_non_null {
            let raw = self.get_flow_type_of_reference_ex(node, &t, Some(&initial_type), Some(&flow_container));
            flow_type = self.get_non_nullable_type(&raw);
        } else {
            flow_type = self.get_flow_type_of_reference_ex(node, &t, Some(&initial_type), Some(&flow_container));
        }
        if !self.is_evolving_array_operation_target(node) && (Arc::ptr_eq(&t, &self.auto_type()) || Arc::ptr_eq(&t, &self.auto_array_type())) {
            if Arc::ptr_eq(&flow_type, &self.auto_type()) || Arc::ptr_eq(&flow_type, &self.auto_array_type()) {
                if self.no_implicit_any {
                    let name = self.symbol_to_string(&symbol);
                    let flow_string = self.type_to_string(&flow_type);
                    let name_of_declaration = tsox_frontend::ast::get_name_of_declaration(&declaration).expect("declaration name");
                    self.error_message(&name_of_declaration,VARIABLE_0_IMPLICITLY_HAS_TYPE_1_IN_SOME_LOCATIONS_WHERE_ITS_TYPE_CANNOT_BE_DETERMINED, &[name.clone(), flow_string.clone()]);
                    self.error_message(node,VARIABLE_0_IMPLICITLY_HAS_AN_1_TYPE, &[name, flow_string]);
                }
                return self.convert_auto_to_any(&flow_type);
            }
        } else if !assume_initialized && !self.contains_undefined_type(&t) && self.contains_undefined_type(&flow_type) {
            let name = self.symbol_to_string(&symbol);
            self.error_message(node,VARIABLE_0_IS_USED_BEFORE_BEING_ASSIGNED, &[name]);
            return t;
        }
        if assignment_kind != AssignmentKind::None {
            return self.get_base_type_of_literal_type(&flow_type);
        }
        flow_type
    }

    pub fn check_if_expression_refines_any_parameter(&mut self, function_node: &Arc<Node>, expr: &Arc<Node>) -> Option<TypePredicate> { ::tsox_core::fntrace::enter("check_if_expression_refines_any_parameter"); 
        let expr = skip_parentheses(expr);
        let return_type = self.check_expression_cached(&expr);
        if !return_type.flags.contains(TypeFlags::Boolean) {
            return None;
        }
        for (i, param) in node_parameters(function_node).iter().enumerate() {
            let Some(param_symbol) = self.program.symbol_map().symbol_of(param).cloned() else {
                continue;
            };
            let init_type = self.get_type_of_symbol(&param_symbol);
            let unusable = init_type.flags.contains(TypeFlags::Boolean)
                || !param.name().is_some_and(|n| is_identifier(n))
                || self.is_symbol_assigned(&param_symbol)
                || is_rest_parameter(param);
            if unusable {
                continue;
            }
            if let Some(true_type) = self.check_if_expression_refines_parameter(function_node, &expr, param, &init_type) {
                let name = param.name().map(|n| n.text()).unwrap_or_default();
                return Some(*self.new_type_predicate(TypePredicateKind::Identifier, name.to_string(), i as i32, true_type));
            }
        }
        None
    }

    pub fn check_if_expression_refines_parameter(&mut self, function_node: &Arc<Node>, expr: &Arc<Node>, param: &Arc<Node>, init_type: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("check_if_expression_refines_parameter"); 
        let mut antecedent = get_flow_node_of_node(expr);
        if antecedent.is_none() && expr.parent().map(|p| p.kind == SyntaxKind::ReturnStatement).unwrap_or(false) {
            let parent = expr.parent().unwrap();
            antecedent = get_flow_node_of_node(&parent);
        }
        let antecedent = antecedent.unwrap_or_else(super::m1b::r24k1_defs::new_start_flow_node);
        let true_condition = super::m1b::r24k1_defs::new_condition_flow_node(FlowFlags::TRUE_CONDITION, expr, &antecedent);
        let param_name = param.name().cloned();
        let name_node = param_name.as_ref().unwrap_or(param);
        let true_type = self.get_flow_type_of_reference_ex(name_node, init_type, Some(init_type), Some(function_node));
        if Arc::ptr_eq(&true_type, init_type) {
            return None;
        }
        let false_condition = super::m1b::r24k1_defs::new_condition_flow_node(FlowFlags::FALSE_CONDITION, expr, &antecedent);
        let false_raw = self.get_flow_type_of_reference_ex(name_node, init_type, Some(&true_type), Some(function_node));
        let false_subtype = self.get_reduced_type(&false_raw);
        if false_subtype.flags.contains(TypeFlags::Never) {
            return Some(true_type);
        }
        None
    }

    pub fn check_if_type_predicate_variable_is_declared_in_binding_pattern(&mut self, pattern: &Arc<Node>, predicate_variable_node: &Arc<Node>, predicate_variable_name: &str) -> bool { ::tsox_core::fntrace::enter("check_if_type_predicate_variable_is_declared_in_binding_pattern"); 
        for element in node_elements(pattern) {
            let Some(name) = element.name() else {
                continue;
            };
            if is_identifier(&name) && name.text() == predicate_variable_name {
                let name_text = name.text();
                self.error_message(predicate_variable_node,A_TYPE_PREDICATE_CANNOT_REFERENCE_ELEMENT_0_IN_A_BINDING_PATTERN, &[name_text.to_string()]);
                return true;
            }
            if is_array_binding_pattern(&name) || is_object_binding_pattern(&name) {
                if self.check_if_type_predicate_variable_is_declared_in_binding_pattern(&name, predicate_variable_node, predicate_variable_name) {
                    return true;
                }
            }
        }
        false
    }

    pub fn check_import_attributes(&mut self, declaration: &Arc<Node>) { ::tsox_core::fntrace::enter("check_import_attributes"); 
        let Some(node) = get_import_attributes(declaration) else {
            return;
        };
        let import_attributes_type = self.get_global_import_attributes_type_checked();
        if !Arc::ptr_eq(&import_attributes_type, &self.empty_object_type()) {
            let nullable = self.get_nullable_type(&import_attributes_type, TypeFlags::Undefined);
            let attributes_type = self.get_type_from_import_attributes(Some(&node));
            self.check_type_assignable_to(attributes_type.as_ref().expect("import attributes type"), &nullable, Some(&node), None);
        }
        let is_type_only = is_exclusively_type_only_import_or_export(declaration)
            || tsox_frontend::ast::is_import_type_node(declaration);
        let override_mode = self.get_resolution_mode_override(&node, is_type_only);
        if is_type_only {
            return;
        }
        if !self.module_kind.supports_import_attributes() {
            self.grammar_error_on_node(&node, &IMPORT_ATTRIBUTES_ARE_ONLY_SUPPORTED_WHEN_THE_MODULE_OPTION_IS_SET_TO_ESNEXT_NODE18_NODE20_NODENEXT_OR_PRESERVE);
            return;
        }
        if let Some(module_specifier) = get_external_module_name(declaration) {
            if self.get_emit_syntax_for_module_specifier_expression(&module_specifier) == ModuleKind::CommonJS {
                self.grammar_error_on_node(&node, &IMPORT_ATTRIBUTES_ARE_NOT_ALLOWED_ON_STATEMENTS_THAT_COMPILE_TO_COMMONJS_REQUIRE_CALLS);
                return;
            }
        }
        if override_mode.is_some() {
            self.grammar_error_on_node(&node, &X_RESOLUTION_MODE_CAN_ONLY_BE_SET_FOR_TYPE_ONLY_IMPORTS);
        }
    }

    pub fn check_import_attributes_expression(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("check_import_attributes_expression"); 
        if self.type_node_links.get(node).and_then(|l| l.resolved_type.clone()).is_none() {
            let symbol = self.new_symbol(SymbolFlags::ObjectLiteral, InternalSymbolName::ImportAttributes);
            let mut members = SymbolTable::new();
            let attributes = import_attributes_list(node);
            for attribute in attributes.iter() {
                let member = self.new_symbol(SymbolFlags::Property, attribute.name().expect("attribute name").text());
                let value = import_attribute_value(attribute);
                let value_type = self.check_expression_cached(&value);
                let regular = self.get_regular_type_of_literal_type(&value_type);
                self.value_symbol_links.get_or_default(&member).resolved_type = Some(regular);
                members.insert(member.name.clone(), member);
            }
            let mut t = self.new_anonymous_type(&symbol, members, vec![], vec![], vec![]);
            if let Some(t_mut) = Arc::get_mut(&mut t) {
                t_mut.object_flags |= ObjectFlags::ObjectLiteral | ObjectFlags::NonInferrableType;
            }
            self.type_node_links.get_or_default(node).resolved_type = Some(Arc::clone(&t));
            return t;
        }
        self.type_node_links.get(node).and_then(|l| l.resolved_type.clone()).unwrap()
    }

    pub fn check_import_attributes_type(&mut self, attributes: &Arc<Node>) { ::tsox_core::fntrace::enter("check_import_attributes_type"); 
        self.check_grammar_import_attributes_type(attributes);
        self.check_source_element(attributes);
        let import_attributes_type = self.get_global_import_attributes_type_checked();
        let module_attributes_type = self.get_type_of_module_declaration_import_attributes(Some(attributes));
        if !Arc::ptr_eq(&import_attributes_type, &self.empty_object_type()) {
            self.check_type_assignable_to(&module_attributes_type, &import_attributes_type, Some(attributes), None);
        }
    }

    pub fn check_import_binding(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_import_binding"); 
        let name = node.name();
        self.check_collisions_for_declaration_name(node, name);
        self.check_alias_symbol(node);
        if tsox_frontend::ast::is_import_specifier(node) {
            let property_name = node.property_name();
            self.check_module_export_name(property_name.expect("import specifier property name"), true);
            if module_export_name_is_default(property_name_or_name(node).unwrap_or(node)) {
                if self.emit_module_format_of_node_source_file(node) == ModuleKind::CommonJS {
                    self.check_external_emit_helpers(node, ExternalEmitHelpers::ImportDefault.bits());
                }
            }
        }
    }

    pub fn check_import_call_expression(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("check_import_call_expression"); 
        self.check_grammar_import_call_expression(node);
        let args: Vec<Arc<Node>> = node.arguments().map(|list| list.nodes.clone()).unwrap_or_default();
        if args.is_empty() {
            let any_type = self.any_type();
            return self.create_promise_return_type(node, &any_type);
        }
        let specifier = Arc::clone(&args[0]);
        let specifier_type = self.check_expression_cached(&specifier);
        let mut options_type: Option<Arc<Type>> = None;
        if args.len() > 1 {
            options_type = Some(self.check_expression_cached(&args[1]));
        }
        for arg in args.iter().skip(2) {
            self.check_expression_cached(arg);
        }
        if specifier_type.flags.contains(TYPE_FLAGS_NULLABLE) || !self.is_type_assignable_to(&specifier_type, &self.string_type()) {
            let type_string = self.type_to_string(&specifier_type);
            self.error_message(&specifier,DYNAMIC_IMPORT_S_SPECIFIER_MUST_BE_OF_TYPE_STRING_BUT_HERE_HAS_TYPE_0, &[type_string]);
        }
        let mut import_attributes_type: Option<Arc<Type>> = None;
        if let Some(options_type) = &options_type {
            let import_call_options_type = self.get_global_import_call_options_type_checked();
            if !Arc::ptr_eq(&import_call_options_type, &self.empty_object_type()) {
                let nullable = self.get_nullable_type(&import_call_options_type, TypeFlags::Undefined);
                self.check_type_assignable_to(options_type, &nullable, Some(&args[1]), None);
            }
            if is_object_literal_expression(&args[1]) {
                for prop in object_literal_properties(&args[1]).iter() {
                    let prop_name = prop.name();
                    if tsox_frontend::ast::is_property_assignment(prop)
                        && prop_name.as_ref().is_some_and(|n| is_identifier(n) && n.text() == "assert")
                    {
                        let name = prop_name.unwrap();
                        self.error_message(&name,IMPORT_ASSERTIONS_HAVE_BEEN_REPLACED_BY_IMPORT_ATTRIBUTES_USE_WITH_INSTEAD_OF_ASSERT, &[]);
                        break;
                    }
                }
            }
            import_attributes_type = self.get_type_of_property_of_type(options_type, "with");
        }
        let module_symbol = self.resolve_external_module_name_worker(node, Some(&specifier), None, false, false, import_attributes_type.as_ref());
        if let Some(module_symbol) = module_symbol {
            let es_module_symbol = self.resolve_external_module_symbol(&module_symbol, true);
            let module_type = self.get_type_of_symbol(&es_module_symbol);
            let synthetic_type = self.get_type_with_synthetic_default_only(Some(&module_type), &es_module_symbol, &module_symbol, &specifier, import_attributes_type.as_ref());
            let synthetic_type = match synthetic_type {
                Some(t) => t,
                None => self.get_type_with_synthetic_default_import_type(&module_type, &es_module_symbol, &module_symbol, Some(&specifier)),
            };
            return self.create_promise_return_type(node, &synthetic_type);
        }
        let any_type = self.any_type();
        self.create_promise_return_type(node, &any_type)
    }

    pub fn check_import_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_import_declaration"); 
        let diagnostic = if is_in_js_file(node) {
            AN_IMPORT_DECLARATION_CAN_ONLY_BE_USED_AT_THE_TOP_LEVEL_OF_A_MODULE
        } else {
            AN_IMPORT_DECLARATION_CAN_ONLY_BE_USED_AT_THE_TOP_LEVEL_OF_A_NAMESPACE_OR_MODULE
        };
        if self.check_grammar_module_element_context(node, &diagnostic) {
            self.check_external_module_name_in_global_scope(node);
            return;
        }
        let has_modifiers = node.modifiers().is_some();
        if !self.check_grammar_modifiers(node) && has_modifiers {
            self.grammar_error_on_first_token(node, &AN_IMPORT_DECLARATION_CANNOT_HAVE_MODIFIERS);
        }
        if self.check_external_import_or_export_declaration(node) {
            let attributes = get_import_attributes(node);
            let mut resolved_module: Option<Arc<Symbol>> = None;
            let import_clause = import_clause(node).cloned();
            let module_specifier = node_module_specifier(node).cloned().unwrap_or_else(|| Arc::clone(node));
            if let Some(import_clause) = &import_clause {
                if !self.check_grammar_import_clause(import_clause) {
                    if import_clause.name().is_some() {
                        self.check_import_binding(import_clause);
                    }
                    let mut needs_import_star = false;
                    let named_bindings: Option<std::sync::Arc<tsox_frontend::ast::Node>> = None;
                    if let Some(named_bindings) = &named_bindings {
                        if tsox_frontend::ast::is_namespace_import(named_bindings) {
                            self.check_import_binding(named_bindings);
                            if self.emit_module_format_of_node_source_file(node) == ModuleKind::CommonJS {
                                needs_import_star = true;
                                self.check_external_emit_helpers(node, ExternalEmitHelpers::ImportStar.bits());
                            }
                        } else {
                            let attributes_type = self.get_type_from_import_attributes_opt(attributes.as_ref());
                            resolved_module = self.resolve_external_module_name_worker(node, Some(&module_specifier), None, false, false, attributes_type.as_ref());
                            if resolved_module.is_some() {
                                let elements: &[std::sync::Arc<tsox_frontend::ast::Node>] = match &named_bindings.data { NodeData::NamedImports(d) => &d.elements.nodes, _ => &[] };
                            for binding in elements {
                                    self.check_import_binding(binding);
                                }
                            }
                        }
                    }
                    if import_clause.name().is_some() && !needs_import_star {
                        if self.emit_module_format_of_node_source_file(node) == ModuleKind::CommonJS {
                            self.check_external_emit_helpers(node, ExternalEmitHelpers::ImportDefault.bits());
                        }
                    }
                    if !node_is_type_only(import_clause)
                        && ModuleKind::Node18 <= self.module_kind
                        && self.module_kind <= ModuleKind::NodeNext
                        && {
                            let attributes_type = self.get_type_from_import_attributes_opt(attributes.as_ref());
                            self.is_only_importable_as_default(&module_specifier, resolved_module.as_ref(), attributes_type.as_ref())
                        }
                        && !has_type_json_import_attribute(node)
                    {
                        let module_kind_string = format!("{:?}", self.module_kind);
                        self.error_message(&module_specifier,IMPORTING_A_JSON_FILE_INTO_AN_ECMASCRIPT_MODULE_REQUIRES_A_TYPE_COLON_JSON_IMPORT_ATTRIBUTE_WHEN_MODULE_IS_SET_TO_0, &[module_kind_string]);
                    }
                }
            } else if self.compiler_options.no_unchecked_side_effect_imports.is_true_or_unknown() && import_clause.is_none() {
                let ignore_errors = self.compiler_options.no_check.is_true();
                let error_message: Option<&'static tsox_core::diagnostics::Message> = if ignore_errors {
                    None
                } else {
                    Some(&CANNOT_FIND_MODULE_OR_TYPE_DECLARATIONS_FOR_SIDE_EFFECT_IMPORT_OF_0)
                };
                let attributes_type = self.get_type_from_import_attributes_opt(attributes.as_ref());
                self.resolve_external_module_name_worker(node, Some(&module_specifier), error_message, ignore_errors, false, attributes_type.as_ref());
            }
        }
        self.check_import_attributes(node);
    }

    pub fn check_import_equals_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_import_equals_declaration"); 
        let diagnostic = if is_in_js_file(node) {
            AN_IMPORT_DECLARATION_CAN_ONLY_BE_USED_AT_THE_TOP_LEVEL_OF_A_MODULE
        } else {
            AN_IMPORT_DECLARATION_CAN_ONLY_BE_USED_AT_THE_TOP_LEVEL_OF_A_NAMESPACE_OR_MODULE
        };
        if self.check_grammar_module_element_context(node, &diagnostic) {
            self.check_external_module_name_in_global_scope(node);
            return;
        }
        self.check_grammar_modifiers(node);
        if self.should_check_erasable_syntax(node) && !node.flags.contains(NodeFlags::Ambient) {
            self.error_message(node,THIS_SYNTAX_IS_NOT_ALLOWED_WHEN_ERASABLESYNTAXONLY_IS_ENABLED, &[]);
        }
        let module_reference = match &node.data {
            NodeData::ImportEqualsDeclaration(data) => Arc::clone(&data.module_reference),
            _ => Arc::clone(node),
        };
        if is_internal_module_import_equals_declaration(node) || self.check_external_import_or_export_declaration(node) {
            self.check_import_binding(node);
            self.mark_linked_references(node, ReferenceHint::ExportImportEquals, None, None);
            if !tsox_frontend::ast::is_external_module_reference(&module_reference) {
                let symbol = self.get_symbol_of_declaration(node).expect("import equals symbol");
                let target = self.resolve_alias(&symbol);
                if !symbol_option_ptr_eq(&Some(Arc::clone(&target)), &self.unknown_symbol) {
                    let target_flags = self.get_symbol_flags(&target);
                    if target_flags.contains(SymbolFlags::VALUE) {
                        let module_name = get_first_identifier(&module_reference);
                        let resolved = self.resolve_entity_name(&module_name, SymbolFlags::VALUE.union(SymbolFlags::NAMESPACE), false, false, None);
                        if resolved.as_ref().map(|s| s.flags.contains(SymbolFlags::NAMESPACE)).unwrap_or(false) {
                            let name = declaration_name_to_string(Some(&module_name));
                            self.error_message(&module_name,MODULE_0_IS_HIDDEN_BY_A_LOCAL_DECLARATION_WITH_THE_SAME_NAME, &[name]);
                        }
                    }
                    if target_flags.contains(SymbolFlags::TYPE) {
                        let name = node.name().unwrap();
                        self.check_type_name_is_reserved(&name, IMPORT_NAME_CANNOT_BE_0);
                    }
                }
                if node_is_type_only(node) {
                    self.grammar_error_on_node(node, &AN_IMPORT_ALIAS_CANNOT_USE_IMPORT_TYPE);
                }
            } else if ModuleKind::ES2015 <= self.module_kind
                && self.module_kind <= ModuleKind::ESNext
                && !node_is_type_only(node)
                && !node.flags.contains(NodeFlags::Ambient)
            {
                self.grammar_error_on_node(node, &IMPORT_ASSIGNMENT_CANNOT_BE_USED_WHEN_TARGETING_ECMASCRIPT_MODULES_CONSIDER_USING_IMPORT_ASTERISK_AS_NS_FROM_MOD_IMPORT_A_FROM_MOD_IMPORT_D_FROM_MOD_OR_ANOTHER_MODULE_FORMAT_INSTEAD);
            }
        }
    }

    pub fn check_import_meta_property(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("check_import_meta_property"); 
        if ModuleKind::Node16 <= self.module_kind && self.module_kind <= ModuleKind::NodeNext {
            if self.emit_module_format_of_node_source_file(node) != ModuleKind::ESNext {
                self.error_message(node,THE_IMPORT_META_META_PROPERTY_IS_NOT_ALLOWED_IN_FILES_WHICH_WILL_BUILD_INTO_COMMONJS_OUTPUT, &[]);
            }
        } else if self.module_kind < ModuleKind::ES2020 && self.module_kind != ModuleKind::System {
            self.error_message(node,THE_IMPORT_META_META_PROPERTY_IS_ONLY_ALLOWED_WHEN_THE_MODULE_OPTION_IS_ES2020_ES2022_ESNEXT_SYSTEM_NODE16_NODE18_NODE20_OR_NODENEXT, &[]);
        }
        if node.name().map(|n| n.text() == "meta").unwrap_or(false) {
            return self.get_global_import_meta_type();
        }
        self.error_type()
    }

    pub fn check_import_type(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_import_type"); 
        let argument = match &node.data {
            NodeData::ImportTypeNode(data) => Arc::clone(&data.argument),
            _ => Arc::clone(node),
        };
        self.check_source_element(&argument);
        if let NodeData::ImportTypeNode(data) = &node.data {
            if let Some(attributes) = &data.attributes {
                self.get_resolution_mode_override(attributes, true);
            }
        }
        self.check_type_reference_or_import(node);
        self.check_import_attributes(node);
    }

    pub fn check_index_constraint_for_index_signature(&mut self, t: &Arc<Type>, check_info: &IndexInfo) { ::tsox_core::fntrace::enter("check_index_constraint_for_index_signature"); 
        let declaration = check_info.declaration.clone();
        let check_key_type = check_info.key_type.as_ref().expect("index info key type");
        let index_infos = self.get_applicable_index_infos(t, check_key_type);
        if index_infos.is_empty() {
            return;
        }
        let mut interface_declaration: Option<Arc<Node>> = None;
        if t.object_flags.contains(ObjectFlags::Interface) {
            interface_declaration = t
                .symbol
                .as_ref()
                .and_then(|symbol| self.get_declaration_of_kind(symbol, SyntaxKind::InterfaceDeclaration));
        }
        let mut local_check_declaration: Option<Arc<Node>> = None;
        if let Some(declaration) = &declaration {
            if let Some(declaration_symbol) = self.get_symbol_of_declaration(declaration) {
                if let Some(parent_symbol) = self.get_parent_of_symbol(&declaration_symbol) {
                    if t.symbol.as_ref().map(|ts| Arc::ptr_eq(&parent_symbol, ts)).unwrap_or(false) {
                        local_check_declaration = Some(Arc::clone(declaration));
                    }
                }
            }
        }
        for info in &index_infos {
            if std::ptr::eq(Arc::as_ptr(info), check_info) {
                continue;
            }
            let mut local_index_declaration: Option<Arc<Node>> = None;
            if let Some(info_declaration) = &info.declaration {
                if let Some(info_symbol) = self.get_symbol_of_declaration(info_declaration) {
                    if let Some(parent_symbol) = self.get_parent_of_symbol(&info_symbol) {
                        if t.symbol.as_ref().map(|ts| Arc::ptr_eq(&parent_symbol, ts)).unwrap_or(false) {
                            local_index_declaration = Some(Arc::clone(info_declaration));
                        }
                    }
                }
            }
            let mut error_node = local_check_declaration.clone().or(local_index_declaration);
            if error_node.is_none() {
                if let Some(interface_declaration) = &interface_declaration {
                    let info_key_type = info.key_type.as_ref().expect("index info key type");
                    let info_index_kind = if info_key_type.flags.contains(TypeFlags::String) {
                        IndexKind::String
                    } else {
                        IndexKind::Number
                    };
                    let both_in_base = self.get_base_types(t).iter().any(|base| {
                        self.get_index_info_of_type(base, check_key_type).is_some()
                            && self.get_index_type_of_type(base, info_index_kind).is_some()
                    });
                    if !both_in_base {
                        error_node = Some(Arc::clone(interface_declaration));
                    }
                }
            }
            if let Some(error_node) = error_node {
                let check_value_type = check_info.value_type.as_ref().expect("index info value type");
                let info_value_type = info.value_type.as_ref().expect("index info value type");
                if !self.is_type_assignable_to(check_value_type, info_value_type) {
                    let key1 = self.type_to_string(check_key_type);
                    let value1 = self.type_to_string(check_value_type);
                    let key2 = self.type_to_string(info.key_type.as_ref().expect("index info key type"));
                    let value2 = self.type_to_string(info_value_type);
                    self.error_message(&error_node,X_0_INDEX_TYPE_1_IS_NOT_ASSIGNABLE_TO_2_INDEX_TYPE_3, &[key1, value1, key2, value2]);
                }
            }
        }
    }

    pub fn check_indexed_access_type(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_indexed_access_type"); 
        tsox_frontend::ast::for_each_child(node, |child| {
            self.check_source_element(&child)
        });
        self.check_indexed_access_index_type(node);
    }

    pub fn check_infer_type(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_infer_type"); 
        let in_extends_clause = find_ancestor_node(node, |n| {
            n.parent()
                .map(|parent| {
                    parent.kind == SyntaxKind::ConditionalType
                        && matches!(&parent.data, NodeData::ConditionalTypeNode(data) if Arc::ptr_eq(&data.extends_type, n))
                })
                .unwrap_or(false)
        })
        .is_some();
        if !in_extends_clause {
            self.grammar_error_on_node(node, &X_INFER_DECLARATIONS_ARE_ONLY_PERMITTED_IN_THE_EXTENDS_CLAUSE_OF_A_CONDITIONAL_TYPE);
        }
        let type_parameter_declaration_node = match &node.data {
            NodeData::InferTypeNode(data) => Arc::clone(&data.type_parameter),
            _ => Arc::clone(node),
        };
        self.check_source_element(&type_parameter_declaration_node);
        let symbol = self.get_symbol_of_declaration(&type_parameter_declaration_node);
        if let Some(symbol) = &symbol {
            if symbol.declarations.len() > 1 {
            let links = self.declared_type_links.get(symbol);
            if !links.map(|l| l.type_parameters_checked).unwrap_or(false) {
                if let Some(l) = self.declared_type_links.get_mut(symbol) { l.type_parameters_checked = true; }
                let type_parameter = self.get_declared_type_of_type_parameter(&symbol);
                let declarations = get_declarations_of_kind(&symbol, SyntaxKind::TypeParameter);
                let identical = self.are_type_parameters_identical(
                    &declarations,
                    &[type_parameter],
                    &|decl: &Arc<Node>| vec![Arc::clone(decl)],
                );
                if !identical {
                    let name = self.symbol_to_string(&symbol);
                    for declaration in &declarations {
                        let decl_name = declaration.name().expect("type parameter name");
                        self.error_message(&decl_name,ALL_DECLARATIONS_OF_0_MUST_HAVE_IDENTICAL_CONSTRAINTS, &[name.clone()]);
                    }
                }
            }
        }
        }
        self.register_for_unused_identifiers_check(node);
    }

    pub fn check_inherited_properties_are_identical(&mut self, t: &Arc<Type>, type_node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("check_inherited_properties_are_identical"); 
        let base_types = self.get_base_types(t);
        if base_types.len() < 2 {
            return true;
        }
        let mut seen: HashMap<String, InheritanceInfo> = HashMap::new();
        self.resolve_declared_members(t);
        let declared_members = t
            .as_interface_type()
            .map(|it| it.declared_members.clone())
            .unwrap_or_default();
        for (id, p) in declared_members.iter() {
            if self.is_named_member(p, id) {
                seen.insert(
                    p.name.clone(),
                    InheritanceInfo {
                        prop: Arc::clone(p),
                        containing_type: Arc::clone(t),
                    },
                );
            }
        }
        let mut identical = true;
        let this_type = t.as_interface_type().and_then(|it| it.this_type.clone());
        for base in &base_types {
            let base_with_this = self.get_type_with_this_argument(base, this_type.as_ref(), false);
            let properties = self.get_properties_of_type(&base_with_this);
            for prop in properties {
                match seen.get(&prop.name) {
                    None => {
                        seen.insert(
                            prop.name.clone(),
                            InheritanceInfo {
                                prop: Arc::clone(&prop),
                                containing_type: Arc::clone(base),
                            },
                        );
                    }
                    Some(existing) => {
                        let is_inherited_property = !Arc::ptr_eq(&existing.containing_type, t);
                        if is_inherited_property && !self.is_property_identical_to(&existing.prop, &prop) {
                            identical = false;
                            let type_name1 = self.type_to_string(&existing.containing_type);
                            let type_name2 = self.type_to_string(base);
                            let prop_name = self.symbol_to_string(&prop);
                            let error_info = new_diagnostic_for_node(
                                Some(type_node),
                                NAMED_PROPERTY_0_OF_TYPES_1_AND_2_ARE_NOT_IDENTICAL,
                                vec![prop_name, type_name1.clone(), type_name2.clone()],
                            );
                            let t_string = self.type_to_string(t);
                            self.add_diagnostic(new_diagnostic_chain(
                                Some(&error_info),
                                INTERFACE_0_CANNOT_SIMULTANEOUSLY_EXTEND_TYPES_1_AND_2,
                                vec![t_string, type_name1, type_name2],
                            ));
                        }
                    }
                }
            }
        }
        identical
    }
}
