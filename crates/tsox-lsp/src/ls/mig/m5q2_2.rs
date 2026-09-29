#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::ls::lsutil_user_preferences::{QuotePreference, UserPreferences};
use tsox_checker::checker::Checker;
use tsox_core::core;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::ast::mig::m3b_2::NodeFactory;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::astnav;
use tsox_frontend::scanner;

use super::m5q2b_3::lsproto;

use super::m5q2::{
    CompletionDataData, CompletionKind, ImportStatementCompletionInfo, MemberCompletionEntry,
    ObjectLiteralMethodSymbol, get_completion_entry_display_name_for_symbol, get_contextual_keywords,
    get_keyword_completions, get_line_of_position, SORT_TEXT_JAVASCRIPT_IDENTIFIERS,
};
use super::m5q_3::{
    create_snippet_printer, deprecate_sort_text, escape_snippet_text, get_completions_symbol_kind, sort_below,
    is_string_and_empty_anonymous_object_intersection, SnippetPrinter,
    SORT_TEXT_LOCATION_PRIORITY,
};
use super::m5r::{
    get_source_from_origin, is_checked_file, is_deprecated, is_object_literal_method_completion_candidate_declaration,
    is_object_literal_method_symbol, is_recommended_completion_match, quote_property_name, symbol_name, SymbolOriginInfo,
    COMPLETION_SOURCE_CLASS_MEMBER_SNIPPET, COMPLETION_SOURCE_OBJECT_LITERAL_MEMBER_WITH_COMMA,
    COMPLETION_SOURCE_OBJECT_LITERAL_METHOD_SNIPPET, SYMBOL_ORIGIN_INFO_KIND_OBJECT_LITERAL_METHOD,
};
use super::m5r_2::{is_class_like_member_completion, KeywordCompletionFilters};
use super::m5q2::SORT_TEXT_AUTO_IMPORT_SUGGESTIONS;

type SortText = String;

#[derive(Debug, Clone, Default)]
pub struct EditRangeWithInsertReplace {
    pub insert: lsproto::Range,
    pub replace: lsproto::Range,
}

#[derive(Debug, Clone, Default)]
pub struct CompletionItemDefaults {
    pub commit_characters: Option<Vec<String>>,
    pub edit_range: Option<EditRangeWithInsertReplace>,
}

#[derive(Debug, Clone, Default)]
pub struct CompletionList {
    pub is_incomplete: bool,
    pub item_defaults: Option<CompletionItemDefaults>,
    pub items: Vec<lsproto::CompletionItem>,
}

impl crate::ls::language_service::LanguageService {
    pub fn completion_info_from_data(
        &self,
        type_checker: &mut Checker,
        file: &Arc<SourceFile>,
        compiler_options: &core::compiler_options::CompilerOptions,
        data: &mut CompletionDataData,
        position: usize,
        optional_replacement_span: Option<&lsproto::Range>,
        include_symbols: bool,
    ) -> Option<Result<CompletionList, String>> {
        let keyword_filters = data.keyword_filters;
        let is_new_identifier_location = data.is_new_identifier_location;
        let context_token = data.context_token.clone();
        let mut literals: Vec<super::m5q_3::LiteralValue> = data
            .literals
            .iter()
            .map(|literal| match literal {
                super::m5q_3::LiteralValue::String(s) => super::m5q_3::LiteralValue::String(s.clone()),
                super::m5q_3::LiteralValue::Number(n) => super::m5q_3::LiteralValue::Number(*n),
                super::m5q_3::LiteralValue::PseudoBigInt(b) => {
                    super::m5q_3::LiteralValue::PseudoBigInt(b.clone())
                }
            })
            .collect();
        let preferences = self.user_preferences().clone();

        // Verify if the file is JSX language variant
        if file.language_variant == ast::node_source_file::LanguageVariant::Jsx {
            if let Some(list) = self.get_jsx_closing_tag_completion(data.location.as_ref(), file, position) {
                return Some(Ok(list));
            }
        }

        // When the completion is for the expression of a case clause (e.g. `case |`),
        // filter literals & enum symbols whose values are already present in existing case clauses.
        let case_clause = context_token
            .as_ref()
            .and_then(|token| ast::find_ancestor(token, ast::is_case_clause));
        if let Some(case_clause) = &case_clause {
            let context_token = context_token.as_ref().unwrap();
            if context_token.kind == SyntaxKind::CaseKeyword
                || ast::is_node_descendant_of(
                    context_token,
                    &super::m5x_3::case_clause_expression(case_clause)?,
                )
            {
                let case_block = case_clause.parent()?;
                let NodeData::CaseBlock(case_block_data) = &case_block.data else {
                    return None;
                };
                let tracker = super::m5q2b_6::new_case_clause_tracker(type_checker, &case_block_data.clauses.nodes);
                literals.retain(|literal| !case_clause_tracker_has_literal(&tracker, literal));
                data.symbols.retain(|symbol| {
                    if let Some(value_declaration) = &symbol.value_declaration {
                        if ast::is_enum_member(value_declaration) {
                            if let Some(value) = type_checker.get_constant_value(value_declaration) {
                                if tracker.has_value(&tsox_checker::checker::mig::m2a::r18k8_flags::ConstantValue::String(
                                    value,
                                )) {
                                    return false;
                                }
                            }
                        }
                    }
                    true
                });
            }
        }

        let is_checked = is_checked_file(file, compiler_options);
        if is_checked
            && !is_new_identifier_location
            && data.symbols.is_empty()
            && keyword_filters == KeywordCompletionFilters::None
        {
            return None;
        }

        let (mut unique_names, mut sorted_entries) = self.get_completion_entries_from_symbols(
            type_checker,
            data,
            None,
            position,
            file,
            compiler_options,
            include_symbols,
        )?;

        if data.keyword_filters != KeywordCompletionFilters::None {
            let keyword_completions = get_keyword_completions(
                data.keyword_filters,
                !data.inside_jsdoc_tag_type_expression && ast::is_source_file_js(file),
            );
            for keyword_entry in keyword_completions {
                let label = keyword_entry.label;
                if data.is_type_only_location
                    && super::m5x_3::is_type_keyword(scanner::string_to_token(&label).unwrap_or(SyntaxKind::Unknown))
                    || !data.is_type_only_location
                        && super::m5r::is_contextual_keyword_in_auto_importable_expression_space(&label)
                    || !unique_names.contains(&label)
                {
                    unique_names.insert(label.clone());
                    sorted_entries.push(keyword_completion_item(
                        label,
                        keyword_item_kind(keyword_entry.kind.map(|k| k as i32)),
                        keyword_entry.sort_text,
                        keyword_entry.insert_text,
                        keyword_entry.filter_text,
                        keyword_entry.commit_characters,
                    ));
                }
            }
        }

        for keyword_entry in get_contextual_keywords(file, context_token.as_ref(), position) {
            let label = keyword_entry.label;
            if !unique_names.contains(&label) {
                unique_names.insert(label.clone());
                sorted_entries.push(keyword_completion_item(
                    label,
                    keyword_item_kind(keyword_entry.kind.map(|k| k as i32)),
                    keyword_entry.sort_text,
                    keyword_entry.insert_text,
                    keyword_entry.filter_text,
                    keyword_entry.commit_characters,
                ));
            }
        }

        for literal in &literals {
            let mut literal_entry = lsproto::CompletionItem::default();
            literal_entry.label = super::m5q_3::completion_name_for_literal(file, &preferences, literal);
            literal_entry.sort_text = Some(SORT_TEXT_LOCATION_PRIORITY.to_string());
            literal_entry.commit_characters = Some(Vec::new());
            unique_names.insert(literal_entry.label.clone());
            sorted_entries.push(literal_entry);
        }

        if !is_checked {
            append_js_completion_entries(file, position, &mut unique_names, &mut sorted_entries);
        }

        if let Some(context_token) = &context_token {
            if !data.is_right_of_open_tag && !data.is_right_of_dot_or_question_dot {
                if let Some(case_block) = ast::find_ancestor_kind(context_token, SyntaxKind::CaseBlock) {
                    let cases_item = self.get_exhaustive_case_snippets(
                        &case_block,
                        file,
                        position,
                        compiler_options,
                        &self.get_program(),
                        type_checker,
                    )?;
                    if let Some(cases_item) = cases_item {
                        sorted_entries.push(cases_item);
                    }
                }
            }
        }

        let item_defaults = self.set_item_defaults(
            position,
            file,
            &mut sorted_entries,
            Some(&data.default_commit_characters),
            optional_replacement_span,
        );

        return Some(Ok(CompletionList {
            is_incomplete: data.has_unresolved_auto_imports,
            item_defaults,
            items: sorted_entries,
        }));
    }

