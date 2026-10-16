#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashSet;
use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{self, Node, SourceFile, SyntaxKind};

use super::m5x_3::range_contains_range;

pub struct M5xCaseClauseTrackerState {
    existing_strings: HashSet<String>,
    existing_numbers: HashSet<String>,
    existing_big_ints: HashSet<String>,
}

impl M5xCaseClauseTrackerState {
    pub fn add_value_string(&mut self, value: &str) { ::tsox_core::fntrace::enter("add_value_string"); 
        self.existing_strings.insert(value.to_string());
    }

    pub fn add_value_number(&mut self, value: &str) { ::tsox_core::fntrace::enter("add_value_number"); 
        self.existing_numbers.insert(value.to_string());
    }

    pub fn has_value_string(&self, value: &str) -> bool { ::tsox_core::fntrace::enter("has_value_string"); 
        self.existing_strings.contains(value)
    }

    pub fn has_value_number(&self, value: &str) -> bool { ::tsox_core::fntrace::enter("has_value_number"); 
        self.existing_numbers.contains(value)
    }

    pub fn has_value_big_int(&self, value: &str) -> bool { ::tsox_core::fntrace::enter("has_value_big_int"); 
        self.existing_big_ints.contains(value)
    }
}

pub fn new_case_clause_tracker(
    type_checker: &mut tsox_checker::checker::Checker,
    clauses: &[Arc<Node>],
) -> M5xCaseClauseTrackerState { ::tsox_core::fntrace::enter("new_case_clause_tracker"); 
    let mut c = M5xCaseClauseTrackerState {
        existing_strings: HashSet::new(),
        existing_numbers: HashSet::new(),
        existing_big_ints: HashSet::new(),
    };
    for clause in clauses {
        if clause.kind != SyntaxKind::DefaultClause {
            let Some(expression) = crate::ls::mig::m5x_3::case_clause_expression(clause) else {
                continue;
            };
            let expression = ast::skip_parentheses(&expression);
            if ast::is_literal_expression(&expression) {
                match expression.kind {
                    SyntaxKind::NoSubstitutionTemplateLiteral | SyntaxKind::StringLiteral => {
                        c.existing_strings.insert(ast::node_text(&expression).to_string());
                    }
                    SyntaxKind::NumericLiteral => {
                        c.existing_numbers.insert(ast::node_text(&expression).to_string());
                    }
                    SyntaxKind::BigIntLiteral => {
                        c.existing_big_ints.insert(ast::node_text(&expression).to_string());
                    }
                    _ => {}
                }
            } else if let Some(symbol) = type_checker.get_symbol_at_location(&expression) {
                if let Some(value_declaration) = symbol.value_declaration.as_ref() {
                    if ast::is_enum_member(value_declaration) {
                        if let Some(enum_value) =
                            type_checker.get_constant_value(value_declaration)
                        {
                            c.add_value_string(&enum_value);
                        }
                    }
                }
            }
        }
    }
    c
}

pub fn find_containing_list(node: &Arc<Node>, file: &Arc<SourceFile>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("find_containing_list"); 
    let mut list: Option<Arc<Node>> = None;
    let Some(parent) = node.parent() else {
        return None;
    };
    visit_nodes(&parent, file, node, &mut list);
    list
}

fn visit_nodes(
    container: &Arc<Node>,
    file: &Arc<SourceFile>,
    target: &Arc<Node>,
    list: &mut Option<Arc<Node>>,
) { ::tsox_core::fntrace::enter("visit_nodes"); 
    for child in ast::mig::m3b::iter_children(container) {
        if range_contains_range(child.loc, target.loc) {
            *list = Some(Arc::clone(&child));
        }
        visit_nodes(&child, file, target, list);
    }
}

pub fn is_equality_operator_kind(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_equality_operator_kind"); 
    matches!(
        kind,
        SyntaxKind::EqualsEqualsEqualsToken
            | SyntaxKind::EqualsEqualsToken
            | SyntaxKind::ExclamationEqualsEqualsToken
            | SyntaxKind::ExclamationEqualsToken
    )
}

