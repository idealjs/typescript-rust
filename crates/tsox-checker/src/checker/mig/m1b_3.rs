#![allow(unused_imports)]

use crate::checker::checker::*;
use std::sync::Arc;
use tsox_frontend::ast::NodeData;
use tsox_frontend::ast::NodeFlags;
use tsox_frontend::ast::SyntaxKind;
use tsox_frontend::ast::SymbolFlags;
use tsox_frontend::ast::{is_identifier, is_in_js_file, is_private_identifier, skip_parentheses};
use tsox_core::core::tristate::Tristate;
use crate::checker::mig::m1c_3::create_diagnostic_for_node_message;
use tsox_frontend::scanner::mig::m3i::declaration_name_to_string;
use crate::checker::utilities_get_assignment_target::entity_name_to_string;
use tsox_frontend::ast::mig::m3e::get_function_flags;
use crate::checker::mig::m2e::get_identifier_from_entity_name_expression;
use crate::checker::mig::wc2::r22k3_defs::interface_this_type;
use crate::checker::mig::wc3::r18k4_node_ext::NodeAccessExt;
use tsox_frontend::ast::Diagnostic;
use crate::checker::mig::m3a_2::has_dot_dot_dot_token;
use crate::checker::utilities_has_only_expression_initialization::is_optional_declaration;
use crate::checker::utilities_is_optional_symbol::is_type_any;
use tsox_frontend::scanner::token_to_string;
use tsox_core::diagnostics::messages_generated::*;
use tsox_frontend::ast::mig::m3e::FunctionFlags;
use tsox_frontend::ast::mig::m3e_4::get_new_target_container;
use tsox_frontend::ast::mig::x4ast::get_containing_function;
use super::m1b::{
    is_default_clause,
    is_js_doc_non_nullable_type,
    LANGUAGE_FEATURE_MINIMUM_TARGET_ASYNC_GENERATORS,
    LANGUAGE_FEATURE_MINIMUM_TARGET_ASYNC_FUNCTIONS,
};
use tsox_core::diagnostics::messages_generated::{
    X_0_AT_THE_END_OF_A_TYPE_IS_NOT_VALID_TYPESCRIPT_SYNTAX_DID_YOU_MEAN_TO_WRITE_1
        as X_0_AT_THE_END_OF_A_TYPE_IS_NOT_VALID_TYPE_SCRIPT_SYNTAX_DID_YOU_MEAN_TO_WRITE_1,
    X_0_AT_THE_START_OF_A_TYPE_IS_NOT_VALID_TYPESCRIPT_SYNTAX_DID_YOU_MEAN_TO_WRITE_1
        as X_0_AT_THE_START_OF_A_TYPE_IS_NOT_VALID_TYPE_SCRIPT_SYNTAX_DID_YOU_MEAN_TO_WRITE_1,
};

impl Checker {
    pub fn check_instance_of_expression(&mut self, left: &Arc<Node>, right: &Arc<Node>, left_type: &Arc<Type>, right_type: &Arc<Type>, check_mode: CheckMode) -> Arc<Type> {
        if Arc::ptr_eq(left_type, &self.silent_never_type()) || Arc::ptr_eq(right_type, &self.silent_never_type()) {
            return self.silent_never_type();
        }
        if !is_type_any(left_type) && self.all_types_assignable_to_kind(left_type, TypeFlags::PRIMITIVE) {
            self.error_message(left,THE_LEFT_HAND_SIDE_OF_AN_INSTANCEOF_EXPRESSION_MUST_BE_OF_TYPE_ANY_AN_OBJECT_TYPE_OR_A_TYPE_PARAMETER, &[]);
        }
        let parent = left.parent().unwrap_or_else(|| Arc::clone(left));
        let signature = self.get_resolved_signature(&parent);
        if signature.as_ref().map(|s| Arc::ptr_eq(s, &self.resolving_signature())).unwrap_or(false) {
            return self.silent_never_type();
        }
        let return_type = match &signature {
            Some(sig) => self.get_return_type_of_signature(sig),
            None => return self.unknown_type(),
        };
        if let Some(return_type) = &return_type {
            self.check_type_assignable_to(
                return_type,
                &self.boolean_type(),
                Some(right),
                Some(&AN_OBJECT_S_SYMBOL_HASINSTANCE_METHOD_MUST_RETURN_A_BOOLEAN_VALUE_FOR_IT_TO_BE_USED_ON_THE_RIGHT_HAND_SIDE_OF_AN_INSTANCEOF_EXPRESSION),
            );
        }
        self.boolean_type()
    }

    pub fn check_interface_declaration(&mut self, node: &Arc<Node>) {
        if !self.check_grammar_modifiers(node) {
            self.check_grammar_interface_declaration(node);
        }
        let parent = node.parent();
        if !parent.as_ref().map(|p| self.container_allows_block_scoped_variable(p)).unwrap_or(true) {
            self.grammar_error_on_node_with_args(node, &X_0_DECLARATIONS_CAN_ONLY_BE_DECLARED_INSIDE_A_BLOCK, &["interface".to_string()]);
        }
        self.check_type_parameters(tsox_frontend::ast::mig::m3c::type_parameter_list(node).map(|l| &**l));
        let name = node.name().cloned().unwrap_or_else(|| Arc::clone(node));
        self.check_type_name_is_reserved(&name, INTERFACE_NAME_CANNOT_BE_0);
        self.check_exports_on_merged_declarations(node);
        let symbol = self.get_symbol_of_declaration(node);
        if let Some(symbol) = &symbol {
            self.check_type_parameter_lists_identical(symbol);
            if !self.declared_type_links.get_or_default(symbol).interface_checked {
                self.declared_type_links.get_or_default(symbol).interface_checked = true;
                let t = self.get_declared_type_of_symbol(symbol);
                let type_with_this = self.get_type_with_this_argument(&t, None, false);
                if self.check_inherited_properties_are_identical(&t, &name) {
                    let this_type = interface_this_type(&t);
                    for base_type in self.get_base_types(&t) {
                        let base_with_this = self.get_type_with_this_argument(&base_type, this_type.as_ref(), false);
                        self.check_type_assignable_to(&type_with_this, &base_with_this, Some(&name), Some(&INTERFACE_0_INCORRECTLY_EXTENDS_INTERFACE_1));
                    }
                    self.check_index_constraints(&t, node);
                }
            }
        }
        self.check_object_type_for_duplicate_declarations(node, false);
        for heritage_element in tsox_frontend::ast::get_extends_heritage_clause_elements(node).iter() {
            if tsox_frontend::ast::is_expression_with_type_arguments(heritage_element) {
                if let Some(expr) = heritage_element.expression() {
                    if !tsox_frontend::ast::is_entity_name_expression(expr) || tsox_frontend::ast::is_optional_chain(expr) {
                        self.error_message(expr,AN_INTERFACE_CAN_ONLY_EXTEND_AN_IDENTIFIER_SLASHQUALIFIED_NAME_WITH_OPTIONAL_TYPE_ARGUMENTS, &[]);
                    }
                }
            }
            self.check_type_reference_node(heritage_element);
        }
        let members = tsox_frontend::ast::mig::m3b::members(node);
        self.check_source_elements_from_list(members);
        self.check_class_or_interface_for_duplicate_index_signatures(node);
        self.register_for_unused_identifiers_check(node);
    }

