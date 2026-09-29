#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::ls::find_all_references::{EntryKind, RefOptions, ReferenceUse};
use tsox_frontend::ast::mig::m3e_4::for_each_return_statement;
use tsox_frontend::ast::mig::m3f_2::has_initializer;
use tsox_frontend::ast::mig::m3f_4::is_declaration_name;
use tsox_frontend::ast::mig::m3g::is_new_expression_target;
use tsox_frontend::ast::subtree_facts::SubtreeContainsJsx;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};

use super::m5s_3::{ImpExpKind, RefSearch, RefState};

use tsox_frontend::ast::mig::m3b;
use tsox_frontend::ast::NodeData;

fn node_body(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::FunctionDeclaration(d) => d.body.as_ref(),
        NodeData::FunctionExpression(d) => Some(&d.body),
        NodeData::ArrowFunction(d) => Some(&d.body),
        NodeData::MethodDeclaration(d) => d.body.as_ref(),
        NodeData::ConstructorDeclaration(d) => d.body.as_ref(),
        NodeData::GetAccessorDeclaration(d) => d.body.as_ref(),
        NodeData::SetAccessorDeclaration(d) => d.body.as_ref(),
        _ => None,
    }
}

pub fn jsx_element_opening_and_closing(node: &Node) -> Option<(Arc<Node>, Arc<Node>)> {
    match &node.data {
        NodeData::JsxElement(d) => Some((Arc::clone(&d.opening_element), Arc::clone(&d.closing_element))),
        _ => None,
    }
}

impl<'a> RefState<'a> {
    pub fn add_implementation_references(&mut self, ref_node: &Arc<Node>, add_ref: &mut impl FnMut(&Arc<Node>)) {
        if is_declaration_name(ref_node)
            && ref_node.parent().map_or(false, |p| super::m5x_3::is_implementation(&p))
        {
            add_ref(ref_node);
            return;
        }

        if ref_node.kind != SyntaxKind::Identifier {
            return;
        }

        if let Some(parent) = ref_node.parent() {
            if parent.kind == SyntaxKind::ShorthandPropertyAssignment {
                super::m5t_4::get_reference_entries_for_shorthand_property_assignment(ref_node, &mut self.checker, add_ref);
            }
        }

        if let Some(containing_node) = super::m5x_3::get_containing_node_if_in_heritage_clause(ref_node) {
            add_ref(&containing_node);
            return;
        }

        let type_node = ast::find_ancestor(ref_node, |a: &Node| match a.parent() {
            Some(parent) => {
                !ast::is_qualified_name(&parent) && !ast::is_type_node(&parent) && !ast::is_type_element(&parent)
            }
            None => true,
        });
        let Some(type_node) = type_node else {
            return;
        };
        let Some(type_having_node) = type_node.parent() else {
            return;
        };
        let is_type_reference = type_having_node
            .type_node()
            .map(|t| Arc::ptr_eq(t, &type_node))
            .unwrap_or(false);
        if !is_type_reference {
            return;
        }
        if !self.seen_containing_type_references_add_if_absent(&type_having_node) {
            return;
        }

        if has_initializer(&type_having_node) {
            if let Some(initializer) = m3b::initializer(&type_having_node) {
                add_if_implementation_expression(initializer, add_ref);
            }
        } else if ast::is_function_like(&type_having_node) {
            if let Some(body) = node_body(&type_having_node) {
                if body.kind == SyntaxKind::Block {
                    for_each_return_statement(body, |return_statement| {
                        if let Some(expr) = return_statement.expression() {
                            add_if_implementation_expression(expr, add_ref);
                        }
                        false
                    });
                } else {
                    add_if_implementation_expression(body, add_ref);
                }
            }
        } else if ast::is_assertion_expression(&type_having_node) || ast::is_satisfies_expression(&type_having_node) {
            if let Some(expr) = type_having_node.expression() {
                add_if_implementation_expression(expr, add_ref);
            }
        }
    }

