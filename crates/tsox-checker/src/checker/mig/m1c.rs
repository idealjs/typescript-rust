#![allow(unused_imports)]

#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
#[allow(unused_imports)]
use tsox_core::diagnostics::messages_generated::*;

pub(crate) use crate::checker::checker::*;
#[allow(unused_imports)]
use tsox_frontend::scanner;
#[allow(unused_imports)]
use tsox_frontend::ast::mig::m3g_2::is_type_declaration;
#[allow(unused_imports)]
use tsox_frontend::ast::mig::m3b::is_type_or_js_type_alias_declaration;
#[allow(unused_imports)]
use std::collections::HashMap;
use std::sync::Arc;

#[path = "r24k14_defs.rs"]
pub mod r24k14_defs;

#[path = "r25k9_defs.rs"]
pub mod r25k9_defs;

#[path = "r26k5_defs.rs"]
pub mod r26k5_defs;

use crate::checker::types::*;
use tsox_frontend::ast::{Diagnostic, Node, NodeData, SyntaxKind};

pub fn create_file_index_map(files: &[Arc<SourceFile>]) -> HashMap<u64, usize> { ::tsox_core::fntrace::enter("create_file_index_map"); 
    let mut result = HashMap::with_capacity(files.len());
    for (i, file) in files.iter().enumerate() {
        result.insert(file.id(), i);
    }
    result
}


impl Checker {
    fn count_global_symbols(&self, files: &[Arc<SourceFile>]) -> usize { ::tsox_core::fntrace::enter("count_global_symbols"); 
        let mut count = 0;
        for file in files {
            if !is_external_or_common_js_module(file) {
                count += self
                    .program
                    .symbol_map()
                    .locals_of(&file.node)
                    .map(|l| l.len())
                    .unwrap_or(0);
            }
        }
        count
    }

    pub fn check_type_parameter(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_type_parameter"); 
        self.check_grammar_modifiers(node);
        if let NodeData::TypeParameterDeclaration(tp) = &node.data {
            if let Some(expr) = &tp.expression {
                self.grammar_error_on_first_token(expr, &TYPE_EXPECTED);
            }
            if let Some(constraint) = &tp.constraint {
                self.check_source_element(constraint);
            }
            if let Some(default_type) = &tp.default_type {
                self.check_source_element(default_type);
            }
        }
        let symbol = self.get_symbol_of_declaration(node).unwrap();
        let type_parameter = self.get_declared_type_of_type_parameter(&symbol);
        self.get_base_constraint_of_type(&type_parameter);
        let resolved_default = self.get_resolved_type_parameter_default(&type_parameter);
        if self
            .circular_constraint_type
            .get()
            .is_some_and(|c| Arc::ptr_eq(c, &resolved_default))
        {
            if let NodeData::TypeParameterDeclaration(tp) = &node.data {
                if let Some(default_type) = &tp.default_type {
                    let name = self.type_to_string(&type_parameter);
                    self.error_message(default_type,TYPE_PARAMETER_0_HAS_A_CIRCULAR_DEFAULT, &[name]);
                }
            }
        }
        let constraint_type = self.get_constraint_of_type_parameter(&type_parameter);
        let default_type = self.get_default_from_type_parameter(&type_parameter);
        if let (Some(constraint_type), Some(default_type)) = (constraint_type, default_type) {
            let mapper = Arc::new(new_simple_type_mapper(type_parameter.clone(), default_type.clone()));
            let instantiated = self.instantiate_type(&constraint_type, Some(&mapper));
            let with_this = self.get_type_with_this_argument(&instantiated, Some(&default_type), false);
            if let NodeData::TypeParameterDeclaration(tp) = &node.data {
                if let Some(default_node) = &tp.default_type {
                    self.check_type_assignable_to(
                        &default_type,
                        &with_this,
                        Some(default_node),
                        Some(&TYPE_0_DOES_NOT_SATISFY_THE_CONSTRAINT_1),
                    );
                }
            }
        }
        if let NodeData::TypeParameterDeclaration(tp) = &node.data {
            self.check_type_name_is_reserved(&tp.name, TYPE_PARAMETER_NAME_CANNOT_BE_0);
        }
        self.check_node_deferred(node);
    }

    fn marker_types_for_variance(&self) -> (Arc<Type>, Arc<Type>) { ::tsox_core::fntrace::enter("marker_types_for_variance"); 
        (
            self.marker_sub_type_for_check.clone().unwrap(),
            self.marker_super_type_for_check.clone().unwrap(),
        )
    }

