use super::super::m3n_5::create_diagnostic_for_node;
use super::{create_empty_exports, throw_diagnostic, DeclarationTransformer};
use crate::mig::m4e::r39k01_defs::R39K01EmitResolverExt;
use super::super::m4e_2::{is_declaration_and_not_visible, is_enclosing_declaration};
use crate::mig::m4e::R42K01EmitResolverExt;
use std::collections::HashMap;
use std::sync::Arc;
use tsox_frontend::ast::mig::m3f::get_node_id;
use tsox_frontend::ast::mig::m3f_4::is_dynamic_name;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_node::Node;
use tsox_frontend::ast::is_external_or_common_js_module;
use tsox_frontend::ast::{is_declaration, is_function_like, NodeList, SourceFile, SyntaxKind};

impl DeclarationTransformer {
    pub fn visit(&mut self, node: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit"); 
        match node.kind {
            SyntaxKind::SourceFile => Some(node),
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::VariableStatement
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::JSImportDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::ExportAssignment => self.visit_declaration_statements(node),
            SyntaxKind::BreakStatement
            | SyntaxKind::ContinueStatement
            | SyntaxKind::DebuggerStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::EmptyStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::ForStatement
            | SyntaxKind::IfStatement
            | SyntaxKind::LabeledStatement
            | SyntaxKind::ReturnStatement
            | SyntaxKind::SwitchStatement
            | SyntaxKind::ThrowStatement
            | SyntaxKind::TryStatement
            | SyntaxKind::WhileStatement
            | SyntaxKind::WithStatement
            | SyntaxKind::NotEmittedStatement
            | SyntaxKind::Block
            | SyntaxKind::MissingDeclaration
            | SyntaxKind::ExpressionStatement => None,
            _ => self.visit_declaration_subtree(node),
        }
    }

    pub fn transform_source_file_entry(&mut self, file: Arc<SourceFile>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_source_file_entry"); 
        self.cjs_export_assignment_name = None;
        if file.is_declaration_file {
            return Some(Arc::clone(&file.node));
        }
        self.needs_declare = true;
        self.needs_scope_fix_marker = false;
        self.result_has_scope_marker = false;
        self.enclosing_declaration = Some(Arc::clone(&file.node));
        self.state.get_symbol_accessibility_diagnostic = Some(Box::new(throw_diagnostic));
        self.result_has_external_module_indicator = false;
        self.suppress_new_diagnostic_contexts = false;
        self.state.late_marked_statements = Vec::new();
        self.tracker.state.late_marked_statements = Vec::new();
        self.late_statement_replacement_map = HashMap::new();
        self.expando_hosts = HashMap::new();
        self.expando_members = HashMap::new();
        self.deferred_expando_assignments = HashMap::new();
        self.raw_referenced_files = Vec::new();
        self.raw_type_reference_directives = Vec::new();
        self.raw_lib_reference_directives = Vec::new();
        self.witnessed_cjs_exports.clear();
        self.state.current_source_file = Some(Arc::clone(&file));
        self.tracker.state.current_source_file = Some(Arc::clone(&file));
        self.collect_file_references(&file);
        self.resolver
            .precalculate_declaration_emit_visibility(&file);
        let updated = self.transform_source_file_worker(&file);
        self.state.current_source_file = None;
        self.tracker.state.current_source_file = None;
        updated
    }

    fn transform_source_file_worker(&mut self, file: &Arc<SourceFile>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_source_file_worker"); 
        self.cjs_export_assignment = None;
        self.cjs_export_assignment_name = None;
        self.cjs_export_members = Vec::new();
        self.cjs_export_assignment_visitor.visit_node(&file.node);
        self.expression_visitor.visit_node(&file.node);
        let statements: Arc<NodeList> = match &file.node.data {
            NodeData::SourceFile(d) => Arc::clone(&d.statements),
            _ => Arc::new(NodeList::new(Vec::new())),
        };
        let visited = self.visit_nodes_via_visit(&statements);
        let mut combined = self.transform_and_replace_late_painted_statements(&visited);
        combined = self.append_cjs_exports(combined);
        if is_external_or_common_js_module(file)
            && (!self.result_has_external_module_indicator
                || (self.needs_scope_fix_marker && !self.result_has_scope_marker))
        {
            let marker = create_empty_exports(&self.factory());
            let mut nodes = combined.nodes.clone();
            nodes.push(marker);
            combined = Arc::new(NodeList {
                loc: statements.loc,
                nodes,
            });
        }
        Some(self.factory().update_source_file(&file.node, combined))
    }

    pub fn visit_nodes_via_visit(&mut self, list: &Arc<NodeList>) -> Arc<NodeList> { ::tsox_core::fntrace::enter("visit_nodes_via_visit"); 
        let nodes: Vec<Arc<Node>> = list
            .nodes
            .iter()
            .filter_map(|n| self.visit(Arc::clone(n)))
            .collect();
        self.factory().new_node_list(nodes)
    }

