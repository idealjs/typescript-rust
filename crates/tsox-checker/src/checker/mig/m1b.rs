#![allow(unused_imports)]

use crate::checker::checker::*;
use std::sync::Arc;
use tsox_frontend::ast::NodeData;
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::NodeFlags;
use tsox_frontend::ast::SyntaxKind;
use tsox_frontend::ast::SymbolFlags;
use tsox_frontend::ast::{is_array_literal_expression, is_binding_pattern, is_object_literal_expression, is_variable_declaration_list};
use tsox_frontend::ast::mig::m3e::FunctionFlags;
use crate::checker::utilities_get_assignment_target::get_containing_function_or_class_static_block;
use tsox_frontend::ast::mig::m3e::get_function_flags;
use crate::checker::utilities_has_only_expression_initialization::has_export_assignment_symbol;
use crate::checker::mig::wc3_2::is_const_enum_object_type;
use crate::checker::utilities_has_only_expression_initialization::is_this_type_parameter;
use crate::checker::utilities_has_only_expression_initialization::is_type_assertion;
use tsox_core::diagnostics::messages_generated::*;
use crate::checker::checker_es_symbol::walk_up_parenthesized_expressions;
use tsox_frontend::ast::mig::m3b::{module_specifier as node_module_specifier, property_name_or_name};
use tsox_frontend::ast::mig::m3g_3::module_export_name_is_default;
use tsox_frontend::ast::mig::m3e_4::get_import_attributes;
use crate::checker::mig::wc3::ReferenceHint;
use crate::checker::types_type_flags_instantiable_non_primitive::TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE;
use crate::checker::types_type_id::TYPE_FLAGS_LITERAL;
use crate::checker::mig::m2c::r18k3_defs::LanguageFeatureMinimumTarget;
use super::m2e_2::r24k19_defs::take_intra_expression_inference_sites;

#[path = "r19k6_defs.rs"]
pub mod r19k6_defs;
pub use r19k6_defs::*;

#[path = "r19k6_checker_ext.rs"]
pub mod r19k6_checker_ext;

#[path = "r20k1_ext.rs"]
pub mod r20k1_ext;

#[path = "r23k1_defs.rs"]
pub mod r23k1_defs;
pub use r23k1_defs::*;

#[path = "r24k1_defs.rs"]
pub mod r24k1_defs;

#[path = "r25k1_defs.rs"]
pub mod r25k1_defs;
pub use r24k1_defs::*;

use tsox_frontend::ast::mig::m3f_2::has_context_sensitive_parameters;

pub(crate) const LANGUAGE_FEATURE_MINIMUM_TARGET_FOR_AWAIT_OF: tsox_core::core::compiler_options::ScriptTarget = tsox_core::core::compiler_options::ScriptTarget::ES2018;
pub(crate) const LANGUAGE_FEATURE_MINIMUM_TARGET_ASYNC_GENERATORS: tsox_core::core::compiler_options::ScriptTarget = tsox_core::core::compiler_options::ScriptTarget::ES2018;
pub(crate) const LANGUAGE_FEATURE_MINIMUM_TARGET_ASYNC_FUNCTIONS: tsox_core::core::compiler_options::ScriptTarget = tsox_core::core::compiler_options::ScriptTarget::ES2017;

pub(crate) fn is_global_source_file(node: &Node) -> bool {
    node.kind == SyntaxKind::SourceFile
}

pub(crate) fn is_node_descendant_of(node: &Node, ancestor: &Node) -> bool {
    let mut current = node.parent();
    while let Some(n) = current {
        if std::ptr::eq(std::sync::Arc::as_ptr(&n) as *const Node, ancestor) {
            return true;
        }
        current = n.parent();
    }
    false
}

impl Checker {

