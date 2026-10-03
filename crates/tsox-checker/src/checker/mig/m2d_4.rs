#![allow(unused_imports)]

use crate::checker::checker_checker::*;
use crate::checker::types::{NodeCheckFlags, Type, TypePredicate, TypePredicateKind};
use crate::checker::mig::wc3::NodeAccessExt;
use crate::checker::utilities_has_only_expression_initialization::{get_assignment_target_kind, AssignmentKind};
use std::sync::Arc;
use tsox_frontend::ast::mig::m3g_3::skip_parentheses;
use tsox_frontend::ast::find_ancestor;
use tsox_frontend::ast::{
    is_access_expression, is_function_or_source_file, is_string_literal, is_type_node, Node,
    NodeData, Symbol, SymbolFlags, SyntaxKind,
};

impl Checker {
    pub fn is_false_expression(&mut self, expr: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_false_expression"); 
        let node = skip_parentheses(expr);
        if node.kind == SyntaxKind::FalseKeyword {
            return true;
        }
        if node.kind == SyntaxKind::BinaryExpression {
            let NodeData::BinaryExpression(binary) = &node.data else {
                return false;
            };
            let operator_kind = binary.operator_token.kind;
            return (operator_kind == SyntaxKind::AmpersandAmpersandToken
                && (self.is_false_expression(&binary.left) || self.is_false_expression(&binary.right)))
                || (operator_kind == SyntaxKind::BarBarToken
                    && self.is_false_expression(&binary.left)
                    && self.is_false_expression(&binary.right));
        }
        false
    }

    pub fn is_symbol_assigned_definitely(&mut self, symbol: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("is_symbol_assigned_definitely"); 
        self.ensure_assignments_marked(symbol);
        self.marked_assignment_symbol_links
            .get(symbol)
            .map(|links| links.has_definite_assignment)
            .unwrap_or(false)
    }

    pub fn is_symbol_assigned(&mut self, symbol: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("is_symbol_assigned"); 
        self.ensure_assignments_marked(symbol);
        self.marked_assignment_symbol_links
            .get(symbol)
            .map(|links| links.last_assignment_pos != 0)
            .unwrap_or(false)
    }

