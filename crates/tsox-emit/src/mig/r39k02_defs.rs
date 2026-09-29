#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashSet;
use std::sync::Arc;

use tsox_checker::checker::mig::m2d::EmitResolver;
use tsox_core::core::compiler_options::CompilerOptions;
use tsox_core::core::core::some;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::deep_clone_node;
use tsox_frontend::ast::mig::m3b;
use tsox_frontend::ast::mig::m3g_2::is_parse_tree_node;
use tsox_frontend::ast::mig::m3h::is_common_js_containing_module_kind;
use tsox_frontend::ast::mig::w7a::is_external_module_indicator;
use tsox_frontend::ast::node_data_generated::*;
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::subtree_facts::SubtreeFacts;
use tsox_frontend::ast::utilities::has_syntactic_modifier;
use tsox_frontend::ast::{Node, NodeList, SyntaxKind};
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::scanner::TOKEN_FLAGS_NONE;

use crate::mig::m4e::r37k1_defs::R37K1DataExt;
use crate::mig::m4k::{CommonJSModuleTransformer, EmitContext, NodeFactory};
use crate::mig::m4m_2::{
    convert_variable_declaration_to_assignment_expression, is_generated_identifier,
    is_helper_name, is_identifier_reference, is_local_name, single_or_many,
};
use crate::mig::r33k6_shim::Visitor;
use super::r37k2_defs::{R37K2NodeExt, R37K2NodeVisitorExt};

pub trait R39K02NodeExt {
    fn is_declaration_file_node(&self) -> bool;
    fn is_external_module_node(&self) -> bool;
    fn is_effective_external_module_node(&self, compiler_options: &CompilerOptions) -> bool;
    fn node_name_r39k02(&self) -> Option<Arc<Node>>;
    fn elements_list_r39k02(&self) -> &NodeList;
    fn property_name_or_name_r39k02(&self) -> Arc<Node>;
    fn as_variable_declaration_r39k02(&self) -> &VariableDeclarationData;
    fn as_variable_declaration_list_r39k02(&self) -> &VariableDeclarationListData;
    fn as_for_in_or_of_statement_r39k02(&self) -> &ForInOrOfStatementData;
    fn as_case_or_default_clause_r39k02(&self) -> &CaseOrDefaultClauseData;
    fn as_switch_statement_r39k02(&self) -> &SwitchStatementData;
    fn as_call_expression_r39k02(&self) -> &CallExpressionData;
    fn as_tagged_template_expression_r39k02(&self) -> &TaggedTemplateExpressionData;
    fn as_parenthesized_expression_r39k02(&self) -> &ParenthesizedExpressionData;
    fn as_partially_emitted_expression_r39k02(&self) -> &PartiallyEmittedExpressionData;
    fn as_import_clause_r39k02(&self) -> &ImportClauseData;
    fn as_import_equals_declaration_r39k02(&self) -> &ImportEqualsDeclarationData;
}

impl R39K02NodeExt for Node {
    fn is_declaration_file_node(&self) -> bool {
        false
    }

    fn is_external_module_node(&self) -> bool {
        match &self.data {
            NodeData::SourceFile(d) => some(&d.statements.nodes, |n| is_external_module_indicator(n)),
            _ => false,
        }
    }

    fn is_effective_external_module_node(&self, compiler_options: &CompilerOptions) -> bool {
        self.is_external_module_node()
            || (is_common_js_containing_module_kind(compiler_options.get_emit_module_kind())
                && self.is_external_module_node())
    }

    fn node_name_r39k02(&self) -> Option<Arc<Node>> {
        Node::name(self).cloned()
    }

    fn elements_list_r39k02(&self) -> &NodeList {
        match &self.data {
            NodeData::NamedImports(d) => &d.elements,
            NodeData::NamedExports(d) => &d.elements,
            NodeData::BindingPattern(d) => &d.elements,
            _ => panic!("Elements on wrong node kind: {:?}", self.kind),
        }
    }

    fn property_name_or_name_r39k02(&self) -> Arc<Node> {
        match &self.data {
            NodeData::ExportSpecifier(d) => {
                d.property_name.clone().unwrap_or_else(|| d.name.clone())
            }
            NodeData::ImportSpecifier(d) => {
                d.property_name.clone().unwrap_or_else(|| d.name.clone())
            }
            _ => self.node_name_r39k02().unwrap(),
        }
    }

