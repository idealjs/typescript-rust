#![allow(unused_imports)]

use crate::checker::checker_checker::*;
use crate::checker::mig::m3a_2::new_diagnostic_chain_for_node;
use crate::checker::mig::wc3::NodeAccessExt;
use crate::checker::relater_relation::Relation;
use crate::checker::utilities_token_is_identifier_or_keyword::{is_type_usable_as_property_name, get_property_name_from_type};
use tsox_frontend::ast::INTERNAL_SYMBOL_NAME_COMPUTED;
use std::sync::Arc;
use tsox_core::diagnostics as msg;
use tsox_frontend::ast::{self, Node, Symbol, SyntaxKind};

impl Checker {
    pub fn is_signature_applicable(
        &mut self,
        node: &Arc<Node>,
        args: &[Arc<Node>],
        signature: &Arc<Signature>,
        relation: &Relation,
        check_mode: CheckMode,
        report_errors: bool,
        diagnostic_output: &mut Vec<ast::Diagnostic>,
    ) -> bool { ::tsox_core::fntrace::enter("is_signature_applicable"); 
        if ast::mig::m3g::is_jsx_call_like(node) {
            return self.check_applicable_signature_for_jsx_call_like_element(
                node,
                signature,
                relation.kind,
                check_mode.bits(),
                report_errors,
                Some(diagnostic_output),
            );
        }
        let this_type = self.get_this_type_of_signature(signature);
        let skip_this_check = this_type.is_none()
            || Arc::ptr_eq(this_type.as_ref().unwrap(), &self.void_type())
            || ast::is_new_expression(node)
            || (ast::is_call_expression(node)
                && ast::mig::m3g_2::is_super_property(&node.expression().unwrap()));
        if !skip_this_check {
            let this_type = this_type.unwrap();
            let this_argument_node = self.get_this_argument_of_call(node);
            let this_argument_type =
                self.get_this_argument_type(this_argument_node.as_deref().unwrap_or(node.as_ref()));
            let error_node = if report_errors {
                this_argument_node.clone().or_else(|| Some(Arc::clone(node)))
            } else {
                None
            };
            let head_message = msg::THE_THIS_CONTEXT_OF_TYPE_0_IS_NOT_ASSIGNABLE_TO_METHOD_S_THIS_OF_TYPE_1;
            if !self.check_type_related_to_ex(
                &this_argument_type,
                &this_type,
                relation.kind,
                error_node.as_ref(),
                Some(&head_message),
                Some(diagnostic_output),
            ) {
                return false;
            }
        }
        let head_message = msg::ARGUMENT_OF_TYPE_0_IS_NOT_ASSIGNABLE_TO_PARAMETER_OF_TYPE_1;
        let rest_type = self.get_non_array_rest_type(signature);
        let arg_count = if rest_type.is_some() {
            (self.get_parameter_count(signature) - 1).min(args.len())
        } else {
            args.len()
        };
        for i in 0..arg_count {
            let arg = &args[i];
            if !ast::is_omitted_expression(arg) {
                let param_type = self.get_type_at_position(signature, i);
                let arg_type = self.check_expression_with_contextual_type(
                    arg,
                    &param_type,
                    None,
                    check_mode,
                );
                let check_arg_type = if check_mode.intersects(CheckMode::SkipContextSensitive) {
                    self.get_regular_type_of_object_literal(&arg_type)
                } else {
                    arg_type.clone()
                };
                let effective_check_argument_node = self.get_effective_check_node(arg);
                if !self.check_type_related_to_and_optionally_elaborate(
                    &check_arg_type,
                    &param_type,
                    relation.kind,
                    if report_errors {
                        Some(&effective_check_argument_node)
                    } else {
                        None
                    },
                    Some(&effective_check_argument_node),
                    Some(&head_message),
                    Some(diagnostic_output),
                ) {
                    self.maybe_add_missing_await_info(
                        Some(arg),
                        &check_arg_type,
                        &param_type,
                        relation,
                        report_errors,
                        Some(diagnostic_output),
                    );
                    return false;
                }
            }
        }
        if let Some(rest_type) = rest_type {
            let spread_type = self.get_spread_argument_type(args, arg_count, args.len());
            let rest_arg_count = args.len() - arg_count;
            let mut error_node: Option<Arc<Node>> = None;
            if report_errors {
                match rest_arg_count {
                    0 => error_node = Some(Arc::clone(node)),
                    1 => error_node = Some(self.get_effective_check_node(&args[arg_count])),
                    _ => {
                        let mut synthetic =
                            self.create_synthetic_expression(node, &spread_type, false, None);
                        Arc::get_mut(&mut synthetic).unwrap().loc = tsox_core::core::text::TextRange::new(
                            args[arg_count].pos(),
                            args[args.len() - 1].end(),
                        );
                        error_node = Some(synthetic);
                    }
                }
            }
            if !self.check_type_related_to_ex(
                &spread_type,
                &rest_type,
                relation.kind,
                error_node.as_ref(),
                Some(&head_message),
                Some(diagnostic_output),
            ) {
                self.maybe_add_missing_await_info(
                    error_node.as_ref(),
                    &spread_type,
                    &rest_type,
                    relation,
                    report_errors,
                    Some(diagnostic_output),
                );
                return false;
            }
        }
        true
    }