    pub fn check_export_declaration(&mut self, node: &Arc<Node>) {
        let diagnostic = if tsox_frontend::ast::is_in_js_file(node) {
            AN_EXPORT_DECLARATION_CAN_ONLY_BE_USED_AT_THE_TOP_LEVEL_OF_A_MODULE
        } else {
            AN_EXPORT_DECLARATION_CAN_ONLY_BE_USED_AT_THE_TOP_LEVEL_OF_A_NAMESPACE_OR_MODULE
        };
        if self.check_grammar_module_element_context(node, &diagnostic) {
            self.check_external_module_name_in_global_scope(node);
            return;
        }
        let export_decl = match &node.data {
            NodeData::ExportDeclaration(data) => data.clone(),
            _ => return,
        };
        let has_modifiers = export_decl.modifiers.is_some();
        if !self.check_grammar_modifiers(node) && has_modifiers {
            self.grammar_error_on_first_token(node, &AN_EXPORT_DECLARATION_CANNOT_HAVE_MODIFIERS);
        }
        self.check_grammar_export_declaration(node);
        let module_specifier = export_decl.module_specifier.clone();
        let export_clause = export_decl.export_clause.clone();
        if module_specifier.is_none() || self.check_external_import_or_export_declaration(node) {
            if let Some(clause) = &export_clause {
                if !tsox_frontend::ast::is_namespace_export(clause) {
                    if let NodeData::NamedExports(named) = &clause.data {
                        for binding in named.elements.iter() {
                            self.check_export_specifier(binding);
                        }
                    }
                    let parent = node.parent();
                    let in_ambient_external_module = parent
                        .as_ref()
                        .map(|p| {
                            tsox_frontend::ast::is_module_block(p)
                                && p
                                    .parent()
                                    .map(|gp| tsox_frontend::ast::is_ambient_module(&gp))
                                    .unwrap_or(false)
                        })
                        .unwrap_or(false);
                    let in_ambient_namespace_declaration = !in_ambient_external_module
                        && parent.as_ref().map(|p| tsox_frontend::ast::is_module_block(p)).unwrap_or(false)
                        && module_specifier.is_none()
                        && node.flags.contains(NodeFlags::Ambient);
                    let parent_is_source_file = parent.as_ref().map(|p| p.kind == SyntaxKind::SourceFile).unwrap_or(false);
                    if !parent_is_source_file && !in_ambient_external_module && !in_ambient_namespace_declaration {
                        self.error_message(node,EXPORT_DECLARATIONS_ARE_NOT_PERMITTED_IN_A_NAMESPACE, &[]);
                    }
                } else {
                    let import_attributes_type = self.get_type_from_import_attributes(get_import_attributes(node).as_ref());
                    let module_symbol = self.resolve_external_module_name_worker(
                        node,
                        module_specifier.as_ref(),
                        Some(&CANNOT_FIND_MODULE_0_OR_ITS_CORRESPONDING_TYPE_DECLARATIONS),
                        false,
                        false,
                        import_attributes_type.as_ref(),
                    );
                    match module_symbol {
                        Some(symbol) if has_export_assignment_symbol(&symbol) => {
                            let specifier = module_specifier.clone().unwrap();
                            let name = self.symbol_to_string(&symbol);
                            self.error_message(&specifier,MODULE_0_USES_EXPORT_AND_CANNOT_BE_USED_WITH_EXPORT_ASTERISK, &[name]);
                        }
                        _ => {
                            if let Some(clause) = &export_clause {
                                self.check_alias_symbol(clause);
                                let name = clause.name().expect("namespace export name");
                                self.check_module_export_name(&name, true);
                            }
                        }
                    }
                    if self.emit_module_format_of_node_source_file(node) == ModuleKind::CommonJS {
                        if export_clause.is_some() {
                            self.check_external_emit_helpers(node, ExternalEmitHelpers::ImportStar.bits());
                        } else {
                            self.check_external_emit_helpers(node, ExternalEmitHelpers::ExportStar.bits());
                        }
                    }
                }
            } else {
                let import_attributes_type = self.get_type_from_import_attributes(get_import_attributes(node).as_ref());
                let module_symbol = self.resolve_external_module_name_worker(
                    node,
                    module_specifier.as_ref(),
                    Some(&CANNOT_FIND_MODULE_0_OR_ITS_CORRESPONDING_TYPE_DECLARATIONS),
                    false,
                    false,
                    import_attributes_type.as_ref(),
                );
                if let Some(symbol) = module_symbol {
                    if has_export_assignment_symbol(&symbol) {
                        let specifier = module_specifier.clone().unwrap();
                        let name = self.symbol_to_string(&symbol);
                        self.error_message(&specifier,MODULE_0_USES_EXPORT_AND_CANNOT_BE_USED_WITH_EXPORT_ASTERISK, &[name]);
                    }
                }
                if self.emit_module_format_of_node_source_file(node) == ModuleKind::CommonJS {
                    self.check_external_emit_helpers(node, ExternalEmitHelpers::ExportStar.bits());
                }
            }
        }
        self.check_import_attributes(node);
    }

