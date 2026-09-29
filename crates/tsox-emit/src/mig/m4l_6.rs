#![allow(unused_imports)]

use std::sync::Arc;

use tsox_core::core::compiler_options::CompilerOptions;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_flags::ModifierFlags;
use tsox_frontend::ast::subtree_facts::SubtreeContainsTypeScript;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::{
    is_binary_expression, is_identifier, is_instantiated_module, is_statement,
};

use super::m3m::TransformOptions;
use super::m4k_2::Transformer;
use super::m4l::r33k9_defs::get_innermost_module_declaration_from_dotted_module;
use crate::printer::{EmitContext, NodeFactory};
use tsox_frontend::ast::visitor::NodeVisitor;

pub use crate::mig::m4f_3::r38k10_defs;
use r38k10_defs::R38K10NodeVisitorExt;
use crate::mig::m4l::r39k20_defs::type_eraser_visit_entry;

#[path = "r38k14_defs.rs"]
pub mod r38k14_defs;

pub struct TypeEraserTransformer {
    pub compiler_options: CompilerOptions,
    pub emit_context: EmitContext,
    pub substitution_visitor: Option<NodeVisitor>,
    pub parent_node: Option<Arc<Node>>,
    pub current_node: Option<Arc<Node>>,
}

impl TypeEraserTransformer {
    pub fn emit_context(&self) -> &EmitContext {
        &self.emit_context
    }

    pub fn factory(&self) -> NodeFactory<'_> {
        NodeFactory::new(&self.emit_context)
    }

    pub fn visitor(&mut self) -> &mut NodeVisitor {
        self.substitution_visitor.as_mut().unwrap()
    }

    pub fn new_transformer(
        &self,
        _visit: fn(&mut Self, Arc<Node>) -> Option<Arc<Node>>,
        emit_context: &EmitContext,
    ) -> Arc<Transformer> {
        Arc::new(Transformer::new(
            type_eraser_visit_entry,
            Some(emit_context.clone()),
        ))
    }

    pub fn new_type_eraser_transformer(opts: &TransformOptions) -> Arc<Transformer> {
        let compiler_options = opts.compiler_options.clone();
        let emit_context = opts.context;
        let tx = Box::new(TypeEraserTransformer {
            compiler_options,
            emit_context: emit_context.clone(),
            substitution_visitor: Some(NodeVisitor::default()),
            parent_node: None,
            current_node: None,
        });
        tx.new_transformer(|tx, node| tx.visit(&node), emit_context)
    }

    pub fn push_node(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        let grandparent_node = self.parent_node.take();
        self.parent_node = self.current_node.take();
        self.current_node = Some(Arc::clone(node));
        grandparent_node
    }

    pub fn pop_node(&mut self, grandparent_node: Option<Arc<Node>>) {
        self.current_node = self.parent_node.take();
        self.parent_node = grandparent_node;
    }

    pub fn elide(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        Some(self.emit_context.new_not_emitted_statement(node))
    }

    pub fn visit(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if !node.subtree_facts().intersects(SubtreeContainsTypeScript) {
            return Some(Arc::clone(node));
        }

        if is_statement(node) && node.has_syntactic_modifier(ModifierFlags::Ambient) {
            return self.elide(node);
        }

        let grandparent_node = self.push_node(node);
        let result = self.visit_inner(node);
        self.pop_node(grandparent_node);
        result
    }

    fn visit_inner(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        match node.kind {
            SyntaxKind::PublicKeyword
            | SyntaxKind::PrivateKeyword
            | SyntaxKind::ProtectedKeyword
            | SyntaxKind::AbstractKeyword
            | SyntaxKind::OverrideKeyword
            | SyntaxKind::ConstKeyword
            | SyntaxKind::DeclareKeyword
            | SyntaxKind::ReadonlyKeyword
            | SyntaxKind::ArrayType
            | SyntaxKind::TupleType
            | SyntaxKind::OptionalType
            | SyntaxKind::RestType
            | SyntaxKind::TypeLiteral
            | SyntaxKind::TypePredicate
            | SyntaxKind::TypeParameter
            | SyntaxKind::AnyKeyword
            | SyntaxKind::UnknownKeyword
            | SyntaxKind::BooleanKeyword
            | SyntaxKind::StringKeyword
            | SyntaxKind::NumberKeyword
            | SyntaxKind::NeverKeyword
            | SyntaxKind::VoidKeyword
            | SyntaxKind::SymbolKeyword
            | SyntaxKind::ConstructorType
            | SyntaxKind::FunctionType
            | SyntaxKind::TypeQuery
            | SyntaxKind::TypeReference
            | SyntaxKind::UnionType
            | SyntaxKind::IntersectionType
            | SyntaxKind::ConditionalType
            | SyntaxKind::ParenthesizedType
            | SyntaxKind::ThisType
            | SyntaxKind::TypeOperator
            | SyntaxKind::IndexedAccessType
            | SyntaxKind::MappedType
            | SyntaxKind::LiteralType
            | SyntaxKind::IndexSignature => None,

            SyntaxKind::InKeyword | SyntaxKind::OutKeyword => {
                let is_binary_parent = self
                    .parent_node
                    .as_ref()
                    .is_some_and(|p| is_binary_expression(p));
                if !is_binary_parent {
                    return None;
                }
                self.visitor().visit_each_child(node)
            }

            SyntaxKind::JSImportDeclaration => None,

            SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::InterfaceDeclaration => self.elide(node),

            SyntaxKind::NamespaceExportDeclaration => None,

            SyntaxKind::ModuleDeclaration => {
                let innermost = get_innermost_module_declaration_from_dotted_module(node);
                let body_missing = match &innermost.data {
                    NodeData::ModuleDeclaration(d) => d.body.is_none(),
                    _ => true,
                };
                if !node.name().is_some_and(|n| is_identifier(n))
                    || !is_instantiated_module(node, self.compiler_options.should_preserve_const_enums())
                    || body_missing
                {
                    return self.elide(node);
                }
                self.visitor().visit_each_child(node)
            }

            SyntaxKind::ExpressionWithTypeArguments => {
                let expression = self
                    .visitor()
                    .visit_node(node.expression().expect("ExpressionWithTypeArguments expression"));
                Some(self.factory().update_expression_with_type_arguments(
                    node,
                    expression,
                    None,
                ))
            }

            _ => self.visit_inner_tail(node),
        }
    }
}