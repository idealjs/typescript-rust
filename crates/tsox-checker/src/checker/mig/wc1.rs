#![allow(unused_imports)]

use crate::checker::checker_checker::*;
use crate::checker::mig::wc3::NodeAccessExt;
use std::sync::Arc;
use tsox_core::diagnostics::messages_generated::*;
use tsox_frontend::ast::{self, Node, Symbol, SyntaxKind};

impl Checker {
    pub fn check_arithmetic_operand_type(
        &mut self,
        operand: &Arc<Node>,
        t: &Arc<Type>,
        diagnostic: tsox_core::diagnostics::Message,
        is_await_valid: bool,
    ) -> bool { ::tsox_core::fntrace::enter("check_arithmetic_operand_type"); 
        let number_or_big_int = Arc::clone(&self.number_or_big_int_type);
        if !self.is_type_assignable_to(t, &number_or_big_int) {
            let mut maybe_missing_await = false;
            if is_await_valid {
                if let Some(awaited) = self.get_awaited_type_of_promise(t) {
                    maybe_missing_await = self.is_type_assignable_to(&awaited, &number_or_big_int);
                }
            }
            self.error_and_maybe_suggest_await_message(operand, maybe_missing_await, diagnostic, &[]);
            return false;
        }
        true
    }

    pub fn check_class_static_block_declaration(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_class_static_block_declaration"); 
        self.check_grammar_modifiers(node);
        node.for_each_child(|c| {
            self.check_source_element(c);
            true
        });
    }

    pub fn check_array_type(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_array_type"); 
        let element_type = Arc::clone(&node.as_array_type_node().element_type);
        self.check_source_element(&element_type);
    }

    pub fn check_conditional_type(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_conditional_type"); 
        node.for_each_child(|c| {
            self.check_source_element(c);
            true
        });
    }

    pub fn check_block(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_block"); 
        if node.kind == SyntaxKind::Block {
            self.check_grammar_statement_in_ambient_context(node);
        }
        if tsox_frontend::ast::mig::w7a::is_function_or_module_block(node) {
            let save_flow_analysis_disabled = self.flow_analysis_disabled;
            self.check_source_elements(tsox_frontend::ast::mig::m3c::statements(node));
            self.flow_analysis_disabled = save_flow_analysis_disabled;
        } else {
            self.check_source_elements(tsox_frontend::ast::mig::m3c::statements(node));
        }
        self.register_for_unused_identifiers_check(node);
    }

    pub fn check_break_or_continue_statement(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_break_or_continue_statement"); 
        if !self.check_grammar_statement_in_ambient_context(node) {
            self.check_grammar_break_or_continue_statement(node);
        }
    }

    pub fn check_binding_element(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_binding_element"); 
        self.check_grammar_binding_element(node);
        self.check_variable_like_declaration(node);
    }

    pub fn check_class_for_static_property_name_conflicts(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_class_for_static_property_name_conflicts"); 
        if self.compiler_options.get_use_define_for_class_fields() {
            return;
        }
        for member in tsox_frontend::ast::mig::m3b::members(node) {
            let Some(member_name_node) = member.name() else {
                continue;
            };
            if !tsox_frontend::ast::is_static(&member_name_node) {
                continue;
            }
            let (member_name, _) =
                self.get_effective_property_name_for_property_name_node(&member_name_node);
            if matches!(
                member_name.as_str(),
                "name" | "length" | "caller" | "arguments"
            ) {
                let symbol = self.get_symbol_of_declaration(node);
                let class_name = symbol
                    .as_ref()
                    .map(|symbol| self.symbol_to_string(symbol))
                    .unwrap_or_default();
                let name = member_name.clone();
                self.error_message(
                    &member_name_node,STATIC_PROPERTY_0_CONFLICTS_WITH_BUILT_IN_PROPERTY_FUNCTION_0_OF_CONSTRUCTOR_FUNCTION_1,
                    &[name, class_name],
                );
            }
        }
    }