    pub fn get_completion_entries_from_symbols(
        &self,
        type_checker: &mut Checker,
        data: &mut CompletionDataData,
        replacement_token: Option<&Arc<Node>>,
        position: usize,
        file: &Arc<SourceFile>,
        compiler_options: &core::compiler_options::CompilerOptions,
        include_symbols: bool,
    ) -> Option<(HashSet<String>, Vec<lsproto::CompletionItem>)> {
        let closest_symbol_declaration = data.location.as_ref()
            .and_then(|location| super::m5q_3::get_closest_symbol_declaration(data.context_token.as_ref(), location));
        let use_semicolons = crate::ls::lsutil_utilities::probably_uses_semicolons(file);
        let preferences = self.user_preferences().clone();
        let legacy_completion_kind = match data.completion_kind {
            CompletionKind::ObjectPropertyDeclaration => crate::ls::completions::CompletionKind::ObjectLiteralMember,
            CompletionKind::MemberLike => crate::ls::completions::CompletionKind::Member,
            CompletionKind::String => crate::ls::completions::CompletionKind::String,
            CompletionKind::None | CompletionKind::Global | CompletionKind::PropertyAccess => {
                crate::ls::completions::CompletionKind::None
            }
        };
        let is_member_completion = super::m5r::is_member_completion_kind(legacy_completion_kind);
        let legacy_data = crate::ls::completions::CompletionDataData {
            symbols: data.symbols.clone(),
            completion_kind: legacy_completion_kind,
            is_in_snippet_scope: data.is_in_snippet_scope,
        };
        let mut sorted_entries: Vec<lsproto::CompletionItem> = Vec::with_capacity(data.symbols.len() + data.auto_imports.len());
        // Tracks unique names. Value is false for globals/module exports (multiple allowed), true otherwise.
        let mut uniques: HashMap<String, bool> = HashMap::new();
        for (index, symbol) in data.symbols.iter().enumerate() {
            let origin = data.symbol_to_origin_info_map.get(&index);
            let (name, needs_convert_property_access) = get_completion_entry_display_name_for_symbol(
                symbol,
                origin,
                data.completion_kind,
                data.is_jsx_identifier_expected,
            );
            if name.is_empty()
                || (uniques.get(&name).copied().unwrap_or(false)
                    && (origin.is_none() || !super::m5r::origin_is_object_literal_method(origin)))
                || data.completion_kind == CompletionKind::Global
                    && !super::m5r_2::should_include_symbol(
                        symbol,
                        data,
                        data.location.as_ref(),
                        closest_symbol_declaration.as_ref(),
                        data.context_token.as_ref(),
                        file,
                        type_checker,
                        compiler_options,
                    )
            {
                continue;
            }

            // When in a value location in a JS file, ignore symbols that definitely seem to be type-only.
            if !data.is_type_only_location && ast::is_source_file_js(file) && super::m5r_2::symbol_appears_to_be_type_only(symbol, type_checker) {
                continue;
            }

            let original_sort_text = data
                .symbol_to_sort_text_map
                .get(&symbol.id())
                .cloned()
                .unwrap_or_else(|| SORT_TEXT_LOCATION_PRIORITY.to_string());

            let sort_text = if is_deprecated(symbol, type_checker) {
                deprecate_sort_text(&original_sort_text)
            } else {
                original_sort_text
            };
            let entry = self.create_completion_item(
                type_checker,
                symbol,
                sort_text,
                replacement_token,
                data,
                position,
                file,
                name.clone(),
                needs_convert_property_access,
                origin,
                use_semicolons,
                compiler_options,
                is_member_completion,
            )?;
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            // True for locals; false for globals, module exports from other files, `this.` completions.
            let should_shadow_later_symbols = (origin.is_none() || super::m5r::origin_is_type_only_alias(origin))
                && !(symbol.parent().is_none()
                    && !core::core::some(&symbol.declarations, |d| {
                        ast::get_source_file_of_node(d).is_some_and(|sf| Arc::ptr_eq(&sf, &file.node))
                    }));
            uniques.insert(name, should_shadow_later_symbols);
            sorted_entries.push(entry);
        }

        for auto_import in &data.auto_imports {
            let mut replacement_span: Option<lsproto::Range> = None;
            let mut insert_text = String::new();
            let mut filter_text = String::new();
            let mut is_snippet = false;
            let mut sort_text = SORT_TEXT_AUTO_IMPORT_SUGGESTIONS.to_string();

            if let Some(import_statement_completion) = &data.import_statement_completion {
                is_snippet = super::m5q_3::client_supports_item_snippet(&client_capabilities_context());
                let (text, span) = super::m5q2b_2::get_insert_text_and_replacement_span_for_import_completion(
                    &auto_import.fix,
                    crate::mig::m5o2::get_import_kind_for_import_statement(file, &auto_import.export_, &self.get_program()),
                    import_statement_completion,
                    use_semicolons,
                    file,
                    &preferences,
                    is_snippet,
                );
                insert_text = text;
                replacement_span = span;
                filter_text = auto_import.fix.auto_import_fix.name.clone();
                sort_text = SORT_TEXT_LOCATION_PRIORITY.to_string();
            }

            // Non-contextual keywords cannot be used as identifiers, so auto-imports with these names
            // should not shadow keyword completions.
            if let Some(token) = scanner::string_to_token(&auto_import.fix.auto_import_fix.name) {
                if token != SyntaxKind::Unknown && crate::ls::lsutil_utilities::is_non_contextual_keyword(Some(token)) {
                    continue;
                }
            }

            if !auto_import.export_.is_unresolved_alias() {
                if data.is_type_only_location {
                    if auto_import.export_.flags & ast::SymbolFlags::TYPE == ast::SymbolFlags::None
                        && auto_import.export_.flags & ast::SymbolFlags::MODULE == ast::SymbolFlags::None
                    {
                        continue;
                    }
                } else if data.import_statement_completion.is_none()
                    && auto_import.export_.flags & ast::SymbolFlags::VALUE == ast::SymbolFlags::None
                {
                    continue;
                }
            }

            let auto_import_entry = lsproto::AutoImportFix {
                kind: auto_import.fix.auto_import_fix.kind as i32,
                name: auto_import.fix.auto_import_fix.name.clone(),
                import_kind: auto_import.fix.auto_import_fix.import_kind as i32,
                use_require: auto_import.fix.auto_import_fix.use_require,
                add_as_type_only: auto_import.fix.auto_import_fix.add_as_type_only as i32,
                module_specifier: auto_import.fix.auto_import_fix.module_specifier.clone(),
                import_index: auto_import.fix.auto_import_fix.import_index,
                usage_position: auto_import.fix.auto_import_fix.usage_position.clone(),
                namespace_prefix: auto_import.fix.auto_import_fix.namespace_prefix.clone(),
            };
            let mut entry = self.create_lsp_completion_item(
                &auto_import.fix.auto_import_fix.name,
                &insert_text,
                &filter_text,
                sort_text,
                auto_import.export_.script_element_kind.clone(),
                auto_import.export_.script_element_kind_modifiers.strings().iter().cloned().collect(),
                replacement_span.clone(),
                None,
                Some(lsproto::CompletionItemLabelDetails {
                    detail: None,
                    description: Some(auto_import.fix.auto_import_fix.module_specifier.clone()),
                }),
                file,
                position,
                false,
                is_snippet,
                data.import_statement_completion.is_none(),
                false,
                &auto_import.fix.auto_import_fix.module_specifier,
                Some(&auto_import_entry),
                None,
                None,
            );
            entry.data.as_mut().unwrap().is_import_statement_completion = data.import_statement_completion.is_none();

            if !uniques.get(&auto_import.fix.auto_import_fix.name).copied().unwrap_or(false) {
                uniques.insert(auto_import.fix.auto_import_fix.name.clone(), false);
                sorted_entries.push(entry);
            }
        }

        Some((uniques.into_keys().collect(), sorted_entries))
    }