    pub fn check_export_specifier(&mut self, node: &Arc<Node>) {
        self.check_alias_symbol(node);
        let has_module_specifier = node
            .parent()
            .and_then(|p| p.parent())
            .and_then(|gp| node_module_specifier(&gp).cloned())
            .is_some();
        let (property_name, name) = match &node.data {
            NodeData::ExportSpecifier(data) => (data.property_name.clone(), Arc::clone(&data.name)),
            _ => (None, Arc::clone(node)),
        };
        if let Some(property_name) = &property_name {
            self.check_module_export_name(property_name, has_module_specifier);
        }
        self.check_module_export_name(&name, true);

        if !has_module_specifier {
            let exported_name = property_name_or_name(node).cloned().unwrap_or_else(|| Arc::clone(node));
            if exported_name.kind == SyntaxKind::StringLiteral {
                return;
            }
            let symbol = self.resolve_name(
                exported_name.text(),
                &exported_name,
                SymbolFlags::VALUE.union(SymbolFlags::TYPE).union(SymbolFlags::NAMESPACE).union(SymbolFlags::Alias),
                false,
            );
            let is_special = symbol
                .as_ref()
                .map(|s| {
                    self.undefined_symbol.as_ref().is_some_and(|u| Arc::ptr_eq(s, u))
                        || self.global_this_symbol.as_ref().is_some_and(|u| Arc::ptr_eq(s, u))
                        || s
                            .declarations
                            .first()
                            .map(|d| Checker::get_declaration_container(d).map(|c| is_global_source_file(&c)).unwrap_or(false))
                            .unwrap_or(false)
                })
                .unwrap_or(false);
            if is_special {
                let text = exported_name.text();
                self.error_message(&exported_name,CANNOT_EXPORT_0_ONLY_LOCAL_DECLARATIONS_CAN_BE_EXPORTED_FROM_A_MODULE, &[text.to_string()]);
            } else {
                self.mark_linked_references(node, ReferenceHint::ExportSpecifier, None, None);
            }
        } else {
            if self.emit_module_format_of_node_source_file(node) == ModuleKind::CommonJS
                && module_export_name_is_default(property_name_or_name(node).unwrap_or(node))
            {
                self.check_external_emit_helpers(node, ExternalEmitHelpers::ImportDefault.bits());
            }
        }
    }

    pub fn check_expression_cached(&mut self, node: &Arc<Node>) -> Arc<Type> {
        self.check_expression_cached_ex(node, CheckMode::Normal)
    }

