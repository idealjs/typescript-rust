use tsox_frontend::ast::mig::m3d_2::new_diagnostic_chain;
use crate::checker::mig::m2a::r19k11_defs::*;
use tsox_frontend::ast::mig::m3e_4::get_import_attributes;
use tsox_frontend::ast::mig::m3g_2::is_parameter_property_declaration;
use crate::checker::mig::m3a_2::{find_in_map, new_diagnostic_for_node};
use crate::checker::mig::wc3::CallState;
use crate::checker::mig::m2b::r23k6_defs;
use crate::checker::utilities_get_assignment_target::entity_name_to_string;
use crate::checker::utilities_token_is_identifier_or_keyword::is_object_literal_type;
use crate::checker::checker::*;
use std::sync::Arc;
use tsox_frontend::ast::mig::m3b::{members, module_specifier, parameters};
use tsox_core::diagnostics::new_ad_hoc_message;
use tsox_frontend::ast::{Diagnostic, Node, Symbol, SyntaxKind};

impl Checker {
    pub fn report_call_resolution_errors(
        &mut self,
        node: &Arc<Node>,
        s: &mut CallState,
        signatures: &[Arc<Signature>],
        head_message: Option<&'static tsox_core::diagnostics::Message>,
    ) {
        let candidates = r23k6_defs::call_state_candidates_for_argument_error();
        if !candidates.is_empty() {
            let last = Arc::clone(candidates.last().unwrap());
            let mut diags: Vec<Diagnostic> = Vec::new();
            let assignable_relation = self.assignable_relation();
            self.is_signature_applicable(
                &s.node,
                &s.args,
                &last,
                &assignable_relation,
                CheckMode::Normal,
                true,
                &mut diags,
            );
            for diagnostic in diags {
                let mut diagnostic = diagnostic;
                if candidates.len() > 1 {
                    diagnostic = new_diagnostic_chain(
                        Some(&diagnostic),
                        tsox_core::diagnostics::messages_generated::THE_LAST_OVERLOAD_GAVE_THE_FOLLOWING_ERROR,
                        vec![],
                    );
                    diagnostic = new_diagnostic_chain(
                        Some(&diagnostic),
                        tsox_core::diagnostics::messages_generated::NO_OVERLOAD_MATCHES_THIS_CALL,
                        vec![],
                    );
                }
                if let Some(head_message) = head_message {
                    diagnostic = new_diagnostic_chain(Some(&diagnostic), *head_message, vec![]);
                }
                if last.declaration.is_some() && candidates.len() > 1 {
                    diagnostic.add_related_info(new_diagnostic_for_node(
                        last.declaration.as_ref(),
                        tsox_core::diagnostics::messages_generated::THE_LAST_OVERLOAD_IS_DECLARED_HERE,
                        vec![],
                    ));
                }
                self.add_implementation_success_elaboration(s, &last, &mut diagnostic);
                self.add_diagnostic(diagnostic);
            }
        } else if let Some(candidate) = r23k6_defs::call_state_candidate_for_argument_arity_error() {
            let head = head_message;
            let diagnostic =
                self.get_argument_arity_error(&s.node, &[candidate], &s.args, head);
            self.add_diagnostic(diagnostic);
        } else if let Some(candidate) = r23k6_defs::call_state_candidate_for_type_argument_error() {
            let type_arguments = s
                .node
                .type_arguments()
                .map(|l| l.nodes.clone())
                .unwrap_or_default();
            self.check_type_arguments(
                &candidate,
                &type_arguments,
                true,
                head_message,
            );
        } else if !tsox_frontend::ast::is_jsx_opening_fragment(node) {
            let mut with_correct_arity: Vec<Arc<Signature>> = Vec::new();
            for sig in signatures {
                if self.has_correct_type_argument_arity(sig, &s.type_arguments) {
                    with_correct_arity.push(Arc::clone(sig));
                }
            }
            if with_correct_arity.is_empty() {
                let diagnostic = self.get_type_argument_arity_error(
                    &s.node,
                    signatures,
                    &s.type_arguments,
                    head_message,
                );
                self.add_diagnostic(diagnostic);
            } else {
                let diagnostic = self.get_argument_arity_error(
                    &s.node,
                    &with_correct_arity,
                    &s.args,
                    head_message,
                );
                self.add_diagnostic(diagnostic);
            }
        }
    }