    pub fn check_base_type_accessibility(&mut self, t: &Arc<Type>, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_base_type_accessibility"); 
        let signatures = self.get_signatures_of_type(t, SignatureKind::Construct);
        if let Some(first) = signatures.first() {
            if let Some(declaration) = &first.declaration {
                if tsox_frontend::ast::mig::m3f_2::has_modifier(declaration, ModifierFlags::Private) {
                    let type_class_declaration = t.symbol().and_then(|s| {
                        tsox_frontend::ast::mig::x4ast::get_class_like_declaration_of_symbol(s)
                    });
                    let outside_of_class = type_class_declaration
                        .as_ref()
                        .map(|c| self.is_node_within_class(node, c))
                        .unwrap_or(true);
                    if outside_of_class {
                        let name = t.symbol()
                            .map(|symbol| self.get_fully_qualified_name(symbol, None))
                            .unwrap_or_default();
                        self.error_message(
                            node,CANNOT_EXTEND_A_CLASS_0_CLASS_CONSTRUCTOR_IS_MARKED_AS_PRIVATE,
                            &[name],
                        );
                    }
                }
            }
        }
    }

    pub fn check_class_or_interface_for_duplicate_index_signatures(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_class_or_interface_for_duplicate_index_signatures"); 
        let Some(symbol) = self.get_symbol_of_declaration(node) else {
            return;
        };
        if !self
            .declared_type_links
            .get(&symbol)
            .map(|links| links.index_signatures_checked)
            .unwrap_or(false)
        {
            self.declared_type_links
                .get_or_default(&symbol)
                .index_signatures_checked = true;
            self.check_type_for_duplicate_index_signatures(node);
        }
    }

