#![allow(unused_imports)]

use crate::checker::checker_checker::*;
use crate::checker::checker_iteration::IterationUse;
use crate::checker::mig::m3a_2::new_diagnostic_for_node;
use crate::checker::relater_relation::RelationKind;
use crate::checker::types::TypeData;
use crate::checker::utilities_token_is_identifier_or_keyword::{
    get_property_name_from_type, is_type_usable_as_property_name,
};
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated::*;
use tsox_core::jsnum;
use tsox_frontend::ast::{self, Node, Symbol, SyntaxKind};
use tsox_frontend::ast::Diagnostic;
use super::m2e::r19k3_defs::NodeAccessExtR19k3;
use crate::checker::mig::wc2::r23k3_defs::filter_type_self;
use super::m1c_3::create_diagnostic_for_node;

pub static TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1_WITH_EXACT_OPTIONAL_PROPERTY_TYPES_COLON_TRUE_CONSIDER_ADDING_UNDEFINED_TO_THE_TYPE_OF_THE_TARGET: tsox_core::diagnostics::Message = tsox_core::diagnostics::Message {
    code: 2412,
    category: tsox_core::diagnostics::Category::Error,
    key: "Type_0_is_not_assignable_to_type_1_with_exactOptionalPropertyTypes_Colon_true_Consider_adding_undefi_2412",
    text: "Type '{0}' is not assignable to type '{1}' with 'exactOptionalPropertyTypes: true'. Consider adding 'undefined' to the type of the target.",
    reports_unnecessary: false,
    elided_in_compatibility_pyramid: false,
    reports_deprecated: false,
};
use super::wc3::NodeAccessExt;

impl TupleTypeData {
    pub fn element_flags(&self) -> Vec<ElementFlags> {
        self.element_infos.iter().map(|info| info.flags).collect()
    }
}

impl Checker {
    pub fn each_type_contained_in(&self, source: &Arc<Type>, types: &[Arc<Type>]) -> bool {
        if let TypeData::Union(data) = &source.data {
            return !data
                .union_or_intersection
                .types
                .iter()
                .any(|t| !types.iter().any(|x| Arc::ptr_eq(x, t)));
        }
        types.iter().any(|t| Arc::ptr_eq(t, source))
    }

