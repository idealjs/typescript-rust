#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_checker::checker::symboltracker::{NodeBuilderFlags, NodeBuilderInternalFlags};
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::deep_clone_node;
use tsox_frontend::ast::mig::m3f::get_node_id;
use super::m3m_2::create_get_symbol_accessibility_diagnostic_for_node;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::{
    is_enum_declaration, is_identifier, is_numeric_literal, is_private_identifier,
    is_omitted_expression, is_variable_declaration, NodeData,
};
use tsox_frontend::ast::utilities::is_string_literal_like;
use crate::mig::m4j::r36k3_defs::R36K3NodeFactoryExt;
use crate::mig::m4m_2::is_simple_inlineable_expression;
use tsox_frontend::ast::mig::x4ast::get_first_constructor_with_body;
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::node_source_file::LanguageVariant;
use tsox_frontend::ast::utilities::{has_syntactic_modifier, is_static};
use tsox_frontend::ast::{ModifierList, NodeList};
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::scanner::mig::m3i::{identifier_to_keyword_kind, is_identifier_text};
use tsox_frontend::scanner::TokenFlags;

const TOKEN_FLAGS_NONE_R37K18: u32 = 0;

#[path = "r36k19_defs.rs"]
pub mod r36k19_defs;

use crate::mig::m3n_5::r33k8_defs::{syntax_list_children, NodeId};
use crate::mig::m3n_7::{get_this_property_assignment_key, DeclarationTransformer};
use crate::mig::m4j_2::extract_modifiers;
use crate::mig::m4k::CommonJSModuleTransformer;
use crate::mig::m4q::r33k12_defs::RuntimeSyntaxTransformer;
use crate::printer::{AutoGenerateOptions, EmitContext, GeneratedIdentifierFlags, NodeFactory};
use crate::mig::m4e::{R37K1DataExt, R39K01EmitResolverExt, R42K01DataExt};
use crate::mig::m4e_2::{is_declaration_and_not_visible, should_emit_function_properties};
use tsox_frontend::ast::mig::w5::get_leftmost_access_expression;
use tsox_frontend::ast::{
    is_declaration, is_element_access_expression, is_function_declaration, is_function_like,
    is_property_access_expression,
};

pub fn declaration_emit_node_builder_flags() -> NodeBuilderFlags {
    NodeBuilderFlags::MultilineObjectLiterals
        | NodeBuilderFlags::WriteClassExpressionAsTypeLiteral
        | NodeBuilderFlags::UseTypeOfFunction
        | NodeBuilderFlags::UseStructuralFallback
        | NodeBuilderFlags::AllowEmptyTuple
        | NodeBuilderFlags::GenerateNamesForShadowedTypeParams
        | NodeBuilderFlags::NoTruncation
}

pub fn declaration_emit_internal_node_builder_flags() -> NodeBuilderInternalFlags {
    NodeBuilderInternalFlags::AllowUnresolvedNames
}

fn deep_clone_modifier_list(modifiers: &Arc<ModifierList>) -> ModifierList {
    ModifierList::new(
        modifiers.list.nodes.iter().map(|n| deep_clone_node(n)).collect(),
        modifiers.modifier_flags,
    )
}

fn class_like_members(class_node: &Arc<Node>) -> &Arc<NodeList> {
    match &class_node.data {
        NodeData::ClassDeclaration(d) => &d.members,
        NodeData::ClassExpression(d) => &d.members,
        _ => panic!("class-like members on wrong node kind"),
    }
}

fn function_type_parameters(node: &Arc<Node>) -> Option<Arc<NodeList>> {
    match &node.data {
        NodeData::FunctionExpression(d) => d.type_parameters.clone(),
        NodeData::ArrowFunction(d) => d.type_parameters.clone(),
        _ => None,
    }
}

fn function_parameters(node: &Arc<Node>) -> Option<Arc<NodeList>> {
    match &node.data {
        NodeData::FunctionExpression(d) => Some(d.parameters.clone()),
        NodeData::ArrowFunction(d) => Some(d.parameters.clone()),
        _ => None,
    }
}