    pub fn check_assertion(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> { ::tsox_core::fntrace::enter("check_assertion"); 
        if node.kind == SyntaxKind::TypeAssertionExpression {
            if let Some(file) = self.get_source_file_of_node(node) {
                if tsox_core::tspath::file_extension_is_one_of(
                    &file.file_name,
                    &[
                        tsox_core::tspath::EXTENSION_MTS,
                        tsox_core::tspath::EXTENSION_CTS,
                    ],
                ) {
                    self.grammar_error_on_node_with_args(
                        node,
                        &THIS_SYNTAX_IS_RESERVED_IN_FILES_WITH_THE_MTS_OR_CTS_EXTENSION_USE_AN_AS_EXPRESSION_INSTEAD,
                        &[],
                    );
                }
                if self.should_check_erasable_syntax(node) {
                    let start = tsox_frontend::scanner::skip_trivia(&file.text, node.pos());
                    let end = node.expression().map(|e| e.pos()).unwrap_or(node.end());
                    let diagnostic = tsox_frontend::ast::Diagnostic::new(
                        Some(file),
                        tsox_core::core::text::TextRange::new(start, end),
                        THIS_SYNTAX_IS_NOT_ALLOWED_WHEN_ERASABLESYNTAXONLY_IS_ENABLED,
                        vec![],
                    );
                    self.diagnostics.add(diagnostic);
                }
            }
        }
        let type_node = match &node.data {
            ast::NodeData::AsExpression(d) => Arc::clone(&d.type_node),
            ast::NodeData::TypeAssertion(d) => Arc::clone(&d.type_node),
            _ => node.type_node().cloned().unwrap_or_else(|| Arc::clone(node)),
        };
        let expr_type = self.check_expression_ex(
            &node.expression().cloned().unwrap_or_else(|| Arc::clone(node)),
            check_mode,
        );
        self.check_source_element(&type_node);
        if crate::checker::utilities_get_assignment_target::is_const_type_reference(
            type_node.as_ref(),
        ) {
            let expr = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
            if !self.is_valid_const_assertion_argument(&expr) {
                self.error_message(
                    &expr,A_CONST_ASSERTION_CAN_ONLY_BE_APPLIED_TO_REFERENCES_TO_ENUM_MEMBERS_OR_STRING_NUMBER_BOOLEAN_ARRAY_OR_OBJECT_LITERALS,
                    &[],
                );
            }
            return self.get_regular_type_of_literal_type(&expr_type);
        }
        self.assertion_links
            .get_or_default(node)
            .expr_type = Some(Arc::clone(&expr_type));
        self.check_node_deferred(node);
        self.get_type_from_type_node(&type_node)
    }

    pub fn check_assertion_deferred(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_assertion_deferred"); 
        let type_node = node.type_node().cloned().unwrap_or_else(|| Arc::clone(node));
        let expr_type = self.get_regular_type_of_object_literal(&self.get_base_type_of_literal_type(
            &self
                .assertion_links
                .get(node)
                .and_then(|l| l.expr_type.clone())
                .unwrap_or_else(|| self.get_unknown_type()),
        ));
        let target_type = self.get_type_from_type_node(&type_node);
        if !self.is_error_type(&target_type) {
            let widened_type = self.get_widened_type(&expr_type);
            if !self.is_type_comparable_to(&target_type, &widened_type) {
                let err_node = if type_node
                    .flags
                    .intersects(tsox_frontend::ast::NodeFlags::Reparsed)
                {
                    Arc::clone(&type_node)
                } else {
                    Arc::clone(node)
                };
                self.check_type_comparable_to(
                    &expr_type,
                    &target_type,
                    Some(&err_node),
                    Some(&CONVERSION_OF_TYPE_0_TO_TYPE_1_MAY_BE_A_MISTAKE_BECAUSE_NEITHER_TYPE_SUFFICIENTLY_OVERLAPS_WITH_THE_OTHER_IF_THIS_WAS_INTENTIONAL_CONVERT_THE_EXPRESSION_TO_UNKNOWN_FIRST),
                );
            }
        }
    }

    pub fn check_const_enum_access(&mut self, node: &Arc<Node>, t: &Arc<Type>) { ::tsox_core::fntrace::enter("check_const_enum_access"); 
        let parent = node.parent();
        let ok = parent.as_ref().is_some_and(|parent| {
            (tsox_frontend::ast::is_property_access_expression(parent)
                && parent.expression().is_some_and(|e| Arc::ptr_eq(&e, node)))
                || (tsox_frontend::ast::is_element_access_expression(parent)
                    && parent.expression().is_some_and(|e| Arc::ptr_eq(&e, node)))
                || ((tsox_frontend::ast::is_identifier(node)
                    || tsox_frontend::ast::is_qualified_name(node))
                    && crate::checker::utilities_is_private_within_ambient::is_in_right_side_of_import_or_export_assignment(node))
                || (parent.kind == SyntaxKind::TypeQuery
                    && Arc::ptr_eq(&parent.as_type_query_node().expr_name, node))
                || tsox_frontend::ast::is_export_specifier(parent)
        });
        if !ok {
            self.error_message(
                node,X_CONST_ENUMS_CAN_ONLY_BE_USED_IN_PROPERTY_OR_INDEX_ACCESS_EXPRESSIONS_OR_THE_RIGHT_HAND_SIDE_OF_AN_IMPORT_DECLARATION_OR_EXPORT_ASSIGNMENT_OR_TYPE_QUERY,
                &[],
            );
        }
        let first_identifier = tsox_frontend::ast::mig::m3e_4::get_first_identifier(node);
        if self.compiler_options.isolated_modules.is_true()
            || (self.compiler_options.verbatim_module_syntax.is_true()
                && ok
                && self
                    .resolve_name(first_identifier.text(), node, SymbolFlags::Alias, false)
                    .is_none())
        {
            if let Some(const_enum_declaration) = t.symbol().and_then(|s| s.value_declaration.clone())
            {
                // Go: redirect == nil || !redirect.Resolved.CompilerOptions()
                //     .ShouldPreserveConstEnums()
                if const_enum_declaration
                    .flags
                    .intersects(tsox_frontend::ast::NodeFlags::Ambient)
                    && !tsox_frontend::ast::mig::m3g_3::is_valid_type_only_alias_use_site(node)
                {
                    let flag_name = self.get_isolated_modules_like_flag_name();
                    self.error_message(
                        node,CANNOT_ACCESS_AMBIENT_CONST_ENUMS_WHEN_0_IS_ENABLED,
                        &[flag_name.to_string()],
                    );
                }
            }
        }
    }
}
