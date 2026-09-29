#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_frontend::ast::{self, Node, NodeData, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::astnav;

use crate::ls::lsutil_user_preferences::QuotePreference;
use crate::ls::lsutil_user_preferences_enums::IncludeInlayParameterNameHints;
use crate::ls::lsutil_user_preferences_preferences::InlayHintsPreferences;

#[derive(Debug, Clone, Default)]
pub struct InlayHintLabelPart {
    pub value: String,
    pub location: Option<crate::lsp::lsproto_lsp::Location>,
}

#[derive(Debug, Clone, Default)]
pub struct StringOrInlayHintLabelParts {
    pub string: Option<String>,
    pub inlay_hint_label_parts: Option<Vec<InlayHintLabelPart>>,
}

#[derive(Debug, Clone)]
pub struct M5uInlayHint {
    pub position: crate::lsp::lsproto_lsp::Position,
    pub label: StringOrInlayHintLabelParts,
    pub kind: Option<i32>,
    pub padding_left: Option<bool>,
    pub padding_right: Option<bool>,
}

pub struct ParameterInfo {
    pub parameter: Arc<Node>,
    pub name: String,
    pub is_rest_parameter: bool,
}

pub struct InlayHintState<'a> {
    pub span: tsox_core::core::text::TextRange,
    pub preferences: InlayHintsPreferences,
    pub quote_preference: QuotePreference,
    pub file: Arc<SourceFile>,
    pub checker: &'a mut Checker,
    pub converters: &'a crate::ls::lsconv_converters::Converters,
    pub result: Vec<M5uInlayHint>,
}

pub fn should_show_parameter_name_hints(preferences: &InlayHintsPreferences) -> bool {
    preferences.include_inlay_parameter_name_hints == IncludeInlayParameterNameHints::Literals
        || preferences.include_inlay_parameter_name_hints == IncludeInlayParameterNameHints::All
}

pub fn should_show_literal_parameter_name_hints_only(
    preferences: &InlayHintsPreferences,
) -> bool {
    preferences.include_inlay_parameter_name_hints == IncludeInlayParameterNameHints::Literals
}

pub fn is_signature_supporting_return_annotation(node: &Arc<Node>) -> bool {
    ast::is_arrow_function(node)
        || ast::is_function_expression(node)
        || ast::is_function_declaration(node)
        || ast::is_method_declaration(node)
        || ast::is_get_accessor_declaration(node)
}

pub fn is_hintable_declaration(node: &Arc<Node>) -> bool {
    if (ast::mig::m3g_2::is_part_of_parameter_declaration(node)
        || ast::is_variable_declaration(node) && ast::mig::m3g_3::is_var_const(node))
        && node.initializer().is_some()
    {
        let initializer = ast::skip_parentheses(&node.initializer().unwrap());
        return !(is_hintable_literal(&initializer)
            || ast::is_new_expression(&initializer)
            || ast::is_object_literal_expression(&initializer)
            || ast::is_assertion_expression(&initializer));
    }
    true
}

pub fn is_hintable_literal(node: &Arc<Node>) -> bool {
    match node.kind {
        SyntaxKind::PrefixUnaryExpression => {
            let operand = prefix_unary_operand(node);
            operand.map_or(false, |operand| {
                ast::is_literal_expression(&operand)
                    || ast::is_identifier(&operand)
                        && crate::ls::mig::m5v_7::is_infinity_or_nan_string(operand.text())
            })
        }
        SyntaxKind::TrueKeyword
        | SyntaxKind::FalseKeyword
        | SyntaxKind::NullKeyword
        | SyntaxKind::NoSubstitutionTemplateLiteral
        | SyntaxKind::TemplateExpression => true,
        SyntaxKind::Identifier => {
            let name = node.text();
            name == "undefined" || crate::ls::mig::m5v_7::is_infinity_or_nan_string(name)
        }
        _ => ast::is_literal_expression(node),
    }
}

pub fn is_module_reference_type(t: &tsox_checker::checker::Type) -> bool {
    t.symbol()
        .map_or(false, |symbol| symbol.flags & ast::SymbolFlags::MODULE != ast::SymbolFlags::None)
}

pub fn get_parameter_declaration_identifier(symbol: &Arc<Symbol>) -> Option<Arc<Node>> {
    symbol
        .value_declaration
        .as_ref()
        .filter(|decl| ast::is_parameter_declaration(decl))
        .and_then(|decl| decl.name())
        .filter(|name| ast::is_identifier(name))
        .map(Arc::clone)
}

pub fn identifier_or_access_expression_postfix_matches_parameter_name(
    expr: &Arc<Node>,
    parameter_name: &str,
) -> bool {
    if ast::is_identifier(expr) {
        return expr.text() == parameter_name;
    }
    if ast::is_property_access_expression(expr) {
        return expr
            .name()
            .map_or(false, |name| name.text() == parameter_name);
    }
    false
}