    pub fn create_completion_item(
        &self,
        type_checker: &mut Checker,
        symbol: &Arc<Symbol>,
        mut sort_text: SortText,
        replacement_token: Option<&Arc<Node>>,
        data: &CompletionDataData,
        position: usize,
        file: &Arc<SourceFile>,
        mut name: String,
        needs_convert_property_access: bool,
        origin: Option<&SymbolOriginInfo>,
        use_semicolons: bool,
        compiler_options: &core::compiler_options::CompilerOptions,
        is_member_completion: bool,
    ) -> Option<Result<lsproto::CompletionItem, String>> {
        let context_token = data.context_token.clone();
        let mut insert_text = String::new();
        let mut filter_text = String::new();
        let mut replacement_span = self.get_replacement_range_for_context_token(file, replacement_token, position);
        let mut is_snippet = false;
        let mut has_action = false;
        let mut source = get_source_from_origin(origin);
        let mut label_details: Option<lsproto::CompletionItemLabelDetails> = None;
        let preferences = self.user_preferences().clone();
        let insert_question_dot = super::m5r::origin_is_nullable_member(origin);
        let use_braces = super::m5r::origin_is_symbol_member(origin) || needs_convert_property_access;
        if super::m5r::origin_is_this_type_node(origin) {
            if needs_convert_property_access {
                insert_text = format!(
                    "this{}[{}]",
                    if insert_question_dot { "?." } else { "" },
                    quote_property_name(file, &preferences, &name)
                );
            } else {
                insert_text = format!(
                    "this{}{}",
                    if insert_question_dot { "?." } else { "." },
                    name
                );
            }
        } else if let Some(property_access_to_convert) = &data.property_access_to_convert {
            if use_braces || insert_question_dot {
                // We should only have needsConvertPropertyAccess if there's a property access to convert. But see microsoft/TypeScript#21790.
                if use_braces {
                    if needs_convert_property_access {
                        insert_text = format!("[{}]", quote_property_name(file, &preferences, &name));
                    } else {
                        insert_text = format!("[{}]", name);
                    }
                } else {
                    insert_text = name.clone();
                }

                if insert_question_dot || property_access_to_convert.question_dot_token().is_some() {
                    insert_text = format!("?.{}", insert_text);
                }

                let mut dot = astnav::find_child_of_kind(property_access_to_convert, SyntaxKind::DotToken);
                if dot.is_none() {
                    dot = astnav::find_child_of_kind(property_access_to_convert, SyntaxKind::QuestionDotToken);
                }

                let dot = dot?;

                // If the text after the '.' starts with this name, write over it. Else, add new text.
                let end = if property_access_to_convert
                    .name()
                    .is_some_and(|n| name.starts_with(n.text()))
                {
                    property_access_to_convert.end()
                } else {
                    dot.end()
                };
                let (lsp_range, fidelity) = self.m5x_create_lsp_range_from_bounds(
                    tsox_frontend::scanner::mig::x5a::get_token_pos_of_node(&dot, file, false),
                    end,
                    file,
                );
                if fidelity != crate::ls::mig::m5v_7::SPANMAP_FIDELITY_EXACT {
                    return Some(Ok(self.empty_completion_item()));
                }
                replacement_span = Some(lsp_range);
            }
        }

        if data.jsx_initializer.is_initializer {
            if insert_text.is_empty() {
                insert_text = name.clone();
            }
            insert_text = format!("{{{}}}", insert_text);
            if let Some(initializer) = &data.jsx_initializer.initializer {
                let (lsp_range, fidelity) = self.create_lsp_range_from_node(initializer, file);
                if fidelity != crate::ls::mig::m5v_7::SPANMAP_FIDELITY_EXACT {
                    return Some(Ok(self.empty_completion_item()));
                }
                replacement_span = Some(lsp_range);
            }
        }

        if super::m5r::origin_is_promise(origin) && data.property_access_to_convert.is_some() {
            let property_access_to_convert = data.property_access_to_convert.as_ref().unwrap();
            if insert_text.is_empty() {
                insert_text = name.clone();
            }
            let preceding_token = astnav::find_preceding_token(&file.node, property_access_to_convert.pos());
            let mut await_text = String::new();
            if let Some(preceding_token) = &preceding_token {
                if crate::ls::lsutil_asi::position_is_asi_candidate(
                    preceding_token.end(),
                    preceding_token.parent().as_ref(),
                    file,
                ) {
                    await_text = ";".to_string();
                }
            }

            if let Some(expression) = property_access_to_convert.expression() {
                await_text += &format!("(await {})", scanner::mig::m3i::get_text_of_node(expression));
                let parent_node = property_access_to_convert.parent();
                let is_in_await_expression =
                    parent_node.as_ref().map_or(false, |p| ast::is_await_expression(p));
                let wrap_node = if is_in_await_expression {
                    parent_node?
                } else {
                    Arc::clone(expression)
                };
                let (lsp_range, fidelity) = self.m5x_create_lsp_range_from_bounds(
                    tsox_frontend::scanner::mig::x5a::get_token_pos_of_node(&wrap_node, file, false),
                    property_access_to_convert.end(),
                    file,
                );
                if fidelity != crate::ls::mig::m5v_7::SPANMAP_FIDELITY_EXACT {
                    return Some(Ok(self.empty_completion_item()));
                }
                replacement_span = Some(lsp_range);
                if needs_convert_property_access {
                    insert_text = format!("{}{}", await_text, insert_text);
                } else {
                    let dot_str = if insert_question_dot { "?." } else { "." };
                    insert_text = format!("{}{}{}", await_text, dot_str, insert_text);
                }
            }
        }

        if super::m5r::origin_is_type_only_alias(origin) {
            has_action = true;
        }

        // Provide object member completions when missing commas, and insert missing commas.
        if data.completion_kind == CompletionKind::ObjectPropertyDeclaration {
            if let Some(context_token) = &context_token {
                let preceding =
                    astnav::find_preceding_token(&file.node, context_token.pos());
                let preceding_is_comma = preceding.map(|p| p.kind == SyntaxKind::CommaToken).unwrap_or(false);
                if !preceding_is_comma {
                    let context_parent = context_token.parent();
                    let context_grandparent = context_parent.as_ref().and_then(|p| p.parent());
                    if context_grandparent.as_ref().map_or(false, |g| {
                        ast::is_method_declaration(g)
                            || ast::is_get_accessor_declaration(g)
                            || ast::is_set_accessor_declaration(g)
                    })
                        || context_parent.as_ref().map_or(false, |p| ast::is_spread_assignment(p))
                        || crate::ls::lsutil_children::get_last_token(
                            context_parent
                                .as_ref()
                                .and_then(|p| ast::find_ancestor(p, ast::is_property_assignment))
                                .as_ref(),
                            file,
                        )
                        .map(|t| t.pos() == context_token.pos() && t.end() == context_token.end())
                        .unwrap_or(false)
                        || context_parent.as_ref().map_or(false, |p| ast::is_shorthand_property_assignment(p))
                            && get_line_of_position(file, context_token.end()) != get_line_of_position(file, position)
                    {
                        source = COMPLETION_SOURCE_OBJECT_LITERAL_MEMBER_WITH_COMMA.to_string();
                        has_action = true;
                    }
                }
            }
        }

        let mut additional_text_edits: Option<Vec<lsproto::TextEdit>> = None;
        if preferences
            .include_completions_with_class_member_snippets
            .is_true()
            && data.completion_kind == CompletionKind::MemberLike
            && data
                .location
                .as_ref()
                .is_some_and(|location| is_class_like_member_completion(symbol, location, file))
        {
            let location = data.location.as_ref().unwrap();
            let member_completion_entry = self.get_entry_for_member_completion(
                type_checker,
                symbol,
                &name,
                location,
                position,
                context_token.as_ref(),
                file,
            )?;
            let member_completion_entry = match member_completion_entry {
                Some(entry) => entry,
                None => return Some(Ok(self.empty_completion_item())),
            };
            insert_text = member_completion_entry.insert_text;
            filter_text = member_completion_entry.filter_text;
            is_snippet = member_completion_entry.is_snippet;
            if !member_completion_entry.additional_text_edits.is_empty() {
                additional_text_edits = Some(member_completion_entry.additional_text_edits);
                has_action = true;
                source = COMPLETION_SOURCE_CLASS_MEMBER_SNIPPET.to_string();
            }
        }

        if super::m5r::origin_is_object_literal_method(origin) {
            let object_literal_method = origin.unwrap().as_object_literal_method();
            insert_text = object_literal_method.insert_text.clone();
            is_snippet = object_literal_method.is_snippet;
            label_details = object_literal_method.label_details.clone();
            if !super::m5q_3::client_supports_item_label_details(&client_capabilities_context()) {
                name = format!(
                    "{}{}",
                    name,
                    object_literal_method.label_details.as_ref().map(|d| d.detail.clone().unwrap_or_default()).unwrap_or_default()
                );
                label_details = None;
            }
            source = COMPLETION_SOURCE_OBJECT_LITERAL_METHOD_SNIPPET.to_string();
            sort_text = sort_below(&sort_text);
        }

        if data.is_jsx_identifier_expected
            && !data.is_right_of_open_tag
            && super::m5q_3::client_supports_item_snippet(&client_capabilities_context())
            && preferences.jsx_attribute_completion_style != crate::ls::lsutil_user_preferences::JsxAttributeCompletionStyle::None
            && !(data
                .location
                .as_ref()
                .map(|l| {
                    l.parent()
                        .is_some_and(|p| ast::is_jsx_attribute(&p) && p.initializer().is_some())
                })
                .unwrap_or(false))
        {
            let mut use_braces = preferences.jsx_attribute_completion_style
                == crate::ls::lsutil_user_preferences::JsxAttributeCompletionStyle::Braces;
            let location = data.location.as_ref().unwrap();
            let t = type_checker.get_type_of_symbol_at_location(symbol, location);

            // If is boolean like or undefined, don't return a snippet, we want to return just the completion.
            if preferences.jsx_attribute_completion_style == crate::ls::lsutil_user_preferences::JsxAttributeCompletionStyle::Auto
                && !t.is_boolean_like()
                && !(t.is_union()
                    && core::core::some(t.types().unwrap_or(&[]), |t| t.is_boolean_like()))
            {
                use tsox_checker::checker::types::{TypeFlags, TYPE_FLAGS_STRING_LIKE};
                if t.is_string_like()
                    || t.is_union()
                        && core::core::every(t.types().unwrap_or(&[]), |t| {
                            t.flags.contains(TYPE_FLAGS_STRING_LIKE | TypeFlags::Undefined)
                                || is_string_and_empty_anonymous_object_intersection(type_checker, t)
                        })
                {
                    // If type is string-like or undefined, use quotes.
                    insert_text = format!(
                        "{}={}",
                        escape_snippet_text(&name),
                        super::m5x_3::quote(file, &preferences, "$1")
                    );
                    is_snippet = true;
                } else {
                    // Use braces for everything else.
                    use_braces = true;
                }
            }

            if use_braces {
                insert_text = format!("{}={{$1}}", escape_snippet_text(&name));
                is_snippet = true;
            }
        }

        let parent_named_import_or_export = data
            .location
            .as_ref()
            .and_then(|location| ast::find_ancestor(location, is_named_imports_or_exports_fn));
        if let Some(parent_named_import_or_export) = parent_named_import_or_export {
            if !scanner::mig::m3i::is_identifier_text(&name, ast::node_source_file::LanguageVariant::Standard) {
                insert_text = quote_property_name(file, &preferences, &name);

                if parent_named_import_or_export.kind == SyntaxKind::NamedImports {
                    // Check if it is `import { ^here as name } from '...'`.
                    let mut scanner_state =
                        scanner::Scanner::new(file.text[position.min(file.text.len())..].to_string());
                    if !(scanner_state.scan() == SyntaxKind::AsKeyword && scanner_state.scan() == SyntaxKind::Identifier) {
                        insert_text += &format!(" as {}", generate_identifier_for_arbitrary_string(&name));
                    }
                }
            } else if parent_named_import_or_export.kind == SyntaxKind::NamedImports {
                let possible_token = scanner::string_to_token(&name);
                if let Some(possible_token) = possible_token {
                    if possible_token == SyntaxKind::AwaitKeyword
                        || crate::ls::lsutil_utilities::is_non_contextual_keyword(Some(possible_token))
                    {
                        insert_text = format!("{} as {}_", name, name);
                    }
                }
            }
        }

        // Commit characters
        let location = data.location.as_ref().unwrap();
        let element_kind = crate::ls::lsutil_symbol_display::get_symbol_kind(Some(&*type_checker), symbol, location);
        let mut commit_characters: Option<Vec<String>> = None;
        if super::m5q_3::client_supports_item_commit_characters(&client_capabilities_context()) {
            if element_kind == crate::ls::lsutil_symbol_display::ScriptElementKind::Warning
                || element_kind == crate::ls::lsutil_symbol_display::ScriptElementKind::String
            {
                commit_characters = Some(Vec::new());
            } else if !super::m5q_3::client_supports_default_commit_characters(&client_capabilities_context()) {
                commit_characters = Some(data.default_commit_characters.clone());
            }
            // Otherwise use the completion list default.
        }

        let preselect = data
            .recommended_completion
            .as_ref()
            .map_or(false, |rc| is_recommended_completion_match(symbol, rc, type_checker));
        let kind_modifiers =
            crate::ls::lsutil_symbol_display::get_symbol_modifiers(Some(&*type_checker), Some(symbol))
                .strings()
                .iter()
                .cloned()
                .collect::<Vec<String>>();

        Some(Ok(self.create_lsp_completion_item(
            &name,
            &insert_text,
            &filter_text,
            sort_text,
            element_kind,
            kind_modifiers,
            replacement_span,
            commit_characters,
            label_details,
            file,
            position,
            is_member_completion,
            is_snippet,
            has_action,
            preselect,
            &source,
            None,
            additional_text_edits.as_ref(),
            None,
        )))
    }

