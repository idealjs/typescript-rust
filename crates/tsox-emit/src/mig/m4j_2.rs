#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashSet;
use std::sync::Arc;

use tsox_core::core::compiler_options::{CompilerOptions, JsxEmit, ModuleKind, ScriptTarget};
use tsox_core::core::text::TextRange;
use tsox_core::tspath::{file_extension_is_one_of, SUPPORTED_JS_EXTENSIONS_FLAT};
use tsox_frontend::ast::node::{ModifierList, Node, NodeList, SourceFile};
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::subtree_facts::SubtreeFacts;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::*;
use tsox_frontend::ast::{node_data_generated::*, *};
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::scanner::TOKEN_FLAGS_NONE;
use tsox_checker::binder::referenceresolver::ReferenceResolver;
use crate::mig::m4k::r39k02_defs::R39K02NodeExt;

#[path = "r37k3_defs.rs"]
pub mod r37k3_defs;
use crate::mig::m4j_2::r37k3_defs::{CjsNodeVisitor, update_source_file_node};
use crate::printer::NodeFactory;

use crate::mig::m4e::r37k1_defs::R37K1DataExt;
use crate::mig::m4e::r39k01_defs::R39K01DataExt;
use crate::mig::m4e_2::{FlattenLevel, flatten_destructuring_assignment};
use crate::mig::m4j::r36k3_defs::R36K3NodeAccessExt;
use crate::mig::m4j::r39k03_defs::R39K03NodeExt;
use crate::mig::m4k::R37K2NodeVisitorExt;
use crate::mig::m4k_2::r37k8_defs::split_custom_prologue_r37k8;

use crate::mig::m4k_2::{
    Transformer, get_external_module_name_literal, is_declaration_name_of_enum_or_namespace,
    is_file_level_reserved_generated_identifier, rewrite_module_specifier,
};
use crate::mig::m4k_3::{
    collect_external_module_info, create_external_helpers_import_declaration_if_needed,
    get_export_needs_import_star_helper, get_import_needs_import_default_helper,
    get_import_needs_import_star_helper, ExternalModuleInfo,
};
use crate::mig::m4m_2::{
    convert_variable_declaration_to_assignment_expression, is_export_name, is_generated_identifier,
    is_helper_name, is_identifier_reference, is_local_name, is_simple_inlineable_expression,
};
use tsox_frontend::ast::mig::m3f_4::is_destructuring_assignment;
use tsox_frontend::ast::mig::m3g_3::should_transform_import_call;
use tsox_frontend::ast::dynamic_imports::is_require_call;
use tsox_frontend::ast::visitor::NodeVisitor;
use crate::printer::generated_identifier_flags::{
    AutoGenerateOptions, GeneratedIdentifierFlags,
};
use crate::printer::EmitContext;

pub struct ModifierVisitor {
    pub emit_context: Arc<EmitContext>,
    pub allowed_modifiers: ModifierFlags,
}

impl ModifierVisitor {
    pub fn visit(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let flags = modifier_to_flag(node.kind);
        if !flags.is_empty() && !flags.intersects(self.allowed_modifiers) {
            return None;
        }
        Some(node.clone())
    }

    pub fn visit_modifiers(&mut self, modifiers: &ModifierList) -> Option<ModifierList> {
        let visited: Vec<Arc<Node>> = modifiers
            .list
            .nodes
            .iter()
            .filter_map(|n| self.visit(n))
            .collect();
        Some(ModifierList::new(visited, modifiers.modifier_flags))
    }
}