impl<'a> InlayHintState<'a> {
    pub fn visit(&mut self, node: Option<&Arc<Node>>) -> bool {
        let Some(node) = node else {
            return false;
        };
        if node.end() - node.pos() == 0 || node.flags.contains(ast::NodeFlags::Reparsed) {
            return false;
        }

        match node.kind {
            SyntaxKind::ModuleDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::FunctionExpression
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::ArrowFunction => {}
            _ => {}
        }

        if !self.span.intersects(&node.loc) {
            return false;
        }

        if ast::is_type_node(node) && !ast::is_expression_with_type_arguments(node) {
            return false;
        }

        if self.preferences.include_inlay_variable_type_hints.is_true()
            && ast::is_variable_declaration(node)
        {
            self.visit_variable_like_declaration(node);
        } else if self
            .preferences
            .include_inlay_property_declaration_type_hints
            .is_true()
            && ast::is_property_declaration(node)
        {
            self.visit_variable_like_declaration(node);
        } else if self
            .preferences
            .include_inlay_enum_member_value_hints
            .is_true()
            && ast::is_enum_member(node)
        {
            self.visit_enum_member(node);
        } else if should_show_parameter_name_hints(&self.preferences)
            && (ast::is_call_expression(node) || ast::is_new_expression(node))
        {
            self.visit_call_or_new_expression(node);
        } else {
            if self
                .preferences
                .include_inlay_function_parameter_type_hints
                .is_true()
                && ast::is_function_like_declaration(node)
                && ast::mig::m3f_2::has_context_sensitive_parameters(node)
            {
                self.visit_function_like_for_parameter_type(node);
            }
            if self
                .preferences
                .include_inlay_function_like_return_type_hints
                .is_true()
                && is_signature_supporting_return_annotation(node)
            {
                self.visit_function_declaration_like_for_return_type(node);
            }
        }
        tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
            self.visit(Some(child))
        })
    }

    pub fn visit_function_declaration_like_for_return_type(&mut self, decl: &Arc<Node>) {
        if ast::is_arrow_function(decl)
            && astnav::find_child_of_kind(decl, SyntaxKind::OpenParenToken).is_none()
        {
            return;
        }

        if decl.type_node().is_some() || decl.body().is_none() {
            return;
        }

        let Some(signature) = self.checker.get_signature_from_declaration(decl) else {
            return;
        };

        let type_predicate = self.checker.get_type_predicate_of_signature(&signature);

        if let Some(type_predicate) = type_predicate {
            if type_predicate.t.is_some() {
                let hint_parts = self.type_predicate_to_inlay_hint_parts(type_predicate);
                let pos = self.get_type_annotation_position(decl);
                self.add_type_hints(hint_parts, pos);
                return;
            }
        }

        let Some(return_type) = self.checker.get_return_type_of_signature(&signature) else {
            return;
        };
        if is_module_reference_type(&return_type) {
            return;
        }

        let hint_parts = self.type_to_inlay_hint_parts(&return_type);
        let pos = self.get_type_annotation_position(decl);
        self.add_type_hints(hint_parts, pos);
    }

    pub fn visit_call_or_new_expression(&mut self, expr: &Arc<Node>) {
        let args = node_arguments(expr);
        if args.is_empty() {
            return;
        }

        let Some(signature) = self.checker.get_resolved_signature(expr) else {
            return;
        };

        let mut signature_param_pos: usize = 0;
        for original_arg in &args {
            let arg = ast::skip_parentheses(original_arg);
            if should_show_literal_parameter_name_hints_only(&self.preferences)
                && !is_hintable_literal(&arg)
            {
                signature_param_pos += 1;
                continue;
            }

            let mut spread_args: usize = 0;
            if ast::is_spread_element(&arg) {
                let spread_type = self.checker.get_type_at_location(&arg.expression().unwrap());
                if self.checker.is_tuple_type(&spread_type) {
                    let Some(tuple_type) = spread_type.target().and_then(|t| t.as_tuple_type())
                    else {
                        continue;
                    };
                    let element_flags = tuple_type.element_flags();
                    let fixed_length = tuple_type.fixed_length;
                    if fixed_length == 0 {
                        continue;
                    }
                    let first_optional_index = element_flags
                        .iter()
                        .position(|f| !f.contains(checker_element_flags_required()));
                    let required_args = match first_optional_index {
                        Some(index) => index,
                        None => fixed_length,
                    };
                    if required_args > 0 {
                        spread_args = required_args;
                    }
                }
            }

            let identifier_info =
                self.get_parameter_identifier_info_at_position(&signature, signature_param_pos);
            signature_param_pos += if spread_args > 0 { spread_args } else { 1 };
            let Some(identifier_info) = identifier_info else {
                return;
            };

            let parameter_name = identifier_info.name.clone();
            let is_first_variadic_argument = identifier_info.is_rest_parameter;
            let parameter_name_not_same_as_argument = self
                .preferences
                .include_inlay_parameter_name_hints_when_argument_matches_name
                .is_true()
                || !identifier_or_access_expression_postfix_matches_parameter_name(
                    &arg,
                    &parameter_name,
                );
            if !parameter_name_not_same_as_argument && !is_first_variadic_argument {
                continue;
            }

            if self.leading_comments_contains_parameter_name(&arg, &parameter_name) {
                continue;
            }

            let position =
                astnav::get_start_of_node(original_arg, &self.file, false);
            self.add_parameter_hints(
                &parameter_name,
                &identifier_info.parameter,
                position,
                is_first_variadic_argument,
            );
        }
    }

    pub fn visit_enum_member(&mut self, member: &Arc<Node>) {
        if member.initializer().is_some() {
            return;
        }

        if let Some(enum_value) = self.checker.get_constant_value(member) {
            let text = evaluator_any_to_string(&enum_value);
            self.add_enum_member_value_hints(text, member.end());
        }
    }

    pub fn visit_variable_like_declaration(&mut self, decl: &Arc<Node>) {
        let name_is_binding_pattern = decl.name().map_or(false, |n| ast::is_binding_pattern(&n));
        if (decl.initializer().is_none()
            && !(ast::is_property_declaration(decl)
                && self
                    .checker
                    .get_type_at_location(decl)
                    .flags
                    .contains(tsox_checker::checker::TypeFlags::Any))
            || name_is_binding_pattern
            || (ast::is_variable_declaration(decl) && !is_hintable_declaration(decl)))
        {
            return;
        }

        if decl.type_node().is_some() {
            return;
        }

        let declaration_type = self.checker.get_type_at_location(decl);
        if is_module_reference_type(&declaration_type) {
            return;
        }

        let hint_parts = self.type_to_inlay_hint_parts(&declaration_type);
        let hint_text = match &hint_parts {
            StringOrInlayHintLabelParts { string: Some(s), .. } => s.clone(),
            StringOrInlayHintLabelParts {
                inlay_hint_label_parts: Some(parts),
                ..
            } => parts.iter().map(|p| p.value.clone()).collect::<String>(),
            _ => String::new(),
        };
        let name_matches_hint = decl.name().map_or(false, |name| {
            equate_string_case_insensitive(name.text(), &hint_text)
        });
        let name_is_computed = decl
            .name()
            .map_or(false, |name| ast::is_computed_property_name(&name));
        if !self
            .preferences
            .include_inlay_variable_type_hints_when_type_matches_name
            .is_true()
            && !name_is_computed
            && name_matches_hint
        {
            return;
        }
        if let Some(name) = decl.name() {
            self.add_type_hints(hint_parts, name.end());
        }
    }

    pub fn visit_function_like_for_parameter_type(&mut self, node: &Arc<Node>) {
        let Some(signature) = self.checker.get_signature_from_declaration(node) else {
            return;
        };

        let mut pos: usize = 0;
        for param in node_parameters(node) {
            if is_hintable_declaration(&param) {
                let symbol: Option<Arc<Symbol>> = if ast::mig::m3g_2::is_this_parameter(&param) {
                    signature.this_parameter().cloned()
                } else {
                    signature.parameters().get(pos).cloned()
                };
                if let Some(symbol) = symbol {
                    self.add_parameter_type_hint(&param, &symbol);
                }
            }
            if ast::mig::m3g_2::is_this_parameter(&param) {
                continue;
            }
            pos += 1;
        }
    }

    pub fn add_parameter_type_hint(&mut self, node: &Arc<Node>, symbol: &Arc<Symbol>) {
        if node.type_node().is_some() {
            return;
        }
        let Some(type_hints) = self.get_parameter_declaration_type_hints(symbol) else {
            return;
        };
        let pos = match node_question_token_of(node) {
            Some(question_token) => question_token.end(),
            None => node.name().map_or(node.end(), |name| name.end()),
        };
        self.add_type_hints(type_hints, pos);
    }

    pub fn get_parameter_declaration_type_hints(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<StringOrInlayHintLabelParts> {
        let value_declaration = symbol.value_declaration.as_ref()?;
        if !ast::is_parameter_declaration(value_declaration) {
            return None;
        }

        let signature_param_type = self
            .checker
            .get_type_of_symbol_at_location(symbol, value_declaration);
        if is_module_reference_type(&signature_param_type) {
            return None;
        }

        Some(self.type_to_inlay_hint_parts(&signature_param_type))
    }

    pub fn type_to_inlay_hint_parts(
        &mut self,
        t: &Arc<tsox_checker::checker::Type>,
    ) -> StringOrInlayHintLabelParts {
        let flags = node_builder_flags_ignore_errors()
            | node_builder_flags_allow_unique_es_symbol_type()
            | node_builder_flags_use_alias_defined_outside_current_scope();
        let (type_node, mut id_to_symbol) = {
            let mut node_builder = tsox_checker::checker::mig::m2f::new_node_builder_ex(
                self.checker,
                tsox_checker::checker::mig::m2f::new_emit_context(),
                HashMap::new(),
            );
            let type_node = node_builder
                .type_to_type_node_ex(
                    t,
                    None,
                    flags,
                    tsox_checker::checker::symboltracker::NodeBuilderInternalFlags::default(),
                )
                .expect("type node");
            let id_to_symbol = std::mem::take(&mut node_builder.impl_.id_to_symbol);
            (type_node, id_to_symbol)
        };
        StringOrInlayHintLabelParts {
            inlay_hint_label_parts: Some(self.get_inlay_hint_label_parts(&type_node, &mut id_to_symbol)),
            string: None,
        }
    }

    pub fn type_predicate_to_inlay_hint_parts(
        &mut self,
        type_predicate: &tsox_checker::checker::TypePredicate,
    ) -> StringOrInlayHintLabelParts {
        let flags = node_builder_flags_ignore_errors()
            | node_builder_flags_allow_unique_es_symbol_type()
            | node_builder_flags_use_alias_defined_outside_current_scope();
        let (type_node, mut id_to_symbol) = {
            let mut node_builder = tsox_checker::checker::mig::m2f::new_node_builder_ex(
                self.checker,
                tsox_checker::checker::mig::m2f::new_emit_context(),
                HashMap::new(),
            );
            let type_node = node_builder
                .type_predicate_to_type_predicate_node_ex(
                    type_predicate,
                    None,
                    flags,
                    tsox_checker::checker::symboltracker::NodeBuilderInternalFlags::default(),
                )
                .expect("type predicate node");
            let id_to_symbol = std::mem::take(&mut node_builder.impl_.id_to_symbol);
            (type_node, id_to_symbol)
        };
        StringOrInlayHintLabelParts {
            inlay_hint_label_parts: Some(self.get_inlay_hint_label_parts(&type_node, &mut id_to_symbol)),
            string: None,
        }
    }

    pub fn add_type_hints(&mut self, mut hint: StringOrInlayHintLabelParts, position: usize) {
        let script = crate::mig::m5u_conv::SourceFileScriptView {
            file: Arc::clone(&self.file),
        };
        let (lsp_position, fidelity) = self.converters.to_lsp_position_for_feature(
            &script,
            position,
            spanmap_feature_inlay_hints(),
        );
        if fidelity_is_none(&fidelity) {
            return;
        }
        if let Some(s) = hint.string.as_mut() {
            *s = format!(": {s}");
        } else if let Some(parts) = hint.inlay_hint_label_parts.as_mut() {
            parts.insert(0, InlayHintLabelPart { value: ": ".to_string(), location: None });
        }
        self.result.push(M5uInlayHint {
            position: lsp_position,
            label: hint,
            kind: Some(INLAY_HINT_KIND_TYPE),
            padding_left: Some(true),
            padding_right: None,
        });
    }

    pub fn add_enum_member_value_hints(&mut self, text: String, position: usize) {
        let script = crate::mig::m5u_conv::SourceFileScriptView {
            file: Arc::clone(&self.file),
        };
        let (lsp_position, fidelity) = self.converters.to_lsp_position_for_feature(
            &script,
            position,
            spanmap_feature_inlay_hints(),
        );
        if fidelity_is_none(&fidelity) {
            return;
        }
        self.result.push(M5uInlayHint {
            position: lsp_position,
            label: StringOrInlayHintLabelParts {
                string: Some(format!("= {text}")),
                inlay_hint_label_parts: None,
            },
            kind: None,
            padding_left: Some(true),
            padding_right: None,
        });
    }

    pub fn add_parameter_hints(
        &mut self,
        text: &str,
        parameter: &Arc<Node>,
        position: usize,
        is_first_variadic_argument: bool,
    ) {
        let script = crate::mig::m5u_conv::SourceFileScriptView {
            file: Arc::clone(&self.file),
        };
        let (lsp_position, fidelity) = self.converters.to_lsp_position_for_feature(
            &script,
            position,
            spanmap_feature_inlay_hints(),
        );
        if fidelity_is_none(&fidelity) {
            return;
        }
        let hint_text = format!("{}{}", if is_first_variadic_argument { "..." } else { "" }, text);
        let display_parts = vec![
            self.get_node_display_part(&hint_text, parameter),
            InlayHintLabelPart { value: ":".to_string(), location: None },
        ];
        self.result.push(M5uInlayHint {
            position: lsp_position,
            label: StringOrInlayHintLabelParts {
                string: None,
                inlay_hint_label_parts: Some(display_parts),
            },
            kind: Some(INLAY_HINT_KIND_PARAMETER),
            padding_left: None,
            padding_right: Some(true),
        });
    }

    pub fn get_inlay_hint_label_parts(
        &self,
        node: &Arc<Node>,
        id_to_symbol: &mut HashMap<u64, Arc<Symbol>>,
    ) -> Vec<InlayHintLabelPart> {
        let mut parts: Vec<InlayHintLabelPart> = Vec::new();

        fn push_part(parts: &mut Vec<InlayHintLabelPart>, value: &str) {
            parts.push(InlayHintLabelPart { value: value.to_string(), location: None });
        }

        fn visit_display_part_list(
            state: &InlayHintState,
            parts: &mut Vec<InlayHintLabelPart>,
            nodes: &[Arc<Node>],
            separator: &str,
            id_to_symbol: &mut HashMap<u64, Arc<Symbol>>,
        ) {
            for (i, n) in nodes.iter().enumerate() {
                if i > 0 {
                    push_part(parts, separator);
                }
                visit_for_display_parts(state, parts, n, id_to_symbol);
            }
        }

        fn visit_parameters_and_type_parameters(
            state: &InlayHintState,
            parts: &mut Vec<InlayHintLabelPart>,
            node: &Arc<Node>,
            id_to_symbol: &mut HashMap<u64, Arc<Symbol>>,
        ) {
            let type_parameters = node_type_parameters(node);
            if !type_parameters.is_empty() {
                push_part(parts, "<");
                visit_display_part_list(state, parts, &type_parameters, ", ", id_to_symbol);
                push_part(parts, ">");
            }
            push_part(parts, "(");
            visit_display_part_list(state, parts, &node_parameters(node), ", ", id_to_symbol);
            push_part(parts, ")");
        }

        fn visit_for_display_parts(
            state: &InlayHintState,
            parts: &mut Vec<InlayHintLabelPart>,
            node: &Arc<Node>,
            id_to_symbol: &mut HashMap<u64, Arc<Symbol>>,
        ) {
            let token_string = tsox_frontend::scanner::token_to_string(node.kind);
            if !token_string.is_empty() {
                push_part(parts, token_string);
                return;
            }

            if ast::is_literal_expression(node) {
                push_part(parts, &state.get_literal_text(node));
                return;
            }

            match node.kind {
                SyntaxKind::Identifier => {
                    let identifier_text = node.text();
                    let mut name: Option<Arc<Node>> = None;
                    if let Some(symbol) = id_to_symbol.get(&node.id()) {
                        if !symbol.declarations.is_empty() {
                            name = ast::get_name_of_declaration(&symbol.declarations[0]);
                        }
                    }
                    if let Some(name) = name {
                        parts.push(state.get_node_display_part(identifier_text, &name));
                    } else {
                        push_part(parts, identifier_text);
                    }
                }
                SyntaxKind::QualifiedName => {
                    if let Some(left) = qualified_name_left(node) {
                        visit_for_display_parts(state, parts, &left, id_to_symbol);
                    }
                    push_part(parts, ".");
                    if let Some(right) = qualified_name_right(node) {
                        visit_for_display_parts(state, parts, &right, id_to_symbol);
                    }
                }
                SyntaxKind::TypePredicate => {
                    if type_predicate_asserts_modifier(node).is_some() {
                        push_part(parts, "asserts ");
                    }
                    if let Some(parameter_name) = type_predicate_parameter_name(node) {
                        visit_for_display_parts(state, parts, &parameter_name, id_to_symbol);
                    }
                    if let Some(t) = node.type_node() {
                        push_part(parts, " is ");
                        visit_for_display_parts(state, parts, &t, id_to_symbol);
                    }
                }
                SyntaxKind::TypeReference => {
                    if let Some(type_name) = type_reference_type_name(node) {
                        visit_for_display_parts(state, parts, &type_name, id_to_symbol);
                    }
                    let type_arguments = node_type_arguments(node);
                    if !type_arguments.is_empty() {
                        push_part(parts, "<");
                        visit_display_part_list(state, parts, &type_arguments, ",", id_to_symbol);
                        push_part(parts, ">");
                    }
                }
                SyntaxKind::TypeParameter => {
                    let modifiers: Vec<Arc<Node>> = node
                        .modifier_nodes()
                        .iter()
                        .cloned()
                        .collect();
                    if !modifiers.is_empty() {
                        visit_display_part_list(state, parts, &modifiers, "", id_to_symbol);
                    }
                    if let Some(name) = node.name() {
                        visit_for_display_parts(state, parts, name, id_to_symbol);
                    }
                    if let Some(constraint) = type_parameter_constraint(node) {
                        push_part(parts, " extends ");
                        visit_for_display_parts(state, parts, &constraint, id_to_symbol);
                    }
                    if let Some(default_type) = type_parameter_default_type(node) {
                        push_part(parts, " = ");
                        visit_for_display_parts(state, parts, &default_type, id_to_symbol);
                    }
                }
                SyntaxKind::Parameter => {
                    let modifiers: Vec<Arc<Node>> = node.modifier_nodes().iter().cloned().collect();
                    if !modifiers.is_empty() {
                        visit_display_part_list(state, parts, &modifiers, " ", id_to_symbol);
                    }
                    if parameter_dot_dot_dot_token(node).is_some() {
                        push_part(parts, "...");
                    }
                    if let Some(name) = node.name() {
                        visit_for_display_parts(state, parts, name, id_to_symbol);
                    }
                    if node_question_token_of(node).is_some() {
                        push_part(parts, "?");
                    }
                    if let Some(t) = node.type_node() {
                        push_part(parts, ": ");
                        visit_for_display_parts(state, parts, &t, id_to_symbol);
                    }
                }
                SyntaxKind::ConstructorType => {
                    push_part(parts, "new ");
                    visit_parameters_and_type_parameters(state, parts, node, id_to_symbol);
                    push_part(parts, " => ");
                    if let Some(t) = node.type_node() {
                        visit_for_display_parts(state, parts, &t, id_to_symbol);
                    }
                }
                SyntaxKind::TypeQuery => {
                    push_part(parts, "typeof ");
                    if let Some(expr_name) = type_query_expr_name(node) {
                        visit_for_display_parts(state, parts, &expr_name, id_to_symbol);
                    }
                    let type_arguments = node_type_arguments(node);
                    if !type_arguments.is_empty() {
                        push_part(parts, "<");
                        visit_display_part_list(state, parts, &type_arguments, ", ", id_to_symbol);
                        push_part(parts, ">");
                    }
                }
                SyntaxKind::TypeLiteral => {
                    push_part(parts, "{");
                    let members = node_members(node);
                    if !members.is_empty() {
                        push_part(parts, " ");
                        visit_display_part_list(state, parts, &members, "; ", id_to_symbol);
                        push_part(parts, " ");
                    }
                    push_part(parts, "}");
                }
                SyntaxKind::ArrayType => {
                    if let Some(element_type) = array_type_element_type(node) {
                        visit_for_display_parts(state, parts, &element_type, id_to_symbol);
                    }
                    push_part(parts, "[]");
                }
                SyntaxKind::TupleType => {
                    push_part(parts, "[");
                    visit_display_part_list(state, parts, &node_elements(node), ", ", id_to_symbol);
                    push_part(parts, "]");
                }
                SyntaxKind::NamedTupleMember => {
                    if named_tuple_member_dot_dot_dot_token(node).is_some() {
                        push_part(parts, "...");
                    }
                    if let Some(name) = node.name() {
                        visit_for_display_parts(state, parts, name, id_to_symbol);
                    }
                    if node_question_token_of(node).is_some() {
                        push_part(parts, "?");
                    }
                    push_part(parts, ": ");
                    if let Some(t) = node.type_node() {
                        visit_for_display_parts(state, parts, &t, id_to_symbol);
                    }
                }
                SyntaxKind::OptionalType => {
                    if let Some(t) = node.type_node() {
                        visit_for_display_parts(state, parts, &t, id_to_symbol);
                    }
                    push_part(parts, "?");
                }
                SyntaxKind::RestType => {
                    push_part(parts, "...");
                    if let Some(t) = node.type_node() {
                        visit_for_display_parts(state, parts, &t, id_to_symbol);
                    }
                }
                SyntaxKind::UnionType => {
                    let types = union_or_intersection_type_nodes(node);
                    if !types.is_empty() {
                        visit_display_part_list(state, parts, &types, " | ", id_to_symbol);
                    }
                }
                SyntaxKind::IntersectionType => {
                    let types = union_or_intersection_type_nodes(node);
                    if !types.is_empty() {
                        visit_display_part_list(state, parts, &types, " & ", id_to_symbol);
                    }
                }
                SyntaxKind::ConditionalType => {
                    if let Some(check_type) = conditional_type_check_type(node) {
                        visit_for_display_parts(state, parts, &check_type, id_to_symbol);
                    }
                    push_part(parts, " extends ");
                    if let Some(extends_type) = conditional_type_extends_type(node) {
                        visit_for_display_parts(state, parts, &extends_type, id_to_symbol);
                    }
                    push_part(parts, " ? ");
                    if let Some(true_type) = conditional_type_true_type(node) {
                        visit_for_display_parts(state, parts, &true_type, id_to_symbol);
                    }
                    push_part(parts, " : ");
                    if let Some(false_type) = conditional_type_false_type(node) {
                        visit_for_display_parts(state, parts, &false_type, id_to_symbol);
                    }
                }
                SyntaxKind::InferType => {
                    push_part(parts, "infer ");
                    if let Some(type_parameter) = infer_type_type_parameter(node) {
                        visit_for_display_parts(state, parts, &type_parameter, id_to_symbol);
                    }
                }
                SyntaxKind::ParenthesizedType => {
                    push_part(parts, "(");
                    if let Some(t) = node.type_node() {
                        visit_for_display_parts(state, parts, &t, id_to_symbol);
                    }
                    push_part(parts, ")");
                }
                SyntaxKind::TypeOperator => {
                    push_part(
                        parts,
                        tsox_frontend::scanner::token_to_string(type_operator_operator(node)),
                    );
                    if let Some(t) = node.type_node() {
                        visit_for_display_parts(state, parts, &t, id_to_symbol);
                    }
                }
                SyntaxKind::IndexedAccessType => {
                    if let Some(object_type) = indexed_access_object_type(node) {
                        visit_for_display_parts(state, parts, &object_type, id_to_symbol);
                    }
                    push_part(parts, "[");
                    if let Some(index_type) = indexed_access_index_type(node) {
                        visit_for_display_parts(state, parts, &index_type, id_to_symbol);
                    }
                    push_part(parts, "]");
                }
                SyntaxKind::MappedType => {
                    push_part(parts, "{ ");
                    if let Some(readonly_token) = mapped_type_readonly_token(node) {
                        if readonly_token.kind == SyntaxKind::PlusToken {
                            push_part(parts, "+");
                        } else if readonly_token.kind == SyntaxKind::MinusToken {
                            push_part(parts, "-");
                        }
                        push_part(parts, "readonly ");
                    }
                    push_part(parts, "[");
                    if let Some(type_parameter) = mapped_type_type_parameter(node) {
                        visit_for_display_parts(state, parts, &type_parameter, id_to_symbol);
                    }
                    if let Some(name_type) = mapped_type_name_type(node) {
                        push_part(parts, " as ");
                        visit_for_display_parts(state, parts, &name_type, id_to_symbol);
                    }
                    push_part(parts, "]");
                    if let Some(question_token) = node_question_token_of(node) {
                        if question_token.kind == SyntaxKind::PlusToken {
                            push_part(parts, "+");
                        } else if question_token.kind == SyntaxKind::MinusToken {
                            push_part(parts, "-");
                        }
                        push_part(parts, "?");
                    }
                    push_part(parts, ": ");
                    if let Some(t) = node.type_node() {
                        visit_for_display_parts(state, parts, &t, id_to_symbol);
                    }
                    push_part(parts, "; }");
                }
                SyntaxKind::LiteralType => {
                    if let Some(literal) = literal_type_literal(node) {
                        visit_for_display_parts(state, parts, &literal, id_to_symbol);
                    }
                }
                SyntaxKind::FunctionType => {
                    visit_parameters_and_type_parameters(state, parts, node, id_to_symbol);
                    push_part(parts, " => ");
                    if let Some(t) = node.type_node() {
                        visit_for_display_parts(state, parts, &t, id_to_symbol);
                    }
                }
                SyntaxKind::ImportType => {
                    if import_type_is_type_of(node) {
                        push_part(parts, "typeof ");
                    }
                    push_part(parts, "import(");
                    if let Some(argument) = import_type_argument(node) {
                        visit_for_display_parts(state, parts, &argument, id_to_symbol);
                    }
                    push_part(parts, ")");
                    if let Some(qualifier) = import_type_qualifier(node) {
                        push_part(parts, ".");
                        visit_for_display_parts(state, parts, &qualifier, id_to_symbol);
                    }
                    let type_arguments = node_type_arguments(node);
                    if !type_arguments.is_empty() {
                        push_part(parts, "<");
                        visit_display_part_list(state, parts, &type_arguments, ", ", id_to_symbol);
                        push_part(parts, ">");
                    }
                }
                SyntaxKind::PropertySignature => {
                    let modifiers: Vec<Arc<Node>> = node.modifier_nodes().iter().cloned().collect();
                    if !modifiers.is_empty() {
                        visit_display_part_list(state, parts, &modifiers, " ", id_to_symbol);
                        push_part(parts, " ");
                    }
                    if let Some(name) = node.name() {
                        visit_for_display_parts(state, parts, name, id_to_symbol);
                    }
                    if let Some(postfix_token) = node_postfix_token(node) {
                        push_part(
                            parts,
                            tsox_frontend::scanner::token_to_string(postfix_token.kind),
                        );
                    }
                    if let Some(t) = node.type_node() {
                        push_part(parts, ": ");
                        visit_for_display_parts(state, parts, &t, id_to_symbol);
                    }
                }
                SyntaxKind::IndexSignature => {
                    push_part(parts, "[");
                    visit_display_part_list(state, parts, &node_parameters(node), ", ", id_to_symbol);
                    push_part(parts, "]");
                    if let Some(t) = node.type_node() {
                        push_part(parts, ": ");
                        visit_for_display_parts(state, parts, &t, id_to_symbol);
                    }
                }
                SyntaxKind::MethodSignature => {
                    let modifiers: Vec<Arc<Node>> = node.modifier_nodes().iter().cloned().collect();
                    if !modifiers.is_empty() {
                        visit_display_part_list(state, parts, &modifiers, " ", id_to_symbol);
                        push_part(parts, " ");
                    }
                    if let Some(name) = node.name() {
                        visit_for_display_parts(state, parts, name, id_to_symbol);
                    }
                    if let Some(postfix_token) = node_postfix_token(node) {
                        push_part(
                            parts,
                            tsox_frontend::scanner::token_to_string(postfix_token.kind),
                        );
                    }
                    visit_parameters_and_type_parameters(state, parts, node, id_to_symbol);
                    if let Some(t) = node.type_node() {
                        push_part(parts, ": ");
                        visit_for_display_parts(state, parts, &t, id_to_symbol);
                    }
                }
                SyntaxKind::CallSignature => {
                    visit_parameters_and_type_parameters(state, parts, node, id_to_symbol);
                    if let Some(t) = node.type_node() {
                        push_part(parts, ": ");
                        visit_for_display_parts(state, parts, &t, id_to_symbol);
                    }
                }
                SyntaxKind::ConstructSignature => {
                    push_part(parts, "new ");
                    visit_parameters_and_type_parameters(state, parts, node, id_to_symbol);
                    if let Some(t) = node.type_node() {
                        push_part(parts, ": ");
                        visit_for_display_parts(state, parts, &t, id_to_symbol);
                    }
                }
                SyntaxKind::ArrayBindingPattern => {
                    push_part(parts, "[");
                    visit_display_part_list(state, parts, &node_elements(node), ", ", id_to_symbol);
                    push_part(parts, "]");
                }
                SyntaxKind::ObjectBindingPattern => {
                    push_part(parts, "{");
                    let elements = node_elements(node);
                    if !elements.is_empty() {
                        push_part(parts, " ");
                        visit_display_part_list(state, parts, &elements, ", ", id_to_symbol);
                        push_part(parts, " ");
                    }
                    push_part(parts, "}");
                }
                SyntaxKind::BindingElement => {
                    if let Some(name) = node.name() {
                        visit_for_display_parts(state, parts, name, id_to_symbol);
                    }
                }
                SyntaxKind::PrefixUnaryExpression => {
                    push_part(
                        parts,
                        tsox_frontend::scanner::token_to_string(prefix_unary_operator(node)),
                    );
                    if let Some(operand) = prefix_unary_operand(node) {
                        visit_for_display_parts(state, parts, &operand, id_to_symbol);
                    }
                }
                SyntaxKind::TemplateLiteralType => {
                    if let Some(head) = template_literal_type_head(node) {
                        visit_for_display_parts(state, parts, &head, id_to_symbol);
                    }
                    for span in template_literal_type_spans(node) {
                        visit_for_display_parts(state, parts, &span, id_to_symbol);
                    }
                }
                SyntaxKind::TemplateHead => {
                    push_part(parts, &state.get_literal_text(node));
                }
                SyntaxKind::TemplateLiteralTypeSpan => {
                    if let Some(t) = node.type_node() {
                        visit_for_display_parts(state, parts, &t, id_to_symbol);
                    }
                    if let Some(literal) = template_literal_type_span_literal(node) {
                        visit_for_display_parts(state, parts, &literal, id_to_symbol);
                    }
                }
                SyntaxKind::TemplateMiddle | SyntaxKind::TemplateTail => {
                    push_part(parts, &state.get_literal_text(node));
                }
                SyntaxKind::ThisType => {
                    push_part(parts, "this");
                }
                SyntaxKind::ComputedPropertyName => {
                    push_part(parts, "[");
                    if let Some(expr) = node.expression() {
                        visit_for_display_parts(state, parts, expr, id_to_symbol);
                    }
                    push_part(parts, "]");
                }
                SyntaxKind::PropertyAccessExpression => {
                    if let Some(expr) = node.expression() {
                        visit_for_display_parts(state, parts, expr, id_to_symbol);
                    }
                    push_part(parts, ".");
                    if let Some(name) = node.name() {
                        visit_for_display_parts(state, parts, name, id_to_symbol);
                    }
                }
                SyntaxKind::ElementAccessExpression => {
                    if let Some(expr) = node.expression() {
                        visit_for_display_parts(state, parts, expr, id_to_symbol);
                    }
                    push_part(parts, "[");
                    if let Some(argument) = element_access_argument_expression(node) {
                        visit_for_display_parts(state, parts, &argument, id_to_symbol);
                    }
                    push_part(parts, "]");
                }
                _ => {}
            }
        }

        visit_for_display_parts(self, &mut parts, node, id_to_symbol);
        parts
    }

    pub fn get_node_display_part(&self, text: &str, node: &Arc<Node>) -> InlayHintLabelPart {
        let part = InlayHintLabelPart { value: text.to_string(), location: None };
        let Some(source_file_node) = ast::get_source_file_of_node(node) else {
            return part;
        };
        let Some(file) = crate::ls::mig::m5u::node_as_source_file(&source_file_node) else {
            return part;
        };
        let pos = astnav::get_start_of_node(node, &file, false);
        let end = node.end();
        let script = crate::mig::m5u_conv::SourceFileScriptView {
            file: Arc::clone(&file),
        };
        let (lsp_range, fidelity) = self.converters.to_lsp_range_for_feature(
            &script,
            tsox_core::core::text::TextRange::new(pos, end),
            spanmap_feature_inlay_hints(),
        );
        if fidelity_is_single_segment(&fidelity) {
            return InlayHintLabelPart {
                value: text.to_string(),
                location: Some(crate::lsp::lsproto_lsp::Location {
                    uri: crate::lsp::lsproto_lsp::DocumentUri(
                        crate::ls::lsconv_converters::file_name_to_document_uri(
                            &file.file_name,
                        ),
                    ),
                    range: lsp_range,
                }),
            };
        }
        part
    }

    pub fn get_literal_text(&self, node: &Arc<Node>) -> String {
        match node.kind {
            SyntaxKind::StringLiteral => {
                if self.quote_preference == QuotePreference::Single {
                    format!(
                        "'{}'",
                        printer_escape_string(node.text(), printer_quote_char_single_quote())
                    )
                } else {
                    format!(
                        "\"{}\"",
                        printer_escape_string(node.text(), printer_quote_char_double_quote())
                    )
                }
            }
            SyntaxKind::TemplateHead | SyntaxKind::TemplateMiddle | SyntaxKind::TemplateTail => {
                let mut raw_text = node_raw_text(node).to_string();
                if raw_text.is_empty() {
                    raw_text =
                        printer_escape_string(node.text(), printer_quote_char_backtick());
                }
                match node.kind {
                    SyntaxKind::TemplateHead => format!("`{raw_text}=${{"),
                    SyntaxKind::TemplateMiddle => format!("}}{raw_text}${{"),
                    SyntaxKind::TemplateTail => format!("}}{raw_text}`"),
                    _ => node.text().to_string(),
                }
            }
            _ => node.text().to_string(),
        }
    }

    pub fn get_parameter_identifier_info_at_position(
        &mut self,
        signature: &tsox_checker::checker::Signature,
        pos: usize,
    ) -> Option<ParameterInfo> {
        let parameters = signature.parameters();
        let param_count =
            parameters.len() - if signature_has_rest_parameter(signature) { 1 } else { 0 };
        if pos < param_count {
            let param = &parameters[pos];
            let param_id = get_parameter_declaration_identifier(param)?;
            return Some(ParameterInfo {
                name: param_id.text().to_string(),
                parameter: param_id,
                is_rest_parameter: false,
            });
        }

        let mut rest_parameter: Option<Arc<Symbol>> = None;
        let mut rest_id: Option<Arc<Node>> = None;
        if param_count < parameters.len() {
            let rest = &parameters[param_count];
            rest_id = get_parameter_declaration_identifier(rest);
            rest_parameter = Some(Arc::clone(rest));
        }
        let rest_id = rest_id?;
        let rest_parameter = rest_parameter?;

        let rest_type = self.checker.get_type_of_symbol(&rest_parameter);
        if self.checker.is_tuple_type(&rest_type) {
            let associated_names: Vec<Option<Arc<Node>>> = rest_type
                .target()
                .and_then(|t| t.as_tuple_type())
                .map(|tuple| {
                    tuple
                        .element_infos
                        .iter()
                        .map(|element_info| element_info.labeled_declaration().cloned())
                        .collect()
                })
                .unwrap_or_default();
            let index = pos - param_count;
            if index < associated_names.len() {
                if let Some(associated_name) = associated_names[index].clone() {
                    let is_rest_tuple_element = if ast::is_named_tuple_member(&associated_name) {
                        named_tuple_member_dot_dot_dot_token(&associated_name).is_some()
                    } else {
                        parameter_dot_dot_dot_token(&associated_name).is_some()
                    };
                    let name = associated_name.name()?;
                    return Some(ParameterInfo {
                        name: name.text().to_string(),
                        parameter: Arc::clone(name),
                        is_rest_parameter: is_rest_tuple_element,
                    });
                }
            }
            return None;
        }

        if pos == param_count {
            return Some(ParameterInfo {
                name: rest_parameter.name.clone(),
                parameter: rest_id,
                is_rest_parameter: true,
            });
        }
        None
    }

    pub fn leading_comments_contains_parameter_name(
        &self,
        node: &Arc<Node>,
        name: &str,
    ) -> bool {
        if !tsox_frontend::scanner::mig::m3i::is_identifier_text(
            name,
            self.file.language_variant,
        ) {
            return false;
        }

        let ranges = get_leading_comment_ranges_of_node(node, &self.file);
        let file_text = &self.file.text;
        for r in ranges {
            let comment_text = file_text[r.pos..r.end]
                .trim_matches(|c: char| c.is_whitespace() || c == '/' || c == '*');
            if comment_text == name {
                return true;
            }
        }

        false
    }

    pub fn get_type_annotation_position(&self, decl: &Arc<Node>) -> usize {
        if let Some(close_paren_token) =
            astnav::find_child_of_kind(decl, SyntaxKind::CloseParenToken)
        {
            return close_paren_token.end();
        }
        node_parameter_list(decl).map_or(decl.end(), |pl| pl.end())
    }
}