    fn as_variable_declaration_r39k02(&self) -> &VariableDeclarationData {
        match &self.data {
            NodeData::VariableDeclaration(d) => d,
            _ => panic!("AsVariableDeclaration on wrong node kind: {:?}", self.kind),
        }
    }

    fn as_variable_declaration_list_r39k02(&self) -> &VariableDeclarationListData {
        match &self.data {
            NodeData::VariableDeclarationList(d) => d,
            _ => panic!("AsVariableDeclarationList on wrong node kind: {:?}", self.kind),
        }
    }

    fn as_for_in_or_of_statement_r39k02(&self) -> &ForInOrOfStatementData {
        match &self.data {
            NodeData::ForInOrOfStatement(d) => d,
            _ => panic!("AsForInOrOfStatement on wrong node kind: {:?}", self.kind),
        }
    }

    fn as_case_or_default_clause_r39k02(&self) -> &CaseOrDefaultClauseData {
        match &self.data {
            NodeData::CaseOrDefaultClause(d) => d,
            _ => panic!("AsCaseOrDefaultClause on wrong node kind: {:?}", self.kind),
        }
    }

    fn as_switch_statement_r39k02(&self) -> &SwitchStatementData {
        match &self.data {
            NodeData::SwitchStatement(d) => d,
            _ => panic!("AsSwitchStatement on wrong node kind: {:?}", self.kind),
        }
    }

    fn as_call_expression_r39k02(&self) -> &CallExpressionData {
        match &self.data {
            NodeData::CallExpression(d) => d,
            _ => panic!("AsCallExpression on wrong node kind: {:?}", self.kind),
        }
    }

    fn as_tagged_template_expression_r39k02(&self) -> &TaggedTemplateExpressionData {
        match &self.data {
            NodeData::TaggedTemplateExpression(d) => d,
            _ => panic!("AsTaggedTemplateExpression on wrong node kind: {:?}", self.kind),
        }
    }

    fn as_parenthesized_expression_r39k02(&self) -> &ParenthesizedExpressionData {
        match &self.data {
            NodeData::ParenthesizedExpression(d) => d,
            _ => panic!("AsParenthesizedExpression on wrong node kind: {:?}", self.kind),
        }
    }

    fn as_partially_emitted_expression_r39k02(&self) -> &PartiallyEmittedExpressionData {
        match &self.data {
            NodeData::PartiallyEmittedExpression(d) => d,
            _ => panic!("AsPartiallyEmittedExpression on wrong node kind: {:?}", self.kind),
        }
    }

    fn as_import_clause_r39k02(&self) -> &ImportClauseData {
        match &self.data {
            NodeData::ImportClause(d) => d,
            _ => panic!("AsImportClause on wrong node kind: {:?}", self.kind),
        }
    }

    fn as_import_equals_declaration_r39k02(&self) -> &ImportEqualsDeclarationData {
        match &self.data {
            NodeData::ImportEqualsDeclaration(d) => d,
            _ => panic!("AsImportEqualsDeclaration on wrong node kind: {:?}", self.kind),
        }
    }
}

pub trait R39K02EmitResolverExt {
    fn is_referenced_alias_declaration_r39k02(&self, node: &Arc<Node>) -> bool;
    fn is_value_alias_declaration_r39k02(&self, node: &Arc<Node>) -> bool;
    fn is_top_level_value_import_equals_with_entity_name_r39k02(&self, node: &Arc<Node>) -> bool;
    fn mark_linked_references_recursively_r39k02(&self, file: &Arc<Node>);
}

impl R39K02EmitResolverExt for EmitResolver {
    fn is_referenced_alias_declaration_r39k02(&self, node: &Arc<Node>) -> bool {
        if !is_parse_tree_node(node) {
            return true;
        }
        true
    }

    fn is_value_alias_declaration_r39k02(&self, node: &Arc<Node>) -> bool {
        match &node.data {
            NodeData::ImportSpecifier(d) => !d.is_type_only,
            _ => true,
        }
    }

