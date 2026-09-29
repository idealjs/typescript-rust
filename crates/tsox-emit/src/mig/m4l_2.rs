#![allow(unused_imports)]

use std::sync::Arc;

use tsox_checker::checker::mig::m2d::EmitResolver;
use tsox_checker::checker::mig::wc1b::class_or_constructor_parameter_is_decorated;
use tsox_core::core::compiler_options_kinds::ScriptTarget;
use tsox_frontend::ast::node::{Node, NodeList};
use tsox_frontend::ast::subtree_facts::SubtreeContainsDecorators;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use crate::mig::m3m::TransformOptions;
use crate::mig::m4g::r33k7_defs::has_decorators;
use crate::mig::m4k_2::Transformer;
use crate::mig::m4l::r39k20_defs::{metadata_transformer_visit_entry, R39K20NodeFactoryExt};
use crate::mig::m4m::{new_metadata_serializer, MetadataSerializer};
use crate::printer::{EmitContext, NodeFactory};
use tsox_frontend::ast::visitor::NodeVisitor;

use super::m4l::get_decorators_of_parameters;
use super::m4f_2::r38k5_defs::{R38K5ArcNodeExt, R38K5NodeVisitorExt};

pub const USE_NEW_TYPE_METADATA_FORMAT: bool = false;

pub struct MetadataTransformer<'a> {
    pub legacy_decorators: bool,
    pub resolver: &'a EmitResolver,
    pub serializer: Option<MetadataSerializer>,
    pub language_version: ScriptTarget,
    pub strict_null_checks: bool,
    pub parent: Option<Arc<Node>>,
    pub current_lexical_scope: Option<Arc<Node>>,
    pub emit_context: Option<EmitContext>,
    pub substitution_visitor: Option<NodeVisitor>,
}

impl<'a> MetadataTransformer<'a> {
    pub fn new_metadata_transformer(opts: &TransformOptions) -> Arc<Transformer> {
        let mut tx = Box::new(MetadataTransformer {
            legacy_decorators: opts.compiler_options.experimental_decorators.is_true(),
            resolver: &opts.emit_resolver,
            serializer: None,
            language_version: opts.compiler_options.get_emit_script_target(),
            strict_null_checks: opts
                .compiler_options
                .get_strict_option_value(opts.compiler_options.strict_null_checks),
            parent: None,
            current_lexical_scope: None,
            emit_context: Some(opts.context.clone()),
            substitution_visitor: Some(NodeVisitor::default()),
        });
        tx.new_transformer(|tx, node| tx.visit(&node), &opts.context)
    }

    pub fn new_transformer(
        &self,
        _visit: fn(&mut Self, Arc<Node>) -> Option<Arc<Node>>,
        emit_context: &EmitContext,
    ) -> Arc<Transformer> {
        Arc::new(Transformer::new(
            metadata_transformer_visit_entry,
            Some(emit_context.clone()),
        ))
    }

    pub fn emit_context_mut(&mut self) -> &mut EmitContext {
        self.emit_context.as_mut().unwrap()
    }