pub const INLAY_HINT_KIND_TYPE: i32 = 1;
pub const INLAY_HINT_KIND_PARAMETER: i32 = 2;

pub fn node_arguments(node: &Arc<Node>) -> Vec<Arc<Node>> {
    match &node.data {
        NodeData::CallExpression(d) => d.arguments.nodes.clone(),
        NodeData::NewExpression(d) => d
            .arguments
            .as_ref()
            .map(|arguments| arguments.nodes.clone())
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

pub fn node_parameters(node: &Arc<Node>) -> Vec<Arc<Node>> {
    match &node.data {
        NodeData::IndexSignatureDeclaration(d) => d.parameters.nodes.clone(),
        _ => ast::mig::m3f_2::node_parameters(node)
            .map(|parameters| parameters.nodes.clone())
            .unwrap_or_default(),
    }
}

pub fn node_type_parameters(node: &Arc<Node>) -> Vec<Arc<Node>> {
    ast::mig::m3f_2::node_type_parameters(node)
        .map(|type_parameters| type_parameters.nodes.clone())
        .unwrap_or_default()
}

pub fn node_type_arguments(node: &Arc<Node>) -> Vec<Arc<Node>> {
    let type_arguments = match &node.data {
        NodeData::TypeReferenceNode(d) => d.type_arguments.as_ref(),
        NodeData::TypeQueryNode(d) => d.type_arguments.as_ref(),
        NodeData::ImportTypeNode(d) => d.type_arguments.as_ref(),
        _ => None,
    };
    type_arguments
        .map(|type_arguments| type_arguments.nodes.clone())
        .unwrap_or_default()
}

pub fn node_members(node: &Arc<Node>) -> Vec<Arc<Node>> {
    match &node.data {
        NodeData::ClassDeclaration(d) => d.members.nodes.clone(),
        NodeData::ClassExpression(d) => d.members.nodes.clone(),
        NodeData::InterfaceDeclaration(d) => d.members.nodes.clone(),
        NodeData::EnumDeclaration(d) => d.members.nodes.clone(),
        NodeData::TypeLiteralNode(d) => d.members.nodes.clone(),
        NodeData::MappedTypeNode(d) => d
            .members
            .as_ref()
            .map(|members| members.nodes.clone())
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

pub fn node_elements(node: &Arc<Node>) -> Vec<Arc<Node>> {
    match &node.data {
        NodeData::TupleTypeNode(d) => d.elements.nodes.clone(),
        NodeData::BindingPattern(d) => d.elements.nodes.clone(),
        _ => Vec::new(),
    }
}

pub fn node_parameter_list(node: &Arc<Node>) -> Option<Arc<tsox_frontend::ast::NodeList>> {
    ast::mig::m3f_2::node_parameters(node).cloned()
}

pub fn node_postfix_token(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::PropertySignatureDeclaration(d) => d.postfix_token.clone(),
        NodeData::MethodSignatureDeclaration(d) => d.postfix_token.clone(),
        NodeData::MethodDeclaration(d) => d.postfix_token.clone(),
        _ => None,
    }
}

pub fn node_question_token_of(node: &Arc<Node>) -> Option<Arc<Node>> {
    ast::mig::m3f_2::question_token(node).cloned()
}

pub fn node_raw_text(node: &Arc<Node>) -> &str {
    match &node.data {
        NodeData::TemplateHead(d) => &d.raw_text,
        NodeData::TemplateMiddle(d) => &d.raw_text,
        NodeData::TemplateTail(d) => &d.raw_text,
        _ => "",
    }
}

pub fn prefix_unary_operand(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::PrefixUnaryExpression(d) => Some(Arc::clone(&d.operand)),
        _ => None,
    }
}

pub fn prefix_unary_operator(node: &Arc<Node>) -> SyntaxKind {
    match &node.data {
        NodeData::PrefixUnaryExpression(d) => d.operator,
        _ => SyntaxKind::EndOfFile,
    }
}

pub fn qualified_name_left(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::QualifiedName(d) => Some(Arc::clone(&d.left)),
        _ => None,
    }
}

pub fn qualified_name_right(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::QualifiedName(d) => Some(Arc::clone(&d.right)),
        _ => None,
    }
}