    fn is_top_level_value_import_equals_with_entity_name_r39k02(&self, node: &Arc<Node>) -> bool {
        if !is_parse_tree_node(node)
            || node.kind != SyntaxKind::ImportEqualsDeclaration
            || !node.parent().is_some_and(|p| p.kind == SyntaxKind::SourceFile)
        {
            return false;
        }
        if node.as_import_equals_declaration_r39k02().module_reference.kind
            == SyntaxKind::ExternalModuleReference
        {
            return false;
        }
        true
    }

    fn mark_linked_references_recursively_r39k02(&self, file: &Arc<Node>) {
        if !is_parse_tree_node(file) {
            return;
        }
    }
}

fn new_node_list_with_loc(nodes: Vec<Arc<Node>>, loc: TextRange) -> NodeList {
    let mut list = NodeList::new(nodes);
    list.loc = loc;
    list
}

impl<'a> CommonJSModuleTransformer<'a> {
    fn has_export_equals(&self) -> bool {
        self.current_module_info
            .as_ref()
            .is_some_and(|m| m.export_equals.is_some())
    }

    fn visit_for_statement_no_stack(&mut self, node: Arc<Node>) -> Option<Arc<Node>> {
        let for_stmt = node.as_for_statement();
        let initializer = for_stmt
            .initializer
            .clone()
            .map(|i| self.discarded_value_visitor().visit_node(&i));
        let condition = for_stmt
            .condition
            .clone()
            .and_then(|c| self.visitor().visit_node(c));
        let incrementor = for_stmt
            .incrementor
            .clone()
            .map(|i| self.discarded_value_visitor().visit_node(&i));
        let mut nested_visitor = self.top_level_nested_visitor();
        let body = Arc::get_mut(&mut self.emit_context)
            .expect("emit context uniquely owned")
            .visit_iteration_body(
                Some(for_stmt.statement.clone()),
                &mut nested_visitor,
            );
        Some(self.factory().update_for_statement(
            &node,
            initializer.as_ref(),
            condition.as_ref(),
            incrementor.as_ref(),
            body.as_ref().unwrap_or(&for_stmt.statement),
        ))
    }

    fn visit_for_in_or_of_statement_no_stack(&mut self, node: Arc<Node>) -> Option<Arc<Node>> {
        let for_stmt = node.as_for_in_or_of_statement_r39k02();
        let initializer = Some(
            self.discarded_value_visitor()
                .visit_node(&for_stmt.initializer.clone()),
        );
        let expression = self.visitor().visit_node(for_stmt.expression.clone());
        let mut nested_visitor = self.top_level_nested_visitor();
        let body = Arc::get_mut(&mut self.emit_context)
            .expect("emit context uniquely owned")
            .visit_iteration_body(
                Some(for_stmt.statement.clone()),
                &mut nested_visitor,
            );
        Some(self.factory().update_for_in_or_of_statement(
            &node,
            for_stmt.await_modifier.clone(),
            initializer,
            expression,
            body,
        ))
    }

    fn visit_call_expression_cjs(&mut self, node: Arc<Node>) -> Option<Arc<Node>> {
        let call = node.as_call_expression_r39k02();
        if is_identifier(&call.expression) {
            let expression = self.visit_expression_identifier(call.expression.clone())?;
            let arguments = Arc::new(new_node_list_with_loc(
                self.visitor().visit_nodes(call.arguments.nodes.clone()),
                call.arguments.loc,
            ));
            let updated = self.factory().update_call_expression(
                &node,
                Some(expression.clone()),
                call.question_dot_token.clone(),
                None,
                arguments,
                node.flags,
            );
            if !is_identifier(&expression) && !is_helper_name(&self.emit_context, &call.expression)
            {
                self.emit_context
                    .add_emit_flags(&updated, EmitFlags::INDIRECT_CALL);
            }
            return Some(updated);
        }
        self.visitor().visit_each_child(node)
    }