    pub fn factory(&self) -> NodeFactory<'_> {
        NodeFactory::new(self.emit_context.as_ref().unwrap())
    }

    pub fn visitor(&mut self) -> &mut NodeVisitor {
        self.substitution_visitor.as_mut().unwrap()
    }

    pub fn visit(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if !node
            .subtree_facts()
            .intersects(SubtreeContainsDecorators)
        {
            return Some(Arc::clone(node));
        }

        match node.kind {
            SyntaxKind::ClassDeclaration => self.visit_class_declaration(node),
            SyntaxKind::ClassExpression => self.visit_class_expression(node),
            SyntaxKind::PropertyDeclaration => self.visit_property_declaration(node),
            SyntaxKind::MethodDeclaration => self.visit_method_declaration(node),
            SyntaxKind::SetAccessor => self.visit_set_accessor(node),
            SyntaxKind::GetAccessor => self.visit_get_accessor(node),
            SyntaxKind::SourceFile => {
                self.parent = None;
                self.current_lexical_scope = Some(Arc::clone(node));
                let updated = self.visitor().visit_each_child(node)?;
                let helpers = self.emit_context_mut().read_emit_helpers();
                self.emit_context_mut().add_emit_helper(&updated, &helpers);
                self.set_parent(None);
                self.set_current_lexical_scope(None);
                Some(updated)
            }
            SyntaxKind::ModuleBlock | SyntaxKind::Block | SyntaxKind::CaseBlock => {
                let old_scope = self.current_lexical_scope.take();
                self.current_lexical_scope = Some(Arc::clone(node));
                let result = self.visitor().visit_each_child(node);
                self.set_current_lexical_scope(old_scope);
                result
            }
            _ => self.visitor().visit_each_child(node),
        }
    }

    pub fn set_parent(&mut self, node: Option<Arc<Node>>) {
        self.parent = node;
    }

    pub fn set_current_lexical_scope(&mut self, node: Option<Arc<Node>>) {
        self.current_lexical_scope = node;
    }

    pub fn visit_class_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let old_parent = self.parent.take();
        self.parent = Some(Arc::clone(node));

        let result = if !class_or_constructor_parameter_is_decorated(self.legacy_decorators, node) {
            self.visitor().visit_each_child(node)
        } else {
            let modifiers = self.visitor().visit_modifiers(&node.modifiers().cloned());
            let name = self.visitor().visit_node_opt(node.name());
            let type_parameters = node.type_parameters().map(|l| self.visitor().visit_nodes_list(l));
            let heritage_clauses = node
                .heritage_clauses()
                .map(|l| self.visitor().visit_nodes_list(l));
            let members = self.visitor().visit_nodes_list(node.members());
            Some(self.factory().update_class_expression(
                node,
                modifiers,
                name.as_ref(),
                type_parameters,
                heritage_clauses.as_deref(),
                &members,
            ))
        };

        self.set_parent(old_parent);
        result
    }

    pub fn visit_class_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let old_parent = self.parent.take();
        self.parent = Some(Arc::clone(node));

        let result = if !class_or_constructor_parameter_is_decorated(self.legacy_decorators, node) {
            self.visitor().visit_each_child(node)
        } else {
            let modifiers = self.visitor().visit_modifiers(&node.modifiers().cloned());
            let name = self.visitor().visit_node_opt(node.name());
            let type_parameters = node.type_parameters().map(|l| self.visitor().visit_nodes_list(l));
            let heritage_clauses = node
                .heritage_clauses()
                .map(|l| self.visitor().visit_nodes_list(l));
            let members = self.visitor().visit_nodes_list(node.members());
            Some(self.factory().update_class_declaration(
                node,
                modifiers,
                name.as_ref(),
                type_parameters,
                heritage_clauses.as_deref(),
                &members,
            ))
        };

        self.set_parent(old_parent);
        result
    }

    pub fn visit_property_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if !has_decorators(node) {
            return self.visitor().visit_each_child(node);
        }

        let visited_modifiers = self.visitor().visit_modifiers(&node.modifiers().cloned());
        let modifiers = self.inject_class_element_type_metadata(
            visited_modifiers,
            node,
            self.parent.clone().as_ref(),
        );
        let name = self.visitor().visit_node_opt(node.name());
        let postfix_token = self.visitor().visit_node_option(node.postfix_token());
        let type_node = self.visitor().visit_node_option(node.type_node());
        let initializer = self.visitor().visit_node_option(node.initializer());
        Some(self.factory().r39k20_update_property_declaration(
            node,
            modifiers,
            &name.expect("property declaration name"),
            postfix_token,
            type_node,
            initializer,
        ))
    }

    pub fn visit_method_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if !has_decorators(node) && get_decorators_of_parameters(Some(node)).is_empty() {
            return self.visitor().visit_each_child(node);
        }

        let visited_modifiers = self.visitor().visit_modifiers(&node.modifiers().cloned());
        let modifiers = self.inject_class_element_type_metadata(
            visited_modifiers,
            node,
            self.parent.clone().as_ref(),
        );
        let asterisk_token = self.visitor().visit_node_option(node.asterisk_token());
        let name = self.visitor().visit_node_opt(node.name());
        let postfix_token = self.visitor().visit_node_option(node.postfix_token());
        let type_parameters = node.type_parameters().map(|l| self.visitor().visit_nodes_list(l));
        let parameters = node
            .parameters()
            .map(|l| self.visitor().visit_nodes_list(l))
            .unwrap_or_default();
        let type_node = self.visitor().visit_node_option(node.type_node());
        let full_signature = self.visitor().visit_node_option(node.full_signature());
        let body = self.visitor().visit_node_option(node.body());
        Some(self.factory().update_method_declaration(
            node,
            modifiers,
            asterisk_token.as_ref(),
            &name.expect("method declaration name"),
            postfix_token,
            type_parameters,
            &parameters,
            type_node,
            full_signature,
            body.as_ref(),
        ))
    }

    pub fn visit_set_accessor(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if !has_decorators(node) && get_decorators_of_parameters(Some(node)).is_empty() {
            return self.visitor().visit_each_child(node);
        }

        let visited_modifiers = self.visitor().visit_modifiers(&node.modifiers().cloned());
        let modifiers = self.inject_class_element_type_metadata(
            visited_modifiers,
            node,
            self.parent.clone().as_ref(),
        );
        let name = self.visitor().visit_node_opt(node.name());
        let type_parameters = node.type_parameters().map(|l| self.visitor().visit_nodes_list(l));
        let parameters = node
            .parameters()
            .map(|l| self.visitor().visit_nodes_list(l))
            .unwrap_or_default();
        let type_node = self.visitor().visit_node_option(node.type_node());
        let full_signature = self.visitor().visit_node_option(node.full_signature());
        let body = self.visitor().visit_node_option(node.body());
        Some(self.factory().update_set_accessor_declaration(
            node,
            modifiers,
            &name.expect("set accessor name"),
            type_parameters,
            &parameters,
            type_node,
            full_signature,
            body.as_ref(),
        ))
    }

    pub fn visit_get_accessor(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if !has_decorators(node) {
            return self.visitor().visit_each_child(node);
        }

        let visited_modifiers = self.visitor().visit_modifiers(&node.modifiers().cloned());
        let modifiers = self.inject_class_element_type_metadata(
            visited_modifiers,
            node,
            self.parent.clone().as_ref(),
        );
        let name = self.visitor().visit_node_opt(node.name());
        let type_parameters = node.type_parameters().map(|l| self.visitor().visit_nodes_list(l));
        let parameters = node
            .parameters()
            .map(|l| self.visitor().visit_nodes_list(l))
            .unwrap_or_default();
        let type_node = self.visitor().visit_node_option(node.type_node());
        let full_signature = self.visitor().visit_node_option(node.full_signature());
        let body = self.visitor().visit_node_option(node.body());
        Some(self.factory().update_get_accessor_declaration(
            node,
            modifiers,
            &name.expect("get accessor name"),
            type_parameters,
            &parameters,
            type_node,
            full_signature,
            body.as_ref(),
        ))
    }

}