pub fn type_predicate_parameter_name(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::TypePredicateNode(d) => Some(Arc::clone(&d.parameter_name)),
        _ => None,
    }
}

pub fn type_predicate_asserts_modifier(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::TypePredicateNode(d) => d.asserts_modifier.as_ref().map(Arc::clone),
        _ => None,
    }
}

pub fn type_reference_type_name(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::TypeReferenceNode(d) => Some(Arc::clone(&d.type_name)),
        _ => None,
    }
}

pub fn type_parameter_constraint(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::TypeParameterDeclaration(d) => d.constraint.as_ref().map(Arc::clone),
        _ => None,
    }
}

pub fn type_parameter_default_type(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::TypeParameterDeclaration(d) => d.default_type.as_ref().map(Arc::clone),
        _ => None,
    }
}

pub fn parameter_dot_dot_dot_token(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::ParameterDeclaration(d) => d.dot_dot_dot_token.as_ref().map(Arc::clone),
        _ => None,
    }
}

pub fn type_query_expr_name(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::TypeQueryNode(d) => Some(Arc::clone(&d.expr_name)),
        _ => None,
    }
}

pub fn array_type_element_type(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::ArrayTypeNode(d) => Some(Arc::clone(&d.element_type)),
        _ => None,
    }
}

pub fn named_tuple_member_dot_dot_dot_token(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::NamedTupleMember(d) => d.dot_dot_dot_token.as_ref().map(Arc::clone),
        _ => None,
    }
}

