use crate::binder::nameresolver::NameResolver;
use crate::binder::*;
use std::sync::Arc;
use super::r19k5_ext::{
    is_eval_or_arguments_identifier, set_export_symbol, set_local_symbol_of_exportable,
    SourceFileNodeExt, SymbolFlagsExt,
};
use super::r19k5_msg as msg;
use tsox_core::diagnostics::Message;
use tsox_frontend::ast::mig::m3b::is_locals_container;
use tsox_frontend::ast::mig::m3f_3::is_block_or_catch_scoped;
use tsox_frontend::ast::mig::m3g_2::is_part_of_parameter_declaration;
use tsox_frontend::ast::mig::m3g_3::is_variable_declaration_initialized_to_require;
use tsox_frontend::ast::mig::w7a::is_implicitly_exported_jsdoc_declaration;
use tsox_frontend::ast::*;
use tsox_frontend::ast::FlowLabel;
use crate::checker::relater_relation::error_range_for_node;
use tsox_frontend::ast::is_assignment_operator;

impl Binder {
    pub(crate) fn bind_source_file_as_external_module(&mut self) {
        let file = self.current_source_file.clone().unwrap();
        let name = format!(
            "\"{}\"",
            tsox_core::tspath::remove_file_extension(&file.file_name)
        );
        self.bind_anonymous_declaration(&file.node, SymbolFlags::ValueModule, &name);
    }

    pub(crate) fn bind_source_file_if_external_module(&mut self) {
        let file = self.current_source_file.clone().unwrap();
        let file_node = file.node.clone();
        self.set_export_context_flag(&file_node);
        if is_external_or_common_js_module(&file) {
            self.bind_source_file_as_external_module();
        } else if is_json_source_file(&file) {
            self.bind_source_file_as_external_module();
            let original_symbol = self.symbol_map.symbol_of(&file_node).cloned();
            if let Some(module_symbol) = &original_symbol {
                self.declare_symbol_into(
                    &file_node,
                    SymbolFlags::Property,
                    SymbolFlags::All,
                    DeclareTarget::Exports(Arc::clone(module_symbol)),
                );
            }
            if let Some(original) = original_symbol {
                self.symbol_map.set_symbol(&file_node, original);
            }
        }
    }

    pub(crate) fn bind_variable_declaration_flow(&mut self, node: &Arc<Node>) {
        self.bind_each_child(node);
        let grandparent_kind = node.parent().and_then(|p| p.parent()).map(|g| g.kind);
        let is_for_in_or_of = matches!(
            grandparent_kind,
            Some(SyntaxKind::ForInStatement) | Some(SyntaxKind::ForOfStatement)
        );
        if node.initializer().is_some() || is_for_in_or_of {
            self.bind_initialized_variable_flow(node);
        }
    }

    pub(crate) fn bind_variable_declaration_or_binding_element(&mut self, node: &Arc<Node>) {
        self.check_strict_mode_eval_or_arguments(node, node.name().as_deref());
        if let Some(name) = node.name()
            && !is_binding_pattern(&name)
        {
            if is_variable_declaration_initialized_to_require(node) {
                self.declare_symbol_and_add_to_symbol_table(
                    node,
                    SymbolFlags::Alias,
                    SymbolFlags::AliasExcludes,
                );
            } else if is_block_or_catch_scoped(node) {
                self.bind_block_scoped_declaration(
                    node,
                    SymbolFlags::BlockScopedVariable,
                    SymbolFlags::BlockScopedVariableExcludes,
                );
            } else if is_part_of_parameter_declaration(node) {
                self.declare_symbol_and_add_to_symbol_table(
                    node,
                    SymbolFlags::FunctionScopedVariable,
                    SymbolFlags::ParameterExcludes,
                );
            } else {
                self.declare_symbol_and_add_to_symbol_table(
                    node,
                    SymbolFlags::FunctionScopedVariable,
                    SymbolFlags::FunctionScopedVariableExcludes,
                );
            }
        }
    }

    pub(crate) fn check_private_identifier(&mut self, node: &Arc<Node>) {
        if node.text() == "#constructor"
            && self
                .current_source_file
                .as_ref()
                .is_some_and(|f| f.parse_error_spans.is_empty())
        {
            self.error_on_node(node, &msg::X_constructor_is_a_reserved_word, &[node.text().to_string()]);
        }
    }

    pub(crate) fn check_strict_mode_binary_expression(&mut self, node: &Arc<Node>) {
        if let NodeData::BinaryExpression(expr) = &node.data
            && is_left_hand_side_expression(&expr.left)
            && is_assignment_operator(expr.operator_token.kind)
        {
            self.check_strict_mode_eval_or_arguments(node, Some(&expr.left));
        }
    }

    pub(crate) fn check_strict_mode_catch_clause(&mut self, node: &Arc<Node>) {
        if let NodeData::CatchClause(clause) = &node.data
            && let Some(var_decl) = &clause.variable_declaration
        {
            self.check_strict_mode_eval_or_arguments(node, var_decl.name());
        }
    }

    pub(crate) fn check_strict_mode_delete_expression(&mut self, node: &Arc<Node>) {
        if let NodeData::DeleteExpression(expr) = &node.data
            && expr.expression.kind == SyntaxKind::Identifier
        {
            self.error_on_node(
                &expr.expression,
                &msg::X_delete_cannot_be_called_on_an_identifier_in_strict_mode,
                &[],
            );
        }
    }

