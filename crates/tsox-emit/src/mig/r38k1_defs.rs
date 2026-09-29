#![allow(dead_code, unused_imports)]

use std::sync::Arc;

use tsox_frontend::ast::mig::m3c;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node::ModifierList;
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::node_source_file::SourceFile;
use tsox_frontend::ast::visitor::NodeVisitor;
use tsox_frontend::ast::{get_source_file_of_node, SyntaxKind};
use tsox_frontend::ast::node_flags::{ModifierFlags, NodeFlags};
use tsox_frontend::ast::mig::w2::create_modifiers_from_modifier_flags;
use tsox_frontend::ast::mig::m3g::is_modifier;
use tsox_frontend::format::mig::m4o::EmitFlags;
use tsox_frontend::scanner::{get_leading_comment_ranges, CommentRange};
use tsox_frontend::scanner::mig::m3i::is_identifier_text;
use tsox_frontend::ast::{is_numeric_literal, is_string_literal_like};
use tsox_core::core::text::TextRange;
use tsox_core::core::core::filter;

use tsox_frontend::ast::node_data_generated::is_export_assignment;
use crate::mig::m3m_2::create_get_symbol_accessibility_diagnostic_for_node;
use crate::mig::m3n_5::r33k8_defs::FileReference;
use crate::mig::m4e_2::{
    can_have_literal_initializer, can_produce_diagnostics, can_reuse_modifier_nodes, is_always_type,
};
use super::{DeclarationTransformer, ReferencedFilePair};
use crate::printer::NodeFactory;
use crate::printer::EmitContext;

pub trait R38K1NodeVisitorExt {
    fn visit(&mut self, node: Arc<Node>) -> Option<Arc<Node>>;
    fn visit_opt(&mut self, node: Option<Arc<Node>>) -> Option<Arc<Node>>;
    fn visit_each_child(&mut self, node: &Arc<Node>) -> Option<Arc<Node>>;
    fn visit_nodes(&mut self, nodes: Arc<NodeList>) -> Arc<NodeList>;
    fn visit_slice(&mut self, nodes: Vec<Arc<Node>>) -> (Vec<Arc<Node>>, bool);
}

fn r38k1_m3c_visitor() -> m3c::NodeVisitor {
    m3c::NodeVisitor {
        factory: m3c::NodeFactory {
            hooks: m3c::NodeFactoryHooks::default(),
            text_count: 0,
            node_count: 0,
        },
    }
}

impl R38K1NodeVisitorExt for NodeVisitor {
    fn visit(&mut self, node: Arc<Node>) -> Option<Arc<Node>> {
        Some(m3c::visit_each_child(&node, &mut r38k1_m3c_visitor()))
    }

    fn visit_opt(&mut self, node: Option<Arc<Node>>) -> Option<Arc<Node>> {
        let node = node?;
        self.visit(node)
    }

    fn visit_each_child(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        Some(m3c::visit_each_child(node, &mut r38k1_m3c_visitor()))
    }

    fn visit_nodes(&mut self, nodes: Arc<NodeList>) -> Arc<NodeList> {
        let visited = nodes
            .nodes
            .iter()
            .map(|n| m3c::visit_each_child(n, &mut r38k1_m3c_visitor()))
            .collect();
        Arc::new(NodeList {
            loc: nodes.loc,
            nodes: visited,
        })
    }

    fn visit_slice(&mut self, nodes: Vec<Arc<Node>>) -> (Vec<Arc<Node>>, bool) {
        let mut changed = false;
        let out = nodes
            .into_iter()
            .map(|n| {
                let v = m3c::visit_each_child(&n, &mut r38k1_m3c_visitor());
                changed |= !Arc::ptr_eq(&v, &n);
                v
            })
            .collect();
        (out, changed)
    }
}

pub trait R38K1NodeExt {
    fn loc(&self) -> TextRange;
    fn set_loc(&mut self, loc: TextRange);
    fn set_kind(&mut self, kind: SyntaxKind);
}

impl R38K1NodeExt for Node {
    fn loc(&self) -> TextRange {
        self.loc
    }

    fn set_loc(&mut self, loc: TextRange) {
        self.loc = loc;
    }

    fn set_kind(&mut self, kind: SyntaxKind) {
        self.kind = kind;
    }
}

impl DeclarationTransformer {
    pub fn binding_name_visitor(&self) -> NodeVisitor {
        NodeVisitor::default()
    }

    pub fn expression_visitor(&self) -> NodeVisitor {
        NodeVisitor::default()
    }