    pub fn issue_member_specific_error(
        &mut self,
        node: &Arc<Node>,
        type_with_this: &Arc<Type>,
        base_with_this: &Arc<Type>,
        broad_diag: msg::Message,
    ) { ::tsox_core::fntrace::enter("issue_member_specific_error"); 
        let mut issued_member_error = false;
        for member in ast::mig::m3b::members(node) {
            if ast::is_static(&member) {
                continue;
            }
            let declared_prop = self.get_symbol_of_declaration(&member);
            if let Some(declared_prop) = declared_prop {
                if declared_prop.name != INTERNAL_SYMBOL_NAME_COMPUTED {
                    let prop = self.get_property_of_type(type_with_this, &declared_prop.name);
                    let base_prop = self.get_property_of_type(base_with_this, &declared_prop.name);
                    if let (Some(prop), Some(base_prop)) = (prop, base_prop) {
                        let mut diags: Vec<ast::Diagnostic> = Vec::new();
                        let error_node = ast::get_name_of_declaration(member).unwrap_or_else(|| Arc::clone(member));
                        let prop_type = self.get_type_of_symbol(&prop);
                        let base_prop_type = self.get_type_of_symbol(&base_prop);
                        if !self.check_type_assignable_to_ex(
                            &prop_type,
                            &base_prop_type,
                            Some(&error_node),
                            None,
                            Some(&mut diags),
                        ) {
                            let declared_prop_str = self.symbol_to_string(&declared_prop);
                            let type_with_this_str = self.type_to_string(type_with_this);
                            let base_with_this_str = self.type_to_string(base_with_this);
                            self.add_diagnostic(new_diagnostic_chain_for_node(
                                Some(&diags[0]),
                                None,
                                msg::PROPERTY_0_IN_TYPE_1_IS_NOT_ASSIGNABLE_TO_THE_SAME_PROPERTY_IN_BASE_TYPE_2,
                                vec![
                                    declared_prop_str,
                                    type_with_this_str,
                                    base_with_this_str,
                                ],
                            ));
                            issued_member_error = true;
                        }
                    }
                }
            }
        }
        if !issued_member_error {
            let name_node = ast::get_name_of_declaration(node).unwrap_or_else(|| Arc::clone(node));
            self.check_type_assignable_to(type_with_this, base_with_this, Some(&name_node), Some(&broad_diag));
        }
    }