    pub fn check_type_parameter_deferred(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_type_parameter_deferred"); 
        let parent = match node.parent() {
            Some(p) => p,
            None => return,
        };
        if is_interface_declaration(&parent) || is_class_like(&parent) || is_type_or_js_type_alias_declaration(&parent) {
            let symbol = self.get_symbol_of_declaration(&parent).unwrap();
            let type_parameter = {
                let decl_symbol = self.get_symbol_of_declaration(node).unwrap();
                self.get_declared_type_of_type_parameter(&decl_symbol)
            };
            let modifiers = self.get_type_parameter_modifiers(&type_parameter)
                & (ModifierFlags::In | ModifierFlags::Out);
            if modifiers != ModifierFlags::empty() {
                let declared = self.get_declared_type_of_symbol(&symbol);
                if is_type_or_js_type_alias_declaration(&parent)
                    && declared.object_flags
                        .intersection(ObjectFlags::Anonymous | ObjectFlags::Mapped)
                        .is_empty()
                {
                    let source_file = self.get_source_file_of_node(node);
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        source_file,
                        node.loc,
                        VARIANCE_ANNOTATIONS_ARE_ONLY_SUPPORTED_IN_TYPE_ALIASES_FOR_OBJECT_FUNCTION_CONSTRUCTOR_AND_MAPPED_TYPES,
                        vec![],
                    ));
                } else if modifiers == ModifierFlags::In || modifiers == ModifierFlags::Out {
                    let (marker_sub, marker_super) = self.marker_types_for_variance();
                    let source = self.create_marker_type(
                        &symbol,
                        &type_parameter,
                        if modifiers == ModifierFlags::Out { &marker_sub } else { &marker_super },
                    ).unwrap();
                    let target = self.create_marker_type(
                        &symbol,
                        &type_parameter,
                        if modifiers == ModifierFlags::Out { &marker_super } else { &marker_sub },
                    ).unwrap();
                    let saved = self.variance_type_parameter.clone();
                    self.variance_type_parameter = Some(type_parameter.clone());
                    self.check_type_assignable_to(
                        &source,
                        &target,
                        Some(node),
                        Some(&TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1_AS_IMPLIED_BY_VARIANCE_ANNOTATION),
                    );
                    self.variance_type_parameter = saved;
                }
            }
        }
    }

    pub fn check_type_reference_node(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_type_reference_node"); 
        if let NodeData::TypeReferenceNode(data) = &node.data {
            if let Some(type_arguments) = &data.type_arguments {
                self.check_grammar_type_arguments(node, type_arguments);
            }
        }
        if is_type_reference_node(node) && !node.flags.contains(NodeFlags::JSDoc) {
            if let NodeData::TypeReferenceNode(data) = &node.data {
                if let Some(type_arguments) = &data.type_arguments {
                    if data.type_name.end() != type_arguments.pos() {
                        if let Some(source_file) = self.get_source_file_of_node(node) {
                            if tsox_frontend::scanner::mig::m4d_2::scan_token_at_position(
                                &source_file,
                                data.type_name.end(),
                            ) == SyntaxKind::DotToken
                            {
                                let start = scanner::skip_trivia(&source_file.text, data.type_name.end());
                                self.grammar_error_at_pos(
                                    node,
                                    start,
                                    1,
                                    &JSDOC_TYPES_CAN_ONLY_BE_USED_INSIDE_DOCUMENTATION_COMMENTS,
                                );
                            }
                        }
                    }
                }
            }
        }
        if let NodeData::TypeReferenceNode(data) = &node.data {
            if let Some(type_arguments) = &data.type_arguments {
                self.check_source_elements(&type_arguments.nodes);
            }
        }
        if !(is_const_type_reference(node)
            && node.parent().as_deref().map(is_assertion_expression).unwrap_or(false))
        {
            self.check_type_reference_or_import(node);
        }
    }

    pub fn check_type_reference_or_import(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_type_reference_or_import"); 
        let t = self.get_type_from_type_node(node);
        if !self.is_error_type(&t) {
            if !tsox_frontend::ast::mig::m3c::type_arguments(node).is_empty() {
                let type_parameters = self.get_type_parameters_for_type_reference_or_import(node);
                if !type_parameters.is_empty() {
                    self.check_type_argument_constraints(node, &type_parameters);
                }
            }
            let symbol = self.get_resolved_symbol_or_nil(node);
            if let Some(symbol) = symbol {
                if symbol
                    .declarations
                    .iter()
                    .any(|d| is_type_declaration(d) && self.is_deprecated_declaration(d))
                {
                    if let Some(suggestion_node) = self.get_deprecated_suggestion_node(node) {
                        self.add_deprecated_suggestion(&suggestion_node, &symbol.declarations, &symbol.name);
                    }
                }
            }
        }
    }

    pub fn check_type_query(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_type_query"); 
        self.get_type_from_type_query_node(node);
    }

    pub fn check_type_literal(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_type_literal"); 
        if let NodeData::TypeLiteralNode(data) = &node.data {
            self.check_source_elements(&data.members.nodes);
        }
        let t = self.get_type_from_type_literal_or_function_or_constructor_type_node(node);
        self.check_index_constraints(&t, node);
        self.check_type_for_duplicate_index_signatures(node);
        self.check_object_type_for_duplicate_declarations(node, false);
    }

    pub fn check_tuple_type(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_tuple_type"); 
        let mut seen_optional_element = false;
        let mut seen_rest_element = false;
        let elements = tsox_frontend::ast::mig::m3b::elements(node);
        for e in elements.iter() {
            let mut flags = self.get_tuple_element_flags(e);
            if flags.contains(ElementFlags::Variadic) {
                let element_type_node = node_type(e).unwrap();
                let t = self.get_type_from_type_node(element_type_node);
                if !self.is_array_like_type(&t) {
                    self.error_message(e,A_REST_ELEMENT_TYPE_MUST_BE_AN_ARRAY_TYPE, &[]);
                    break;
                }
                if self.is_array_type(&t)
                    || t.target_tuple_type().map(|d| d.combined_flags.contains(ElementFlags::Rest)).unwrap_or(false)
                {
                    flags |= ElementFlags::Rest;
                }
            }
            if flags.contains(ElementFlags::Rest) {
                if seen_rest_element {
                    self.grammar_error_on_node(
                        e,
                        &A_REST_ELEMENT_CANNOT_FOLLOW_ANOTHER_REST_ELEMENT,
                    );
                    break;
                }
                seen_rest_element = true;
            } else if flags.contains(ElementFlags::Optional) {
                if seen_rest_element {
                    self.grammar_error_on_node(
                        e,
                        &AN_OPTIONAL_ELEMENT_CANNOT_FOLLOW_A_REST_ELEMENT,
                    );
                    break;
                }
                seen_optional_element = true;
            } else if flags.contains(ElementFlags::Required) && seen_optional_element {
                self.grammar_error_on_node(
                    e,
                    &A_REQUIRED_ELEMENT_CANNOT_FOLLOW_AN_OPTIONAL_ELEMENT,
                );
                break;
            }
        }
        self.check_source_elements(elements);
        self.get_type_from_type_node(node);
    }

    pub fn check_union_or_intersection_type(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_union_or_intersection_type"); 
        for_each_child(node, |c| {
            self.check_source_element(c);
            true
        });
        self.get_type_from_type_node(node);
    }

    pub fn check_type_operator(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_type_operator"); 
        self.check_grammar_type_operator_node(node);
        if let Some(operand) = node_type(node) {
            self.check_source_element(operand);
        }
    }

    pub fn check_while_statement(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_while_statement"); 
        self.check_grammar_statement_in_ambient_context(node);
        let expression = node_expression(node).unwrap();
        self.check_truthiness_expression(expression, CheckMode::Normal);
        let statement = match &node.data {
            NodeData::WhileStatement(d) => Arc::clone(&d.statement),
            _ => return,
        };
        self.check_source_element(&statement);
    }

    pub fn check_with_statement(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_with_statement"); 
        if !self.check_grammar_statement_in_ambient_context(node) {
            if node.flags.contains(NodeFlags::AwaitContext) {
                self.grammar_error_on_first_token(
                    node,
                    &X_WITH_STATEMENTS_ARE_NOT_ALLOWED_IN_AN_ASYNC_FUNCTION_BLOCK,
                );
            }
        }
        let expression = node_expression(node).unwrap();
        self.check_expression(expression);
        let source_file = get_source_file_of_node(node).unwrap();
        if !self.has_parse_diagnostics(&source_file) {
            let start = scanner::skip_trivia(source_file.text(), node.pos());
            let end = match &node.data {
                NodeData::WithStatement(d) => d.statement.pos(),
                _ => return,
            };
            self.grammar_error_at_pos(
                node,
                start,
                end - start,
                &THE_WITH_STATEMENT_IS_NOT_SUPPORTED_ALL_SYMBOLS_IN_A_WITH_BLOCK_WILL_HAVE_TYPE_ANY,
            );
        }
    }

}