    pub fn cjs_export_assignment_visitor(&self) -> NodeVisitor {
        NodeVisitor::default()
    }

    pub fn export_stripping_visitor(&self) -> NodeVisitor {
        NodeVisitor::default()
    }

    pub fn this_property_visitor(&self) -> NodeVisitor {
        NodeVisitor::default()
    }

    pub fn declare_stripping_visitor(&self) -> NodeVisitor {
        NodeVisitor::default()
    }

    pub fn should_strip_internal(&self, node: Option<&Arc<Node>>) -> bool {
        match node {
            Some(node) => {
                self.state_data.strip_internal
                    && self.is_internal_declaration(node, self.state_data.file.as_deref())
            }
            None => false,
        }
    }

    pub fn is_internal_declaration(&self, node: &Arc<Node>, source_file: Option<&SourceFile>) -> bool {
        let Some(source_file) = source_file else {
            return false;
        };
        for comment_range in
            get_leading_comment_ranges(&source_file.text, node.pos())
        {
            if crate::mig::m3n_7::has_internal_annotation(comment_range, source_file) {
                return true;
            }
        }
        false
    }

    pub fn check_entity_name_visibility(
        &mut self,
        entity_name: &Arc<Node>,
        enclosing_declaration: Option<&Arc<Node>>,
    ) {
        let visibility_result = self
            .resolver
            .is_entity_name_visible(entity_name, enclosing_declaration);
        self.tracker_handle_symbol_accessibility_error(visibility_result);
    }

    pub fn check_name(&mut self, node: &Arc<Node>) {
        let old_diag = self.state_data.get_symbol_accessibility_diagnostic.take();
        if !self.suppress_new_diagnostic_contexts {
            self.state_data.get_symbol_accessibility_diagnostic =
                Some(create_get_symbol_accessibility_diagnostic_for_node(node));
        }
        self.state_data.error_name_node = node.name().cloned();
        let entity_name = node
            .name()
            .and_then(|n| n.expression())
            .expect("dynamic name");
        let enclosing = self.enclosing_declaration.clone();
        self.check_entity_name_visibility(&entity_name, enclosing.as_ref());
        if !self.suppress_new_diagnostic_contexts {
            self.state_data.get_symbol_accessibility_diagnostic = old_diag;
        }
        self.state_data.error_name_node = None;
    }

    pub fn ensure_modifiers(&self, node: &Arc<Node>) -> Option<Arc<ModifierList>> {
        let current_flags = get_combined_modifier_flags_of(&self.emit_context(), node);
        let new_flags = self.ensure_modifier_flags(node);
        if current_flags == new_flags {
            let mods = node.modifiers();
            let Some(mods) = mods else {
                return None;
            };
            if can_reuse_modifier_nodes(&mods.nodes) {
                return Some(
                    self.factory()
                        .new_modifier_list(filter(&mods.nodes, |m| is_modifier(m))),
                );
            }
        }
        let result =
            create_modifiers_from_modifier_flags(new_flags, |k| self.factory().new_modifier(k));
        if result.is_empty() {
            return None;
        }
        Some(self.factory().new_modifier_list(result))
    }

    pub fn ensure_modifier_flags(&self, node: &Arc<Node>) -> ModifierFlags {
        let mut mask = ModifierFlags::all()
            - ModifierFlags::Public
            - ModifierFlags::Async
            - ModifierFlags::Override;
        let mut additions = ModifierFlags::empty();
        if self.needs_declare && !is_always_type(node) {
            additions = ModifierFlags::Ambient;
        }
        let parent_is_file = matches!(node.parent(), Some(p) if p.kind == SyntaxKind::SourceFile);
        if !parent_is_file {
            mask = mask - ModifierFlags::Ambient;
            additions = ModifierFlags::empty();
        }
        if is_implicitly_exported(node) {
            additions |= ModifierFlags::Export;
        }
        (get_combined_modifier_flags_of(&self.emit_context(), node) & mask) | additions
    }

