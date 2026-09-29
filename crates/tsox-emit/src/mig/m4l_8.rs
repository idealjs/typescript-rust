#![allow(unused_imports)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_flags::ModifierFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::node_is_missing;

use super::m4l_6::TypeEraserTransformer;
use super::m4f_2::r38k5_defs::R38K5NodeVisitorExt;
use super::m4l::r39k20_defs::R39K20NodeVisitorExt;
use super::m4p_5::r36k12_defs::{
    attributes, export_clause, import_clause, is_type_only, module_specifier, named_bindings,
    phase_modifier,
};
use tsox_frontend::ast::mig::x6a::is_enum_const;

impl TypeEraserTransformer {
    pub(crate) fn visit_import_export_tail(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        match node.kind {
            SyntaxKind::ImportEqualsDeclaration => {
                if is_type_only(node) {
                    return None;
                }
                self.visitor().visit_each_child(node)
            }

            SyntaxKind::ImportDeclaration => {
                if import_clause(node).is_none() {
                    return Some(Arc::clone(node));
                }
                let visited_clause = self.visitor().visit_node_opt(import_clause(node));
                if visited_clause.is_none() {
                    return None;
                }
                Some(self.factory().update_import_declaration(
                    node,
                    node.modifiers().cloned(),
                    visited_clause,
                    module_specifier(node).cloned().unwrap(),
                    attributes(node).cloned(),
                ))
            }

            SyntaxKind::ImportClause => {
                if is_type_only(node) {
                    return None;
                }
                let name = node.name();
                let visited_bindings = self.visitor().visit_node_opt(named_bindings(node));
                if name.is_none() && visited_bindings.is_none() {
                    return None;
                }
                Some(
                    self.factory().update_import_clause(
                        node,
                        phase_modifier(node),
                        name.cloned(),
                        visited_bindings,
                    ),
                )
            }

            SyntaxKind::NamedImports => {
                let Some(elements_node) = node.elements() else {
                    return Some(Arc::clone(node));
                };
                if elements_node.nodes.is_empty() {
                    return Some(Arc::clone(node));
                }
                let elements = self.visitor().visit_nodes(node.elements());
                if !self.compiler_options.verbatim_module_syntax.is_true()
                    && elements.as_ref().is_some_and(|e| e.nodes.is_empty())
                {
                    return None;
                }
                Some(self.factory().update_named_imports(node, &elements.unwrap_or_default()))
            }

            SyntaxKind::ImportSpecifier => {
                if is_type_only(node) {
                    return None;
                }
                Some(Arc::clone(node))
            }

            SyntaxKind::ExportDeclaration => {
                if is_type_only(node) {
                    return None;
                }
                let mut visited_export_clause: Option<Arc<Node>> = None;
                if export_clause(node).is_some() {
                    visited_export_clause =
                        self.visitor()
                            .visit_node_opt(export_clause(node));
                    if visited_export_clause.is_none() {
                        return None;
                    }
                }
                let visited_module_specifier =
                    self.visitor().visit_node_opt(module_specifier(node));
                let visited_attributes = self.visitor().visit_node_opt(attributes(node));
                Some(self.factory().update_export_declaration(
                    node,
                    None,
                    false,
                    visited_export_clause,
                    visited_module_specifier,
                    visited_attributes,
                ))
            }

            SyntaxKind::NamedExports => {
                let Some(elements_node) = node.elements() else {
                    return Some(Arc::clone(node));
                };
                if elements_node.nodes.is_empty() {
                    return Some(Arc::clone(node));
                }
                let elements = self.visitor().visit_nodes(node.elements());
                if !self.compiler_options.verbatim_module_syntax.is_true()
                    && elements.as_ref().is_some_and(|e| e.nodes.is_empty())
                {
                    return None;
                }
                Some(self.factory().update_named_exports(node, &elements.unwrap_or_default()))
            }

            SyntaxKind::ExportSpecifier => {
                if is_type_only(node) {
                    return None;
                }
                Some(Arc::clone(node))
            }

            SyntaxKind::EnumDeclaration => {
                if is_enum_const(node) {
                    return Some(Arc::clone(node));
                }
                self.visitor().visit_each_child(node)
            }

            _ => self.visitor().visit_each_child(node),
        }
    }

    fn visit_accessor_declaration(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> {
        if node.body().map(|b| node_is_missing(Some(b))).unwrap_or(true)
            && node.has_syntactic_modifier(ModifierFlags::Abstract)
        {
            return None;
        }
        let body = node
            .body()
            .map(|b| self.visitor().visit_node(b))
            .unwrap_or_else(|| {
                self.factory()
                    .new_block(&self.factory().new_node_list(Vec::new()), false)
            });
        let modifiers = self.visitor().visit_modifiers(&node.modifiers().cloned());
        let name = self.visitor().visit_node(&node.name().unwrap());
        let parameters = self
            .visitor()
            .visit_nodes(node.parameters())
            .unwrap_or_default();
        if node.kind == SyntaxKind::GetAccessor {
            Some(self.factory().update_get_accessor_declaration(
                node,
                modifiers,
                &name,
                None,
                &parameters,
                None,
                None,
                Some(&body),
            ))
        } else {
            Some(self.factory().update_set_accessor_declaration(
                node,
                modifiers,
                &name,
                None,
                &parameters,
                None,
                None,
                Some(&body),
            ))
        }
    }
}