pub fn union_or_intersection_type_nodes(node: &Arc<Node>) -> Vec<Arc<Node>> {
    match &node.data {
        NodeData::UnionTypeNode(d) => d.types.nodes.clone(),
        NodeData::IntersectionTypeNode(d) => d.types.nodes.clone(),
        _ => Vec::new(),
    }
}

pub fn conditional_type_check_type(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::ConditionalTypeNode(d) => Some(Arc::clone(&d.check_type)),
        _ => None,
    }
}

pub fn conditional_type_extends_type(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::ConditionalTypeNode(d) => Some(Arc::clone(&d.extends_type)),
        _ => None,
    }
}

pub fn conditional_type_true_type(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::ConditionalTypeNode(d) => Some(Arc::clone(&d.true_type)),
        _ => None,
    }
}

pub fn conditional_type_false_type(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::ConditionalTypeNode(d) => Some(Arc::clone(&d.false_type)),
        _ => None,
    }
}

pub fn infer_type_type_parameter(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::InferTypeNode(d) => Some(Arc::clone(&d.type_parameter)),
        _ => None,
    }
}

pub fn type_operator_operator(node: &Arc<Node>) -> SyntaxKind {
    match &node.data {
        NodeData::TypeOperatorNode(d) => d.operator,
        _ => SyntaxKind::EndOfFile,
    }
}

