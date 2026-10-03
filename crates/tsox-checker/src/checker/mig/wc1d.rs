#![allow(unused_imports)]

use crate::checker::checker::*;
use super::r19k5_flags_ext::TypeFlagsExt;
use super::wc3::ReferenceHint;
use crate::checker::types::*;
use std::collections::HashMap;
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated as msgs;
use tsox_frontend::ast::{self, Node, NodeData, Symbol, SyntaxKind};

use super::m2h::r21k10_defs::{find_constructor_declaration, get_class_like_declaration_of_symbol};
use super::m2a::is_prototype_property;
use super::m3a_2::get_containing_class_excluding_class_decorators;
use super::m2c::r18k3_defs::LanguageFeatureMinimumTarget::{
    ClassAndClassElementDecorators as CLASS_AND_CLASS_ELEMENT_DECORATORS,
    PrivateNamesAndClassStaticBlocks as PRIVATE_NAMES_AND_CLASS_STATIC_BLOCKS,
};
use super::wc1c::r24k17_defs::is_js_literal_type;
use tsox_frontend::ast::mig::m3g_2::is_plain_js_file;

use super::wc1b::is_type_any;
use super::wc3_2::is_const_enum_object_type;
use crate::checker::checker_resolve_access::is_write_only_access;
use crate::checker::exports_union_reduction::get_declaration_modifier_flags_from_symbol;
use crate::checker::utilities_get_assignment_target::is_delete_target;
use crate::checker::utilities_is_optional_symbol::is_exclamation_token;
use crate::checker::utilities_token_is_identifier_or_keyword::{
    get_property_name_from_type, is_type_usable_as_property_name,
};
use tsox_frontend::ast::{is_in_js_file, is_in_json_file};
use tsox_frontend::ast::mig::m3b::is_write_access;

struct MemberInfo {
    error_node: Option<Arc<Node>>,
    missed_properties: Vec<String>,
    base_type_name: String,
    type_name: String,
}

