#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::types::*;
use crate::checker::utilities_token_is_identifier_or_keyword::is_unit_type;
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated as msg;
use tsox_frontend::ast::mig::m3e::{get_function_flags, FunctionFlags};
use tsox_frontend::ast::mig::m3f_4::is_declaration_name;
use tsox_frontend::ast::mig::m3g::is_name_of_heritage_clause_type_reference;
use tsox_frontend::ast::mig::m3g_2::is_right_side_of_qualified_name_or_property_access;
use tsox_frontend::ast::mig::m3g_3::is_part_of_type_node;
use tsox_frontend::ast::mig::w7a::{
    is_expression_with_type_arguments_in_class_extends_clause, is_jsdoc_name_reference_context,
};
use tsox_frontend::ast::mig::x4ast::get_host_signature_from_jsdoc;
use crate::checker::jsx_impl_chunk::is_jsx_intrinsic_tag_name;
use crate::checker::mig::m1f::r24k9_defs::is_expression_node;
use crate::checker::mig::m3a_2::is_in_name_of_expression_with_type_arguments_or_heritage_type_reference;
use crate::checker::utilities_get_assignment_target::is_type_reference_identifier;
use crate::checker::utilities_is_private_within_ambient::{
    is_import_type_qualifier_part, is_in_right_side_of_import_or_export_assignment,
};
use tsox_frontend::ast::{self, Node, Symbol, SyntaxKind};

