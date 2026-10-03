#![allow(unused_imports)]
use tsox_core::diagnostics::{Message, messages_generated::*};

use tsox_frontend::ast::skip_parentheses;
use tsox_frontend::scanner::mig::m3i::declaration_name_to_string;
use tsox_frontend::ast::mig::m3d_2::new_diagnostic_chain;
use tsox_frontend::ast::utilities::get_containing_class;
use tsox_frontend::ast::utilities::find_ancestor;
use tsox_frontend::ast::utilities::is_in_js_file;
use tsox_frontend::ast::utilities::is_string_literal_like;
use tsox_frontend::ast::node_data_generated::{
    is_identifier, is_private_identifier, is_decorator,
};
use tsox_frontend::ast::utilities_r16a::is_for_in_statement;
use tsox_frontend::ast::mig::m3f_2::has_abstract_modifier;
use crate::checker::utilities_is_optional_symbol::{is_type_any, is_late_bound_name};
use crate::checker::utilities_token_is_identifier_or_keyword::is_object_literal_type;
use crate::checker::exports_union_reduction::{
    get_declaration_modifier_flags_from_symbol, get_declaration_modifier_flags_from_symbol_ex,
};
use crate::checker::grammarchecks_is_this_parameter_2::is_binding_pattern;
use crate::checker::mig::m1c_3::create_diagnostic_for_node_message;
use crate::checker::mig::m3a_2::new_diagnostic_for_node;
use crate::checker::mig::wc3::r18k4_node_ext::NodeAccessExt;
use crate::checker::mig::m2a::{is_rest_parameter, is_prototype_property};
use crate::checker::mig::wc3_2::is_const_enum_object_type;
use crate::checker::mig::w9a::new_type_mapper;
use crate::checker::relater_recursion_identity::RecursionIdentity as RecursionId;
use super::wc1b::{every_type, walk_up_parenthesized_expressions, class_or_constructor_parameter_is_decorated, is_tuple_type, is_array_or_tuple_type, is_literal_type, is_initialized_property};
use super::wc1c::r24k17_defs::{error_node_for_call_node_arc};
use tsox_frontend::ast::Diagnostic;

use crate::checker::checker_checker::*;
use std::sync::Arc;

