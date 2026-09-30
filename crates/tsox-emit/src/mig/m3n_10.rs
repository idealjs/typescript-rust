use super::m3n::*;
use super::m3n_7::r39k18_defs::{source_file_is_js, R39K18EmitResolverExt};
use super::m3n_7::DeclarationTransformer;
use super::m4e_2::{can_have_literal_initializer, can_produce_diagnostics, unwrap_parenthesized_expression};
use crate::mig::m3m_2::create_get_symbol_accessibility_diagnostic_for_node;
use crate::mig::wt1b_3::{
    declaration_emit_internal_node_builder_flags, declaration_emit_node_builder_flags,
};
use crate::printer::EmitContext;
use crate::mig::m4e::{R37K1DataExt, R38K1NodeVisitorExt, R42K01DataExt};
use super::m3n_7::r40k21_defs::R40K21NodeFactoryExt;
use std::sync::Arc;
use tsox_checker::checker::symboltracker::NodeBuilderFlags;
use tsox_frontend::ast::mig::m3f_2::has_inferred_type;
use tsox_frontend::ast::mig::m3g_2::is_primitive_literal_value;
use tsox_frontend::ast::node_data_generated::*;
use tsox_frontend::ast::node_flags::ModifierFlags;
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::visitor::NodeVisitor;
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::is_function_like;

pub fn parameter_dot_dot_dot_token(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::ParameterDeclaration(d) => d.dot_dot_dot_token.clone(),
        _ => None,
    }
}

impl DeclarationTransformer {
    pub fn emit_context(&self) -> EmitContext {
        self.transformer.emit_context()
    }

    pub fn visitor(&mut self) -> &mut NodeVisitor {
        self.transformer.visitor()
    }

    pub fn setup_diagnostic_context(
        &mut self,
        input: &Arc<Node>,
    ) -> (bool, Box<dyn FnOnce(&mut Self)>) {
        let can_produce_diagnostic = can_produce_diagnostics(input);
        let old_within_object_literal_type = self.suppress_new_diagnostic_contexts;
        let should_enter_suppress_new_diagnostics_context_context = (input.kind
            == SyntaxKind::TypeLiteral
            || input.kind == SyntaxKind::MappedType)
            && !matches!(
                input.parent(),
                Some(p) if p.kind == SyntaxKind::TypeAliasDeclaration || p.kind == SyntaxKind::JSTypeAliasDeclaration
            );

        let old_diag = self.state.get_symbol_accessibility_diagnostic.take();
        if can_produce_diagnostic && !self.suppress_new_diagnostic_contexts {
            self.state.get_symbol_accessibility_diagnostic =
                Some(create_get_symbol_accessibility_diagnostic_for_node(input));
        }
        let old_name = self.state.error_name_node.clone();

        if should_enter_suppress_new_diagnostics_context_context {
            self.suppress_new_diagnostic_contexts = true;
        }

        (
            can_produce_diagnostic,
            Box::new(move |tx: &mut Self| {
                tx.state.get_symbol_accessibility_diagnostic = old_diag;
                tx.state.error_name_node = old_name;
                tx.suppress_new_diagnostic_contexts = old_within_object_literal_type;
            }),
        )
    }

