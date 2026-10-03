#![allow(unused_imports)]
use tsox_core::diagnostics::{Message, messages_generated::*};

use crate::checker::utilities_is_optional_symbol::is_type_any;
use crate::checker::mig::w9a::new_type_mapper;
use crate::checker::string_mapping_checker::{string_mapping_kind, StringMappingKind};
use tsox_frontend::ast::mig::m3e_4::JsDeclarationKind;
use crate::checker::relater_recursion_identity::RecursionIdentity as RecursionId;

use tsox_frontend::ast::utilities::is_in_js_file;
use tsox_frontend::ast::utilities::is_class_element;
use tsox_frontend::ast::utilities::is_statement;
use tsox_frontend::ast::node_data_generated::{
    is_parameter_declaration, is_conditional_type_node, is_mapped_type_node,
    is_binary_expression, is_call_expression,
};
#[path = "r24k17_defs.rs"]
pub mod r24k17_defs;
pub use r24k17_defs::*;

use tsox_frontend::ast::mig::m3f_2::has_abstract_modifier;
use tsox_frontend::ast::mig::m3f::get_right_most_assigned_expression;
use tsox_frontend::ast::mig::m3e_4::get_assignment_declaration_kind;
use super::wc1b::{every_type, walk_up_parenthesized_expressions, class_or_constructor_parameter_is_decorated, is_tuple_type, is_object_literal_type, is_array_or_tuple_type, is_literal_type, is_initialized_property};
use crate::checker::exports_union_reduction::get_declaration_modifier_flags_from_symbol;
use crate::checker::mig::wc3_3::is_generic_tuple_type;
use crate::checker::mig::wc3::r18k4_node_ext::NodeAccessExt;
use crate::checker::types_type_flags_instantiable_non_primitive::SIGNATURE_FLAGS_PROPAGATING_FLAGS;

use crate::checker::checker_checker::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

#[path = "r23k4_defs.rs"]
pub mod r23k4_defs;
pub use r23k4_defs::*;

pub(crate) struct WideningContext {
    pub parent: Option<std::sync::Weak<WideningContext>>,
    pub property_name: String,
    pub siblings: Option<Vec<Arc<Type>>>,
    pub resolved_properties: Option<Vec<Arc<Symbol>>>,
    pub child_contexts: RefCell<HashMap<String, Arc<WideningContext>>>,
    pub widened_types: RefCell<HashMap<TypeId, Arc<Type>>>,
}

impl WideningContext {
    pub fn get_child_context(self: &Arc<Self>, property_name: &str) -> Arc<WideningContext> { ::tsox_core::fntrace::enter("get_child_context"); 
        if let Some(cached) = self.child_contexts.borrow().get(property_name) {
            return Arc::clone(cached);
        }
        let result = Arc::new(WideningContext {
            parent: Some(Arc::downgrade(self)),
            property_name: property_name.to_string(),
            siblings: None,
            resolved_properties: None,
            child_contexts: RefCell::new(HashMap::new()),
            widened_types: RefCell::new(HashMap::new()),
        });
        self.child_contexts
            .borrow_mut()
            .insert(property_name.to_string(), Arc::clone(&result));
        result
    }
}