    pub fn late_bind_member(
        &mut self,
        parent: &Arc<Symbol>,
        early_symbols: &mut SymbolTable,
        late_symbols: &mut SymbolTable,
        decl: &Arc<Node>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("late_bind_member"); 
        if self
            .symbol_node_links
            .get(decl)
            .and_then(|l| l.resolved_symbol.clone())
            .is_none()
        {
            let decl_symbol = self
                .get_symbol_of_declaration(decl)
                .expect("The member is expected to have a symbol.");
            self.symbol_node_links.get_or_default(decl).resolved_symbol =
                Some(Arc::clone(&decl_symbol));
            let decl_name = if ast::is_binary_expression(decl) {
                decl.as_binary_expression().left.clone()
            } else {
                ast::get_name_of_declaration(decl).unwrap()
            };
            let t = if ast::is_element_access_expression(&decl_name) {
                self.check_expression_cached(&decl_name.as_element_access_expression().argument_expression)
            } else {
                self.check_computed_property_name_type(&decl_name)
            };
            if is_type_usable_as_property_name(&t) {
                let member_name = get_property_name_from_type(&t);
                let symbol_flags = decl_symbol.flags;
                let mut late_symbol = match late_symbols.get(&member_name) {
                    Some(s) => Arc::clone(s),
                    None => {
                        let s = self.new_symbol_ex(SymbolFlags::empty(), &member_name, CheckFlags::Late);
                        late_symbols.insert(member_name.clone(), Arc::clone(&s));
                        s
                    }
                };
                let early_symbol = early_symbols.get(&member_name).cloned();
                if late_symbol
                    .flags
                    .intersects(get_excluded_symbol_flags(symbol_flags))
                {
                    let declarations: Vec<Arc<Node>> = match &early_symbol {
                        Some(early) => early
                            .declarations
                            .iter()
                            .chain(late_symbol.declarations.iter())
                            .cloned()
                            .collect(),
                        None => late_symbol.declarations.clone(),
                    };
                    let mut name = member_name.clone();
                    if t.flags.contains(TypeFlags::UniqueESSymbol) {
                        name = tsox_frontend::scanner::mig::m3i::declaration_name_to_string(Some(&decl_name));
                    }
                    for d in &declarations {
                        self.error_message(
                            &ast::get_name_of_declaration(d)
                                .unwrap_or_else(|| Arc::clone(d)),msg::DUPLICATE_IDENTIFIER_0,
                            &[name.clone()],
                        );
                    }
                    self.error_message(
                        decl_name.parent().as_ref().and(Some(&decl_name)).unwrap_or(decl),msg::DUPLICATE_IDENTIFIER_0,
                        &[name.clone()],
                    );
                    if late_symbol.flags.intersects(SymbolFlags::ACCESSOR)
                        && late_symbol.flags.intersects(SymbolFlags::ACCESSOR)
                            != symbol_flags.intersects(SymbolFlags::ACCESSOR)
                    {
                        let mut late_mut = Arc::clone(&late_symbol);
                        Arc::get_mut(&mut late_mut)
                            .unwrap()
                            .flags
                            .insert(SymbolFlags::ACCESSOR);
                    }
                    late_symbol =
                        self.new_symbol_ex(SymbolFlags::empty(), &member_name, CheckFlags::Late);
                    late_symbols.insert(member_name.clone(), Arc::clone(&late_symbol));
                }
                self.value_symbol_links
                    .get_or_default(&late_symbol)
                    .name_type = Some(Arc::clone(&t));
                self.add_declaration_to_late_bound_symbol(&late_symbol, decl, symbol_flags);
                if late_symbol.parent().is_none() {
                    late_symbol.set_parent(parent);
                }
                self.symbol_node_links.get_or_default(decl).resolved_symbol =
                    Some(late_symbol);
            }
        }
        self.symbol_node_links
            .get(decl)
            .and_then(|l| l.resolved_symbol.clone())
    }