pub fn indexed_access_object_type(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::IndexedAccessTypeNode(d) => Some(Arc::clone(&d.object_type)),
        _ => None,
    }
}

pub fn indexed_access_index_type(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::IndexedAccessTypeNode(d) => Some(Arc::clone(&d.index_type)),
        _ => None,
    }
}

pub fn mapped_type_readonly_token(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::MappedTypeNode(d) => d.readonly_token.as_ref().map(Arc::clone),
        _ => None,
    }
}

pub fn mapped_type_type_parameter(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::MappedTypeNode(d) => Some(Arc::clone(&d.type_parameter)),
        _ => None,
    }
}

pub fn mapped_type_name_type(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::MappedTypeNode(d) => d.name_type.as_ref().map(Arc::clone),
        _ => None,
    }
}

pub fn literal_type_literal(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::LiteralTypeNode(d) => Some(Arc::clone(&d.literal)),
        _ => None,
    }
}

pub fn template_literal_type_head(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::TemplateLiteralTypeNode(d) => Some(Arc::clone(&d.head)),
        _ => None,
    }
}

pub fn template_literal_type_spans(node: &Arc<Node>) -> Vec<Arc<Node>> {
    match &node.data {
        NodeData::TemplateLiteralTypeNode(d) => d.template_spans.nodes.clone(),
        _ => Vec::new(),
    }
}