impl Checker {
    pub fn check_member_for_override_modifier_worker(
        &mut self,
        node: &Arc<Node>,
        static_type: &Arc<Type>,
        base_static_type: &Arc<Type>,
        base_with_this: Option<&Arc<Type>>,
        t: &Arc<Type>,
        type_with_this: &Arc<Type>,
        member_has_override_modifier: bool,
        member_has_abstract_modifier: bool,
        member_is_static: bool,
        member_is_parameter_property: bool,
        member: &Arc<Symbol>,
        error_node: Option<&Arc<Node>>,
    ) -> MemberOverrideStatus {
        let is_js = is_in_js_file(node);
        if member_has_override_modifier
            && member.value_declaration.is_some()
            && member
                .value_declaration
                .as_ref()
                .is_some_and(|d| is_class_element(d))
            && member
                .value_declaration
                .as_ref()
                .and_then(|d| d.name())
                .is_some_and(|n| self.is_non_bindable_dynamic_name(&n))
        {
            if let Some(error_node) = error_node {
                if is_js {
                    self.error_message(
                        error_node,THIS_MEMBER_CANNOT_HAVE_A_JSDOC_COMMENT_WITH_AN_OVERRIDE_TAG_BECAUSE_ITS_NAME_IS_DYNAMIC,
                        &[],
                    );
                } else {
                    self.error_message(
                        error_node,THIS_MEMBER_CANNOT_HAVE_AN_OVERRIDE_MODIFIER_BECAUSE_ITS_NAME_IS_DYNAMIC,
                        &[],
                    );
                }
            }
            return MemberOverrideStatus::HasInvalidOverride;
        }
        if base_with_this.is_some() && (member_has_override_modifier || self.compiler_options.no_implicit_override.is_true()) {
            let this_type = if member_is_static { static_type } else { type_with_this };
            let base_type = if member_is_static {
                base_static_type
            } else {
                base_with_this.unwrap()
            };
            let prop = self.get_property_of_type(this_type, &member.name);
            let base_prop = self.get_property_of_type(base_type, &member.name);
            if prop.is_some() && base_prop.is_none() && member_has_override_modifier {
                if let Some(error_node) = error_node {
                    let suggestion =
                        self.get_suggested_symbol_for_nonexistent_class_member(&member.name, base_type);
                    if let Some(suggestion) = suggestion {
                        let base_str = self.type_to_string(base_with_this.unwrap());
                        let sug_str = self.symbol_to_string(&suggestion);
                        if is_js {
                            self.error_message(error_node,THIS_MEMBER_CANNOT_HAVE_A_JSDOC_COMMENT_WITH_AN_OVERRIDE_TAG_BECAUSE_IT_IS_NOT_DECLARED_IN_THE_BASE_CLASS_0_DID_YOU_MEAN_1, &[base_str, sug_str]);
                        } else {
                            self.error_message(error_node,THIS_MEMBER_CANNOT_HAVE_AN_OVERRIDE_MODIFIER_BECAUSE_IT_IS_NOT_DECLARED_IN_THE_BASE_CLASS_0_DID_YOU_MEAN_1, &[base_str, sug_str]);
                        }
                    } else {
                        let base_str = self.type_to_string(base_with_this.unwrap());
                        if is_js {
                            self.error_message(error_node,THIS_MEMBER_CANNOT_HAVE_A_JSDOC_COMMENT_WITH_AN_OVERRIDE_TAG_BECAUSE_IT_IS_NOT_DECLARED_IN_THE_BASE_CLASS_0, &[base_str]);
                        } else {
                            self.error_message(error_node,THIS_MEMBER_CANNOT_HAVE_AN_OVERRIDE_MODIFIER_BECAUSE_IT_IS_NOT_DECLARED_IN_THE_BASE_CLASS_0, &[base_str]);
                        }
                    }
                }
                return MemberOverrideStatus::HasInvalidOverride;
            }
            if prop.is_some()
                && base_prop.is_some()
                && base_prop
                    .as_ref()
                    .is_some_and(|p| !p.declarations.is_empty())
                && self.compiler_options.no_implicit_override.is_true()
                && !node.flags.intersects(NodeFlags::Ambient)
            {
                let base_prop = base_prop.unwrap();
                let base_has_abstract = base_prop
                    .declarations
                    .iter()
                    .any(|d| has_abstract_modifier(d));
                if member_has_override_modifier {
                    return MemberOverrideStatus::None;
                }
                if !base_has_abstract {
                    if let Some(error_node) = error_node {
                        let base_str = self.type_to_string(base_with_this.unwrap());
                        if member_is_parameter_property {
                            if is_js {
                                self.error_message(error_node,THIS_PARAMETER_PROPERTY_MUST_HAVE_A_JSDOC_COMMENT_WITH_AN_OVERRIDE_TAG_BECAUSE_IT_OVERRIDES_A_MEMBER_IN_THE_BASE_CLASS_0, &[base_str]);
                            } else {
                                self.error_message(error_node,THIS_PARAMETER_PROPERTY_MUST_HAVE_AN_OVERRIDE_MODIFIER_BECAUSE_IT_OVERRIDES_A_MEMBER_IN_BASE_CLASS_0, &[base_str]);
                            }
                        } else if is_js {
                            self.error_message(error_node,THIS_MEMBER_MUST_HAVE_A_JSDOC_COMMENT_WITH_AN_OVERRIDE_TAG_BECAUSE_IT_OVERRIDES_A_MEMBER_IN_THE_BASE_CLASS_0, &[base_str]);
                        } else {
                            self.error_message(error_node,THIS_MEMBER_MUST_HAVE_AN_OVERRIDE_MODIFIER_BECAUSE_IT_OVERRIDES_A_MEMBER_IN_THE_BASE_CLASS_0, &[base_str]);
                        }
                    }
                    return MemberOverrideStatus::NeedsOverride;
                }
                if member_has_abstract_modifier {
                    if let Some(error_node) = error_node {
                        let base_str = self.type_to_string(base_with_this.unwrap());
                        self.error_message(error_node,THIS_MEMBER_MUST_HAVE_AN_OVERRIDE_MODIFIER_BECAUSE_IT_OVERRIDES_AN_ABSTRACT_METHOD_THAT_IS_DECLARED_IN_THE_BASE_CLASS_0, &[base_str]);
                    }
                    return MemberOverrideStatus::NeedsOverride;
                }
            }
        } else if member_has_override_modifier {
            if let Some(error_node) = error_node {
                let t_str = self.type_to_string(t);
                if is_js {
                    self.error_message(error_node,THIS_MEMBER_CANNOT_HAVE_A_JSDOC_COMMENT_WITH_AN_OVERRIDE_TAG_BECAUSE_ITS_CONTAINING_CLASS_0_DOES_NOT_EXTEND_ANOTHER_CLASS, &[t_str]);
                } else {
                    self.error_message(error_node,THIS_MEMBER_CANNOT_HAVE_AN_OVERRIDE_MODIFIER_BECAUSE_ITS_CONTAINING_CLASS_0_DOES_NOT_EXTEND_ANOTHER_CLASS, &[t_str]);
                }
            }
            return MemberOverrideStatus::HasInvalidOverride;
        }
        MemberOverrideStatus::None
    }

    pub fn get_suggested_symbol_for_nonexistent_class_member(
        &mut self,
        name: &str,
        base_type: &Arc<Type>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_suggested_symbol_for_nonexistent_class_member"); 
        let properties = self.get_properties_of_type(base_type);
        self.get_spelling_suggestion_for_name(name, properties, SymbolFlags::CLASS_MEMBER)
    }