    fn empty_completion_item(&self) -> lsproto::CompletionItem {
        lsproto::CompletionItem::default()
    }

    fn get_jsx_closing_tag_completion(
        &self,
        location: Option<&Arc<Node>>,
        file: &Arc<SourceFile>,
        position: usize,
    ) -> Option<CompletionList> {
        use super::m5q_3::get_default_commit_characters;
        use tsox_frontend::ast::mig::m3e_4::{find_ancestor_or_quit, FindAncestorResult};

        // We wanna walk up the tree till we find a JSX closing element.
        let jsxClosingElement = find_ancestor_or_quit(location, |node| match node.kind {
            SyntaxKind::JsxClosingElement => FindAncestorResult::True,
            SyntaxKind::LessThanSlashToken
            | SyntaxKind::GreaterThanToken
            | SyntaxKind::Identifier
            | SyntaxKind::PropertyAccessExpression => FindAncestorResult::False,
            _ => FindAncestorResult::Quit,
        })?;

        // In the TypeScript JSX element, if such element is not defined. When users query for completion at closing tag,
        // instead of simply giving unknown value, the completion will return the tag-name of an associated opening-element.
        let has_closing_angle_bracket =
            astnav::find_child_of_kind(&jsxClosingElement, SyntaxKind::GreaterThanToken).is_some();
        let NodeData::JsxElement(jsx_element) = &jsxClosingElement.parent()?.data else {
            return None;
        };
        let NodeData::JsxOpeningElement(opening_element) = &jsx_element.opening_element.data else {
            return None;
        };
        let closing_tag = scanner::mig::m3i::get_text_of_node(&opening_element.tag_name);
        let full_closing_tag = format!(
            "{}{}",
            closing_tag,
            if has_closing_angle_bracket { "" } else { ">" }
        );
        let tag_name = jsxClosingElement.name()?;
        let (optional_replacement_span, fidelity) = self.create_lsp_range_from_node(&tag_name, file);
        if fidelity != crate::ls::mig::m5v_7::SPANMAP_FIDELITY_EXACT {
            return None;
        }
        let default_commit_characters = get_default_commit_characters(false);

        let mut items = vec![self.create_lsp_completion_item(
            &full_closing_tag,
            "",
            "",
            SORT_TEXT_LOCATION_PRIORITY.to_string(),
            crate::ls::lsutil_symbol_display::ScriptElementKind::ClassElement,
            Vec::new(),
            None,
            None,
            None,
            file,
            position,
            true,
            false,
            false,
            false,
            "",
            None,
            None,
            None,
        )];
        let item_defaults = self.set_item_defaults(
            position,
            file,
            &mut items,
            Some(&default_commit_characters),
            Some(&optional_replacement_span),
        );

        Some(CompletionList {
            is_incomplete: false,
            item_defaults,
            items,
        })
    }