impl Checker {
    pub fn check_property_access_expression_or_qualified_name(
        &mut self,
        node: &Arc<Node>,
        left: &Arc<Node>,
        left_type: &Arc<Type>,
        right: &Arc<Node>,
        check_mode: CheckMode,
        write_only: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("check_property_access_expression_or_qualified_name"); 
        let parent_symbol = self.get_resolved_symbol_or_nil(left);
        let assignment_kind = get_assignment_target_kind(node);
        let mut widened_type = Arc::clone(left_type);
        if assignment_kind != AssignmentKind::None || self.is_method_access_for_call(node) {
            widened_type = self.get_widened_type(left_type);
        }
        let apparent_type = self.get_apparent_type(&widened_type);
        let is_any_like =
            is_type_any(&apparent_type) || Arc::ptr_eq(&apparent_type, &self.silent_never_type());
        let mut prop: Option<Arc<Symbol>> = None;
        if ast::is_private_identifier(right) {
            if self.language_version < PRIVATE_NAMES_AND_CLASS_STATIC_BLOCKS
                || self.language_version
                    < CLASS_AND_CLASS_ELEMENT_DECORATORS
                || !self.compiler_options.get_use_define_for_class_fields()
            {
                if assignment_kind != AssignmentKind::None {
                    self.check_external_emit_helpers(
                        node,
                        ExternalEmitHelpers::ClassPrivateFieldSet.bits(),
                    );
                }
                if assignment_kind != AssignmentKind::Definite {
                    self.check_external_emit_helpers(
                        node,
                        ExternalEmitHelpers::ClassPrivateFieldGet.bits(),
                    );
                }
            }
            let lexically_scoped_symbol =
                self.lookup_symbol_for_private_identifier_declaration(right.text(), right);
            if assignment_kind != AssignmentKind::None
                && lexically_scoped_symbol.as_ref().is_some_and(|s| {
                    s.value_declaration
                        .as_ref()
                        .is_some_and(|d| ast::is_method_declaration(d))
                })
            {
                self.grammar_error_on_node_with_args(
                    right,
                    &msgs::CANNOT_ASSIGN_TO_PRIVATE_METHOD_0_PRIVATE_METHODS_ARE_NOT_WRITABLE,
                    &[right.text().to_string()],
                );
            }
            if is_any_like {
                if let Some(lexically_scoped_symbol) = &lexically_scoped_symbol {
                    if self.is_error_type(&apparent_type) {
                        return self.error_type();
                    }
                    return apparent_type;
                }
                if get_containing_class_excluding_class_decorators(right).is_none() {
                    self.grammar_error_on_node(
                        right,
                        &msgs::PRIVATE_IDENTIFIERS_ARE_NOT_ALLOWED_OUTSIDE_CLASS_BODIES,
                    );
                    return self.any_type();
                }
            }
            if let Some(lexically_scoped_symbol) = &lexically_scoped_symbol {
                prop =
                    self.get_private_identifier_property_of_type(left_type, lexically_scoped_symbol);
            }
            if prop.is_none() {
                if self.check_private_identifier_property_access(
                    left_type,
                    right,
                    lexically_scoped_symbol.as_ref(),
                ) {
                    return self.error_type();
                }
                if let Some(containing_class) =
                    get_containing_class_excluding_class_decorators(right)
                {
                    let source_file = self.get_source_file_of_node(&containing_class);
                    if is_plain_js_file(source_file.as_deref(), self.compiler_options.check_js) {
                        self.grammar_error_on_node_with_args(
                            right,
                            &msgs::PRIVATE_FIELD_0_MUST_BE_DECLARED_IN_AN_ENCLOSING_CLASS,
                            &[right.text().to_string()],
                        );
                    }
                }
            } else {
                let prop_symbol = prop.as_ref().unwrap();
                let is_setonly_accessor = prop_symbol.flags.contains(SymbolFlags::SetAccessor)
                    && !prop_symbol.flags.contains(SymbolFlags::GetAccessor);
                if is_setonly_accessor && assignment_kind != AssignmentKind::Definite {
                    self.error_message(
                        right,msgs::PRIVATE_ACCESSOR_WAS_DEFINED_WITHOUT_A_GETTER,
                        &[],
                    );
                }
            }
        } else {
            if is_any_like {
                if ast::is_identifier(left) && parent_symbol.is_some() {
                    self.mark_linked_references(node, ReferenceHint::Property, None, Some(left_type));
                }
                if self.is_error_type(&apparent_type) {
                    return self.error_type();
                }
                return apparent_type;
            }
            prop = self.get_property_of_type_ex(
                &apparent_type,
                right.text(),
                is_const_enum_object_type(&apparent_type),
                node.kind == SyntaxKind::QualifiedName,
            );
        }
        self.mark_linked_references(node, ReferenceHint::Property, prop.as_ref(), Some(left_type));
        let prop_type = if prop.is_none() {
            let mut index_info: Option<Arc<IndexInfo>> = None;
            if !ast::is_private_identifier(right)
                && (assignment_kind == AssignmentKind::None
                    || !self.is_generic_object_type(left_type)
                    || self.is_this_type_parameter(left_type))
            {
                index_info = self.get_applicable_index_info_for_name(&apparent_type, right.text());
            }
            let Some(index_info) = index_info else {
                let is_unchecked_js =
                    self.is_unchecked_js_suggestion(Some(node), left_type.symbol.as_ref(), true);
                if !is_unchecked_js && is_js_literal_type(self, left_type) {
                    return self.any_type();
                }
                let is_global_this = left_type
                    .symbol
                    .as_ref()
                    .zip(self.global_this_symbol.as_ref())
                    .map(|(a, b)| Arc::ptr_eq(a, b))
                    .unwrap_or(false);
                if is_global_this {
                    let global_symbol = self
                        .global_this_symbol
                        .as_ref()
                        .and_then(|s| s.exports.get(right.text()).cloned());
                    if let Some(global_symbol) = global_symbol {
                        if global_symbol.flags.contains(SymbolFlags::BLOCK_SCOPED) {
                            let left_type_string = self.type_to_string(left_type);
                            self.error_message(
                                right,msgs::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                                &[right.text().to_string(), left_type_string],
                            );
                        }
                    } else if self.no_implicit_any {
                        let left_type_string = self.type_to_string(left_type);
                        self.error_message(
                            right,msgs::ELEMENT_IMPLICITLY_HAS_AN_ANY_TYPE_BECAUSE_TYPE_0_HAS_NO_INDEX_SIGNATURE,
                            &[left_type_string],
                        );
                    }
                    return self.any_type();
                }
                if !right.text().is_empty()
                    && !self.check_and_report_error_for_extending_interface(node)
                {
                    let containing_type = if self.is_this_type_parameter(left_type) {
                        Arc::clone(&apparent_type)
                    } else {
                        Arc::clone(left_type)
                    };
                    self.report_nonexistent_property(right, &containing_type, is_unchecked_js);
                }
                return self.error_type();
            };
            if index_info.is_readonly && (is_assignment_target(node) || is_delete_target(node)) {
                let apparent_type_string = self.type_to_string(&apparent_type);
                self.error_message(
                    node,msgs::INDEX_SIGNATURE_IN_TYPE_0_ONLY_PERMITS_READING,
                    &[apparent_type_string],
                );
            }
            let mut prop_type = index_info
                .value_type
                .clone()
                .unwrap_or_else(|| self.missing_type());
            if self.no_unchecked_indexed_access
                && get_assignment_target_kind(node) != AssignmentKind::Definite
            {
                prop_type = self.get_union_type(vec![Arc::clone(&prop_type), self.missing_type()]);
            }
            if self.compiler_options.no_property_access_from_index_signature.is_true()
                && ast::is_property_access_expression(node)
            {
                self.error_message(
                    right,msgs::PROPERTY_0_COMES_FROM_AN_INDEX_SIGNATURE_SO_IT_MUST_BE_ACCESSED_WITH_0,
                    &[right.text().to_string()],
                );
            }
            if let Some(declaration) = &index_info.declaration {
                if self.is_deprecated_declaration(declaration) {
                    self.add_deprecated_suggestion(
                        right,
                        std::slice::from_ref(declaration),
                        right.text(),
                    );
                }
            }
            prop_type
        } else {
            let prop = prop.as_ref().unwrap();
            let target_prop_symbol = self.resolve_alias_with_deprecation_check(prop, right);
            if self.is_deprecated_symbol(&target_prop_symbol)
                && self.is_uncalled_function_reference(node, &target_prop_symbol)
                && !target_prop_symbol.declarations.is_empty()
            {
                self.add_deprecated_suggestion(
                    right,
                    &target_prop_symbol.declarations,
                    right.text(),
                );
            }
            self.check_property_not_used_before_declaration(prop, node, right);
            let is_self = self.is_self_type_access(left, &apparent_type);
            self.mark_property_as_referenced_ex(prop, Some(node), Some(is_self));
            self.symbol_node_links.get_or_default(node).resolved_symbol = Some(Arc::clone(prop));
            self.check_property_accessibility(
                node,
                left.kind == SyntaxKind::SuperKeyword,
                is_write_access(node),
                &apparent_type,
                prop,
            );
            if self.is_assignment_to_readonly_entity(node, prop, assignment_kind) {
                self.error_message(
                    right,msgs::CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY,
                    &[right.text().to_string()],
                );
                return self.error_type();
            }
            if self.is_this_property_access_in_constructor(node, prop) {
                self.auto_type()
            } else if write_only || is_write_only_access(node) {
                self.get_write_type_of_symbol(prop)
            } else {
                self.get_type_of_symbol(prop)
            }
        };
        self.flow_type_of_access_expression(node, prop.as_ref(), Arc::clone(&prop_type))
    }