pub fn get_switched_type(
    case_clause: &Arc<Node>,
    type_checker: &mut tsox_checker::checker::Checker,
) -> Option<Arc<tsox_checker::checker::types::Type>> { ::tsox_core::fntrace::enter("get_switched_type"); 
    let switch_statement = crate::ls::mig::m5x_3::case_clause_parent_switch_statement(case_clause)?;
    let expression = crate::ls::mig::m5x_3::switch_statement_expression(&switch_statement)?;
    Some(type_checker.get_type_at_location(&expression))
}

pub fn get_range_of_node(
    node: &Arc<Node>,
    file: &Arc<SourceFile>,
    end_node: Option<&Arc<Node>>,
) -> TextRange { ::tsox_core::fntrace::enter("get_range_of_node"); 
    let start = tsox_frontend::astnav::get_start_of_node(node, file, false);
    let end = end_node.map(|n| n.end()).unwrap_or_else(|| node.end());
    TextRange::new(start, end)
}

pub fn is_module_specifier_like(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_module_specifier_like"); 
    ast::is_string_literal_like(node)
        || (ast::is_template_expression(node)
            && crate::ls::mig::m5x_3::template_expression_head(node)
                .map_or(false, |head| ast::node_text(&head).is_empty()))
}

pub fn is_parameter_declaration(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_parameter_declaration"); 
    node.kind == SyntaxKind::Parameter
}

pub fn get_possible_type_arguments_info_worker(
    token_in: &Arc<Node>,
    source_file: &Arc<SourceFile>,
) -> Option<super::m5x_5::M5xPossibleTypeArgumentInfo> { ::tsox_core::fntrace::enter("get_possible_type_arguments_info_worker"); 
    let mut token = Arc::clone(token_in);
    let mut token_at_position = if ast::is_identifier(&token) || token.kind == SyntaxKind::LessThanToken {
        Some(Arc::clone(&token))
    } else {
        let t = tsox_frontend::astnav::find_preceding_token(&source_file.node, token.end());
        t.filter(|t| ast::is_identifier(t))
    };
    let Some(token_at_position) = token_at_position.take() else {
        return None;
    };
    token = token_at_position;
    if ast::is_identifier(&token) && token.parent().map_or(true, |p| p.kind != SyntaxKind::LessThanToken) {
        let mut preceding = token.parent();
        if let Some(parent) = preceding.clone() {
            if ast::is_qualified_name(&parent)
                && crate::ls::mig::m5u_2::qualified_name_right(&parent)
                    .is_some_and(|r| Arc::ptr_eq(&r, &token))
            {
                preceding = parent.parent();
            }
        }
        match preceding {
            Some(p) if p.kind == SyntaxKind::LessThanToken => token = p,
            _ => return None,
        }
    }
    let mut token = token;
    if token.kind != SyntaxKind::LessThanToken {
        return None;
    }
    let mut called: Option<Arc<Node>> = token.parent().filter(|p| {
        matches!(
            p.kind,
            SyntaxKind::CallExpression
                | SyntaxKind::NewExpression
                | SyntaxKind::TaggedTemplateExpression
        )
    });
    let mut n_type_arguments = 0usize;
    while let Some(current) = called {
        if current.kind == SyntaxKind::LessThanToken {
            return None;
        }
        if current.kind == SyntaxKind::CallExpression
            || current.kind == SyntaxKind::NewExpression
        {
            let invocation = current.clone();
            let type_arg_count = crate::ls::mig::m5x_3::call_or_new_expression_type_arguments(&invocation)
                .map_or(0, |args| args.len());
            if type_arg_count != 0 {
                n_type_arguments += type_arg_count;
            }
            if let Some(expr) = crate::ls::mig::m5x_3::call_or_new_expression_expression(&invocation) {
                let expr = crate::ls::mig::m5x::walk_up_parenthesized_expressions(&expr);
                if ast::is_identifier(&expr) || ast::is_property_access_expression(&expr) {
                    return Some(super::m5x_5::M5xPossibleTypeArgumentInfo {
                        called: expr,
                        n_type_arguments,
                    });
                }
                return None;
            }
        }
        called = current.parent();
    }
    None
}