    pub(crate) fn set_item_defaults(
        &self,
        position: usize,
        file: &Arc<SourceFile>,
        items: &mut [lsproto::CompletionItem],
        default_commit_characters: Option<&Vec<String>>,
        optional_replacement_span: Option<&lsproto::Range>,
    ) -> Option<CompletionItemDefaults> {
        let mut item_defaults: Option<CompletionItemDefaults> = None;
        if let Some(default_commit_characters) = default_commit_characters {
            let supports_item_commit_characters =
                super::m5q_3::client_supports_item_commit_characters(&client_capabilities_context());
            if super::m5q_3::client_supports_default_commit_characters(&client_capabilities_context())
                && supports_item_commit_characters
            {
                item_defaults = Some(CompletionItemDefaults {
                    commit_characters: Some(default_commit_characters.clone()),
                    edit_range: None,
                });
            } else if supports_item_commit_characters {
                for item in items.iter_mut() {
                    if item.commit_characters.is_none() {
                        item.commit_characters = Some(default_commit_characters.clone());
                    }
                }
            }
        }
        if let Some(optional_replacement_span) = optional_replacement_span {
            // Ported from vscode ts extension.
            let (end, fidelity) = self.create_lsp_position_m5x(position, file);
            if fidelity != crate::ls::mig::m5v_7::SPANMAP_FIDELITY_EXACT {
                return item_defaults;
            }
            let insert_range = lsproto::Range {
                start: optional_replacement_span.start.clone(),
                end,
            };
            if super::m5q_3::client_supports_default_edit_range(&client_capabilities_context()) {
                let defaults = item_defaults.get_or_insert_with(CompletionItemDefaults::default);
                defaults.edit_range = Some(EditRangeWithInsertReplace {
                    insert: insert_range.clone(),
                    replace: optional_replacement_span.clone(),
                });
                for item in items.iter_mut() {
                    // If `editRange` is set, `insertText` is ignored by the client, so we need to
                    // provide `textEdit` instead.
                    if item.insert_text.is_some() && item.text_edit.is_none() {
                        item.text_edit = Some(lsproto::TextEditOrInsertReplaceEdit {
                            text_edit: None,
                            insert_replace_edit: Some(lsproto::InsertReplaceEdit {
                                new_text: item.insert_text.clone().unwrap_or_default(),
                                insert: insert_range.clone(),
                                replace: optional_replacement_span.clone(),
                            }),
                        });
                        item.insert_text = None;
                    }
                }
            } else if super::m5q_3::client_supports_item_insert_replace(&client_capabilities_context()) {
                for item in items.iter_mut() {
                    if item.text_edit.is_none() {
                        item.text_edit = Some(lsproto::TextEditOrInsertReplaceEdit {
                            text_edit: None,
                            insert_replace_edit: Some(lsproto::InsertReplaceEdit {
                                new_text: item
                                    .insert_text
                                    .clone()
                                    .unwrap_or_else(|| item.label.clone()),
                                insert: insert_range.clone(),
                                replace: optional_replacement_span.clone(),
                            }),
                        });
                    }
                }
            }
        }

        item_defaults
    }

