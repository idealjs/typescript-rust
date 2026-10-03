#![allow(unused_imports)]
use super::r18k6_defs::InternalSymbolName;
use tsox_frontend::ast::mig::m3e_4::find_constructor_declaration;
use crate::checker::utilities_token_is_identifier_or_keyword::get_property_name_from_type;
use tsox_frontend::ast::has_accessor_modifier;
use super::m3a_2::has_dot_dot_dot_token;
use tsox_frontend::ast::has_static_modifier;
use tsox_frontend::ast::node_data_generated::is_binding_element;
use tsox_frontend::ast::node_data_generated::is_binding_pattern;
use tsox_frontend::ast::mig::m3f_4::is_catch_clause_variable_declaration_or_binding_element;
use tsox_frontend::ast::node_data_generated::is_class_static_block_declaration;
use tsox_frontend::ast::utilities_r16a::is_object_binding_pattern;
use crate::checker::utilities_has_only_expression_initialization::is_optional_declaration;
use tsox_frontend::ast::node_data_generated::is_parameter_declaration;
use tsox_frontend::ast::node_data_generated::is_property_declaration;
use tsox_frontend::ast::node_data_generated::is_property_signature_declaration;
use tsox_frontend::ast::node_data_generated::is_set_accessor_declaration;
use crate::checker::utilities_token_is_identifier_or_keyword::is_type_usable_as_property_name;
use tsox_frontend::ast::node_data_generated::is_variable_declaration;
use tsox_frontend::ast::mig::m3b::{elements, initializer, members, property_name_or_name};
use tsox_frontend::ast::has_syntactic_modifier;

use crate::checker::checker::*;
use tsox_frontend::ast::{get_name_of_declaration, Node, NodeData, Symbol, SymbolFlags, SyntaxKind};

use super::m1g::r24k16_defs::{
    contextual_binding_patterns_pop, contextual_binding_patterns_push, pattern_for_type_insert,
};