impl Checker {
    pub fn check_element_access_chain(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> { ::tsox_core::fntrace::enter("check_element_access_chain"); 
        let expr_type = self.check_expression_ex(&node.expression().unwrap(), CheckMode::Normal);
        let non_optional_type = self.get_optional_expression_type(&expr_type, &node.expression().unwrap());
        let non_null = self.check_non_null_type(&non_optional_type, &node.expression().unwrap());
        let checked = self.check_element_access_expression(node, &non_null, check_mode);
        self.propagate_optional_type_marker(&checked, node, !Arc::ptr_eq(&non_optional_type, &expr_type))
    }

    pub fn check_element_access_expression(
        &mut self,
        node: &Arc<Node>,
        expr_type: &Arc<Type>,
        check_mode: CheckMode,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("check_element_access_expression"); 
        let mut object_type = Arc::clone(expr_type);
        if get_assignment_target_kind(node) != AssignmentKind::None || self.is_method_access_for_call(node) {
            object_type = self.get_widened_type(&object_type);
        }
        let index_expression = node.as_element_access_expression().argument_expression.clone();
        let index_type = self.check_expression_ex(&index_expression, CheckMode::Normal);
        if self.is_error_type(&object_type) || Arc::ptr_eq(&object_type, &self.silent_never_type()) {
            return object_type;
        }
        if is_const_enum_object_type(&object_type) && !is_string_literal_like(&index_expression) {
            self.error_message(&index_expression,A_CONST_ENUM_MEMBER_CAN_ONLY_BE_ACCESSED_USING_A_STRING_LITERAL, &[]);
            return self.error_type();
        }
        let mut effective_index_type = Arc::clone(&index_type);
        if self.is_for_in_variable_for_numeric_property_names(&index_expression) {
            effective_index_type = self.number_type();
        }
        let assignment_target_kind = get_assignment_target_kind(node);
        let mut access_flags = AccessFlags::None;
        if assignment_target_kind == AssignmentKind::None {
            access_flags |= AccessFlags::ExpressionPosition;
        } else {
            access_flags |= AccessFlags::Writing;
            if assignment_target_kind == AssignmentKind::Compound {
                access_flags |= AccessFlags::ExpressionPosition;
            }
            if self.is_generic_object_type(&object_type) && !self.is_this_type_parameter(&object_type) {
                access_flags |= AccessFlags::NoIndexSignatures;
            }
        }
        let indexed_access_type = self
            .get_indexed_access_type_or_undefined(&object_type, &effective_index_type, access_flags, Some(node), None)
            .unwrap_or_else(|| self.error_type());
        let flow = self.flow_type_of_access_expression(
            node,
            self.get_resolved_symbol_or_nil(node).as_ref(),
            Arc::clone(&indexed_access_type),
        );
        self.check_indexed_access_index_type(node);
        flow
    }


    pub fn check_private_identifier_property_access(
        &mut self,
        left_type: &Arc<Type>,
        right: &Arc<Node>,
        lexically_scoped_identifier: Option<&Arc<Symbol>>,
    ) -> bool { ::tsox_core::fntrace::enter("check_private_identifier_property_access"); 
        let properties = self.get_properties_of_type(left_type);
        let mut property_on_type: Option<Arc<Symbol>> = None;
        for symbol in &properties {
            let decl = symbol.value_declaration.clone();
            if let Some(decl) = decl {
                if let Some(name) = decl.name() {
                    if is_private_identifier(&name) && name.text() == right.text() {
                        property_on_type = Some(Arc::clone(symbol));
                        break;
                    }
                }
            }
        }
        let diag_name = declaration_name_to_string(Some(right));
        if let Some(property_on_type) = property_on_type {
            let type_value_decl = property_on_type.value_declaration.clone();
            let type_class = type_value_decl.as_ref().and_then(|d| get_containing_class(d));
            if let Some(lexically_scoped_identifier) = lexically_scoped_identifier {
                if let Some(lexical_value_decl) = lexically_scoped_identifier.value_declaration.clone() {
                    let lexical_class = get_containing_class(&lexical_value_decl);
                    let shadowed = lexical_class.as_ref().is_some_and(|lc| {
                        find_ancestor(&lexical_value_decl, |n: &Node| {
                            type_class.as_ref().is_some_and(|tc| std::ptr::eq(Arc::as_ref(tc), n))
                        })
                        .is_some()
                    });
                    if shadowed {
                        let left_type_str = self.type_to_string(left_type);
                        if let Some(mut diagnostic) = self.error_message(right,THE_PROPERTY_0_CANNOT_BE_ACCESSED_ON_TYPE_1_WITHIN_THIS_CLASS_BECAUSE_IT_IS_SHADOWED_BY_ANOTHER_PRIVATE_IDENTIFIER_WITH_THE_SAME_SPELLING, &[diag_name.clone(), left_type_str]) {
                            Arc::get_mut(&mut diagnostic).unwrap().add_related_info(Arc::unwrap_or_clone(create_diagnostic_for_node_message(&lexical_value_decl,THE_SHADOWING_DECLARATION_OF_0_IS_DEFINED_HERE, &[diag_name.clone()])));
                            if let Some(type_value_decl) = &type_value_decl {
                                Arc::get_mut(&mut diagnostic).unwrap().add_related_info(Arc::unwrap_or_clone(create_diagnostic_for_node_message(type_value_decl,THE_DECLARATION_OF_0_THAT_YOU_PROBABLY_INTENDED_TO_USE_IS_DEFINED_HERE, &[diag_name.clone()])));
                            }
                        }
                        return true;
                    }
                }
            }
            if let Some(type_class) = &type_class {
                if let Some(class_symbol) = self.get_symbol_of_node(type_class) {
                    let class_symbol_str = self.symbol_to_string(&class_symbol);
                    self.error_message(right,PROPERTY_0_IS_NOT_ACCESSIBLE_OUTSIDE_CLASS_1_BECAUSE_IT_HAS_A_PRIVATE_IDENTIFIER, &[diag_name.clone(), class_symbol_str]);
                }
            }
            return true;
        }
        false
    }

    pub fn get_argument_arity_error(
        &mut self,
        node: &Arc<Node>,
        signatures: &[Arc<Signature>],
        args: &[Arc<Node>],
        head_message: Option<&'static Message>,
    ) -> Diagnostic { ::tsox_core::fntrace::enter("get_argument_arity_error"); 
        let spread_index = self.get_spread_argument_index(args);
        if spread_index > -1 {
            return new_diagnostic_for_node(Some(&args[spread_index as usize]), A_SPREAD_ARGUMENT_MUST_EITHER_HAVE_A_TUPLE_TYPE_OR_BE_PASSED_TO_A_REST_PARAMETER, vec![]);
        }
        let mut min_count = i32::MAX;
        let mut max_count = i32::MIN;
        let mut max_below = i32::MIN;
        let mut min_above = i32::MAX;
        let mut closest_signature: Option<Arc<Signature>> = None;
        for sig in signatures {
            let min_parameter = self.get_min_argument_count(sig) as i32;
            let max_parameter = self.get_parameter_count(sig) as i32;
            if min_parameter < min_count {
                min_count = min_parameter;
                closest_signature = Some(Arc::clone(sig));
            }
            max_count = max_count.max(max_parameter);
            if min_parameter < args.len() as i32 && min_parameter > max_below {
                max_below = min_parameter;
            }
            if (args.len() as i32) < max_parameter && max_parameter < min_above {
                min_above = max_parameter;
            }
        }
        let has_rest_parameter = signatures
            .iter()
            .any(|sig| self.has_effective_rest_parameter(sig));
        let parameter_range = if has_rest_parameter {
            min_count.to_string()
        } else if min_count < max_count {
            format!("{}-{}", min_count, max_count)
        } else {
            min_count.to_string()
        };
        let is_void_promise_error = !has_rest_parameter
            && parameter_range == "1"
            && args.is_empty()
            && self.is_promise_resolve_arity_error(node);
        let error_node = error_node_for_call_node_arc(node);
        if is_void_promise_error && is_in_js_file(node) {
            return new_diagnostic_for_node(Some(&error_node), EXPECTED_1_ARGUMENT_BUT_GOT_0_NEW_PROMISE_NEEDS_A_JSDOC_HINT_TO_PRODUCE_A_RESOLVE_THAT_CAN_BE_CALLED_WITHOUT_ARGUMENTS, vec![]);
        }
        let message = if is_decorator(node) {
            if has_rest_parameter {
                THE_RUNTIME_WILL_INVOKE_THE_DECORATOR_WITH_1_ARGUMENTS_BUT_THE_DECORATOR_EXPECTS_AT_LEAST_0
            } else {
                THE_RUNTIME_WILL_INVOKE_THE_DECORATOR_WITH_1_ARGUMENTS_BUT_THE_DECORATOR_EXPECTS_0
            }
        } else if has_rest_parameter {
            EXPECTED_AT_LEAST_0_ARGUMENTS_BUT_GOT_1
        } else if is_void_promise_error {
            EXPECTED_0_ARGUMENTS_BUT_GOT_1_DID_YOU_FORGET_TO_INCLUDE_VOID_IN_YOUR_TYPE_ARGUMENT_TO_PROMISE
        } else {
            EXPECTED_0_ARGUMENTS_BUT_GOT_1
        };
        let arg_count = args.len() as i32;
        if min_count < arg_count && arg_count < max_count {
            let diagnostic = new_diagnostic_for_node(Some(&error_node), NO_OVERLOAD_EXPECTS_0_ARGUMENTS_BUT_OVERLOADS_DO_EXIST_THAT_EXPECT_EITHER_1_OR_2_ARGUMENTS, vec![arg_count.to_string(), max_below.to_string(), min_above.to_string()]);
            return match head_message {
                Some(head_message) => new_diagnostic_chain(Some(&diagnostic), *head_message, vec![]),
                None => diagnostic,
            };
        }
        if arg_count < min_count {
            let mut diagnostic = new_diagnostic_for_node(Some(&error_node), message, vec![parameter_range.clone(), arg_count.to_string()]);
            let mut diagnostic = match head_message {
                Some(head_message) => new_diagnostic_chain(Some(&diagnostic), *head_message, vec![]),
                None => diagnostic,
            };
            let parameter = closest_signature.as_ref().and_then(|sig| {
                sig.declaration.as_ref().and_then(|d| {
                    let offset = usize::from(sig.this_parameter.is_some());
                    d.parameters()
                        .and_then(|l| l.nodes.get(arg_count as usize + offset).cloned())
                })
            });
            if let Some(parameter) = parameter {
                let name = parameter.name().unwrap();
                let related = if is_binding_pattern(&name) {
                    new_diagnostic_for_node(Some(&parameter), AN_ARGUMENT_MATCHING_THIS_BINDING_PATTERN_WAS_NOT_PROVIDED, vec![])
                } else if is_rest_parameter(&parameter) {
                    new_diagnostic_for_node(Some(&parameter), ARGUMENTS_FOR_THE_REST_PARAMETER_0_WERE_NOT_PROVIDED, vec![name.text().to_string()])
                } else {
                    new_diagnostic_for_node(Some(&parameter), AN_ARGUMENT_FOR_0_WAS_NOT_PROVIDED, vec![name.text().to_string()])
                };
                diagnostic.add_related_info(related);
            }
            return diagnostic;
        }
        if max_count >= arg_count {
            let diagnostic = new_diagnostic_for_node(Some(&error_node), message, vec![parameter_range.clone(), arg_count.to_string()]);
            return match head_message {
                Some(head_message) => new_diagnostic_chain(Some(&diagnostic), *head_message, vec![]),
                None => diagnostic,
            };
        }
        new_diagnostic_for_node(Some(&error_node), message, vec![parameter_range.clone(), arg_count.to_string()])
    }

    pub fn create_union_or_intersection_property(
        &mut self,
        containing_type: &Arc<Type>,
        name: &str,
        skip_object_function_property_augment: bool,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("create_union_or_intersection_property"); 
        let mut prop_flags = SymbolFlags::None;
        let mut single_prop: Option<Arc<Symbol>> = None;
        let mut prop_set: Vec<Arc<Symbol>> = Vec::new();
        let mut index_types: Vec<Arc<Type>> = Vec::new();
        let is_union = containing_type.flags.intersects(TypeFlags::Union);
        let mut check_flags = CheckFlags::None;
        let mut optional_flag = SymbolFlags::None;
        if !is_union {
            check_flags = CheckFlags::Readonly;
            optional_flag = SymbolFlags::Optional;
        }
        let mut synthetic_flag = CheckFlags::SyntheticMethod;
        let mut merged_instantiations = false;
        for current in containing_type.types().unwrap_or(&[]) {
            let t = self.get_apparent_type(current);
            if !self.is_error_type(&t) && !t.flags.intersects(TypeFlags::Never) {
                let prop = self.get_property_of_type_ex(&t, name, skip_object_function_property_augment, false);
                if let Some(prop) = prop {
                    let modifiers = get_declaration_modifier_flags_from_symbol(&prop);
                    let write_modifiers = get_declaration_modifier_flags_from_symbol_ex(&prop, true);
                    if prop.flags.intersects(SymbolFlags::CLASS_MEMBER) {
                        if is_union {
                            optional_flag |= prop.flags & SymbolFlags::Optional;
                        } else {
                            optional_flag &= prop.flags;
                        }
                    }
                    let mut replace_single = false;
                    match &single_prop {
                        None => {
                            single_prop = Some(Arc::clone(&prop));
                            prop_flags = if prop.flags.intersects(SymbolFlags::ACCESSOR) {
                                prop_flags | SymbolFlags::Property
                            } else {
                                prop_flags
                            };
                        }
                        Some(single) => {
                            if !Arc::ptr_eq(&prop, single) {
                                let is_instantiation =
                                    Arc::ptr_eq(&self.get_target_symbol(&prop), &self.get_target_symbol(single));
                                if is_instantiation
                                    && self.compare_properties(single, &prop, &|c, s, t| {
                                        compare_types_equal(s, t)
                                    })
                                        == Ternary::True
                                {
                                    merged_instantiations = single.parent().is_some()
                                        && !self.get_local_type_parameters_of_class_or_interface_or_type_alias(
                                            single.parent().as_ref().unwrap(),
                                        )
                                        .is_empty();
                                } else {
                                    if prop_set.is_empty() {
                                        prop_set.push(Arc::clone(single));
                                    }
                                    if !prop_set.iter().any(|p| Arc::ptr_eq(p, &prop)) {
                                        prop_set.push(Arc::clone(&prop));
                                    }
                                }
                                if prop_flags.intersects(SymbolFlags::ACCESSOR)
                                    && prop.flags & SymbolFlags::ACCESSOR != prop_flags & SymbolFlags::ACCESSOR
                                {
                                    prop_flags = (prop_flags - SymbolFlags::ACCESSOR) | SymbolFlags::Property;
                                }
                            }
                            let _ = &mut replace_single;
                        }
                    }
                    if is_union && self.is_readonly_symbol(&prop) {
                        check_flags |= CheckFlags::Readonly;
                    } else if !is_union && !self.is_readonly_symbol(&prop) {
                        check_flags -= CheckFlags::Readonly;
                    }
                    if modifiers.intersects(ModifierFlags::Protected)
                        && !modifiers.intersects(ModifierFlags::Public)
                    {
                        check_flags |= CheckFlags::ContainsProtected;
                    } else if modifiers.intersects(ModifierFlags::Private)
                        && !modifiers.intersects(ModifierFlags::Public)
                    {
                        check_flags |= CheckFlags::ContainsPrivate;
                    } else {
                        check_flags |= CheckFlags::ContainsPublic;
                    }
                    if write_modifiers.intersects(ModifierFlags::Protected)
                        && !write_modifiers.intersects(ModifierFlags::Public)
                    {
                        check_flags |= (CheckFlags::ContainsProtected.union(CheckFlags::WritePartial));
                    } else if write_modifiers.intersects(ModifierFlags::Private)
                        && !write_modifiers.intersects(ModifierFlags::Public)
                    {
                        check_flags |= (CheckFlags::ContainsPrivate.union(CheckFlags::WritePartial));
                    } else {
                        check_flags |= (CheckFlags::ContainsPublic.union(CheckFlags::WritePartial));
                    }
                    if modifiers.intersects(ModifierFlags::Static) {
                        check_flags |= CheckFlags::ContainsStatic;
                    }
                    if !is_prototype_property(&prop) {
                        synthetic_flag = CheckFlags::SyntheticProperty;
                    }
                } else if is_union {
                    let mut index_info: Option<Arc<IndexInfo>> = None;
                    if !is_late_bound_name(name) {
                        index_info = self.get_applicable_index_info_for_name(&t, name);
                    }
                    if let Some(index_info) = index_info {
                        prop_flags = (prop_flags - SymbolFlags::ACCESSOR) | SymbolFlags::Property;
                        check_flags |= CheckFlags::WritePartial;
                        if index_info.is_readonly {
                            check_flags |= CheckFlags::Readonly;
                        }
                        if is_tuple_type(&t) {
                            let index_type = self.get_rest_type_of_tuple_type(&t);
                            if Arc::ptr_eq(&index_type, &self.error_type()) {
                                index_types.push(self.undefined_type());
                            } else {
                                index_types.push(index_type);
                            }
                        } else {
                            index_types.push(index_info.value_type.clone().unwrap());
                        }
                    } else if is_object_literal_type(&t)
                        && !t.object_flags.intersects(ObjectFlags::ContainsSpread)
                    {
                        check_flags |= CheckFlags::WritePartial;
                        index_types.push(self.undefined_type());
                    } else {
                        check_flags |= CheckFlags::ReadPartial;
                    }
                }
            }
        }
        let single_prop = match single_prop {
            Some(single_prop) => single_prop,
            None => return None,
        };
        if is_union
            && (!prop_set.is_empty() || check_flags.intersects((CheckFlags::ReadPartial.union(CheckFlags::WritePartial))))
            && check_flags.intersects(
                CheckFlags::ContainsPrivate
                    | CheckFlags::ContainsProtected
                    | (CheckFlags::ContainsPrivate.union(CheckFlags::WritePartial))
                    | (CheckFlags::ContainsProtected.union(CheckFlags::WritePartial)),
            )
            && !(!prop_set.is_empty() && self.has_common_declaration(&prop_set))
        {
            if check_flags
                .intersects(CheckFlags::ContainsPrivate | CheckFlags::ContainsProtected)
            {
                return None;
            }
            if check_flags.intersects((CheckFlags::ContainsPrivate.union(CheckFlags::WritePartial))) {
                check_flags -= (CheckFlags::ContainsPublic.union(CheckFlags::WritePartial)) | (CheckFlags::ContainsProtected.union(CheckFlags::WritePartial));
            } else if check_flags.intersects((CheckFlags::ContainsProtected.union(CheckFlags::WritePartial))) {
                check_flags -= (CheckFlags::ContainsPublic.union(CheckFlags::WritePartial));
            }
        }
        if prop_set.is_empty()
            && !check_flags.intersects(CheckFlags::ReadPartial)
            && index_types.is_empty()
        {
            if !merged_instantiations {
                return Some(single_prop);
            }
            let mut single_prop_type: Option<Arc<Type>> = None;
            let mut single_prop_mapper: Option<Arc<TypeMapper>> = None;
            if single_prop.flags.intersects(SymbolFlags::Transient) {
                let links = self.value_symbol_links.get_or_default(&single_prop);
                single_prop_type = links.resolved_type.clone();
                single_prop_mapper = links.mapper.clone();
            }
            let single_prop_type = single_prop_type.unwrap_or_else(|| self.unknown_type());
            let clone = self.create_symbol_with_type(&single_prop, &single_prop_type);
            if let Some(value_declaration) = single_prop.value_declaration.clone() {
                if let Some(parent_symbol) =
                    self.get_symbol_of_node(&value_declaration).and_then(|s| s.parent())
                {
                    clone.set_parent(&parent_symbol);
                }
            }
            let write_type = self.get_write_type_of_symbol(&single_prop);
            let links = self.value_symbol_links.get_or_default(&clone);
            links.containing_type = Some(Arc::clone(&containing_type));
            links.mapper = single_prop_mapper;
            links.write_type = Some(write_type);
            return Some(clone);
        }
        if prop_set.is_empty() {
            prop_set.push(Arc::clone(&single_prop));
        }
        let mut declarations: Vec<Arc<Node>> = Vec::new();
        let mut first_type: Option<Arc<Type>> = None;
        let mut name_type: Option<Arc<Type>> = None;
        let mut prop_types: Vec<Arc<Type>> = Vec::new();
        let mut write_types: Vec<Arc<Type>> = Vec::new();
        let mut first_value_declaration: Option<Arc<Node>> = None;
        let mut has_non_uniform_value_declaration = false;
        for prop in &prop_set {
            match &first_value_declaration {
                None => first_value_declaration = prop.value_declaration.clone(),
                Some(first) => {
                    if prop
                        .value_declaration
                        .as_ref()
                        .is_some_and(|d| !Arc::ptr_eq(d, first))
                    {
                        has_non_uniform_value_declaration = true;
                    }
                }
            }
            for declaration in &prop.declarations {
                if !declarations.iter().any(|d| Arc::ptr_eq(d, declaration)) {
                    declarations.push(Arc::clone(declaration));
                }
            }
            let t = self.get_type_of_symbol(prop);
            if first_type.is_none() {
                first_type = Some(t.clone());
                name_type = self.value_symbol_links.get(prop).and_then(|l| l.name_type.clone());
            }
            let write_type = self.get_write_type_of_symbol(prop);
            if !write_types.is_empty() || !Arc::ptr_eq(&write_type, &t) {
                if write_types.is_empty() {
                    write_types = prop_types.clone();
                }
                write_types.push(write_type);
            }
            if !Arc::ptr_eq(&t, first_type.as_ref().unwrap()) {
                check_flags |= CheckFlags::HasNonUniformType;
            }
            if is_literal_type(&t) || self.is_pattern_literal_type(&t) {
                check_flags |= CheckFlags::HasLiteralType;
            }
            if t.flags.intersects(TypeFlags::Never)
                && !Arc::ptr_eq(&t, &self.unique_literal_type())
            {
                check_flags |= CheckFlags::HasNeverType;
            }
            prop_types.push(t);
        }
        prop_types.extend(index_types);
        let mut result = self.new_symbol_ex(
            prop_flags | optional_flag,
            name,
            check_flags | synthetic_flag,
        );
        {
            let mut result_mut = Arc::get_mut(&mut result).unwrap();
            result_mut.declarations = declarations;
            if !has_non_uniform_value_declaration {
                if let Some(first_value_declaration) = &first_value_declaration {
                    result_mut.value_declaration = Some(Arc::clone(first_value_declaration));
                    if let Some(parent_symbol) = self
                        .get_symbol_of_node(first_value_declaration)
                        .and_then(|s| s.parent())
                    {
                        result_mut.set_parent(&parent_symbol);
                    }
                }
            }
        }
        let links = self.value_symbol_links.get_or_default(&result);
        links.containing_type = Some(Arc::clone(containing_type));
        links.name_type = name_type;
        drop(links);
        if prop_types.len() > 2 {
            let mut result_mut = Arc::get_mut(&mut result).unwrap();
            result_mut.check_flags |= CheckFlags::DeferredType;
            let deferred = self.deferred_symbol_links.get_or_default(&result);
            deferred.parent = Some(Arc::clone(containing_type));
            deferred.constituents = prop_types;
            deferred.write_constituents = write_types;
            return Some(result);
        }
        let resolved_type = if is_union {
            self.get_union_type(prop_types)
        } else {
            self.get_intersection_type(prop_types)
        };
        let write_type = if write_types.is_empty() {
            None
        } else if is_union {
            Some(self.get_union_type(write_types))
        } else {
            Some(self.get_intersection_type(write_types))
        };
        let links = self.value_symbol_links.get_or_default(&result);
        links.resolved_type = Some(resolved_type);
        links.write_type = write_type;
        Some(result)
    }
}

pub fn compare_types_equal(s: &Arc<Type>, t: &Arc<Type>) -> Ternary { ::tsox_core::fntrace::enter("compare_types_equal"); 
    if Arc::ptr_eq(s, t) {
        Ternary::True
    } else {
        Ternary::False
    }
}
