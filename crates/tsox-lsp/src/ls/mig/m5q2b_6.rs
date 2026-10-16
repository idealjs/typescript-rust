#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashSet;
use std::sync::Arc;

use crate::lsp::lsproto;
use crate::ls::lsutil_user_preferences::{QuotePreference, UserPreferences};
use tsox_checker::checker::Checker;
use tsox_checker::checker::mig::m2a::r18k8_flags::ConstantValue;
use tsox_core::core;
use tsox_core::jsnum;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::ast::mig::m3b_2::NodeFactory;
use tsox_frontend::astnav;
use tsox_frontend::scanner;

use super::m5q2::{ImportStatementCompletionInfo, get_jsdoc_param_name_with_initializer, generate_jsdoc_param_tags_for_destructuring, SORT_TEXT_GLOBALS_OR_KEYWORDS};
use super::m5q_3::{create_snippet_printer, escape_snippet_text, get_completions_symbol_kind, get_dot_accessor, client_supports_item_snippet, SnippetPrinter, SORT_TEXT_LOCATION_PRIORITY};
use super::m5r::{get_dot_accessor as get_dot_accessor_m5r, get_word_length_and_start, is_module_specifier_missing_or_empty, get_potentially_invalid_import_specifier, is_non_contextual_keyword, str_ptr_to, COMPLETION_SOURCE_SWITCH_CASES};

/// Go internal/ls/utilities.go caseClauseTrackerState 的最小等价移植。
pub struct CaseClauseTracker {
    existing_strings: HashSet<String>,
    existing_numbers: HashSet<jsnum::Number>,
    existing_big_ints: HashSet<String>,
}

impl CaseClauseTracker {
    fn add_value(&mut self, value: &ConstantValue) { ::tsox_core::fntrace::enter("add_value"); 
        match value {
            ConstantValue::String(v) => {
                self.existing_strings.insert(v.clone());
            }
            ConstantValue::Number(v) => {
                self.existing_numbers.insert(jsnum::Number(*v));
            }
            ConstantValue::BigInt(v) => {
                self.existing_big_ints.insert(v.to_string());
            }
        }
    }

    pub fn has_value(&self, value: &ConstantValue) -> bool { ::tsox_core::fntrace::enter("has_value"); 
        match value {
            ConstantValue::String(v) => self.existing_strings.contains(v),
            ConstantValue::Number(v) => self.existing_numbers.contains(&jsnum::Number(*v)),
            ConstantValue::BigInt(v) => self.existing_big_ints.contains(&v.to_string()),
        }
    }
}

/// Go newCaseClauseTracker(ls/utilities.go) 的最小等价移植。
pub fn new_case_clause_tracker(
    type_checker: &mut Checker,
    clauses: &[Arc<Node>],
) -> CaseClauseTracker { ::tsox_core::fntrace::enter("new_case_clause_tracker"); 
    let mut tracker = CaseClauseTracker {
        existing_strings: HashSet::new(),
        existing_numbers: HashSet::new(),
        existing_big_ints: HashSet::new(),
    };
    for clause in clauses {
        if clause.kind == SyntaxKind::DefaultClause {
            continue;
        }
        let Some(expression) = clause.expression() else {
            continue;
        };
        let expression = ast::mig::m3g_3::skip_parentheses(expression);
        match expression.kind {
            SyntaxKind::NoSubstitutionTemplateLiteral | SyntaxKind::StringLiteral => {
                tracker.existing_strings.insert(expression.text().to_string());
            }
            SyntaxKind::NumericLiteral => {
                tracker
                    .existing_numbers
                    .insert(jsnum::Number::from_string(&expression.text()));
            }
            SyntaxKind::BigIntLiteral => {
                tracker
                    .existing_big_ints
                    .insert(jsnum::PseudoBigInt::parse(&expression.text()).to_string());
            }
            _ => {}
        }
    }
    tracker
}