    pub fn get_entry_for_object_literal_method_completion(
        &self,
        type_checker: &mut Checker,
        symbol: &Arc<Symbol>,
        enclosing_declaration: &Arc<Node>,
        file: &Arc<SourceFile>,
    ) -> Option<super::m5r::SymbolOriginInfoObjectLiteralMethod> {
        let mut snippet_printer = create_snippet_printer(
            tsox_frontend::format::mig::m4o_2::PrinterOptions {
                remove_comments: true,
                new_line: core::mig::m3j_3::get_new_line_kind(&self.format_options().new_line_character),
                target: self.get_program().options().get_emit_script_target(),
                omit_trailing_semicolon: false,
                no_emit_helpers: false,
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

        let is_snippet = super::m5q_3::client_supports_item_snippet(&client_capabilities_context());
        let method = self.create_object_literal_method(
            &mut snippet_printer,
            type_checker,
            symbol,
            enclosing_declaration,
            file,
            is_snippet,
        )??;

        let mut insert_text = snippet_printer.print_unescaped_node(&method);
        insert_text += ",";

        Some(super::m5r::SymbolOriginInfoObjectLiteralMethod {
            insert_text,
            label_details: Some(lsproto::CompletionItemLabelDetails {
                detail: Some(self.print_object_literal_method_label_detail(&method, file)),
                description: None,
            }),
            is_snippet,
        })
    }

    fn print_object_literal_method_label_detail(&self, method: &Arc<Node>, file: &Arc<SourceFile>) -> String {
        let NodeData::MethodDeclaration(method_declaration) = &method.data else {
            return String::new();
        };
        let factory = NodeFactory::new();
        let method_signature = Arc::new(Node::new(
            SyntaxKind::MethodSignature,
            NodeData::MethodSignatureDeclaration(
                tsox_frontend::ast::node_data_generated::MethodSignatureDeclarationData {
                    modifiers: None,
                    name: factory.new_identifier(""),
                    postfix_token: method_declaration.postfix_token.clone(),
                    type_parameters: method_declaration.type_parameters.clone(),
                    parameters: Arc::clone(&method_declaration.parameters),
                    type_node: method_declaration.type_node.clone(),
                },
            ),
        ));
        let mut signature_printer = create_snippet_printer(
            tsox_frontend::format::mig::m4o_2::PrinterOptions {
                remove_comments: true,
                omit_trailing_semicolon: true,
                new_line: core::mig::m3j_3::get_new_line_kind(&self.format_options().new_line_character),
                target: self.get_program().options().get_emit_script_target(),
                no_emit_helpers: false,
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
        signature_printer.print_unescaped_node(&method_signature)
    }

    pub fn create_object_literal_method(
        &self,
        snippet_printer: &mut SnippetPrinter,
        type_checker: &mut Checker,
        symbol: &Arc<Symbol>,
        enclosing_declaration: &Arc<Node>,
        file: &Arc<SourceFile>,
        is_snippet: bool,
    ) -> Option<Option<Arc<Node>>> {
        use tsox_checker::checker::symboltracker::NodeBuilderFlags;
        use tsox_checker::checker::types::{SignatureKind, TypeFlags, UnionReduction};

        let factory = NodeFactory::new();

        let declaration = symbol.declarations.first().cloned();
        if !is_object_literal_method_completion_candidate_declaration(declaration.as_ref()) {
            return Some(None);
        }

        let symbol_type =
            type_checker.get_type_of_symbol_at_location(symbol, enclosing_declaration);
        let mut effective_type = type_checker.get_widened_type(&symbol_type);
        if effective_type.flags & TypeFlags::Union != TypeFlags::None
            && effective_type.types().map_or(0, |ts| ts.len()) < 10
        {
            effective_type = type_checker.get_union_type_ex(
                effective_type.types().unwrap_or(&[]).to_vec(),
                UnionReduction::Subtype,
            );
        }
        if effective_type.flags & TypeFlags::Union != TypeFlags::None {
            let mut function_type: Option<Arc<tsox_checker::checker::Type>> = None;
            for union_type in effective_type.types().into_iter().flatten() {
                if type_checker
                    .get_signatures_of_type(union_type, SignatureKind::Call)
                    .is_empty()
                {
                    continue;
                }
                if function_type.is_some() {
                    return Some(None);
                }
                function_type = Some(Arc::clone(union_type));
            }
            let function_type = function_type?;
            effective_type = function_type;
        }

        let signatures = type_checker.get_signatures_of_type(&effective_type, SignatureKind::Call);
        if signatures.len() != 1 {
            return Some(None);
        }

        let mut flags = NodeBuilderFlags::OmitThisParameter;
        if crate::ls::lsutil_utilities::get_quote_preference(file, self.user_preferences()) == QuotePreference::Single {
            flags |= NodeBuilderFlags::UseSingleQuotesForStringLiteralType;
        }
        let type_node = type_checker.type_to_type_node_ex(
            &effective_type,
            Some(enclosing_declaration),
            flags,
            Default::default(),
            Default::default(),
        );
        if type_node.kind != SyntaxKind::FunctionType {
            return Some(None);
        }

        let NodeData::FunctionTypeNode(function_type_node) = &type_node.data else {
            return Some(None);
        };

        let mut parameters: Vec<Arc<Node>> = Vec::new();
        for parameter in &function_type_node.parameters.nodes {
            let NodeData::ParameterDeclaration(parameter_data) = &parameter.data else {
                continue;
            };
            let Some(parameter_name) = parameter.name() else {
                continue;
            };
            parameters.push(Arc::new(Node::new(
                SyntaxKind::Parameter,
                NodeData::ParameterDeclaration(
                    tsox_frontend::ast::node_data_generated::ParameterDeclarationData {
                        modifiers: None,
                        dot_dot_dot_token: parameter_data.dot_dot_dot_token.clone(),
                        name: ast::deep_clone_node(parameter_name),
                        question_token: None,
                        type_node: None,
                        initializer: parameter_data.initializer.clone(),
                    },
                ),
            )));
        }

        let mut body = Arc::new(Node::new(
            SyntaxKind::Block,
            NodeData::Block(tsox_frontend::ast::node_data_generated::BlockData {
                statements: Arc::new(factory.new_node_list(Vec::new())),
                multi_line: true,
            }),
        ));
        if is_snippet {
            let mut snippet_emit_context = tsox_emit::printer::EmitContext::new();
            body = super::m5q_3::create_snippet_tab_stop_body(&factory, &mut snippet_emit_context)?;
        }

        let Some(declaration) = declaration else {
            return Some(None);
        };
        let Some(declaration_name) = declaration.name() else {
            return Some(None);
        };

        Some(Some(Arc::new(Node::new(
            SyntaxKind::MethodDeclaration,
            NodeData::MethodDeclaration(
                tsox_frontend::ast::node_data_generated::MethodDeclarationData {
                    modifiers: None,
                    asterisk_token: None,
                    name: ast::deep_clone_node(declaration_name),
                    postfix_token: None,
                    type_parameters: None,
                    parameters: Arc::new(factory.new_node_list(parameters)),
                    type_node: None,
                    full_signature: None,
                    body: Some(body),
                },
            ),
        ))))
    }

    pub fn collect_object_literal_method_symbols(
        &self,
        type_checker: &mut Checker,
        members: &[Arc<Symbol>],
        enclosing_declaration: &Arc<Node>,
        file: &Arc<SourceFile>,
    ) -> Vec<ObjectLiteralMethodSymbol> {
        if ast::is_source_file_js(file) {
            return Vec::new();
        }

        let mut methods = Vec::new();
        for member in members {
            if !is_object_literal_method_symbol(member) {
                continue;
            }
            let (display_name, _) = get_completion_entry_display_name_for_symbol(
                member,
                None,
                CompletionKind::ObjectPropertyDeclaration,
                false,
            );
            if display_name.is_empty() {
                continue;
            }
            let entry =
                self.get_entry_for_object_literal_method_completion(type_checker, member, enclosing_declaration, file);
            let entry = match entry {
                Some(entry) => entry,
                None => continue,
            };
            methods.push(ObjectLiteralMethodSymbol {
                symbol: Arc::clone(member),
                origin: SymbolOriginInfo {
                    kind: SYMBOL_ORIGIN_INFO_KIND_OBJECT_LITERAL_METHOD,
                    is_default_export: false,
                    is_from_package_json: false,
                    file_name: String::new(),
                    data: Some(super::m5r::SymbolOriginInfoData::ObjectLiteralMethod(entry)),
                },
            });
        }
        methods
    }
}

fn case_clause_tracker_has_literal(
    tracker: &super::m5q2b_6::CaseClauseTracker,
    literal: &super::m5q_3::LiteralValue,
) -> bool {
    use tsox_checker::checker::mig::m2a::r18k8_flags::ConstantValue;
    let value = match literal {
        super::m5q_3::LiteralValue::String(s) => ConstantValue::String(s.clone()),
        super::m5q_3::LiteralValue::Number(n) => ConstantValue::Number(*n),
        super::m5q_3::LiteralValue::PseudoBigInt(b) => ConstantValue::BigInt(b.clone()),
    };
    tracker.has_value(&value)
}

fn is_named_imports_or_exports_fn(node: &Node) -> bool {
    ast::is_named_imports(node) || ast::is_named_exports(node)
}

pub fn generate_identifier_for_arbitrary_string(text: &str) -> String {
    let mut needs_underscore = false;
    let mut identifier = String::new();

    // Convert "(example, text)" into "_example_text_"
    for (pos, ch) in text.char_indices() {
        let valid_char = if pos == 0 {
            is_identifier_start_char(ch)
        } else {
            scanner::mig::m4d_2::is_identifier_part_ex(ch, ast::node_source_file::LanguageVariant::Standard)
        };
        if valid_char {
            if needs_underscore {
                identifier.push('_');
            }
            identifier.push(ch);
            needs_underscore = false;
        } else {
            needs_underscore = true;
        }
    }

    if needs_underscore {
        identifier.push('_');
    }

    // Default to "_" if the provided text was empty
    if identifier.is_empty() {
        return "_".to_string();
    }

    identifier
}

fn is_identifier_start_char(c: char) -> bool {
    c.is_ascii_alphabetic()
        || c == '_'
        || c == '$'
        || (!c.is_ascii() && unicode_ident::is_xid_start(c))
}

fn client_capabilities_context() -> crate::mig::m5m::ResolvedClientCapabilitiesContext {
    crate::mig::m5m::ResolvedClientCapabilitiesContext {
        capabilities: None,
    }
}

fn keyword_item_kind(kind: Option<i32>) -> Option<lsproto::CompletionItemKind> {
    kind.map(|k| match k {
        2 => lsproto::CompletionItemKind::Method,
        3 => lsproto::CompletionItemKind::Function,
        4 => lsproto::CompletionItemKind::Constructor,
        5 => lsproto::CompletionItemKind::Field,
        6 => lsproto::CompletionItemKind::Variable,
        7 => lsproto::CompletionItemKind::Class,
        8 => lsproto::CompletionItemKind::Interface,
        9 => lsproto::CompletionItemKind::Module,
        10 => lsproto::CompletionItemKind::Property,
        13 => lsproto::CompletionItemKind::Enum,
        14 => lsproto::CompletionItemKind::Keyword,
        17 => lsproto::CompletionItemKind::File,
        19 => lsproto::CompletionItemKind::Folder,
        20 => lsproto::CompletionItemKind::EnumMember,
        21 => lsproto::CompletionItemKind::Constant,
        _ => lsproto::CompletionItemKind::Text,
    })
}

fn keyword_completion_item(
    label: String,
    kind: Option<lsproto::CompletionItemKind>,
    sort_text: Option<String>,
    insert_text: Option<String>,
    filter_text: Option<String>,
    commit_characters: Option<Vec<String>>,
) -> lsproto::CompletionItem {
    let mut item = lsproto::CompletionItem::default();
    item.label = label;
    item.kind = kind;
    item.sort_text = sort_text;
    item.insert_text = insert_text;
    item.filter_text = filter_text;
    item.commit_characters = commit_characters;
    item
}

fn append_js_completion_entries(
    file: &Arc<SourceFile>,
    position: usize,
    unique_names: &mut HashSet<String>,
    sorted_entries: &mut Vec<lsproto::CompletionItem>,
) {
    let name_table = tsox_frontend::ast::mig::m3b::get_name_table(file);
    for (name, pos) in name_table.iter() {
        if *pos as usize == position {
            continue;
        }
        if !unique_names.contains(name)
            && scanner::mig::m3i::is_identifier_text(name, ast::node_source_file::LanguageVariant::Standard)
        {
            unique_names.insert(name.clone());
            let mut item = lsproto::CompletionItem::default();
            item.label = name.clone();
            item.kind = Some(lsproto::CompletionItemKind::Text);
            item.sort_text = Some(SORT_TEXT_JAVASCRIPT_IDENTIFIERS.to_string());
            item.commit_characters = Some(Vec::new());
            sorted_entries.push(item);
        }
    }
}