    pub fn ensure_type(&mut self, node: &Arc<Node>, ignore_private: bool) -> Option<Arc<Node>> {
        if !ignore_private
            && self
                .host()
                .get_effective_declaration_flags(
                    &self.emit_context().parse_node(node).expect("parse node"),
                    ModifierFlags::Private,
                )
                != ModifierFlags::empty()
        {
            return None;
        }
        if self.should_print_with_initializer(node) {
            return None;
        }
        let enclosing = self.enclosing_declaration.clone();
        let mut type_node = None;
        if !is_export_assignment(node)
            && !is_binding_element_of(node)
            && node.type_node().is_some()
        {
            if is_private_method_type_parameter_of(self.host(), node) {
                // private method type parameters are elided, type stays None
            } else {
                let old_diag = self.state_data.get_symbol_accessibility_diagnostic.take();
                if !self.suppress_new_diagnostic_contexts {
                    self.state_data.get_symbol_accessibility_diagnostic =
                        Some(create_get_symbol_accessibility_diagnostic_for_node(node));
                }
                self.state_data.error_name_node = Some(node.clone());
                type_node = self.visitor().visit(node.type_node().unwrap().clone());
                if !self.suppress_new_diagnostic_contexts && old_diag.is_some() {
                    self.state_data.get_symbol_accessibility_diagnostic = old_diag;
                }
                self.state_data.error_name_node = None;
            }
        }
        if type_node.is_none() {
            return Some(
                tsox_checker::checker::mig::m2c::r19k8_defs::new_keyword_type_node(
                    SyntaxKind::AnyKeyword,
                ),
            );
        }
        type_node
    }

    pub fn ensure_type_params(
        &mut self,
        node: &Arc<Node>,
        params: &Arc<NodeList>,
    ) -> Option<Arc<NodeList>> {
        if self
            .host()
            .get_effective_declaration_flags(
                &self.emit_context().parse_node(node).expect("parse node"),
                ModifierFlags::Private,
            )
            != ModifierFlags::empty()
        {
            return None;
        }
        let visited = params
            .nodes
            .iter()
            .map(|p| self.visitor().visit(p.clone()))
            .collect::<Option<Vec<_>>>()?;
        Some(Arc::new(NodeList {
            loc: params.loc,
            nodes: visited,
        }))
    }

    pub fn rewrite_module_specifier(
        &mut self,
        parent: &Arc<Node>,
        input: Option<&Arc<Node>>,
    ) -> Option<Arc<Node>> {
        let input = input?;
        self.result_has_external_module_indicator = self.result_has_external_module_indicator
            || (parent.kind != SyntaxKind::ModuleDeclaration
                && parent.kind != SyntaxKind::ImportType);
        Some(Arc::clone(input))
    }

    pub fn preserve_js_doc(&mut self, updated: &Arc<Node>, original: &Arc<Node>) {
        self.emit_context.assign_comment_range(updated, original);
    }

    pub fn preserve_partial_js_doc(&mut self, updated: &Arc<Node>, original: &Arc<Node>) {
        if !original.flags.contains(NodeFlags::Reparsed) {
            return;
        }
        let Some(source_file) = self.state_data.file.as_ref() else {
            return;
        };
        let Some(jsdoc) = source_file.eager_jsdoc(original).first().map(Arc::clone) else {
            return;
        };
        let description = tsox_frontend::scanner::mig::w5::get_text_of_jsdoc_comment(match &jsdoc.data {
            tsox_frontend::ast::node_data_generated::NodeData::JSDoc(data) => {
                Some(data.comment.as_ref())
            }
            _ => None,
        });
        if description.is_empty() {
            return;
        }
        let comment = "*\n * ".to_string() + &description.replace('\n', "\n * ") + "\n ";
        self.emit_context.add_synthetic_leading_comment(
            updated,
            SyntaxKind::MultiLineCommentTrivia,
            &comment,
            true,
        );
    }

    pub fn should_print_with_initializer(&self, node: &Arc<Node>) -> bool {
        can_have_literal_initializer(self.host(), node)
            && node.initializer().is_some()
            && self
                .resolver()
                .is_literal_const_declaration(&self.emit_context().most_original(node))
    }

    pub fn setup_diagnostic_context(
        &mut self,
        input: &Arc<Node>,
    ) -> (bool, Box<dyn FnOnce(&mut Self)>) {
        let can_produce_diagnostic = can_produce_diagnostics(input);
        let old_suppress = self.suppress_new_diagnostic_contexts;
        let should_enter_suppress = (input.kind == SyntaxKind::TypeLiteral
            || input.kind == SyntaxKind::MappedType)
            && !matches!(
                input.parent(),
                Some(p) if p.kind == SyntaxKind::TypeAliasDeclaration
                    || p.kind == SyntaxKind::JSTypeAliasDeclaration
            );

        let old_diag = self.state_data.get_symbol_accessibility_diagnostic.take();
        if can_produce_diagnostic && !self.suppress_new_diagnostic_contexts {
            self.state_data.get_symbol_accessibility_diagnostic =
                Some(create_get_symbol_accessibility_diagnostic_for_node(input));
        }
        let old_name = self.state_data.error_name_node.clone();

        if should_enter_suppress {
            self.suppress_new_diagnostic_contexts = true;
        }

        (
            can_produce_diagnostic,
            Box::new(move |tx: &mut Self| {
                tx.state_data.get_symbol_accessibility_diagnostic = old_diag;
                tx.state_data.error_name_node = old_name;
                tx.suppress_new_diagnostic_contexts = old_suppress;
            }),
        )
    }