pub fn tracker_has_value_m5q2b(tracker: &CaseClauseTracker, value: &ConstantValue) -> bool { ::tsox_core::fntrace::enter("tracker_has_value_m5q2b"); 
    tracker.has_value(value)
}

pub fn tracker_add_value_m5q2b(
    tracker: &mut CaseClauseTracker,
    value: &ConstantValue,
) { ::tsox_core::fntrace::enter("tracker_add_value_m5q2b"); 
    tracker.add_value(value)
}

pub fn format_number(n: f64) -> String { ::tsox_core::fntrace::enter("format_number"); 
    jsnum::Number(n).to_string()
}

pub fn format_number_abs(n: f64) -> String { ::tsox_core::fntrace::enter("format_number_abs"); 
    format_number(n.abs())
}

pub fn get_jsdoc_param_annotation(
    param_name: &str,
    initializer: Option<&Arc<Node>>,
    dot_dot_dot_token: Option<&Arc<Node>>,
    is_js: bool,
    is_object: bool,
    is_snippet: bool,
    type_checker: &mut Checker,
    options: &core::compiler_options::CompilerOptions,
    preferences: &UserPreferences,
    tabstop_counter: &mut i32,
) -> String { ::tsox_core::fntrace::enter("get_jsdoc_param_annotation"); 
    let mut param_name = param_name.to_string();
    if let Some(initializer) = initializer {
        param_name = get_jsdoc_param_name_with_initializer(&param_name, initializer);
    }
    if is_snippet {
        param_name = escape_snippet_text(&param_name);
    }
    if is_js {
        let mut t = "*".to_string();
        if is_object {
            t = "object".to_string();
        } else {
            if let Some(initializer) = initializer {
                let initializer_parent = initializer.parent().expect("initializer should have a parent");
                let inferred_type = type_checker.get_type_at_location(&initializer_parent);
                if inferred_type.flags
                    & (tsox_checker::checker::types::TypeFlags::ANY
                        | tsox_checker::checker::types::TypeFlags::VOID)
                    == tsox_checker::checker::types::TypeFlags::empty()
                {
                    let file_node =
                        ast::get_source_file_of_node(initializer).expect("initializer should be in a source file");
                    let file = type_checker
                        .program
                        .source_files()
                        .iter()
                        .find(|f| Arc::ptr_eq(&f.node, &file_node))
                        .cloned()
                        .expect("source file for initializer");
                    let quote_preference = crate::ls::lsutil_utilities::get_quote_preference(&file, preferences);
                    let builder_flags = if quote_preference == QuotePreference::Single {
                        tsox_checker::checker::symboltracker::NodeBuilderFlags::UseSingleQuotesForStringLiteralType
                    } else {
                        tsox_checker::checker::symboltracker::NodeBuilderFlags::None
                    };
                    let type_node = type_checker.type_to_type_node(&inferred_type);
                    let mut p = tsox_frontend::format::mig::m4o_2::new_printer(
                        tsox_frontend::format::mig::m4o_2::PrinterOptions {
                            remove_comments: true,
                            new_line: tsox_core::core::compiler_options_kinds::NewLineKind::None,
                            omit_trailing_semicolon: false,
                            no_emit_helpers: false,
                            target: tsox_core::core::compiler_options_kinds::ScriptTarget::None,
                            source_map: false,
                            inline_source_map: false,
                            inline_sources: false,
                            omit_brace_source_map_positions: false,
                            only_print_jsdoc_style: false,
                            never_ascii_escape: false,
                            preserve_source_newlines: false,
                            terminate_unterminated_literals: false,
                        },
                        tsox_frontend::format::mig::m4o_2::PrintHandlers {
                            has_global_name: None,
                            map_source_position: None,
                            on_before_emit_node: None,
                            on_after_emit_node: None,
                            on_before_emit_node_list: None,
                            on_after_emit_node_list: None,
                            on_before_emit_token: None,
                            on_after_emit_token: None,
                        },
                        tsox_frontend::format::mig::m4o_2::EmitContext::default(),
                    );
                    t = p.emit(&type_node, Some(&file));
                }
            }
            if is_snippet && t == "*" {
                let tabstop = *tabstop_counter;
                *tabstop_counter += 1;
                t = format!("${{{}:{}}}", tabstop, t);
            }
        }
        let dot_dot_dot = if !is_object && dot_dot_dot_token.is_some() {
            "..."
        } else {
            ""
        };
        let mut description = String::new();
        if is_snippet {
            let tabstop = *tabstop_counter;
            *tabstop_counter += 1;
            description = format!("${{{}}}", tabstop);
        }
        format!("@param {{{}{}}} {} {}", dot_dot_dot, t, param_name, description)
    } else {
        let mut description = String::new();
        if is_snippet {
            let tabstop = *tabstop_counter;
            *tabstop_counter += 1;
            description = format!("${{{}}}", tabstop);
        }
        format!("@param {} {}", param_name, description)
    }
}