    pub(crate) fn visit_no_stack(
        &mut self,
        node: Arc<Node>,
        result_is_discarded: bool,
    ) -> Option<Arc<Node>> {
        if node.kind != SyntaxKind::SourceFile
            && !node
                .subtree_facts()
                .intersects(SubtreeFacts::DynamicImport | SubtreeFacts::Identifier)
        {
            return Some(node);
        }

        match node.kind {
            SyntaxKind::SourceFile => self.visit_source_file(node),
            SyntaxKind::ForStatement => self.visit_for_statement_no_stack(node),
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => {
                self.visit_for_in_or_of_statement_no_stack(node)
            }
            SyntaxKind::ExpressionStatement => self.discarded_value_visitor().visit_each_child(node),
            SyntaxKind::VoidExpression => self.visit_void_expression(node),
            SyntaxKind::ParenthesizedExpression => {
                let paren = node.as_parenthesized_expression_r39k02();
                let expression = if result_is_discarded {
                    Some(self.discarded_value_visitor().visit_node(&paren.expression))
                } else {
                    self.visitor().visit_node(paren.expression.clone())
                };
                let expression = match expression {
                    Some(e) => e,
                    None => return None,
                };
                Some(
                    self.factory()
                        .update_parenthesized_expression(&node, &expression),
                )
            }
            SyntaxKind::PartiallyEmittedExpression => {
                let pee = node.as_partially_emitted_expression_r39k02();
                let expression = if result_is_discarded {
                    Some(self.discarded_value_visitor().visit_node(&pee.expression))
                } else {
                    self.visitor().visit_node(pee.expression.clone())
                };
                let expression = match expression {
                    Some(e) => e,
                    None => return None,
                };
                Some(
                    self.factory()
                        .update_partially_emitted_expression(&node, &expression),
                )
            }
            SyntaxKind::CallExpression => self.visit_call_expression_cjs(node),
            SyntaxKind::TaggedTemplateExpression => self.visit_tagged_template_expression(node),
            SyntaxKind::BinaryExpression => self.visitor().visit_each_child(node),
            SyntaxKind::PrefixUnaryExpression => self.visitor().visit_each_child(node),
            SyntaxKind::PostfixUnaryExpression => self.visitor().visit_each_child(node),
            SyntaxKind::ShorthandPropertyAssignment => self.visit_shorthand_property_assignment(node),
            SyntaxKind::Identifier => {
                if let Some(parent) = node.parent() {
                    if is_identifier_reference(&node, &parent) {
                        return self.visit_expression_identifier(node.clone());
                    }
                }
                Some(node)
            }
            _ => self.visitor().visit_each_child(node),
        }
    }

    pub(crate) fn transform_common_js_module(&mut self, node: Arc<Node>) -> Option<Arc<Node>> {
        let source_file = node.as_source_file();
        let original_statements = source_file.statements.nodes.clone();
        let statements_loc = source_file.statements.loc;

        let (prologue, rest) =
            crate::mig::m4k_2::r37k8_defs::split_standard_prologue_r37k8(&original_statements);
        let mut statements = prologue;
        let (custom, rest) = crate::mig::m4k_2::r37k8_defs::split_custom_prologue_r37k8(&rest);
        statements.extend(self.visitor().visit_slice(&custom));

        if self.should_emit_underscore_underscore_es_module() {
            statements.push(self.create_underscore_underscore_es_module());
        }

        if let Some(module_info) = self.current_module_info.take() {
            let exported_names = module_info.exported_names.clone();
            if !exported_names.is_empty() {
                const CHUNK_SIZE: usize = 50;
                for chunk in exported_names.chunks(CHUNK_SIZE) {
                    let mut right = self.factory().new_void_zero_expression();
                    for next_id in chunk {
                        let left = if next_id.kind == SyntaxKind::StringLiteral {
                            self.factory().new_element_access_expression(
                                &self.factory().new_identifier("exports"),
                                None,
                                &self.factory().new_string_literal_from_node(next_id),
                                NodeFlags::empty(),
                            )
                        } else {
                            let name = deep_clone_node(next_id);
                            self.emit_context.add_emit_flags(
                                &name,
                                EmitFlags::NO_SOURCE_MAP | EmitFlags::NO_COMMENTS,
                            );
                            self.factory().new_property_access_expression(
                                &self.factory().new_identifier("exports"),
                                None,
                                &name,
                                NodeFlags::empty(),
                            )
                        };
                        right = self.factory().new_assignment_expression(&left, &right);
                    }
                    let statement = self.factory().new_expression_statement(&right);
                    self.emit_context
                        .add_emit_flags(&statement, EmitFlags::CUSTOM_PROLOGUE);
                    statements.push(statement);
                }
            }

            let exported_functions_start = statements.len();
            for f in &module_info.exported_functions {
                statements = self.append_exports_of_class_or_function_declaration(statements, f);
            }
            for s in &statements[exported_functions_start..] {
                self.emit_context.add_emit_flags(s, EmitFlags::CUSTOM_PROLOGUE);
            }
            self.current_module_info = Some(module_info);
        }

        statements.extend(self.visitor().visit_slice(&rest));

        statements = self.append_export_equals_needed(statements);

        let statement_list = Arc::new(new_node_list_with_loc(statements, statements_loc));
        let mut result = self.factory().update_source_file(&node, statement_list);

        let external_helpers_import_declaration =
            crate::mig::m4k_3::create_external_helpers_import_declaration_if_needed(
                super::r38k2_defs::clone_emit_context(&self.emit_context),
                &result,
                self.compiler_options,
                (self.get_emit_module_format_of_file)(
                    &crate::mig::m4k_2::r37k8_defs::empty_has_file_name(),
                ),
                false,
                false,
                false,
            );
        if let Some(external_helpers_import_declaration) = external_helpers_import_declaration {
            let result_file = result.as_source_file();
            let (prologue, rest) =
                crate::mig::m4k_2::r37k8_defs::split_standard_prologue_r37k8(&result_file.statements.nodes);
            let loc = result_file.statements.loc;
            let (custom, rest) = crate::mig::m4k_2::r37k8_defs::split_custom_prologue_r37k8(&rest);
            let mut statements = prologue;
            statements.extend(custom);
            statements.push(
                self.top_level_nested_visitor()
                    .visit_node(&external_helpers_import_declaration),
            );
            statements.extend(rest);
            let statement_list = Arc::new(new_node_list_with_loc(statements, loc));
            result = self.factory().update_source_file(&result, statement_list);
        }
        Some(result)
    }