impl Checker {
    pub(crate) fn get_type_for_variable_like_declaration(
        &mut self,
        declaration: &Arc<Node>,
        include_optionality: bool,
        check_mode: CheckMode,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_for_variable_like_declaration"); 
        if is_variable_declaration(declaration) {
            let grand_parent = declaration.parent().and_then(|p| p.parent());
            if let Some(grand_parent) = grand_parent {
                match grand_parent.kind {
                    SyntaxKind::ForInStatement => {
                        let expression = grand_parent.expression()?;
                        let expr_type =
                            self.check_expression_ex(&expression, check_mode);
                        let expr_type = self.get_non_nullable_type_if_needed(&expr_type);
                        let index_type = self.get_index_type(&expr_type);
                        if index_type
                            .flags
                            .intersects(TypeFlags::TypeParameter | TypeFlags::Index)
                        {
                            return Some(self.get_extract_string_type(&index_type));
                        }
                        return Some(self.string_type());
                    }
                    SyntaxKind::ForOfStatement => {
                        return self.check_right_hand_side_of_for_of(&grand_parent);
                    }
                    _ => {}
                }
            }
        } else if is_binding_element(declaration) {
            return self.get_type_for_binding_element(declaration);
        }
        let is_property = (is_property_declaration(declaration)
            && !has_accessor_modifier(declaration))
            || is_property_signature_declaration(declaration);
        let is_optional = include_optionality && is_optional_declaration(declaration);
        let declared_type = self.try_get_type_from_type_node(declaration);
        if is_catch_clause_variable_declaration_or_binding_element(declaration) {
            if let Some(declared_type) = declared_type {
                if declared_type
                    .flags
                    .intersects(TypeFlags::Any | TypeFlags::Unknown)
                {
                    return Some(declared_type);
                }
                return Some(self.error_type());
            }
            if self.use_unknown_in_catch_variables {
                return Some(self.unknown_type());
            }
            return Some(self.any_type());
        }
        if let Some(declared_type) = declared_type {
            return Some(self.add_optionality_ex(&declared_type, is_property, is_optional));
        }
        let decl_name = declaration.name();
        if self.no_implicit_any
            && is_variable_declaration(declaration)
            && !decl_name.as_ref().is_some_and(|n| is_binding_pattern(n))
            && self.get_combined_modifier_flags_cached(declaration)
                .intersection(ModifierFlags::Export)
                .is_empty()
            && declaration
                .flags
                .intersection(NodeFlags::Ambient)
                .is_empty()
        {
            let initializer = initializer(declaration);
            let is_constant = !self
                .get_combined_node_flags_cached(declaration)
                .contains(NodeFlags::Constant);
            if is_constant
                && (initializer.is_none()
                    || initializer.as_ref().is_some_and(|i| self.is_null_or_undefined(i)))
            {
                return Some(self.auto_type());
            }
            if let Some(initializer) = initializer {
                if self.is_empty_array_literal(initializer) {
                    return Some(self.auto_array_type());
                }
            }
        }
        if is_parameter_declaration(declaration) {
            let symbol = self.get_symbol_of_declaration(declaration)?;
            let fn_node = declaration.parent()?;
            if is_set_accessor_declaration(&fn_node) && self.has_bindable_name(&fn_node) {
                let parent_symbol = self.get_symbol_of_declaration(&fn_node);
                let getter = parent_symbol
                    .and_then(|s| self.get_declaration_of_kind(&s, SyntaxKind::GetAccessor));
                if let Some(getter) = getter {
                    let getter_signature = self.get_signature_from_declaration(&getter)?;
                    let this_parameter = self.get_accessor_this_parameter(&fn_node);
                    if let Some(this_parameter) = this_parameter {
                        if Arc::ptr_eq(&this_parameter, declaration) {
                            let getter_this = getter_signature
                                .this_parameter
                                .as_ref()
                                .expect("getter signature has this parameter");
                            return Some(self.get_type_of_symbol(getter_this));
                        }
                    }
                    return self.get_return_type_of_signature(&getter_signature);
                }
            }
            if let Some(t) = self.get_parameter_type_of_full_signature(&fn_node, declaration) {
                return Some(t);
            }
            let t = if symbol.name == tsox_frontend::ast::INTERNAL_SYMBOL_NAME_THIS {
                self.get_contextual_this_parameter_type(&fn_node)
            } else {
                self.get_contextually_typed_parameter_type(declaration)
            };
            if let Some(t) = t {
                return Some(self.add_optionality_ex(&t, false, is_optional));
            }
        }
        if initializer(declaration).is_some() {
            let t = self.check_declaration_initializer(declaration, check_mode, None);
            let t = self.widen_type_inferred_from_initializer(declaration, &t);
            return Some(self.add_optionality_ex(&t, is_property, is_optional));
        }
        if self.no_implicit_any && is_property_declaration(declaration) {
            if !has_static_modifier(declaration) {
                let parent = declaration.parent()?;
                let constructor = find_constructor_declaration(&parent);
                let t = match constructor {
                    Some(constructor) => self.symbol_of_node(declaration).and_then(|sym| {
                        self.get_flow_type_in_constructor(&sym, &constructor)
                    }),
                    None => {
                        if has_syntactic_modifier(declaration, ModifierFlags::Ambient) {
                            self.symbol_of_node(declaration)
                                .and_then(|sym| self.get_type_of_property_in_base_class(&sym))
                        } else {
                            None
                        }
                    }
                };
                let t = t?;
                return Some(self.add_optionality_ex(&t, true, is_optional));
            }
            let parent = declaration.parent()?;
            let static_blocks: Vec<Arc<Node>> = members(&parent)
                .iter()
                .filter(|m| is_class_static_block_declaration(m))
                .cloned()
                .collect();
            let t = if !static_blocks.is_empty() {
                self.symbol_of_node(declaration)
                    .and_then(|sym| self.get_flow_type_in_static_blocks(&sym, &static_blocks))
            } else if has_syntactic_modifier(declaration, ModifierFlags::Ambient) {
                self.symbol_of_node(declaration)
                    .and_then(|sym| self.get_type_of_property_in_base_class(&sym))
            } else {
                None
            };
            let t = t?;
            return Some(self.add_optionality_ex(&t, true, is_optional));
        }
        if declaration.kind == SyntaxKind::JsxAttribute {
            return Some(self.true_type());
        }
        if let Some(n) = declaration.name() {
            if is_binding_pattern(n) {
                return Some(self.get_type_from_binding_pattern(n, false, true));
            }
        }
        None
    }

    pub(crate) fn get_type_from_binding_pattern(
        &mut self,
        pattern: &Arc<Node>,
        include_pattern_in_type: bool,
        report_errors: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_binding_pattern"); 
        if include_pattern_in_type {
            contextual_binding_patterns_push(Arc::clone(pattern));
        }
        let result = if is_object_binding_pattern(pattern) {
            self.get_type_from_object_binding_pattern(pattern, include_pattern_in_type, report_errors)
        } else {
            self.get_type_from_array_binding_pattern(pattern, include_pattern_in_type, report_errors)
        };
        if include_pattern_in_type {
            contextual_binding_patterns_pop();
        }
        result
    }