    pub fn add_constructor_references(
        &mut self,
        reference_location: &Arc<Node>,
        symbol: &Arc<Symbol>,
        search: &RefSearch,
        add_references_here: bool,
    ) {
        if is_new_expression_target(reference_location, false, false) && add_references_here {
            self.add_reference(reference_location.clone(), Some(symbol.clone()), EntryKind::Node);
        }

        if let Some(parent) = reference_location.parent() {
            if ast::is_class_like(&parent) {
                if let Some(source_file) = super::m5t_3::source_file_of_node(&self.program, reference_location) {
                    let search_symbol = search.symbol.clone();
                    super::m5s_3::find_own_constructor_references(Some(&search_symbol), &source_file, &mut |n| {
                        let mut adder = self.reference_adder(&search_symbol);
                        adder(&n, EntryKind::Node);
                    });
                }
            } else if let Some(class_extending) = super::m5t_4::try_get_class_by_extending_identifier(reference_location) {
                super::m5s_3::find_super_constructor_accesses(&class_extending, &mut |n| {
                    let mut adder = self.reference_adder(&search.symbol);
                    adder(&n, EntryKind::Node);
                });
                self.find_inherited_constructor_references(&class_extending);
            }
        }
    }

    pub fn add_class_static_this_references(
        &mut self,
        reference_location: &Arc<Node>,
        symbol: &Arc<Symbol>,
        search: &RefSearch,
        add_references_here: bool,
    ) {
        if add_references_here {
            self.add_reference(reference_location.clone(), Some(symbol.clone()), EntryKind::Node);
        }

        let Some(class_like) = reference_location.parent() else {
            return;
        };
        if self.options.use_ == ReferenceUse::Rename || !ast::is_class_like(&class_like) {
            return;
        }

        let search_symbol = search.symbol.clone();
        let mut adder = self.reference_adder(&search_symbol);
        for member in m3b::members(&class_like) {
            if !(super::m5t_4::is_method_or_accessor(member) && ast::has_static_modifier(member)) {
                continue;
            }
            if let Some(body) = node_body(member) {
                visit_static_this_reference(body, &mut |n, kind| adder(n, kind));
            }
        }
    }

    pub fn find_inherited_constructor_references(&mut self, class_declaration: &Arc<Node>) {
        if super::m5t_4::has_own_constructor(class_declaration) {
            return;
        }
        let Some(class_symbol) = self.checker.get_symbol_of_node(class_declaration) else {
            return;
        };
        let search = RefSearch {
            symbol: class_symbol.clone(),
            coming_from: ImpExpKind::None,
            text: String::new(),
            all_search_symbols: Vec::new(),
            check_children: false,
            is_crossing_had_file: false,
        };
        self.get_references_in_container_or_files(&class_symbol, &search);
    }
}

fn add_if_implementation_expression(expr: &Arc<Node>, add_ref: &mut impl FnMut(&Arc<Node>)) {
    if super::m5x_3::is_implementation_expression(expr) {
        add_ref(expr);
    }
}

fn visit_static_this_reference(node: &Arc<Node>, add_ref: &mut impl FnMut(&Arc<Node>, EntryKind)) {
    if node.kind == SyntaxKind::ThisKeyword {
        add_ref(node, EntryKind::Node);
    } else if !ast::is_function_like(node) && !ast::is_class_like(node) {
        ast::for_each_child(node, |child| {
            visit_static_this_reference(child, add_ref);
            false
        });
    }
}

pub fn find_first_jsx_node(root: &Arc<Node>) -> Option<Arc<Node>> {
    fn visit(node: &Arc<Node>) -> Option<Arc<Node>> {
        match node.kind {
            SyntaxKind::JsxElement | SyntaxKind::JsxSelfClosingElement | SyntaxKind::JsxFragment => {
                return Some(node.clone());
            }
            _ => {}
        }
        if !node.subtree_facts().contains(SubtreeContainsJsx) {
            return None;
        }
        let mut result = None;
        ast::for_each_child(node, |child| {
            result = visit(child);
            result.is_some()
        });
        result
    }
    visit(root)
}