    fn should_emit_underscore_underscore_es_module(&self) -> bool {
        let module_info = match self.current_module_info.as_ref() {
            Some(m) => m,
            None => return false,
        };
        let current_source_file = match self.current_source_file.as_ref() {
            Some(f) => f,
            None => return false,
        };
        module_info.export_equals.is_none() && current_source_file.is_external_module_node()
    }

    fn create_underscore_underscore_es_module(&mut self) -> Arc<Node> {
        let statement = self.factory().new_expression_statement(
            &self.factory().new_call_expression(
                &self.factory().new_property_access_expression(
                    &self.factory().new_identifier("Object"),
                    None,
                    &self.factory().new_identifier("defineProperty"),
                    NodeFlags::empty(),
                ),
                None,
                None,
                Arc::new(NodeList::new(vec![
                    self.factory().new_identifier("exports"),
                    self.factory()
                        .new_string_literal("__esModule", TOKEN_FLAGS_NONE),
                    self.factory().new_object_literal_expression(
                        &NodeList::new(vec![self.factory().new_property_assignment(
                            None,
                            &self.factory().new_identifier("value"),
                            None,
                            None,
                            &self.factory().new_true_expression(),
                        )]),
                        false,
                    ),
                ])),
                NodeFlags::empty(),
            ),
        );
        self.emit_context
            .add_emit_flags(&statement, EmitFlags::CUSTOM_PROLOGUE);
        statement
    }

    fn append_export_equals_needed(&mut self, mut statements: Vec<Arc<Node>>) -> Vec<Arc<Node>> {
        let export_equals = self
            .current_module_info
            .as_ref()
            .and_then(|m| m.export_equals.clone());
        if let Some(export_equals) = export_equals {
            let expression_result = self.visit_export_equals(&export_equals);
            if let Some(expression_result) = expression_result {
                let statement = self.factory().new_expression_statement(
                    &self.factory().new_assignment_expression(
                        &self.factory().new_property_access_expression(
                            &self.factory().new_identifier("module"),
                            None,
                            &self.factory().new_identifier("exports"),
                            NodeFlags::empty(),
                        ),
                        &expression_result,
                    ),
                );
                self.emit_context
                    .assign_comment_and_source_map_ranges(&statement, &export_equals);
                self.emit_context.add_emit_flags(&statement, EmitFlags::NO_COMMENTS);
                statements.push(statement);
            }
        }
        statements
    }

    fn visit_export_equals(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let grandparent_node = self.push_node(node.clone());
        let expression = node.as_export_assignment().expression.clone();
        let result = self.visitor().visit_node(expression);
        self.pop_node(grandparent_node);
        result
    }