pub fn template_literal_type_span_literal(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::TemplateLiteralTypeSpan(d) => Some(Arc::clone(&d.literal)),
        _ => None,
    }
}

pub fn element_access_argument_expression(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::ElementAccessExpression(d) => Some(Arc::clone(&d.argument_expression)),
        _ => None,
    }
}

pub fn import_type_is_type_of(node: &Arc<Node>) -> bool {
    match &node.data {
        NodeData::ImportTypeNode(d) => d.is_type_of,
        _ => false,
    }
}

pub fn import_type_argument(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::ImportTypeNode(d) => Some(Arc::clone(&d.argument)),
        _ => None,
    }
}

pub fn import_type_qualifier(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::ImportTypeNode(d) => d.qualifier.as_ref().map(Arc::clone),
        _ => None,
    }
}

pub fn checker_element_flags_required() -> tsox_checker::checker::ElementFlags {
    tsox_checker::checker::ElementFlags::Required
}

pub fn node_builder_flags_ignore_errors() -> tsox_checker::checker::symboltracker::NodeBuilderFlags {
    tsox_checker::checker::symboltracker::NodeBuilderFlags::IGNORE_ERRORS
}

pub fn node_builder_flags_allow_unique_es_symbol_type(
) -> tsox_checker::checker::symboltracker::NodeBuilderFlags {
    tsox_checker::checker::symboltracker::NodeBuilderFlags::AllowUniqueESSymbolType
}