    pub fn check_jsdoc_augments_tag_matches_extends(&mut self, node: &Arc<Node>, base_type_node: &Arc<Node>, base_type: &Arc<Type>) {
        if !is_in_js_file(node) {
            return;
        }
        let file = self.get_source_file_of_node(node);
        let jsdocs = file.as_ref().map(|f| f.eager_jsdoc(node)).unwrap_or_default();
        for j in jsdocs.iter() {
            let tags = match &j.data {
                NodeData::JSDoc(data) => data.tags.clone(),
                _ => None,
            };
            let Some(tags) = tags else {
                continue;
            };
            for tag in tags.nodes.iter() {
                if tag.kind != SyntaxKind::JSDocAugmentsTag {
                    continue;
                }
                let NodeData::JSDocAugmentsTag(data) = &tag.data else {
                    continue;
                };
                let source_type_node = Arc::clone(&data.class_name);
                let source_type = self.get_type_from_type_node(&source_type_node);
                if self.is_type_identical_to(&source_type, base_type) {
                    continue;
                }
                let Some(base_expr) = base_type_node.expression() else {
                    continue;
                };
                let target_name = get_identifier_from_entity_name_expression(base_expr);
                let Some(source_expr) = source_type_node.expression() else {
                    continue;
                };
                let source_name = get_identifier_from_entity_name_expression(source_expr);
                if let (Some(target_name), Some(source_name)) = (target_name, source_name) {
                    let tag_name = tsox_frontend::ast::mig::m3c::tag_name(tag).text().to_string();
                    let source_text = source_name.text().to_string();
                    let target_text = target_name.text().to_string();
                    self.error_message(&source_name,JSDOC_0_1_DOES_NOT_MATCH_THE_EXTENDS_2_CLAUSE, &[tag_name, source_text, target_text]);
                }
            }
        }
    }

    pub fn check_jsdoc_comments(&mut self, node: &Arc<Node>) {
        let comments = match &node.data {
            NodeData::JSDoc(data) => Some(Arc::clone(&data.comment)),
            _ => None,
        };
        if let Some(comments) = comments {
            for comment in comments.nodes.iter() {
                self.check_jsdoc_comment(comment);
            }
        }
    }