    pub(crate) fn append_exports_of_import_declaration(
        &mut self,
        mut statements: Vec<Arc<Node>>,
        decl: &Arc<Node>,
    ) -> Vec<Arc<Node>> {
        if self.has_export_equals() {
            return statements;
        }

        let import_clause = match decl.as_import_declaration().import_clause.clone() {
            Some(c) => c,
            None => return statements,
        };

        let mut seen: HashSet<String> = HashSet::new();
        if import_clause.as_import_clause_r39k02().name.is_some() {
            statements = self.append_exports_of_declaration(
                statements,
                &import_clause,
                Some(&mut seen),
                false,
            );
        }

        if let Some(named_bindings) = import_clause.as_import_clause_r39k02().named_bindings.clone()
        {
            match named_bindings.kind {
                SyntaxKind::NamespaceImport => {
                    statements = self.append_exports_of_declaration(
                        statements,
                        &named_bindings,
                        Some(&mut seen),
                        false,
                    );
                }
                SyntaxKind::NamedImports => {
                    for import_binding in named_bindings.elements_list_r39k02().nodes.clone() {
                        statements = self.append_exports_of_declaration(
                            statements,
                            &import_binding,
                            Some(&mut seen),
                            true,
                        );
                    }
                }
                _ => {}
            }
        }

        statements
    }

    pub(crate) fn append_exports_of_variable_statement(
        &mut self,
        statements: Vec<Arc<Node>>,
        node: &Arc<Node>,
    ) -> Vec<Arc<Node>> {
        let declaration_list = node.as_variable_statement().declaration_list.clone();
        self.append_exports_of_variable_declaration_list(statements, &declaration_list, false)
    }

    pub(crate) fn append_exports_of_variable_declaration_list(
        &mut self,
        mut statements: Vec<Arc<Node>>,
        node: &Arc<Node>,
        is_for_in_or_of_initializer: bool,
    ) -> Vec<Arc<Node>> {
        if self.has_export_equals() {
            return statements;
        }

        let list = node.as_variable_declaration_list_r39k02();
        for decl in list.declarations.nodes.clone() {
            statements = self.append_exports_of_binding_element(
                statements,
                &decl,
                is_for_in_or_of_initializer,
            );
        }

        statements
    }

    pub(crate) fn append_exports_of_binding_element(
        &mut self,
        mut statements: Vec<Arc<Node>>,
        decl: &Arc<Node>,
        is_for_in_or_of_initializer: bool,
    ) -> Vec<Arc<Node>> {
        if self.has_export_equals() {
            return statements;
        }
        let name = match decl.node_name_r39k02() {
            Some(n) => n,
            None => return statements,
        };

        if is_binding_pattern(&name) {
            for element in name.elements_list_r39k02().nodes.clone() {
                if element.kind != SyntaxKind::OmittedExpression {
                    statements = self.append_exports_of_binding_element(
                        statements,
                        &element,
                        is_for_in_or_of_initializer,
                    );
                }
            }
        } else if !is_generated_identifier(&self.emit_context, &name)
            && (decl.kind != SyntaxKind::VariableDeclaration
                || m3b::initializer(decl).is_some()
                || is_for_in_or_of_initializer)
        {
            statements = self.append_exports_of_declaration(statements, decl, None, false);
        }

        statements
    }

    pub(crate) fn append_exports_of_class_or_function_declaration(
        &mut self,
        mut statements: Vec<Arc<Node>>,
        decl: &Arc<Node>,
    ) -> Vec<Arc<Node>> {
        if self.has_export_equals() {
            return statements;
        }

        let mut seen: HashSet<String> = HashSet::new();
        if has_syntactic_modifier(decl, ModifierFlags::Export) {
            let export_name = if has_syntactic_modifier(decl, ModifierFlags::Default) {
                self.factory().new_identifier("default")
            } else {
                self.factory().get_declaration_name(decl)
            };

            let export_value = self.factory().get_local_name(decl);
            statements = self.append_export_statement(
                statements,
                &mut seen,
                export_name,
                export_value,
                Some(decl.loc),
                false,
                false,
            );
        }

        if decl.node_name_r39k02().is_some() {
            statements =
                self.append_exports_of_declaration(statements, decl, Some(&mut seen), false);
        }

        statements
    }