fn function_asterisk_token(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::FunctionExpression(d) => d.asterisk_token.clone(),
        _ => None,
    }
}

impl RuntimeSyntaxTransformer {
    pub fn get_enum_qualified_element(&mut self, enum_: &Arc<Node>, member: &Arc<Node>) -> Arc<Node> {
        let ns_name = self.get_namespace_container_name(enum_);
        let prop_name = self.get_expression_for_property_name(member);
        let prop = self.get_namespace_qualified_element(ns_name, prop_name);
        self.emit_context().add_emit_flags(
            &prop,
            EmitFlags::NO_COMMENTS
                | EmitFlags::NO_NESTED_COMMENTS
                | EmitFlags::NO_SOURCE_MAP
                | EmitFlags::NO_NESTED_SOURCE_MAPS,
        );
        prop
    }

    pub fn get_export_qualified_reference_to_declaration(&mut self, node: &Arc<Node>) -> Arc<Node> {
        if self.is_export_of_namespace(node) {
            return self.factory().get_external_module_or_namespace_export_name(
                Some(&self.get_namespace_container_name(self.current_namespace.as_ref().unwrap())),
                node,
                false,
                true,
            );
        }
        self.factory().get_declaration_name_ex(
            node,
            super::m4n_4::NameOptions {
                allow_comments: false,
                allow_source_maps: true,
            },
        )
    }

    pub(crate) fn record_declaration_in_scope(&mut self, node: &Arc<Node>) {
        match &node.data {
            NodeData::VariableStatement(d) => {
                let declaration_list = d.declaration_list.clone();
                self.record_declaration_in_scope(&declaration_list);
                return;
            }
            NodeData::VariableDeclarationList(d) => {
                for decl in d.declarations.nodes.iter() {
                    self.record_declaration_in_scope(decl);
                }
                return;
            }
            NodeData::BindingPattern(d) => {
                for element in d.elements.nodes.iter() {
                    self.record_declaration_in_scope(element);
                }
                return;
            }
            _ => {}
        }
        if let Some(name) = node.name() {
            if is_identifier(name) {
                let text = name.text().to_string();
                self.current_scope_first_declarations_of_name
                    .entry(text)
                    .or_insert_with(|| Arc::clone(node));
            } else if tsox_frontend::ast::node_data_generated::is_binding_pattern(name) {
                self.record_declaration_in_scope(name);
            }
        }
    }

    pub(crate) fn get_expression_for_property_name(&mut self, member: &Arc<Node>) -> Arc<Node> {
        let name = member.name().cloned().unwrap_or_else(|| member.clone());
        match name.kind {
            SyntaxKind::PrivateIdentifier => self.factory().new_identifier(""),
            SyntaxKind::ComputedPropertyName => {
                let expression = match &name.data {
                    NodeData::ComputedPropertyName(d) => d.expression.clone(),
                    _ => name.clone(),
                };
                self.transformer.visitor().visit_node(&expression)
            }
            SyntaxKind::Identifier | SyntaxKind::StringLiteral => {
                self.factory().new_string_literal(name.text(), TOKEN_FLAGS_NONE_R37K18)
            }
            SyntaxKind::NumericLiteral => {
                self.factory().new_numeric_literal(name.text(), TOKEN_FLAGS_NONE_R37K18)
            }
            _ => name.clone(),
        }
    }