    pub fn ensure_type(&mut self, node: &Arc<Node>, ignore_private: bool) -> Option<Arc<Node>> {
        let parse_node = self.emit_context().parse_node(node);
        let is_private = parse_node.as_ref().is_some_and(|pn| {
            self.host
                .get_effective_declaration_flags(pn, ModifierFlags::Private)
                != ModifierFlags::empty()
        });
        if !ignore_private && is_private {
            return None;
        }
        if self.should_print_with_initializer(node) {
            return None;
        }
        let enclosing = self.enclosing_declaration.clone();
        if !is_export_assignment(node)
            && !is_binding_element(node)
            && node.type_node().is_some()
            && (!is_parameter_declaration(node)
                || !self
                    .resolver
                    .requires_adding_implicit_undefined_unsafe(node, None, enclosing.as_ref()))
        {
            if self
                .state
                .current_source_file
                .as_ref()
                .map(|f| source_file_is_js(f.script_kind))
                .unwrap_or(false)
            {
                let mut js_flags = declaration_emit_node_builder_flags();
                if self.in_class_expression_declaration {
                    js_flags.remove(NodeBuilderFlags::WriteClassExpressionAsTypeLiteral);
                }
                let res = self.resolver.try_js_type_node_to_type_node(
                    node.type_node().unwrap(),
                    enclosing.as_ref(),
                    js_flags,
                    declaration_emit_internal_node_builder_flags(),
                    &self.tracker,
                );
                if let Some(res) = res {
                    return Some(res);
                }
            } else {
                return Some(
                    self.visitor()
                        .visit_node(node.type_node().unwrap())
                        .clone(),
                );
            }
        }

        let old_error_name_node = self.state.error_name_node.clone();
        self.state.error_name_node = node.name().cloned();
        let mut old_diag: Option<GetSymbolAccessibilityDiagnostic> = None;
        if !self.suppress_new_diagnostic_contexts && can_produce_diagnostics(node) {
            old_diag = self.state.get_symbol_accessibility_diagnostic.take();
            self.state.get_symbol_accessibility_diagnostic =
                Some(create_get_symbol_accessibility_diagnostic_for_node(node));
        }
        let mut flags = declaration_emit_node_builder_flags();
        if self.in_class_expression_declaration {
            flags.remove(NodeBuilderFlags::WriteClassExpressionAsTypeLiteral);
        }
        let type_node = if has_inferred_type(node) {
            self.resolver.create_type_of_declaration(
                node,
                enclosing.as_ref(),
                flags,
                declaration_emit_internal_node_builder_flags(),
                &self.tracker,
            )
        } else if is_function_like(node) {
            self.resolver.create_return_type_of_signature_declaration(
                node,
                enclosing.as_ref(),
                flags,
                declaration_emit_internal_node_builder_flags(),
                &self.tracker,
            )
        } else {
            panic!("ensure_type: unexpected node kind {:?}", node.kind)
        };

        self.state.error_name_node = old_error_name_node;
        if !self.suppress_new_diagnostic_contexts && old_diag.is_some() {
            self.state.get_symbol_accessibility_diagnostic = old_diag;
        }
        if type_node.is_none() {
            return Some(self.factory().new_keyword_type_node(SyntaxKind::AnyKeyword));
        }
        type_node
    }

    pub fn should_print_with_initializer(&self, node: &Arc<Node>) -> bool {
        can_have_literal_initializer(&self.host, node)
            && node.initializer().is_some()
            && self
                .resolver
                .is_literal_const_declaration(&self.emit_context().most_original(node))
    }

    pub fn ensure_type_params(
        &mut self,
        node: &Arc<Node>,
        params: Option<&Arc<NodeList>>,
    ) -> Option<Arc<NodeList>> {
        let parse_node = self.emit_context().parse_node(node);
        let is_private = parse_node.as_ref().is_some_and(|pn| {
            self.host
                .get_effective_declaration_flags(pn, ModifierFlags::Private)
                != ModifierFlags::empty()
        });
        if is_private {
            return None;
        }
        let type_parameters = params.map(|list| self.visit_nodes_via_visit(list));
        if let Some(type_parameters) = type_parameters {
            return Some(type_parameters);
        }
        let old_error_name_node = self.state.error_name_node.clone();
        self.state.error_name_node = node.name().cloned();
        let old_diag = self.state.get_symbol_accessibility_diagnostic.take();
        if !self.suppress_new_diagnostic_contexts && can_produce_diagnostics(node) {
            self.state.get_symbol_accessibility_diagnostic =
                Some(create_get_symbol_accessibility_diagnostic_for_node(node));
        }
        let enclosing = self.enclosing_declaration.clone();
        let nodes = self.resolver.create_type_parameters_of_signature_declaration(
            node,
            enclosing.as_ref(),
            declaration_emit_node_builder_flags(),
            declaration_emit_internal_node_builder_flags(),
            &self.tracker,
        );
        if let Some(nodes) = nodes {
            let type_parameters = self.factory().new_node_list(nodes);
            self.state.error_name_node = old_error_name_node;
            self.state.get_symbol_accessibility_diagnostic = old_diag;
            return Some(type_parameters);
        }
        self.state.error_name_node = old_error_name_node;
        self.state.get_symbol_accessibility_diagnostic = old_diag;
        None
    }

    pub fn ensure_parameter(&mut self, p: &Arc<Node>) -> Arc<Node> {
        let old_diag = self.state.get_symbol_accessibility_diagnostic.take();
        if !self.suppress_new_diagnostic_contexts {
            self.state.get_symbol_accessibility_diagnostic =
                Some(create_get_symbol_accessibility_diagnostic_for_node(p));
        }
        let question_token = if self.resolver.is_optional_parameter(p) {
            match &p.data {
                NodeData::ParameterDeclaration(d) => d.question_token.clone(),
                _ => None,
            }
            .or_else(|| Some(self.factory().new_token(SyntaxKind::QuestionToken)))
        } else {
            None
        };
        let name = p.name().map(|n| self.binding_name_visitor.visit_node(n));
        let type_node = self.ensure_type(p, true);
        let initializer = self.ensure_no_initializer(p);
        let result = self.factory().update_parameter_declaration(
            p,
            None,
            parameter_dot_dot_dot_token(p).as_ref(),
            name.as_ref().expect("parameter must have a name"),
            question_token.as_ref(),
            type_node.as_ref(),
            initializer.as_ref(),
        );
        self.state.get_symbol_accessibility_diagnostic = old_diag;
        result
    }