    pub fn mark_decorator_alias_referenced(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("mark_decorator_alias_referenced"); 
        if self.compiler_options.emit_decorator_metadata.is_false_or_unknown() {
            return;
        }
        let first_decorator = node
            .modifiers()
            .and_then(|ml| {
                ml.list
                    .nodes
                    .iter()
                    .find(|m| m.kind == SyntaxKind::Decorator)
                    .cloned()
            });
        let Some(first_decorator) = first_decorator else {
            return;
        };
        self.check_external_emit_helpers(&first_decorator, ExternalEmitHelpers::Metadata.bits());
        match node.kind {
            SyntaxKind::ClassDeclaration => {
                if let Some(ctor) = ast::mig::x4ast::get_first_constructor_with_body(node) {
                    if let Some(params) = tsox_frontend::ast::mig::m3f_2::node_parameters(&ctor) {
                        for p in &params.nodes {
                            let type_node = self.get_parameter_type_node_for_decorator_check(p);
                            if let Some(type_node) = type_node {
                                self.mark_decorator_medata_data_type_node_as_referenced(&type_node);
                            }
                        }
                    }
                }
            }
            SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
                let other_kind = if node.kind == SyntaxKind::SetAccessor {
                    SyntaxKind::GetAccessor
                } else {
                    SyntaxKind::SetAccessor
                };
                let symbol = self.get_symbol_of_declaration(node);
                let other_accessor = symbol
                    .as_ref()
                    .and_then(|s| tsox_frontend::ast::mig::m3e_4::get_declaration_of_kind(s, other_kind));
                let mut annotation = self.get_annotated_accessor_type_node(node);
                if annotation.is_none() {
                    if let Some(other_accessor) = other_accessor {
                        annotation = self.get_annotated_accessor_type_node(&other_accessor);
                    }
                }
                if let Some(annotation) = annotation {
                    self.mark_decorator_medata_data_type_node_as_referenced(&annotation);
                }
            }
            SyntaxKind::MethodDeclaration => {
                if let Some(params) = tsox_frontend::ast::mig::m3f_2::node_parameters(node) {
                    for p in &params.nodes {
                        let type_node = self.get_parameter_type_node_for_decorator_check(p);
                        if let Some(type_node) = type_node {
                            self.mark_decorator_medata_data_type_node_as_referenced(&type_node);
                        }
                    }
                }
                if let Some(typ) = node.typ() {
                    self.mark_decorator_medata_data_type_node_as_referenced(&typ);
                }
            }
            SyntaxKind::PropertyDeclaration => {
                if let Some(typ) = node.typ() {
                    self.mark_decorator_medata_data_type_node_as_referenced(&typ);
                }
            }
            SyntaxKind::Parameter => {
                let type_node = self.get_parameter_type_node_for_decorator_check(node);
                if let Some(type_node) = type_node {
                    self.mark_decorator_medata_data_type_node_as_referenced(&type_node);
                }
                let containing_signature = node.parent().unwrap();
                if let Some(params) = tsox_frontend::ast::mig::m3f_2::node_parameters(&containing_signature)
                {
                    for p in &params.nodes {
                        let type_node = self.get_parameter_type_node_for_decorator_check(p);
                        if let Some(type_node) = type_node {
                            self.mark_decorator_medata_data_type_node_as_referenced(&type_node);
                        }
                    }
                }
                if let Some(typ) = containing_signature.typ() {
                    self.mark_decorator_medata_data_type_node_as_referenced(&typ);
                }
            }
            _ => {}
        }
    }

    pub fn report_implicit_any(
        &mut self,
        declaration: &Arc<Node>,
        t: &Arc<Type>,
        widening_kind: WideningKind,
    ) { ::tsox_core::fntrace::enter("report_implicit_any"); 
        if ast::is_in_js_file(declaration)
            && !self
                .get_source_file_of_node(declaration)
                .is_some_and(|file| {
                    ast::mig::m3f_4::is_check_js_enabled_for_file(&file, &self.compiler_options)
                })
        {
            return;
        }
        let widened = self.get_widened_type(t);
        let type_as_string = self.type_to_string(&widened);
        let mut diagnostic: Option<msg::Message> = None;
        match declaration.kind {
            SyntaxKind::BinaryExpression
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature => {
                diagnostic = Some(if self.no_implicit_any {
                    msg::MEMBER_0_IMPLICITLY_HAS_AN_1_TYPE
                } else {
                    msg::MEMBER_0_IMPLICITLY_HAS_AN_1_TYPE_BUT_A_BETTER_TYPE_MAY_BE_INFERRED_FROM_USAGE
                });
            }
            SyntaxKind::Parameter => {
                let param_name = ast::get_name_of_declaration(declaration).unwrap();
                if ast::is_identifier(&param_name) {
                    let name_text = param_name.text();
                    let original_keyword_kind =
                        tsox_frontend::scanner::mig::m3i::identifier_to_keyword_kind(&param_name);
                    let parent_is_signature = ast::is_call_signature_declaration(declaration.parent().as_ref().unwrap())
                        || ast::is_method_signature_declaration(declaration.parent().as_ref().unwrap())
                        || ast::is_function_type_node(declaration.parent().as_ref().unwrap());
                    let parameters =
                        tsox_frontend::ast::mig::m3f_2::node_parameters(&declaration.parent().unwrap())
                            .map(|pl| pl.nodes.clone())
                            .unwrap_or_default();
                    let is_in_parent_parameters = parameters
                        .iter()
                        .any(|p| Arc::ptr_eq(p, declaration));
                    if parent_is_signature
                        && is_in_parent_parameters
                        && (tsox_frontend::ast::utilities::is_type_node_kind(original_keyword_kind)
                            || self
                                .resolve_name(&name_text, declaration, SymbolFlags::TYPE, false)
                                .is_some())
                    {
                        let index = parameters
                            .iter()
                            .position(|p| Arc::ptr_eq(p, declaration))
                            .unwrap();
                        let new_name = format!("arg{}", index);
                        let dot_dot_dot = declaration.as_parameter_declaration().dot_dot_dot_token.is_some();
                        let type_name = format!(
                            "{}{}",
                            tsox_frontend::scanner::mig::m3i::declaration_name_to_string(Some(&param_name)),
                            if dot_dot_dot { "[]" } else { "" }
                        );
                        self.error_or_suggestion_message(
                            self.no_implicit_any,
                            declaration,msg::PARAMETER_HAS_A_NAME_BUT_NO_TYPE_DID_YOU_MEAN_0_COLON_1,
                            &[new_name, type_name],
                        );
                        return;
                    }
                }
                let dot_dot_dot = declaration.as_parameter_declaration().dot_dot_dot_token.is_some();
                if dot_dot_dot {
                    diagnostic = Some(if self.no_implicit_any {
                        msg::REST_PARAMETER_0_IMPLICITLY_HAS_AN_ANY_TYPE
                    } else {
                        msg::REST_PARAMETER_0_IMPLICITLY_HAS_AN_ANY_TYPE_BUT_A_BETTER_TYPE_MAY_BE_INFERRED_FROM_USAGE
                    });
                } else if self.no_implicit_any {
                    diagnostic = Some(msg::PARAMETER_0_IMPLICITLY_HAS_AN_1_TYPE);
                } else {
                    diagnostic = Some(
                        msg::PARAMETER_0_IMPLICITLY_HAS_AN_1_TYPE_BUT_A_BETTER_TYPE_MAY_BE_INFERRED_FROM_USAGE,
                    );
                }
            }
            SyntaxKind::BindingElement => {
                diagnostic = Some(msg::BINDING_ELEMENT_0_IMPLICITLY_HAS_AN_1_TYPE);
                if !self.no_implicit_any {
                    return;
                }
            }
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction => {
                if self.no_implicit_any && ast::get_name_of_declaration(declaration).is_none() {
                    if widening_kind == WideningKind::GeneratorYield {
                        self.error_message(
                            declaration,msg::GENERATOR_IMPLICITLY_HAS_YIELD_TYPE_0_CONSIDER_SUPPLYING_A_RETURN_TYPE_ANNOTATION,
                            &[type_as_string.clone()],
                        );
                    } else {
                        self.error_message(
                            declaration,msg::FUNCTION_EXPRESSION_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_0_RETURN_TYPE,
                            &[type_as_string.clone()],
                        );
                    }
                    return;
                }
                if !self.no_implicit_any {
                    diagnostic = Some(
                        msg::X_0_IMPLICITLY_HAS_AN_1_RETURN_TYPE_BUT_A_BETTER_TYPE_MAY_BE_INFERRED_FROM_USAGE,
                    );
                } else if declaration.flags.contains(NodeFlags::Reparsed) {
                    let name = tsox_frontend::scanner::mig::m3i::declaration_name_to_string(
                        Some(&ast::get_name_of_declaration(declaration).unwrap()),
                    );
                    if !name.is_empty() {
                        self.error_message(
                            declaration,msg::X_0_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_1_RETURN_TYPE,
                            &[name, type_as_string.clone()],
                        );
                    } else {
                        self.error_message(
                            declaration,msg::THIS_OVERLOAD_IMPLICITLY_RETURNS_THE_TYPE_0_BECAUSE_IT_LACKS_A_RETURN_TYPE_ANNOTATION,
                            &[type_as_string.clone()],
                        );
                    }
                    return;
                } else if widening_kind == WideningKind::GeneratorYield {
                    diagnostic = Some(msg::X_0_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_1_YIELD_TYPE);
                } else {
                    diagnostic = Some(msg::X_0_WHICH_LACKS_RETURN_TYPE_ANNOTATION_IMPLICITLY_HAS_AN_1_RETURN_TYPE);
                }
            }
            SyntaxKind::MappedType => {
                if self.no_implicit_any {
                    self.error_message(
                        declaration,msg::MAPPED_OBJECT_TYPE_IMPLICITLY_HAS_AN_ANY_TEMPLATE_TYPE,
                        &[],
                    );
                }
                return;
            }
            _ => {
                diagnostic = Some(if self.no_implicit_any {
                    msg::VARIABLE_0_IMPLICITLY_HAS_AN_1_TYPE
                } else {
                    msg::VARIABLE_0_IMPLICITLY_HAS_AN_1_TYPE_BUT_A_BETTER_TYPE_MAY_BE_INFERRED_FROM_USAGE
                });
            }
        }
        let diagnostic = diagnostic.unwrap();
        let name = tsox_frontend::scanner::mig::m3i::declaration_name_to_string(
            Some(&ast::get_name_of_declaration(declaration).unwrap()),
        );
        self.error_or_suggestion_message(self.no_implicit_any, declaration, diagnostic, &[name, type_as_string]);
    }
}