    pub fn add_var_for_declaration(
        &mut self,
        statements: Vec<Arc<Node>>,
        node: &Arc<Node>,
    ) -> (Vec<Arc<Node>>, bool) {
        self.record_declaration_in_scope(node);
        if !self.is_first_declaration_in_scope(node) {
            return (statements, false);
        }

        let name = self.factory().get_local_name_ex(
            node,
            super::m4n_4::AssignedNameOptions {
                allow_comments: false,
                allow_source_maps: true,
                ignore_assigned_name: false,
            },
        );
        let var_decl = self.factory().new_variable_declaration(&name, None, None, None);
        let var_flags = if Arc::ptr_eq(
            self.current_scope.as_ref().unwrap(),
            self.current_source_file.as_ref().unwrap(),
        ) {
            NodeFlags::empty()
        } else {
            NodeFlags::Let
        };
        let var_decls = self.factory().new_variable_declaration_list(
            &self.factory().new_node_list(vec![var_decl]),
            var_flags,
        );
        let mut modifier_mask = !(ModifierFlags::Modifier | ModifierFlags::Decorator);
        if self.current_namespace.is_some() {
            modifier_mask &= !ModifierFlags::Export;
        }
        let modifiers = extract_modifiers(
            &Arc::new(self.transformer.emit_context()),
            node.modifiers().map(|m| &**m),
            modifier_mask,
        )
        .map(Arc::new);
        let var_statement = self.factory().new_variable_statement(modifiers, &var_decls);

        self.emit_context().set_original(&var_decls, node);
        self.emit_context().set_original(&var_statement, node);

        if is_enum_declaration(node) {
            self.emit_context()
                .set_source_map_range(&var_decls, node.loc);
        } else {
            self.emit_context()
                .set_source_map_range(&var_statement, node.loc);
        }

        let mut statements = statements;
        statements.push(var_statement);
        (statements, true)
    }

    pub fn create_namespace_export_expression(
        &mut self,
        export_name: &Arc<Node>,
        export_value: &Arc<Node>,
        location: Option<&TextRange>,
    ) -> Arc<Node> {
        let member_name = self.get_namespace_qualified_property(
            self.get_namespace_container_name(self.current_namespace.as_ref().unwrap()),
            export_name.clone(),
        );
        let expression = self
            .factory()
            .new_assignment_expression(&member_name, export_value);
        if let Some(location) = location {
            self.emit_context()
                .set_source_map_range(&expression, *location);
        }
        expression
    }

    pub fn create_export_statement_for_declaration(&mut self, node: &Arc<Node>) -> Arc<Node> {
        let export_name = self.factory().get_external_module_or_namespace_export_name(
            Some(&self.get_namespace_container_name(self.current_namespace.as_ref().unwrap())),
            node,
            false,
            true,
        );
        let local_name = self.factory().get_local_name(node);
        let expression = self
            .factory()
            .new_assignment_expression(&export_name, &local_name);
        let mut export_assignment_source_map_range = node.loc;
        if let Some(name) = node.name() {
            export_assignment_source_map_range =
                export_assignment_source_map_range.with_pos(name.pos());
        }
        self.emit_context()
            .set_source_map_range(&expression, export_assignment_source_map_range);

        let statement = self.factory().new_expression_statement(&expression);
        let export_statement_source_map_range =
            node.loc.with_pos((-1i32) as usize);
        self.emit_context()
            .set_source_map_range(&statement, export_statement_source_map_range);
        statement
    }

    pub fn create_export_assignment(
        &mut self,
        name: &Arc<Node>,
        expression: &Arc<Node>,
        export_assignment_source_map_range: TextRange,
        original: &Arc<Node>,
    ) -> Arc<Node> {
        let export_name = self.get_namespace_qualified_property(
            self.get_namespace_container_name(self.current_namespace.as_ref().unwrap()),
            name.clone(),
        );
        let export_assignment = self
            .factory()
            .new_assignment_expression(&export_name, expression);
        self.emit_context().set_original(&export_assignment, original);
        self.emit_context()
            .set_source_map_range(&export_assignment, export_assignment_source_map_range);
        export_assignment
    }
}