    pub fn check_kinds_of_property_member_overrides(&mut self, t: &Arc<Type>, base_type: &Arc<Type>) { ::tsox_core::fntrace::enter("check_kinds_of_property_member_overrides"); 
        let mut not_implemented_info: HashMap<u64, MemberInfo> = HashMap::new();
        for base_property in self.get_properties_of_type(base_type) {
            let base = self.get_target_symbol(&base_property);
            if base.flags.contains(SymbolFlags::Prototype) {
                continue;
            }
            let Some(base_symbol) = self.get_property_of_object_type(t, &base.name) else {
                continue;
            };
            let derived = self.get_target_symbol(&base_symbol);
            let base_declaration_flags = get_declaration_modifier_flags_from_symbol(&base);
            if Arc::ptr_eq(&derived, &base) {
                if base_declaration_flags.contains(ModifierFlags::Abstract) {
                    let derived_class_decl = t
                        .symbol
                        .as_ref()
                        .and_then(|s| get_class_like_declaration_of_symbol(s));
                    let declared_abstract = derived_class_decl
                        .as_ref()
                        .map(|d| d.has_syntactic_modifier(ModifierFlags::Abstract))
                        .unwrap_or(false);
                    if !declared_abstract {
                        let mut found_elsewhere = false;
                        for other_base_type in self.get_base_types(t) {
                            if Arc::ptr_eq(&other_base_type, base_type) {
                                continue;
                            }
                            if let Some(other_base_symbol) =
                                self.get_property_of_object_type(&other_base_type, &base.name)
                            {
                                if !Arc::ptr_eq(&base, &self.get_target_symbol(&other_base_symbol)) {
                                    found_elsewhere = true;
                                    break;
                                }
                            }
                        }
                        if found_elsewhere {
                            continue;
                        }
                        let base_type_name = self.type_to_string(base_type);
                        let type_name = self.type_to_string(t);
                        let missed_property = self.symbol_to_string(&base_property);
                        let key = derived_class_decl
                            .as_ref()
                            .map(|d| d.id())
                            .unwrap_or(0);
                        let entry = not_implemented_info.entry(key).or_insert_with(|| MemberInfo {
                            error_node: derived_class_decl.clone(),
                            missed_properties: Vec::new(),
                            base_type_name: base_type_name.clone(),
                            type_name: type_name.clone(),
                        });
                        entry.missed_properties.push(missed_property);
                        entry.base_type_name = base_type_name;
                        entry.type_name = type_name;
                    }
                }
            } else {
                let derived_declaration_flags = get_declaration_modifier_flags_from_symbol(&derived);
                if base_declaration_flags.contains(ModifierFlags::Private)
                    || derived_declaration_flags.contains(ModifierFlags::Private)
                {
                    continue;
                }
                let mut error_message: Option<&'static tsox_core::diagnostics::Message> = None;
                let base_property_flags = base.flags & SymbolFlags::PROPERTY_OR_ACCESSOR;
                let derived_property_flags = derived.flags & SymbolFlags::PROPERTY_OR_ACCESSOR;
                if !base_property_flags.is_empty() && !derived_property_flags.is_empty() {
                    if base.check_flags.contains(CheckFlags::Mapped)
                        || derived
                            .value_declaration
                            .as_ref()
                            .is_some_and(|d| ast::is_binary_expression(d))
                        || self.are_properties_abstract_or_interface(&base, base_declaration_flags)
                    {
                        continue;
                    }
                    let overridden_instance_property =
                        base_property_flags != SymbolFlags::Property
                            && derived_property_flags == SymbolFlags::Property;
                    let overridden_instance_accessor =
                        base_property_flags == SymbolFlags::Property
                            && derived_property_flags != SymbolFlags::Property;
                    if overridden_instance_property || overridden_instance_accessor {
                        let message = if overridden_instance_property {
                            &msgs::X_0_IS_DEFINED_AS_AN_ACCESSOR_IN_CLASS_1_BUT_IS_OVERRIDDEN_HERE_IN_2_AS_AN_INSTANCE_PROPERTY
                        } else {
                            &msgs::X_0_IS_DEFINED_AS_A_PROPERTY_IN_CLASS_1_BUT_IS_OVERRIDDEN_HERE_IN_2_AS_AN_ACCESSOR
                        };
                        let derived_value_declaration = derived.value_declaration.clone();
                        let error_node = derived_value_declaration
                            .as_ref()
                            .and_then(|d| ast::get_name_of_declaration(d))
                            .or_else(|| derived_value_declaration.clone());
                        if let Some(error_node) = error_node {
                            let base_str = self.symbol_to_string(&base);
                            let base_type_str = self.type_to_string(base_type);
                            let t_str = self.type_to_string(t);
                            self.error_message(&error_node, *message, &[base_str, base_type_str, t_str]);
                        }
                    } else if self.compiler_options.get_use_define_for_class_fields() {
                        let uninitialized = derived.declarations.iter().find(|d| {
                            ast::is_property_declaration(d)
                                && matches!(
                                    &d.data,
                                    NodeData::PropertyDeclaration(pd) if pd.initializer.is_none()
                                )
                        });
                        if let Some(uninitialized) = uninitialized {
                            if !derived.flags.contains(SymbolFlags::Transient)
                                && !base_declaration_flags.contains(ModifierFlags::Abstract)
                                && !derived_declaration_flags.contains(ModifierFlags::Abstract)
                                && !derived
                                    .declarations
                                    .iter()
                                    .any(|d| d.flags.contains(NodeFlags::Ambient))
                            {
                                let class_decl = t
                                    .symbol
                                    .as_ref()
                                    .and_then(|s| get_class_like_declaration_of_symbol(s));
                                let constructor = class_decl
                                    .as_ref()
                                    .and_then(|cd| find_constructor_declaration(cd));
                                let postfix_is_exclamation = matches!(
                                    &uninitialized.data,
                                    NodeData::PropertyDeclaration(pd) if pd
                                        .postfix_token
                                        .as_ref()
                                        .map(|t| is_exclamation_token(t))
                                        .unwrap_or(false)
                                );
                                let prop_name = uninitialized.name().cloned();
                                let should_error = postfix_is_exclamation
                                    || constructor.is_none()
                                    || !prop_name
                                        .as_ref()
                                        .map(|n| ast::is_identifier(n))
                                        .unwrap_or(false)
                                    || !self.strict_null_checks
                                    || match (prop_name.as_ref(), constructor.as_ref()) {
                                        (Some(n), Some(c)) => {
                                            !super::wc1c::r24k17_defs::is_property_initialized_in_constructor(self, n, t, c)
                                        }
                                        _ => true,
                                    };
                                if should_error {
                                    let derived_value_declaration = derived.value_declaration.clone();
                                    let error_node = derived_value_declaration
                                        .as_ref()
                                        .and_then(|d| ast::get_name_of_declaration(d))
                                        .or_else(|| derived_value_declaration.clone());
                                    if let Some(error_node) = error_node {
                                        let base_str = self.symbol_to_string(&base);
                                        let base_type_str = self.type_to_string(base_type);
                                        self.error_message(
                                            &error_node,msgs::PROPERTY_0_WILL_OVERWRITE_THE_BASE_PROPERTY_IN_1_IF_THIS_IS_INTENTIONAL_ADD_AN_INITIALIZER_OTHERWISE_ADD_A_DECLARE_MODIFIER_OR_REMOVE_THE_REDUNDANT_DECLARATION,
                                            &[base_str, base_type_str],
                                        );
                                    }
                                }
                            }
                        }
                    }
                    continue;
                } else if is_prototype_property(&base) {
                    if is_prototype_property(&derived) || derived.flags.contains(SymbolFlags::Property)
                    {
                        continue;
                    } else {
                        error_message = Some(&msgs::CLASS_0_DEFINES_INSTANCE_MEMBER_FUNCTION_1_BUT_EXTENDED_CLASS_2_DEFINES_IT_AS_INSTANCE_MEMBER_ACCESSOR);
                    }
                } else if base.flags.contains(SymbolFlags::GetAccessor | SymbolFlags::SetAccessor) {
                    error_message = Some(&msgs::CLASS_0_DEFINES_INSTANCE_MEMBER_ACCESSOR_1_BUT_EXTENDED_CLASS_2_DEFINES_IT_AS_INSTANCE_MEMBER_FUNCTION);
                } else {
                    error_message = Some(&msgs::CLASS_0_DEFINES_INSTANCE_MEMBER_PROPERTY_1_BUT_EXTENDED_CLASS_2_DEFINES_IT_AS_INSTANCE_MEMBER_FUNCTION);
                }
                let derived_value_declaration = derived.value_declaration.clone();
                let error_node = derived_value_declaration
                    .as_ref()
                    .and_then(|d| ast::get_name_of_declaration(d))
                    .or_else(|| derived_value_declaration.clone());
                if let Some(error_node) = error_node {
                    let base_type_str = self.type_to_string(base_type);
                    let base_str = self.symbol_to_string(&base);
                    let t_str = self.type_to_string(t);
                    self.error_message(
                        &error_node,
                        *error_message.unwrap(),
                        &[base_type_str, base_str, t_str],
                    );
                }
            }
        }
        for member_info in not_implemented_info.into_values() {
            let Some(error_node) = &member_info.error_node else {
                continue;
            };
            if member_info.missed_properties.len() == 1 {
                let missed_property = member_info.missed_properties[0].clone();
                if ast::is_class_expression(error_node) {
                    self.error_message(
                        error_node,msgs::NON_ABSTRACT_CLASS_EXPRESSION_DOES_NOT_IMPLEMENT_INHERITED_ABSTRACT_MEMBER_0_FROM_CLASS_1,
                        &[missed_property, member_info.base_type_name],
                    );
                } else {
                    self.error_message(
                        error_node,msgs::NON_ABSTRACT_CLASS_0_DOES_NOT_IMPLEMENT_INHERITED_ABSTRACT_MEMBER_1_FROM_CLASS_2,
                        &[member_info.type_name, missed_property, member_info.base_type_name],
                    );
                }
            } else if member_info.missed_properties.len() > 5 {
                let missed_properties = member_info.missed_properties[..4]
                    .iter()
                    .map(|p| format!("'{p}'"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let remaining_missed_properties = member_info.missed_properties.len() - 4;
                if ast::is_class_expression(error_node) {
                    self.error_message(
                        error_node,msgs::NON_ABSTRACT_CLASS_EXPRESSION_IS_MISSING_IMPLEMENTATIONS_FOR_THE_FOLLOWING_MEMBERS_OF_0_COLON_1_AND_2_MORE,
                        &[member_info.base_type_name, missed_properties, remaining_missed_properties.to_string()],
                    );
                } else {
                    self.error_message(
                        error_node,msgs::NON_ABSTRACT_CLASS_0_IS_MISSING_IMPLEMENTATIONS_FOR_THE_FOLLOWING_MEMBERS_OF_1_COLON_2_AND_3_MORE,
                        &[member_info.type_name, member_info.base_type_name, missed_properties, remaining_missed_properties.to_string()],
                    );
                }
            } else {
                let missed_properties = member_info
                    .missed_properties
                    .iter()
                    .map(|p| format!("'{p}'"))
                    .collect::<Vec<_>>()
                    .join(", ");
                if ast::is_class_expression(error_node) {
                    self.error_message(
                        error_node,msgs::NON_ABSTRACT_CLASS_EXPRESSION_IS_MISSING_IMPLEMENTATIONS_FOR_THE_FOLLOWING_MEMBERS_OF_0_COLON_1,
                        &[member_info.base_type_name, missed_properties],
                    );
                } else {
                    self.error_message(
                        error_node,msgs::NON_ABSTRACT_CLASS_0_IS_MISSING_IMPLEMENTATIONS_FOR_THE_FOLLOWING_MEMBERS_OF_1_COLON_2,
                        &[member_info.type_name, member_info.base_type_name, missed_properties],
                    );
                }
            }
        }
    }

    fn are_properties_abstract_or_interface(
        &mut self,
        base: &Arc<Symbol>,
        base_declaration_flags: ModifierFlags,
    ) -> bool { ::tsox_core::fntrace::enter("are_properties_abstract_or_interface"); 
        if base.check_flags.contains(CheckFlags::SYNTHETIC) {
            return base
                .declarations
                .iter()
                .any(|d| self.is_property_abstract_or_interface(d, base_declaration_flags));
        }
        base.declarations
            .iter()
            .all(|d| self.is_property_abstract_or_interface(d, base_declaration_flags))
    }

    fn is_property_abstract_or_interface(
        &self,
        declaration: &Arc<Node>,
        base_declaration_flags: ModifierFlags,
    ) -> bool { ::tsox_core::fntrace::enter("is_property_abstract_or_interface"); 
        declaration
            .parent()
            .map(|p| ast::is_interface_declaration(&p))
            .unwrap_or(false)
            || base_declaration_flags.contains(ModifierFlags::Abstract)
                && (!ast::is_property_declaration(declaration)
                    || matches!(
                        &declaration.data,
                        NodeData::PropertyDeclaration(pd) if pd.initializer.is_none()
                    ))
    }
}