    pub fn generate_jsx_children<'a>(
        &'a mut self,
        node: &Arc<Node>,
        get_invalid_text_diagnostic: impl Fn() -> (Option<&'static str>, Vec<String>) + Clone + 'static,
    ) -> impl Iterator<Item = JsxElaborationElement> + 'a {
        let mut member_offset = 0usize;
        let mut children: Vec<Arc<Node>> = Vec::new();
        tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
            children.push(Arc::clone(child));
            false
        });
        children.into_iter().enumerate().filter_map(move |(i, child)| {
            let name_type = self.get_number_literal_type(jsnum::Number((i as f64) - member_offset as f64));
            let e = self.get_elaboration_element_for_jsx_child(&child, &name_type, get_invalid_text_diagnostic.clone());
            if e.error_node.is_some() {
                Some(e)
            } else {
                member_offset += 1;
                None
            }
        })
    }

    pub fn get_elaboration_element_for_jsx_child(
        &mut self,
        child: &Arc<Node>,
        name_type: &Arc<Type>,
        get_invalid_text_diagnostic: impl Fn() -> (Option<&'static str>, Vec<String>) + Clone + 'static,
    ) -> JsxElaborationElement {
        match child.kind {
            SyntaxKind::JsxExpression => JsxElaborationElement {
                error_node: Some(Arc::clone(child)),
                inner_expression: child.expression().cloned(),
                name_type: Some(Arc::clone(name_type)),
                create_diagnostic: None,
            },
            SyntaxKind::JsxText => {
                if child.as_jsx_text().contains_only_trivia_white_spaces {
                    return JsxElaborationElement::default();
                }
                let name_type = Arc::clone(name_type);
                JsxElaborationElement {
                    error_node: Some(Arc::clone(child)),
                    inner_expression: None,
                    name_type: Some(name_type),
                    create_diagnostic: Some(Box::new(move |prop| {
                        let (error_message, error_args) = get_invalid_text_diagnostic();
                        Arc::new(new_diagnostic_for_node(
                            Some(prop),
                            *key_to_message(error_message.unwrap()).unwrap(),
                            error_args,
                        ))
                    })),
                }
            }
            SyntaxKind::JsxElement | SyntaxKind::JsxSelfClosingElement | SyntaxKind::JsxFragment => {
                JsxElaborationElement {
                    error_node: Some(Arc::clone(child)),
                    inner_expression: Some(Arc::clone(child)),
                    name_type: Some(Arc::clone(name_type)),
                    create_diagnostic: None,
                }
            }
            _ => panic!("Unhandled case in getElaborationElementForJsxChild"),
        }
    }

    pub fn elaborate_iterable_or_array_like_target_elementwise(
        &mut self,
        iterator: impl Iterator<Item = JsxElaborationElement>,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        mut diagnostic_output: Option<&mut Vec<Diagnostic>>,
    ) -> bool {
        let tuple_or_array_like_target_parts =
            filter_type_self(self, target, |checker, t| checker.is_array_or_tuple_like_type(t));
        let non_tuple_or_array_like_target_parts = filter_type_self(self, target, |checker, t| {
            !checker.is_array_or_tuple_like_type(t)
        });
        let mut iteration_type: Option<Arc<Type>> = None;
        if !Arc::ptr_eq(&non_tuple_or_array_like_target_parts, &self.never_type()) {
            iteration_type = self.get_iteration_type_of_iterable(
                IterationUse::ForOf { for_await: false },
                IterationTypeKind::YIELD,
                &non_tuple_or_array_like_target_parts,
                None,
            );
        }
        let mut reported_error = false;
        for e in iterator {
            let prop = match &e.error_node {
                Some(p) => Arc::clone(p),
                None => continue,
            };
            let next = e.inner_expression.clone();
            let name_type = match &e.name_type {
                Some(t) => Arc::clone(t),
                None => continue,
            };
            let mut target_prop_type = iteration_type.clone();
            let mut target_indexed_prop_type: Option<Arc<Type>> = None;
            if !Arc::ptr_eq(&tuple_or_array_like_target_parts, &self.never_type()) {
                target_indexed_prop_type = self.get_best_match_indexed_access_type_or_undefined(
                    source,
                    &tuple_or_array_like_target_parts,
                    &name_type,
                );
            }
            if let Some(tip) = &target_indexed_prop_type {
                if !tip.flags.contains(TypeFlags::IndexedAccess) {
                    if let Some(it) = &iteration_type {
                        target_prop_type =
                            Some(self.get_union_type(vec![Arc::clone(it), Arc::clone(tip)]));
                    } else {
                        target_prop_type = Some(Arc::clone(tip));
                    }
                }
            }
            let target_prop_type = match target_prop_type {
                Some(t) => t,
                None => continue,
            };
            let source_prop_type = self.get_indexed_access_type_or_undefined(
                source,
                &name_type,
                AccessFlags::None,
                None,
                None,
            );
            let source_prop_type = match source_prop_type {
                Some(t) => t,
                None => continue,
            };
            let prop_name = self.property_name_from_index(&name_type).unwrap_or_default();
            if !self.check_type_related_to(&source_prop_type, &target_prop_type, relation, None) {
                let elaborated = match &next {
                    Some(next) => self.elaborate_error(
                        next,
                        &source_prop_type,
                        &target_prop_type,
                        relation,
                        diagnostic_output.as_deref_mut(),
                    ),
                    None => false,
                };
                reported_error = true;
                if !elaborated {
                    let specific_source = match &next {
                        Some(next) => {
                            self.check_expression_for_mutable_location(next, CheckMode::Normal)
                        }
                        None => Arc::clone(&source_prop_type),
                    };
                    if let Some(create_diagnostic) = &e.create_diagnostic {
                        let diag = create_diagnostic(&prop);
                        self.report_diagnostic(&diag, diagnostic_output.as_deref_mut());
                    } else if self.exact_optional_property_types
                        && self.is_exact_optional_property_mismatch(Some(&specific_source), Some(&target_prop_type))
                    {
                        let diag = new_diagnostic_for_node(
                            Some(&prop),
                            TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1_WITH_EXACT_OPTIONAL_PROPERTY_TYPES_COLON_TRUE_CONSIDER_ADDING_UNDEFINED_TO_THE_TYPE_OF_THE_TARGET,
                            vec![
                                self.type_to_string(&specific_source),
                                self.type_to_string(&target_prop_type),
                            ],
                        );
                        self.report_diagnostic(&diag, diagnostic_output.as_deref_mut());
                    } else {
                        let mut target_is_optional = false;
                        let mut source_is_optional = false;
                        if prop_name != ast::INTERNAL_SYMBOL_NAME_MISSING {
                            target_is_optional = self
                                .get_property_of_type(&tuple_or_array_like_target_parts, &prop_name)
                                .is_some_and(|s| {
                                    s.flags.intersects(ast::SymbolFlags::Optional)
                                        || self.unknown_symbol.as_ref().is_some_and(|u| Arc::ptr_eq(u, &s))
                                });
                            source_is_optional = self
                                .get_property_of_type(source, &prop_name)
                                .is_some_and(|s| {
                                    s.flags.intersects(ast::SymbolFlags::Optional)
                                        || self.unknown_symbol.as_ref().is_some_and(|u| Arc::ptr_eq(u, &s))
                                });
                        }
                        let target_prop_type =
                            self.remove_missing_type(Arc::clone(&target_prop_type), target_is_optional);
                        let source_prop_type = self.remove_missing_type(
                            Arc::clone(&source_prop_type),
                            target_is_optional && source_is_optional,
                        );
                        let result = self.check_type_related_to(
                            &specific_source,
                            &target_prop_type,
                            relation,
                            Some(&prop),
                        );
                        if result && !Arc::ptr_eq(&specific_source, &source_prop_type) {
                            self.check_type_related_to(
                                &source_prop_type,
                                &target_prop_type,
                                relation,
                                Some(&prop),
                            );
                        }
                    }
                }
            }
        }
        reported_error
    }

    pub fn elaborate_element(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        prop: &Arc<Node>,
        next: Option<&Arc<Node>>,
        name_type: &Arc<Type>,
        error_message: Option<&'static str>,
        diagnostic_factory: Option<&dyn Fn(&Arc<Node>) -> Arc<Diagnostic>>,
        mut diagnostic_output: Option<&mut Vec<Diagnostic>>,
    ) -> bool {
        let target_prop_type =
            self.get_best_match_indexed_access_type_or_undefined(source, target, name_type);
        let target_prop_type = match target_prop_type {
            Some(t) if !t.flags.contains(TypeFlags::IndexedAccess) => t,
            _ => return false,
        };
        let source_prop_type = self.get_indexed_access_type_or_undefined(
            source,
            name_type,
            AccessFlags::None,
            None,
            None,
        );
        let source_prop_type = match source_prop_type {
            Some(t) if !self.check_type_related_to(&t, &target_prop_type, relation, None) => t,
            _ => return false,
        };
        if let Some(next) = next {
            if self.elaborate_error(next, &source_prop_type, &target_prop_type, relation, diagnostic_output.as_deref_mut()) {
                return true;
            }
        }
        let mut diags: Vec<Arc<Diagnostic>> = Vec::new();
        let mut specific_source = Arc::clone(&source_prop_type);
        if let Some(next) = next {
            specific_source =
                self.check_expression_for_mutable_location(next, CheckMode::Normal);
        }
        if let Some(diagnostic_factory) = diagnostic_factory {
            diags.push(diagnostic_factory(prop));
        } else if self.exact_optional_property_types
            && self.is_exact_optional_property_mismatch(Some(&specific_source), Some(&target_prop_type))
        {
            diags.push(Arc::new(new_diagnostic_for_node(
                Some(prop),
                TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1_WITH_EXACT_OPTIONAL_PROPERTY_TYPES_COLON_TRUE_CONSIDER_ADDING_UNDEFINED_TO_THE_TYPE_OF_THE_TARGET,
                vec![
                    self.type_to_string(&specific_source),
                    self.type_to_string(&target_prop_type),
                ],
            )));
        } else {
            let prop_name = self.property_name_from_index(name_type).unwrap_or_default();
            let target_is_optional = self
                .get_property_of_type(target, &prop_name)
                .is_some_and(|s| s.flags.intersects(ast::SymbolFlags::Optional));
            let source_is_optional = self
                .get_property_of_type(source, &prop_name)
                .is_some_and(|s| s.flags.intersects(ast::SymbolFlags::Optional));
            let target_prop_type =
                self.remove_missing_type(Arc::clone(&target_prop_type), target_is_optional);
            let source_prop_type = self.remove_missing_type(
                Arc::clone(&source_prop_type),
                target_is_optional && source_is_optional,
            );
            let result = self.check_type_related_to(
                &specific_source,
                &target_prop_type,
                relation,
                Some(prop),
            );
            if result && !Arc::ptr_eq(&specific_source, &source_prop_type) {
                self.check_type_related_to(
                    &source_prop_type,
                    &target_prop_type,
                    relation,
                    Some(prop),
                );
            }
        }
        if diags.is_empty() {
            return false;
        }
        let mut diagnostic = diags.into_iter().next().unwrap();
        let mut property_name = String::new();
        let mut target_prop: Option<Arc<Symbol>> = None;
        if is_type_usable_as_property_name(name_type) {
            property_name = get_property_name_from_type(name_type);
            target_prop = self.get_property_of_type(target, &property_name);
        }
        let mut issued_elaboration = false;
        if target_prop.is_none() {
            if let Some(index_info) = self.get_applicable_index_info(target, name_type) {
                if let Some(declaration) = &index_info.declaration {
                    if let Some(source_file) = self.get_source_file_of_node(declaration) {
                        if !self.program.is_source_file_default_library(&source_file.file_name) {
                            issued_elaboration = true;
                            Arc::get_mut(&mut diagnostic).unwrap().add_related_info(new_diagnostic_for_node(
                                Some(declaration),
                                THE_EXPECTED_TYPE_COMES_FROM_THIS_INDEX_SIGNATURE,
                                vec![],
                            ));
                        }
                    }
                }
            }
        }
        let target_has_declarations = target_prop
            .as_ref()
            .map(|p| !p.declarations.is_empty())
            .unwrap_or(false);
        let target_symbol_has_declarations = target
            .symbol
            .as_ref()
            .map(|s| !s.declarations.is_empty())
            .unwrap_or(false);
        if !issued_elaboration && (target_has_declarations || target_symbol_has_declarations) {
            let target_node = if target_has_declarations {
                Arc::clone(&target_prop.as_ref().unwrap().declarations[0])
            } else {
                Arc::clone(&target.symbol.as_ref().unwrap().declarations[0])
            };
            if property_name.is_empty() || name_type.flags.contains(TypeFlags::UniqueESSymbol) {
                property_name = self.type_to_string(name_type);
            }
            if let Some(source_file) = self.get_source_file_of_node(&target_node) {
                if !self.program.is_source_file_default_library(&source_file.file_name) {
                Arc::get_mut(&mut diagnostic).unwrap().add_related_info(new_diagnostic_for_node(
                    Some(&target_node),
                    THE_EXPECTED_TYPE_COMES_FROM_PROPERTY_0_WHICH_IS_DECLARED_HERE_ON_TYPE_1,
                    vec![property_name, self.type_to_string(target)],
                ));
                }
            }
        }
        self.report_diagnostic(&diagnostic, diagnostic_output.as_deref_mut());
        true
    }
}

#[derive(Default)]
pub struct JsxElaborationElement {
    pub error_node: Option<Arc<Node>>,
    pub inner_expression: Option<Arc<Node>>,
    pub name_type: Option<Arc<Type>>,
    pub create_diagnostic: Option<Box<dyn Fn(&Arc<Node>) -> Arc<Diagnostic>>>,
}