impl DeclarationTransformer {
    pub(crate) fn factory(&self) -> NodeFactory<'_> {
        self.transformer.factory()
    }

    pub fn build_class_members(
        &mut self,
        class_node: &Arc<Node>,
        extra_members: &[Arc<Node>],
    ) -> NodeList {
        let ctor = get_first_constructor_with_body(class_node);
        let mut parameter_properties: Vec<Arc<Node>> = Vec::new();
        if let Some(ctor) = ctor {
            let old_diag = self.state.get_symbol_accessibility_diagnostic.take();
            for param in ctor.parameters().unwrap().nodes.iter() {
                if !has_syntactic_modifier(param, ModifierFlags::ParameterPropertyModifier)
                    || self.should_strip_internal(Some(param))
                {
                    continue;
                }
                self.state.get_symbol_accessibility_diagnostic =
                    Some(create_get_symbol_accessibility_diagnostic_for_node(param));
                if param.name().is_some_and(|n| is_identifier(n)) {
                    let question_token = match &param.data {
                        NodeData::ParameterDeclaration(d) => d.question_token.clone(),
                        _ => None,
                    };
                    let ensured_modifiers = self.ensure_modifiers(param);
                    let ensured_type = self.ensure_type(param, false);
                    let ensured_no_initializer = self.ensure_no_initializer(param);
                    let updated = self.factory().new_property_declaration(
                        ensured_modifiers,
                        &param.name().unwrap(),
                        question_token.as_ref(),
                        ensured_type.as_ref(),
                        ensured_no_initializer.as_ref(),
                    );
                    self.preserve_js_doc(&updated, param);
                    parameter_properties.push(updated);
                } else {
                    let pattern = param.name().unwrap();
                    parameter_properties.extend(self.walk_binding_pattern(&pattern, param));
                }
            }
            self.state.get_symbol_accessibility_diagnostic = old_diag;
        }

        let mut private_identifier: Option<Arc<Node>> = None;
        if class_like_members(class_node)
            .nodes
            .iter()
            .any(|member| member.name().is_some_and(|n| is_private_identifier(n)))
        {
            private_identifier = Some(self.factory().new_property_declaration(
                None,
                &self.factory().new_private_identifier("#private"),
                None,
                None,
                None,
            ));
        }

        let late_indexes = self.resolver.create_late_bound_index_signatures(
            class_node,
            Some(self.enclosing_declaration.as_ref().expect("enclosing declaration")),
            declaration_emit_node_builder_flags(),
            declaration_emit_internal_node_builder_flags(),
            &self.tracker,
        );

        let mut member_nodes: Vec<Arc<Node>> = Vec::new();
        if let Some(private_identifier) = private_identifier {
            member_nodes.push(private_identifier);
        }
        member_nodes.extend(late_indexes);
        member_nodes.extend(parameter_properties);
        member_nodes.extend(extra_members.iter().cloned());
        let visit_result = self
            .transformer
            .visitor()
            .visit_node_list(Some(class_like_members(class_node)));
        member_nodes.extend(visit_result);
        let result_list = self.factory().new_node_list(member_nodes);
        crate::mig::m4h_4::r36k9_defs::cloned_node_list(&result_list)
    }

    pub(crate) fn walk_binding_pattern(
        &mut self,
        pattern: &Arc<Node>,
        param: &Arc<Node>,
    ) -> Vec<Arc<Node>> {
        let mut elems: Vec<Arc<Node>> = Vec::new();
        let elements = match (&pattern.kind, &pattern.data) {
            (
                SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern,
                NodeData::BindingPattern(d),
            ) => d.elements.clone(),
            _ => return elems,
        };
        for elem in elements.nodes.iter() {
            if is_omitted_expression(elem) {
                continue;
            }
            let elem_name = elem.name().cloned();
            if elem_name
                .as_ref()
                .is_some_and(|n| tsox_frontend::ast::node_data_generated::is_binding_pattern(n))
            {
                let nested = elem_name.unwrap();
                elems.extend(self.walk_binding_pattern(&nested, param));
                continue;
            }
            if let Some(elem_name) = elem_name {
                let ensured_type = self.ensure_type(elem, false);
                elems.push(self.factory().new_property_declaration(
                    self.ensure_modifiers(param),
                    &elem_name,
                    None,
                    ensured_type.as_ref(),
                    None,
                ));
            }
        }
        elems
    }

    pub fn collect_this_property_assignments(&mut self, class_node: &Arc<Node>) -> Vec<Arc<Node>> {
        let members = class_like_members(class_node).clone();
        let mut seen = std::collections::HashSet::new();
        for member in members.nodes.iter() {
            if let Some(name) = member.name() {
                let is_static = is_static(member);
                seen.insert(get_this_property_assignment_key(Some(name), member, is_static));
            }
        }
        self.seen_properties = seen;
        self.this_property_assignments_collected = Vec::new();

        for n in members.nodes.iter() {
            self.this_property_visitor.visit_node(n);
        }
        std::mem::take(&mut self.this_property_assignments_collected)
    }

    pub fn get_name_expression_preferring_identifier(&mut self, name_expr: &Arc<Node>) -> Arc<Node> {
        let mut name_expr = name_expr.clone();
        if is_numeric_literal(&name_expr) {
            name_expr = self
                .factory()
                .new_string_literal(name_expr.text(), TOKEN_FLAGS_NONE_R37K18);
        }
        if is_string_literal_like(&name_expr)
            && is_identifier_text(name_expr.text(), LanguageVariant::Standard)
        {
            let result = self.factory().new_identifier(name_expr.text());
            let kw_kind = identifier_to_keyword_kind(&result);
            if kw_kind == SyntaxKind::Unknown || kw_kind == SyntaxKind::DefaultKeyword {
                if let Some(parent) = name_expr.parent() {
                    result.set_parent(&parent);
                }
                return result;
            }
        }
        name_expr
    }

    pub fn get_expando_host_id(&self, declaration: &Arc<Node>) -> NodeId {
        let root = if is_variable_declaration(declaration) {
            declaration.parent().unwrap().parent().unwrap()
        } else {
            declaration.clone()
        };
        get_node_id(&self.transformer.emit_context().most_original(&root))
    }

    pub fn create_full_expando_block(&mut self, id: NodeId) -> Option<Arc<Node>> {
        if let Some(deferred) = self.deferred_expando_assignments.remove(&id) {
            for assignment in deferred {
                self.transform_expando_assignment(&assignment);
            }
        }
        let n = self.expando_hosts.get(&id).cloned();
        if let Some(add_ons) = self.expando_members.remove(&id) {
            let mut modifiers: Option<Arc<ModifierList>> = None;
            let mut name: Option<Arc<Node>> = None;
            let mut host: Vec<Arc<Node>> = Vec::new();
            if let Some(n) = &n {
                if n.kind == SyntaxKind::SyntaxList {
                    for c in syntax_list_children(n) {
                        if let Some(c_name) = c.name() {
                            name = Some(deep_clone_node(&c_name));
                            if let Some(c_modifiers) = c.modifiers() {
                                modifiers = Some(Arc::new(deep_clone_modifier_list(c_modifiers)));
                            }
                            break;
                        }
                    }
                    host = syntax_list_children(n);
                } else {
                    name = n.name().map(|n| deep_clone_node(&n));
                    if let Some(n_modifiers) = n.modifiers() {
                        modifiers = Some(Arc::new(deep_clone_modifier_list(n_modifiers)));
                    }
                    host = vec![n.clone()];
                }
            }
            if let Some(name) = name {
                let module_block = self
                    .factory()
                    .new_module_block(self.factory().new_node_list(add_ons));
                let module_decl = self.factory().new_module_declaration(
                    modifiers,
                    SyntaxKind::NamespaceKeyword,
                    &name,
                    None,
                    Some(&module_block),
                );
                let mut members = host;
                members.push(module_decl);
                return Some(self.factory().new_syntax_list(members));
            }
        }
        n
    }

    pub fn transform_expando_assignment(&mut self, node: &Arc<Node>) {
        let left = node.as_binary_expression().left.clone();
        let ns = get_leftmost_access_expression(&left);
        if ns.kind != SyntaxKind::Identifier {
            return;
        }
        let declaration = match self.resolver.get_referenced_value_declaration_unsafe(&ns) {
            Some(d) => d,
            None => return,
        };
        if self.should_strip_internal(Some(&declaration)) {
            return;
        }
        if is_variable_declaration(&declaration) && declaration.type_().is_some() {
            return;
        }
        if is_function_declaration(&declaration)
            && declaration
                .as_function_declaration_r42k01()
                .full_signature
                .is_some()
        {
            return;
        }
        if is_variable_declaration(&declaration) {
            let initializer_is_function_like = declaration
                .initializer()
                .map(|i| is_function_like(&i))
                .unwrap_or(false);
            if !initializer_is_function_like {
                return;
            }
        }
        let property = self.try_get_property_name(&left);
        if property.is_empty() || !is_identifier_text(&property, LanguageVariant::Standard) {
            return;
        }
        let host_root = if is_variable_declaration(&declaration) {
            declaration.parent().and_then(|p| p.parent())
        } else {
            Some(declaration.clone())
        };
        let Some(host_root) = host_root else {
            return;
        };
        let host_id = get_node_id(&self.emit_context().most_original(&host_root));
        if is_declaration(&declaration)
            && is_declaration_and_not_visible(&self.emit_context(), &self.resolver, &declaration)
        {
            self.deferred_expando_assignments
                .entry(host_id)
                .or_default()
                .push(node.clone());
            return;
        }
        if is_function_declaration(&declaration) && !should_emit_function_properties(&declaration) {
            return;
        }
        let export_name = self.factory().new_identifier(&property);
        let local_name = self.try_get_name_of_assigned_expression(node);
        let enclosing = self.enclosing_declaration.clone();
        let local_name = match local_name {
            Some(local_name) => {
                if is_identifier_text(local_name.text(), LanguageVariant::Standard) {
                    Some(local_name)
                } else {
                    None
                }
            }
            None => {
                if !self.resolver.is_name_resolvable(enclosing.as_ref(), &property)
                    && is_identifier_text(&property, LanguageVariant::Standard)
                {
                    Some(export_name.clone())
                } else {
                    None
                }
            }
        };
        let local_name = match local_name {
            Some(local_name) => local_name,
            None => {
                let generated =
                    self.factory()
                        .new_unique_name_ex(&property, AutoGenerateOptions {
                            flags: GeneratedIdentifierFlags::OPTIMISTIC,
                            prefix: String::new(),
                            suffix: String::new(),
                        });
                self.factory().generated_name_node(&generated)
            }
        };
        let (_, cleanup_diagnostic_context) = self.setup_diagnostic_context(node);
        if is_identifier(node.as_binary_expression().right.as_ref()) {
            let result =
                self.transform_binary_expression_to_export_declaration(node, &export_name);
            if let Some(result) = result {
                self.expando_members.entry(host_id).or_default().push(result);
            }
            cleanup_diagnostic_context(self);
            return;
        }
        let ensured_type = self.ensure_type(node, false);
        let var_decl = self.factory().new_variable_declaration(
            &local_name,
            None,
            ensured_type.as_ref(),
            None,
        );
        let decl_list = self.factory().new_variable_declaration_list(
            &self.factory().new_node_list(vec![var_decl]),
            NodeFlags::empty(),
        );
        let statement = self.factory().new_variable_statement_r42k01(None, decl_list);
        let mut statements = vec![statement];
        if local_name.text() != export_name.text() {
            let export_specifier =
                self.factory()
                    .new_export_specifier(false, Some(&local_name), &export_name);
            let named_exports = self
                .factory()
                .new_named_exports(&self.factory().new_node_list(vec![export_specifier]));
            let export_decl = self.factory().new_export_declaration(
                None,
                false,
                &named_exports,
                None,
                None,
            );
            statements.push(export_decl);
        }
        self.expando_members
            .entry(host_id)
            .or_default()
            .extend(statements);
        cleanup_diagnostic_context(self);
    }

    pub fn try_get_name_of_assigned_expression(&mut self, unwrapped: &Arc<Node>) -> Option<Arc<Node>> {
        let mut name_text = String::new();
        if !is_property_access_expression(unwrapped) && unwrapped.name().is_some() {
            name_text = unwrapped.name().unwrap().text().to_string();
        } else if is_identifier(unwrapped) {
            name_text = unwrapped.text().to_string();
        }
        if !name_text.is_empty() && name_text != "default" {
            let enclosing = self.enclosing_declaration.clone();
            if self
                .resolver
                .is_name_resolvable(enclosing.as_ref(), &name_text)
            {
                let generated = self.factory().new_unique_name_ex(
                    &name_text,
                    AutoGenerateOptions {
                        flags: GeneratedIdentifierFlags::OPTIMISTIC,
                        prefix: String::new(),
                        suffix: String::new(),
                    },
                );
                return Some(self.factory().generated_name_node(&generated));
            } else {
                return Some(self.factory().new_identifier(&name_text));
            }
        }
        None
    }

    pub fn transform_binary_expression_to_export_declaration(
        &mut self,
        input: &Arc<Node>,
        name: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        let property_name = input.as_binary_expression().right.clone();
        let property_name = if is_identifier(name) && property_name.text() == name.text() {
            None
        } else {
            Some(property_name)
        };
        let export_specifier =
            self.factory()
                .new_export_specifier(false, property_name.as_ref(), name);
        let named_exports = self
            .factory()
            .new_named_exports(&self.factory().new_node_list(vec![export_specifier]));
        Some(self.factory().new_export_declaration(
            None,
            false,
            &named_exports,
            None,
            None,
        ))
    }

    pub fn try_get_property_name(&mut self, node: &Arc<Node>) -> String {
        if is_element_access_expression(node) {
            return self.resolver.get_element_access_expression_name(node);
        }
        if is_property_access_expression(node) {
            return node.name().map(|n| n.text().to_string()).unwrap_or_default();
        }
        String::new()
    }
}