pub fn is_literal_type_m5q2b(t: &tsox_checker::checker::types::Type) -> bool { ::tsox_core::fntrace::enter("is_literal_type_m5q2b"); 
    t.flags & (tsox_checker::checker::types::TypeFlags::STRING_LITERAL
        | tsox_checker::checker::types::TypeFlags::NUMBER_LITERAL
        | tsox_checker::checker::types::TypeFlags::BigIntLiteral
        | tsox_checker::checker::types::TypeFlags::ENUM_LITERAL)
        != tsox_checker::checker::types::TypeFlags::empty()
}

pub fn type_node_to_expression_m5q2b(
    type_node: &Arc<Node>,
    target: core::compiler_options::ScriptTarget,
    quote_preference: QuotePreference,
    factory: &NodeFactory,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("type_node_to_expression_m5q2b"); 
    match type_node.kind {
        SyntaxKind::TypeReference => {
            let type_name = match &type_node.data {
                tsox_frontend::ast::NodeData::TypeReferenceNode(d) => d.type_name.clone(),
                _ => return None,
            };
            entity_name_to_expression_m5q2b(&type_name, target, quote_preference, factory)
        }
        SyntaxKind::IndexedAccessType => {
            let (object_type, index_type) = match &type_node.data {
                tsox_frontend::ast::NodeData::IndexedAccessTypeNode(d) => {
                    (d.object_type.clone(), d.index_type.clone())
                }
                _ => return None,
            };
            let object_expression = type_node_to_expression_m5q2b(&object_type, target, quote_preference, factory)?;
            let index_expression = type_node_to_expression_m5q2b(&index_type, target, quote_preference, factory)?;
            Some(Arc::new(Node::new(
                SyntaxKind::ElementAccessExpression,
                tsox_frontend::ast::NodeData::ElementAccessExpression(
                    tsox_frontend::ast::node_data_generated::ElementAccessExpressionData {
                        expression: object_expression,
                        question_dot_token: None,
                        argument_expression: index_expression,
                    },
                ),
            )))
        }
        _ => None,
    }
}

fn entity_name_to_expression_m5q2b(
    entity_name: &Arc<Node>,
    target: core::compiler_options::ScriptTarget,
    quote_preference: QuotePreference,
    factory: &NodeFactory,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("entity_name_to_expression_m5q2b"); 
    if ast::is_identifier(entity_name) {
        return Some(Arc::clone(entity_name));
    }
    let (left, right) = match &entity_name.data {
        tsox_frontend::ast::NodeData::QualifiedName(d) => (d.left.clone(), d.right.clone()),
        _ => return None,
    };
    let left = entity_name_to_expression_m5q2b(&left, target, quote_preference, factory)?;
    Some(Arc::new(Node::new(
        SyntaxKind::PropertyAccessExpression,
        tsox_frontend::ast::NodeData::PropertyAccessExpression(
            tsox_frontend::ast::node_data_generated::PropertyAccessExpressionData {
                expression: left,
                question_dot_token: None,
                name: right,
            },
        ),
    )))
}