impl Checker {
    pub fn get_return_type_from_body(&mut self, function: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> {
        let Some(body) = function.body() else {
            return self.error_type();
        };
        let function_flags = get_function_flags(Some(function));
        let is_async = function_flags.contains(FunctionFlags::ASYNC);
        let is_generator = function_flags.contains(FunctionFlags::GENERATOR);
        let mut return_type: Option<Arc<Type>> = None;
        let mut yield_type: Option<Arc<Type>> = None;
        let mut next_type: Option<Arc<Type>> = None;
        let mut fallback_return_type = self.void_type();
        if !ast::is_block(&body) {
            let mut rt = self.check_expression_cached_ex(&body, check_mode - CheckMode::SkipGenericFunctions);
            if self.is_const_context(&body) {
                rt = self.get_regular_type_of_literal_type(&rt);
            }
            if is_async {
                let awaited = self.check_awaited_type(
                    &rt,
                    false,
                    Some(function),
                    Some(&msg::THE_RETURN_TYPE_OF_AN_ASYNC_FUNCTION_MUST_EITHER_BE_A_VALID_PROMISE_OR_MUST_NOT_CONTAIN_A_CALLABLE_THEN_MEMBER),
                );
                rt = self.unwrap_awaited_type(&awaited);
            }
            return_type = Some(rt);
            } else if is_generator {
                let (return_types, is_never_returning) = self.check_and_aggregate_return_expression_types(function, check_mode);
                if is_never_returning {
                    fallback_return_type = self.never_type();
                } else if let Some(return_types) = return_types.filter(|rts| !rts.is_empty()) {
                    return_type = Some(self.get_union_type_ex(return_types, UnionReduction::Subtype));
                }
                let (yield_types, next_types) = self.check_and_aggregate_yield_operand_types(function, check_mode);
                if !yield_types.is_empty() {
                    yield_type = Some(self.get_union_type_ex(yield_types, UnionReduction::Subtype));
                }
                if !next_types.is_empty() {
                    next_type = Some(self.get_intersection_type(next_types));
                }
            } else {
                let (types, is_never_returning) = self.check_and_aggregate_return_expression_types(function, check_mode);
                if is_never_returning {
                    if is_async {
                        return self.create_promise_return_type(function, &self.never_type());
                    }
                    return self.never_type();
                }
                if types.as_ref().map_or(true, |ts| ts.is_empty()) {
                let contextual_return_type = self.get_contextual_return_type(function, ContextFlags::None);
                let mut rt = self.void_type();
                if let Some(contextual_return_type) = contextual_return_type {
                    let unwrapped = self
                        .unwrap_return_type(&contextual_return_type, function_flags)
                        .unwrap_or_else(|| self.void_type());
                    if unwrapped.flags.intersects(TypeFlags::Undefined) {
                        rt = self.undefined_type();
                    }
                }
                if is_async {
                    return self.create_promise_return_type(function, &rt);
                }
                return rt;
            }
            return_type = Some(self.get_union_type_ex(types.unwrap_or_default(), UnionReduction::Subtype));
        }
        if return_type.is_some() || yield_type.is_some() || next_type.is_some() {
            if let Some(yield_type) = &yield_type {
                self.report_errors_from_widening(function, yield_type, WideningKind::GeneratorYield);
            }
            if let Some(return_type) = &return_type {
                self.report_errors_from_widening(function, return_type, WideningKind::FunctionReturn);
            }
            if let Some(next_type) = &next_type {
                self.report_errors_from_widening(function, next_type, WideningKind::GeneratorNext);
            }
            let any_unit = return_type.as_ref().map(|t| is_unit_type(t)).unwrap_or(false)
                || yield_type.as_ref().map(|t| is_unit_type(t)).unwrap_or(false)
                || next_type.as_ref().map(|t| is_unit_type(t)).unwrap_or(false);
            if any_unit {
                let contextual_signature = self.get_contextual_signature_for_function_like_declaration(function);
                let contextual_type = match &contextual_signature {
                    None => None,
                    Some(sig) if self
                        .get_signature_from_declaration(function)
                        .map(|s| Arc::ptr_eq(&s, sig))
                        .unwrap_or(false) =>
                    {
                        if !is_generator {
                            return_type.clone()
                        } else {
                            None
                        }
                    }
                    Some(sig) => {
                        let signature_return_type = self.get_return_type_of_signature(sig);
                        signature_return_type
                            .map(|rt| self.instantiate_contextual_type(&rt, function, ContextFlags::None))
                    }
                };
                if is_generator {
                    yield_type = self.get_widened_literal_like_type_for_contextual_iteration_type_if_needed(
                        yield_type.as_ref(),
                        contextual_type.as_ref(),
                        IterationTypeKind::YIELD,
                        is_async,
                    );
                    return_type = self.get_widened_literal_like_type_for_contextual_iteration_type_if_needed(
                        return_type.as_ref(),
                        contextual_type.as_ref(),
                        IterationTypeKind::RETURN,
                        is_async,
                    );
                    next_type = self.get_widened_literal_like_type_for_contextual_iteration_type_if_needed(
                        next_type.as_ref(),
                        contextual_type.as_ref(),
                        IterationTypeKind::NEXT,
                        is_async,
                    );
                } else {
                    return_type = self.get_widened_literal_like_type_for_contextual_return_type_if_needed(
                        return_type.as_ref(),
                        contextual_type.as_ref(),
                        is_async,
                    );
                }
            }
            yield_type = yield_type.map(|t| self.get_widened_type(&t));
            return_type = return_type.map(|t| self.get_widened_type(&t));
            next_type = next_type.map(|t| self.get_widened_type(&t));
        }
        let return_type = return_type.unwrap_or(fallback_return_type);
        if is_generator {
            let yield_type = yield_type.unwrap_or_else(|| self.never_type());
            let next_type = match next_type {
                Some(next_type) => next_type,
                None => self
                    .get_contextual_iteration_type(IterationTypeKind::NEXT, Some(function))
                    .unwrap_or_else(|| self.unknown_type()),
            };
            return self.create_generator_type(&yield_type, &return_type, &next_type, is_async);
        }
        if is_async {
            return self.create_promise_type(&return_type);
        }
        return_type
    }
}

impl Checker {
    pub fn get_symbol_of_name_or_property_access_expression(&mut self, name: &Arc<Node>) -> Option<Arc<Symbol>> {
        if is_declaration_name(name) {
            return name.parent().and_then(|p| self.get_symbol_of_node(&p));
        }
        if name.parent().is_some_and(|p| p.kind == SyntaxKind::ExportAssignment) && ast::is_entity_name_expression(name) {
            let success = self.resolve_entity_name(
                name,
                SymbolFlags::VALUE | SymbolFlags::TYPE | SymbolFlags::NAMESPACE | SymbolFlags::Alias,
                true,
                false,
                None,
            );
            if let Some(success) = success {
                let unknown = self.unknown_symbol();
                if !Arc::ptr_eq(&success, &unknown) {
                    return Some(success);
                }
            }
        } else if ast::is_entity_name(name) && is_in_right_side_of_import_or_export_assignment(name) {
            let import_equals_declaration =
                ast::find_ancestor_kind(name, SyntaxKind::ImportEqualsDeclaration)
                    .expect("ImportEqualsDeclaration should be defined");
            let _ = import_equals_declaration;
            return self.get_symbol_of_part_of_right_hand_side_of_import_equals(name);
        }
        if ast::is_entity_name(name) {
            if let Some(possible_import_node) = is_import_type_qualifier_part(name) {
                self.get_type_from_type_node(&possible_import_node);
                let sym = self.get_resolved_symbol_or_nil(name);
                if let Some(sym) = sym {
                    let unknown = self.unknown_symbol();
                    if !Arc::ptr_eq(&sym, &unknown) {
                        return Some(sym);
                    }
                }
                return None;
            }
        }
        let mut name = Arc::clone(name);
        while is_right_side_of_qualified_name_or_property_access(&name) {
            name = name.parent().expect("qualified name should have a parent");
        }
        if is_in_name_of_expression_with_type_arguments_or_heritage_type_reference(&name) {
            let mut meaning = SymbolFlags::NAMESPACE;
            let parent_kind = name.parent().map(|p| p.kind);
            if parent_kind == Some(SyntaxKind::ExpressionWithTypeArguments)
                || parent_kind == Some(SyntaxKind::TypeReference)
            {
                meaning = if is_part_of_type_node(&name) {
                    SymbolFlags::TYPE
                } else {
                    SymbolFlags::VALUE
                };
                if is_expression_with_type_arguments_in_class_extends_clause(
                    &name.parent().expect("parent should exist"),
                ) {
                    meaning.insert(SymbolFlags::VALUE);
                }
            }
            meaning.insert(SymbolFlags::Alias);
            let mut entity_name_symbol: Option<Arc<Symbol>> = None;
            if ast::is_entity_name_expression(&name) {
                entity_name_symbol = self.resolve_entity_name(&name, meaning, true, false, None);
            }
            if let Some(entity_name_symbol) = entity_name_symbol {
                return Some(entity_name_symbol);
            }
        }
        if is_expression_node(&name) {
            if ast::node_is_missing(Some(&name)) {
                return None;
            }
            let is_jsdoc = is_jsdoc_name_reference_context(&name);
            if ast::is_identifier(&name) {
                if ast::is_jsx_tag_name(&name) && is_jsx_intrinsic_tag_name(&name) {
                    let parent = name.parent().expect("jsx tag name should have a parent");
                    let symbol = self.get_intrinsic_tag_symbol(&parent);
                    return symbol.filter(|s| !Arc::ptr_eq(s, &self.unknown_symbol()));
                }
                let meaning = if is_jsdoc {
                    SymbolFlags::VALUE | SymbolFlags::TYPE | SymbolFlags::NAMESPACE
                } else {
                    SymbolFlags::VALUE
                };
                let location = if is_jsdoc {
                    get_host_signature_from_jsdoc(&name)
                } else {
                    None
                };
                let mut result = self.resolve_entity_name(&name, meaning, true, true, location.as_ref());
                if result.is_none() && is_jsdoc {
                    if let Some(container) = ast::find_ancestor(&name, ast::is_class_or_interface_like) {
                        let symbol = self
                            .get_symbol_of_declaration(&container)
                            .unwrap_or_else(|| panic!("missing symbol"));
                        let exports = self.get_exports_of_symbol(&symbol);
                        result = self
                            .get_symbol(&exports, &name.text(), meaning)
                            .map(|s| self.get_merged_symbol(&s))
                            .or_else(|| {
                                let declared_type = self.get_declared_type_of_symbol(&symbol);
                                self.get_property_of_type(&declared_type, &name.text())
                            });
                    }
                }
                return result;
            } else if ast::is_private_identifier(&name) {
                return self.get_symbol_for_private_identifier_expression(&name);
            } else if ast::is_property_access_expression(&name) || ast::is_qualified_name(&name) {
                if self
                    .symbol_node_links
                    .get(&name)
                    .and_then(|l| l.resolved_symbol.clone())
                    .is_some()
                {
                    return self
                        .symbol_node_links
                        .get(&name)
                        .and_then(|l| l.resolved_symbol.clone());
                }
                if ast::is_property_access_expression(&name) {
                    self.check_property_access_expression(&name, CheckMode::Normal, false);
                    if self
                        .symbol_node_links
                        .get(&name)
                        .and_then(|l| l.resolved_symbol.clone())
                        .is_none()
                        && name.name().is_none_or(|n| !ast::is_private_identifier(n))
                    {
                        let expression_type = self
                            .check_expression_ex(name.expression().expect("expression"), CheckMode::Normal);
                        let literal_type = self
                            .get_literal_type_from_property_name(name.name().expect("property access should have a name"))
                            .unwrap_or_else(|| self.unknown_type());
                        let applicable = self.get_applicable_index_symbol(&expression_type, &literal_type);
                        if let Some(applicable) = applicable {
                            self.symbol_node_links.get_or_default(&name).resolved_symbol = Some(applicable);
                        }
                    }
                } else {
                    self.check_qualified_name(&name, CheckMode::Normal);
                }
                let resolved = self
                    .symbol_node_links
                    .get(&name)
                    .and_then(|l| l.resolved_symbol.clone());
                if resolved.is_none() && is_jsdoc && ast::is_qualified_name(&name) {
                    return self.resolve_jsdoc_member_name(Some(&name));
                }
                return resolved;
            }
        } else if ast::is_entity_name(&name) && is_type_reference_identifier(&name) {
            let meaning = if name.parent().is_some_and(|p| p.kind == SyntaxKind::TypeReference) {
                SymbolFlags::TYPE
            } else {
                SymbolFlags::NAMESPACE
            };
            let symbol = self.resolve_entity_name(&name, meaning, true, true, None);
            if let Some(symbol) = symbol {
                if !Arc::ptr_eq(&symbol, &self.unknown_symbol()) {
                    return Some(symbol);
                }
            }
            if is_name_of_heritage_clause_type_reference(&name) {
                return None;
            }
            return self.get_unresolved_symbol_for_entity_name(&name);
        }
        if name.parent().is_some_and(|p| p.kind == SyntaxKind::TypePredicate) {
            return self.resolve_entity_name(
                &name,
                SymbolFlags::FunctionScopedVariable,
                true,
                false,
                None,
            );
        }
        None
    }
}