    pub fn check_jsdoc_comment(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::JSDocLink | SyntaxKind::JSDocLinkCode | SyntaxKind::JSDocLinkPlain => {
                let name = node.name();
                self.resolve_jsdoc_member_name(name);
            }
            _ => {}
        }
    }

    pub fn check_jsdoc_type(&mut self, node: &Arc<Node>) {
        self.check_jsdoc_type_is_in_js_file(node);
        node.for_each_child(|child| {
            self.check_source_element(child);
            true
        });
    }

    pub fn check_jsdoc_type_is_in_js_file(&mut self, node: &Arc<Node>) {
        if !is_in_js_file(node) {
            if tsox_frontend::ast::is_jsdoc_non_nullable_type(node) || tsox_frontend::ast::is_jsdoc_nullable_type(node) {
                let non_nullable = is_js_doc_non_nullable_type(node);
                let token = if non_nullable { "!" } else { "?" };
                let type_node = node.type_node().cloned().unwrap_or_else(|| Arc::clone(node));
                let postfix = node.loc.pos == type_node.loc.pos;
                let message = if postfix {
                    X_0_AT_THE_END_OF_A_TYPE_IS_NOT_VALID_TYPE_SCRIPT_SYNTAX_DID_YOU_MEAN_TO_WRITE_1
                } else {
                    X_0_AT_THE_START_OF_A_TYPE_IS_NOT_VALID_TYPE_SCRIPT_SYNTAX_DID_YOU_MEAN_TO_WRITE_1
                };
                let mut t = self.get_type_from_type_node(&type_node);
                if tsox_frontend::ast::is_jsdoc_nullable_type(node) && !Arc::ptr_eq(&t, &self.never_type()) && !Arc::ptr_eq(&t, &self.void_type()) {
                    let flags = if postfix { TypeFlags::Undefined } else { TypeFlags::NULLABLE };
                    t = self.get_nullable_type(&t, flags);
                }
                let type_string = self.type_to_string(&t);
                self.grammar_error_on_node_with_args(node, &message, &[token.to_string(), type_string]);
            } else {
                self.grammar_error_on_node(node, &JSDOC_TYPES_CAN_ONLY_BE_USED_INSIDE_DOCUMENTATION_COMMENTS);
            }
        }
    }

    pub fn check_labeled_statement(&mut self, node: &Arc<Node>) {
        let (label_node, statement) = match &node.data {
            NodeData::LabeledStatement(data) => (Arc::clone(&data.label), Arc::clone(&data.statement)),
            _ => return,
        };
        let label_text = label_node.text();
        if !self.check_grammar_statement_in_ambient_context(node) {
            let mut current = node.parent();
            while let Some(current_node) = &current {
                if tsox_frontend::ast::is_function_like(current_node) {
                    break;
                }
                if tsox_frontend::ast::is_labeled_statement(current_node)
                    && tsox_frontend::ast::mig::m3b::label(current_node).map(|l| l.text() == label_text).unwrap_or(false)
                {
                    let text = label_text.clone();
                    self.grammar_error_on_node_with_args(&label_node, &DUPLICATE_LABEL_0, &[text.to_string()]);
                    break;
                }
                current = current_node.parent();
            }
        }
        if label_node.flags.contains(NodeFlags::Unreachable) && self.compiler_options.allow_unused_labels != Tristate::True {
            let is_suggestion = self.compiler_options.allow_unused_labels == Tristate::False;
            self.error_or_suggestion_message(is_suggestion, &label_node,UNUSED_LABEL, &[]);
        }
        self.check_source_element(&statement);
    }

    pub fn check_mapped_type(&mut self, node: &Arc<Node>) {
        self.check_grammar_mapped_type(node);
        let (type_parameter, name_type, type_node) = match &node.data {
            NodeData::MappedTypeNode(data) => (
                Arc::clone(&data.type_parameter),
                data.name_type.clone(),
                data.type_node.clone(),
            ),
            _ => return,
        };
        self.check_source_element(&type_parameter);
        if let Some(name_type) = &name_type {
            self.check_source_element(name_type);
        }
        if let Some(type_node) = &type_node {
            self.check_source_element(type_node);
        }
        if type_node.is_none() {
            let any_type = self.any_type();
            self.report_implicit_any(node, &any_type, WideningKind::Normal);
        }
        let t = self.get_type_from_mapped_type_node(node);
        let name_type_of_t = self.get_name_type_from_mapped_type(&t);
        if let Some(name_type_of_t) = name_type_of_t {
            let target = name_type.clone().unwrap_or_else(|| Arc::clone(node));
            let string_number_symbol = Arc::clone(&self.string_number_symbol_type);
            self.check_type_assignable_to(&name_type_of_t, &string_number_symbol, Some(&target), None);
        } else {
            let constraint = match &type_parameter.data {
                NodeData::TypeParameterDeclaration(data) => data.constraint.clone(),
                _ => None,
            };
            let constraint_target = constraint.unwrap_or_else(|| Arc::clone(&type_parameter));
            if let Some(constraint_type) = self.get_constraint_type_from_mapped_type(&t) {
                let string_number_symbol = Arc::clone(&self.string_number_symbol_type);
                // Go relater.go:3726-3731 源侧 keyof 分支：Index 源的 relate 先按
                // stringNumberSymbolType 桥接比较，本处目标恰为 stringNumberSymbolType，
                // 判定恒真，keyof 约束由此不报 TS2322；非 Index 约束照常全检
                let keyof_bridge = constraint_type.flags.contains(TypeFlags::Index)
                    && self.is_type_assignable_to(&string_number_symbol, &string_number_symbol);
                if !keyof_bridge {
                    self.check_type_assignable_to(&constraint_type, &string_number_symbol, Some(&constraint_target), None);
                }
            }
        }
    }

    pub fn check_meta_property(&mut self, node: &Arc<Node>) -> Arc<Type> {
        self.check_grammar_meta_property(node);
        let keyword_token = match &node.data {
            NodeData::MetaProperty(data) => data.keyword_token,
            _ => node.kind,
        };
        match keyword_token {
            SyntaxKind::NewKeyword => self.check_new_target_meta_property(node),
            SyntaxKind::ImportKeyword => {
                if node.name().map(|n| n.text() == "defer").unwrap_or(false) {
                    return self.error_type();
                }
                self.check_import_meta_property(node)
            }
            _ => panic!("Unhandled case in checkMetaProperty"),
        }
    }

    pub fn check_meta_property_keyword(&mut self, _node: &Arc<Node>) -> Arc<Type> {
        self.error_type()
    }

    pub fn check_method_declaration(&mut self, node: &Arc<Node>) {
        if !self.check_grammar_method(node) {
            let name = node.name();
            if let Some(name) = node.name() {
                self.check_grammar_computed_property_name(name);
            }
            let has_asterisk_token = match &node.data {
                NodeData::MethodDeclaration(d) => d.asterisk_token.is_some(),
                _ => false,
            };
            let is_generator_named_constructor = tsox_frontend::ast::is_method_declaration(node)
                && has_asterisk_token
                && name.map(|n| is_identifier(n) && n.text() == "constructor").unwrap_or(false);
            if is_generator_named_constructor {
                let name = name.unwrap();
                self.error_message(&name,CLASS_CONSTRUCTOR_MAY_NOT_BE_A_GENERATOR, &[]);
            }
        }
        self.check_function_or_method_declaration(node);
        let has_abstract = tsox_frontend::ast::has_syntactic_modifier(node, ModifierFlags::Abstract);
        if has_abstract && tsox_frontend::ast::is_method_declaration(node) && function_body(node).is_some() {
            let name = node.name();
            let name_string = name.map(|n| declaration_name_to_string(Some(n))).unwrap_or_default();
            self.error_message(node,METHOD_0_CANNOT_HAVE_AN_IMPLEMENTATION_BECAUSE_IT_IS_MARKED_ABSTRACT, &[name_string]);
        }
        let name = node.name();
        if name.map(|n| tsox_frontend::ast::is_private_identifier(n)).unwrap_or(false)
            && tsox_frontend::ast::get_containing_class(node).is_none()
        {
            self.error_message(node,PRIVATE_IDENTIFIERS_ARE_NOT_ALLOWED_OUTSIDE_CLASS_BODIES, &[]);
        }
        self.set_node_links_for_private_identifier_scope(node);
    }

    pub fn check_missing_declaration(&mut self, node: &Arc<Node>) {
        self.check_decorators(node);
    }

    pub fn check_nan_equality(&mut self, error_node: &Arc<Node>, operator: SyntaxKind, left: &Arc<Node>, right: &Arc<Node>) {
        let is_left_nan = self.is_global_nan(&skip_parentheses(left));
        let is_right_nan = self.is_global_nan(&skip_parentheses(right));
        if !is_left_nan && !is_right_nan {
            return;
        }
        let always_returns = if operator == SyntaxKind::EqualsEqualsEqualsToken || operator == SyntaxKind::EqualsEqualsToken {
            token_to_string(SyntaxKind::FalseKeyword)
        } else {
            token_to_string(SyntaxKind::TrueKeyword)
        };
        self.error_message(error_node,THIS_CONDITION_WILL_ALWAYS_RETURN_0, &[always_returns.to_string()]);
        if is_left_nan && is_right_nan {
            return;
        }
        let mut operator_string = String::new();
        if operator == SyntaxKind::ExclamationEqualsEqualsToken || operator == SyntaxKind::ExclamationEqualsToken {
            operator_string = token_to_string(SyntaxKind::ExclamationToken).to_string();
        }
        let location = if is_left_nan { right } else { left };
        let expression = skip_parentheses(location);
        let entity_name = if tsox_frontend::ast::is_entity_name_expression(&expression) {
            entity_name_to_string(&expression)
        } else {
            "...".to_string()
        };
        let suggestion = format!("{}Number.isNaN({})", operator_string, entity_name);
        let related = create_diagnostic_for_node_message(location,DID_YOU_MEAN_0, &[suggestion]);
        self.add_related_info(error_node, (*related).clone());
    }

    pub fn check_named_tuple_member(&mut self, node: &Arc<Node>) {
        let (dot_dot_dot_token, question_token, type_node) = match &node.data {
            NodeData::NamedTupleMember(data) => (data.dot_dot_dot_token.clone(), data.question_token.clone(), Arc::clone(&data.type_node)),
            _ => return,
        };
        if dot_dot_dot_token.is_some() && question_token.is_some() {
            self.grammar_error_on_node(node, &A_TUPLE_MEMBER_CANNOT_BE_BOTH_OPTIONAL_AND_REST);
        }
        if type_node.kind == SyntaxKind::OptionalType {
            self.grammar_error_on_node(&type_node, &A_LABELED_TUPLE_ELEMENT_IS_DECLARED_AS_OPTIONAL_WITH_A_QUESTION_MARK_AFTER_THE_NAME_AND_BEFORE_THE_COLON_RATHER_THAN_AFTER_THE_TYPE);
        }
        if type_node.kind == SyntaxKind::RestType {
            self.grammar_error_on_node(&type_node, &A_LABELED_TUPLE_ELEMENT_IS_DECLARED_AS_REST_WITH_A_BEFORE_THE_NAME_RATHER_THAN_BEFORE_THE_TYPE);
        }
        self.check_source_element(&type_node);
        self.get_type_from_type_node(node);
    }

    pub fn check_new_target_meta_property(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let container = get_new_target_container(node);
        let Some(container) = container else {
            self.error_message(node,META_PROPERTY_0_IS_ONLY_ALLOWED_IN_THE_BODY_OF_A_FUNCTION_DECLARATION_FUNCTION_EXPRESSION_OR_CONSTRUCTOR, &["new.target".to_string()]);
            return self.error_type();
        };
        if container.kind == SyntaxKind::Constructor {
            let parent = container.parent().unwrap_or_else(|| Arc::clone(&container));
            let symbol = self.get_symbol_of_declaration(&parent);
            return match symbol {
                Some(symbol) => self.get_type_of_symbol(&symbol),
                None => self.error_type(),
            };
        }
        let symbol = self.get_symbol_of_declaration(&container);
        match symbol {
            Some(symbol) => self.get_type_of_symbol(&symbol),
            None => self.error_type(),
        }
    }

    pub fn check_no_type_arguments(&mut self, node: &Arc<Node>, symbol: Option<&Arc<Symbol>>) -> bool {
        if node.type_arguments().map(|l| !l.nodes.is_empty()).unwrap_or(false) {
            let type_name = match symbol {
                Some(symbol) => self.symbol_to_string(symbol),
                None => {
                    let type_reference_name = match &node.data {
                        NodeData::TypeReferenceNode(data) => Arc::clone(&data.type_name),
                        _ => Arc::clone(node),
                    };
                    declaration_name_to_string(Some(&type_reference_name))
                }
            };
            self.error_message(node,TYPE_0_IS_NOT_GENERIC, &[type_name]);
            return false;
        }
        true
    }

    pub fn check_node_deferred(&mut self, node: &Arc<Node>) {
        if let Some(enclosing_file) = self.get_source_file_of_node(node) {
            let links = self.source_file_links.get_or_default(&enclosing_file);
            if !links.type_checked {
                super::m1b::r25k1_defs::source_file_deferred_nodes_insert(&enclosing_file, Arc::clone(node));
            }
        }
    }

    pub fn check_parameter(&mut self, node: &Arc<Node>) {
        self.check_grammar_modifiers(node);
        self.check_variable_like_declaration(node);
        let function_node = get_containing_function(node);
        let function_node = function_node.unwrap_or_else(|| Arc::clone(node));
        let name = node.name();
        let param_name = name
            .map(|n| if is_identifier(n) { n.text().to_string() } else { String::new() })
            .unwrap_or_default();
        if tsox_frontend::ast::has_syntactic_modifier(node, ModifierFlags::ParameterPropertyModifier) {
            if self.should_check_erasable_syntax(node) {
                self.error_message(node,THIS_SYNTAX_IS_NOT_ALLOWED_WHEN_ERASABLESYNTAXONLY_IS_ENABLED, &[]);
            }
            let is_ctor_with_body = function_node.kind == SyntaxKind::Constructor
                && function_body(&function_node).map(|b| tsox_frontend::ast::node_is_present(Some(&b))).unwrap_or(false);
            if !is_ctor_with_body {
                self.error_message(node,A_PARAMETER_PROPERTY_IS_ONLY_ALLOWED_IN_A_CONSTRUCTOR_IMPLEMENTATION, &[]);
            }
            if function_node.kind == SyntaxKind::Constructor && param_name == "constructor" {
                let name = name.unwrap();
                self.error_message(name,X_CONSTRUCTOR_CANNOT_BE_USED_AS_A_PARAMETER_PROPERTY_NAME, &[]);
            }
        }
        let is_binding_pattern_name = name.map(|n| tsox_frontend::ast::is_binding_pattern(n)).unwrap_or(false);
        if tsox_frontend::ast::mig::m3b::initializer(node).is_none()
            && is_optional_declaration(node)
            && is_binding_pattern_name
            && function_body(&function_node).is_some()
        {
            self.error_message(node,A_BINDING_PATTERN_PARAMETER_CANNOT_BE_OPTIONAL_IN_AN_IMPLEMENTATION_SIGNATURE, &[]);
        }
        if param_name == "this" || param_name == "new" {
            let index = tsox_frontend::ast::mig::m3b::parameters(&function_node).iter().position(|p| Arc::ptr_eq(p, node));
            if index != Some(0) {
                self.error_message(node,A_0_PARAMETER_MUST_BE_THE_FIRST_PARAMETER, &[param_name.clone()]);
            }
            if matches!(function_node.kind, SyntaxKind::Constructor | SyntaxKind::ConstructSignature | SyntaxKind::ConstructorType) {
                self.error_message(node,A_CONSTRUCTOR_CANNOT_HAVE_A_THIS_PARAMETER, &[]);
            }
            if function_node.kind == SyntaxKind::ArrowFunction {
                self.error_message(node,AN_ARROW_FUNCTION_CANNOT_HAVE_A_THIS_PARAMETER, &[]);
            }
            if tsox_frontend::ast::is_accessor(&function_node) {
                self.error_message(node,X_GET_AND_SET_ACCESSORS_CANNOT_DECLARE_THIS_PARAMETERS, &[]);
            }
        }
        if has_dot_dot_dot_token(node) && !is_binding_pattern_name {
            let symbol = self.get_resolved_symbol(node);
            let symbol_type = match &symbol { Some(s) => self.get_type_of_symbol(s), None => return };
            let reduced = self.get_reduced_type(&symbol_type);
            let any_readonly_array = self.any_readonly_array_type();
            if !self.is_type_assignable_to(&reduced, &any_readonly_array) {                self.error_message(node,A_REST_PARAMETER_MUST_BE_OF_AN_ARRAY_TYPE, &[]);
            }
        }
    }

    pub fn check_property_declaration(&mut self, node: &Arc<Node>) {
        if !self.check_grammar_modifiers(node) && !self.check_grammar_property(node) {
            let name = node.name();
            if let Some(name) = node.name() {
                self.check_grammar_computed_property_name(name);
            }
        }
        self.check_variable_like_declaration(node);
        self.set_node_links_for_private_identifier_scope(node);
        let has_abstract = tsox_frontend::ast::has_syntactic_modifier(node, ModifierFlags::Abstract);
        if has_abstract && node.kind == SyntaxKind::PropertyDeclaration && tsox_frontend::ast::mig::m3b::initializer(node).is_some() {
            let name = node.name();
            let name_string = name.map(|n| declaration_name_to_string(Some(n))).unwrap_or_default();
            self.error_message(node,PROPERTY_0_CANNOT_HAVE_AN_INITIALIZER_BECAUSE_IT_IS_MARKED_ABSTRACT, &[name_string]);
        }
    }

    pub fn check_property_signature(&mut self, node: &Arc<Node>) {
        let name = match &node.data {
            NodeData::PropertySignatureDeclaration(data) => Arc::clone(&data.name),
            _ => node.name().cloned().unwrap_or_else(|| Arc::clone(node)),
        };
        if tsox_frontend::ast::is_private_identifier(&name) {
            self.error_message(node,PRIVATE_IDENTIFIERS_ARE_NOT_ALLOWED_OUTSIDE_CLASS_BODIES, &[]);
        }
        self.check_property_declaration(node);
    }

    pub fn check_signature_declaration(&mut self, node: &Arc<Node>) {
        match node.kind {
            SyntaxKind::IndexSignature => {
                let _ = self.check_grammar_index_signature(node);
            }
            SyntaxKind::FunctionType
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::ConstructorType
            | SyntaxKind::CallSignature
            | SyntaxKind::Constructor
            | SyntaxKind::ConstructSignature => {
                let _ = self.check_grammar_function_like_declaration(node);
            }
            _ => {}
        }
        let function_flags = get_function_flags(Some(node));
        if !function_flags.contains(FunctionFlags::INVALID) {
            if function_flags == FunctionFlags::ASYNC_GENERATOR && self.language_version < LANGUAGE_FEATURE_MINIMUM_TARGET_ASYNC_GENERATORS {
                self.check_external_emit_helpers(
                    node,
                    (ExternalEmitHelpers::AsyncGenerator | ExternalEmitHelpers::AsyncDelegator | ExternalEmitHelpers::AsyncValues).bits(),
                );
            }
            if function_flags == FunctionFlags::ASYNC && self.language_version < LANGUAGE_FEATURE_MINIMUM_TARGET_ASYNC_FUNCTIONS {
                self.check_external_emit_helpers(node, ExternalEmitHelpers::Awaiter.bits());
            }
        }
        self.check_type_parameters(tsox_frontend::ast::mig::m3c::type_parameter_list(node).map(|l| &**l));
        self.check_unmatched_jsdoc_parameters(node);
        let parameters = tsox_frontend::ast::mig::m3b::parameters(node);
        self.check_source_elements_from_list(parameters);
        if matches!(
            node.kind,
            SyntaxKind::FunctionType
                | SyntaxKind::ConstructorType
                | SyntaxKind::CallSignature
                | SyntaxKind::ConstructSignature
        ) && let Some(parameter_list) = tsox_frontend::ast::mig::m3b::parameter_list(node)
        {
            self.check_parameter_implicit_any(node, parameter_list, 0);
        }
        let return_type_node = node.type_node();
        if let Some(return_type_node) = &return_type_node {
            self.check_source_element(return_type_node);
        }
        if self.no_implicit_any && return_type_node.is_none() {
            match node.kind {
                SyntaxKind::ConstructSignature => {
                    self.error_message(node,CONSTRUCT_SIGNATURE_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_ANY_RETURN_TYPE, &[]);
                }
                SyntaxKind::CallSignature => {
                    self.error_message(node,CALL_SIGNATURE_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_ANY_RETURN_TYPE, &[]);
                }
                _ => {}
            }
        }
        if let Some(return_type_node) = &return_type_node {
            if function_flags.0 & (FunctionFlags::INVALID.0 | FunctionFlags::GENERATOR.0) == FunctionFlags::GENERATOR.0 {
                let return_type = self.get_type_from_type_node(return_type_node);
                if Arc::ptr_eq(&return_type, &self.void_type()) {
                    self.error_message(return_type_node,A_GENERATOR_CANNOT_HAVE_A_VOID_TYPE_ANNOTATION, &[]);
                } else {
                    self.check_generator_instantiation_assignability_to_return_type(&return_type, function_flags, return_type_node);
                }
            } else if function_flags.0 & FunctionFlags::ASYNC_GENERATOR.0 == FunctionFlags::ASYNC.0 {
                self.check_async_function_return_type(node, return_type_node);
            }
        }
        if !tsox_frontend::ast::is_index_signature_declaration(node) {
            self.register_for_unused_identifiers_check(node);
        }
    }

    pub fn check_resolved_block_scoped_variable(&mut self, result: &Arc<Symbol>, error_location: &Arc<Node>) {
        if result.flags.intersects(SymbolFlags::Function.union(SymbolFlags::FunctionScopedVariable).union(SymbolFlags::Assignment))
            && result.flags.contains(SymbolFlags::Class)
        {
            return;
        }
        let declaration = result
            .declarations
            .iter()
            .find(|d| {
                tsox_frontend::ast::mig::m3f_3::is_block_or_catch_scoped(d)
                    || tsox_frontend::ast::is_class_like(d)
                    || tsox_frontend::ast::is_enum_declaration(d)
            })
            .cloned();
        let declaration = declaration.expect("checkResolvedBlockScopedVariable could not find block-scoped declaration");
        if !declaration.flags.contains(NodeFlags::Ambient) && !self.is_block_scoped_name_declared_before_use(&declaration, error_location) {
            let declaration_name_node = tsox_frontend::ast::get_name_of_declaration(&declaration)
                .unwrap_or_else(|| Arc::clone(&declaration));
            let declaration_name = declaration_name_to_string(Some(&declaration_name_node));
            let mut diagnostic: Option<Arc<Diagnostic>> = None;
            if result.flags.contains(SymbolFlags::BlockScopedVariable) {
                diagnostic = self.error_message(error_location,BLOCK_SCOPED_VARIABLE_0_USED_BEFORE_ITS_DECLARATION, &[declaration_name.clone()]);
            } else if result.flags.contains(SymbolFlags::Class) {
                diagnostic = self.error_message(error_location,CLASS_0_USED_BEFORE_ITS_DECLARATION, &[declaration_name.clone()]);
            } else if result.flags.contains(SymbolFlags::RegularEnum) {
                diagnostic = self.error_message(error_location,ENUM_0_USED_BEFORE_ITS_DECLARATION, &[declaration_name.clone()]);
            } else if self.compiler_options.get_isolated_modules() {
                diagnostic = self.error_message(error_location,ENUM_0_USED_BEFORE_ITS_DECLARATION, &[declaration_name.clone()]);
            }
            if let Some(diagnostic) = diagnostic {
                let related = create_diagnostic_for_node_message(&declaration,X_0_IS_DECLARED_HERE, &[declaration_name]);
                self.add_related_info_to_diagnostic(&diagnostic, (*related).clone());
            }
        }
    }

    pub fn check_source_elements(&mut self, nodes: &[Arc<Node>]) {
        for node in nodes {
            if self.is_canceled() {
                break;
            }
            self.check_source_element(node);
        }
    }

    pub fn check_source_element(&mut self, node: &Arc<Node>) -> bool {
        let save_current_node = self.current_node.clone();
        let save_within_unreachable_code = self.within_unreachable_code;
        self.current_node = Some(Arc::clone(node));
        self.instantiation_count = 0;
        self.check_source_element_worker(node);
        self.current_node = save_current_node;
        self.within_unreachable_code = save_within_unreachable_code;
        false
    }

    pub fn check_source_element_worker(&mut self, node: &Arc<Node>) {
        let file = self.get_source_file_of_node(node);
        let jsdocs = file.as_ref().map(|f| f.eager_jsdoc(node)).unwrap_or_default();
        for jsdoc in jsdocs.iter() {
            self.check_jsdoc_comments(jsdoc);
            let tags = match &jsdoc.data {
                NodeData::JSDoc(data) => data.tags.clone(),
                _ => None,
            };
            if let Some(tags) = tags {
                for tag in tags.nodes.iter() {
                    self.check_jsdoc_comments(tag);
                }
            }
        }
        if !self.within_unreachable_code && self.compiler_options.allow_unreachable_code != Tristate::True && self.check_source_element_unreachable(node) {
            self.within_unreachable_code = true;
        }
        match node.kind {
            SyntaxKind::TypeParameter => self.check_type_parameter(node),
            SyntaxKind::Parameter => self.check_parameter(node),
            SyntaxKind::PropertyDeclaration => self.check_property_declaration(node),
            SyntaxKind::PropertySignature => self.check_property_signature(node),
            SyntaxKind::ConstructorType | SyntaxKind::FunctionType | SyntaxKind::CallSignature | SyntaxKind::ConstructSignature | SyntaxKind::IndexSignature => {
                self.check_signature_declaration(node)
            }
            SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature => self.check_method_declaration(node),
            SyntaxKind::ClassStaticBlockDeclaration => self.check_class_static_block_declaration(node),
            SyntaxKind::Constructor => self.check_constructor_declaration(node),
            SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => self.check_accessor_declaration(node),
            SyntaxKind::TypeReference => self.check_type_reference_node(node),
            SyntaxKind::TypePredicate => self.check_type_predicate(node),
            SyntaxKind::TypeQuery => self.check_type_query(node),
            SyntaxKind::TypeLiteral => self.check_type_literal(node),
            SyntaxKind::ArrayType => self.check_array_type(node),
            SyntaxKind::TupleType => self.check_tuple_type(node),
            SyntaxKind::UnionType | SyntaxKind::IntersectionType => self.check_union_or_intersection_type(node),
            SyntaxKind::ParenthesizedType | SyntaxKind::OptionalType | SyntaxKind::RestType => {
                node.for_each_child(|child| {
                    self.check_source_element(child);
                    true
                });
            }
            SyntaxKind::ThisType => self.check_this_type(node),
            SyntaxKind::TypeOperator => self.check_type_operator(node),
            SyntaxKind::ConditionalType => self.check_conditional_type(node),
            SyntaxKind::InferType => self.check_infer_type(node),
            SyntaxKind::TemplateLiteralType => self.check_template_literal_type(node),
            SyntaxKind::ImportType => self.check_import_type(node),
            SyntaxKind::NamedTupleMember => self.check_named_tuple_member(node),
            SyntaxKind::IndexedAccessType => self.check_indexed_access_type(node),
            SyntaxKind::MappedType => self.check_mapped_type(node),
            SyntaxKind::FunctionDeclaration => self.check_function_declaration(node),
            SyntaxKind::Block | SyntaxKind::ModuleBlock => self.check_block(node),
            SyntaxKind::VariableStatement => self.check_variable_statement(node),
            SyntaxKind::ExpressionStatement => self.check_expression_statement(node),
            SyntaxKind::IfStatement => self.check_if_statement(node),
            SyntaxKind::DoStatement => self.check_do_statement(node),
            SyntaxKind::WhileStatement => self.check_while_statement(node),
            SyntaxKind::ForStatement => self.check_for_statement(node),
            SyntaxKind::ForInStatement => self.check_for_in_statement(node),
            SyntaxKind::ForOfStatement => self.check_for_of_statement(node),
            SyntaxKind::ContinueStatement | SyntaxKind::BreakStatement => self.check_break_or_continue_statement(node),
            SyntaxKind::ReturnStatement => self.check_return_statement(node),
            SyntaxKind::WithStatement => self.check_with_statement(node),
            SyntaxKind::SwitchStatement => self.check_switch_statement(node),
            SyntaxKind::LabeledStatement => self.check_labeled_statement(node),
            SyntaxKind::ThrowStatement => self.check_throw_statement(node),
            SyntaxKind::TryStatement => self.check_try_statement(node, false),
            SyntaxKind::VariableDeclaration => self.check_variable_declaration(node),
            SyntaxKind::BindingElement => self.check_binding_element(node),
            SyntaxKind::ClassDeclaration => self.check_class_declaration(node),
            SyntaxKind::InterfaceDeclaration => self.check_interface_declaration(node),
            SyntaxKind::TypeAliasDeclaration | SyntaxKind::JSTypeAliasDeclaration => self.check_type_alias_declaration(node),
            SyntaxKind::EnumDeclaration => self.check_enum_declaration(node),
            SyntaxKind::EnumMember => self.check_enum_member(node),
            SyntaxKind::ModuleDeclaration => self.check_module_declaration(node),
            SyntaxKind::ImportDeclaration | SyntaxKind::JSImportDeclaration => self.check_import_declaration(node),
            SyntaxKind::ImportEqualsDeclaration => self.check_import_equals_declaration(node),
            SyntaxKind::ExportDeclaration => self.check_export_declaration(node),
            SyntaxKind::ExportAssignment => self.check_export_assignment(node),            SyntaxKind::EmptyStatement => {
                self.check_grammar_statement_in_ambient_context(node);
            }
            SyntaxKind::DebuggerStatement => {
                self.check_grammar_statement_in_ambient_context(node);
            }
            SyntaxKind::MissingDeclaration => self.check_missing_declaration(node),
            SyntaxKind::JSDocNonNullableType | SyntaxKind::JSDocNullableType | SyntaxKind::JSDocAllType | SyntaxKind::JSDocTypeLiteral => {
                self.check_jsdoc_type(node)
            }
            _ => {}
        }
    }

    pub fn check_switch_statement(&mut self, node: &Arc<Node>) {
        self.check_grammar_statement_in_ambient_context(node);
        let mut first_default_clause: Option<Arc<Node>> = None;
        let mut has_duplicate_default_clause = false;
        let expression = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
        let expression_type = self.check_expression_ex(&expression, CheckMode::Normal);
        let case_block = match &node.data {
            NodeData::SwitchStatement(data) => Arc::clone(&data.case_block),
            _ => return,
        };
        let clauses = match &case_block.data {
            NodeData::CaseBlock(data) => Arc::clone(&data.clauses),
            _ => return,
        };
        for clause in clauses.iter() {
            if is_default_clause(clause) && !has_duplicate_default_clause {
                if first_default_clause.is_none() {
                    first_default_clause = Some(Arc::clone(clause));
                } else {
                    self.grammar_error_on_node(clause, &A_DEFAULT_CLAUSE_CANNOT_APPEAR_MORE_THAN_ONCE_IN_A_SWITCH_STATEMENT);
                    has_duplicate_default_clause = true;
                }
            }
            if tsox_frontend::ast::is_case_clause(clause) {
                let clause_expression = clause.expression().cloned().unwrap_or_else(|| Arc::clone(clause));
                let case_type = self.check_expression_ex(&clause_expression, CheckMode::Normal);
                if !self.is_type_equality_comparable_to(&expression_type, &case_type) {
                    self.check_type_comparable_to(&case_type, &expression_type, Some(&clause_expression), None);
                }
            }
            let statements = tsox_frontend::ast::mig::m3c::statements(clause);
            self.check_source_elements(statements);
            if self.compiler_options.no_fallthrough_cases_in_switch.is_true() {
                if let Some(flow_node) = crate::checker::mig::m2d_3::get_flow_node_of_node(clause) {
                    if self.is_reachable_flow_node(&flow_node) {
                        self.error_message(clause,FALLTHROUGH_CASE_IN_SWITCH, &[]);
                    }
                }
            }
        }
        if self.program.symbol_map().locals_of(&case_block).is_some() {
            self.register_for_unused_identifiers_check(&case_block);
        }
    }

    pub fn check_template_literal_type(&mut self, node: &Arc<Node>) {
        let template_spans = match &node.data {
            NodeData::TemplateLiteralTypeNode(data) => Arc::clone(&data.template_spans),
            _ => return,
        };
        for span in template_spans.iter() {
            let span_type = span.type_node().cloned().unwrap_or_else(|| Arc::clone(span));
            self.check_source_element(&span_type);
            let t = self.get_type_from_type_node(&span_type);
            let template_constraint_type = self.template_constraint_type();
            self.check_type_assignable_to(&t, &template_constraint_type, Some(&span_type), None);
        }
        self.get_type_from_type_node(node);
    }

    pub fn check_testing_known_truthy_callable_or_awaitable_or_enum_member_type(&mut self, cond_expr: &Arc<Node>, cond_type: &Arc<Type>, body: Option<&Arc<Node>>) {
        if !self.strict_null_checks {
            return;
        }
        self.check_testing_known_truthy_types(cond_expr, cond_type, body);
    }

    pub fn check_testing_known_truthy_types(&mut self, cond_expr: &Arc<Node>, cond_type: &Arc<Type>, body: Option<&Arc<Node>>) {
        let mut cond_expr = skip_parentheses(cond_expr);
        self.check_testing_known_truthy_type(&cond_expr, cond_type, body);
        while tsox_frontend::ast::is_binary_expression(&cond_expr) {
            let operator_kind = match &cond_expr.data {
                NodeData::BinaryExpression(data) => data.operator_token.kind,
                _ => break,
            };
            if operator_kind != SyntaxKind::BarBarToken && operator_kind != SyntaxKind::QuestionQuestionToken {
                break;
            }
            let left = match &cond_expr.data {
                NodeData::BinaryExpression(data) => Arc::clone(&data.left),
                _ => break,
            };
            cond_expr = skip_parentheses(&left);
            self.check_testing_known_truthy_type(&cond_expr, cond_type, body);
        }
    }

    pub fn check_this_type(&mut self, node: &Arc<Node>) {
        self.get_type_from_this_type_node(node);
    }

    pub fn check_throw_statement(&mut self, node: &Arc<Node>) {
        if !self.check_grammar_statement_in_ambient_context(node) {
            if let Some(throw_expr) = node.expression() {
                if is_identifier(throw_expr) && throw_expr.text().is_empty() {
                    self.grammar_error_at_pos(node, throw_expr.loc.pos as usize, 0, &LINE_BREAK_NOT_PERMITTED_HERE);
                }
            }
        }
        let throw_expr = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
        self.check_expression_ex(&throw_expr, CheckMode::Normal);
    }
}