    pub fn combine_union_or_intersection_member_signatures(
        &mut self,
        left: &Arc<Signature>,
        right: &Arc<Signature>,
        is_union: bool,
    ) -> Arc<Signature> { ::tsox_core::fntrace::enter("combine_union_or_intersection_member_signatures"); 
        let mut type_params: Vec<Arc<Type>> = left.type_parameters.clone();
        if type_params.is_empty() {
            type_params = right.type_parameters.clone();
        }
        let mut param_mapper: Option<Arc<TypeMapper>> = None;
        if !left.type_parameters.is_empty() && !right.type_parameters.is_empty() {
            param_mapper = Some(Arc::new(new_type_mapper(right.type_parameters.clone(), left.type_parameters.clone())));
        }
        let mut flags = (left.flags | right.flags)
            & (SIGNATURE_FLAGS_PROPAGATING_FLAGS & !SignatureFlags::HasRestParameter);
        let declaration = left.declaration.clone();
        let params = self.combine_union_or_intersection_parameters(left, right, param_mapper.as_ref(), is_union);
        if params
            .last()
            .is_some_and(|p| p.check_flags.intersects(CheckFlags::RestParameter))
        {
            flags |= SignatureFlags::HasRestParameter;
        }
        let this_param = self.combine_union_or_intersection_this_param(
            left.this_parameter.as_ref(),
            right.this_parameter.as_ref(),
            param_mapper.as_ref(),
            is_union,
        );
        let min_arg_count = left.min_argument_count.max(right.min_argument_count);
        let mut result = Signature::new();
        result.flags = flags;
        result.declaration = declaration;
        result.type_parameters = type_params;
        result.this_parameter = this_param;
        result.parameters = params;
        result.min_argument_count = min_arg_count;
        let mut result = Arc::new(result);
        let left_signatures: Vec<Arc<Signature>> = match signature_composite(left) {
            Some(composite) if composite.is_union == true => composite.signatures.clone(),
            _ => vec![Arc::clone(left)],
        };
        {
            let mut signatures = left_signatures;
            signatures.push(Arc::clone(right));
            set_signature_composite(
                &result,
                CompositeSignature { is_union, signatures },
            );
        }
        if let Some(param_mapper) = &param_mapper {
            let left_composite = signature_composite(left);
            if left_composite.as_ref().is_some_and(|c| c.is_union == is_union) && left.mapper.is_some() {
                let mapper = self.combine_type_mappers(left.mapper.as_ref(), Some(param_mapper));
                Arc::get_mut(&mut result).unwrap().mapper = mapper;
            } else {
                Arc::get_mut(&mut result).unwrap().mapper = Some(Arc::clone(param_mapper));
            }
        } else if signature_composite(left)
            .as_ref()
            .is_some_and(|c| c.is_union == is_union)
        {
            Arc::get_mut(&mut result).unwrap().mapper = left.mapper.clone();
        }
        result
    }

    pub fn combine_union_or_intersection_this_param(
        &mut self,
        left: Option<&Arc<Symbol>>,
        right: Option<&Arc<Symbol>>,
        mapper: Option<&Arc<TypeMapper>>,
        is_union: bool,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("combine_union_or_intersection_this_param"); 
        let left = match left {
            Some(l) => l,
            None => return right.cloned(),
        };
        let right = match right {
            Some(r) => r,
            None => return Some(Arc::clone(left)),
        };
        let right_type = self.get_type_of_symbol(right);
        let instantiated = self.instantiate_type(&right_type, mapper);
        let left_type = self.get_type_of_symbol(left);
        let this_type = self.get_union_or_intersection_type(
            vec![left_type, instantiated],
            !is_union,
            UnionReduction::Literal,
        );
        Some(self.create_symbol_with_type(left, &this_type))
    }