    pub fn get_resolved_symbol_on_demand(&mut self, node: &Arc<Node>) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_resolved_symbol_on_demand"); 
        if let Some(cached) = self.get_resolved_symbol_or_nil(node) {
            return Some(cached);
        }
        let resolved = if tsox_frontend::ast::node_is_missing(Some(node)) {
            None
        } else {
            self.resolve_name(
                node.text(),
                node,
                SymbolFlags::VALUE | SymbolFlags::ExportValue,
                false,
            )
        };
        if let Some(symbol) = &resolved {
            self.symbol_node_links
                .get_or_default(node)
                .resolved_symbol = Some(Arc::clone(symbol));
        }
        resolved
    }

    pub fn is_past_last_assignment(
        &mut self,
        symbol: &Arc<Symbol>,
        location: Option<&Arc<Node>>,
    ) -> bool { ::tsox_core::fntrace::enter("is_past_last_assignment"); 
        self.ensure_assignments_marked(symbol);
        let last_assignment_pos = self
            .marked_assignment_symbol_links
            .get(symbol)
            .map(|links| links.last_assignment_pos)
            .unwrap_or(0);
        last_assignment_pos == 0
            || location.is_some_and(|location| (last_assignment_pos as i64) < location.pos() as i64)
    }

    pub fn ensure_assignments_marked(&mut self, symbol: &Arc<Symbol>) { ::tsox_core::fntrace::enter("ensure_assignments_marked"); 
        let Some(value_declaration) = symbol.value_declaration.clone() else {
            return;
        };
        let parent = find_ancestor(&value_declaration, is_function_or_source_file);
        let Some(parent) = parent else {
            return;
        };
        if !self
            .node_links
            .get(&parent)
            .map(|links| links.flags.intersects(NodeCheckFlags::AssignmentsMarked))
            .unwrap_or(false)
        {
            if let Some(links) = self.node_links.get_mut(&parent) {
                links.flags |= NodeCheckFlags::AssignmentsMarked;
            }
            if !self.has_parent_with_assignments_marked(&parent) {
                self.mark_node_assignments(&parent);
            }
        }
    }

    pub fn has_parent_with_assignments_marked(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_parent_with_assignments_marked"); 
        let mut current = node.parent();
        while let Some(current_node) = current {
            if is_function_or_source_file(&current_node)
                && self
                    .node_links
                    .get(&current_node)
                    .map(|links| links.flags.intersects(NodeCheckFlags::AssignmentsMarked))
                    .unwrap_or(false)
            {
                return true;
            }
            current = current_node.parent();
        }
        false
    }

    pub fn extend_assignment_position(&self, node: &Arc<Node>, declaration: &Arc<Node>) -> i32 { ::tsox_core::fntrace::enter("extend_assignment_position"); 
        let mut pos = node.pos();
        let mut current = Some(Arc::clone(node));
        while let Some(node) = current {
            if node.pos() > declaration.pos() {
                match node.kind {
                    SyntaxKind::VariableStatement
                    | SyntaxKind::ExpressionStatement
                    | SyntaxKind::IfStatement
                    | SyntaxKind::DoStatement
                    | SyntaxKind::WhileStatement
                    | SyntaxKind::ForStatement
                    | SyntaxKind::ForInStatement
                    | SyntaxKind::ForOfStatement
                    | SyntaxKind::WithStatement
                    | SyntaxKind::SwitchStatement
                    | SyntaxKind::TryStatement
                    | SyntaxKind::ClassDeclaration => {
                        pos = node.end();
                    }
                    _ => {}
                }
            }
            current = node.parent();
        }
        pos as i32
    }

    pub fn get_assigned_type_of_spread_expression(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_assigned_type_of_spread_expression"); 
        let parent = node.parent().expect("spread expression has parent");
        let assigned = self.get_assigned_type(&parent);
        self.get_type_of_destructured_spread_expression(&assigned)
    }

    pub fn mark_node_assignments(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("mark_node_assignments"); 
        match node.kind {
            SyntaxKind::Identifier => {
                let assignment_kind = get_assignment_target_kind(node);
                if assignment_kind != AssignmentKind::None {
                    if let Some(symbol) = self.get_resolved_symbol_on_demand(node) {
                        if self.is_parameter_or_mutable_local_variable(&symbol) {
                            let should_update = {
                                let links = self.marked_assignment_symbol_links.get_or_default(&symbol);
                                let pos = links.last_assignment_pos;
                                pos == 0 || pos != i32::MAX
                            };
                            if should_update {
                                let referencing_function =
                                    find_ancestor(node, is_function_or_source_file);
                                let declaring_function = symbol
                                    .value_declaration
                                    .as_ref()
                                    .and_then(|d| find_ancestor(d, is_function_or_source_file));
                                let same_function = match (&referencing_function, &declaring_function) {
                                    (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                                    (None, None) => true,
                                    _ => false,
                                };
                                if same_function {
                                    if let Some(declaration) = symbol.value_declaration.as_ref() {
                                        let pos =
                                            self.extend_assignment_position(node, declaration);
                                        self.marked_assignment_symbol_links
                                            .get_or_default(&symbol)
                                            .last_assignment_pos = pos;
                                    }
                                } else {
                                    self.marked_assignment_symbol_links
                                        .get_or_default(&symbol)
                                        .last_assignment_pos = i32::MAX;
                                }
                            }
                            if assignment_kind == AssignmentKind::Definite {
                                self.marked_assignment_symbol_links
                                    .get_or_default(&symbol)
                                    .has_definite_assignment = true;
                            }
                        }
                    }
                }
                return false;
            }
            SyntaxKind::ExportSpecifier => {
                let NodeData::ExportSpecifier(specifier) = &node.data else {
                    return false;
                };
                let export_declaration = node
                    .parent()
                    .and_then(|p| p.parent())
                    .filter(|p| p.kind == SyntaxKind::ExportDeclaration);
                if let Some(export_declaration) = export_declaration {
                    let NodeData::ExportDeclaration(export_data) = &export_declaration.data else {
                        return false;
                    };
                    let name = specifier
                        .property_name
                        .clone()
                        .unwrap_or_else(|| Arc::clone(&specifier.name));
                    if !specifier.is_type_only
                        && !export_data.is_type_only
                        && export_data.module_specifier.is_none()
                        && !is_string_literal(&name)
                    {
                        if let Some(symbol) =
                            self.resolve_entity_name(&name, SymbolFlags::VALUE, true, true, None)
                        {
                            if self.is_parameter_or_mutable_local_variable(&symbol) {
                                self.marked_assignment_symbol_links
                                    .get_or_default(&symbol)
                                    .last_assignment_pos = i32::MAX;
                            }
                        }
                    }
                }
                return false;
            }
            SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::EnumDeclaration => {
                return false;
            }
            _ => {
                if is_type_node(node) {
                    return false;
                }
                return node.for_each_child(|child| self.mark_node_assignments(child));
            }
        }
    }
}