    pub fn ensure_no_initializer(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if self.should_print_with_initializer(node) {
            let initializer = node.initializer();
            let unwrapped = initializer.map(unwrap_parenthesized_expression);
            if let Some(unwrapped) = unwrapped {
                if !is_primitive_literal_value(&unwrapped, true) {
                    self.tracker.report_inference_fallback(node);
                }
            }
            let parse_node = self.emit_context().parse_node(node);
            return self
                .resolver
                .create_literal_const_value(&parse_node, &self.tracker);
        }
        None
    }

    pub fn update_param_list(&mut self, node: &Arc<Node>, params: &Arc<NodeList>) -> Arc<NodeList> {
        let parse_node = self.emit_context().parse_node(node).expect("parse node");
        if self
            .host
            .get_effective_declaration_flags(&parse_node, ModifierFlags::Private)
            != ModifierFlags::empty()
            || params.nodes.is_empty()
        {
            return self.factory().new_node_list(Vec::new());
        }
        let results: Vec<Arc<Node>> = params
            .nodes
            .iter()
            .map(|p| self.ensure_parameter(p))
            .collect();
        self.factory().new_node_list(results)
    }

    pub fn transform_function_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let fd = input.as_function_declaration_r42k01();
        let type_parameters = fd
            .type_parameters
            .clone()
            .and_then(|tp| self.ensure_type_params(input, Some(&tp)));
        let params = fd.parameters.clone();
        let new_params = self.update_param_list(input, &params);
        let ensured_type = self.ensure_type(input, false);
        Some(self.factory().update_function_declaration(
            input,
            self.ensure_modifiers(input),
            None,
            fd.name.as_ref(),
            type_parameters,
            &new_params,
            ensured_type,
            None,
            None,
        ))
    }

    pub fn transform_class_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let previous_enclosing_declaration = self.enclosing_declaration.clone();
        self.enclosing_declaration = Some(input.clone());
        let old_name = self.state.error_name_node.clone();
        self.state.error_name_node = input.name().cloned();

        let cd = input.as_class_declaration_r42k01();
        if let Some(clauses) = &cd.heritage_clauses {
            for clause in &clauses.nodes {
                let hc = clause.as_heritage_clause();
                if hc.token != SyntaxKind::ExtendsKeyword {
                    continue;
                }
                for t in &hc.types.nodes {
                    let Some(expr) = t.expression() else {
                        continue;
                    };
                    if !tsox_frontend::ast::is_entity_name_expression(expr)
                        && expr.kind != SyntaxKind::NullKeyword
                    {
                        self.tracker.report_inference_fallback(expr);
                    }
                }
            }
        }
        let type_parameters = cd
            .type_parameters
            .clone()
            .and_then(|tp| self.ensure_type_params(input, Some(&tp)));
        let heritage_clauses = cd
            .heritage_clauses
            .clone()
            .map(|h| self.visit_nodes_via_visit(&h));
        let members = self.visit_nodes_via_visit(&cd.members);
        let result = self.factory().update_class_declaration(
            input,
            self.ensure_modifiers(input),
            cd.name.as_ref(),
            type_parameters,
            heritage_clauses.as_deref(),
            &members,
        );

        self.state.error_name_node = old_name;
        self.enclosing_declaration = previous_enclosing_declaration;
        Some(result)
    }

    pub fn transform_enum_declaration(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> {
        let ed = input.as_enum_declaration_r42k01();
        let mut kept: Vec<Arc<Node>> = Vec::with_capacity(ed.members.nodes.len());
        for m in &ed.members.nodes {
            if self.should_strip_internal(Some(m)) {
                continue;
            }
            let enum_value = self.resolver.get_enum_member_value(m);
            let computed_name = m
                .name()
                .map(|n| is_computed_property_name(&n))
                .unwrap_or(false);
            if self.tracker.state.isolated_declarations
                && m.initializer().is_some()
                && enum_value.has_external_references
                && !computed_name
            {
                let diag = super::m3n_5::create_diagnostic_for_node(
                    m,
                    Some(
                        &tsox_core::diagnostics::messages_generated::ENUM_MEMBER_INITIALIZERS_MUST_BE_COMPUTABLE_WITHOUT_REFERENCES_TO_EXTERNAL_SYMBOLS_WITH_ISOLATEDDECLARATIONS,
                    ),
                    &[],
                );
                self.tracker.state.add_diagnostic(diag);
            }
            kept.push(Arc::clone(m));
        }
        let members = self.factory().new_node_list(kept);
        Some(self.factory().update_enum_declaration_r42k01(
            input,
            self.ensure_modifiers(input),
            input.name().unwrap(),
            members,
        ))
    }
}