    pub fn visit_slice_via_visit(&mut self, nodes: Vec<Arc<Node>>) -> (Vec<Arc<Node>>, bool) { ::tsox_core::fntrace::enter("visit_slice_via_visit"); 
        let before = nodes.len();
        let count = nodes.len();
        let visited: Vec<Arc<Node>> = nodes.into_iter().filter_map(|n| self.visit(n)).collect();
        let changed = visited.len() != count;
        (visited, changed)
    }

    pub fn visit_declaration_statements(&mut self, input: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_declaration_statements"); 
        if self.should_strip_internal(Some(&input)) {
            return None;
        }
        match input.kind {
            SyntaxKind::ExportDeclaration => {
                if input
                    .parent()
                    .as_deref()
                    .map(|p| p.kind == SyntaxKind::SourceFile)
                    .unwrap_or(false)
                {
                    self.result_has_external_module_indicator = true;
                }
                self.result_has_scope_marker = true;
                Some(input)
            }
            SyntaxKind::ExportAssignment => Some(input),
            _ => {
                let id = get_node_id(&self.emit_context().most_original(&input));
                if !self.late_statement_replacement_map.contains_key(&id) {
                    let transformed = self.transform_top_level_declaration(Arc::clone(&input));
                    self.late_statement_replacement_map.insert(id, transformed);
                }
                Some(input)
            }
        }
    }

    pub fn visit_declaration_subtree(&mut self, input: Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_declaration_subtree"); 
        if self.should_strip_internal(Some(&input)) {
            return None;
        }
        if is_declaration(&input)
            && is_declaration_and_not_visible(&self.emit_context(), &self.resolver, &input)
        {
            return None;
        }
        if is_dynamic_name(&input) {
            return self.elide_or_diagnose_dynamic_name(&input);
        }
        if is_function_like(&input) && self.resolver.is_implementation_of_overload(&input) {
            return None;
        }
        if input.kind == SyntaxKind::SemicolonClassElement {
            return None;
        }
        let previous_enclosing_declaration = self.enclosing_declaration.clone();
        if is_enclosing_declaration(&input) {
            self.enclosing_declaration = Some(Arc::clone(&input));
        }
        let (can_produce_diagnostic, cleanup) = self.setup_diagnostic_context(&input);
        let result = super::m3n_12::transform_subtree_kind(self, &input);
        if result.is_some() && can_produce_diagnostic && is_dynamic_name(&input) {
            self.check_name(&input);
        }
        cleanup(self);
        self.enclosing_declaration = previous_enclosing_declaration;
        result
    }

    fn elide_or_diagnose_dynamic_name(&mut self, input: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("elide_or_diagnose_dynamic_name"); 
        if !self.state.isolated_declarations {
            let parse_node = self.emit_context().parse_node(input);
            let late_bound = self.resolver.is_late_bound(self.emit_context().parse_node(input).as_ref());
            if !late_bound
                || !is_entity_name_expr(input)
            {
                return None;
            }
            return Some(Arc::clone(input));
        }
        let is_global_ref = input
            .name()
            .and_then(|n| n.expression())
            .map(|e| self.resolver.is_definitely_reference_to_global_symbol_object(e))
            .unwrap_or(false);
        if is_global_ref {
            return Some(Arc::clone(input));
        }
        let parent_kind = input.parent().as_deref().map(|p| p.kind);
        if matches!(
            parent_kind,
            Some(SyntaxKind::ClassDeclaration) | Some(SyntaxKind::ObjectLiteralExpression)
        ) {
            self.tracker.state.add_diagnostic(create_diagnostic_for_node(
                input,
                Some(&tsox_core::diagnostics::messages_generated::COMPUTED_PROPERTY_NAMES_ON_CLASS_OR_OBJECT_LITERALS_CANNOT_BE_INFERRED_WITH_ISOLATEDDECLARATIONS),
                &[],
            ));
            return None;
        }
        if matches!(
            parent_kind,
            Some(SyntaxKind::InterfaceDeclaration) | Some(SyntaxKind::TypeLiteral)
        ) && !is_entity_name_expr(input)
        {
            self.tracker.state.add_diagnostic(create_diagnostic_for_node(
                input,
                Some(&tsox_core::diagnostics::messages_generated::COMPUTED_PROPERTIES_MUST_BE_NUMBER_OR_STRING_LITERALS_VARIABLES_OR_DOTTED_EXPRESSIONS_WITH_ISOLATEDDECLARATIONS),
                &[],
            ));
            return None;
        }
        Some(Arc::clone(input))
    }
}

fn is_entity_name_expr(input: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_entity_name_expr"); 
    input
        .name()
        .and_then(|n| n.expression())
        .map(|e| tsox_frontend::ast::is_entity_name_expression(e))
        .unwrap_or(false)
}
