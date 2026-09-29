use std::sync::Arc;

use tsox_core::diagnostics::messages_generated as msg;
use tsox_core::jsnum::Number;
use tsox_frontend::ast::node_data_generated::{
    is_element_access_expression, is_indexed_access_type_node, is_private_identifier,
};
use tsox_frontend::ast::{INTERNAL_SYMBOL_NAME_MISSING, Node, NodeData, SymbolFlags, SyntaxKind};
use tsox_frontend::scanner::mig::m3i::get_text_of_node;

use crate::checker::checker::Checker;
use crate::checker::mig::m1c_3::create_diagnostic_for_node_message;
use crate::checker::mig::m1d::r21k7_defs::get_property_name_for_property_name_node;
use crate::checker::mig::m3a_2::{new_diagnostic_chain_for_node, new_diagnostic_for_node};
use crate::checker::mig::wc1b::{every_type, is_object_literal_type, is_tuple_type};
use crate::checker::mig::wc1c::r24k17_defs::is_js_literal_type;
use crate::checker::mig::wc3::NodeAccessExt;
use crate::checker::mig::wc3_2::is_const_enum_object_type;
use crate::checker::types::{AccessFlags, LiteralValue, Type, TypeData, TypeFlags, ELEMENT_FLAGS_VARIABLE};
use crate::checker::utilities_has_only_expression_initialization::get_assignment_target_kind;
use crate::checker::utilities_is_optional_symbol::is_numeric_literal_name;
use crate::checker::utilities_token_is_identifier_or_keyword::{
    get_property_name_from_type, is_type_usable_as_property_name, AssignmentKind,
};

use super::get_index_node_for_access_expression;

fn is_property_name(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::Identifier
            | SyntaxKind::PrivateIdentifier
            | SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::ComputedPropertyName
    )
}

fn node_arc(n: &Node) -> Arc<Node> {
    let ptr: *const Node = n;
    // SAFETY: AST 节点均存活于 Arc<Node> 分配内(r23k4_defs.rs 同型桥)
    unsafe {
        Arc::increment_strong_count(ptr);
        Arc::from_raw(ptr)
    }
}

