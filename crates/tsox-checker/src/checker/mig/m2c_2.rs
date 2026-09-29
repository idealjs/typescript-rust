#![allow(unused_imports)]

use crate::checker::checker::*;
use std::sync::Arc;
use std::sync::OnceLock;
use tsox_core::diagnostics::messages_generated::*;
use tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS;
use tsox_frontend::ast::NodeData;
use tsox_frontend::ast::SourceFile;
use tsox_frontend::ast::node_data_generated::is_qualified_name;
use crate::checker::types_type_id::EXTERNAL_HELPERS_MODULE_NAME_TEXT;
use crate::checker::mig::m1b::is_string_literal_like;
use crate::checker::mig::m2c_3::some_signature;
use crate::checker::mig::m1e::r20k2_defs::has_syntactic_modifier;
use crate::checker::mig::m2a::r19k11_defs::R19K11CheckerExt;
use crate::checker::mig::wc1b::is_type_any;
use crate::checker::mig::wc3::NodeAccessExt;
use tsox_frontend::ast::mig::m3c::type_arguments;
use super::m2c::r21k2_defs::{
    get_class_like_declaration_of_symbol, get_host_signature_from_jsdoc, is_entity_name,
};

impl Checker {
    pub fn resolve_jsdoc_member_name(&mut self, name: Option<&Arc<Node>>) -> Option<Arc<Symbol>> {
        if let Some(name) = name {
            if is_entity_name(name) {
                let meaning =
                    SymbolFlags::TYPE | SymbolFlags::NAMESPACE | SymbolFlags::VALUE;
                let symbol = self.resolve_entity_name(
                    name,
                    meaning,
                    true,
                    true,
                    get_host_signature_from_jsdoc(name).as_ref(),
                );
                if symbol.is_some() {
                    return symbol;
                }
                if is_qualified_name(name) {
                    let symbol = self.resolve_jsdoc_member_name(Some(&name.as_qualified_name().left))?;
                    let mut t: Option<Arc<Type>> = None;
                    if symbol.flags.intersects(SymbolFlags::VALUE) {
                        let type_of_symbol = self.get_type_of_symbol(&symbol);
                        let proto = self.get_property_of_type(&type_of_symbol, "prototype");
                        if let Some(proto) = proto {
                            t = Some(self.get_type_of_symbol(&proto));
                        }
                    }
                    let t = match t {
                        Some(t) => t,
                        None => self.get_declared_type_of_symbol(&symbol),
                    };
                    return self.get_property_of_type(&t, &name.as_qualified_name().right.text());
                }
            }
        }
        None
    }

    pub fn resolve_new_expression(
        &mut self,
        node: &Arc<Node>,
        candidates_out_array: Option<&mut Vec<Arc<Signature>>>,
        check_mode: CheckMode,
    ) -> Option<Arc<Signature>> {
        let Some(expression) = node.expression() else {
            return None;
        };
        let expression_type = self.check_non_null_expression(&expression);
        if Arc::ptr_eq(&expression_type, &self.silent_never_type()) {
            return Some(self.silent_never_signature());
        }
        let expression_type = self.get_apparent_type(&expression_type);
        if self.is_error_type(&expression_type) {
            return Some(self.resolve_error_call(node));
        }
        if is_type_any(&expression_type) {
            if !type_arguments(node).is_empty() {
                self.error_message(
                    node,UNTYPED_FUNCTION_CALLS_MAY_NOT_ACCEPT_TYPE_ARGUMENTS,
                    &[],
                );
            }
            return Some(self.resolve_untyped_call(node));
        }
        let construct_signatures =
            self.get_signatures_of_type(&expression_type, SignatureKind::Construct);
        if !construct_signatures.is_empty() {
            if !self.is_constructor_accessible(node, Some(&construct_signatures[0])) {
                return Some(self.resolve_error_call(node));
            }
            if some_signature(&construct_signatures, &|sig| {
                sig.flags.intersects(SignatureFlags::Abstract)
            }) {
                self.error_message(
                    node,CANNOT_CREATE_AN_INSTANCE_OF_AN_ABSTRACT_CLASS,
                    &[],
                );
                return Some(self.resolve_error_call(node));
            }
            if let Some(symbol) = &expression_type.symbol {
                let value_decl = get_class_like_declaration_of_symbol(symbol);
                if let Some(value_decl) = value_decl {
                    if has_syntactic_modifier(&value_decl, ModifierFlags::Abstract) {
                        self.error_message(
                            node,CANNOT_CREATE_AN_INSTANCE_OF_AN_ABSTRACT_CLASS,
                            &[],
                        );
                        return Some(self.resolve_error_call(node));
                    }
                }
            }
            return self.resolve_call(
                node,
                &construct_signatures,
                candidates_out_array,
                check_mode,
                SignatureFlags::None,
                None,
            );
        }
        let call_signatures = self.get_signatures_of_type(&expression_type, SignatureKind::Call);
        if !call_signatures.is_empty() {
            let signature = self.resolve_call(
                node,
                &call_signatures,
                candidates_out_array,
                check_mode,
                SignatureFlags::None,
                None,
            )?;
            if !self.no_implicit_any {
                if signature.declaration.is_some()
                    && !self
                        .get_return_type_of_signature(&signature)
                        .is_some_and(|rt| Arc::ptr_eq(&rt, &self.void_type()))
                {
                    self.error_message(
                        node,ONLY_A_VOID_FUNCTION_CAN_BE_CALLED_WITH_THE_NEW_KEYWORD,
                        &[],
                    );
                }
                if self
                    .get_this_type_of_signature(&signature)
                    .is_some_and(|tt| Arc::ptr_eq(&tt, &self.void_type()))
                {
                    self.error_message(
                        node,A_FUNCTION_THAT_IS_CALLED_WITH_THE_NEW_KEYWORD_CANNOT_HAVE_A_THIS_TYPE_THAT_IS_VOID,
                        &[],
                    );
                }
            }
            return Some(signature);
        }
        self.invocation_error(
            &expression,
            &expression_type,
            SignatureKind::Construct,
            None,
        );
        Some(self.resolve_error_call(node))
    }