    pub fn check_expression_cached_ex(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> {
        if check_mode != CheckMode::Normal {
            return self.check_expression_ex(node, check_mode);
        }
        if self.type_node_links.get(node).and_then(|l| l.resolved_type.clone()).is_none() {
            let save_flow_loop_stack = std::mem::take(&mut self.flow_loop_stack);
            let save_flow_type_cache = std::mem::take(&mut self.flow_type_cache);
            let resolved = self.check_expression_ex(node, check_mode);
            self.type_node_links.get_or_default(node).resolved_type = Some(Arc::clone(&resolved));
            self.flow_type_cache = save_flow_type_cache;
            self.flow_loop_stack = save_flow_loop_stack;
            return resolved;
        }
        self.type_node_links.get(node).and_then(|l| l.resolved_type.clone()).unwrap()
    }

    pub fn check_expression_ex(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> {
        let save_current_node = self.current_node.clone();
        self.current_node = Some(Arc::clone(node));
        self.instantiation_count = 0;
        let uninstantiated_type = self.check_expression_worker(node, check_mode);
        let t = self.instantiate_type_with_single_generic_call_signature(node, &uninstantiated_type, check_mode);
        if is_const_enum_object_type(&t) {
            self.check_const_enum_access(node, &t);
        }
        self.current_node = save_current_node;
        t
    }

    pub fn check_expression_for_mutable_location(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> {
        let t = self.check_expression_ex(node, check_mode);
        if self.is_const_context(node) {
            self.get_regular_type_of_literal_type(&t)
        } else if is_type_assertion(node) {
            t
        } else {
            let contextual = self.get_contextual_type(node, ContextFlags::empty())
                .map(|ct| self.instantiate_contextual_type(&ct, node, ContextFlags::empty()));
            self.get_widened_literal_like_type_for_contextual_type(&t, contextual.as_ref())
        }
    }

    pub fn check_expression_statement(&mut self, node: &Arc<Node>) {
        self.check_grammar_statement_in_ambient_context(node);
        if let NodeData::ExpressionStatement(data) = &node.data {
            self.check_expression(&data.expression);
        }
    }

    pub fn check_expression_with_contextual_type(&mut self, node: &Arc<Node>, contextual_type: &Arc<Type>, inference_context: Option<&Arc<InferenceContext>>, check_mode: CheckMode) -> Arc<Type> {
        let context_node = self.get_context_node(node);
        self.push_contextual_type(&context_node, contextual_type, false);
        if let Some(ctx) = inference_context {
            self.push_inference_context(&context_node, ctx);
        }
        let mut extra = CheckMode::Contextual;
        if inference_context.is_some() {
            extra |= CheckMode::Inferential;
        }
        let mut t = self.check_expression_ex(node, check_mode | extra);
        if let Some(context) = inference_context {
            take_intra_expression_inference_sites(context);
        }
        let instantiated = self.instantiate_contextual_type(contextual_type, node, ContextFlags::empty());
        if self.maybe_type_of_kind(&t, TYPE_FLAGS_LITERAL) && self.is_literal_of_contextual_type(&t, &instantiated) {
            t = self.get_regular_type_of_literal_type(&t);
        }
        self.pop_inference_context();
        self.pop_contextual_type();
        t
    }

    pub fn check_expression_with_type_arguments(&mut self, node: &Arc<Node>) -> Arc<Type> {
        self.check_grammar_expression_with_type_arguments(node);
        if let Some(type_arguments) = node.type_arguments() {
            self.check_source_elements_from_list(&type_arguments.nodes);
        }
        if tsox_frontend::ast::is_expression_with_type_arguments(node) {
            let parent = node.parent().map(|p| walk_up_parenthesized_expressions(&p));
            if let Some(parent) = parent {
                if let NodeData::BinaryExpression(binary) = &parent.data {
                    if binary.operator_token.kind == SyntaxKind::InstanceOfKeyword && is_node_descendant_of(node, &binary.right) {
                        self.error_message(node,THE_RIGHT_HAND_SIDE_OF_AN_INSTANCEOF_EXPRESSION_MUST_NOT_BE_AN_INSTANTIATION_EXPRESSION, &[]);
                    }
                }
            }
        }
        let expr_type;
        if tsox_frontend::ast::is_expression_with_type_arguments(node) {
            let expression = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
            expr_type = self.check_expression_ex(&expression, CheckMode::Normal);
        } else {
            let expr_name = match &node.data {
                NodeData::TypeQueryNode(data) => Arc::clone(&data.expr_name),
                _ => Arc::clone(node),
            };
            if tsox_frontend::ast::is_this_identifier(Some(expr_name.as_ref())) {
                expr_type = self.check_this_expression(&expr_name);
            } else {
                expr_type = self.check_expression_ex(&expr_name, CheckMode::Normal);
            }
        }
        self.get_instantiation_expression_type(&expr_type, node)
    }

    pub fn is_skip_direct_inference_node(&self, node: &Arc<Node>) -> bool {
        crate::checker::mig::m2h::r21k10_defs::skip_direct_inference_nodes_has(
            Arc::as_ptr(node) as *const () as usize,
        )
    }

    pub fn check_indexed_access(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> {
        if node.flags.contains(NodeFlags::OptionalChain) {
            return self.check_element_access_chain(node, check_mode);
        }
        let expression = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
        let expr_type = self.check_non_null_expression(&expression);
        self.check_element_access_expression(node, &expr_type, check_mode)
    }

    pub fn check_expression_worker(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> {
        match node.kind {
            SyntaxKind::Identifier => self.check_identifier(node, check_mode),
            SyntaxKind::PrivateIdentifier => {
                self.check_private_identifier_expression(node);
                self.error_type()
            }
            SyntaxKind::ThisKeyword => self.check_this_expression(node),
            SyntaxKind::SuperKeyword => {
                self.check_super_expression(node);
                self.get_type_of_node(node)
            }
            SyntaxKind::NullKeyword => Arc::clone(&self.null_widening_type),
            SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral => {
                if self.is_skip_direct_inference_node(node) {
                    return Arc::clone(&self.blocked_string_type);
                }
                let literal = self.get_string_literal_type(node.text());
                self.get_fresh_type_of_literal_type(&literal)
            }
            SyntaxKind::NumericLiteral => {
                self.check_grammar_numeric_literal(node);
                let value = jsnum_from_string(&node.text());
                let literal = self.get_number_literal_type(value);
                self.get_fresh_type_of_literal_type(&literal)
            }
            SyntaxKind::BigIntLiteral => {
                self.check_grammar_big_int_literal(node);
                let pseudo = parse_pseudo_big_int(&node.text());
                let literal = self.get_big_int_literal_type(pseudo);
                self.get_fresh_type_of_literal_type(&literal)
            }
            SyntaxKind::TrueKeyword => self.true_type(),
            SyntaxKind::FalseKeyword => self.false_type(),
            SyntaxKind::TemplateExpression => self.check_template_expression(node),
            SyntaxKind::RegularExpressionLiteral => self.check_regular_expression_literal(node),
            SyntaxKind::ArrayLiteralExpression => self.check_array_literal(node, check_mode),
            SyntaxKind::ObjectLiteralExpression => self.check_object_literal(node, check_mode),
            SyntaxKind::PropertyAccessExpression => self.check_property_access_expression(node, check_mode, false),
            SyntaxKind::QualifiedName => self.check_qualified_name(node, check_mode),
            SyntaxKind::ElementAccessExpression => self.check_indexed_access(node, check_mode),
            SyntaxKind::CallExpression => {
                if tsox_frontend::ast::is_import_call(node) {
                    self.check_import_call_expression(node)
                } else {
                    self.check_call_expression(node, check_mode)
                }
            }
            SyntaxKind::NewExpression => self.check_call_expression(node, check_mode),
            SyntaxKind::TaggedTemplateExpression => self.check_tagged_template_expression(node),
            SyntaxKind::ParenthesizedExpression => self.check_parenthesized_expression(node, check_mode),
            SyntaxKind::ClassExpression => self.check_class_expression(node),
            SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction => self.check_function_expression_or_object_literal_method(node, check_mode),
            SyntaxKind::TypeAssertionExpression | SyntaxKind::AsExpression => self.check_assertion(node, check_mode),
            SyntaxKind::TypeOfExpression => self.check_type_of_expression(node),
            SyntaxKind::NonNullExpression => self.check_non_null_assertion(node),
            SyntaxKind::ExpressionWithTypeArguments => self.check_expression_with_type_arguments(node),
            SyntaxKind::SatisfiesExpression => self.check_satisfies_expression(node),
            SyntaxKind::MetaProperty => self.check_meta_property(node),
            SyntaxKind::DeleteExpression => self.check_delete_expression(node),
            SyntaxKind::VoidExpression => self.check_void_expression(node),
            SyntaxKind::AwaitExpression => self.check_await_expression(node),
            SyntaxKind::PrefixUnaryExpression => self.check_prefix_unary_expression(node),
            SyntaxKind::PostfixUnaryExpression => self.check_postfix_unary_expression(node),
            SyntaxKind::BinaryExpression => {
                self.check_binary_expression(node);
                self.get_type_of_node(node)
            }
            SyntaxKind::ConditionalExpression => self.check_conditional_expression(node, check_mode),
            SyntaxKind::SpreadElement => self.check_spread_expression(node, check_mode),
            SyntaxKind::OmittedExpression => Arc::clone(&self.undefined_widening_type),
            SyntaxKind::YieldExpression => self.check_yield_expression(node),
            SyntaxKind::SyntheticExpression => self.check_synthetic_expression(node),
            SyntaxKind::JsxExpression => self.check_jsx_expression(node, check_mode.bits()),
            SyntaxKind::JsxElement => {
                self.check_jsx_element(node);
                self.get_type_of_node(node)
            }
            SyntaxKind::JsxSelfClosingElement => self.check_jsx_self_closing_element(node, check_mode.bits()),
            SyntaxKind::JsxFragment => self.check_jsx_fragment(node),
            SyntaxKind::JsxAttributes => self.check_jsx_attributes(node, check_mode.bits()),
            SyntaxKind::JsxOpeningElement => panic!("Should never directly check a JsxOpeningElement"),
            _ => self.error_type(),
        }
    }

    pub fn check_for_in_statement(&mut self, node: &Arc<Node>) {
        let data = match &node.data {
            NodeData::ForInOrOfStatement(data) => data.clone(),
            _ => return,
        };
        self.check_grammar_for_in_or_for_of_statement(node);
        let expr_type = self.check_expression_ex(&data.expression, CheckMode::Normal);
        let right_type = self.get_non_nullable_type_if_needed(&expr_type);
        if is_variable_declaration_list(&data.initializer) {
            if let NodeData::VariableDeclarationList(list) = &data.initializer.data {
                if let Some(first) = list.declarations.iter().next() {
                    if let Some(name) = first.name() {
                        if is_binding_pattern(&name) {
                            self.error_message(&name,THE_LEFT_HAND_SIDE_OF_A_FOR_IN_STATEMENT_CANNOT_BE_A_DESTRUCTURING_PATTERN, &[]);
                        }
                    }
                }
            }
            self.check_variable_declaration_list(&data.initializer);
        } else {
            let var_expr = Arc::clone(&data.initializer);
            let left_type = self.check_expression_ex(&var_expr, CheckMode::Normal);
            if is_array_literal_expression(&var_expr) || is_object_literal_expression(&var_expr) {
                self.error_message(&var_expr,THE_LEFT_HAND_SIDE_OF_A_FOR_IN_STATEMENT_CANNOT_BE_A_DESTRUCTURING_PATTERN, &[]);
            } else {
                let index_type = self.get_index_type_or_string(&right_type);
                if !self.is_type_assignable_to(&index_type, &left_type) {
                    self.error_message(&var_expr,THE_LEFT_HAND_SIDE_OF_A_FOR_IN_STATEMENT_MUST_BE_OF_TYPE_STRING_OR_ANY, &[]);
                } else {
                self.check_reference_expression(
                    &var_expr,
                    THE_LEFT_HAND_SIDE_OF_A_FOR_IN_STATEMENT_MUST_BE_A_VARIABLE_OR_A_PROPERTY_ACCESS,
                    THE_LEFT_HAND_SIDE_OF_A_FOR_IN_STATEMENT_MAY_NOT_BE_AN_OPTIONAL_PROPERTY_ACCESS,
                );
                }
            }
        }
        if Arc::ptr_eq(&right_type, &self.never_type())
            || !self.is_type_assignable_to_kind(&right_type, TypeFlags::NonPrimitive | TYPE_FLAGS_INSTANTIABLE_NON_PRIMITIVE)
        {
            let type_string = self.type_to_string(&right_type);
            self.error_message(&data.expression,THE_RIGHT_HAND_SIDE_OF_A_FOR_IN_STATEMENT_MUST_BE_OF_TYPE_ANY_AN_OBJECT_TYPE_OR_A_TYPE_PARAMETER_BUT_HERE_HAS_TYPE_0, &[type_string]);
        }
        self.check_source_element(&data.statement);
        if self.program.symbol_map().locals_of(node).is_some() {
            self.register_for_unused_identifiers_check(node);
        }
    }

    pub fn check_for_of_statement(&mut self, node: &Arc<Node>) {
        let data = match &node.data {
            NodeData::ForInOrOfStatement(data) => data.clone(),
            _ => return,
        };
        self.check_grammar_for_in_or_for_of_statement(node);
        let container = get_containing_function_or_class_static_block(node);
        if let Some(await_modifier) = &data.await_modifier {
            let in_static_block = container
                .as_ref()
                .map(|c| c.kind == SyntaxKind::ClassStaticBlockDeclaration)
                .unwrap_or(false);
            if in_static_block {
                self.grammar_error_on_node(await_modifier, &X_FOR_AWAIT_LOOPS_CANNOT_BE_USED_INSIDE_A_CLASS_STATIC_BLOCK);
            } else {
                let function_flags = container.as_ref().map(|c| get_function_flags(Some(c))).unwrap_or(FunctionFlags::NORMAL);
                if function_flags == FunctionFlags::ASYNC && self.language_version < LANGUAGE_FEATURE_MINIMUM_TARGET_FOR_AWAIT_OF {
                    self.check_external_emit_helpers(node, ExternalEmitHelpers::AsyncValues.bits());
                }
            }
        }
        if is_variable_declaration_list(&data.initializer) {
            self.check_variable_declaration_list(&data.initializer);
        } else {
            let var_expr = Arc::clone(&data.initializer);
            let iterated_type = self.check_right_hand_side_of_for_of(node);
            if is_array_literal_expression(&var_expr) || is_object_literal_expression(&var_expr) {
                let target = iterated_type.clone().unwrap_or_else(|| self.error_type());
                self.check_destructuring_assignment_ex(&var_expr, &target, false);
            } else {
                let left_type = self.check_expression_ex(&var_expr, CheckMode::Normal);
                self.check_reference_expression(
                    &var_expr,
                    THE_LEFT_HAND_SIDE_OF_A_FOR_OF_STATEMENT_MUST_BE_A_VARIABLE_OR_A_PROPERTY_ACCESS,
                    THE_LEFT_HAND_SIDE_OF_A_FOR_OF_STATEMENT_MAY_NOT_BE_AN_OPTIONAL_PROPERTY_ACCESS,
                );
                if let Some(iterated_type) = &iterated_type {
                    self.check_type_assignable_to_and_optionally_elaborate(iterated_type, &left_type, Some(&var_expr), Some(&data.expression), None, None);
                }
            }
        }
        self.check_source_element(&data.statement);
        if self.program.symbol_map().locals_of(node).is_some() {
            self.register_for_unused_identifiers_check(node);
        }
    }

    pub fn check_for_statement(&mut self, node: &Arc<Node>) {
        if !self.check_grammar_statement_in_ambient_context(node) {
            if let Some(init) = node.initializer() {
                if init.kind == SyntaxKind::VariableDeclarationList {
                    self.check_grammar_variable_declaration_list(&init);
                }
            }
        }
        let data = match &node.data {
            NodeData::ForStatement(data) => data.clone(),
            _ => return,
        };
        if let Some(initializer) = &data.initializer {
            if is_variable_declaration_list(initializer) {
                self.check_variable_declaration_list(initializer);
            } else {
                self.check_expression(initializer);
            }
        }
        if let Some(condition) = &data.condition {
            self.check_truthiness_expression(condition, CheckMode::Normal);
        }
        if let Some(incrementor) = &data.incrementor {
            self.check_expression(incrementor);
        }
        self.check_source_element(&data.statement);
        if self.program.symbol_map().locals_of(node).is_some() {
            self.register_for_unused_identifiers_check(node);
        }
    }

    pub fn check_function_expression_or_object_literal_method(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> {
        self.check_node_deferred(node);
        if tsox_frontend::ast::is_function_expression(node) {
            self.check_collisions_for_declaration_name(node, node.name());
        }
        if check_mode.contains(CheckMode::SkipContextSensitive) && self.is_context_sensitive(node) {
            if node.type_node().is_none() && !has_context_sensitive_parameters(node) {
                let contextual_signature = self.get_contextual_signature(node);
                if let Some(contextual_signature) = contextual_signature {
                    if let Some(return_type) = self.get_return_type_of_signature(&contextual_signature) {
                        if self.could_contain_type_variables(&return_type) {
                            if let Some(cached) = r23k1_defs::context_free_types_get(node) {
                                return cached;
                            }
                            let return_type = self.get_return_type_from_body(node, check_mode);
                            let return_only_signature = self.new_signature(
                                SignatureFlags::IsNonInferrable,
                                None,
                                &[],
                                None,
                                &[],
                                &return_type,
                                None,
                                0,
                            );
                            let symbol = self.program.symbol_map().symbol_of(node).cloned();
                            let mut return_only_type = self.new_anonymous_type(
                                symbol.as_ref().expect("function expression symbol"),
                                SymbolTable::new(),
                                vec![return_only_signature],
                                vec![],
                                vec![],
                            );
                            if let Some(return_only_type_mut) = Arc::get_mut(&mut return_only_type) {
                                return_only_type_mut.object_flags |= ObjectFlags::NonInferrableType;
                            }
                            r23k1_defs::context_free_types_insert(node, &return_only_type);
                            return return_only_type;
                        }
                    }
                }
            }
            return self.any_function_type();
        }
        let has_grammar_error = self.check_grammar_function_like_declaration(node);
        if !has_grammar_error && tsox_frontend::ast::is_function_expression(node) {
            self.check_grammar_for_generator(node);
        }
        if let Some(full_signature) = function_like_data_full_signature(node) {
            let signature_type = self.get_type_from_type_node(&full_signature);
            if self.get_contextual_call_signature(&signature_type, node).is_none() {
                self.error_message(&full_signature,A_JSDOC_TYPE_TAG_ON_A_FUNCTION_MUST_HAVE_A_SIGNATURE_WITH_THE_CORRECT_NUMBER_OF_ARGUMENTS, &[]);
            }
        }
        self.contextually_check_function_expression_or_object_literal_method(node, check_mode);
        let symbol = self.get_symbol_of_declaration(node).expect("function declaration symbol");
        self.get_type_of_symbol(&symbol)
    }

    pub fn check_function_expression_or_object_literal_method_deferred(&mut self, node: &Arc<Node>) {
        let function_flags = get_function_flags(Some(node));
        let return_type = self.get_return_type_from_annotation(node);
        self.check_all_code_paths_in_non_void_function_return_or_throw(node, Some(&return_type));
        let body = node.body();
        if let Some(body) = &body {
            if node.type_node().is_none() {
                if let Some(signature) = self.get_signature_from_declaration(node) {
                    self.get_return_type_of_signature(&signature);
                }
            }
            if tsox_frontend::ast::is_block(body) {
                self.check_source_element(body);
            } else {
                let expr_type = self.check_expression_ex(body, CheckMode::Normal);
                let return_or_promised_type = self.unwrapReturnType(&return_type, function_flags);
                self.check_return_expression(node, &return_or_promised_type, body, body, &expr_type, false);
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct PredicateSemantics(u32);

impl PredicateSemantics {
    pub(crate) const Unknown: PredicateSemantics = PredicateSemantics(0);
    pub(crate) const Always: PredicateSemantics = PredicateSemantics(1 << 0);
    pub(crate) const Never: PredicateSemantics = PredicateSemantics(1 << 1);
    pub(crate) const Sometimes: PredicateSemantics = PredicateSemantics(PredicateSemantics::Always.0 | PredicateSemantics::Never.0);

    pub(crate) fn intersects(self, other: PredicateSemantics) -> bool {
        (self.0 & other.0) != 0
    }
}

impl std::ops::BitAnd for PredicateSemantics {
    type Output = PredicateSemantics;
    fn bitand(self, rhs: PredicateSemantics) -> PredicateSemantics {
        PredicateSemantics(self.0 & rhs.0)
    }
}

impl std::ops::BitOr for PredicateSemantics {
    type Output = PredicateSemantics;
    fn bitor(self, rhs: PredicateSemantics) -> PredicateSemantics {
        PredicateSemantics(self.0 | rhs.0)
    }
}

pub(crate) fn is_default_clause(node: &Node) -> bool {
    node.kind == SyntaxKind::DefaultClause
}

pub(crate) fn is_string_literal_like(node: &Node) -> bool {
    node.kind == SyntaxKind::StringLiteral
        || matches!(
            node.kind,
            SyntaxKind::NoSubstitutionTemplateLiteral
                | SyntaxKind::TemplateHead
                | SyntaxKind::TemplateMiddle
                | SyntaxKind::TemplateTail
                | SyntaxKind::TemplateExpression
        )
}

pub(crate) fn jsnum_from_string(text: &str) -> tsox_core::jsnum::Number {
    tsox_core::jsnum::Number::from_string(text)
}

pub(crate) fn parse_pseudo_big_int(text: &str) -> tsox_core::jsnum::PseudoBigInt {
    tsox_core::jsnum::PseudoBigInt::parse(text)
}

pub(crate) fn find_ancestor_node(node: &Node, predicate: impl Fn(&Arc<Node>) -> bool) -> Option<Arc<Node>> {
    let mut current = node.parent();
    while let Some(n) = current {
        if predicate(&n) {
            return Some(n);
        }
        current = n.parent();
    }
    None
}

pub(crate) fn function_like_data_full_signature(node: &Node) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::FunctionDeclaration(d) => d.full_signature.clone(),
        NodeData::MethodDeclaration(d) => d.full_signature.clone(),
        NodeData::ConstructorDeclaration(d) => d.full_signature.clone(),
        NodeData::GetAccessorDeclaration(d) => d.full_signature.clone(),
        NodeData::SetAccessorDeclaration(d) => d.full_signature.clone(),
        NodeData::ArrowFunction(d) => d.full_signature.clone(),
        NodeData::FunctionExpression(d) => d.full_signature.clone(),
        _ => None,
    }
}

pub(crate) fn variable_declaration_exclamation_token(node: &Node) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::VariableDeclaration(d) => d.exclamation_token.clone(),
        _ => None,
    }
}

pub(crate) fn import_attributes_list(node: &Node) -> Vec<Arc<Node>> {
    match &node.data {
        NodeData::ImportAttributes(d) => d.attributes.nodes.clone(),
        _ => Vec::new(),
    }
}

pub(crate) fn import_attribute_value(node: &Arc<Node>) -> Arc<Node> {
    match &node.data {
        NodeData::ImportAttribute(d) => Arc::clone(&d.value),
        _ => Arc::clone(node),
    }
}

pub(crate) fn object_literal_properties(node: &Node) -> Vec<Arc<Node>> {
    match &node.data {
        NodeData::ObjectLiteralExpression(d) => d.properties.nodes.clone(),
        _ => Vec::new(),
    }
}

pub(crate) struct InternalSymbolName;

#[allow(dead_code)]
impl InternalSymbolName {
    pub(crate) const ImportAttributes: &'static str = "@@importAttributes";
}

pub(crate) fn get_external_module_name(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::ImportDeclaration(d) => Some(d.module_specifier.clone()),
        NodeData::ExportDeclaration(d) => d.module_specifier.clone(),
        NodeData::ImportEqualsDeclaration(d) => {
            if d.module_reference.kind == SyntaxKind::ExternalModuleReference {
                if let NodeData::ExternalModuleReference(ref_data) = &d.module_reference.data {
                    return Some(ref_data.expression.clone());
                }
            }
            None
        }
        _ => None,
    }
}