pub fn extract_expando_host_params(
    node: &Arc<Node>,
) -> (
    Option<Arc<NodeList>>,
    Option<Arc<NodeList>>,
    Option<Arc<Node>>,
) {
    (
        function_type_parameters(node),
        function_parameters(node),
        function_asterisk_token(node),
    )
}

impl CommonJSModuleTransformer<'_> {
    pub fn create_import_call_expression_common_js(&mut self, arg: Option<&Arc<Node>>) -> Arc<Node> {
        let factory = NodeFactory::new(&self.emit_context);
        let need_sync_eval = arg.is_some_and(|arg| !is_simple_inlineable_expression(arg));

        let mut promise_resolve_arguments: Vec<Arc<Node>> = Vec::new();
        if need_sync_eval {
            let template = factory.new_template_expression(
                &factory.new_template_head("", "", TOKEN_FLAGS_NONE_R37K18),
                factory.new_node_list(vec![factory.new_template_span(
                    arg.unwrap(),
                    &factory.new_template_tail("", "", TOKEN_FLAGS_NONE_R37K18),
                )]),
            );
            promise_resolve_arguments.push(template);
        }
        let promise_resolve_call = factory.new_call_expression(
            &factory.new_property_access_expression(
                &factory.new_identifier("Promise"),
                None,
                &factory.new_identifier("resolve"),
                NodeFlags::empty(),
            ),
            None,
            None,
            factory.new_node_list(promise_resolve_arguments),
            NodeFlags::empty(),
        );

        let mut require_arguments: Vec<Arc<Node>> = Vec::new();
        if need_sync_eval {
            require_arguments.push(factory.new_identifier("s"));
        } else if let Some(arg) = arg {
            require_arguments.push(arg.clone());
        }

        let require_call = factory.new_import_star_helper(&factory.new_call_expression(
            &factory.new_identifier("require"),
            None,
            None,
            factory.new_node_list(require_arguments),
            NodeFlags::empty(),
        ));

        let mut parameters: Vec<Arc<Node>> = Vec::new();
        if need_sync_eval {
            parameters.push(factory.new_parameter_declaration(
                None,
                None,
                &factory.new_identifier("s"),
                None,
                None,
                None,
            ));
        }

        let function = factory.new_arrow_function(
            None,
            None,
            &factory.new_node_list(parameters),
            None,
            None,
            &factory.new_token(SyntaxKind::EqualsGreaterThanToken),
            &require_call,
        );

        factory.new_call_expression(
            &factory.new_property_access_expression(
                &promise_resolve_call,
                None,
                &factory.new_identifier("then"),
                NodeFlags::empty(),
            ),
            None,
            None,
            factory.new_node_list(vec![function]),
            NodeFlags::empty(),
        )
    }
}