pub fn node_builder_flags_use_alias_defined_outside_current_scope(
) -> tsox_checker::checker::symboltracker::NodeBuilderFlags {
    tsox_checker::checker::symboltracker::NodeBuilderFlags::UseAliasDefinedOutsideCurrentScope
}

pub fn spanmap_feature_inlay_hints() -> u32 {
    crate::mig::m5u_conv::SPANMAP_FEATURE_INLAY_HINTS
}

pub fn fidelity_is_none(fidelity: &u32) -> bool {
    *fidelity == crate::mig::m5u_conv::SPANMAP_FIDELITY_NONE
}

pub fn fidelity_is_single_segment(fidelity: &u32) -> bool {
    *fidelity == crate::mig::m5u_conv::SPANMAP_FIDELITY_SINGLE_SEGMENT
}

pub fn printer_escape_string(
    text: &str,
    quote: tsox_frontend::format::mig::m4t_2::QuoteChar,
) -> String {
    tsox_frontend::format::mig::m4t_2::escape_string(text, quote)
}

pub fn printer_quote_char_single_quote() -> tsox_frontend::format::mig::m4t_2::QuoteChar {
    tsox_frontend::format::mig::m4t_2::QuoteChar::SingleQuote
}

pub fn printer_quote_char_double_quote() -> tsox_frontend::format::mig::m4t_2::QuoteChar {
    tsox_frontend::format::mig::m4t_2::QuoteChar::DoubleQuote
}

pub fn printer_quote_char_backtick() -> tsox_frontend::format::mig::m4t_2::QuoteChar {
    tsox_frontend::format::mig::m4t_2::QuoteChar::Backtick
}

pub fn evaluator_any_to_string(value: &str) -> String {
    value.to_string()
}

pub fn equate_string_case_insensitive(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

pub fn signature_has_rest_parameter(signature: &tsox_checker::checker::Signature) -> bool {
    signature.has_rest_parameter()
}

pub fn get_leading_comment_ranges_of_node(
    node: &Arc<Node>,
    file: &Arc<SourceFile>,
) -> Vec<tsox_frontend::scanner::CommentRange> {
    crate::ls::utilities::get_leading_comment_ranges_of_node(node, file)
}