    pub fn get_lib_references(&self) -> Vec<FileReference> {
        self.state_data
            .raw_lib_reference_directives
            .iter()
            .filter(|r| r.preserve)
            .map(|r| FileReference {
                text_range: TextRange::new(usize::MAX, usize::MAX),
                file_name: r.file_name.clone(),
                resolution_mode: r.resolution_mode,
                preserve: r.preserve,
            })
            .collect()
    }

    pub fn get_type_references(&self) -> Vec<FileReference> {
        self.state_data
            .raw_type_reference_directives
            .iter()
            .filter(|r| r.preserve)
            .map(|r| FileReference {
                text_range: TextRange::new(usize::MAX, usize::MAX),
                file_name: r.file_name.clone(),
                resolution_mode: r.resolution_mode,
                preserve: r.preserve,
            })
            .collect()
    }

    pub fn get_name_expression_preferring_identifier(&self, name_expr: &Arc<Node>) -> Arc<Node> {
        if is_numeric_literal(name_expr) {
            return self
                .factory()
                .new_string_literal(&name_expr.text().to_string(), Default::default());
        }
        if is_string_literal_like(name_expr)
            && is_identifier_text(
                &name_expr.text().to_string(),
                tsox_frontend::ast::node_source_file::LanguageVariant::Standard,
            )
        {
            let result = self.factory().new_identifier(&name_expr.text().to_string());
            let kw_kind =
                tsox_frontend::scanner::mig::m3i::identifier_to_keyword_kind(&result);
            if kw_kind == SyntaxKind::Unknown || kw_kind == SyntaxKind::DefaultKeyword {
                return result;
            }
        }
        name_expr.clone()
    }

    pub fn transform_and_replace_late_painted_statements(
        &mut self,
        statements: Arc<NodeList>,
    ) -> Arc<NodeList> {
        let mut visited: Vec<Arc<Node>> = Vec::new();
        for statement in &statements.nodes {
            let id = tsox_frontend::ast::mig::m3f::get_node_id(
                &self.emit_context().most_original(statement),
            );
            let replacement = self.late_statement_replacement_map.remove(&id).flatten();
            match replacement {
                Some(replacement) => visited.push(replacement),
                None => visited.push(Arc::clone(statement)),
            }
        }
        Arc::new(NodeList {
            loc: statements.loc,
            nodes: visited,
        })
    }

    fn tracker_handle_symbol_accessibility_error(
        &mut self,
        visibility_result: tsox_checker::checker::types::SymbolAccessibilityResult,
    ) {
        use tsox_checker::checker::types::SymbolAccessibility;
        if visibility_result.accessibility != SymbolAccessibility::Accessible {
            let error_node = self
                .state_data
                .error_name_node
                .clone()
                .or_else(|| self.enclosing_declaration.clone());
            if let Some(error_node) = error_node {
                let diagnostic_name = self
                    .state_data
                    .get_symbol_accessibility_diagnostic
                    .as_ref()
                    .and_then(|f| f(&visibility_result));
                self.state_data.add_diagnostic(crate::mig::m3n_5::create_diagnostic_for_node(
                    &error_node,
                    diagnostic_name.as_ref().map(|d| d.diagnostic_message),
                    &[],
                ));
            }
        }
    }
}

fn get_combined_modifier_flags_of(
    emit_context: &EmitContext,
    node: &Arc<Node>,
) -> ModifierFlags {
    tsox_frontend::ast::get_combined_modifier_flags(&emit_context.parse_node(node).expect("parse node"))
        & ModifierFlags::all()
}

fn is_implicitly_exported(node: &Arc<Node>) -> bool {
    tsox_frontend::ast::mig::x6a::is_implicitly_exported_jsdoc_declaration(node)
}

fn is_binding_element_of(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::BindingElement
    )
}

fn is_private_method_type_parameter_of(
    host: &dyn super::DeclarationEmitHost,
    node: &Arc<Node>,
) -> bool {
    crate::mig::m4e_2::is_private_method_type_parameter(host, node)
}