    pub fn combine_union_or_intersection_parameters(
        &mut self,
        left: &Arc<Signature>,
        right: &Arc<Signature>,
        mapper: Option<&Arc<TypeMapper>>,
        is_union: bool,
    ) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("combine_union_or_intersection_parameters"); 
        let left_count = self.get_parameter_count(left);
        let right_count = self.get_parameter_count(right);
        let (longest_count, longest, shorter) = if left_count >= right_count {
            (left_count, left, right)
        } else {
            (right_count, right, left)
        };
        let either_has_effective_rest = self.has_effective_rest_parameter(left)
            || self.has_effective_rest_parameter(right);
        let needs_extra_rest_element =
            either_has_effective_rest && !self.has_effective_rest_parameter(longest);
        let mut params: Vec<Arc<Symbol>> = Vec::with_capacity(longest_count + usize::from(needs_extra_rest_element));
        for i in 0..longest_count {
            let mut longest_param_type = self.try_get_type_at_position(longest, i).unwrap_or_else(|| self.unknown_type());
            if std::ptr::eq(longest, right) {
                longest_param_type = self.instantiate_type(&longest_param_type, mapper);
            }
            let mut shorter_param_type = self.try_get_type_at_position(shorter, i).unwrap_or_else(|| self.unknown_type());
            if std::ptr::eq(shorter, right) {
                shorter_param_type = self.instantiate_type(&shorter_param_type, mapper);
            }
            let combined_param_type = self.get_union_or_intersection_type(
                vec![longest_param_type, shorter_param_type],
                !is_union,
                UnionReduction::Literal,
            );
            let is_rest_param = either_has_effective_rest
                && !needs_extra_rest_element
                && i == longest_count - 1;
            let is_optional = i >= self.get_min_argument_count(longest)
                && i >= self.get_min_argument_count(shorter);
            let left_name = if i < left_count {
                self.get_parameter_name_at_position(left, i)
            } else {
                String::new()
            };
            let right_name = if i < right_count {
                self.get_parameter_name_at_position(right, i)
            } else {
                String::new()
            };
            let mut param_name = if left_name == right_name {
                left_name.clone()
            } else if left_name.is_empty() {
                right_name.clone()
            } else if right_name.is_empty() {
                left_name.clone()
            } else {
                String::new()
            };
            if param_name.is_empty() {
                param_name = format!("arg{}", i);
            }
            let mut flags = SymbolFlags::FunctionScopedVariable;
            let mut check_flags = CheckFlags::None;
            if is_optional && !is_rest_param {
                flags |= SymbolFlags::Optional;
                check_flags = CheckFlags::OptionalParameter;
            }
            if is_rest_param {
                check_flags = CheckFlags::RestParameter;
            }
            let param_symbol = self.new_symbol_ex(flags, &param_name, check_flags);
            let resolved = if is_rest_param {
                Some(self.create_array_type(combined_param_type))
            } else {
                Some(combined_param_type)
            };
            let links = self.value_symbol_links.get_or_default(&param_symbol);
            links.resolved_type = resolved;
            params.push(param_symbol);
        }
        if needs_extra_rest_element {
            let rest_param_symbol = self.new_symbol_ex(
                SymbolFlags::FunctionScopedVariable,
                "args",
                CheckFlags::RestParameter,
            );
            let elem = self.get_type_at_position(shorter, longest_count);
            let mut resolved_type = self.create_array_type(elem);
            if std::ptr::eq(shorter, right) {
                resolved_type = self.instantiate_type(&resolved_type, mapper);
            }
            let links = self.value_symbol_links.get_or_default(&rest_param_symbol);
            links.resolved_type = Some(resolved_type);
            params.push(rest_param_symbol);
        }
        params
    }

    pub fn compare_properties(
        &mut self,
        source_prop: &Arc<Symbol>,
        target_prop: &Arc<Symbol>,
        compare_types: &dyn Fn(&mut Checker, &Arc<Type>, &Arc<Type>) -> Ternary,
    ) -> Ternary { ::tsox_core::fntrace::enter("compare_properties"); 
        if Arc::ptr_eq(source_prop, target_prop) {
            return Ternary::True;
        }
        let source_prop_accessibility = get_declaration_modifier_flags_from_symbol(source_prop)
            & ModifierFlags::NonPublicAccessibilityModifier;
        let target_prop_accessibility = get_declaration_modifier_flags_from_symbol(target_prop)
            & ModifierFlags::NonPublicAccessibilityModifier;
        if source_prop_accessibility != target_prop_accessibility {
            return Ternary::False;
        }
        if source_prop_accessibility != ModifierFlags::empty() {
            if !Arc::ptr_eq(
                &self.get_target_symbol(source_prop),
                &self.get_target_symbol(target_prop),
            ) {
                return Ternary::False;
            }
        } else if source_prop.flags.intersects(SymbolFlags::Optional)
            != target_prop.flags.intersects(SymbolFlags::Optional)
        {
            return Ternary::False;
        }
        if self.is_readonly_symbol(source_prop) != self.is_readonly_symbol(target_prop) {
            return Ternary::False;
        }
        let source_type = self.get_non_missing_type_of_symbol(source_prop);
        let target_type = self.get_non_missing_type_of_symbol(target_prop);
        compare_types(self, &source_type, &target_type)
    }

    pub fn compute_is_uniform_union_type(&mut self, types: &[Arc<Type>]) -> bool { ::tsox_core::fntrace::enter("compute_is_uniform_union_type"); 
        let mut enum_symbol: Option<Arc<Symbol>> = None;
        let mut has_string_or_number_literal = false;
        for t in types {
            if t.flags.intersects(TYPE_FLAGS_ENUM_LIKE) {
                if has_string_or_number_literal {
                    return false;
                }
                let parent = self.get_parent_of_symbol(t.symbol().unwrap()).unwrap();
                match &enum_symbol {
                    None => enum_symbol = Some(parent),
                    Some(prev) => {
                        if !Arc::ptr_eq(prev, &parent) {
                            return false;
                        }
                    }
                }
            } else if t.flags.intersects(TYPE_FLAGS_STRING_OR_NUMBER_LITERAL) {
                if enum_symbol.is_some() {
                    return false;
                }
                has_string_or_number_literal = true;
            }
        }
        true
    }

    pub fn create_canonical_signature(&mut self, signature: &Arc<Signature>) -> Arc<Signature> { ::tsox_core::fntrace::enter("create_canonical_signature"); 
        let type_arguments: Vec<Arc<Type>> = signature
            .type_parameters
            .iter()
            .map(|tp| {
                if tp.target().is_some()
                    && self
                        .get_constraint_of_type_parameter(tp.target().unwrap())
                        .is_none()
                {
                    return tp.target().unwrap().clone();
                }
                Arc::clone(tp)
            })
            .collect();
        self.get_signature_instantiation(signature, &type_arguments)
    }

    pub fn create_tuple_target_type(
        &mut self,
        element_infos: &[TupleElementInfo],
        readonly: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("create_tuple_target_type"); 
        let arity = element_infos.len();
        let min_length = element_infos
            .iter()
            .filter(|e| {
                e.flags
                    .intersects(ElementFlags::Required | ElementFlags::Variadic)
            })
            .count();
        let mut type_parameters: Vec<Arc<Type>> = Vec::with_capacity(arity);
        let mut members = tsox_frontend::ast::SymbolTable::new();
        let mut combined_flags = ElementFlags::None;
        for i in 0..arity {
            let type_parameter = self.new_type_parameter(None);
            type_parameters.push(type_parameter.clone());
            let flags = element_infos[i].flags;
            combined_flags |= flags;
            if !combined_flags.intersects(ELEMENT_FLAGS_VARIABLE) {
                let mut check_flags = CheckFlags::None;
                if readonly {
                    check_flags = CheckFlags::Readonly;
                }
                let mut prop_flags = SymbolFlags::Property;
                if flags.intersects(ElementFlags::Optional) {
                    prop_flags |= SymbolFlags::Optional;
                }
                let property = self.new_symbol_ex(prop_flags, &i.to_string(), check_flags);
                let links = self.value_symbol_links.get_or_default(&property);
                links.resolved_type = Some(Arc::clone(&type_parameter));
                members.insert(property.name.clone(), property);
            }
        }
        let fixed_length = members.len();
        let length_symbol = self.new_symbol_ex(
            SymbolFlags::Property,
            "length",
            if readonly {
                CheckFlags::Readonly
            } else {
                CheckFlags::None
            },
        );
        if combined_flags.intersects(ELEMENT_FLAGS_VARIABLE) {
            let resolved = self.number_type();
            let links = self.value_symbol_links.get_or_default(&length_symbol);
            links.resolved_type = Some(resolved);
        } else {
            let mut literal_types: Vec<Arc<Type>> = Vec::new();
            for i in min_length..=arity {
                literal_types.push(self.get_number_literal_type(tsox_core::jsnum::Number::from(i as i32)));
            }
            let resolved = self.get_union_type(literal_types);
            let links = self.value_symbol_links.get_or_default(&length_symbol);
            links.resolved_type = Some(resolved);
        }
        members.insert(length_symbol.name.clone(), length_symbol);
        let mut t = self.new_object_type(ObjectFlags::Tuple | ObjectFlags::Reference, None);
        let this_type = self.new_type_parameter(None);
        self.set_tuple_type_this_type(&mut t, this_type);
        let mut all_type_parameters = type_parameters;
        let tuple_this = self.get_tuple_type_this_type(&t);
        all_type_parameters.push(tuple_this);
        self.set_tuple_type_data(
            &mut t,
            all_type_parameters,
            members,
            element_infos.to_vec(),
            min_length,
            fixed_length,
            combined_flags,
            readonly,
        );
        t
    }

    pub fn expand_signature_parameters_with_tuple_members(
        &mut self,
        signature: &Arc<Signature>,
        rest_type: &Arc<Type>,
        rest_index: usize,
        rest_symbol: &Arc<Symbol>,
    ) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("expand_signature_parameters_with_tuple_members"); 
        let element_types = self.get_type_arguments(rest_type);
        let element_infos = rest_type.target_tuple_type().unwrap().element_infos.clone();
        let associated_names =
            self.get_uniq_associated_names_from_tuple_type(rest_type, rest_symbol);
        let mut expanded: Vec<Arc<Symbol>> = signature.parameters[..rest_index].to_vec();
        expanded.reserve(element_types.len());
        for (i, t) in element_types.iter().enumerate() {
            let flags = element_infos[i].flags;
            let check_flags = if flags.intersects(ELEMENT_FLAGS_VARIABLE) {
                CheckFlags::RestParameter
            } else if flags.intersects(ElementFlags::Optional) {
                CheckFlags::OptionalParameter
            } else {
                CheckFlags::None
            };
            let symbol = self.new_symbol_ex(
                SymbolFlags::FunctionScopedVariable,
                &associated_names[i],
                check_flags,
            );
            let resolved = if flags.intersects(ElementFlags::Rest) {
                Some(self.create_array_type(Arc::clone(t)))
            } else {
                Some(Arc::clone(t))
            };
            let links = self.value_symbol_links.get_or_default(&symbol);
            links.resolved_type = resolved;
            expanded.push(symbol);
        }
        expanded
    }

    pub fn extract_redundant_template_literals(
        &mut self,
        mut types: Vec<Arc<Type>>,
    ) -> (Vec<Arc<Type>>, bool) { ::tsox_core::fntrace::enter("extract_redundant_template_literals"); 
        let literals: Vec<Arc<Type>> = types
            .iter()
            .filter(|t| t.flags.intersects(TypeFlags::StringLiteral))
            .cloned()
            .collect();
        let mut i = types.len();
        while i > 0 {
            i -= 1;
            let t = Arc::clone(&types[i]);
            if !t
                .flags
                .intersects(TypeFlags::TemplateLiteral | TypeFlags::StringMapping)
            {
                continue;
            }
            let mut removed = false;
            for t2 in &literals {
                if self.is_type_subtype_of(t2, &t) {
                    types.remove(i);
                    removed = true;
                    break;
                }
                if self.is_pattern_literal_type(&t) {
                    return (types, true);
                }
            }
            let _ = removed;
        }
        (types, false)
    }

    pub fn filter_types(&mut self, types: &mut [Arc<Type>], predicate: &mut dyn FnMut(&Arc<Type>) -> bool) { ::tsox_core::fntrace::enter("filter_types"); 
        for t in types.iter_mut() {
            *t = self.filter_type(t, predicate);
        }
    }

    pub fn filter_type(
        &mut self,
        t: &Arc<Type>,
        f: &mut dyn FnMut(&Arc<Type>) -> bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("filter_type"); 
        if t.flags.intersects(TypeFlags::Union) {
            let types = t.types().unwrap_or(&[]).to_vec();
            let filtered: Vec<Arc<Type>> = types.iter().filter(|u| f(u)).cloned().collect();
            let same = types.len() == filtered.len()
                && types
                    .iter()
                    .zip(filtered.iter())
                    .all(|(a, b)| Arc::ptr_eq(a, b));
            if same {
                return Arc::clone(t);
            }
            let origin = t.as_union_type().and_then(|u| u.origin.clone());
            let mut new_origin: Option<Arc<Type>> = None;
            if let Some(origin) = &origin {
                if origin.flags.intersects(TypeFlags::Union) {
                    let origin_types = origin.types().unwrap_or(&[]).to_vec();
                    let origin_filtered: Vec<Arc<Type>> = origin_types
                        .iter()
                        .filter(|u| u.flags.intersects(TypeFlags::Union) || f(u))
                        .cloned()
                        .collect();
                    if origin_types.len() - origin_filtered.len() == types.len() - filtered.len() {
                        if origin_filtered.len() == 1 {
                            return Arc::clone(&origin_filtered[0]);
                        }
                        new_origin = Some(self.new_union_type(ObjectFlags::None, &origin_filtered));
                    }
                }
            }
            let object_flags = t.object_flags
                & (ObjectFlags::PrimitiveUnion | ObjectFlags::ContainsIntersections);
            return self.get_union_type_from_sorted_list(
                filtered,
                object_flags,
                None,
                new_origin.as_ref(),
            );
        }
        if t.flags.intersects(TypeFlags::Never) || f(t) {
            return Arc::clone(t);
        }
        self.never_type()
    }

    pub fn get_awaited_type_no_alias_ex(
        &mut self,
        t: &Arc<Type>,
        error_node: Option<&Arc<Node>>,
        diagnostic_message: Option<&'static Message>,
        args: &[&str],
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_awaited_type_no_alias_ex"); 
        if is_type_any(t) {
            return Some(Arc::clone(t));
        }
        if self.is_awaited_type_instantiation(t) {
            return Some(Arc::clone(t));
        }
        let key = CachedTypeKey {
            kind: CachedTypeKind::AwaitedType,
            type_id: t.id(),
        };
        if let Some(awaited_type) = self.cached_types.get(&key) {
            return Some(Arc::clone(awaited_type));
        }
        if t.flags.intersects(TypeFlags::Union) {
            if self.awaited_type_stack.iter().any(|u| Arc::ptr_eq(u, t)) {
                if let Some(error_node) = error_node {
                    self.error_message(error_node,TYPE_IS_REFERENCED_DIRECTLY_OR_INDIRECTLY_IN_THE_FULFILLMENT_CALLBACK_OF_ITS_OWN_THEN_METHOD, &[]);
                }
                return None;
            }
            self.awaited_type_stack.push(Arc::clone(t));
            let checker_ptr: *mut Checker = self;
            let mapped = unsafe {
                (*checker_ptr).map_type(t, &mut |t: &Arc<Type>| {
                    (*checker_ptr).get_awaited_type_no_alias_ex(t, error_node, diagnostic_message, args)
                })
            };
            self.awaited_type_stack.pop();
            if let Some(m) = &mapped {
                self.cached_types.insert(key, Arc::clone(m));
            }
            return mapped;
        }
        if self.is_awaited_type_needed(t) {
            self.cached_types.insert(key, Arc::clone(t));
            return Some(Arc::clone(t));
        }
        let mut this_type_for_error: Option<Arc<Type>> = None;
        let promised_type =
            self.get_promised_type_of_promise_ex(t, None, Some(&mut this_type_for_error));
        if let Some(promised_type) = promised_type {
            let is_bad_actor = Arc::ptr_eq(t, &promised_type)
                || self
                    .awaited_type_stack
                    .iter()
                    .any(|u| Arc::ptr_eq(u, &promised_type));
            if is_bad_actor {
                if let Some(error_node) = error_node {
                    self.error_message(error_node,TYPE_IS_REFERENCED_DIRECTLY_OR_INDIRECTLY_IN_THE_FULFILLMENT_CALLBACK_OF_ITS_OWN_THEN_METHOD, &[]);
                }
                return None;
            }
            self.awaited_type_stack.push(Arc::clone(t));
            let awaited_type =
                self.get_awaited_type_no_alias_ex(&promised_type, error_node, diagnostic_message, args);
            self.awaited_type_stack.pop();
            let awaited_type = awaited_type?;
            self.cached_types.insert(key, Arc::clone(&awaited_type));
            return Some(awaited_type);
        }
        if self.is_thenable_type(t) {
            if let Some(error_node) = error_node {
                if let Some(message) = diagnostic_message {
                    let message_args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                    self.error_message(error_node, *message, &message_args);
                }
            }
            return None;
        }
        self.cached_types.insert(key, Arc::clone(t));
        Some(Arc::clone(t))
    }

    pub fn get_conditional_flow_type_of_type(&mut self, t: &Arc<Type>, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_conditional_flow_type_of_type"); 
        let mut constraints: Vec<Arc<Type>> = Vec::new();
        let mut covariant = true;
        let mut node = Some(Arc::clone(node));
        while let Some(current) = node {
            if is_statement(&current) || current.kind == SyntaxKind::JSDoc {
                break;
            }
            let parent = match current.parent() {
                Some(p) => p,
                None => break,
            };
            if is_parameter_declaration(&parent) {
                covariant = !covariant;
            }
            if (covariant || t.flags.intersects(TYPE_FLAGS_TYPE_VARIABLE))
                && is_conditional_type_node(&parent)
                && Arc::ptr_eq(
                    &current,
                    &parent.as_conditional_type_node().true_type,
                )
            {
                let constraint = self.get_implied_constraint(
                    t,
                    &parent.as_conditional_type_node().check_type,
                    &parent.as_conditional_type_node().extends_type,
                );
                if let Some(constraint) = constraint {
                    constraints.push(constraint);
                }
            } else if t.flags.intersects(TypeFlags::TypeParameter)
                && is_mapped_type_node(&parent)
                && parent.as_mapped_type_node().name_type.is_none()
                && parent.type_node().is_some_and(|tn| Arc::ptr_eq(&current, tn))
            {
                let mapped_type = self.get_type_from_type_node(&parent);
                let from_mapped = self.get_type_parameter_from_mapped_type(&mapped_type);
                let actual = self.get_actual_type_variable(t);
                if from_mapped.is_some_and(|tp| Arc::ptr_eq(&tp, &actual)) {
                    if let Some(type_parameter) = self.get_homomorphic_type_variable(&mapped_type) {
                        if let Some(constraint) = self.get_constraint_of_type_parameter(&type_parameter) {
                            if every_type(&constraint, &|n: &Arc<Type>| is_array_or_tuple_type(n)) {
                                let number_type = self.number_type();
                                let numeric_string_type = self.numeric_string_type();
                                constraints.push(self.get_union_type(vec![
                                    number_type,
                                    numeric_string_type,
                                ]));
                            }
                        }
                    }
                }
            }
            node = Some(parent);
        }
        if !constraints.is_empty() {
            let intersection = self.get_intersection_type(constraints);
            return self.get_substitution_type(t, &intersection);
        }
        Arc::clone(t)
    }

    pub fn compute_base_constraint(&mut self, t: &Arc<Type>, stack: Vec<RecursionId>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("compute_base_constraint"); 
        if t.flags.intersects(TypeFlags::TypeParameter) {
            let constraint = self.get_constraint_from_type_parameter(t);
            if t.as_type_parameter().unwrap().is_this_type {
                return Some(constraint);
            }
            return self.get_next_base_constraint(&constraint, stack.as_slice());
        }
        if t.flags.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION) {
            let types = t.types().unwrap_or(&[]).to_vec();
            let mut constraints: Vec<Arc<Type>> = Vec::with_capacity(types.len());
            let mut different = false;
            for s in &types {
                let constraint = self.get_next_base_constraint(s, stack.clone().as_slice());
                match constraint {
                    Some(constraint) => {
                        if !Arc::ptr_eq(&constraint, s) {
                            different = true;
                        }
                        constraints.push(constraint);
                    }
                    None => different = true,
                }
            }
            if !different {
                return Some(Arc::clone(t));
            }
            if t.flags.intersects(TypeFlags::Union) && constraints.len() == types.len() {
                return Some(self.get_union_type(constraints));
            }
            if t.flags.intersects(TypeFlags::Intersection) && !constraints.is_empty() {
                return Some(self.get_intersection_type(constraints));
            }
            return None;
        }
        if t.flags.intersects(TypeFlags::Index) {
            let target = t.as_index_type().unwrap().target.clone().unwrap();
            if self.is_generic_mapped_type(&target) {
                if self.get_name_type_from_mapped_type(&target).is_some()
                    && !Checker::is_mapped_type_with_keyof_constraint_declaration(&target)
                {
                    let index_type = self.get_index_type_for_mapped_type(&target, IndexFlags::None);
                    return self.get_next_base_constraint(&index_type, stack.as_slice());
                }
            }
            return Some(self.string_number_symbol_type());
        }
        if t.flags.intersects(TypeFlags::TemplateLiteral) {
            let types = t.types().unwrap_or(&[]).to_vec();
            let mut constraints: Vec<Arc<Type>> = Vec::with_capacity(types.len());
            for s in &types {
                if let Some(constraint) = self.get_next_base_constraint(s, stack.clone().as_slice()) {
                    constraints.push(constraint);
                }
            }
            if constraints.len() == types.len() {
                let texts = t.as_template_literal_type().unwrap().texts.clone();
                return Some(self.get_template_literal_type(&texts, &constraints));
            }
            return Some(self.string_type());
        }
        if t.flags.intersects(TypeFlags::StringMapping) {
            let target = t.target().cloned().unwrap_or_else(|| Arc::clone(t));
            let constraint = self.get_next_base_constraint(&target, stack.as_slice());
            if let Some(constraint) = constraint {
                if !Arc::ptr_eq(&constraint, &target) {
                    let symbol = t.symbol().unwrap();
                    let kind =
                        string_mapping_kind(&symbol.name).unwrap_or(StringMappingKind::Uppercase);
                    return Some(
                        self.get_string_mapping_type(kind, Some(Arc::clone(symbol)), &constraint),
                    );
                }
            }
            return Some(self.string_type());
        }
        if t.flags.intersects(TypeFlags::IndexedAccess) {
            if self.is_mapped_type_generic_indexed_access(t) {
                let object_type = t.as_indexed_access_type().unwrap().object_type.clone().unwrap();
                let index_type = t.as_indexed_access_type().unwrap().index_type.clone().unwrap();
                let substituted = self.substitute_indexed_mapped_type(&object_type, &index_type);
                return self.get_next_base_constraint(&substituted, stack.as_slice());
            }
            let object_type = t.as_indexed_access_type().unwrap().object_type.clone().unwrap();
            let index_type = t.as_indexed_access_type().unwrap().index_type.clone().unwrap();
            let base_object_type = self.get_next_base_constraint(&object_type, stack.clone().as_slice())?;
            let base_index_type = self.get_next_base_constraint(&index_type, stack.clone().as_slice())?;
            let access_flags = t.as_indexed_access_type().unwrap().access_flags;
            let indexed = self.get_indexed_access_type_or_undefined(
                &base_object_type,
                &base_index_type,
                access_flags,
                None,
                None,
            )?;
            return self.get_next_base_constraint(&indexed, stack.as_slice());
        }
        if t.flags.intersects(TypeFlags::Conditional) {
            if self.conditional_constraint_depth >= 100 {
                return None;
            }
            self.conditional_constraint_depth += 1;
            let constraint = self.get_constraint_from_conditional_type(t);
            self.conditional_constraint_depth -= 1;
            if let Some(constraint) = constraint {
                return self.get_next_base_constraint(&constraint, stack.as_slice());
            }
            return None;
        }
        if t.flags.intersects(TypeFlags::Substitution) {
            let intersection = self.get_substitution_intersection(t);
            return self.get_next_base_constraint(&intersection, stack.as_slice());
        }
        if is_generic_tuple_type(t) {
            let element_types = self.get_element_types(t);
            let element_infos = t.target_tuple_type().unwrap().element_infos.clone();
            let mut new_elements: Vec<Arc<Type>> = Vec::with_capacity(element_types.len());
            for (i, v) in element_types.iter().enumerate() {
                let mut new_element = Arc::clone(v);
                if v.flags.intersects(TypeFlags::TypeParameter)
                    && element_infos[i].flags.intersects(ElementFlags::Variadic)
                {
                    if let Some(constraint) = self.get_next_base_constraint(v, stack.clone().as_slice()) {
                        if !Arc::ptr_eq(&constraint, v)
                            && every_type(&constraint, &|n: &Arc<Type>| {
                                is_array_or_tuple_type(n) && !is_generic_tuple_type(n)
                            })
                        {
                            new_element = constraint;
                        }
                    }
                }
                new_elements.push(new_element);
            }
            let readonly = t.target_tuple_type().unwrap().readonly;
            return Some(self.create_tuple_type_ex(new_elements, element_infos, readonly));
        }
        Some(Arc::clone(t))
    }


    pub fn get_assignment_declaration_initializer_type(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_assignment_declaration_initializer_type"); 
        if is_binary_expression(node) {
            let t: Arc<Type>;
            match get_assignment_declaration_kind(node) {
                JsDeclarationKind::ModuleExports => {
                    let right = get_right_most_assigned_expression(node);
                    let checked = self.check_expression_cached(&right);
                    t = self.get_regular_type_of_literal_type(&checked);
                }
                JsDeclarationKind::ThisProperty => {
                    let binary = node.as_binary_expression();
                    if self.contains_same_named_this_property(&binary.left, &binary.right) {
                        return None;
                    }
                    let right = binary.right.clone();
                    t = self.check_expression_for_mutable_location(&right, CheckMode::Normal);
                }
                _ => {
                    let right = node.as_binary_expression().right.clone();
                    t = self.check_expression_for_mutable_location(&right, CheckMode::Normal);
                }
            }
            if self.is_empty_array_literal_type(&t)
                && !self.get_symbol_of_node(node).map(|s| self.has_parent_with_type_annotation(&s)).unwrap_or(false)
            {
                let any_array = self.any_array_type();
                self.report_implicit_any(node, &any_array, WideningKind::Normal);
                return Some(self.any_array_type());
            }
            return Some(t);
        }
        if is_call_expression(node) {
            let args = node.arguments().map(|l| l.nodes.as_slice()).unwrap_or(&[]);
            return Some(self.get_type_from_property_descriptor(&args[2]));
        }
        None
    }

    pub fn numeric_string_type(&mut self) -> Arc<Type> { ::tsox_core::fntrace::enter("numeric_string_type"); 
        self.get_union_type(vec![self.number_type(), self.string_type()])
    }

    pub fn get_constraint_from_type_parameter(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_constraint_from_type_parameter"); 
        if let Some(constraint) = t.as_type_parameter().unwrap().constraint.clone() {
            return constraint;
        }
        self.unknown_type()
    }
}