fn literal_value_arg(t: &Type) -> String {
    match t.literal_value() {
        Some(LiteralValue::String(s)) => s.clone(),
        Some(LiteralValue::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

impl Checker {
    pub fn get_property_name_from_index(&self, index_type: &Arc<Type>, access_node: Option<&Node>) -> String {
        if is_type_usable_as_property_name(index_type) {
            return get_property_name_from_type(index_type);
        }
        if let Some(node) = access_node
            && is_property_name(node)
        {
            return get_property_name_for_property_name_node(node);
        }
        INTERNAL_SYMBOL_NAME_MISSING.to_string()
    }

    pub fn get_suggestion_for_nonexistent_property(&mut self, name: &str, containing_type: &Arc<Type>) -> String {
        let properties = self.get_properties_of_type(containing_type);
        self.get_spelling_suggestion_for_name(name, properties, SymbolFlags::VALUE)
            .map(|s| s.name.clone())
            .unwrap_or_default()
    }

    pub fn get_property_type_for_index_type(
        &mut self,
        original_object_type: &Arc<Type>,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
        full_index_type: &Arc<Type>,
        access_node: Option<&Node>,
        access_flags: AccessFlags,
    ) -> Option<Arc<Type>> {
        let access_node = access_node.map(node_arc);
        let access_expression: Option<Arc<Node>> = match &access_node {
            Some(n) if is_element_access_expression(n) => Some(Arc::clone(n)),
            _ => None,
        };
        let mut prop_name = String::new();
        let mut has_prop_name = false;
        if !access_node.as_ref().is_some_and(|n| is_private_identifier(n)) {
            prop_name = self.get_property_name_from_index(index_type, access_node.as_deref());
            has_prop_name = prop_name != INTERNAL_SYMBOL_NAME_MISSING;
        }
        if has_prop_name {
            if access_flags.intersects(AccessFlags::Contextual) {
                return Some(
                    self.get_type_of_property_of_contextual_type(object_type, &prop_name)
                        .unwrap_or_else(|| self.any_type()),
                );
            }
            let prop = self.get_property_of_type(object_type, &prop_name);
            if let Some(prop) = &prop {
                if access_flags.intersects(AccessFlags::REPORT_DEPRECATED)
                    && let Some(node) = access_node.as_ref()
                    && !prop.declarations.is_empty()
                    && self.is_deprecated_symbol(prop)
                    && self.is_uncalled_function_reference(node, prop)
                {
                    let deprecated_node: Arc<Node> = if let Some(ae) = &access_expression {
                        Arc::clone(&ae.as_element_access_expression().argument_expression)
                    } else if is_indexed_access_type_node(node) {
                        match &node.data {
                            NodeData::IndexedAccessTypeNode(d) => Arc::clone(&d.index_type),
                            _ => unreachable!(),
                        }
                    } else {
                        Arc::clone(node)
                    };
                    self.add_deprecated_suggestion(&deprecated_node, &prop.declarations, &prop_name);
                }
                if let Some(ae) = &access_expression {
                    let receiver = ae.expression().expect("ElementAccessExpression.expression");
                    let self_type_access = self.is_self_type_access(receiver, object_type);
                    self.mark_property_as_referenced_ex(prop, Some(ae), Some(self_type_access));
                    let assignment_kind = get_assignment_target_kind(ae);
                    if self.is_assignment_to_readonly_entity(ae, prop, assignment_kind) {
                        let arg = Arc::clone(&ae.as_element_access_expression().argument_expression);
                        let arg_str = self.symbol_to_string(prop);
                        self.error_message(&arg,msg::CANNOT_ASSIGN_TO_0_BECAUSE_IT_IS_A_READ_ONLY_PROPERTY, &[arg_str]);
                        return None;
                    }
                    if access_flags.intersects(AccessFlags::CACHE_SYMBOL) {
                        self.symbol_node_links
                            .get_or_default(access_node.as_deref().expect("access node"))
                            .resolved_symbol = Some(Arc::clone(prop));
                    }
                    if self.is_this_property_access_in_constructor(ae, prop) {
                        return Some(self.auto_type());
                    }
                }
                let prop_type = if access_flags.intersects(AccessFlags::WRITING) {
                    self.get_write_type_of_symbol(prop)
                } else {
                    self.get_type_of_symbol(prop)
                };
                if let Some(ae) = &access_expression
                    && get_assignment_target_kind(ae) != AssignmentKind::Definite
                {
                    return Some(self.get_flow_type_of_reference(ae, &prop_type));
                } else if access_node.as_ref().is_some_and(|n| is_indexed_access_type_node(n))
                    && self.contains_missing_type(&prop_type)
                {
                    return Some(self.get_union_type(vec![prop_type, self.undefined_type()]));
                } else {
                    return Some(prop_type);
                }
            }
            if every_type(object_type, &is_tuple_type) && is_numeric_literal_name(&prop_name) {
                let index = Number::from_string(&prop_name);
                let zero = Number::from(0);
                let all_fixed = every_type(object_type, &|t: &Arc<Type>| match &t.data {
                    TypeData::Tuple(tuple) => !tuple.combined_flags.intersects(ELEMENT_FLAGS_VARIABLE),
                    _ => false,
                });
                if access_node.is_some() && all_fixed && !access_flags.intersects(AccessFlags::AllowMissing) {
                    let index_node = get_index_node_for_access_expression(access_node.as_deref().expect("access node"));
                    if is_tuple_type(object_type) {
                        if index < zero {
                            self.error_message(&node_arc(index_node),msg::A_TUPLE_TYPE_CANNOT_BE_INDEXED_WITH_A_NEGATIVE_VALUE, &[]);
                            return Some(self.undefined_type());
                        }
                        let arg1 = self.type_to_string(object_type);
                        let arg2 = self.get_type_reference_arity(object_type).to_string();
                        self.error_message(
                            &node_arc(index_node),msg::TUPLE_TYPE_0_OF_LENGTH_1_HAS_NO_ELEMENT_AT_INDEX_2,
                            &[arg1, arg2, prop_name.clone()],
                        );
                    } else {
                        let arg2 = self.type_to_string(object_type);
                        self.error_message(
                            &node_arc(index_node),msg::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                            &[prop_name.clone(), arg2],
                        );
                    }
                }
                if index >= zero {
                    let number_index_info = self.get_index_info_of_type(object_type, &self.number_type());
                    self.error_if_writing_to_readonly_index(number_index_info.as_deref(), object_type, access_expression.as_ref());
                    let undefined_like = if access_flags.intersects(AccessFlags::INCLUDE_UNDEFINED) {
                        Some(self.missing_type())
                    } else {
                        None
                    };
                    return Some(self.get_tuple_element_type_out_of_start_count(object_type, index, undefined_like.as_ref()));
                }
            }
        }
        if !index_type.flags.intersects(TypeFlags::NULLABLE)
            && self.is_type_assignable_to_kind(index_type, TypeFlags::STRING_LIKE | TypeFlags::NUMBER_LIKE | TypeFlags::ES_SYMBOL_LIKE)
        {
            if object_type.flags.intersects(TypeFlags::ANY | TypeFlags::NEVER) {
                return Some(Arc::clone(object_type));
            }
            let mut index_info = self.get_applicable_index_info(object_type, index_type);
            if index_info.is_none() {
                index_info = self.get_index_info_of_type(object_type, &self.string_type());
            }
            if let Some(index_info) = index_info {
                let number_key = self.number_type();
                let string_key = self.string_type();
                if access_flags.intersects(AccessFlags::NoIndexSignatures)
                    && !index_info.key_type.as_ref().is_some_and(|k| Arc::ptr_eq(k, &number_key))
                {
                    if let Some(ae) = &access_expression {
                        if access_flags.intersects(AccessFlags::WRITING) {
                            let arg1 = self.type_to_string(original_object_type);
                            self.error_message(ae,msg::TYPE_0_IS_GENERIC_AND_CAN_ONLY_BE_INDEXED_FOR_READING, &[arg1]);
                        } else {
                            let arg1 = self.type_to_string(index_type);
                            let arg2 = self.type_to_string(original_object_type);
                            self.error_message(ae,msg::TYPE_0_CANNOT_BE_USED_TO_INDEX_TYPE_1, &[arg1, arg2]);
                        }
                    }
                    return None;
                }
                if let Some(node) = access_node.as_deref()
                    && index_info.key_type.as_ref().is_some_and(|k| Arc::ptr_eq(k, &string_key))
                    && !self.is_type_assignable_to_kind(index_type, TypeFlags::STRING | TypeFlags::NUMBER)
                {
                    let index_node = get_index_node_for_access_expression(node);
                    let arg1 = self.type_to_string(index_type);
                    self.error_message(&node_arc(index_node),msg::TYPE_0_CANNOT_BE_USED_AS_AN_INDEX_TYPE, &[arg1]);
                    let value_type = index_info.value_type.clone().expect("IndexInfo.valueType");
                    if access_flags.intersects(AccessFlags::INCLUDE_UNDEFINED) {
                        return Some(self.get_union_type(vec![value_type, self.missing_type()]));
                    }
                    return Some(value_type);
                }
                self.error_if_writing_to_readonly_index(Some(index_info.as_ref()), object_type, access_expression.as_ref());
                let value_type = index_info.value_type.clone().expect("IndexInfo.valueType");
                let enum_object_access = object_type.symbol.as_ref().is_some_and(|o| {
                    o.flags.intersects(SymbolFlags::RegularEnum | SymbolFlags::ConstEnum)
                        && index_type.flags.intersects(TypeFlags::ENUM_LITERAL)
                        && index_type
                            .symbol
                            .as_ref()
                            .is_some_and(|i| self.get_parent_of_symbol(i).is_some_and(|p| Arc::ptr_eq(&p, o)))
                });
                if access_flags.intersects(AccessFlags::INCLUDE_UNDEFINED) && !enum_object_access {
                    return Some(self.get_union_type(vec![value_type, self.missing_type()]));
                }
                return Some(value_type);
            }
            if index_type.flags.intersects(TypeFlags::NEVER) {
                return Some(self.never_type());
            }
            if is_js_literal_type(self, object_type) {
                return Some(self.any_type());
            }
            if let Some(ae) = &access_expression
                && !is_const_enum_object_type(object_type)
            {
                if is_object_literal_type(object_type)
                    && self.no_implicit_any
                    && index_type.flags.intersects(TypeFlags::STRING_LITERAL | TypeFlags::NUMBER_LITERAL)
                {
                    let arg1 = literal_value_arg(index_type);
                    let arg2 = self.type_to_string(object_type);
                    let diagnostic = create_diagnostic_for_node_message(ae,msg::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1, &[arg1, arg2]);
                    self.add_diagnostic((*diagnostic).clone());
                    return Some(self.undefined_type());
                } else if is_object_literal_type(object_type)
                    && index_type.flags.intersects(TypeFlags::NUMBER | TypeFlags::STRING)
                {
                    let mut types: Vec<Arc<Type>> = Vec::new();
                    if let Some(structured) = object_type.as_structured() {
                        for p in &structured.properties {
                            types.push(self.get_type_of_symbol(p));
                        }
                    }
                    types.push(self.undefined_type());
                    return Some(self.get_union_type(types));
                }
                let is_global_this = object_type
                    .symbol
                    .as_ref()
                    .is_some_and(|o| self.global_this_symbol.as_ref().is_some_and(|g| Arc::ptr_eq(o, g)));
                let global_block_scoped = is_global_this
                    && has_prop_name
                    && self.global_this_symbol.as_ref().is_some_and(|g| {
                        g.exports
                            .get(&prop_name)
                            .is_some_and(|e| e.flags.intersects(SymbolFlags::FunctionScopedVariable | SymbolFlags::BlockScopedVariable))
                    });
                if global_block_scoped {
                    let arg2 = self.type_to_string(object_type);
                    self.error_message(ae,msg::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1, &[prop_name.clone(), arg2]);
                } else if self.no_implicit_any && !access_flags.intersects(AccessFlags::SuppressNoImplicitAnyError) {
                    if has_prop_name && self.type_has_static_property(&prop_name, object_type) {
                        let type_name = self.type_to_string(object_type);
                        let arg_node = Arc::clone(&ae.as_element_access_expression().argument_expression);
                        let arg3 = format!("{}[{}]", type_name, get_text_of_node(&arg_node));
                        self.error_message(
                            ae,msg::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1_DID_YOU_MEAN_TO_ACCESS_THE_STATIC_MEMBER_2_INSTEAD,
                            &[prop_name.clone(), type_name, arg3],
                        );
                    } else if self.get_index_info_of_type(object_type, &self.number_type()).is_some() {
                        let arg = Arc::clone(&ae.as_element_access_expression().argument_expression);
                        self.error_message(&arg,msg::ELEMENT_IMPLICITLY_HAS_AN_ANY_TYPE_BECAUSE_INDEX_EXPRESSION_IS_NOT_OF_TYPE_NUMBER, &[]);
                    } else {
                        let mut suggestion = String::new();
                        if has_prop_name {
                            suggestion = self.get_suggestion_for_nonexistent_property(&prop_name, object_type);
                        }
                        if !suggestion.is_empty() {
                            let arg2 = self.type_to_string(object_type);
                            let arg = Arc::clone(&ae.as_element_access_expression().argument_expression);
                            self.error_message(&arg,msg::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1_DID_YOU_MEAN_2, &[prop_name.clone(), arg2, suggestion]);
                        } else {
                            suggestion = self.get_suggestion_for_nonexistent_index_signature(object_type, ae, index_type);
                            if !suggestion.is_empty() {
                                let arg1 = self.type_to_string(object_type);
                                self.error_message(ae,msg::ELEMENT_IMPLICITLY_HAS_AN_ANY_TYPE_BECAUSE_TYPE_0_HAS_NO_INDEX_SIGNATURE_DID_YOU_MEAN_TO_CALL_1, &[arg1, suggestion]);
                            } else {
                                let diagnostic = if index_type.flags.intersects(TypeFlags::ENUM_LITERAL) {
                                    let arg1 = format!("[{}]", self.type_to_string(index_type));
                                    let arg2 = self.type_to_string(object_type);
                                    Some(new_diagnostic_for_node(Some(ae), msg::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1, vec![arg1, arg2]))
                                } else if index_type.flags.intersects(TypeFlags::UNIQUE_ES_SYMBOL) {
                                    let symbol = index_type.symbol.as_ref().expect("unique ESSymbol type symbol");
                                    let symbol_name = self.get_fully_qualified_name(symbol, Some(ae.as_ref()));
                                    let arg2 = self.type_to_string(object_type);
                                    Some(new_diagnostic_for_node(Some(ae), msg::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1, vec![format!("[{}]", symbol_name), arg2]))
                                } else if index_type.flags.intersects(TypeFlags::STRING_LITERAL) {
                                    let arg1 = literal_value_arg(index_type);
                                    let arg2 = self.type_to_string(object_type);
                                    Some(new_diagnostic_for_node(Some(ae), msg::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1, vec![arg1, arg2]))
                                } else if index_type.flags.intersects(TypeFlags::NUMBER_LITERAL) {
                                    let arg1 = literal_value_arg(index_type);
                                    let arg2 = self.type_to_string(object_type);
                                    Some(new_diagnostic_for_node(Some(ae), msg::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1, vec![arg1, arg2]))
                                } else if index_type.flags.intersects(TypeFlags::NUMBER | TypeFlags::STRING) {
                                    let arg1 = self.type_to_string(index_type);
                                    let arg2 = self.type_to_string(object_type);
                                    Some(new_diagnostic_for_node(Some(ae), msg::NO_INDEX_SIGNATURE_WITH_A_PARAMETER_OF_TYPE_0_WAS_FOUND_ON_TYPE_1, vec![arg1, arg2]))
                                } else {
                                    None
                                };
                                let arg1 = self.type_to_string(full_index_type);
                                let arg2 = self.type_to_string(object_type);
                                let chained = new_diagnostic_chain_for_node(
                                    diagnostic.as_ref(),
                                    Some(ae),
                                    msg::ELEMENT_IMPLICITLY_HAS_AN_ANY_TYPE_BECAUSE_EXPRESSION_OF_TYPE_0_CAN_T_BE_USED_TO_INDEX_TYPE_1,
                                    vec![arg1, arg2],
                                );
                                self.add_diagnostic(chained);
                            }
                        }
                    }
                }
                return None;
            }
        }
        if access_flags.intersects(AccessFlags::AllowMissing) && is_object_literal_type(object_type) {
            return Some(self.undefined_type());
        }
        if is_js_literal_type(self, object_type) {
            return Some(self.any_type());
        }
        if let Some(node) = access_node.as_deref() {
            let index_node = get_index_node_for_access_expression(node);
            if index_node.kind != SyntaxKind::BigIntLiteral
                && index_type.flags.intersects(TypeFlags::STRING_LITERAL | TypeFlags::NUMBER_LITERAL)
            {
                let arg1 = literal_value_arg(index_type);
                let arg2 = self.type_to_string(object_type);
                self.error_message(&node_arc(index_node),msg::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1, &[arg1, arg2]);
            } else if index_type.flags.intersects(TypeFlags::STRING | TypeFlags::NUMBER) {
                let arg1 = self.type_to_string(object_type);
                let arg2 = self.type_to_string(index_type);
                self.error_message(&node_arc(index_node),msg::TYPE_0_HAS_NO_MATCHING_INDEX_SIGNATURE_FOR_TYPE_1, &[arg1, arg2]);
            } else {
                let type_string = if index_node.kind == SyntaxKind::BigIntLiteral {
                    "bigint".to_string()
                } else {
                    self.type_to_string(index_type)
                };
                self.error_message(&node_arc(index_node),msg::TYPE_0_CANNOT_BE_USED_AS_AN_INDEX_TYPE, &[type_string]);
            }
        }
        if index_type.flags.intersects(TypeFlags::ANY) {
            return Some(Arc::clone(index_type));
        }
        None
    }
}