pub fn extract_modifiers(
    emit_context: &Arc<EmitContext>,
    modifiers: Option<&ModifierList>,
    allowed: ModifierFlags,
) -> Option<ModifierList> {
    let modifiers = modifiers?;
    let mut tx = ModifierVisitor {
        emit_context: emit_context.clone(),
        allowed_modifiers: allowed,
    };
    tx.visit_modifiers(modifiers)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CjsVisitorKind {
    TopLevel,
    TopLevelNested,
    DiscardedValue,
    AssignmentPattern,
}

pub struct CommonJsModuleTransformer {
    pub emit_context: Arc<EmitContext>,
    pub compiler_options: Arc<CompilerOptions>,
    pub resolver: Arc<dyn ReferenceResolver>,
    pub get_emit_module_format_of_file: Box<dyn Fn(&Arc<Node>) -> ModuleKind>,
    pub module_kind: ModuleKind,
    pub language_version: ScriptTarget,
    pub current_source_file: Option<Arc<Node>>,
    pub current_module_info: Option<ExternalModuleInfo>,
    pub parent_node: Option<Arc<Node>>,
    pub current_node: Option<Arc<Node>>,
}

pub fn new_common_js_module_transformer(
    emit_context: Arc<EmitContext>,
    compiler_options: Arc<CompilerOptions>,
    resolver: Arc<dyn ReferenceResolver>,
    get_emit_module_format_of_file: Box<dyn Fn(&Arc<Node>) -> ModuleKind>,
) -> CommonJsModuleTransformer {
    let language_version = compiler_options.get_emit_script_target();
    let module_kind = compiler_options.get_emit_module_kind();
    CommonJsModuleTransformer {
        emit_context,
        compiler_options,
        resolver,
        get_emit_module_format_of_file,
        module_kind,
        language_version,
        current_source_file: None,
        current_module_info: None,
        parent_node: None,
        current_node: None,
    }
}

impl CommonJsModuleTransformer {
    fn factory(&self) -> NodeFactory<'_> {
        NodeFactory::new(&self.emit_context)
    }

    fn visitor(&mut self) -> CjsNodeVisitor<'_> {
        CjsNodeVisitor {
            tx: self,
            kind: CjsVisitorKind::TopLevel,
        }
    }

    fn assignment_pattern_visitor(&mut self) -> CjsNodeVisitor<'_> {
        CjsNodeVisitor {
            tx: self,
            kind: CjsVisitorKind::AssignmentPattern,
        }
    }

    fn discarded_value_visitor(&mut self) -> CjsNodeVisitor<'_> {
        CjsNodeVisitor {
            tx: self,
            kind: CjsVisitorKind::DiscardedValue,
        }
    }

    fn top_level_nested_visitor(&mut self) -> CjsNodeVisitor<'_> {
        CjsNodeVisitor {
            tx: self,
            kind: CjsVisitorKind::TopLevelNested,
        }
    }

    fn visit_with(
        &mut self,
        kind: CjsVisitorKind,
        node: &Arc<Node>,
        visit: impl FnOnce(&mut Self, &Arc<Node>) -> Option<Arc<Node>>,
    ) -> Option<Arc<Node>> {
        let grandparent_node = self.push_node(node);
        let result = visit(self, node);
        self.pop_node(grandparent_node);
        result
    }

    fn visit_slice_with(
        &mut self,
        kind: CjsVisitorKind,
        nodes: &[Arc<Node>],
    ) -> (Vec<Arc<Node>>, bool) {
        let mut changed = false;
        let mut result = Vec::with_capacity(nodes.len());
        for node in nodes {
            let mut visitor = CjsNodeVisitor { tx: self, kind };
            match visitor.visit_node(Some(node)) {
                Some(visited) => {
                    changed |= !Arc::ptr_eq(&visited, node);
                    result.push(visited);
                }
                None => changed = true,
            }
        }
        (result, changed)
    }

    fn visit_node_with(&mut self, kind: CjsVisitorKind, node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut visitor = CjsNodeVisitor { tx: self, kind };
        visitor.visit_node(Some(node))
    }

    pub fn push_node(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let grandparent_node = self.parent_node.clone();
        self.parent_node = self.current_node.clone();
        self.current_node = Some(node.clone());
        grandparent_node
    }

    pub fn pop_node(&mut self, grandparent_node: Option<Arc<Node>>) {
        self.current_node = self.parent_node.clone();
        self.parent_node = grandparent_node;
    }

    pub fn visit(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let grandparent_node = self.push_node(node);
        let result = self.visit_no_stack(node, false);
        self.pop_node(grandparent_node);
        result
    }

    pub fn visit_no_stack(&mut self, node: &Arc<Node>, result_is_discarded: bool) -> Option<Arc<Node>> {
        if !is_source_file(node)
            && !node
                .subtree_facts()
                .intersects(SubtreeFacts::DynamicImport | SubtreeFacts::Identifier)
        {
            return Some(node.clone());
        }

        match node.kind {
            SyntaxKind::SourceFile => self.visit_source_file(node),
            SyntaxKind::ForStatement => self.visit_for_statement(node),
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => {
                self.visit_for_in_or_of_statement(node)
            }
            SyntaxKind::ExpressionStatement => self.visit_expression_statement(node),
            SyntaxKind::VoidExpression => self.visit_void_expression(node),
            SyntaxKind::ParenthesizedExpression => {
                self.visit_parenthesized_expression(node, result_is_discarded)
            }
            SyntaxKind::PartiallyEmittedExpression => {
                self.visit_partially_emitted_expression(node, result_is_discarded)
            }
            SyntaxKind::CallExpression => self.visit_call_expression(node),
            SyntaxKind::TaggedTemplateExpression => self.visit_tagged_template_expression(node),
            SyntaxKind::BinaryExpression => self.visit_binary_expression(node, result_is_discarded),
            SyntaxKind::PrefixUnaryExpression => {
                self.visit_prefix_unary_expression(node, result_is_discarded)
            }
            SyntaxKind::PostfixUnaryExpression => {
                self.visit_postfix_unary_expression(node, result_is_discarded)
            }
            SyntaxKind::ShorthandPropertyAssignment => {
                self.visit_shorthand_property_assignment(node)
            }
            SyntaxKind::Identifier => self.visit_identifier(node),
            _ => self.visitor().visit_each_child(node),
        }
    }

    fn visit_source_file(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if node.is_declaration_file()
            || !(node.is_effective_external_module_node(&self.compiler_options)
                || node.subtree_facts().intersects(SubtreeFacts::DynamicImport))
        {
            return Some(node.clone());
        }

        self.current_source_file = Some(node.clone());
        self.current_module_info = Some(collect_external_module_info(
            node,
            &self.compiler_options,
            (*self.emit_context).clone(),
            self.resolver.clone(),
        ));
        let updated = self.transform_common_js_module(node);
        self.current_source_file = None;
        self.current_module_info = None;
        Some(updated)
    }

    pub fn visit_discarded_value(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let grandparent_node = self.push_node(node);
        let result = self.visit_no_stack(node, true);
        self.pop_node(grandparent_node);
        result
    }

    pub fn visit_assignment_pattern(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let grandparent_node = self.push_node(node);
        let result = self.visit_assignment_pattern_no_stack(node);
        self.pop_node(grandparent_node);
        result
    }

    pub fn visit_assignment_pattern_no_stack(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        match node.kind {
            SyntaxKind::ObjectLiteralExpression | SyntaxKind::ArrayLiteralExpression => {
                self.visit_with(CjsVisitorKind::AssignmentPattern, node, |tx, n| {
                    tx.assignment_pattern_visitor_visit_each_child(n)
                })
            }
            SyntaxKind::PropertyAssignment => self.visit_assignment_property(node),
            SyntaxKind::ShorthandPropertyAssignment => {
                self.visit_shorthand_assignment_property(node)
            }
            SyntaxKind::SpreadAssignment => self.visit_assignment_rest_property(node),
            SyntaxKind::SpreadElement => self.visit_assignment_rest_element(node),
            _ => {
                if is_expression(node) {
                    self.visit_assignment_element(node)
                } else {
                    self.visit_no_stack(node, false)
                }
            }
        }
    }

    fn assignment_pattern_visitor_visit_each_child(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        self.assignment_pattern_visitor().visit_each_child(node)
    }

    pub fn should_emit_underscore_underscore_es_module(&self) -> bool {
        self.current_module_info
            .as_ref()
            .unwrap()
            .export_equals
            .is_none()
            && self
                .current_source_file
                .as_ref()
                .unwrap()
                .is_effective_external_module_node(&self.compiler_options)
    }
    pub fn create_underscore_underscore_es_module(&mut self) -> Arc<Node> {
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
                self.factory().new_node_list(vec![
                    self.factory().new_identifier("exports"),
                    self.factory()
                        .new_string_literal("__esModule", TOKEN_FLAGS_NONE),
                    self.factory().new_object_literal_expression(
                        &self.factory().new_node_list(vec![self.factory().new_property_assignment(
                            None,
                            &self.factory().new_identifier("value"),
                            None,
                            None,
                            &self.factory().new_true_expression(),
                        )]),
                        false,
                    ),
                ]),
                NodeFlags::empty(),
            ),
        );
        self.emit_context
            .add_emit_flags(&statement, EmitFlags::CUSTOM_PROLOGUE);
        statement
    }

    pub fn transform_common_js_module(&mut self, node: &Arc<Node>) -> Arc<Node> {
        Arc::get_mut(&mut self.emit_context)
            .expect("emit context uniquely owned")
            .start_variable_environment();

        let file_data = node.as_source_file();
        let (prologue, mut rest) = self
            .factory()
            .split_standard_prologue(&file_data.statements.nodes);
        let mut statements: Vec<Arc<Node>> = prologue.clone();

        let (custom, rest) = split_custom_prologue_r37k8(&rest);
        let (visited_custom, _) = self.visit_slice_with(CjsVisitorKind::TopLevel, &custom);
        statements.extend(visited_custom);

        if self.should_emit_underscore_underscore_es_module() {
            statements.push(self.create_underscore_underscore_es_module());
        }

        let exported_names = self.current_module_info.as_ref().unwrap().exported_names.clone();
        if !exported_names.is_empty() {
            const CHUNK_SIZE: usize = 50;
            let l = exported_names.len();
            let mut i = 0;
            while i < l {
                let mut right = self.factory().new_void_zero_expression();
                for next_id in &exported_names[i..(i + CHUNK_SIZE).min(l)] {
                    let left = if next_id.kind == SyntaxKind::StringLiteral {
                        self.factory().new_element_access_expression(
                            &self.factory().new_identifier("exports"),
                            None,
                            &new_string_literal_from_node_r40k08(&self.factory(), next_id),
                            NodeFlags::empty(),
                        )
                    } else {
                        let name = next_id.clone();
                        Arc::get_mut(&mut self.emit_context)
                            .expect("emit context uniquely owned")
                            .set_emit_flags(
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
                i += CHUNK_SIZE;
            }
        }

        let exported_functions_start = statements.len();
        let exported_functions = self
            .current_module_info
            .as_ref()
            .unwrap()
            .exported_functions
            .clone();
        for f in exported_functions {
            statements = self.append_exports_of_class_or_function_declaration(statements, &f);
        }
        for s in statements[exported_functions_start..].to_vec() {
            self.emit_context.add_emit_flags(&s, EmitFlags::CUSTOM_PROLOGUE);
        }

        let (visited_rest, _) = self.visit_slice_with(CjsVisitorKind::TopLevel, &rest);
        statements.extend(visited_rest);

        statements = self.append_export_equals_if_needed(statements);

        statements = Arc::get_mut(&mut self.emit_context)
            .expect("emit context uniquely owned")
            .end_and_merge_variable_environment(&statements)
            .0;

        let mut statement_list = NodeList::new(statements);
        statement_list.loc = file_data.statements.loc;
        let mut result = update_source_file_node(
            node,
            Arc::new(statement_list),
            file_data.end_of_file_token.clone(),
        );
        let helpers = Arc::get_mut(&mut self.emit_context)
            .expect("emit context uniquely owned")
            .read_emit_helpers();
        Arc::get_mut(&mut self.emit_context)
            .expect("emit context uniquely owned")
            .add_emit_helper(&result, &helpers);

        let external_helpers_import_declaration = create_external_helpers_import_declaration_if_needed(
            (*self.emit_context).clone(),
            &result,
            &self.compiler_options,
            (self.get_emit_module_format_of_file)(node),
            false,
            false,
            false,
        );
        if let Some(external_helpers_import_declaration) = external_helpers_import_declaration {
            let result_data = result.as_source_file();
            let (prologue, rest) = self
                .factory()
                .split_standard_prologue(&result_data.statements.nodes);
            let (custom, rest) = split_custom_prologue_r37k8(&rest);
            let mut statements = prologue.clone();
            statements.extend(custom);
            statements.push(self.visit_node_with(
                CjsVisitorKind::TopLevel,
                &external_helpers_import_declaration,
            ).unwrap());
            statements.extend(rest);
            let mut statement_list = NodeList::new(statements);
            statement_list.loc = result_data.statements.loc;
            result = update_source_file_node(
                &result,
                Arc::new(statement_list),
                file_data.end_of_file_token.clone(),
            );
        }

        result
    }

    pub fn append_export_equals_if_needed(
        &mut self,
        mut statements: Vec<Arc<Node>>,
    ) -> Vec<Arc<Node>> {
        if let Some(export_equals) = self
            .current_module_info
            .as_ref()
            .unwrap()
            .export_equals
            .clone()
        {
            if let Some(expression_result) = self.visit_export_equals(&export_equals) {
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

    pub fn visit_export_equals(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let grandparent_node = self.push_node(node);
        let result = self.visitor().visit_node(node.expression());
        self.pop_node(grandparent_node);
        result
    }

    pub fn append_exports_of_import_declaration(
        &mut self,
        mut statements: Vec<Arc<Node>>,
        decl: &Arc<Node>,
    ) -> Vec<Arc<Node>> {
        if self
            .current_module_info
            .as_ref()
            .unwrap()
            .export_equals
            .is_some()
        {
            return statements;
        }

        let Some(import_clause) = decl.as_import_declaration().import_clause.clone() else {
            return statements;
        };

        let mut seen = HashSet::new();
        if import_clause.name().is_some() {
            statements = self.append_exports_of_declaration(statements, &import_clause, &mut seen, false);
        }

        let named_bindings = import_clause.as_import_clause().named_bindings.clone();
        if let Some(named_bindings) = named_bindings {
            match named_bindings.kind {
                SyntaxKind::NamespaceImport => {
                    statements = self.append_exports_of_declaration(
                        statements,
                        &named_bindings,
                        &mut seen,
                        false,
                    );
                }
                SyntaxKind::NamedImports => {
                    for import_binding in named_bindings
                        .as_named_imports()
                        .elements
                        .nodes
                        .iter()
                    {
                        statements = self.append_exports_of_declaration(
                            statements,
                            import_binding,
                            &mut seen,
                            true,
                        );
                    }
                }
                _ => {}
            }
        }

        statements
    }

    pub fn append_exports_of_variable_statement(
        &mut self,
        statements: Vec<Arc<Node>>,
        node: &Arc<Node>,
    ) -> Vec<Arc<Node>> {
        self.append_exports_of_variable_declaration_list(
            statements,
            &node.as_variable_statement().declaration_list,
            false,
        )
    }

    pub fn append_exports_of_variable_declaration_list(
        &mut self,
        mut statements: Vec<Arc<Node>>,
        node: &Arc<Node>,
        is_for_in_or_of_initializer: bool,
    ) -> Vec<Arc<Node>> {
        if self
            .current_module_info
            .as_ref()
            .unwrap()
            .export_equals
            .is_some()
        {
            return statements;
        }

        for decl in node.as_variable_declaration_list().declarations.nodes.iter() {
            statements =
                self.append_exports_of_binding_element(statements, &decl, is_for_in_or_of_initializer);
        }

        statements
    }

    pub fn append_exports_of_binding_element(
        &mut self,
        mut statements: Vec<Arc<Node>>,
        decl: &Arc<Node>,
        is_for_in_or_of_initializer: bool,
    ) -> Vec<Arc<Node>> {
        if self
            .current_module_info
            .as_ref()
            .unwrap()
            .export_equals
            .is_some()
            || decl.name().is_none()
        {
            return statements;
        }

        let Some(name) = decl.name() else {
            return statements;
        };
        if is_binding_pattern(name) {
            for element in name.as_binding_pattern().elements.nodes.iter() {
                if !is_omitted_expression(element) {
                    statements = self.append_exports_of_binding_element(
                        statements,
                        element,
                        is_for_in_or_of_initializer,
                    );
                }
            }
        } else if !is_generated_identifier(&self.emit_context, name)
            && (!is_variable_declaration(decl)
                || decl.initializer().is_some()
                || is_for_in_or_of_initializer)
        {
            let mut seen = HashSet::new();
            statements = self.append_exports_of_declaration(statements, decl, &mut seen, false);
        }

        statements
    }

    pub fn append_exports_of_class_or_function_declaration(
        &mut self,
        mut statements: Vec<Arc<Node>>,
        decl: &Arc<Node>,
    ) -> Vec<Arc<Node>> {
        if self
            .current_module_info
            .as_ref()
            .unwrap()
            .export_equals
            .is_some()
        {
            return statements;
        }

        let mut seen = HashSet::new();
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
                &export_name,
                &export_value,
                Some(decl.loc),
                false,
                false,
            );
        }

        if decl.name().is_some() {
            return self.append_exports_of_declaration(statements, decl, &mut seen, false);
        }

        statements
    }

    pub fn append_exports_of_declaration(
        &mut self,
        mut statements: Vec<Arc<Node>>,
        decl: &Arc<Node>,
        seen: &mut HashSet<String>,
        live_binding: bool,
    ) -> Vec<Arc<Node>> {
        if self
            .current_module_info
            .as_ref()
            .unwrap()
            .export_equals
            .is_some()
        {
            return statements;
        }

        let name = decl.name();
        if self
            .current_module_info
            .as_ref()
            .unwrap()
            .export_specifiers
            .len()
            > 0
        {
            if let Some(name_node) = name {
                if is_identifier(name_node) {
                    let name = self.factory().get_declaration_name(decl);
                    let export_specifiers = self
                        .current_module_info
                        .as_ref()
                        .unwrap()
                        .export_specifiers
                        .get(name.text())
                        .cloned()
                        .unwrap_or_default();
                    if !export_specifiers.is_empty() {
                        let export_value = self.visit_expression_identifier(&name).unwrap();
                        for export_specifier in &export_specifiers {
                            statements = self.append_export_statement(
                                statements,
                                seen,
                                export_specifier.name().unwrap(),
                                &export_value,
                                Some(export_specifier.name().unwrap().loc),
                                false,
                                live_binding,
                            );
                        }
                    }
                }
            }
        }

        statements
    }

    pub fn append_export_statement(
        &mut self,
        mut statements: Vec<Arc<Node>>,
        seen: &mut HashSet<String>,
        export_name: &Arc<Node>,
        expression: &Arc<Node>,
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

    pub fn create_export_statement(
        &mut self,
        name: &Arc<Node>,
        value: &Arc<Node>,
        location: Option<TextRange>,
        allow_comments: bool,
        live_binding: bool,
    ) -> Arc<Node> {
        let expression = self.create_export_expression(name, value, None, live_binding);
        let statement = self.factory().new_expression_statement(&expression);
        if let Some(location) = location {
            Arc::get_mut(&mut self.emit_context)
                .expect("emit context uniquely owned")
                .set_comment_range(&statement, location);
        }
        self.emit_context
            .add_emit_flags(&statement, EmitFlags::START_ON_NEW_LINE);
        if !allow_comments {
            self.emit_context
                .add_emit_flags(&statement, EmitFlags::NO_COMMENTS);
        }
        statement
    }

    pub fn create_export_expression(
        &mut self,
        name: &Arc<Node>,
        value: &Arc<Node>,
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
                self.factory().new_node_list(vec![
                    self.factory().new_identifier("exports"),
                    new_string_literal_from_node_r40k08(&self.factory(), name),
                    self.factory().new_object_literal_expression(
                        &self.factory().new_node_list(vec![
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
                                    &self.factory().new_node_list(vec![]),
                                    None,
                                    None,
                                    &self.factory().new_block(
                                        &self.factory().new_node_list(vec![
                                            self.factory().new_return_statement(Some(value)),
                                        ]),
                                        false,
                                    ),
                                ),
                            ),
                        ]),
                        false,
                    ),
                ]),
                NodeFlags::empty(),
            )
        } else {
            let left = if name.kind == SyntaxKind::StringLiteral {
                self.factory().new_element_access_expression(
                    &self.factory().new_identifier("exports"),
                    None,
                    &new_string_literal_from_node_r40k08(&self.factory(), name),
                    NodeFlags::empty(),
                )
            } else {
                self.factory().new_property_access_expression(
                    &self.factory().new_identifier("exports"),
                    None,
                    name,
                    NodeFlags::empty(),
                )
            };
            self.factory().new_assignment_expression(&left, value)
        };
        if let Some(location) = location {
            Arc::get_mut(&mut self.emit_context)
                .expect("emit context uniquely owned")
                .set_comment_range(&expression, location);
        }
        expression
    }

    pub fn create_require_call(&mut self, node: &Arc<Node>) -> Arc<Node> {
        let mut args: Vec<Arc<Node>> = Vec::new();
        let module_name = get_external_module_name_literal(
            &self.factory(),
            node,
            Some(self.current_source_file.as_ref().unwrap()),
            None,
            None,
            &self.compiler_options,
        );
        if let Some(module_name) = module_name {
            if let Some(specifier) = rewrite_module_specifier(
                (*self.emit_context).clone(),
                Some(&module_name),
                &self.compiler_options,
            ) {
                args.push(specifier);
            }
        }
        self.factory().new_call_expression(
            &self.factory().new_identifier("require"),
            None,
            None,
            self.factory().new_node_list(args),
            NodeFlags::empty(),
        )
    }

    pub fn get_helper_expression_for_export(
        &mut self,
        node: &Arc<Node>,
        inner_expr: Arc<Node>,
    ) -> Arc<Node> {
        if get_export_needs_import_star_helper(node) {
            let helper = self.factory().new_import_star_helper(&inner_expr);
            return self.visitor().visit_node(Some(&helper)).unwrap();
        }
        inner_expr
    }

    pub fn get_helper_expression_for_import(
        &mut self,
        node: &Arc<Node>,
        inner_expr: Arc<Node>,
    ) -> Arc<Node> {
        if get_import_needs_import_star_helper(node) {
            let helper = self.factory().new_import_star_helper(&inner_expr);
            return self.visitor().visit_node(Some(&helper)).unwrap();
        }
        if get_import_needs_import_default_helper(node) {
            let helper = self.factory().new_import_default_helper(&inner_expr);
            return self.visitor().visit_node(Some(&helper)).unwrap();
        }
        inner_expr
    }

    pub fn transform_initialized_variable(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if node.initializer().is_none() {
            return None;
        }
        let name = node.name()?;
        if is_binding_pattern(name) {
            let assignment = convert_variable_declaration_to_assignment_expression(
                &self.emit_context,
                node,
            )
            .unwrap();
            let grandparent_node = self.push_node(&assignment);
            let result = self.visit_destructuring_assignment(&assignment, true);
            self.pop_node(grandparent_node);
            return result;
        }
        let property_access = self.factory().new_property_access_expression(
            &self.factory().new_identifier("exports"),
            None,
            name,
            NodeFlags::empty(),
        );
        self.emit_context
            .assign_comment_and_source_map_ranges(&property_access, name);
        Some(self.factory().new_assignment_expression(&property_access, node.initializer().unwrap()))
    }

    pub fn destructuring_needs_flattening(&mut self, node: &Arc<Node>) -> bool {
        if is_object_literal_expression(node) {
            for elem in node.properties() {
                match elem.kind {
                    SyntaxKind::PropertyAssignment => {
                        if let Some(init) = elem.initializer()
                            && self.destructuring_needs_flattening(init)
                        {
                            return true;
                        }
                    }
                    SyntaxKind::ShorthandPropertyAssignment => {
                        if let Some(name) = elem.name()
                            && self.destructuring_needs_flattening(name)
                        {
                            return true;
                        }
                    }
                    SyntaxKind::SpreadAssignment => {
                        if let Some(expr) = elem.expression()
                            && self.destructuring_needs_flattening(expr)
                        {
                            return true;
                        }
                    }
                    SyntaxKind::MethodDeclaration
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor => return false,
                    _ => {}
                }
            }
        } else if is_array_literal_expression(node) {
            for elem in node.as_array_literal_expression().elements.nodes.iter() {
                if is_spread_element(elem) {
                    if let Some(expr) = elem.expression()
                        && self.destructuring_needs_flattening(expr)
                    {
                        return true;
                    }
                } else if self.destructuring_needs_flattening(elem) {
                    return true;
                }
            }
        } else if is_identifier(node) {
            let exported_names = self.get_exports(node).unwrap_or_default();
            if is_export_name(&self.emit_context, node) {
                return exported_names.len() > 1;
            }
            if exported_names.is_empty() {
                return false;
            }
            if exported_names.len() == 1
                && self.is_direct_export(node)
                && exported_names[0].text() == node.text()
            {
                return false;
            }
            return true;
        }
        false
    }

    pub fn create_all_export_expressions(
        &mut self,
        name: &Arc<Node>,
        value: &Arc<Node>,
        location: Option<TextRange>,
    ) -> Arc<Node> {
        let exported_names = self.get_exports(name).unwrap_or_default();
        if !exported_names.is_empty() {
            let mut expression = if self.is_direct_export(name) {
                let export_name = name.clone();
                self.emit_context.add_emit_flags(
                    &export_name,
                    EmitFlags::NO_COMMENTS | EmitFlags::NO_SOURCE_MAP,
                );
                let property_access = self.factory().new_property_access_expression(
                    &self.factory().new_identifier("exports"),
                    None,
                    &export_name,
                    NodeFlags::empty(),
                );
                self.emit_context
                    .add_emit_flags(&property_access, EmitFlags::NO_COMMENTS);
                let expression = self.factory().new_assignment_expression(&property_access, value);
                self.emit_context
                    .assign_comment_and_source_map_ranges(&expression, name);
                expression
            } else {
                self.factory().new_assignment_expression(name, value)
            };
            for export_name in &exported_names {
                expression =
                    self.create_export_expression(export_name, &expression, location, false);
            }
            return expression;
        }
        if self.is_direct_export(name) {
            let export_name = name.clone();
            self.emit_context.add_emit_flags(
                &export_name,
                EmitFlags::NO_COMMENTS | EmitFlags::NO_SOURCE_MAP,
            );
            let property_access = self.factory().new_property_access_expression(
                &self.factory().new_identifier("exports"),
                None,
                &export_name,
                NodeFlags::empty(),
            );
            self.emit_context
                .add_emit_flags(&property_access, EmitFlags::NO_COMMENTS);
            let result = self.factory().new_assignment_expression(&property_access, value);
            self.emit_context.assign_comment_and_source_map_ranges(&result, name);
            return result;
        }
        self.factory().new_assignment_expression(name, value)
    }

    pub fn is_direct_export(&mut self, name: &Arc<Node>) -> bool {
        let original = self.emit_context.most_original(name);
        let export_container = self.resolver.get_referenced_export_container(&original, false);
        export_container.is_some_and(|c| is_source_file(&c))
    }

    pub fn get_exports(&mut self, name: &Arc<Node>) -> Option<Vec<Arc<Node>>> {
        if !is_generated_identifier(&self.emit_context, name) {
            let original = self.emit_context.most_original(name);
            let import_declaration = self.resolver.get_referenced_import_declaration(&original);
            if let Some(import_declaration) = import_declaration {
                return Some(
                    self.current_module_info
                        .as_ref()
                        .unwrap()
                        .exported_bindings
                        .iter()
                        .filter(|(k, _)| Arc::ptr_eq(k, &import_declaration))
                        .map(|(_, v)| v.clone())
                        .collect(),
                );
            }

            let mut bindings_set: HashSet<usize> = HashSet::new();
            let mut bindings: Vec<Arc<Node>> = Vec::new();
            let declarations = self.resolver.get_referenced_value_declarations(&original);
            for declaration in &declarations {
                let exported_bindings = self
                    .current_module_info
                    .as_ref()
                    .unwrap()
                    .exported_bindings
                    .iter()
                    .filter(|(k, _)| Arc::ptr_eq(k, declaration))
                    .map(|(_, v)| v.clone())
                    .collect::<Vec<Arc<Node>>>();
                for binding in exported_bindings {
                    if !bindings_set.contains(&(Arc::as_ptr(&binding) as usize)) {
                        bindings_set.insert(Arc::as_ptr(&binding) as usize);
                        bindings.push(binding);
                    }
                }
            }
            return Some(bindings);
        } else if is_file_level_reserved_generated_identifier(
            (*self.emit_context).clone(),
            name,
        ) {
            let export_specifiers = self
                .current_module_info
                .as_ref()
                .unwrap()
                .export_specifiers
                .get(name.text())
                .cloned();
            if let Some(export_specifiers) = export_specifiers {
                let mut exported_names = Vec::new();
                for export_specifier in &export_specifiers {
                    if let Some(n) = export_specifier.name() {
                        exported_names.push(n.clone());
                    }
                }
                return Some(exported_names);
            }
        }
        None
    }

    pub fn visit_for_statement(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let (initializer, condition, incrementor, statement) = match &node.data {
            NodeData::ForStatement(d) => (
                d.initializer.clone(),
                d.condition.clone(),
                d.incrementor.clone(),
                d.statement.clone(),
            ),
            _ => return Some(node.clone()),
        };
        let initializer = initializer.map(|i| {
            self.discarded_value_visitor()
                .visit_node(Some(&i))
                .unwrap_or(i)
        });
        let condition = condition.and_then(|c| self.visitor().visit_node(Some(&c)));
        let incrementor = incrementor.map(|i| {
            self.discarded_value_visitor()
                .visit_node(Some(&i))
                .unwrap_or(i)
        });
        let body = self.visit_iteration_body(Some(statement));
        Some(update_for_statement_r40k08(
            node,
            initializer.as_ref(),
            condition.as_ref(),
            incrementor.as_ref(),
            &body.unwrap_or_else(|| match &node.data {
                NodeData::ForStatement(d) => d.statement.clone(),
                _ => node.clone(),
            }),
        ))
    }

    pub fn visit_for_in_or_of_statement(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let (await_modifier, initializer, expression, statement) = match &node.data {
            NodeData::ForInOrOfStatement(d) => (
                d.await_modifier.clone(),
                d.initializer.clone(),
                d.expression.clone(),
                d.statement.clone(),
            ),
            _ => return Some(node.clone()),
        };
        let initializer = self
            .discarded_value_visitor()
            .visit_node(Some(&initializer))
            .unwrap_or(initializer);
        let expression = self
            .visitor()
            .visit_node(Some(&expression))
            .unwrap_or(expression);
        let body = self.visit_iteration_body(Some(statement));
        Some(update_for_in_or_of_statement_r40k08(
            node,
            await_modifier,
            initializer,
            expression,
            body.unwrap_or_else(|| match &node.data {
                NodeData::ForInOrOfStatement(d) => d.statement.clone(),
                _ => node.clone(),
            }),
        ))
    }

    fn visit_iteration_body(&mut self, body: Option<Arc<Node>>) -> Option<Arc<Node>> {
        let mut nested_visitor = NodeVisitor::default();
        Arc::get_mut(&mut self.emit_context)
            .expect("emit context uniquely owned")
            .visit_iteration_body(body, &mut nested_visitor)
    }

    pub fn visit_expression_statement(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        self.discarded_value_visitor().visit_each_child(node)
    }

    pub fn visit_void_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        self.discarded_value_visitor().visit_each_child(node)
    }

    pub fn visit_parenthesized_expression(
        &mut self,
        node: &Arc<Node>,
        result_is_discarded: bool,
    ) -> Option<Arc<Node>> {
        let expression = match &node.data {
            NodeData::ParenthesizedExpression(d) => d.expression.clone(),
            _ => return Some(node.clone()),
        };
        let expression = if result_is_discarded {
            self.discarded_value_visitor()
                .visit_node(Some(&expression))
                .unwrap_or(expression)
        } else {
            self.visitor().visit_node(Some(&expression)).unwrap_or(expression)
        };
        Some(update_parenthesized_expression_r40k08(node, &expression))
    }

    pub fn visit_partially_emitted_expression(
        &mut self,
        node: &Arc<Node>,
        result_is_discarded: bool,
    ) -> Option<Arc<Node>> {
        let expression = match &node.data {
            NodeData::PartiallyEmittedExpression(d) => d.expression.clone(),
            _ => return Some(node.clone()),
        };
        let expression = if result_is_discarded {
            self.discarded_value_visitor()
                .visit_node(Some(&expression))
                .unwrap_or(expression)
        } else {
            self.visitor().visit_node(Some(&expression)).unwrap_or(expression)
        };
        Some(update_partially_emitted_expression_r40k08(node, &expression))
    }

    pub fn visit_binary_expression(
        &mut self,
        node: &Arc<Node>,
        result_is_discarded: bool,
    ) -> Option<Arc<Node>> {
        if is_destructuring_assignment(node) {
            return self.visit_destructuring_assignment(node, result_is_discarded);
        }
        if is_assignment_expression(node, false) {
            return self.visit_assignment_expression(node);
        }
        if is_comma_expression(node) {
            return self.visit_comma_expression(node, result_is_discarded);
        }
        self.visitor().visit_each_child(node)
    }

    fn visit_assignment_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let (left, _operator_token, _right) = match &node.data {
            NodeData::BinaryExpression(d) => (d.left.clone(), d.operator_token.clone(), d.right.clone()),
            _ => return self.visitor().visit_each_child(node),
        };
        if is_identifier(&left)
            && (!is_generated_identifier(&self.emit_context, &left)
                || is_file_level_reserved_generated_identifier((*self.emit_context).clone(), &left))
            && !is_local_name(&self.emit_context, &left)
        {
            let exported_names = self.get_exports(&left).unwrap_or_default();
            if !exported_names.is_empty() {
                let mut expression = self.visitor().visit_each_child(node)?;
                for export_name in &exported_names {
                    expression =
                        self.create_export_expression(export_name, &expression, Some(node.loc), false);
                }
                return Some(expression);
            }
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_destructuring_assignment(
        &mut self,
        node: &Arc<Node>,
        value_is_discarded: bool,
    ) -> Option<Arc<Node>> {
        let left = match &node.data {
            NodeData::BinaryExpression(d) => d.left.clone(),
            _ => return self.visitor().visit_each_child(node),
        };
        if self.destructuring_needs_flattening(&left) {
            let mut tx = Transformer::new(|_, node| Some(node), Some((*self.emit_context).clone()));
            return flatten_destructuring_assignment(
                &mut tx,
                node.clone(),
                !value_is_discarded,
                FlattenLevel::All,
                None,
            );
        }
        self.visitor().visit_each_child(node)
    }

    fn visit_comma_expression(
        &mut self,
        node: &Arc<Node>,
        result_is_discarded: bool,
    ) -> Option<Arc<Node>> {
        let (left, operator_token, right) = match &node.data {
            NodeData::BinaryExpression(d) => {
                (d.left.clone(), d.operator_token.clone(), d.right.clone())
            }
            _ => return self.visitor().visit_each_child(node),
        };
        let left = self
            .discarded_value_visitor()
            .visit_node(Some(&left))
            .unwrap_or(left);
        let right = if result_is_discarded {
            self.discarded_value_visitor()
                .visit_node(Some(&right))
                .unwrap_or(right)
        } else {
            self.visitor().visit_node(Some(&right)).unwrap_or(right)
        };
        Some(update_binary_expression_r40k08(
            node,
            &left,
            &operator_token,
            &right,
        ))
    }

    pub fn visit_prefix_unary_expression(
        &mut self,
        node: &Arc<Node>,
        result_is_discarded: bool,
    ) -> Option<Arc<Node>> {
        let (operator, operand) = match &node.data {
            NodeData::PrefixUnaryExpression(d) => (d.operator, d.operand.clone()),
            _ => return self.visitor().visit_each_child(node),
        };
        if (operator == SyntaxKind::PlusPlusToken || operator == SyntaxKind::MinusMinusToken)
            && is_identifier(&operand)
            && !is_local_name(&self.emit_context, &operand)
        {
            let exported_names = self.get_exports(&operand).unwrap_or_default();
            if !exported_names.is_empty() {
                let visited = self.visitor().visit_node(Some(&operand)).unwrap_or(operand);
                let mut expression =
                    update_prefix_unary_expression_r40k08(node, operator, &visited);
                for export_name in &exported_names {
                    expression =
                        self.create_export_expression(export_name, &expression, None, false);
                    self.emit_context
                        .assign_comment_and_source_map_ranges(&expression, node);
                }
                return Some(expression);
            }
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_postfix_unary_expression(
        &mut self,
        node: &Arc<Node>,
        result_is_discarded: bool,
    ) -> Option<Arc<Node>> {
        let (operand, operator) = match &node.data {
            NodeData::PostfixUnaryExpression(d) => (d.operand.clone(), d.operator),
            _ => return self.visitor().visit_each_child(node),
        };
        if (operator == SyntaxKind::PlusPlusToken || operator == SyntaxKind::MinusMinusToken)
            && is_identifier(&operand)
            && !is_local_name(&self.emit_context, &operand)
        {
            let exported_names = self.get_exports(&operand).unwrap_or_default();
            if !exported_names.is_empty() {
                let visited = self
                    .visitor()
                    .visit_node(Some(&operand))
                    .unwrap_or_else(|| operand.clone());
                let mut expression =
                    update_postfix_unary_expression_r40k08(node, &visited, operator);
                let mut temp: Option<Arc<Node>> = None;
                if !result_is_discarded {
                    let generated = self.factory().new_temp_variable();
                    let temp_node = self.factory().generated_name_node(&generated);
                    Arc::get_mut(&mut self.emit_context)
                        .expect("emit context uniquely owned")
                        .add_variable_declaration(&temp_node);
                    temp = Some(temp_node);
                    expression = self
                        .factory()
                        .new_assignment_expression(&temp.clone().unwrap(), &expression);
                    self.emit_context
                        .assign_comment_and_source_map_ranges(&expression, node);
                }
                expression = self
                    .factory()
                    .new_comma_expression(&expression, &operand);
                self.emit_context
                    .assign_comment_and_source_map_ranges(&expression, node);
                for export_name in &exported_names {
                    expression =
                        self.create_export_expression(export_name, &expression, None, false);
                    self.emit_context
                        .assign_comment_and_source_map_ranges(&expression, node);
                }
                if let Some(temp) = temp {
                    expression = self.factory().new_comma_expression(&expression, &temp);
                    self.emit_context
                        .assign_comment_and_source_map_ranges(&expression, node);
                }
                return Some(expression);
            }
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_call_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let (expression, question_dot_token, arguments) = match &node.data {
            NodeData::CallExpression(d) => (
                d.expression.clone(),
                d.question_dot_token.clone(),
                d.arguments.clone(),
            ),
            _ => return self.visitor().visit_each_child(node),
        };
        let is_import_call = is_import_call(node);
        let mut needs_rewrite = false;
        if self.compiler_options.rewrite_relative_import_extensions.is_true() {
            if (is_import_call && !arguments.nodes.is_empty())
                || (is_in_js_file(node) && is_require_call(node, false))
            {
                needs_rewrite = true;
            }
        }
        if is_import_call && self.should_transform_import_call() {
            return self.visit_import_call_expression(node, needs_rewrite);
        }
        if needs_rewrite {
            return self.shim_or_rewrite_import_or_require_call(node);
        }
        if is_identifier(&expression) {
            let visited = self.visit_expression_identifier(&expression)?;
            let (visited_args, _) = self.visitor().visit_slice(&arguments.nodes);
            let arguments = Arc::new(NodeList {
                loc: arguments.loc,
                nodes: visited_args,
            });
            let updated = update_call_expression_r40k08(
                node,
                Some(visited.clone()),
                question_dot_token,
                arguments,
            );
            if !is_identifier(&visited) && !is_helper_name(&self.emit_context, &expression) {
                self.emit_context
                    .add_emit_flags(&updated, EmitFlags::INDIRECT_CALL);
            }
            return Some(updated);
        }
        self.visitor().visit_each_child(node)
    }

    fn should_transform_import_call(&self) -> bool {
        let file_node = self.current_source_file.clone();
        should_transform_import_call(
            "",
            &self.compiler_options,
            file_node
                .as_ref()
                .map(|f| (self.get_emit_module_format_of_file)(f))
                .unwrap_or(self.module_kind),
        )
    }

    fn visit_import_call_expression(
        &mut self,
        node: &Arc<Node>,
        rewrite_or_shim: bool,
    ) -> Option<Arc<Node>> {
        if self.module_kind == ModuleKind::None && self.language_version >= ScriptTarget::ES2020 {
            return self.visitor().visit_each_child(node);
        }
        let current = self.current_source_file.clone();
        let arguments = match &node.data {
            NodeData::CallExpression(d) => d.arguments.clone(),
            _ => return self.visitor().visit_each_child(node),
        };
        let external_module_name = get_external_module_name_literal(
            &self.factory(),
            node,
            current.as_ref(),
            None,
            None,
            &self.compiler_options,
        );
        let first_argument = arguments.nodes.first().and_then(|a| {
            self.visitor().visit_node(Some(a))
        });
        let argument = if external_module_name.is_some()
            && first_argument
                .as_ref()
                .map_or(true, |a| !is_string_literal(a) || a.text() != external_module_name.as_ref().unwrap().text())
        {
            external_module_name
        } else if let Some(first_argument) = first_argument {
            if rewrite_or_shim {
                if is_string_literal(&first_argument) {
                    rewrite_module_specifier(
                        (*self.emit_context).clone(),
                        Some(&first_argument),
                        &self.compiler_options,
                    )
                } else {
                    Some(self.factory().new_rewrite_relative_import_extensions_helper(
                        first_argument.clone(),
                        self.compiler_options.jsx == JsxEmit::Preserve,
                    ))
                }
            } else {
                Some(first_argument)
            }
        } else {
            None
        };
        Some(self.create_import_call_expression_common_js(argument.as_ref()))
    }

    fn create_import_call_expression_common_js(&mut self, arg: Option<&Arc<Node>>) -> Arc<Node> {
        let need_sync_eval =
            arg.map_or(false, |a| !is_simple_inlineable_expression(a));
        let mut promise_resolve_arguments: Vec<Arc<Node>> = Vec::new();
        if need_sync_eval {
            let span = self.factory().new_template_span(
                arg.unwrap(),
                &self.factory().new_template_tail("", "", TokenFlags::default()),
            );
            promise_resolve_arguments.push(self.factory().new_template_expression(
                &self.factory().new_template_head("", "", TokenFlags::default()),
                Arc::new(NodeList::new(vec![span])),
            ));
        }
        let promise_resolve_call = self.factory().new_call_expression(
            &self.factory().new_property_access_expression(
                &self.factory().new_identifier("Promise"),
                None,
                &self.factory().new_identifier("resolve"),
                NodeFlags::empty(),
            ),
            None,
            None,
            self.factory().new_node_list(promise_resolve_arguments),
            NodeFlags::empty(),
        );
        let require_arguments: Vec<Arc<Node>> = if need_sync_eval {
            vec![self.factory().new_identifier("s")]
        } else {
            arg.map(|a| vec![a.clone()]).unwrap_or_default()
        };
        let require_call = self.factory().new_call_expression(
            &self.factory().new_identifier("require"),
            None,
            None,
            self.factory().new_node_list(require_arguments),
            NodeFlags::empty(),
        );
        let require_call = self.factory().new_import_star_helper(&require_call);
        let mut parameters: Vec<Arc<Node>> = Vec::new();
        if need_sync_eval {
            parameters.push(self.factory().new_parameter_declaration(
                None,
                None,
                &self.factory().new_identifier("s"),
                None,
                None,
                None,
            ));
        }
        let equals_greater_than_token = self.factory().new_token(SyntaxKind::EqualsGreaterThanToken);
        let function = self.factory().new_arrow_function(
            None,
            None,
            &NodeList::new(parameters),
            None,
            None,
            &equals_greater_than_token,
            &require_call,
        );
        self.factory().new_call_expression(
            &self.factory().new_property_access_expression(
                &promise_resolve_call,
                None,
                &self.factory().new_identifier("then"),
                NodeFlags::empty(),
            ),
            None,
            None,
            self.factory().new_node_list(vec![function]),
            NodeFlags::empty(),
        )
    }

    fn shim_or_rewrite_import_or_require_call(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let (expression, question_dot_token, arguments) = match &node.data {
            NodeData::CallExpression(d) => (
                d.expression.clone(),
                d.question_dot_token.clone(),
                d.arguments.clone(),
            ),
            _ => return self.visitor().visit_each_child(node),
        };
        let expression = self.visitor().visit_node(Some(&expression)).unwrap_or(expression);
        let mut arguments_list = arguments.clone();
        if !arguments.nodes.is_empty() {
            let first = self
                .visitor()
                .visit_node(Some(&arguments.nodes[0]))
                .unwrap_or_else(|| arguments.nodes[0].clone());
            let mut first_argument_changed = false;
            let first_argument = if is_string_literal_like(&first) {
                let rewritten = rewrite_module_specifier(
                    (*self.emit_context).clone(),
                    Some(&first),
                    &self.compiler_options,
                )
                .unwrap_or_else(|| first.clone());
                first_argument_changed = !Arc::ptr_eq(&rewritten, &first);
                rewritten
            } else {
                first_argument_changed = true;
                self.factory()
                    .new_rewrite_relative_import_extensions_helper(
                        first.clone(),
                        self.compiler_options.jsx == JsxEmit::Preserve,
                    )
            };
            let (rest, rest_changed) = self.visitor().visit_slice(&arguments.nodes[1..]);
            if first_argument_changed || rest_changed {
                let mut nodes = vec![first_argument];
                nodes.extend(rest);
                let mut list = NodeList::new(nodes);
                list.loc = arguments.loc;
                arguments_list = Arc::new(list);
            }
        }
        Some(update_call_expression_r40k08(
            node,
            Some(expression),
            question_dot_token,
            arguments_list,
        ))
    }

    pub fn visit_tagged_template_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let (tag, question_dot_token, template) = match &node.data {
            NodeData::TaggedTemplateExpression(d) => (
                d.tag.clone(),
                d.question_dot_token.clone(),
                d.template.clone(),
            ),
            _ => return self.visitor().visit_each_child(node),
        };
        if is_identifier(&tag) {
            let visited = self.visit_expression_identifier(&tag)?;
            let template = self.visitor().visit_node(Some(&template)).unwrap_or(template);
            let updated = update_tagged_template_expression_r40k08(
                node,
                &visited,
                question_dot_token,
                &template,
            );
            if !is_identifier(&visited) && !is_helper_name(&self.emit_context, &tag) {
                self.emit_context
                    .add_emit_flags(&updated, EmitFlags::INDIRECT_CALL);
            }
            return Some(updated);
        }
        self.visitor().visit_each_child(node)
    }

    pub fn visit_shorthand_property_assignment(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let (name, equals_token, object_assignment_initializer) = match &node.data {
            NodeData::ShorthandPropertyAssignment(d) => (
                d.name.clone(),
                d.equals_token.clone(),
                d.object_assignment_initializer.clone(),
            ),
            _ => return self.visitor().visit_each_child(node),
        };
        let exported_or_imported_name = self.visit_expression_identifier(&name)?;
        if !Arc::ptr_eq(&exported_or_imported_name, &name) {
            let mut expression = exported_or_imported_name;
            if let Some(object_assignment_initializer) = object_assignment_initializer {
                let visited = self
                    .visitor()
                    .visit_node(Some(&object_assignment_initializer))
                    .unwrap_or(object_assignment_initializer);
                expression = self.factory().new_assignment_expression(&expression, &visited);
            }
            let assignment =
                self.factory()
                    .new_property_assignment(None, &name, None, None, &expression);
            let assignment = set_node_loc(assignment, node.loc);
            self.emit_context
                .assign_comment_and_source_map_ranges(&assignment, node);
            return Some(assignment);
        }
        let object_assignment_initializer = match object_assignment_initializer {
            Some(o) => Some(self.visitor().visit_node(Some(&o)).unwrap_or(o)),
            None => None,
        };
        let _ = equals_token;
        Some(update_shorthand_property_assignment_r40k08(
            node,
            &exported_or_imported_name,
            object_assignment_initializer.as_ref(),
        ))
    }

    pub fn visit_identifier(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if let Some(parent) = node.parent() {
            if is_identifier_reference(node, &parent) {
                return self.visit_expression_identifier(node);
            }
        }
        Some(node.clone())
    }

    pub fn visit_expression_identifier(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let skip_for_generated = self
            .emit_context
            .get_auto_generate_info(node)
            .map_or(false, |info| {
                (info.flags.0 & GeneratedIdentifierFlags::ALLOW_NAME_SUBSTITUTION.0) == 0
            });
        if !skip_for_generated
            && !is_helper_name(&self.emit_context, node)
            && !is_local_name(&self.emit_context, node)
            && !is_declaration_name_of_enum_or_namespace(
                (*self.emit_context).clone(),
                node,
            )
        {
            let original = self.emit_context.most_original(node);
            let prefix_locals = is_export_name(&self.emit_context, node);
            let export_container = self
                .resolver
                .get_referenced_export_container(&original, prefix_locals);
            if let Some(export_container) = export_container {
                if is_source_file(&export_container) {
                    let reference = self.factory().new_property_access_expression(
                        &self.factory().new_identifier("exports"),
                        None,
                        &node.clone(),
                        NodeFlags::empty(),
                    );
                    self.emit_context
                        .assign_comment_and_source_map_ranges(&reference, node);
                    return Some(set_node_loc(reference, node.loc));
                }
            }
            let import_declaration = self.resolver.get_referenced_import_declaration(&original);
            if let Some(import_declaration) = import_declaration {
                if import_declaration.kind == SyntaxKind::ImportClause {
                    let parent = import_declaration.parent().unwrap_or(import_declaration.clone());
                    let generated = self.factory().new_generated_name_for_node(&parent);
                    let target = self.factory().generated_name_node(&generated);
                    let reference = self.factory().new_property_access_expression(
                        &target,
                        None,
                        &self.factory().new_identifier("default"),
                        NodeFlags::empty(),
                    );
                    self.emit_context
                        .assign_comment_and_source_map_ranges(&reference, node);
                    return Some(set_node_loc(reference, node.loc));
                }
                if import_declaration.kind == SyntaxKind::ImportSpecifier {
                    let name = match &import_declaration.data {
                        NodeData::ImportSpecifier(d) => {
                            d.property_name.clone().unwrap_or_else(|| d.name.clone())
                        }
                        _ => return Some(node.clone()),
                    };
                    let decl = find_ancestor_import_declaration(&import_declaration);
                    let target_source = decl.unwrap_or_else(|| import_declaration.clone());
                    let generated = self.factory().new_generated_name_for_node(&target_source);
                    let target = self.factory().generated_name_node(&generated);
                    let reference = if is_string_literal(&name) {
                        self.factory().new_element_access_expression(
                            &target,
                            None,
                            &new_string_literal_from_node_r40k08(&self.factory(), &name),
                            NodeFlags::empty(),
                        )
                    } else {
                        let reference_name = name.clone();
                        self.emit_context.add_emit_flags(
                            &reference_name,
                            EmitFlags::NO_SOURCE_MAP | EmitFlags::NO_COMMENTS,
                        );
                        self.factory().new_property_access_expression(
                            &target,
                            None,
                            &reference_name,
                            NodeFlags::empty(),
                        )
                    };
                    self.emit_context
                        .assign_comment_and_source_map_ranges(&reference, node);
                    return Some(set_node_loc(reference, node.loc));
                }
            }
        }
        Some(node.clone())
    }

    pub fn visit_assignment_property(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let (name, initializer) = match &node.data {
            NodeData::PropertyAssignment(d) => (d.name.clone(), d.initializer.clone()),
            _ => return self.visitor().visit_each_child(node),
        };
        let name = self.visitor().visit_node(Some(&name)).unwrap_or(name);
        let initializer = self
            .assignment_pattern_visitor()
            .visit_node(Some(&initializer))
            .unwrap_or(initializer);
        Some(update_property_assignment_r40k08(
            node,
            &name,
            Some(&initializer),
        ))
    }

    pub fn visit_shorthand_assignment_property(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let (name, equals_token, object_assignment_initializer) = match &node.data {
            NodeData::ShorthandPropertyAssignment(d) => (
                d.name.clone(),
                d.equals_token.clone(),
                d.object_assignment_initializer.clone(),
            ),
            _ => return self.visitor().visit_each_child(node),
        };
        let target = self.visit_destructuring_assignment_target_no_stack(&name)?;
        if is_identifier(&target) {
            let object_assignment_initializer = match object_assignment_initializer {
                Some(o) => Some(self.visitor().visit_node(Some(&o)).unwrap_or(o)),
                None => None,
            };
            let _ = equals_token;
            return Some(update_shorthand_property_assignment_r40k08(
                node,
                &target,
                object_assignment_initializer.as_ref(),
            ));
        }
        if let Some(object_assignment_initializer) = object_assignment_initializer {
            let equals_token = match equals_token {
                Some(t) => t,
                None => self.factory().new_token(SyntaxKind::EqualsToken),
            };
            let visited = self
                .visitor()
                .visit_node(Some(&object_assignment_initializer))
                .unwrap_or(object_assignment_initializer);
            let target = self.factory().new_binary_expression(
                None,
                &target,
                None,
                &equals_token,
                &visited,
            );
            let updated = update_property_assignment_r40k08(node, &name, Some(&target));
            self.emit_context.set_original(&updated, node);
            self.emit_context
                .assign_comment_and_source_map_ranges(&updated, node);
            return Some(updated);
        }
        let updated = update_property_assignment_r40k08(node, &name, None);
        self.emit_context.set_original(&updated, node);
        self.emit_context
            .assign_comment_and_source_map_ranges(&updated, node);
        Some(updated)
    }

    pub fn visit_assignment_rest_property(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let expression = match &node.data {
            NodeData::SpreadAssignment(d) => d.expression.clone(),
            _ => return self.visitor().visit_each_child(node),
        };
        let expression = self
            .visit_destructuring_assignment_target(&expression)?
            .unwrap_or(expression);
        Some(update_spread_assignment_r40k08(node, &expression))
    }

    pub fn visit_assignment_rest_element(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let expression = match &node.data {
            NodeData::SpreadElement(d) => d.expression.clone(),
            _ => return self.visitor().visit_each_child(node),
        };
        let expression = self
            .visit_destructuring_assignment_target(&expression)?
            .unwrap_or(expression);
        Some(update_spread_element_r40k08(node, &expression))
    }

    pub fn visit_assignment_element(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if node.kind == SyntaxKind::BinaryExpression {
            let (left, operator_token, right) = match &node.data {
                NodeData::BinaryExpression(d) => {
                    (d.left.clone(), d.operator_token.clone(), d.right.clone())
                }
                _ => return self.visit_destructuring_assignment_target_no_stack(node),
            };
            if operator_token.kind == SyntaxKind::EqualsToken {
                let left = self
                    .visit_destructuring_assignment_target(&left)?
                    .unwrap_or(left);
                let right = self.visitor().visit_node(Some(&right)).unwrap_or(right);
                return Some(update_binary_expression_r40k08(
                    node,
                    &left,
                    &operator_token,
                    &right,
                ));
            }
        }
        self.visit_destructuring_assignment_target_no_stack(node)
    }

    fn visit_destructuring_assignment_target(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Option<Arc<Node>>> {
        let grandparent_node = self.push_node(node);
        let result = match node.kind {
            SyntaxKind::ObjectLiteralExpression | SyntaxKind::ArrayLiteralExpression => {
                self.visit_assignment_pattern_no_stack(node)
            }
            _ => self.visit_destructuring_assignment_target_no_stack(node),
        };
        self.pop_node(grandparent_node);
        Some(result)
    }

    fn visit_destructuring_assignment_target_no_stack(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Node>> {
        if is_identifier(node)
            && (!is_generated_identifier(&self.emit_context, node)
                || is_file_level_reserved_generated_identifier(
                    (*self.emit_context).clone(),
                    node,
                ))
            && !is_local_name(&self.emit_context, node)
        {
            let mut expression = self.visit_expression_identifier(node)?;
            let exported_names = self.get_exports(node).unwrap_or_default();
            if !exported_names.is_empty() {
                let generated = self.factory().new_unique_name_ex(
                    "value",
                    AutoGenerateOptions {
                        flags: GeneratedIdentifierFlags::OPTIMISTIC,
                        ..Default::default()
                    },
                );
                let value = self.factory().generated_name_node(&generated);
                expression = self
                    .factory()
                    .new_assignment_expression(&expression, &value);
                for export_name in &exported_names {
                    expression =
                        self.create_export_expression(export_name, &expression, None, false);
                }
                let statement = self.factory().new_expression_statement(&expression);
                let statement_list = self.factory().new_node_list(vec![statement]);
                let param = self.factory().new_parameter_declaration(
                    None,
                    None,
                    &value,
                    None,
                    None,
                    None,
                );
                let param_list = self.factory().new_node_list(vec![param]);
                let value_setter = self.factory().new_set_accessor_declaration(
                    None,
                    &self.factory().new_identifier("value"),
                    None,
                    param_list,
                    None,
                    None,
                    self.factory().new_block(&statement_list, false),
                );
                let property_list = self.factory().new_node_list(vec![value_setter]);
                expression = self
                    .factory()
                    .new_object_literal_expression(&property_list, false);
                expression = self.factory().new_property_access_expression(
                    &expression,
                    None,
                    &self.factory().new_identifier("value"),
                    NodeFlags::empty(),
                );
            }
            return Some(expression);
        }
        self.visit_no_stack(node, false)
    }
}

fn find_ancestor_import_declaration(node: &Arc<Node>) -> Option<Arc<Node>> {
    let mut current = node.parent()?;
    loop {
        if current.kind == SyntaxKind::ImportDeclaration {
            return Some(current);
        }
        current = current.parent()?;
    }
}

fn set_node_loc(node: Arc<Node>, loc: TextRange) -> Arc<Node> {
    match Arc::try_unwrap(node) {
        Ok(mut n) => {
            n.loc = loc;
            Arc::new(n)
        }
        Err(shared) => shared,
    }
}

fn new_string_literal_from_node_r40k08(f: &NodeFactory, text_source_node: &Arc<Node>) -> Arc<Node> {
    f.new_string_literal(text_source_node.text(), TOKEN_FLAGS_NONE)
}

fn update_for_statement_r40k08(
    node: &Arc<Node>,
    initializer: Option<&Arc<Node>>,
    condition: Option<&Arc<Node>>,
    incrementor: Option<&Arc<Node>>,
    statement: &Arc<Node>,
) -> Arc<Node> {
    let mut updated = Node::new(
        SyntaxKind::ForStatement,
        NodeData::ForStatement(ForStatementData {
            initializer: initializer.cloned(),
            condition: condition.cloned(),
            incrementor: incrementor.cloned(),
            statement: statement.clone(),
        }),
    );
    updated.loc = node.loc;
    updated.flags = node.flags;
    Arc::new(updated)
}

fn update_for_in_or_of_statement_r40k08(
    node: &Arc<Node>,
    await_modifier: Option<Arc<Node>>,
    initializer: Arc<Node>,
    expression: Arc<Node>,
    statement: Arc<Node>,
) -> Arc<Node> {
    let mut updated = Node::new(
        node.kind,
        NodeData::ForInOrOfStatement(ForInOrOfStatementData {
            await_modifier,
            initializer,
            expression,
            statement,
        }),
    );
    updated.loc = node.loc;
    updated.flags = node.flags;
    Arc::new(updated)
}

fn update_parenthesized_expression_r40k08(node: &Arc<Node>, expression: &Arc<Node>) -> Arc<Node> {
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

fn update_partially_emitted_expression_r40k08(
    node: &Arc<Node>,
    expression: &Arc<Node>,
) -> Arc<Node> {
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

fn update_binary_expression_r40k08(
    node: &Arc<Node>,
    left: &Arc<Node>,
    operator_token: &Arc<Node>,
    right: &Arc<Node>,
) -> Arc<Node> {
    let (modifiers, type_node) = match &node.data {
        NodeData::BinaryExpression(d) => (d.modifiers.clone(), d.type_node.clone()),
        _ => (None, None),
    };
    let mut updated = Node::new(
        SyntaxKind::BinaryExpression,
        NodeData::BinaryExpression(BinaryExpressionData {
            modifiers,
            left: left.clone(),
            type_node,
            operator_token: operator_token.clone(),
            right: right.clone(),
        }),
    );
    updated.loc = node.loc;
    updated.flags = node.flags;
    Arc::new(updated)
}

fn update_prefix_unary_expression_r40k08(
    node: &Arc<Node>,
    operator: SyntaxKind,
    operand: &Arc<Node>,
) -> Arc<Node> {
    let mut updated = Node::new(
        SyntaxKind::PrefixUnaryExpression,
        NodeData::PrefixUnaryExpression(PrefixUnaryExpressionData {
            operator,
            operand: operand.clone(),
        }),
    );
    updated.loc = node.loc;
    updated.flags = node.flags;
    Arc::new(updated)
}

fn update_postfix_unary_expression_r40k08(
    node: &Arc<Node>,
    operand: &Arc<Node>,
    operator: SyntaxKind,
) -> Arc<Node> {
    let mut updated = Node::new(
        SyntaxKind::PostfixUnaryExpression,
        NodeData::PostfixUnaryExpression(PostfixUnaryExpressionData {
            operand: operand.clone(),
            operator,
        }),
    );
    updated.loc = node.loc;
    updated.flags = node.flags;
    Arc::new(updated)
}

fn update_call_expression_r40k08(
    node: &Arc<Node>,
    expression: Option<Arc<Node>>,
    question_dot_token: Option<Arc<Node>>,
    arguments: Arc<NodeList>,
) -> Arc<Node> {
    let type_arguments = match &node.data {
        NodeData::CallExpression(d) => d.type_arguments.clone(),
        _ => None,
    };
    let mut updated = Node::new(
        SyntaxKind::CallExpression,
        NodeData::CallExpression(CallExpressionData {
            expression: expression.unwrap_or_else(|| match &node.data {
                NodeData::CallExpression(d) => d.expression.clone(),
                _ => node.clone(),
            }),
            question_dot_token,
            type_arguments,
            arguments,
        }),
    );
    updated.loc = node.loc;
    updated.flags = node.flags;
    Arc::new(updated)
}

fn update_tagged_template_expression_r40k08(
    node: &Arc<Node>,
    tag: &Arc<Node>,
    question_dot_token: Option<Arc<Node>>,
    template: &Arc<Node>,
) -> Arc<Node> {
    let mut updated = Node::new(
        SyntaxKind::TaggedTemplateExpression,
        NodeData::TaggedTemplateExpression(TaggedTemplateExpressionData {
            tag: tag.clone(),
            question_dot_token,
            type_arguments: None,
            template: template.clone(),
        }),
    );
    updated.loc = node.loc;
    updated.flags = node.flags;
    Arc::new(updated)
}

fn update_property_assignment_r40k08(
    node: &Arc<Node>,
    name: &Arc<Node>,
    initializer: Option<&Arc<Node>>,
) -> Arc<Node> {
    let (modifiers, postfix_token, type_node, fallback_initializer) = match &node.data {
        NodeData::PropertyAssignment(d) => (
            d.modifiers.clone(),
            d.postfix_token.clone(),
            Some(d.type_node.clone()),
            d.initializer.clone(),
        ),
        _ => (None, None, None, node.clone()),
    };
    let mut updated = Node::new(
        SyntaxKind::PropertyAssignment,
        NodeData::PropertyAssignment(PropertyAssignmentData {
            modifiers,
            name: name.clone(),
            postfix_token,
            type_node: type_node.unwrap_or_else(|| fallback_initializer.clone()),
            initializer: initializer.cloned().unwrap_or(fallback_initializer),
        }),
    );
    updated.loc = node.loc;
    updated.flags = node.flags;
    Arc::new(updated)
}

fn update_shorthand_property_assignment_r40k08(
    node: &Arc<Node>,
    name: &Arc<Node>,
    object_assignment_initializer: Option<&Arc<Node>>,
) -> Arc<Node> {
    let (modifiers, postfix_token, type_node, equals_token, fallback_initializer) =
        match &node.data {
            NodeData::ShorthandPropertyAssignment(d) => (
                d.modifiers.clone(),
                d.postfix_token.clone(),
                Some(d.type_node.clone()),
                d.equals_token.clone(),
                d.object_assignment_initializer.clone(),
            ),
            _ => (None, None, None, None, None),
        };
    let mut updated = Node::new(
        SyntaxKind::ShorthandPropertyAssignment,
        NodeData::ShorthandPropertyAssignment(ShorthandPropertyAssignmentData {
            modifiers,
            name: name.clone(),
            postfix_token,
            type_node: type_node.unwrap_or_else(|| {
                Arc::new(Node::new(SyntaxKind::Unknown, NodeData::Token))
            }),
            equals_token,
            object_assignment_initializer: object_assignment_initializer
                .cloned()
                .or(fallback_initializer),
        }),
    );
    updated.loc = node.loc;
    updated.flags = node.flags;
    Arc::new(updated)
}

fn update_spread_assignment_r40k08(node: &Arc<Node>, expression: &Arc<Node>) -> Arc<Node> {
    let mut updated = Node::new(
        SyntaxKind::SpreadAssignment,
        NodeData::SpreadAssignment(SpreadAssignmentData {
            expression: expression.clone(),
        }),
    );
    updated.loc = node.loc;
    updated.flags = node.flags;
    Arc::new(updated)
}

fn update_spread_element_r40k08(node: &Arc<Node>, expression: &Arc<Node>) -> Arc<Node> {
    let mut updated = Node::new(
        SyntaxKind::SpreadElement,
        NodeData::SpreadElement(SpreadElementData {
            expression: expression.clone(),
        }),
    );
    updated.loc = node.loc;
    updated.flags = node.flags;
    Arc::new(updated)
}