    pub fn report_cannot_invoke_possibly_null_or_undefined_error(
        &mut self,
        node: &Arc<Node>,
        facts: TypeFacts,
    ) {
        let message = if facts.intersects(TypeFacts::IS_UNDEFINED) {
            if facts.intersects(TypeFacts::IS_NULL) {
                tsox_core::diagnostics::messages_generated::CANNOT_INVOKE_AN_OBJECT_WHICH_IS_POSSIBLY_NULL_OR_UNDEFINED
            } else {
                tsox_core::diagnostics::messages_generated::CANNOT_INVOKE_AN_OBJECT_WHICH_IS_POSSIBLY_UNDEFINED
            }
        } else {
            tsox_core::diagnostics::messages_generated::CANNOT_INVOKE_AN_OBJECT_WHICH_IS_POSSIBLY_NULL
        };
        self.error_message(node, message, &[]);
    }

    pub fn report_circular_base_type(&mut self, node: &Arc<Node>, t: &Arc<Type>) {
        let type_string = self.type_to_string_ex(
            t,
            crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags::WRITE_ARRAY_AS_GENERIC,
        );
        self.error_message(
            node,tsox_core::diagnostics::messages_generated::TYPE_0_RECURSIVELY_REFERENCES_ITSELF_AS_A_BASE_TYPE,
            &[type_string],
        );
    }

    pub fn report_duplicate_member_errors(
        &mut self,
        node: &Arc<Node>,
        name: &str,
        check_static: bool,
        is_static: bool,
        message: tsox_core::diagnostics::Message,
    ) {
        for member in members(node) {
            if tsox_frontend::ast::is_constructor_declaration(member) {
            for param in parameters(member) {
                if tsox_frontend::ast::mig::m3g_2::is_parameter_property_declaration(param, member)
                        && !tsox_frontend::ast::is_binding_pattern(param.name().unwrap())
                    {
                        if let Some(symbol) = self.get_symbol_of_declaration(param) {
                            if symbol.name == name {
                                let symbol_string = self.symbol_to_string(&symbol);
                                self.error_message(param.name().unwrap(), message, &[symbol_string]);
                            }
                        }
                    }
                }
            } else if let Some(symbol) = self.get_symbol_of_declaration(member) {
                if symbol.name == name
                    && (!check_static || is_static == tsox_frontend::ast::is_static(member))
                {
                    let symbol_string = self.symbol_to_string(&symbol);
                    self.error_message(member.name().unwrap(), message, &[symbol_string]);
                }
            }
        }
    }

    pub fn report_errors_from_widening(
        &mut self,
        declaration: &Arc<Node>,
        t: &Arc<Type>,
        widening_kind: WideningKind,
    ) {
        if self.no_implicit_any
            && t.object_flags
                .intersects(ObjectFlags::ContainsWideningType)
        {
            if widening_kind == WideningKind::Normal
                || tsox_frontend::ast::is_function_like_declaration(declaration)
                    && self.should_report_errors_from_widening_with_contextual_signature(
                        declaration, widening_kind,
                    )
            {
                if !self.report_widening_errors_in_type(t) {
                    self.report_implicit_any(declaration, t, widening_kind);
                }
            }
        }
    }

    pub fn report_invalid_import_equals_export_member(
        &mut self,
        name: &Arc<Node>,
        declaration_name: &str,
        module_name: &str,
    ) {
        if self.module_kind >= ModuleKind::ES2015 {
            self.error_message(
                name,tsox_core::diagnostics::messages_generated::X_0_CAN_ONLY_BE_IMPORTED_BY_USING_A_DEFAULT_IMPORT,
                &[declaration_name.to_string()],
            );
        } else if tsox_frontend::ast::is_in_js_file(name) {
            self.error_message(
                name,tsox_core::diagnostics::messages_generated::X_0_CAN_ONLY_BE_IMPORTED_BY_USING_A_REQUIRE_CALL_OR_BY_USING_A_DEFAULT_IMPORT,
                &[declaration_name.to_string()],
            );
        } else {
            self.error_message(
                name,tsox_core::diagnostics::messages_generated::X_0_CAN_ONLY_BE_IMPORTED_BY_USING_IMPORT_1_REQUIRE_2_OR_A_DEFAULT_IMPORT,
                &[
                    declaration_name.to_string(),
                    declaration_name.to_string(),
                    module_name.to_string(),
                ],
            );
        }
    }

