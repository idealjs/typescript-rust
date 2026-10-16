#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::ls::lsutil_user_preferences::{QuotePreference, UserPreferences};
use tsox_checker::checker::Checker;
use tsox_checker::checker::mig::m2a::r18k8_flags::ConstantValue;
use tsox_core::core;
use tsox_core::tspath;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::ast::mig::m3b_2::NodeFactory;
use tsox_frontend::astnav;
use tsox_frontend::scanner;
use tsox_frontend::scanner::{TOKEN_FLAGS_NONE, TOKEN_FLAGS_SINGLE_QUOTE};

use super::m5q2::SORT_TEXT_GLOBALS_OR_KEYWORDS;
use super::m5q_3::{client_supports_item_snippet, create_snippet_printer, SnippetPrinter, SORT_TEXT_LOCATION_PRIORITY};
use super::m5r::{get_potentially_invalid_import_specifier, str_ptr_to, COMPLETION_SOURCE_SWITCH_CASES};
use super::m5q2b_2::supplemental_file_index_m5q2b;
use super::m5q2b_3::lsproto;
use super::m5q2b_6::{
    format_number, format_number_abs, is_literal_type_m5q2b, new_case_clause_tracker,
    tracker_add_value_m5q2b, tracker_has_value_m5q2b, type_node_to_expression_m5q2b,
};

impl crate::ls::mig::m5q_3::SnippetPrinter {
    pub fn print_and_format_node_with_settings(
        &mut self,
        node: &Arc<Node>,
        _source_file: &Arc<SourceFile>,
        _format_options: &crate::ls::lsutil_format_code_options::FormatCodeSettings,
    ) -> String { ::tsox_core::fntrace::enter("print_and_format_node_with_settings"); 
        self.print_unescaped_node(node)
    }
}