    pub(crate) fn check_strict_mode_eval_or_arguments(
        &mut self,
        context_node: &Arc<Node>,
        name: Option<&Arc<Node>>,
    ) {
        if let Some(name) = name
            && is_eval_or_arguments_identifier(name)
        {
            let message = self.get_strict_mode_eval_or_arguments_message(context_node);
            self.error_on_node(name, message, &[name.text().to_string()]);
        }
    }

    pub(crate) fn check_strict_mode_function_name(&mut self, node: &Arc<Node>) {
        if !node.flags.contains(NodeFlags::Ambient) {
            self.check_strict_mode_eval_or_arguments(node, node.name().as_deref());
        }
    }

    pub(crate) fn check_strict_mode_labeled_statement(&mut self, node: &Arc<Node>) {
        if let NodeData::LabeledStatement(data) = &node.data
            && (is_declaration_statement(&data.statement) || is_variable_statement(&data.statement))
        {
            self.error_on_first_token(&data.label, &msg::A_label_is_not_allowed_here, &[]);
        }
    }

    pub(crate) fn check_strict_mode_postfix_unary_expression(&mut self, node: &Arc<Node>) {
        if let NodeData::PostfixUnaryExpression(expr) = &node.data {
            self.check_strict_mode_eval_or_arguments(node, Some(&expr.operand));
        }
    }

    pub(crate) fn check_strict_mode_prefix_unary_expression(&mut self, node: &Arc<Node>) {
        if let NodeData::PrefixUnaryExpression(expr) = &node.data
            && matches!(
                expr.operator,
                SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
            )
        {
            self.check_strict_mode_eval_or_arguments(node, Some(&expr.operand));
        }
    }

    pub(crate) fn check_strict_mode_with_statement(&mut self, node: &Arc<Node>) {
        self.error_on_first_token(
            node,
            &msg::X_with_statements_are_not_allowed_in_strict_mode,
            &[],
        );
    }

    pub(crate) fn create_branch_label(&mut self) -> FlowLabel {
        FlowLabel::new(FlowFlags::BRANCH_LABEL)
    }

    pub(crate) fn create_loop_label(&mut self) -> FlowLabel {
        FlowLabel::new(FlowFlags::LOOP_LABEL)
    }

    pub(crate) fn create_diagnostic_for_node(
        &self,
        node: &Arc<Node>,
        message: &'static Message,
        args: &[String],
    ) -> Diagnostic {
        let file = self.current_source_file.clone();
        let span = error_range_for_node(node);
        Diagnostic::new(file, span, *message, args.to_vec())
    }

    pub(crate) fn declare_class_member(
        &mut self,
        node: &Arc<Node>,
        symbol_flags: SymbolFlags,
        symbol_excludes: SymbolFlags,
    ) -> Arc<Symbol> {
        let container = self.container.clone().unwrap();
        let class_symbol = self.symbol_map.symbol_of(&container).cloned().unwrap();
        if is_static(&container) {
            self.declare_symbol_into(
                node,
                symbol_flags,
                symbol_excludes,
                DeclareTarget::Exports(Arc::clone(&class_symbol)),
            )
        } else {
            self.declare_symbol_into(
                node,
                symbol_flags,
                symbol_excludes,
                DeclareTarget::Members(Arc::clone(&class_symbol)),
            )
        }
    }

    pub(crate) fn declare_module_member(
        &mut self,
        node: &Arc<Node>,
        symbol_flags: SymbolFlags,
        symbol_excludes: SymbolFlags,
    ) -> Arc<Symbol> {
        let container = self.container.clone().unwrap();
        let has_export_modifier = get_combined_modifier_flags(node)
            .contains(ModifierFlags::Export)
            || is_implicitly_exported_jsdoc_declaration(node);
        if symbol_flags.contains(SymbolFlags::Alias) {
            if node.kind == SyntaxKind::ExportSpecifier
                || (node.kind == SyntaxKind::ImportEqualsDeclaration && has_export_modifier)
            {
                let module_symbol = self.symbol_map.symbol_of(&container).cloned().unwrap();
                return self.declare_symbol_into(
                    node,
                    symbol_flags,
                    symbol_excludes,
                    DeclareTarget::Exports(module_symbol),
                );
            }
            return self.declare_symbol_into(
                node,
                symbol_flags,
                symbol_excludes,
                DeclareTarget::Locals(container),
            );
        }
        if !is_ambient_module(node)
            && (has_export_modifier || container.flags.contains(NodeFlags::ExportContext))
        {
            if !is_locals_container(&container)
                || (has_syntactic_modifier(node, ModifierFlags::Default)
                    && self.get_declaration_name(node) == "___missing")
            {
                let module_symbol = self.symbol_map.symbol_of(&container).cloned().unwrap();
                return self.declare_symbol_into(
                    node,
                    symbol_flags,
                    symbol_excludes,
                    DeclareTarget::Exports(module_symbol),
                );
            }
            let export_kind = if symbol_flags.contains(SymbolFlags::Value) {
                SymbolFlags::ExportValue
            } else {
                SymbolFlags::empty()
            };
            let local = self.declare_symbol_into(
                node,
                export_kind,
                symbol_excludes,
                DeclareTarget::Locals(Arc::clone(&container)),
            );
            let module_symbol = self.symbol_map.symbol_of(&container).cloned().unwrap();
            let export_symbol = self.declare_symbol_into(
                node,
                symbol_flags,
                symbol_excludes,
                DeclareTarget::Exports(module_symbol),
            );
            set_export_symbol(&local, export_symbol);
            set_local_symbol_of_exportable(node, local.clone());
            return local;
        }
        self.declare_symbol_into(
            node,
            symbol_flags,
            symbol_excludes,
            DeclareTarget::Locals(container),
        )
    }
}