pub(crate) fn function_body(node: &tsox_frontend::ast::Node) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::FunctionDeclaration(d) => d.body.clone(),
        NodeData::FunctionExpression(d) => Some(Arc::clone(&d.body)),
        NodeData::MethodDeclaration(d) => d.body.clone(),
        NodeData::ArrowFunction(d) => Some(Arc::clone(&d.body)),
        NodeData::ConstructorDeclaration(d) => d.body.clone(),
        NodeData::GetAccessorDeclaration(d) => d.body.clone(),
        NodeData::SetAccessorDeclaration(d) => d.body.clone(),
        _ => None,
    }
}

impl Checker {
    pub fn any_readonly_array_type(&mut self) -> Arc<Type> {
        match self.any_readonly_array_type.get() {
            Some(t) => Arc::clone(t),
            None => {
                let global_readonly_array_type = self
                    .global_readonly_array_type
                    .get()
                    .cloned()
                    .expect("globalReadonlyArrayType not initialized");
                let any_type = self.any_type();
                self.create_type_from_generic_global_type(&global_readonly_array_type, &[any_type])
            }
        }
    }

    pub fn template_constraint_type(&mut self) -> Arc<Type> {
        self.get_union_type(vec![
            self.string_type(),
            self.number_type(),
            self.boolean_type(),
            self.bigint_type(),
            self.null_type(),
            self.undefined_type(),
        ])
    }