    pub(crate) fn get_type_from_object_binding_pattern(
        &mut self,
        pattern: &Arc<Node>,
        include_pattern_in_type: bool,
        report_errors: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_object_binding_pattern"); 
        let mut members = SymbolTable::new();
        let mut string_index_info: Option<Arc<IndexInfo>> = None;
        let mut object_flags =
            ObjectFlags::ObjectLiteral | ObjectFlags::ContainsObjectOrArrayLiteral;
        for e in elements(pattern) {
            let name = match property_name_or_name(e) {
                Some(n) => Arc::clone(n),
                None => continue,
            };
            if has_dot_dot_dot_token(e) {
                let string_type = self.string_type();
                let any_type = self.any_type();
                string_index_info = Some(self.new_index_info(
                    &string_type,
                    &any_type,
                    false,
                    None,
                    &[],
                ));
                continue;
            }
            let expr_type = match self.get_literal_type_from_property_name(&name) {
                Some(t) => t,
                None => {
                    object_flags |=
                        ObjectFlags::ObjectLiteralPatternWithComputedProperties;
                    continue;
                }
            };
            if !is_type_usable_as_property_name(&expr_type) {
                object_flags |=
                    ObjectFlags::ObjectLiteralPatternWithComputedProperties;
                continue;
            }
            let text = get_property_name_from_type(&expr_type);
            let mut flags = SymbolFlags::Property;
            if initializer(e).is_some() {
                flags |= SymbolFlags::Optional;
            }
            let symbol = self.new_symbol(flags, &text);
            let resolved = self.get_type_from_binding_element(
                e,
                include_pattern_in_type,
                report_errors,
            );
            self.value_symbol_links
                .get_or_default(&symbol)
                .resolved_type = Some(resolved);
            members.insert(symbol.name.clone(), symbol);
        }
        let index_infos = string_index_info.into_iter().collect::<Vec<_>>();
        let mut result = self.new_object_type(ObjectFlags::Anonymous, None);
        self.set_structured_type_members(&result, Some(members), Vec::new(), Vec::new(), index_infos);
        if let Some(t_mut) = Arc::get_mut(&mut result) {
            t_mut.object_flags |= object_flags;
        }
        if include_pattern_in_type {
            pattern_for_type_insert(Arc::clone(&result), Arc::clone(pattern));
            if let Some(t_mut) = Arc::get_mut(&mut result) {
                t_mut.object_flags |= ObjectFlags::ContainsObjectOrArrayLiteral;
            }
        }
        result
    }

    pub(crate) fn get_type_from_array_binding_pattern(
        &mut self,
        pattern: &Arc<Node>,
        include_pattern_in_type: bool,
        report_errors: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_array_binding_pattern"); 
        let element_list = elements(pattern);
        let last_element = element_list.last().cloned();
        let rest_element = last_element.filter(|e| is_binding_element(e) && has_dot_dot_dot_token(e));
        if element_list.is_empty() || (element_list.len() == 1 && rest_element.is_some()) {
            if self.language_version >= ScriptTarget::ES2015 {
                return self.create_iterable_type(&self.any_type());
            }
            return self.any_array_type();
        }
        let min_length = element_list
            .iter()
            .rposition(|e| {
                !(rest_element.as_ref().is_some_and(|r| Arc::ptr_eq(r, e))
                    || e.name().is_none()
                    || self.has_default_value(e))
            })
            .map(|i| i + 1)
            .unwrap_or(0);
        let mut element_types: Vec<Arc<Type>> = Vec::with_capacity(element_list.len());
        let mut element_infos: Vec<TupleElementInfo> = Vec::with_capacity(element_list.len());
        for (i, e) in element_list.iter().enumerate() {
            let t = if e.name().is_none() {
                self.any_type()
            } else {
                self.get_type_from_binding_element(e, include_pattern_in_type, report_errors)
            };
            let flags = if rest_element.as_ref().is_some_and(|r| Arc::ptr_eq(r, e)) {
                ElementFlags::Rest
            } else if i >= min_length {
                ElementFlags::Optional
            } else {
                ElementFlags::Required
            };
            element_types.push(t);
            element_infos.push(TupleElementInfo {
                label: None,
                flags,
                labeled_declaration: None,
                type_: None,
            });
        }
        let mut result = self.create_tuple_type_ex(element_types, element_infos, false);
        if include_pattern_in_type {
            result = self.clone_type_reference(&result);
            pattern_for_type_insert(Arc::clone(&result), Arc::clone(pattern));
            if let Some(t_mut) = Arc::get_mut(&mut result) {
                t_mut.object_flags |= ObjectFlags::ContainsObjectOrArrayLiteral;
            }
        }
        result
    }

    pub(crate) fn get_type_from_binding_element(
        &mut self,
        element: &Arc<Node>,
        include_pattern_in_type: bool,
        report_errors: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_from_binding_element"); 
        if let Some(_initializer) = initializer(element) {
            let mut contextual_type = self.unknown_type();
            if let Some(name) = get_name_of_declaration(element) {
                if is_binding_pattern(&name) {
                    contextual_type = self.get_type_from_binding_pattern(&name, true, false);
                }
            }
            let checked =
                self.check_declaration_initializer(element, CheckMode::Normal, Some(&contextual_type));
            let widened = self.get_widened_literal_type_for_initializer(element, &checked);
            return self.add_optionality(&widened);
        }
        if let Some(name) = get_name_of_declaration(element) {
            if is_binding_pattern(&name) {
                return self.get_type_from_binding_pattern(
                    &name,
                    include_pattern_in_type,
                    report_errors,
                );
            }
        }
        if report_errors && !self.declaration_belongs_to_private_ambient_member(element) {
            self.report_implicit_any(element, &self.any_type(), WideningKind::Normal);
        }
        if include_pattern_in_type {
            return self.non_inferrable_any_type();
        }
        self.any_type()
    }
}