    pub fn resolve_tagged_template_expression(
        &mut self,
        node: &Arc<Node>,
        candidates_out_array: Option<&mut Vec<Arc<Signature>>>,
        check_mode: CheckMode,
    ) -> Option<Arc<Signature>> {
        let tag = &node.as_tagged_template_expression().tag;
        let tag_type = self.check_expression_cached(tag);
        let apparent_type = self.get_apparent_type(&tag_type);
        if self.is_error_type(&apparent_type) {
            return Some(self.resolve_error_call(node));
        }
        let call_signatures = self.get_signatures_of_type(&apparent_type, SignatureKind::Call);
        let num_construct_signatures = self
            .get_signatures_of_type(&apparent_type, SignatureKind::Construct)
            .len();
        if self.is_untyped_function_call(
            &tag_type,
            &apparent_type,
            call_signatures.len(),
            num_construct_signatures,
        ) {
            return Some(self.resolve_untyped_call(node));
        }
        if call_signatures.is_empty() {
            if node.parent().map(|p| p.kind) == Some(SyntaxKind::ArrayLiteralExpression) {
                let source_file = self.get_source_file_of_node(tag);
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    source_file,
                    tag.loc,
                    IT_IS_LIKELY_THAT_YOU_ARE_MISSING_A_COMMA_TO_SEPARATE_THESE_TWO_TEMPLATE_EXPRESSIONS_THEY_FORM_A_TAGGED_TEMPLATE_EXPRESSION_WHICH_CANNOT_BE_INVOKED,
                    vec![],
                ));
                return Some(self.resolve_error_call(node));
            }
            self.invocation_error(tag, &apparent_type, SignatureKind::Call, None);
            return Some(self.resolve_error_call(node));
        }
        self.resolve_call(
            node,
            &call_signatures,
            candidates_out_array,
            check_mode,
            SignatureFlags::None,
            None,
        )
    }

    pub fn resolve_instanceof_expression(
        &mut self,
        node: &Arc<Node>,
        candidates_out_array: Option<&mut Vec<Arc<Signature>>>,
        check_mode: CheckMode,
    ) -> Option<Arc<Signature>> {
        let right = &node.as_binary_expression().right;
        let right_type = self.check_expression_cached(right);
        if !is_type_any(&right_type) {
            let has_instance_method_type =
                self.get_symbol_has_instance_method_of_object_type(&right_type);
            if let Some(has_instance_method_type) = has_instance_method_type {
                let apparent_type = self.get_apparent_type(&has_instance_method_type);
                if self.is_error_type(&apparent_type) {
                    return Some(self.resolve_error_call(node));
                }
                let call_signatures =
                    self.get_signatures_of_type(&apparent_type, SignatureKind::Call);
                let construct_signatures =
                    self.get_signatures_of_type(&apparent_type, SignatureKind::Construct);
                if self.is_untyped_function_call(
                    &has_instance_method_type,
                    &apparent_type,
                    call_signatures.len(),
                    construct_signatures.len(),
                ) {
                    return Some(self.resolve_untyped_call(node));
                }
                if !call_signatures.is_empty() {
                    return self.resolve_call(
                        node,
                        &call_signatures,
                        candidates_out_array,
                        check_mode,
                        SignatureFlags::None,
                        None,
                    );
                }
            } else if !(self.type_has_call_or_construct_signatures(&right_type)
                || self.is_type_subtype_of(&right_type, &self.global_function_type()))
            {
                self.error_message(
                    right,THE_RIGHT_HAND_SIDE_OF_AN_INSTANCEOF_EXPRESSION_MUST_BE_EITHER_OF_TYPE_ANY_A_CLASS_FUNCTION_OR_OTHER_TYPE_ASSIGNABLE_TO_THE_FUNCTION_INTERFACE_TYPE_OR_AN_OBJECT_TYPE_WITH_A_SYMBOL_HASINSTANCE_METHOD,
                    &[],
                );
                return Some(self.resolve_error_call(node));
            }
        }
        Some(self.any_signature())
    }

    pub fn resolve_untyped_call(&mut self, node: &Arc<Node>) -> Arc<Signature> {
        if self.call_like_expression_may_have_type_arguments(node) {
            self.check_source_elements(type_arguments(node));
        }
        match node.kind {
            SyntaxKind::TaggedTemplateExpression => {
                self.check_expression(&node.as_tagged_template_expression().template);
            }
            SyntaxKind::JsxOpeningElement | SyntaxKind::JsxSelfClosingElement => {
                let attributes = match &node.data {
                    NodeData::JsxOpeningElement(d) => Arc::clone(&d.attributes),
                    NodeData::JsxSelfClosingElement(d) => Arc::clone(&d.attributes),
                    _ => unreachable!(),
                };
                self.check_expression(&attributes);
            }
            SyntaxKind::BinaryExpression => {
                self.check_expression(&node.as_binary_expression().left);
            }
            SyntaxKind::CallExpression | SyntaxKind::NewExpression => {
                for argument in tsox_frontend::ast::mig::x1a::arguments(node) {
                    self.check_expression(argument);
                }
            }
            _ => {}
        }
        Arc::clone(&self.any_signature())
    }

    pub fn resolve_symbol_ex(
        &mut self,
        symbol: &Arc<Symbol>,
        dont_resolve_alias: bool,
    ) -> Arc<Symbol> {
        if !dont_resolve_alias
            && is_non_local_alias(
                symbol,
                SymbolFlags::VALUE | SymbolFlags::TYPE | SymbolFlags::NAMESPACE,
            )
        {
            return self.resolve_alias(symbol);
        }
        Arc::clone(symbol)
    }

    pub fn resolve_external_module_type_by_literal(&mut self, name: &Arc<Node>) -> Arc<Type> {
        let module_sym = self.resolve_external_module_name(name);
        if let Some(module_sym) = module_sym {
            let resolved_module_symbol = self.resolve_external_module_symbol(&module_sym, false);
            return self.get_type_of_symbol(&resolved_module_symbol);
        }
        self.any_type()
    }

    pub fn resolve_export_by_name(
        &mut self,
        module_symbol: &Arc<Symbol>,
        name: &str,
        source_node: Option<&Arc<Node>>,
        dont_resolve_alias: bool,
    ) -> Option<Arc<Symbol>> {
        let export_value = module_symbol.exports.get(INTERNAL_SYMBOL_NAME_EXPORT_EQUALS);
        let export_symbol = match export_value {
            Some(export_value) => {
                let export_value_type = self.get_type_of_symbol(export_value);
                self.get_property_of_type_ex(&export_value_type, name, true, false)
            }
            None => module_symbol.exports.get(name).cloned(),
        };
        let resolved = match export_symbol {
            Some(export_symbol) => Some(self.resolve_symbol_ex(&export_symbol, dont_resolve_alias)),
            None => None,
        };
        self.mark_symbol_of_alias_declaration_if_type_only(source_node, None);
        resolved
    }

    pub fn resolve_indirection_alias(
        &mut self,
        source: &Arc<Symbol>,
        target: &Arc<Symbol>,
    ) -> Arc<Symbol> {
        let alias = self.resolve_alias(target);
        let result = self.get_merged_symbol(&alias);
        let target_type_only = self
            .alias_symbol_links
            .get(target)
            .and_then(|l| l.type_only_declaration.clone());
        if target_type_only.is_some() {
            let source_links = self.alias_symbol_links.get_or_default(source);
            if source_links.type_only_declaration.is_none() {
                source_links.type_only_declaration = target_type_only;
            }
        }
        result
    }

    pub fn try_resolve_alias(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        let has_alias_target = self
            .alias_symbol_links
            .get(symbol)
            .map(|l| l.alias_target.is_some())
            .unwrap_or(false);
        if has_alias_target
            || self.find_resolution_cycle_start_index(
                TypeSystemEntity::Symbol(Arc::clone(symbol)),
                TypeSystemPropertyName::AliasTarget,
            ) < 0
        {
            return Some(self.resolve_alias(symbol));
        }
        None
    }

    pub fn resolve_external_module_name_worker(
        &mut self,
        location: &Arc<Node>,
        module_reference_expression: Option<&Arc<Node>>,
        module_not_found_error: Option<&'static tsox_core::diagnostics::Message>,
        ignore_errors: bool,
        is_for_augmentation: bool,
        import_attributes_type: Option<&Arc<Type>>,
    ) -> Option<Arc<Symbol>> {
        if let Some(module_reference_expression) = module_reference_expression {
            if is_string_literal_like(module_reference_expression) {
                let error_node = if !ignore_errors {
                    Some(Arc::clone(module_reference_expression))
                } else {
                    None
                };
                return self.resolve_external_module(
                    location,
                    &module_reference_expression.text(),
                    module_not_found_error,
                    error_node.as_ref(),
                    is_for_augmentation,
                    import_attributes_type,
                );
            }
        }
        None
    }

    pub fn resolve_helpers_module(
        &mut self,
        file: &Arc<SourceFile>,
        error_node: Option<&Arc<Node>>,
    ) -> Arc<Symbol> {
        let has_helpers = self
            .source_file_links
            .get(file.as_ref())
            .map(|l| l.external_helpers_module.is_some())
            .unwrap_or(false);
        if !has_helpers {
            let helpers_module = self.resolve_external_module(
                &file.node,
                EXTERNAL_HELPERS_MODULE_NAME_TEXT,
                Some(
                    &THIS_SYNTAX_REQUIRES_AN_IMPORTED_HELPER_BUT_MODULE_0_CANNOT_BE_FOUND,
                ),
                error_node,
                false,
                None,
            );
            let helpers_module =
                helpers_module.unwrap_or_else(|| self.unknown_symbol());
            self.source_file_links
                .get_or_default(file.as_ref())
                .external_helpers_module = Some(helpers_module);
        }
        self.source_file_links
            .get(file.as_ref())
            .and_then(|l| l.external_helpers_module.clone())
            .unwrap_or_else(|| self.unknown_symbol())
    }

    pub fn resolve_type_reference_members(&mut self, t: &Arc<Type>) {
        let source = t.target().cloned().unwrap_or_else(|| Arc::clone(t));
        let type_parameters = source
            .as_interface_type()
            .map(|i| i.all_type_parameters.clone())
            .unwrap_or_default();
        let type_arguments_of_t = self.get_type_arguments(t);
        let mut padded_type_arguments = type_arguments_of_t.clone();
        if type_arguments_of_t.len() == type_parameters.len().saturating_sub(1) {
            padded_type_arguments.push(Arc::clone(t));
        }
        self.resolve_object_type_members(
            t,
            &source,
            &type_parameters,
            &padded_type_arguments,
        );
    }

    pub fn resolve_object_type_members(
        &mut self,
        t: &Arc<Type>,
        source: &Arc<Type>,
        type_parameters: &[Arc<Type>],
        type_arguments: &[Arc<Type>],
    ) {
        let mut mapper: Option<Arc<TypeMapper>> = None;
        let mut members: SymbolTable;
        let mut call_signatures: Vec<Arc<Signature>>;
        let mut construct_signatures: Vec<Arc<Signature>>;
        let mut index_infos: Vec<Arc<IndexInfo>>;
        let mut instantiated = false;
        self.resolve_declared_members(source);
        let (declared_members, declared_call_signatures, declared_construct_signatures, declared_index_infos) =
            match &source.data {
                TypeData::Interface(interface) => (
                    &interface.declared_members,
                    &interface.declared_call_signatures,
                    &interface.declared_construct_signatures,
                    &interface.declared_index_infos,
                ),
                _ => return,
            };
        if type_parameters.len() == type_arguments.len()
            && type_parameters
                .iter()
                .zip(type_arguments.iter())
                .all(|(p, a)| Arc::ptr_eq(p, a))
        {
            members = declared_members.clone();
            call_signatures = declared_call_signatures.clone();
            construct_signatures = declared_construct_signatures.clone();
            index_infos = declared_index_infos.clone();
        } else {
            instantiated = true;
            let mapper_value = Arc::new(super::w9a::new_type_mapper(
                type_parameters.to_vec(),
                type_arguments.to_vec(),
            ));
            members = self.instantiate_symbol_table(declared_members, Some(&mapper_value));
            call_signatures =
                self.instantiate_signatures(declared_call_signatures, Some(&mapper_value));
            construct_signatures = self.instantiate_signatures(
                declared_construct_signatures,
                Some(&mapper_value),
            );
            index_infos =
                self.instantiate_index_infos(declared_index_infos, Some(&mapper_value));
            mapper = Some(mapper_value);
        }
        let base_types = self.get_base_types(source);
        if !base_types.is_empty() {
            if !instantiated {
                members = members.clone();
            }
            self.set_structured_type_members(
                t,
                Some(members.clone()),
                call_signatures.clone(),
                construct_signatures.clone(),
                index_infos.clone(),
            );
            let this_argument = type_arguments.last().cloned();
            let mut t_flags = Arc::clone(t);
            if let Some(t_flags) = Arc::get_mut(&mut t_flags) {
                t_flags.object_flags |= ObjectFlags::UnresolvedMembers;
            }
            for base_type in &base_types {
                let instantiated_base_type = match &this_argument {
                    Some(this_argument) => {
                        let instantiated =
                            self.instantiate_type(base_type, mapper.as_ref());
                        self.get_type_with_this_argument(&instantiated, Some(this_argument), false)
                    }
                    None => Arc::clone(base_type),
                };
                let inherited_properties = self.get_properties_of_type(&instantiated_base_type);
                members = self.add_inherited_members(members.clone(), &inherited_properties);
                let inherited_call =
                    self.get_signatures_of_type(&instantiated_base_type, SignatureKind::Call);
                call_signatures.extend(inherited_call);
                let inherited_construct =
                    self.get_signatures_of_type(&instantiated_base_type, SignatureKind::Construct);
                construct_signatures.extend(inherited_construct);
                let inherited_index_infos = if !Arc::ptr_eq(&instantiated_base_type, &self.any_type())
                {
                    self.get_index_infos_of_type(&instantiated_base_type)
                } else {
                    vec![self.any_base_type_index_info()]
                };
                for info in inherited_index_infos {
                    let Some(key_type) = info.key_type.as_ref() else { continue };
                    if self.find_index_info(&index_infos, key_type).is_none() {
                        index_infos.push(info);
                    }
                }
            }
            if let Some(t_flags) = Arc::get_mut(&mut t_flags) {
                t_flags.object_flags &= !ObjectFlags::UnresolvedMembers;
            }
        }
        self.set_structured_type_members(
            t,
            Some(members),
            call_signatures,
            construct_signatures,
            index_infos,
        );
    }

    pub fn resolve_union_type_members(&mut self, t: &Arc<Type>) {
        let mut call_signature_sets: Vec<Vec<Arc<Signature>>> = Vec::new();
        for constituent in t.types().into_iter().flatten() {
            if Arc::ptr_eq(constituent, &self.global_function_type()) {
                call_signature_sets.push(vec![self.unknown_signature()]);
            } else {
                call_signature_sets
                    .push(self.get_signatures_of_type(constituent, SignatureKind::Call));
            }
        }
        let mut call_signatures = self.get_union_signatures(&call_signature_sets);
        if call_signatures.is_empty() {
            call_signatures = self.get_array_member_call_signatures(t);
        }
        let mut construct_signature_sets: Vec<Vec<Arc<Signature>>> = Vec::new();
        for constituent in t.types().into_iter().flatten() {
            construct_signature_sets
                .push(self.get_signatures_of_type(constituent, SignatureKind::Construct));
        }
        let construct_signatures = self.get_union_signatures(&construct_signature_sets);
        let mut index_infos: Vec<Arc<IndexInfo>> = Vec::new();
        for constituent in t.types().into_iter().flatten() {
            for info in self.get_index_infos_of_type(constituent) {
                index_infos = self.append_index_info(index_infos, &info, false);
            }
        }
        self.set_structured_type_members(
            t,
            None,
            call_signatures,
            construct_signatures,
            index_infos,
        );
    }

    pub fn resolve_intersection_type_members(&mut self, t: &Arc<Type>) {
        let mut call_signatures: Vec<Arc<Signature>> = Vec::new();
        let mut construct_signatures: Vec<Arc<Signature>> = Vec::new();
        let mut index_infos: Vec<Arc<IndexInfo>> = Vec::new();
        let types = t.types().map(|ts| ts.to_vec()).unwrap_or_default();
        let (mixin_flags, mixin_count) = self.find_mixins(&types);
        for (i, constituent) in types.iter().enumerate() {
            if !mixin_flags[i] {
                let mut signatures =
                    self.get_signatures_of_type(constituent, SignatureKind::Construct);
                if !signatures.is_empty() && mixin_count > 0 {
                    signatures = signatures
                        .iter()
                        .map(|s| {
                            let mut clone = self.clone_signature(s);
                            if let Some(return_type) = self.get_return_type_of_signature(s) {
                                let mixed =
                                    self.include_mixin_type(&return_type, &types, &mixin_flags, i);
                                if let Some(clone) = Arc::get_mut(&mut clone) {
                                    clone.resolved_return_type = OnceLock::from(mixed);
                                }
                            }
                            clone
                        })
                        .collect();
                }
                construct_signatures = self.append_signatures(construct_signatures, &signatures);
            }
            let constituent_call =
                self.get_signatures_of_type(constituent, SignatureKind::Call);
            call_signatures = self.append_signatures(call_signatures, &constituent_call);
            for info in self.get_index_infos_of_type(constituent) {
                index_infos = self.append_index_info(index_infos, &info, false);
            }
        }
        self.set_structured_type_members(
            t,
            None,
            call_signatures,
            construct_signatures,
            index_infos,
        );
    }

    pub fn set_structured_type_members(
        &mut self,
        t: &Arc<Type>,
        members: Option<SymbolTable>,
        call_signatures: Vec<Arc<Signature>>,
        construct_signatures: Vec<Arc<Signature>>,
        index_infos: Vec<Arc<IndexInfo>>,
    ) {
        let call_signatures_count = call_signatures.len();
        let mut t = Arc::clone(t);
        let Some(t) = Arc::get_mut(&mut t) else { return };
        t.object_flags |= ObjectFlags::MembersResolved;
        let Some(data) = (
            match &mut t.data {
                TypeData::Object(d) => Some(&mut d.structured),
                TypeData::Interface(d) => Some(&mut d.object.structured),
                TypeData::Tuple(d) => Some(&mut d.interface_data.object.structured),
                TypeData::Mapped(d) => Some(&mut d.object.structured),
                TypeData::ReverseMapped(d) => Some(&mut d.object.structured),
                TypeData::EvolvingArray(d) => Some(&mut d.object.structured),
                TypeData::InstantiationExpression(d) => Some(&mut d.object.structured),
                TypeData::Union(d) => Some(&mut d.union_or_intersection.structured),
                TypeData::Intersection(d) => Some(&mut d.union_or_intersection.structured),
                _ => None,
            }
        ) else {
            return;
        };
        data.members = members.unwrap_or_default();
        data.properties = data
            .members
            .iter()
            .filter(|(_, s)| {
                !crate::checker::utilities_is_optional_symbol::is_reserved_member_name(&s.name)
            })
            .map(|(_, s)| Arc::clone(s))
            .collect();
        if !call_signatures.is_empty() {
            if !construct_signatures.is_empty() {
                let mut signatures = call_signatures.clone();
                signatures.extend(construct_signatures.iter().cloned());
                data.signatures = signatures;
            } else {
                data.signatures = call_signatures;
            }
            data.call_signature_count = call_signatures_count;
        } else {
            data.signatures = construct_signatures;
            data.call_signature_count = 0;
        }
        data.index_infos = index_infos;
    }
}