impl crate::ls::language_service::LanguageService {
    #[allow(clippy::too_many_arguments)]
    pub fn get_exhaustive_case_snippets(
        &self,
        case_block: &Arc<Node>,
        file: &Arc<SourceFile>,
        position: usize,
        options: &core::compiler_options::CompilerOptions,
        program: &tsox_compile::compiler::Program,
        c: &mut Checker,
    ) -> Option<Option<lsproto::CompletionItem>> { ::tsox_core::fntrace::enter("get_exhaustive_case_snippets"); 
        let clauses: &[Arc<Node>] = match &case_block.data {
            ast::node_data_generated::NodeData::CaseBlock(d) => &d.clauses.nodes,
            _ => &[],
        };
        let switch_parent = case_block.parent();
        let Some(switch_expression) = switch_parent
            .as_ref()
            .and_then(|p| p.expression().cloned())
        else {
            return Some(None);
        };
        let switch_type = c.get_type_at_location(&switch_expression);
        let switch_type_types: &[Arc<tsox_checker::checker::Type>] =
            switch_type.types().unwrap_or(&[]);
        if !(switch_type.is_union()
            && core::core::every(switch_type_types, |t| is_literal_type_m5q2b(t)))
        {
            return Some(None);
        }

        let mut tracker = new_case_clause_tracker(c, clauses);
        let target = options.get_emit_script_target();
        let quote_preference = crate::ls::lsutil_utilities::get_quote_preference(file, self.user_preferences());
        // ImportAdder::new 需 Arc<Checker>（progress_notes_r59A.md 已登记的 clone_arc 缺口），暂以 None 缺省
        let mut import_adder: Option<crate::ls::autoimport_import_adder::ImportAdder> = None;

        let mut elements: Vec<Arc<Node>> = Vec::new();
        let factory = NodeFactory::new();
        for t in switch_type.types().unwrap_or(&[]) {
            if t.is_enum_literal() {
                let symbol = t.symbol();
                let mut enum_value: Option<ConstantValue> = None;
                if let Some(symbol) = &symbol {
                    if let Some(value_declaration) = &symbol.value_declaration {
                        if let Some(value) = c.get_constant_value(value_declaration) {
                            enum_value = Some(ConstantValue::String(value));
                        }
                    }
                }
                if let Some(enum_value) = enum_value {
                    if tracker_has_value_m5q2b(&tracker, &enum_value) {
                        continue;
                    }
                    tracker_add_value_m5q2b(&mut tracker, &enum_value);
                }
                let type_node = crate::ls::autoimport_import_adder::type_to_auto_importable_type_node(
                    c,
                    import_adder.as_mut().map(|ia| ia as &mut dyn crate::ls::autoimport_import_adder::ImportAdderTrait),
                    &t,
                    case_block,
                );
                let Some(type_node) = type_node else {
                    return Some(None);
                };
                let Some(expr) = type_node_to_expression_m5q2b(&type_node, target, quote_preference, &factory) else {
                    return Some(None);
                };
                elements.push(expr);
            } else {
                let Some(literal_type) = t.as_literal_type() else {
                    continue;
                };
                let value: ConstantValue = match &literal_type.value {
                    tsox_checker::checker::types::LiteralValue::String(s) => ConstantValue::String(s.clone()),
                    tsox_checker::checker::types::LiteralValue::Number(n) => ConstantValue::Number(n.0),
                    tsox_checker::checker::types::LiteralValue::BigInt(b) => ConstantValue::BigInt(b.clone()),
                    _ => continue,
                };
                if tracker_has_value_m5q2b(&tracker, &value) {
                    continue;
                }
                match &value {
                    ConstantValue::BigInt(big_int) => {
                        let mut negative = big_int.negative;
                        let literal;
                        if negative {
                            negative = false;
                            literal = new_prefix_unary_expression_m5q2b4(
                                &factory,
                                SyntaxKind::MinusToken,
                                factory.new_big_int_literal(&format!("{}n", big_int.base10_value), TOKEN_FLAGS_NONE),
                            );
                        } else {
                            literal = factory.new_big_int_literal(&format!("{}n", big_int.base10_value), TOKEN_FLAGS_NONE);
                        }
                        elements.push(literal);
                    }
                    ConstantValue::Number(number) => {
                        let number_literal;
                        if *number < 0.0 {
                            number_literal = new_prefix_unary_expression_m5q2b4(
                                &factory,
                                SyntaxKind::MinusToken,
                                factory.new_numeric_literal(&format_number_abs(*number), TOKEN_FLAGS_NONE),
                            );
                        } else {
                            number_literal = factory.new_numeric_literal(&format_number(*number), TOKEN_FLAGS_NONE);
                        }
                        elements.push(number_literal);
                    }
                    ConstantValue::String(string) => {
                        let literal = factory.new_string_literal(
                            string.as_str(),
                            if quote_preference == QuotePreference::Single {
                                TOKEN_FLAGS_SINGLE_QUOTE
                            } else {
                                TOKEN_FLAGS_NONE
                            },
                        );
                        elements.push(literal);
                    }
                    _ => {}
                }
            }
        }
        if elements.is_empty() {
            return Some(None);
        }

        let new_clauses: Vec<Arc<Node>> = elements
            .iter()
            .map(|element| {
                new_case_clause_m5q2b4(
                    &factory,
                    Arc::clone(element),
                    factory.new_node_list(Vec::new()),
                )
            })
            .collect();
        let new_line_char = self.format_options().new_line_character.clone();
        let mut printer = create_snippet_printer(
            tsox_frontend::format::mig::m4o_2::PrinterOptions {
                remove_comments: true,
                new_line: core::mig::m3j_3::get_new_line_kind(&new_line_char),
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
            None,
        );
        let format_code_settings = crate::ls::change_tracker_impl::get_format_code_settings_for_writing(self.format_options().clone(), file);
        let mut insert_text_parts: Vec<String> = Vec::with_capacity(new_clauses.len());
        for (i, clause) in new_clauses.iter().enumerate() {
            if client_supports_item_snippet(&crate::mig::m5m::ResolvedClientCapabilitiesContext {
                capabilities: None,
            }) {
                insert_text_parts.push(format!(
                    "{}${}",
                    printer.print_and_format_node_with_settings(clause, file, &format_code_settings),
                    i + 1
                ));
            } else {
                insert_text_parts.push(printer.print_unescaped_node(clause));
            }
        }
        let insert_text = insert_text_parts.join(&new_line_char);

        let first_clause = printer.print_unescaped_node(&new_clauses[0]);
        let name = format!("{} ...", first_clause);

        let mut additional_text_edits: Option<Vec<lsproto::TextEdit>> = None;
        if let Some(adder) = import_adder.as_mut() {
            let edits = crate::ls::autoimport_import_adder::ImportAdderTrait::edits(adder);
            if !edits.is_empty() {
                additional_text_edits = Some(edits);
            }
        }

        let mut item = lsproto::CompletionItem::default();
        item.label = name.clone();
        item.kind = Some(lsproto::CompletionItemKind::Snippet);
        item.sort_text = Some(SORT_TEXT_GLOBALS_OR_KEYWORDS.to_string());
        item.insert_text = str_ptr_to(&insert_text);
        item.additional_text_edits = additional_text_edits;
        item.insert_text_format = if client_supports_item_snippet(&crate::mig::m5m::ResolvedClientCapabilitiesContext {
            capabilities: None,
        }) {
            Some(lsproto::InsertTextFormat::Snippet)
        } else {
            None
        };
        item.data = Some(lsproto::CompletionItemData {
            file_name: ast::mig::m3b_2::original_file_name(file).to_string(),
            position: position as i32,
            supplemental_file_index: supplemental_file_index_m5q2b(file),
            name,
            source: COMPLETION_SOURCE_SWITCH_CASES.to_string(),
            ..Default::default()
        });
        Some(Some(item))
    }
}

fn new_prefix_unary_expression_m5q2b4(
    _factory: &NodeFactory,
    operator: SyntaxKind,
    operand: Arc<Node>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("new_prefix_unary_expression_m5q2b4"); 
    Arc::new(Node::new(
        SyntaxKind::PrefixUnaryExpression,
        ast::node_data_generated::NodeData::PrefixUnaryExpression(
            ast::node_data_generated::PrefixUnaryExpressionData { operator, operand },
        ),
    ))
}

fn new_case_clause_m5q2b4(
    _factory: &NodeFactory,
    expression: Arc<Node>,
    statements: tsox_frontend::ast::node_node_list::NodeList,
) -> Arc<Node> { ::tsox_core::fntrace::enter("new_case_clause_m5q2b4"); 
    Arc::new(Node::new(
        SyntaxKind::CaseClause,
        ast::node_data_generated::NodeData::CaseOrDefaultClause(
            ast::node_data_generated::CaseOrDefaultClauseData {
                expression,
                statements: Arc::new(statements),
            },
        ),
    ))
}