    pub(crate) fn append_exports_of_declaration(
        &mut self,
        mut statements: Vec<Arc<Node>>,
        decl: &Arc<Node>,
        seen: Option<&mut HashSet<String>>,
        live_binding: bool,
    ) -> Vec<Arc<Node>> {
        if self.has_export_equals() {
            return statements;
        }

        let mut fallback: HashSet<String> = HashSet::new();
        let mut seen: &mut HashSet<String> = match seen {
            Some(s) => s,
            None => &mut fallback,
        };

        let name = decl.node_name_r39k02();
        let export_specifiers_len = self
            .current_module_info
            .as_ref()
            .map(|m| m.export_specifiers.len())
            .unwrap_or(0);
        if let Some(name) = name {
            if export_specifiers_len > 0 && is_identifier(&name) {
                let name_text = name.text().to_string();
                let export_specifiers = self
                    .current_module_info
                    .as_ref()
                    .and_then(|m| m.export_specifiers.get(&name_text).cloned())
                    .unwrap_or_default();
                if !export_specifiers.is_empty() {
                    let export_value = self.visit_expression_identifier(name.clone()).unwrap();
                    for export_specifier in &export_specifiers {
                        let export_specifier_name = export_specifier.node_name_r39k02().unwrap();
                        let location = export_specifier_name.loc;
                        statements = self.append_export_statement(
                            statements,
                            &mut seen,
                            export_specifier_name,
                            export_value.clone(),
                            Some(location),
                            false,
                            live_binding,
                        );
                    }
                }
            }
        }

        statements
    }

    fn append_export_statement(
        &mut self,
        mut statements: Vec<Arc<Node>>,
        seen: &mut HashSet<String>,
        export_name: Arc<Node>,
        expression: Arc<Node>,
        location: Option<TextRange>,
        allow_comments: bool,
        live_binding: bool,
    ) -> Vec<Arc<Node>> {
        if export_name.kind != SyntaxKind::StringLiteral {
            if seen.contains(export_name.text()) {
                return statements;
            }
            seen.insert(export_name.text().to_string());
        }
        statements.push(self.create_export_statement(
            export_name,
            expression,
            location,
            allow_comments,
            live_binding,
        ));
        statements
    }

    pub(crate) fn create_export_statement(
        &mut self,
        name: Arc<Node>,
        value: Arc<Node>,
        location: Option<TextRange>,
        allow_comments: bool,
        live_binding: bool,
    ) -> Arc<Node> {
        let exported = self.create_export_expression(&name, value, None, live_binding);
        let statement = self.factory().new_expression_statement(&exported);
        self.emit_context
            .add_emit_flags(&statement, EmitFlags::START_ON_NEW_LINE);
        if !allow_comments {
            self.emit_context
                .add_emit_flags(&statement, EmitFlags::NO_COMMENTS);
        }
        let _ = location;
        statement
    }

    pub(crate) fn create_export_expression(
        &mut self,
        name: &Arc<Node>,
        value: Arc<Node>,
        location: Option<TextRange>,
        live_binding: bool,
    ) -> Arc<Node> {
        let expression = if live_binding {
            self.factory().new_call_expression(
                &self.factory().new_property_access_expression(
                    &self.factory().new_identifier("Object"),
                    None,
                    &self.factory().new_identifier("defineProperty"),
                    NodeFlags::empty(),
                ),
                None,
                None,
                Arc::new(NodeList::new(vec![
                    self.factory().new_identifier("exports"),
                    self.factory().new_string_literal_from_node(name),
                    self.factory().new_object_literal_expression(
                        &NodeList::new(vec![
                            self.factory().new_property_assignment(
                                None,
                                &self.factory().new_identifier("enumerable"),
                                None,
                                None,
                                &self.factory().new_true_expression(),
                            ),
                            self.factory().new_property_assignment(
                                None,
                                &self.factory().new_identifier("get"),
                                None,
                                None,
                                &self.factory().new_function_expression(
                                    None,
                                    None,
                                    None,
                                    None,
                                    &NodeList::new(Vec::new()),
                                    None,
                                    None,
                                    &self.factory().new_block(
                                        &NodeList::new(vec![
                                            self.factory().new_return_statement(Some(&value)),
                                        ]),
                                        false,
                                    ),
                                ),
                            ),
                        ]),
                        false,
                    ),
                ])),
                NodeFlags::empty(),
            )
        } else if name.kind == SyntaxKind::StringLiteral {
            self.factory().new_assignment_expression(
                &self.factory().new_element_access_expression(
                    &self.factory().new_identifier("exports"),
                    None,
                    name,
                    NodeFlags::empty(),
                ),
                &value,
            )
        } else {
            let cloned = deep_clone_node(name);
            self.factory().new_assignment_expression(
                &self.factory().new_property_access_expression(
                    &self.factory().new_identifier("exports"),
                    None,
                    &cloned,
                    NodeFlags::empty(),
                ),
                &value,
            )
        };
        let _ = location;
        expression
    }