    pub fn check_export_assignment(&mut self, node: &Arc<Node>) {
        let is_export_equals = match &node.data {
            NodeData::ExportAssignment(d) => d.is_export_equals,
            _ => false,
        };
        let expression = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
        self.check_expression_cached(&expression);
        let illegal_context_message = if is_export_equals {
            &AN_EXPORT_ASSIGNMENT_MUST_BE_AT_THE_TOP_LEVEL_OF_A_FILE_OR_MODULE_DECLARATION
        } else {
            &A_DEFAULT_EXPORT_MUST_BE_AT_THE_TOP_LEVEL_OF_A_FILE_OR_MODULE_DECLARATION
        };
        if self.check_grammar_module_element_context(node, illegal_context_message) {
            return;
        }
        if self.should_check_erasable_syntax(node) && is_export_equals && !node.flags.contains(NodeFlags::Ambient) {
            self.error_message(node,THIS_SYNTAX_IS_NOT_ALLOWED_WHEN_ERASABLESYNTAXONLY_IS_ENABLED, &[]);
        }
        // Go checkExportAssignment：ambient 下表达式须为标识符或限定名；
        // undefined 在 Go parser 产 Identifier，移植侧 scanner 归
        // UndefinedKeyword，等价视为标识符
        let expression_is_entity_name = tsox_frontend::ast::is_entity_name_expression(&expression)
            || expression.kind == SyntaxKind::UndefinedKeyword;
        if self.declaration_is_ambient(node) && !expression_is_entity_name {
            self.grammar_error_on_node(
                &expression,
                &THE_EXPRESSION_OF_AN_EXPORT_ASSIGNMENT_MUST_BE_AN_IDENTIFIER_OR_QUALIFIED_NAME_IN_AN_AMBIENT_CONTEXT,
            );
        }
    }
}
