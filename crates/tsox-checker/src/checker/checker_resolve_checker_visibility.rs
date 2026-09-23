#![allow(unused_imports)]

use crate::checker::checker_resolve::*;
use std::sync::Arc;

use tsox_core::core::compiler_options::ScriptTarget;
use tsox_frontend::ast::for_each_child;
use tsox_frontend::ast::is_function_like;
use tsox_frontend::ast::is_nullish_coalesce;
use tsox_frontend::ast::is_optional_chain;
use tsox_frontend::ast::is_type_node;
use tsox_frontend::ast::ModifierFlags;
use tsox_frontend::ast::NodeData;
use tsox_frontend::ast::NodeFlags;

pub(crate) fn fn_like_body(node: &Arc<Node>) -> Option<Arc<Node>> {
    match &node.data {
        NodeData::FunctionDeclaration(d) => d.body.clone(),
        NodeData::FunctionExpression(d) => Some(d.body.clone()),
        NodeData::ArrowFunction(d) => Some(d.body.clone()),
        NodeData::MethodDeclaration(d) => d.body.clone(),
        NodeData::ConstructorDeclaration(d) => d.body.clone(),
        NodeData::GetAccessorDeclaration(d) => d.body.clone(),
        NodeData::SetAccessorDeclaration(d) => d.body.clone(),
        NodeData::ClassStaticBlockDeclaration(d) => Some(d.body.clone()),
        _ => None,
    }
}

fn container_parameters(node: &Arc<Node>) -> Vec<Arc<Node>> {
    match &node.data {
        NodeData::FunctionDeclaration(d) => d.parameters.iter().cloned().collect(),
        NodeData::FunctionExpression(d) => d.parameters.iter().cloned().collect(),
        NodeData::ArrowFunction(d) => d.parameters.iter().cloned().collect(),
        NodeData::MethodDeclaration(d) => d.parameters.iter().cloned().collect(),
        NodeData::MethodSignatureDeclaration(d) => d.parameters.iter().cloned().collect(),
        NodeData::ConstructorDeclaration(d) => d.parameters.iter().cloned().collect(),
        NodeData::GetAccessorDeclaration(d) => d.parameters.iter().cloned().collect(),
        NodeData::SetAccessorDeclaration(d) => d.parameters.iter().cloned().collect(),
        NodeData::FunctionTypeNode(d) => d.parameters.iter().cloned().collect(),
        NodeData::ConstructorTypeNode(d) => d.parameters.iter().cloned().collect(),
        _ => Vec::new(),
    }
}

fn declaration_is_in_parameter(node: &Arc<Node>) -> bool {
    let mut current = Arc::clone(node);
    loop {
        if current.kind == SyntaxKind::Parameter {
            return true;
        }
        match current.parent() {
            Some(p) => current = p,
            None => return false,
        }
    }
}

impl Checker {
    fn requires_scope_change_worker(&self, node: &Arc<Node>) -> bool {
        match node.kind {
            SyntaxKind::ArrowFunction
            | SyntaxKind::FunctionExpression
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::Constructor => false,
            SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::PropertyAssignment => node
                .name()
                .is_some_and(|n| self.requires_scope_change_worker(n)),
            SyntaxKind::PropertyDeclaration => {
                if node.has_syntactic_modifier(ModifierFlags::Static) {
                    !self.emit_standard_class_fields
                } else {
                    node.name()
                        .is_some_and(|n| self.requires_scope_change_worker(n))
                }
            }
            _ => {
                if is_nullish_coalesce(node) || is_optional_chain(node) {
                    return (self.language_version as u32) < (ScriptTarget::ES2020 as u32);
                }
                if node.kind == SyntaxKind::BindingElement
                    && let NodeData::BindingElement(be) = &node.data
                    && be.dot_dot_dot_token.is_some()
                    && node
                        .parent()
                        .is_some_and(|p| p.kind == SyntaxKind::ObjectBindingPattern)
                {
                    return (self.language_version as u32) < (ScriptTarget::ES2017 as u32);
                }
                if is_type_node(node) {
                    return false;
                }
                let mut hit = false;
                for_each_child(node, |c| {
                    if !hit && self.requires_scope_change_worker(c) {
                        hit = true;
                    }
                    hit
                });
                hit
            }
        }
    }

    fn requires_scope_change(&self, param: &Arc<Node>) -> bool {
        let NodeData::ParameterDeclaration(d) = &param.data else {
            return false;
        };
        self.requires_scope_change_worker(&d.name)
            || d
                .initializer
                .as_ref()
                .is_some_and(|i| self.requires_scope_change_worker(i))
    }

    fn use_outer_variable_scope_in_parameter(
        &self,
        sym: &Arc<Symbol>,
        container: &Arc<Node>,
        child_below: &Arc<Node>,
    ) -> bool {
        if child_below.kind != SyntaxKind::Parameter {
            return false;
        }
        let Some(body) = fn_like_body(container) else {
            return false;
        };
        let Some(vd) = sym.value_declaration.as_ref() else {
            return false;
        };
        if !(vd.loc.pos() >= body.loc.pos() && vd.loc.end() <= body.loc.end()) {
            return false;
        }
        !container_parameters(container)
            .iter()
            .any(|p| self.requires_scope_change(p))
    }

    pub(crate) fn locals_symbol_visible_at(
        &self,
        container: &Arc<Node>,
        child_below: &Arc<Node>,
        sym: &Arc<Symbol>,
        meaning: SymbolFlags,
    ) -> bool {
        if !is_function_like(container) {
            return true;
        }
        if fn_like_body(container).is_some_and(|body| Arc::ptr_eq(&body, child_below)) {
            return true;
        }
        let mut use_result = true;
        if meaning.intersects(sym.flags.intersection(SymbolFlags::TYPE))
            && child_below.kind != SyntaxKind::JSDoc
        {
            let last_in_type_position = child_below
                .flags
                .intersects(NodeFlags::Synthesized)
                || container
                    .type_node()
                    .is_some_and(|t| Arc::ptr_eq(t, child_below))
                || matches!(
                    child_below.kind,
                    SyntaxKind::Parameter
                        | SyntaxKind::JSDocParameterTag
                        | SyntaxKind::JSDocReturnTag
                        | SyntaxKind::TypeParameter
                );
            use_result =
                sym.flags.contains(SymbolFlags::TypeParameter) && last_in_type_position;
        }
        if meaning.intersects(sym.flags.intersection(SymbolFlags::VARIABLE)) {
            if self.use_outer_variable_scope_in_parameter(sym, container, child_below) {
                use_result = false;
            } else if sym.flags.intersects(SymbolFlags::FunctionScopedVariable) {
                use_result = child_below.kind == SyntaxKind::Parameter
                    || child_below.flags.intersects(NodeFlags::Synthesized)
                    || (container
                        .type_node()
                        .is_some_and(|t| Arc::ptr_eq(t, child_below))
                        && sym
                            .value_declaration
                            .as_ref()
                            .is_some_and(declaration_is_in_parameter));
            }
        }
        use_result
    }
}