    pub fn report_non_default_export(&mut self, module_symbol: &Arc<Symbol>, node: &Arc<Node>) {
        let node_symbol = self.symbol_of_node(node);
        let node_symbol_name = node_symbol
            .as_ref()
            .map(|s| s.name.clone())
            .unwrap_or_default();
        if module_symbol
            .exports
            .entries
            .contains_key(&node_symbol_name)
        {
            let module_string = self.symbol_to_string(module_symbol);
            self.error_message(
                node,tsox_core::diagnostics::messages_generated::MODULE_0_HAS_NO_DEFAULT_EXPORT_DID_YOU_MEAN_TO_USE_IMPORT_1_FROM_0_INSTEAD,
                &[module_string, node_symbol_name],
            );
        } else {
            let module_string = self.symbol_to_string(module_symbol);
            let diagnostic = self.error_message(
                &node.name().unwrap(),tsox_core::diagnostics::messages_generated::MODULE_0_HAS_NO_DEFAULT_EXPORT,
                &[module_string],
            );
            let export_star = module_symbol
                .exports
                .entries
                .get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_STAR)
                .cloned();
            if let (Some(mut diagnostic), Some(export_star)) = (diagnostic, export_star) {
                let default_export = export_star.declarations.iter().find(|decl| {
                    if !(tsox_frontend::ast::is_export_declaration(decl)
                        && module_specifier(decl).is_some())
                    {
                        return false;
                    }
                    let import_attributes =
                        tsox_frontend::ast::mig::m3e_4::get_import_attributes(decl);
                    let attributes_type =
                        self.get_type_from_import_attributes(import_attributes.as_ref());
                    let resolved = self.resolve_external_module_name_worker(
                        decl,
                        module_specifier(decl),
                        None,
                        false,
                        false,
                        attributes_type.as_ref(),
                    );
                    resolved.is_some_and(|resolved| {
                        resolved
                            .exports
                            .entries
                            .contains_key(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_DEFAULT)
                    })
                });
                if let Some(default_export) = default_export {
                    if let Some(diagnostic) = Arc::get_mut(&mut diagnostic) {
                        diagnostic.add_related_info(new_diagnostic_for_node(
                            Some(default_export),
                            tsox_core::diagnostics::messages_generated::X_EXPORT_ASTERISK_DOES_NOT_RE_EXPORT_A_DEFAULT,
                            vec![],
                        ));
                    }
                }
            }
        }
    }

    pub fn report_non_exported_member(
        &mut self,
        name: &Arc<Node>,
        declaration_name: &str,
        module_symbol: &Arc<Symbol>,
        module_name: &str,
    ) {
        let local_symbol: Option<Arc<Symbol>> = None;
        let exports = module_symbol.exports.clone();
        if let Some(local_symbol) = local_symbol {
            if let Some(exported_equals_symbol) =
                exports.entries.get(tsox_frontend::ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
            {
                if self
                    .get_symbol_if_same_reference(exported_equals_symbol, &local_symbol)
                    .is_some()
                {
                    self.report_invalid_import_equals_export_member(name, declaration_name, module_name);
                } else {
                    self.error_message(
                        name,tsox_core::diagnostics::messages_generated::MODULE_0_HAS_NO_EXPORTED_MEMBER_1,
                        &[module_name.to_string(), declaration_name.to_string()],
                    );
                }
            } else {
                let exported_symbol = exports.entries.values().find(|symbol| {
                    self.get_symbol_if_same_reference(symbol, &local_symbol).is_some()
                });
                let diagnostic = if let Some(exported_symbol) = &exported_symbol {
                    let symbol_string = self.symbol_to_string(exported_symbol);
                    self.error_message(
                        name,tsox_core::diagnostics::messages_generated::MODULE_0_DECLARES_1_LOCALLY_BUT_IT_IS_EXPORTED_AS_2,
                        &[
                            module_name.to_string(),
                            declaration_name.to_string(),
                            symbol_string,
                        ],
                    )
                } else {
                    self.error_message(
                        name,tsox_core::diagnostics::messages_generated::MODULE_0_DECLARES_1_LOCALLY_BUT_IT_IS_NOT_EXPORTED,
                        &[module_name.to_string(), declaration_name.to_string()],
                    )
                };
                if let Some(mut diagnostic) = diagnostic {
                    for (i, decl) in local_symbol.declarations.iter().enumerate() {
                        let message = if i == 0 {
                            tsox_core::diagnostics::messages_generated::X_0_IS_DECLARED_HERE
                        } else {
                            tsox_core::diagnostics::messages_generated::X_AND_HERE
                        };
                        if let Some(diagnostic) = Arc::get_mut(&mut diagnostic) {
                            diagnostic.add_related_info(new_diagnostic_for_node(
                                Some(decl),
                                message,
                                vec![declaration_name.to_string()],
                            ));
                        }
                    }
                }
            }
        } else {
            self.error_message(
                name,tsox_core::diagnostics::messages_generated::MODULE_0_HAS_NO_EXPORTED_MEMBER_1,
                &[module_name.to_string(), declaration_name.to_string()],
            );
        }
    }

    pub fn report_object_possibly_null_or_undefined_error(
        &mut self,
        node: &Arc<Node>,
        facts: TypeFacts,
    ) {
        let node_text = if tsox_frontend::ast::is_entity_name_expression(node) {
            entity_name_to_string(node)
        } else {
            String::new()
        };
        if node.kind == SyntaxKind::NullKeyword {
            self.error_message(
                node,tsox_core::diagnostics::messages_generated::THE_VALUE_0_CANNOT_BE_USED_HERE,
                &["null".to_string()],
            );
            return;
        }
        if !node_text.is_empty() && node_text.len() < 100 {
            if tsox_frontend::ast::is_identifier(node) && node_text == "undefined" {
                self.error_message(
                    node,tsox_core::diagnostics::messages_generated::THE_VALUE_0_CANNOT_BE_USED_HERE,
                    &["undefined".to_string()],
                );
                return;
            }
            let message = if facts.intersects(TypeFacts::IS_UNDEFINED) {
                if facts.intersects(TypeFacts::IS_NULL) {
                    tsox_core::diagnostics::messages_generated::X_0_IS_POSSIBLY_NULL_OR_UNDEFINED
                } else {
                    tsox_core::diagnostics::messages_generated::X_0_IS_POSSIBLY_UNDEFINED
                }
            } else {
                tsox_core::diagnostics::messages_generated::X_0_IS_POSSIBLY_NULL
            };
            self.error_message(node, message, &[node_text]);
        } else {
            let message = if facts.intersects(TypeFacts::IS_UNDEFINED) {
                if facts.intersects(TypeFacts::IS_NULL) {
                    tsox_core::diagnostics::messages_generated::OBJECT_IS_POSSIBLY_NULL_OR_UNDEFINED
                } else {
                    tsox_core::diagnostics::messages_generated::OBJECT_IS_POSSIBLY_UNDEFINED
                }
            } else {
                tsox_core::diagnostics::messages_generated::OBJECT_IS_POSSIBLY_NULL
            };
            self.error_message(node, message, &[]);
        }
    }

    pub fn report_operator_error(
        &mut self,
        left_type: &Arc<Type>,
        operator: SyntaxKind,
        right_type: &Arc<Type>,
        error_node: &Arc<Node>,
        is_related: Option<&mut dyn FnMut(&Checker, &Arc<Type>, &Arc<Type>) -> bool>,
    ) {
        let mut would_work_with_await = false;
        let mut is_related = is_related;
        if let Some(is_related_fn) = is_related.as_deref_mut() {
            let awaited_left_type = self.get_awaited_type_no_alias(left_type);
            let awaited_right_type = self.get_awaited_type_no_alias(right_type);
            if let (Some(awaited_left), Some(awaited_right)) =
                (awaited_left_type, awaited_right_type)
            {
                would_work_with_await = !(Arc::ptr_eq(&awaited_left, left_type)
                    && Arc::ptr_eq(&awaited_right, right_type))
                    && is_related_fn(self, &awaited_left, &awaited_right);
            }
        }
        let mut effective_left = Arc::clone(left_type);
        let mut effective_right = Arc::clone(right_type);
        if !would_work_with_await {
            if let Some(is_related_fn) = is_related.as_deref_mut() {
                let (base_left, base_right) = r23k6_defs::get_base_types_if_unrelated_ext(
                    self,
                    &effective_left,
                    &effective_right,
                    is_related_fn,
                );
                effective_left = base_left;
                effective_right = base_right;
            }
        }
        let (left_str, right_str) =
            self.get_type_names_for_error_display(&effective_left, &effective_right);
        match operator {
            SyntaxKind::EqualsEqualsEqualsToken
            | SyntaxKind::EqualsEqualsToken
            | SyntaxKind::ExclamationEqualsEqualsToken
            | SyntaxKind::ExclamationEqualsToken => {
                self.error_and_maybe_suggest_await_message(
                    error_node,
                    would_work_with_await,tsox_core::diagnostics::messages_generated::THIS_COMPARISON_APPEARS_TO_BE_UNINTENTIONAL_BECAUSE_THE_TYPES_0_AND_1_HAVE_NO_OVERLAP,
                    &[left_str, right_str],
                );
            }
            _ => {
                let operator_string = tsox_frontend::scanner::token_to_string(operator);
                self.error_and_maybe_suggest_await_message(
                    error_node,
                    would_work_with_await,tsox_core::diagnostics::messages_generated::OPERATOR_0_CANNOT_BE_APPLIED_TO_TYPES_1_AND_2,
                    &[operator_string.to_string(), left_str, right_str],
                );
            }
        }
    }

    pub fn report_operator_error_unless(
        &mut self,
        left_type: &Arc<Type>,
        operator: SyntaxKind,
        right_type: &Arc<Type>,
        error_node: &Arc<Node>,
        types_are_compatible: &mut dyn FnMut(&Checker, &Arc<Type>, &Arc<Type>) -> bool,
    ) {
        if !types_are_compatible(self, left_type, right_type) {
            self.report_operator_error(left_type, operator, right_type, error_node, None);
        }
    }

    pub fn report_unmeasurable_worker(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let marker_super_type = self.marker_super_type();
        let marker_sub_type = self.marker_sub_type();
        let marker_other_type = self.marker_other_type();
        if Arc::ptr_eq(t, &marker_super_type)
            || Arc::ptr_eq(t, &marker_sub_type)
            || Arc::ptr_eq(t, &marker_other_type)
        {
            self.reliability_flags |=
                crate::checker::relater_relation::RelationComparisonResult::ReportsUnmeasurable.bits()
                    as u8;
        }
        Arc::clone(t)
    }

    pub fn report_unused_variable(
        &mut self,
        location: &Arc<Node>,
        diagnostic: Arc<Diagnostic>,
    ) {
        let mut location = Arc::clone(location);
        while tsox_frontend::ast::is_binding_element(&location)
            || tsox_frontend::ast::is_binding_pattern(&location)
        {
            location = location.parent().unwrap();
        }
        let unused_kind = if tsox_frontend::ast::is_parameter_declaration(&location) {
            crate::checker::mig::m2c::r18k3_defs::UnusedKind::Parameter
        } else {
            crate::checker::mig::m2c::r18k3_defs::UnusedKind::Local
        };
        let is_parameter =
            unused_kind == crate::checker::mig::m2c::r18k3_defs::UnusedKind::Parameter;
        let diagnostic = Arc::try_unwrap(diagnostic).unwrap_or_else(|d| (*d).clone());
        let loc = diagnostic.loc;
        let args = diagnostic.message_args;
        let message: &'static tsox_core::diagnostics::Message = Box::leak(Box::new(
            diagnostic.message.expect("unused diagnostic carries message"),
        ));
        self.report_unused(&location, is_parameter, loc, message, args);
    }

    pub fn report_widening_errors_in_type(&mut self, t: &Arc<Type>) -> bool {
        let mut error_reported = false;
        if t
            .object_flags
            .intersects(ObjectFlags::ContainsWideningType)
        {
            if t.flags.intersects(TypeFlags::Union) {
                let empty_object_type = self.empty_object_type();
                if t.types().unwrap().iter().any(|s| Arc::ptr_eq(s, &empty_object_type)) {
                    error_reported = true;
                } else {
                    for s in t.types().unwrap() {
                        error_reported = self.report_widening_errors_in_type(s) || error_reported;
                    }
                }
            } else if self.is_array_or_tuple_type(t) {
                for s in self.get_type_arguments(t) {
                    error_reported = self.report_widening_errors_in_type(&s) || error_reported;
                }
            } else if is_object_literal_type(t) {
                for p in self.get_properties_of_object_type(t) {
                    let s = self.get_type_of_symbol(&p);
                    if s
                        .object_flags
                        .intersects(ObjectFlags::ContainsWideningType)
                    {
                        error_reported = self.report_widening_errors_in_type(&s);
                        if !error_reported {
                            let value_declaration = p.declarations.iter().find(|d| {
                                self.symbol_of_node(d)
                                    .and_then(|d_symbol| d_symbol.value_declaration.clone())
                                    .is_some_and(|vd| {
                                        vd.parent()
                                            .as_ref()
                                            .zip(
                                                t.symbol
                                                    .as_ref()
                                                    .and_then(|s| s.value_declaration.as_ref()),
                                            )
                                            .is_some_and(|(a, b)| Arc::ptr_eq(a, b))
                                    })
                            });
                            if let Some(value_declaration) = value_declaration {
                                let property_string = self.symbol_to_string(&p);
                                let widened = self.get_widened_type(&s);
                                let type_string = self.type_to_string(&widened);
                                self.error_message(
                                    value_declaration,tsox_core::diagnostics::messages_generated::OBJECT_LITERAL_S_PROPERTY_0_IMPLICITLY_HAS_AN_1_TYPE,
                                    &[property_string, type_string],
                                );
                                error_reported = true;
                            }
                        }
                    }
                }
            }
        }
        error_reported
    }
}