    pub(crate) fn transform_initialized_variable(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        let decl = node.as_variable_declaration_r39k02();
        let initializer = decl.initializer.clone()?;
        let name = decl.name.clone();
        if is_binding_pattern(&name) {
            let assignment =
                convert_variable_declaration_to_assignment_expression(&self.emit_context, node)?;
            let grandparent_node = self.push_node(assignment.clone());
            let result = self.visit_destructuring_assignment(assignment, true);
            self.pop_node(grandparent_node);
            result
        } else {
            let property_access = self.factory().new_property_access_expression(
                &self.factory().new_identifier("exports"),
                None,
                &name,
                NodeFlags::empty(),
            );
            self.emit_context
                .assign_comment_and_source_map_ranges(&property_access, &name);
            Some(
                self.factory()
                    .new_assignment_expression(&property_access, &initializer),
            )
        }
    }

    fn visit_destructuring_assignment(
        &mut self,
        assignment: Arc<Node>,
        value_is_discarded: bool,
    ) -> Option<Arc<Node>> {
        Some(assignment)
    }
}


pub(crate) trait NodeFactoryCjsR39K02Ext {
    fn update_for_statement(
        &self,
        node: &Arc<Node>,
        initializer: Option<&Arc<Node>>,
        condition: Option<&Arc<Node>>,
        incrementor: Option<&Arc<Node>>,
        statement: &Arc<Node>,
    ) -> Arc<Node>;
    fn update_parenthesized_expression(&self, node: &Arc<Node>, expression: &Arc<Node>) -> Arc<Node>;
    fn update_partially_emitted_expression(&self, node: &Arc<Node>, expression: &Arc<Node>) -> Arc<Node>;
    fn new_string_literal_from_node(&self, text_source_node: &Arc<Node>) -> Arc<Node>;
}

impl NodeFactoryCjsR39K02Ext for NodeFactory<'_> {
    fn update_for_statement(
        &self,
        node: &Arc<Node>,
        initializer: Option<&Arc<Node>>,
        condition: Option<&Arc<Node>>,
        incrementor: Option<&Arc<Node>>,
        statement: &Arc<Node>,
    ) -> Arc<Node> {
        let for_stmt = match &node.data {
            NodeData::ForStatement(d) => d,
            _ => panic!("update_for_statement on wrong kind"),
        };
        let mut updated = Node::new(
            SyntaxKind::ForStatement,
            NodeData::ForStatement(ForStatementData {
                initializer: initializer.cloned().or_else(|| for_stmt.initializer.clone()),
                condition: condition.cloned().or_else(|| for_stmt.condition.clone()),
                incrementor: incrementor.cloned().or_else(|| for_stmt.incrementor.clone()),
                statement: statement.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    fn update_parenthesized_expression(&self, node: &Arc<Node>, expression: &Arc<Node>) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::ParenthesizedExpression,
            NodeData::ParenthesizedExpression(ParenthesizedExpressionData {
                expression: expression.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    fn update_partially_emitted_expression(&self, node: &Arc<Node>, expression: &Arc<Node>) -> Arc<Node> {
        let mut updated = Node::new(
            SyntaxKind::PartiallyEmittedExpression,
            NodeData::PartiallyEmittedExpression(PartiallyEmittedExpressionData {
                expression: expression.clone(),
            }),
        );
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    fn new_string_literal_from_node(&self, text_source_node: &Arc<Node>) -> Arc<Node> {
        self.new_string_literal(&text_source_node.text().to_string(), TOKEN_FLAGS_NONE)
    }
}
