#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::types::*;
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated as msgs;
use tsox_core::diagnostics::{Category, Message};

static PRIVATE_IDENTIFIERS_CANNOT_BE_USED_IN_DESTRUCTURING_PATTERNS: Message = Message {
    code: 18064,
    category: Category::Error,
    key: "Private_identifiers_cannot_be_used_in_destructuring_patterns_18064",
    text: "Private identifiers cannot be used in destructuring patterns.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};
use tsox_frontend::ast::{self, Node, NodeData, NodeList, Symbol, SymbolTable, SyntaxKind};

use super::m2c::r18k3_defs::LanguageFeatureMinimumTarget::ObjectSpreadRest as OBJECT_SPREAD_REST;
use super::wc1c::r24k17_defs::{pattern_for_type_of, set_pattern_for_type};
use crate::checker::mig::m2e_2::r24k19_defs::add_intra_expression_inference_site_to;

use crate::checker::types_type_id::TYPE_FLAGS_STRING_OR_NUMBER_LITERAL_OR_UNIQUE;
use crate::checker::utilities_token_is_identifier_or_keyword::{
    get_property_name_from_type, is_type_usable_as_property_name,
};
use tsox_frontend::ast::{is_in_js_file, is_in_json_file};

impl Checker {
    pub fn check_object_literal_destructuring_property_assignment(
        &mut self,
        node: &Arc<Node>,
        object_literal_type: &Arc<Type>,
        property_index: usize,
        all_properties: Option<&NodeList>,
        right_is_this: bool,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("check_object_literal_destructuring_property_assignment"); 
        let properties: &NodeList = match &node.data {
            NodeData::ObjectLiteralExpression(d) => &d.properties,
            _ => return None,
        };
        let property = Arc::clone(&properties.nodes[property_index]);
        if ast::is_property_assignment(&property) || ast::is_shorthand_property_assignment(&property)
        {
            let name = property.name().cloned();
            if let Some(name) = &name {
                if ast::is_private_identifier(name) {
                    self.grammar_error_on_node(
                        name,
                        &PRIVATE_IDENTIFIERS_CANNOT_BE_USED_IN_DESTRUCTURING_PATTERNS,
                    );
                }
            }
            let expr_type = name
                .as_ref()
                .and_then(|n| self.get_literal_type_from_property_name(n));
            if let Some(expr_type) = &expr_type {
                if is_type_usable_as_property_name(expr_type) {
                    let text = get_property_name_from_type(expr_type);
                    if let Some(prop) = self.get_property_of_type(object_literal_type, &text) {
                        self.mark_property_as_referenced_ex(
                            &prop,
                            Some(&property),
                            Some(right_is_this),
                        );
                        self.check_property_accessibility(&property, false, true, object_literal_type, &prop);
                    }
                }
            }
            let expr_type = expr_type.unwrap_or_else(|| self.error_type());
            let allow_missing = if self.has_default_value(&property) {
                AccessFlags::AllowMissing
            } else {
                AccessFlags::empty()
            };
            let access_flags = AccessFlags::ExpressionPosition | allow_missing;
            let element_type = self.get_indexed_access_type_ex(
                object_literal_type,
                &expr_type,
                access_flags,
                name.as_ref(),
                None,
            );
            let t = self.get_flow_type_of_destructuring(&property, &element_type);
            let expr = match &property.data {
                NodeData::PropertyAssignment(pa) => Arc::clone(&pa.initializer),
                _ => Arc::clone(&property),
            };
            self.check_destructuring_assignment(&expr, &t);
            return Some(t);
        }
        if ast::is_spread_assignment(&property) {
            if property_index < properties.nodes.len() - 1 {
                self.error_message(
                    &property,msgs::A_REST_ELEMENT_MUST_BE_LAST_IN_A_DESTRUCTURING_PATTERN,
                    &[],
                );
                return None;
            }
            if self.language_version < OBJECT_SPREAD_REST {
                self.check_external_emit_helpers(&property, ExternalEmitHelpers::Rest.bits());
            }
            let mut non_rest_names: Vec<Arc<Node>> = Vec::new();
            if let Some(all_properties) = all_properties {
                for other_property in all_properties.nodes.iter() {
                    if !ast::is_spread_assignment(other_property) {
                        if let Some(name) = other_property.name() {
                            non_rest_names.push(Arc::clone(name));
                        }
                    }
                }
            }
            let t = self.get_rest_type(
                object_literal_type,
                &non_rest_names,
                object_literal_type.symbol.as_ref(),
            );
            if let Some(all_properties) = all_properties {
                self.check_grammar_for_disallowed_trailing_comma(
                    all_properties,
                    &msgs::A_REST_PARAMETER_OR_BINDING_PATTERN_MAY_NOT_HAVE_A_TRAILING_COMMA,
                );
            }
            let rest_expression = match &property.data {
                NodeData::SpreadAssignment(d) => Arc::clone(&d.expression),
                _ => property.expression().cloned().unwrap_or_else(|| Arc::clone(&property)),
            };
            self.check_destructuring_assignment(&rest_expression, &t);
            return Some(t);
        }
        self.error_message(&property,msgs::PROPERTY_ASSIGNMENT_EXPECTED, &[]);
        None
    }

    pub fn check_object_literal(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> { ::tsox_core::fntrace::enter("check_object_literal"); 
        let properties = match &node.data {
            NodeData::ObjectLiteralExpression(d) => Arc::clone(&d.properties),
            _ => return self.error_type(),
        };
        let node_symbol = self.get_symbol_of_declaration(node);
        if properties.nodes.is_empty() {
            if let Some(symbol) = &node_symbol {
                if !symbol.exports.is_empty() {
                    let mut result = self.new_anonymous_type(
                        symbol,
                        symbol.exports.clone(),
                        Vec::new(),
                        Vec::new(),
                        Vec::new(),
                    );
                    if let Some(result_mut) = Arc::get_mut(&mut result) {
                        if is_in_js_file(node) && !is_in_json_file(node) {
                            result_mut.object_flags |= ObjectFlags::JSLiteral;
                        }
                    }
                    return result;
                }
            }
        }
        self.check_node_deferred(node);
        let in_destructuring_pattern = is_assignment_target(node);
        self.check_grammar_object_literal_expression(node, in_destructuring_pattern);
        let mut all_properties_table: Option<SymbolTable> = if self.strict_null_checks {
            Some(SymbolTable::default())
        } else {
            None
        };
        let mut properties_table = SymbolTable::default();
        let mut properties_array: Vec<Arc<Symbol>> = Vec::new();
        let mut spread = self.empty_object_type();
        self.push_cached_contextual_type(node);
        let contextual_type = self.get_apparent_type_of_contextual_type(node, ContextFlags::None);
        let mut contextual_type_has_pattern = false;
        if let Some(contextual_type) = &contextual_type {
            if let Some(pattern) = pattern_for_type_of(contextual_type) {
                if ast::is_object_binding_pattern(pattern.as_ref())
                    || ast::is_object_literal_expression(pattern.as_ref())
                {
                    contextual_type_has_pattern = true;
                }
            }
        }
        let in_const_context = self.is_const_context(node);
        let mut check_flags = CheckFlags::None;
        if in_const_context {
            check_flags = CheckFlags::Readonly;
        }
        let mut object_flags = ObjectFlags::FreshLiteral;
        let mut pattern_with_computed_properties = false;
        let mut has_computed_string_property = false;
        let mut has_computed_number_property = false;
        let mut has_computed_symbol_property = false;
        for elem in properties.nodes.iter() {
            if let Some(name) = elem.name() {
                if ast::is_computed_property_name(name) {
                    self.check_computed_property_name(name);
                }
            }
        }
        let mut offset: usize = 0;
        for member_decl in properties.nodes.iter() {
            let member = self
                .get_symbol_of_declaration(member_decl)
                .unwrap_or_else(|| self.unknown_symbol());
            let mut computed_name_type: Option<Arc<Type>> = None;
            if let Some(name) = member_decl.name() {
                if name.kind == SyntaxKind::ComputedPropertyName {
                    computed_name_type = Some(self.check_computed_property_name_type(name));
                }
            }
            if ast::is_property_assignment(member_decl)
                || ast::is_shorthand_property_assignment(member_decl)
                || member_decl.kind == SyntaxKind::MethodDeclaration
            {
                let t = match member_decl.kind {
                    SyntaxKind::PropertyAssignment => {
                        self.check_property_assignment(member_decl, check_mode)
                    }
                    SyntaxKind::ShorthandPropertyAssignment => self
                        .check_shorthand_property_assignment(
                            member_decl,
                            in_destructuring_pattern,
                            check_mode,
                        ),
                    _ => self.check_object_literal_method(member_decl, check_mode),
                };
                object_flags |= t.object_flags & OBJECT_FLAGS_PROPAGATING_FLAGS;
                let name_type = computed_name_type
                    .as_ref()
                    .filter(|ct| is_type_usable_as_property_name(ct))
                    .cloned();
                let prop = if let Some(name_type) = &name_type {
                    self.new_symbol_ex(
                        SymbolFlags::Property | member.flags,
                        &get_property_name_from_type(name_type),
                        check_flags | CheckFlags::Late,
                    )
                } else {
                    self.new_symbol_ex(
                        SymbolFlags::Property | member.flags,
                        &member.name,
                        check_flags,
                    )
                };
                {
                    let links = self.value_symbol_links.get_or_default(&prop);
                    if let Some(name_type) = &name_type {
                        links.name_type = Some(Arc::clone(name_type));
                    }
                }
                let mut prop = prop;
                if let Some(p) = Arc::get_mut(&mut prop) {
                    if in_destructuring_pattern && self.has_default_value(member_decl) {
                        p.flags |= SymbolFlags::Optional;
                    } else if contextual_type_has_pattern
                        && !contextual_type
                            .as_ref()
                            .map(|ct| {
                                ct.object_flags
                                    .contains(ObjectFlags::ObjectLiteralPatternWithComputedProperties)
                            })
                            .unwrap_or(false)
                    {
                        let implied_prop = contextual_type
                            .as_ref()
                            .and_then(|ct| self.get_property_of_type(ct, &member.name));
                        if let Some(implied_prop) = implied_prop {
                            p.flags |= implied_prop.flags & SymbolFlags::Optional;
                        } else if let Some(ct) = &contextual_type {
                            let string_type = self.string_type();
                            if self.get_index_info_of_type(ct, &string_type).is_none() {
                                let error_node = member_decl.name().cloned();
                                if let Some(error_node) = error_node {
                                    let member_string = self.symbol_to_string(&member);
                                    let type_string = self.type_to_string(ct);
                                    self.error_message(
                                        &error_node,msgs::OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1,
                                        &[member_string, type_string],
                                    );
                                }
                            }
                        }
                    }
                    p.declarations = member.declarations.clone();
                    if let Some(parent) = member.parent() {
                        p.set_parent(&parent);
                    }
                    p.value_declaration = member.value_declaration.clone();
                }
                {
                    let links = self.value_symbol_links.get_or_default(&prop);
                    links.resolved_type = Some(Arc::clone(&t));
                    links.target = Some(Arc::clone(&member));
                }
                let member = prop;
                if let Some(all_properties_table) = all_properties_table.as_mut() {
                    all_properties_table.insert(member.name.clone(), Arc::clone(&member));
                }
                if contextual_type.is_some()
                    && check_mode.contains(CheckMode::Inferential)
                    && !check_mode.contains(CheckMode::SkipContextSensitive)
                    && (ast::is_property_assignment(member_decl)
                        || ast::is_method_declaration(member_decl))
                    && self.is_context_sensitive(member_decl)
                {
                    let inference_context = self.get_inference_context(node);
                    if let Some(inference_context) = inference_context {
                        let inference_node = match &member_decl.data {
                            NodeData::PropertyAssignment(pa) => Arc::clone(&pa.initializer),
                            _ => Arc::clone(member_decl),
                        };
                        add_intra_expression_inference_site_to(
                            inference_context,
                            Arc::clone(&inference_node),
                            Arc::clone(&t),
                        );
                    }
                }
                Self::record_member(
                    member,
                    computed_name_type.as_ref(),
                    &mut properties_table,
                    &mut properties_array,
                    &mut has_computed_string_property,
                    &mut has_computed_number_property,
                    &mut has_computed_symbol_property,
                    &mut pattern_with_computed_properties,
                    in_destructuring_pattern,
                    self,
                );
                continue;
            } else if member_decl.kind == SyntaxKind::SpreadAssignment {
                if !properties_array.is_empty() {
                    let created = self.create_object_literal_type_for_check(
                        node,
                        node_symbol.as_ref(),
                        &properties_table,
                        &properties_array[offset..],
                        object_flags,
                        contextual_type.as_ref(),
                        pattern_with_computed_properties,
                        in_destructuring_pattern,
                        has_computed_string_property,
                        has_computed_number_property,
                        has_computed_symbol_property,
                    );
                    spread = self.get_spread_type(
                        &spread,
                        &created,
                        node_symbol.clone(),
                        object_flags,
                        in_const_context,
                    );
                    properties_array = Vec::new();
                    properties_table = SymbolTable::default();
                    has_computed_string_property = false;
                    has_computed_number_property = false;
                    has_computed_symbol_property = false;
                }
                let spread_expression = match &member_decl.data {
                    NodeData::SpreadAssignment(d) => Arc::clone(&d.expression),
                    NodeData::SpreadElement(d) => Arc::clone(&d.expression),
                    _ => Arc::clone(member_decl),
                };
                let spread_type = self.check_expression_ex(
                    &spread_expression,
                    check_mode & CheckMode::Inferential,
                );
                let t = self.get_reduced_type(&spread_type);
                if self.is_valid_spread_type(&t) {
                    let merged_type = self.try_merge_union_of_object_type_and_empty_object(&t);
                    if all_properties_table.is_some() {
                        let own_props: Vec<(String, Arc<Node>)> = all_properties_table
                            .as_ref()
                            .unwrap()
                            .entries
                            .iter()
                            .filter_map(|(name, sym)| {
                                sym.value_declaration
                                    .clone()
                                    .or_else(|| sym.declarations.first().cloned())
                                    .map(|d| (name.clone(), d))
                            })
                            .collect();
                        let mut reported = std::collections::HashSet::new();
                        self.check_spread_prop_overrides(
                            &merged_type,
                            &own_props,
                            &mut reported,
                            member_decl,
                        );
                    }
                    offset = properties_array.len();
                    if self.is_error_type(&spread) {
                        continue;
                    }
                    spread = self.get_spread_type(
                        &spread,
                        &merged_type,
                        node_symbol.clone(),
                        object_flags,
                        in_const_context,
                    );
                } else {
                    self.error_message(
                        member_decl,msgs::SPREAD_TYPES_MAY_ONLY_BE_CREATED_FROM_OBJECT_TYPES,
                        &[],
                    );
                    spread = self.error_type();
                }
                continue;
            } else {
                self.check_node_deferred(member_decl);
            }
            Self::record_member(
                member,
                computed_name_type.as_ref(),
                &mut properties_table,
                &mut properties_array,
                &mut has_computed_string_property,
                &mut has_computed_number_property,
                &mut has_computed_symbol_property,
                &mut pattern_with_computed_properties,
                in_destructuring_pattern,
                self,
            );
        }
        self.pop_contextual_type();
        if self.is_error_type(&spread) {
            return self.error_type();
        }
        if !Arc::ptr_eq(&spread, &self.empty_object_type()) {
            if !properties_array.is_empty() {
                let created = self.create_object_literal_type_for_check(
                    node,
                    node_symbol.as_ref(),
                    &properties_table,
                    &properties_array[offset..],
                    object_flags,
                    contextual_type.as_ref(),
                    pattern_with_computed_properties,
                    in_destructuring_pattern,
                    has_computed_string_property,
                    has_computed_number_property,
                    has_computed_symbol_property,
                );
                spread = self.get_spread_type(
                    &spread,
                    &created,
                    node_symbol.clone(),
                    object_flags,
                    in_const_context,
                );
                properties_array = Vec::new();
                properties_table = SymbolTable::default();
                has_computed_string_property = false;
                has_computed_number_property = false;
            }
            let empty_object = self.empty_object_type();
            if spread.flags.contains(TypeFlags::Union) {
                let members = spread.types().unwrap_or_default();
                let mut mapped: Vec<Arc<Type>> = Vec::with_capacity(members.len());
                for member_type in members.iter() {
                    if Arc::ptr_eq(member_type, &empty_object) {
                        mapped.push(self.create_object_literal_type_for_check(
                            node,
                            node_symbol.as_ref(),
                            &properties_table,
                            &properties_array[offset..],
                            object_flags,
                            contextual_type.as_ref(),
                            pattern_with_computed_properties,
                            in_destructuring_pattern,
                            has_computed_string_property,
                            has_computed_number_property,
                            has_computed_symbol_property,
                        ));
                    } else {
                        mapped.push(Arc::clone(member_type));
                    }
                }
                return self.get_union_type(mapped);
            }
            return self.create_object_literal_type_for_check(
                node,
                node_symbol.as_ref(),
                &properties_table,
                &properties_array[offset..],
                object_flags,
                contextual_type.as_ref(),
                pattern_with_computed_properties,
                in_destructuring_pattern,
                has_computed_string_property,
                has_computed_number_property,
                has_computed_symbol_property,
            );
        }
        self.create_object_literal_type_for_check(
            node,
            node_symbol.as_ref(),
            &properties_table,
            &properties_array[offset..],
            object_flags,
            contextual_type.as_ref(),
            pattern_with_computed_properties,
            in_destructuring_pattern,
            has_computed_string_property,
            has_computed_number_property,
            has_computed_symbol_property,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn record_member(
        member: Arc<Symbol>,
        computed_name_type: Option<&Arc<Type>>,
        properties_table: &mut SymbolTable,
        properties_array: &mut Vec<Arc<Symbol>>,
        has_computed_string_property: &mut bool,
        has_computed_number_property: &mut bool,
        has_computed_symbol_property: &mut bool,
        pattern_with_computed_properties: &mut bool,
        in_destructuring_pattern: bool,
        checker: &mut Checker,
    ) { ::tsox_core::fntrace::enter("record_member"); 
        let is_non_literal_computed = computed_name_type
            .is_some_and(|ct| !ct.flags.contains(TYPE_FLAGS_STRING_OR_NUMBER_LITERAL_OR_UNIQUE));
        if is_non_literal_computed {
            let ct = computed_name_type.unwrap();
            let string_number_symbol_type = checker.string_number_symbol_type();
            let number_type = checker.number_type();
            let es_symbol_type = checker.es_symbol_type();
            if checker.is_type_assignable_to(ct, &string_number_symbol_type) {
                if checker.is_type_assignable_to(ct, &number_type) {
                    *has_computed_number_property = true;
                } else if checker.is_type_assignable_to(ct, &es_symbol_type) {
                    *has_computed_symbol_property = true;
                } else {
                    *has_computed_string_property = true;
                }
                if in_destructuring_pattern {
                    *pattern_with_computed_properties = true;
                }
            }
        } else {
            properties_table.insert(member.name.clone(), Arc::clone(&member));
        }
        properties_array.push(member);
    }

    #[allow(clippy::too_many_arguments)]
    fn create_object_literal_type_for_check(
        &mut self,
        node: &Arc<Node>,
        node_symbol: Option<&Arc<Symbol>>,
        properties_table: &SymbolTable,
        properties_slice: &[Arc<Symbol>],
        object_flags: ObjectFlags,
        contextual_type: Option<&Arc<Type>>,
        pattern_with_computed_properties: bool,
        in_destructuring_pattern: bool,
        has_computed_string_property: bool,
        has_computed_number_property: bool,
        has_computed_symbol_property: bool,
    ) -> Arc<Type> {
        let mut index_infos: Vec<Arc<IndexInfo>> = Vec::new();
        let is_readonly = self.is_const_context(node);
        if has_computed_string_property {
            let string_type = self.string_type();
            index_infos.push(Arc::new(self.get_object_literal_index_info(
                is_readonly,
                properties_slice,
                &string_type,
            )));
        }
        if has_computed_number_property {
            let number_type = self.number_type();
            index_infos.push(Arc::new(self.get_object_literal_index_info(
                is_readonly,
                properties_slice,
                &number_type,
            )));
        }
        if has_computed_symbol_property {
            let es_symbol_type = self.es_symbol_type();
            index_infos.push(Arc::new(self.get_object_literal_index_info(
                is_readonly,
                properties_slice,
                &es_symbol_type,
            )));
        }
        let fallback_symbol = self.unknown_symbol();
        let mut result = self.new_anonymous_type(
            node_symbol.unwrap_or(&fallback_symbol),
            properties_table.clone(),
            Vec::new(),
            Vec::new(),
            index_infos,
        );
        if let Some(result_mut) = Arc::get_mut(&mut result) {
            result_mut.object_flags |= object_flags
                | ObjectFlags::ObjectLiteral
                | ObjectFlags::ContainsObjectOrArrayLiteral;
            if contextual_type.is_none() && is_in_js_file(node) && !is_in_json_file(node) {
                result_mut.object_flags |= ObjectFlags::JSLiteral;
            }
            if pattern_with_computed_properties {
                result_mut.object_flags |= ObjectFlags::ObjectLiteralPatternWithComputedProperties;
            }
        }
        if in_destructuring_pattern {
            set_pattern_for_type(&result, Arc::clone(node));
        }
        result
    }
}
