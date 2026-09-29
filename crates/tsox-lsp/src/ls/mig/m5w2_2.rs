#![allow(dead_code, unused_imports, unused_variables)]

//! string_completions.go 移植第二部分:顶层补全入口、类型/签名族、triple-slash 族。
//! 缺失依赖按命名约定调用待接线(walk_up_parentheses、get_argument_info_for_completions 等)。

use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_checker::checker::mig::m2a::r18k8_flags::ConstantValue;
use tsox_compile::compiler::Program;
use tsox_core::core::text::TextRange;
use tsox_core::tspath as tsp;
use tsox_frontend::ast::{Node, SourceFile, Symbol};

use crate::ls::language_service::LanguageService;
use crate::ls::mig::m5q2_2::{CompletionItemDefaults, CompletionList};
use crate::ls::mig::m5q2b_3::lsproto as lsp;
use crate::ls::lsutil_symbol_display::{ScriptElementKind, ScriptElementKindModifier};
use crate::ls::mig::m5w2::{
    get_directory_fragment_range, get_fragment_directory, is_in_reference_comment,
    kind_modifiers_from_extension, m5w2_to_path_completions, modulet_to_script_element_kind,
    ModuleCompletionKind, ModuleCompletionNameAndKind, PathCompletion, PathCompletions,
    ReferenceKind,
};

#[derive(Default)]
pub struct StringLiteralCompletions {
    pub from_types: Option<CompletionsFromTypes>,
    pub from_properties: Option<CompletionsFromProperties>,
    pub from_paths: Option<PathCompletions>,
}

pub struct CompletionsFromTypes {
    pub types: Vec<Arc<tsox_checker::checker::Type>>,
    pub is_new_identifier: bool,
}

pub struct CompletionsFromProperties {
    pub symbols: Vec<Arc<Symbol>>,
    pub has_index_signature: bool,
}

impl LanguageService {
    pub fn m5w2_get_string_literal_completions(
        &self,
        file: &Arc<SourceFile>,
        position: usize,
        context_token: Option<&Arc<Node>>,
        checker: &mut Checker,
        compiler_options: &tsox_core::core::compiler_options::CompilerOptions,
        include_symbols: bool,
    ) -> Option<CompletionList> {
        if is_in_reference_comment(file, position) {
            let completion = self.m5w2_get_triple_slash_reference_completions(
                file,
                position,
                &self.get_program(),
                Some(checker),
            )?;
            return self.m5w2_convert_path_completions(completion, file, position);
        }
        if crate::ls::utilities::is_in_string(file, position, context_token) {
            let context_token = context_token?;
            if !tsox_frontend::ast::is_string_literal_like(context_token) {
                return None;
            }
            let entries = self.m5w2_get_string_literal_completion_entries(
                file,
                context_token,
                position,
                checker,
            )?;
            return self.m5w2_convert_string_literal_completions(
                entries,
                context_token,
                file,
                position,
                checker,
                compiler_options,
                include_symbols,
            );
        }
        None
    }

    pub fn m5w2_convert_string_literal_completions(
        &self,
        completion: StringLiteralCompletions,
        context_token: &Arc<Node>,
        file: &Arc<SourceFile>,
        position: usize,
        type_checker: &mut Checker,
        options: &tsox_core::core::compiler_options::CompilerOptions,
        include_symbols: bool,
    ) -> Option<CompletionList> {
        let optional_replacement_range =
            crate::ls::mig::m5q_3::create_range_from_string_literal_like_content(
                self,
                file,
                context_token,
                position as i32,
            );
        if let Some(from_paths) = completion.from_paths {
            return self.m5w2_convert_path_completions(from_paths, file, position);
        }
        if let Some(from_properties) = completion.from_properties {
            let mut data = crate::ls::mig::m5q2::CompletionDataData {
                symbols: from_properties.symbols,
                completion_kind: crate::ls::mig::m5q2::CompletionKind::String,
                is_new_identifier_location: from_properties.has_index_signature,
                location: Some(Arc::clone(&file.node)),
                context_token: Some(context_token.clone()),
                ..Default::default()
            };
            let (_, mut items) = self.get_completion_entries_from_symbols(
                type_checker,
                &mut data,
                Some(context_token),
                position,
                file,
                options,
                include_symbols,
            )?;
            let default_commit_characters =
                crate::ls::mig::m5q_3::get_default_commit_characters(
                    from_properties.has_index_signature,
                );
            let item_defaults = self.m5w2_set_item_defaults(
                position,
                file,
                &mut items,
                Some(&default_commit_characters),
                optional_replacement_range.as_ref(),
            );
            return Some(CompletionList {
                is_incomplete: false,
                item_defaults,
                items,
            });
        }
        if let Some(from_types) = completion.from_types {
            let quote_char = if context_token.kind == tsox_frontend::ast::SyntaxKind::NoSubstitutionTemplateLiteral {
                tsox_emit::printer::QuoteChar::Backtick
            } else if context_token.text().starts_with('\'') {
                tsox_emit::printer::QuoteChar::SingleQuote
            } else {
                tsox_emit::printer::QuoteChar::DoubleQuote
            };
            let mut items: Vec<lsp::CompletionItem> = from_types
                .types
                .iter()
                .map(|t| {
                    let name = tsox_emit::printer::escape_string(
                        &literal_value_as_string(t),
                        quote_char,
                    );
                    self.create_lsp_completion_item(
                        &name,
                        "",
                        "",
                        crate::ls::mig::m5q_3::SORT_TEXT_LOCATION_PRIORITY.to_string(),
                        ScriptElementKind::String,
                        Vec::new(),
                        self.get_replacement_range_for_context_token(
                            file,
                            Some(context_token),
                            position,
                        ),
                        None,
                        None,
                        file,
                        position,
                        false,
                        false,
                        false,
                        false,
                        "",
                        None,
                        None,
                        None,
                    )
                })
                .collect();
            let default_commit_characters =
                crate::ls::mig::m5q_3::get_default_commit_characters(
                    from_types.is_new_identifier,
                );
            let item_defaults = self.m5w2_set_item_defaults(
                position,
                file,
                &mut items,
                Some(&default_commit_characters),
                None,
            );
            return Some(CompletionList {
                is_incomplete: false,
                item_defaults,
                items,
            });
        }
        None
    }

    pub fn m5w2_convert_path_completions(
        &self,
        completion: PathCompletions,
        file: &Arc<SourceFile>,
        position: usize,
    ) -> Option<CompletionList> {
        let is_new_identifier_location = true;
        let default_commit_characters =
            crate::ls::mig::m5q_3::get_default_commit_characters(is_new_identifier_location);
        let mut items: Vec<lsp::CompletionItem> = completion
            .entries
            .iter()
            .map(|path_completion| {
                let mut detail = path_completion.name.clone();
                if !path_completion.name.ends_with(&path_completion.extension) {
                    detail.push_str(&path_completion.extension);
                }
                self.create_lsp_completion_item(
                    &path_completion.name,
                    "",
                    "",
                    crate::ls::mig::m5q_3::SORT_TEXT_LOCATION_PRIORITY.to_string(),
                    path_completion.kind,
                    kind_modifiers_from_extension(&path_completion.extension)
                        .strings()
                        .iter()
                        .cloned()
                        .collect(),
                    completion.replacement_span.clone(),
                    None,
                    None,
                    file,
                    position,
                    false,
                    false,
                    false,
                    false,
                    "",
                    None,
                    None,
                    Some(detail.as_str()),
                )
            })
            .collect();
        let item_defaults = self.m5w2_set_item_defaults(
            position,
            file,
            &mut items,
            Some(&default_commit_characters),
            None,
        );
        Some(CompletionList {
            is_incomplete: false,
            item_defaults,
            items,
        })
    }

    pub fn m5w2_get_string_literal_completion_entries(
        &self,
        file: &Arc<SourceFile>,
        node: &Arc<Node>,
        position: usize,
        type_checker: &mut Checker,
    ) -> Option<StringLiteralCompletions> {
        let parent = walk_up_parentheses(node.parent()?)?;
        match parent.kind {
            tsox_frontend::ast::SyntaxKind::LiteralType => {
                let grandparent = walk_up_parentheses(parent.parent()?)?;
                if grandparent.kind == tsox_frontend::ast::SyntaxKind::ImportType {
                    return self.m5w2_get_string_literal_completions_from_module_names(
                        file,
                        node,
                        &self.get_program(),
                        type_checker,
                    );
                }
                from_unionable_literal_type(&grandparent, &parent, position, type_checker)
                    .map(|r| r)
            }
            tsox_frontend::ast::SyntaxKind::PropertyAssignment => {
                let parent_parent = parent.parent()?;
                if tsox_frontend::ast::is_object_literal_expression(&parent_parent)
                    && parent.name().is_some_and(|n| Arc::ptr_eq(n, node))
                {
                    return Some(StringLiteralCompletions {
                        from_properties: string_literal_completions_for_object_literal(
                            type_checker,
                            &parent_parent,
                        ),
                        from_types: None,
                        from_paths: None,
                    });
                }
                if tsox_frontend::ast::find_ancestor(&parent_parent, |n| {
                    tsox_frontend::ast::mig::m3f_4::is_call_like_expression(n)
                })
                .is_some()
                {
                    let mut uniques = tsox_core::collections::set::Set::new();
                    let mut string_literal_types = get_string_literal_types(
                        type_checker
                            .get_contextual_type(node, tsox_checker::checker::ContextFlags::None)
                            .as_ref(),
                        Some(&mut uniques),
                        type_checker,
                    );
                    string_literal_types.extend(get_string_literal_types(
                        type_checker
                            .get_contextual_type(
                                node,
                                tsox_checker::checker::ContextFlags::IgnoreNodeInferences,
                            )
                            .as_ref(),
                        Some(&mut uniques),
                        type_checker,
                    ));
                    return to_string_literal_completions_from_types(string_literal_types);
                }
                Some(StringLiteralCompletions {
                    from_types: from_contextual_type(
                        tsox_checker::checker::ContextFlags::None,
                        node,
                        type_checker,
                    ),
                    from_properties: None,
                    from_paths: None,
                })
            }
            tsox_frontend::ast::SyntaxKind::ElementAccessExpression => {
                let tsox_frontend::ast::NodeData::ElementAccessExpression(d) = &parent.data else {
                    return None;
                };
                let expression = d.expression.clone();
                let argument_expression = d.argument_expression.clone();
                if Arc::ptr_eq(&tsox_frontend::ast::skip_parentheses(&argument_expression), node) {
                    let t = type_checker.get_type_at_location(&expression);
                    return Some(StringLiteralCompletions {
                        from_properties: Some(string_literal_completions_from_properties(
                            &t, type_checker,
                        )),
                        from_types: None,
                        from_paths: None,
                    });
                }
                None
            }
            tsox_frontend::ast::SyntaxKind::CallExpression
            | tsox_frontend::ast::SyntaxKind::NewExpression
            | tsox_frontend::ast::SyntaxKind::JsxAttribute => {
                if !is_require_call_argument(node) && !tsox_frontend::ast::is_import_call(&parent) {
                    let argument_node = if parent.kind == tsox_frontend::ast::SyntaxKind::JsxAttribute {
                        parent.parent()?
                    } else {
                        node.clone()
                    };
                    let argument_info =
                        crate::ls::mig::m5q_3::get_argument_info_for_completions(
                            &argument_node,
                            position as i32,
                            file,
                            type_checker,
                        )?;
                    let result = get_string_literal_completions_from_signature(
                        &parent,
                        node,
                        &argument_info,
                        type_checker,
                    );
                    if let Some(result) = result {
                        return Some(StringLiteralCompletions {
                            from_types: Some(result),
                            from_properties: None,
                            from_paths: None,
                        });
                    }
                    return Some(StringLiteralCompletions {
                        from_types: from_contextual_type(
                            tsox_checker::checker::ContextFlags::None,
                            node,
                            type_checker,
                        ),
                        from_properties: None,
                        from_paths: None,
                    });
                }
                self.m5w2_get_string_literal_completions_from_module_names(
                    file,
                    node,
                    &self.get_program(),
                    type_checker,
                )
            }
            tsox_frontend::ast::SyntaxKind::ImportDeclaration
            | tsox_frontend::ast::SyntaxKind::ExportDeclaration
            | tsox_frontend::ast::SyntaxKind::ExternalModuleReference
            | tsox_frontend::ast::SyntaxKind::JSDocImportTag => self
                .m5w2_get_string_literal_completions_from_module_names(
                    file,
                    node,
                    &self.get_program(),
                    type_checker,
                ),
            tsox_frontend::ast::SyntaxKind::CaseClause => {
                let case_block = parent.parent()?;
                let tsox_frontend::ast::NodeData::CaseBlock(cb) = &case_block.data else {
                    panic!("parent of CaseClause is not CaseBlock")
                };
                let tracker = crate::ls::mig::m5q2b_6::new_case_clause_tracker(
                    type_checker,
                    &cb.clauses.nodes,
                );
                let contextual_types = from_contextual_type(
                    tsox_checker::checker::ContextFlags::IgnoreNodeInferences,
                    node,
                    type_checker,
                )?;
                let literals: Vec<_> = contextual_types
                    .types
                    .iter()
                    .filter(|t| {
                        !crate::ls::mig::m5q2b_6::tracker_has_value_m5q2b(
                            &tracker,
                            &ConstantValue::String(literal_value_as_string(t)),
                        )
                    })
                    .cloned()
                    .collect();
                Some(StringLiteralCompletions {
                    from_types: Some(CompletionsFromTypes {
                        types: literals,
                        is_new_identifier: false,
                    }),
                    from_properties: None,
                    from_paths: None,
                })
            }
            tsox_frontend::ast::SyntaxKind::ImportSpecifier
            | tsox_frontend::ast::SyntaxKind::ExportSpecifier => {
                let specifier = &parent;
                if let Some(property_name) = tsox_frontend::ast::mig::m3b::property_name(specifier)
                {
                    if !Arc::ptr_eq(node, property_name) {
                        return None;
                    }
                }
                let named_imports_or_exports = specifier.parent()?;
                let module_specifier = if named_imports_or_exports.kind
                    == tsox_frontend::ast::SyntaxKind::NamedImports
                {
                    named_imports_or_exports.parent()?.parent()?
                } else {
                    named_imports_or_exports.parent()?
                };
                let module_specifier_symbol =
                    type_checker.get_symbol_at_location(&module_specifier)?;
                let exports = type_checker
                    .get_exports_and_properties_of_module(&module_specifier_symbol);
                let existing: tsox_core::collections::set::Set<String> =
                    tsox_core::collections::set::Set::from_items(
                        tsox_frontend::ast::mig::m3b::elements(&named_imports_or_exports)
                            .iter()
                            .filter_map(|n| {
                                tsox_frontend::ast::mig::m3b::property_name_or_name(n)
                                    .map(|p| p.text().to_string())
                            }),
                    );
                let uniques: Vec<Arc<Symbol>> = exports
                    .into_iter()
                    .filter(|e| {
                        e.name != tsox_frontend::ast::INTERNAL_SYMBOL_NAME_DEFAULT
                            && !existing.has(&e.name)
                    })
                    .collect();
                Some(StringLiteralCompletions {
                    from_properties: Some(CompletionsFromProperties {
                        symbols: uniques,
                        has_index_signature: false,
                    }),
                    from_types: None,
                    from_paths: None,
                })
            }
            tsox_frontend::ast::SyntaxKind::BinaryExpression => {
                let tsox_frontend::ast::NodeData::BinaryExpression(be) = &parent.data else {
                    unreachable!()
                };
                if be.operator_token.kind == tsox_frontend::ast::SyntaxKind::InKeyword {
                    let t = type_checker.get_type_at_location(&be.right);
                    let properties =
                        crate::ls::mig::m5r::get_properties_for_completion(&t, type_checker);
                    return Some(StringLiteralCompletions {
                        from_properties: Some(CompletionsFromProperties {
                            symbols: properties
                                .into_iter()
                                .filter(|s| match &s.value_declaration {
                                    None => true,
                                    Some(vd) => !tsox_frontend::ast::mig::m3g_2::is_private_identifier_class_element_declaration(vd),
                                })
                                .collect(),
                            has_index_signature: false,
                        }),
                        from_types: None,
                        from_paths: None,
                    });
                }
                Some(StringLiteralCompletions {
                    from_types: from_contextual_type(
                        tsox_checker::checker::ContextFlags::None,
                        node,
                        type_checker,
                    ),
                    from_properties: None,
                    from_paths: None,
                })
            }
            _ => {
                let result = from_contextual_type(
                    tsox_checker::checker::ContextFlags::IgnoreNodeInferences,
                    node,
                    type_checker,
                );
                if let Some(result) = result {
                    return Some(StringLiteralCompletions {
                        from_types: Some(result),
                        from_properties: None,
                        from_paths: None,
                    });
                }
                Some(StringLiteralCompletions {
                    from_types: from_contextual_type(
                        tsox_checker::checker::ContextFlags::None,
                        node,
                        type_checker,
                    ),
                    from_properties: None,
                    from_paths: None,
                })
            }
        }
    }

    pub fn m5w2_get_string_literal_completion_details(
        &self,
        checker: &mut Checker,
        item: lsp::CompletionItem,
        name: &str,
        file: &Arc<SourceFile>,
        position: usize,
        context_token: Option<&Arc<Node>>,
        doc_format: crate::lsp::lsproto::MarkupKind,
    ) -> lsp::CompletionItem {
        let Some(context_token) = context_token else {
            return item;
        };
        if !tsox_frontend::ast::is_string_literal_like(context_token) {
            return item;
        }
        let Some(completions) =
            self.m5w2_get_string_literal_completion_entries(file, context_token, position, checker)
        else {
            return item;
        };
        self.m5w2_string_literal_completion_details(
            item,
            name,
            context_token,
            position,
            &completions,
            file,
            checker,
            doc_format,
        )
    }

    fn m5w2_string_literal_completion_details(
        &self,
        mut item: lsp::CompletionItem,
        name: &str,
        location: &Arc<Node>,
        position: usize,
        completion: &StringLiteralCompletions,
        file: &Arc<SourceFile>,
        checker: &mut Checker,
        doc_format: crate::lsp::lsproto::MarkupKind,
    ) -> lsp::CompletionItem {
        if completion.from_paths.is_some() {
            return item;
        }
        if let Some(from_properties) = &completion.from_properties {
            for symbol in &from_properties.symbols {
                if symbol.name == name {
                    let detailed = self.m5w2_create_completion_details_for_symbol(
                        &mut item,
                        symbol,
                        checker,
                        location,
                        position,
                        doc_format,
                    );
                    return detailed.clone();
                }
            }
        }
        if let Some(from_types) = &completion.from_types {
            for t in &from_types.types {
                if literal_value_as_string(t) == name {
                    return m5w2_create_completion_details(&mut item, name, "", doc_format)
                        .clone();
                }
            }
        }
        item
    }

    pub fn m5w2_get_triple_slash_reference_completions(
        &self,
        file: &Arc<SourceFile>,
        position: usize,
        program: &Arc<Program>,
        checker: Option<&mut Checker>,
    ) -> Option<PathCompletions> {
        let compiler_options = program.options();
        let token = tsox_frontend::astnav::get_token_at_position(&file.node, position)?;

        let comment_ranges =
            tsox_frontend::scanner::get_leading_comment_ranges(&file.text, token.pos());

        let mut found_range: Option<tsox_frontend::scanner::CommentRange> = None;
        for comment_range in comment_ranges {
            if position >= comment_range.pos && position <= comment_range.end {
                found_range = Some(comment_range);
                break;
            }
        }
        let found_range = found_range?;

        let text = &file.text[found_range.pos..position];
        let (prefix, kind, to_complete, ok) = parse_triple_slash_directive_fragment(text);
        if !ok {
            return None;
        }
        let replacement_span = self.path_completion_replacement_span(
            file,
            get_directory_fragment_range(&to_complete, found_range.pos + prefix.len()),
        )?;

        let script_path = tsp::get_directory_path(&file.file_name);

        let names: Vec<ModuleCompletionNameAndKind> = match kind.as_str() {
            "path" => {
                let extension_options = self.m5w2_get_extension_options(
                    compiler_options,
                    ReferenceKind::FileName,
                    Some(file),
                    tsox_core::core::compiler_options_kinds::ResolutionMode::None,
                    None,
                );
                let mut result = crate::ls::mig::m5w2::ModuleCompletionNameAndKindSet::default();
                self.m5w2_get_completion_entries_for_directory_fragment(
                    &to_complete,
                    &script_path,
                    &extension_options,
                    program,
                    true,
                    &file.file_name,
                    &mut result,
                );
                result.names.into_values().collect()
            }
            "types" => {
                let extension_options = self.m5w2_get_extension_options(
                    compiler_options,
                    ReferenceKind::ModuleSpecifier,
                    Some(file),
                    tsox_core::core::compiler_options_kinds::ResolutionMode::None,
                    None,
                );
                let mut result = crate::ls::mig::m5w2::ModuleCompletionNameAndKindSet::default();
                self.m5w2_get_completion_entries_from_typings(
                    program,
                    &script_path,
                    &get_fragment_directory(&to_complete),
                    &extension_options,
                    &mut result,
                );
                result.names.into_values().collect()
            }
            _ => Vec::new(),
        };

        Some(PathCompletions {
            entries: m5w2_to_path_completions(names),
            replacement_span: Some(replacement_span),
        })
    }
}

pub fn from_contextual_type(
    context_flags: tsox_checker::checker::ContextFlags,
    node: &Arc<Node>,
    type_checker: &mut Checker,
) -> Option<CompletionsFromTypes> {
    to_completions_from_types(get_string_literal_types(
        crate::ls::mig::m5x_6::get_contextual_type_from_parent(node, type_checker, context_flags)
            .as_ref(),
        None,
        type_checker,
    ))
}

pub fn to_completions_from_types(
    types: Vec<Arc<tsox_checker::checker::Type>>,
) -> Option<CompletionsFromTypes> {
    if types.is_empty() {
        return None;
    }
    Some(CompletionsFromTypes {
        types,
        is_new_identifier: false,
    })
}

pub fn to_string_literal_completions_from_types(
    types: Vec<Arc<tsox_checker::checker::Type>>,
) -> Option<StringLiteralCompletions> {
    let result = to_completions_from_types(types)?;
    Some(StringLiteralCompletions {
        from_types: Some(result),
        from_properties: None,
        from_paths: None,
    })
}

pub fn from_unionable_literal_type(
    grandparent: &Arc<Node>,
    parent: &Arc<Node>,
    position: usize,
    type_checker: &mut Checker,
) -> Option<StringLiteralCompletions> {
    use tsox_frontend::ast::SyntaxKind;
    match grandparent.kind {
        SyntaxKind::CallExpression
        | SyntaxKind::ExpressionWithTypeArguments
        | SyntaxKind::JsxOpeningElement
        | SyntaxKind::JsxSelfClosingElement
        | SyntaxKind::NewExpression
        | SyntaxKind::TaggedTemplateExpression
        | SyntaxKind::TypeReference => {
            let type_argument = tsox_frontend::ast::find_ancestor(parent, |n| {
                n.parent().is_some_and(|p| Arc::ptr_eq(&p, grandparent))
            })?;
            let t = type_checker.get_type_argument_constraint(&type_argument);
            Some(StringLiteralCompletions {
                from_types: Some(CompletionsFromTypes {
                    types: get_string_literal_types(t.as_ref(), None, type_checker),
                    is_new_identifier: false,
                }),
                from_properties: None,
                from_paths: None,
            })
        }
        SyntaxKind::IndexedAccessType => {
            let tsox_frontend::ast::NodeData::IndexedAccessTypeNode(d) = &grandparent.data else {
                unreachable!()
            };
            let index_type = d.index_type.clone();
            let object_type = d.object_type.clone();
            if !index_type.loc.contains_inclusive(position) {
                return None;
            }
            let t = type_checker.get_type_from_type_node(&object_type);
            Some(StringLiteralCompletions {
                from_properties: Some(string_literal_completions_from_properties(
                    &t, type_checker,
                )),
                from_types: None,
                from_paths: None,
            })
        }
        SyntaxKind::UnionType => {
            let result = from_unionable_literal_type(
                &walk_up_parentheses(grandparent.parent()?)?,
                parent,
                position,
                type_checker,
            )?;
            let already_used_types =
                get_already_used_types_in_string_literal_union(grandparent, parent);
            if let Some(from_properties) = result.from_properties {
                return Some(StringLiteralCompletions {
                    from_properties: Some(CompletionsFromProperties {
                        symbols: from_properties
                            .symbols
                            .into_iter()
                            .filter(|s| !already_used_types.contains(&s.name))
                            .collect(),
                        has_index_signature: from_properties.has_index_signature,
                    }),
                    from_types: None,
                    from_paths: None,
                });
            }
            if let Some(from_types) = result.from_types {
                return Some(StringLiteralCompletions {
                    from_types: Some(CompletionsFromTypes {
                        types: from_types
                            .types
                            .into_iter()
                            .filter(|t| {
                                !already_used_types.contains(&literal_value_as_string(t))
                            })
                            .collect(),
                        is_new_identifier: false,
                    }),
                    from_properties: None,
                    from_paths: None,
                });
            }
            None
        }
        SyntaxKind::PropertySignature => Some(StringLiteralCompletions {
            from_types: Some(CompletionsFromTypes {
                types: get_string_literal_types(
                    m5w2_get_constraint_of_type_argument_property(
                        grandparent,
                        type_checker,
                    )
                    .as_ref(),
                    None,
                    type_checker,
                ),
                is_new_identifier: false,
            }),
            from_properties: None,
            from_paths: None,
        }),
        _ => None,
    }
}

pub fn string_literal_completions_for_object_literal(
    type_checker: &mut Checker,
    object_literal_expression: &Arc<Node>,
) -> Option<CompletionsFromProperties> {
    let contextual_type = type_checker.get_contextual_type(
        object_literal_expression,
        tsox_checker::checker::ContextFlags::None,
    )?;
    let completions_type = type_checker.get_contextual_type(
        object_literal_expression,
        tsox_checker::checker::ContextFlags::IgnoreNodeInferences,
    );
    let symbols = m5w2_get_properties_for_object_expression(
        &contextual_type,
        completions_type.as_ref(),
        object_literal_expression,
        type_checker,
    );

    Some(CompletionsFromProperties {
        symbols,
        has_index_signature: has_index_signature(&contextual_type, type_checker),
    })
}

pub fn string_literal_completions_from_properties(
    t: &Arc<tsox_checker::checker::Type>,
    type_checker: &mut Checker,
) -> CompletionsFromProperties {
    CompletionsFromProperties {
        symbols: type_checker
            .get_apparent_properties(t)
            .into_iter()
            .filter(|s| match &s.value_declaration {
                None => true,
                Some(vd) => !tsox_frontend::ast::mig::m3g_2::is_private_identifier_class_element_declaration(vd),
            })
            .collect(),
        has_index_signature: has_index_signature(t, type_checker),
    }
}

pub fn get_string_literal_completions_from_signature(
    call: &Arc<Node>,
    arg: &Arc<Node>,
    argument_info: &crate::ls::mig::m5q_3::ArgumentInfoForCompletions,
    type_checker: &mut Checker,
) -> Option<CompletionsFromTypes> {
    let mut is_new_identifier = false;
    let mut uniques = tsox_core::collections::set::Set::new();
    let editing_argument = if tsox_frontend::ast::mig::m3g::is_jsx_opening_like_element(call) {
        tsox_frontend::ast::find_ancestor(&arg.parent()?, |n| {
            tsox_frontend::ast::is_jsx_attribute(n)
        })?
    } else {
        arg.clone()
    };
    let candidates =
        type_checker.get_candidate_signatures_for_string_literal_completions(call, &editing_argument);
    let mut types: Vec<Arc<tsox_checker::checker::Type>> = Vec::new();
    for candidate in candidates {
        if !candidate.has_rest_parameter() && argument_info.argument_count > candidate.parameters().len()
        {
            continue;
        }
        let mut t =
            type_checker.get_type_parameter_at_position(&candidate, argument_info.argument_index);
        if tsox_frontend::ast::mig::m3g::is_jsx_opening_like_element(call) {
            let attr_name = match &editing_argument.data {
                tsox_frontend::ast::NodeData::JsxAttribute(a) => a.name.text().to_string(),
                _ => String::new(),
            };
            if let Some(prop_type) =
                type_checker.get_type_of_property_of_type(&t, &attr_name)
            {
                t = prop_type;
            }
        }
        is_new_identifier = is_new_identifier || t.is_string();
        types.extend(get_string_literal_types(Some(&t), Some(&mut uniques), type_checker));
    }
    if !types.is_empty() {
        return Some(CompletionsFromTypes {
            types,
            is_new_identifier,
        });
    }
    None
}

pub fn get_string_literal_types(
    t: Option<&Arc<tsox_checker::checker::Type>>,
    uniques: Option<&mut tsox_core::collections::set::Set<String>>,
    type_checker: &mut Checker,
) -> Vec<Arc<tsox_checker::checker::Type>> {
    let Some(t) = t else {
        return Vec::new();
    };
    let mut local_uniques;
    let uniques = match uniques {
        Some(u) => u,
        None => {
            local_uniques = tsox_core::collections::set::Set::new();
            &mut local_uniques
        }
    };
    let t = crate::ls::mig::m5x_5::skip_constraint(t, type_checker);
    if t.is_union() {
        let mut types = Vec::new();
        for element_type in t.types().unwrap_or(&[]) {
            types.extend(get_string_literal_types(
                Some(element_type),
                Some(uniques),
                type_checker,
            ));
        }
        return types;
    }
    if t.is_string_literal() && !t.is_enum_literal() && uniques.add_if_absent(literal_value_as_string(&t))
    {
        return vec![t.clone()];
    }
    Vec::new()
}

pub fn get_already_used_types_in_string_literal_union(
    union: &Arc<Node>,
    current: &Arc<Node>,
) -> Vec<String> {
    let tsox_frontend::ast::NodeData::UnionTypeNode(d) = &union.data else {
        unreachable!()
    };
    let types_list = d.types.clone();
    let mut values = Vec::new();
    for type_node in types_list.nodes.iter() {
        if !Arc::ptr_eq(type_node, current)
            && tsox_frontend::ast::is_literal_type_node(type_node)
        {
            if let tsox_frontend::ast::NodeData::LiteralTypeNode(lt) = &type_node.data {
                if tsox_frontend::ast::is_string_literal(&lt.literal) {
                    values.push(lt.literal.text().to_string());
                }
            }
        }
    }
    values
}

pub fn has_index_signature(
    t: &Arc<tsox_checker::checker::Type>,
    type_checker: &mut Checker,
) -> bool {
    type_checker.get_string_index_type(t).is_some() || type_checker.get_number_index_type(t).is_some()
}

pub fn is_require_call_argument(node: &Arc<Node>) -> bool {
    let parent = node.parent();
    match parent {
        Some(parent) if tsox_frontend::ast::is_call_expression(&parent) => {
            let arguments = tsox_frontend::ast::mig::x1a::arguments(&parent);
            !arguments.is_empty()
                && Arc::ptr_eq(&arguments[0], node)
                && parent
                    .expression()
                    .map(|e| tsox_frontend::ast::is_identifier(e))
                    .unwrap_or(false)
                && parent.expression().map(|e| e.text()) == Some("require")
        }
        _ => false,
    }
}

pub fn walk_up_parentheses(node: Arc<Node>) -> Option<Arc<Node>> {
    match node.kind {
        tsox_frontend::ast::SyntaxKind::ParenthesizedType => {
            tsox_frontend::ast::mig::m3h::walk_up_parenthesized_types(&node)
        }
        tsox_frontend::ast::SyntaxKind::ParenthesizedExpression => {
            tsox_frontend::ast::mig::m3h::walk_up_parenthesized_expressions(&node)
        }
        _ => Some(node),
    }
}


fn literal_value_as_string(t: &Arc<tsox_checker::checker::Type>) -> String {
    t.as_literal_type()
        .map(|l| match &l.value {
            tsox_checker::checker::types::LiteralValue::String(s) => s.clone(),
            _ => String::new(),
        })
        .unwrap_or_default()
}

pub fn has_triple_slash_prefix(comment_text: &str) -> bool {
    comment_text.starts_with("///") && comment_text[3..].trim_start().starts_with('<')
}

impl LanguageService {
    /// Go completions.go:4834 setItemDefaults（m5q2_2 版为私有，此处按 Go 语义本片等价实现）。
    fn m5w2_set_item_defaults(
        &self,
        position: usize,
        file: &Arc<SourceFile>,
        items: &mut [lsp::CompletionItem],
        default_commit_characters: Option<&Vec<String>>,
        optional_replacement_span: Option<&crate::lsp::lsproto::Range>,
    ) -> Option<CompletionItemDefaults> {
        use crate::ls::mig::m5q_3 as q3;
        let ctx = crate::mig::m5m::ResolvedClientCapabilitiesContext {
            capabilities: None,
        };

        let mut item_defaults: Option<CompletionItemDefaults> = None;
        if let Some(default_commit_characters) = default_commit_characters {
            let supports_item_commit_characters =
                q3::client_supports_item_commit_characters(&ctx);
            if q3::client_supports_default_commit_characters(&ctx)
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
            let (end, fidelity) = self.create_lsp_position_m5x(position, file);
            if fidelity != crate::ls::mig::m5v_7::SPANMAP_FIDELITY_EXACT {
                return item_defaults;
            }
            let insert_range = crate::lsp::lsproto::Range {
                start: optional_replacement_span.start.clone(),
                end,
            };
            if q3::client_supports_default_edit_range(&ctx) {
                let defaults = item_defaults.get_or_insert_with(CompletionItemDefaults::default);
                defaults.edit_range = Some(crate::ls::mig::m5q2_2::EditRangeWithInsertReplace {
                    insert: insert_range.clone(),
                    replace: optional_replacement_span.clone(),
                });
                for item in items.iter_mut() {
                    if item.insert_text.is_some() && item.text_edit.is_none() {
                        item.text_edit = Some(lsp::TextEditOrInsertReplaceEdit {
                            text_edit: None,
                            insert_replace_edit: Some(lsp::InsertReplaceEdit {
                                new_text: item.insert_text.clone().unwrap_or_default(),
                                insert: insert_range.clone(),
                                replace: optional_replacement_span.clone(),
                            }),
                        });
                        item.insert_text = None;
                    }
                }
            } else if q3::client_supports_item_insert_replace(&ctx) {
                for item in items.iter_mut() {
                    if item.text_edit.is_none() {
                        item.text_edit = Some(lsp::TextEditOrInsertReplaceEdit {
                            text_edit: None,
                            insert_replace_edit: Some(lsp::InsertReplaceEdit {
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

    /// Go completions.go:5728 createCompletionDetailsForSymbol（m5q_3 版参数为其私有影子类型，此处按 Go 语义本片等价实现）。
    fn m5w2_create_completion_details_for_symbol<'a>(
        &self,
        item: &'a mut lsp::CompletionItem,
        symbol: &Arc<Symbol>,
        checker: &mut Checker,
        location: &Arc<Node>,
        position: usize,
        doc_format: crate::lsp::lsproto::MarkupKind,
    ) -> &'a mut lsp::CompletionItem {
        let content_format = match &doc_format {
            crate::lsp::lsproto::MarkupKind::Markdown => "markdown",
            crate::lsp::lsproto::MarkupKind::PlainText => "plaintext",
        };
        let (quick_info, documentation, _, _) = self.get_quick_info_and_documentation_for_symbol(
            checker,
            Some(symbol),
            location,
            content_format,
            None,
            false,
        );
        m5w2_create_completion_details(item, &quick_info, &documentation, doc_format)
    }
}

/// Go completions.go:5700 createCompletionDetails。
fn m5w2_create_completion_details<'a>(
    item: &'a mut lsp::CompletionItem,
    detail: &str,
    documentation: &str,
    doc_format: crate::lsp::lsproto::MarkupKind,
) -> &'a mut lsp::CompletionItem {
    if item.detail.is_none() && !detail.is_empty() {
        item.detail = Some(detail.to_string());
    }
    if !documentation.is_empty() {
        item.documentation = Some(lsp::StringOrMarkupContent {
            string: None,
            markup_content: Some(lsp::MarkupContent {
                kind: doc_format,
                value: documentation.to_string(),
            }),
        });
    }
    item
}

/// Go completions.go:4284 getConstraintOfTypeArgumentProperty（completions_helpers 缺失，本片按 Go 移植）。
fn m5w2_get_constraint_of_type_argument_property(
    node: &Arc<Node>,
    type_checker: &mut Checker,
) -> Option<Arc<tsox_checker::checker::Type>> {
    if tsox_frontend::ast::is_type_node(node) {
        if let Some(constraint) = type_checker.get_type_argument_constraint(node) {
            return Some(constraint);
        }
    }

    let parent = node.parent()?;
    let t = m5w2_get_constraint_of_type_argument_property(&parent, type_checker)?;
    match node.kind {
        tsox_frontend::ast::SyntaxKind::PropertySignature => {
            let reparsed = tsox_frontend::ast::mig::m3f::get_reparsed_node_for_node(node);
            if let Some(symbol) = type_checker.get_symbol_at_location(&reparsed) {
                return type_checker.get_type_of_property_of_contextual_type(&t, &symbol.name);
            }
            if let Some(name) = reparsed.name() {
                if let Some(text) =
                    tsox_frontend::ast::mig::m3h::try_get_text_of_property_name(name)
                {
                    return type_checker.get_type_of_property_of_contextual_type(&t, &text);
                }
            }
            None
        }
        tsox_frontend::ast::SyntaxKind::ColonToken => {
            if parent.kind == tsox_frontend::ast::SyntaxKind::PropertySignature {
                return Some(t);
            }
            None
        }
        tsox_frontend::ast::SyntaxKind::IntersectionType
        | tsox_frontend::ast::SyntaxKind::TypeLiteral
        | tsox_frontend::ast::SyntaxKind::UnionType => Some(t),
        tsox_frontend::ast::SyntaxKind::OpenBracketToken => {
            type_checker.get_element_type_of_array_type(&t)
        }
        _ => None,
    }
}

/// Go completions.go:4411 getPropertiesForObjectExpression（completions_helpers 缺失，本片按 Go 移植；
/// getApparentProperties/containsNonPublicProperties 复用 m5q_3 既有移植）。
fn m5w2_get_properties_for_object_expression(
    contextual_type: &Arc<tsox_checker::checker::Type>,
    completions_type: Option<&Arc<tsox_checker::checker::Type>>,
    obj: &Arc<Node>,
    type_checker: &mut Checker,
) -> Vec<Arc<Symbol>> {
    use tsox_checker::checker::types::{TYPE_FLAGS_ANY_OR_UNKNOWN, UnionReduction};
    use tsox_checker::checker::Type;

    let has_completions_type = completions_type.is_some_and(|ct| !Arc::ptr_eq(ct, contextual_type));
    let types: Vec<Arc<Type>> = if contextual_type.is_union() {
        contextual_type.types().unwrap_or(&[]).to_vec()
    } else {
        vec![Arc::clone(contextual_type)]
    };
    let promise_filtered: Vec<Arc<Type>> = types
        .into_iter()
        .filter(|t| type_checker.get_promised_type_of_promise_ex(t, None, None).is_none())
        .collect();
    let promise_filtered_contextual_type =
        type_checker.get_union_type_ex(promise_filtered, UnionReduction::Literal);

    let t: Arc<Type> = if let Some(ct) = completions_type {
        if has_completions_type && !ct.flags.intersects(TYPE_FLAGS_ANY_OR_UNKNOWN) {
            type_checker.get_union_type_ex(
                vec![promise_filtered_contextual_type, Arc::clone(ct)],
                UnionReduction::Literal,
            )
        } else {
            promise_filtered_contextual_type
        }
    } else {
        promise_filtered_contextual_type
    };

    let properties = crate::ls::mig::m5q_3::get_apparent_properties(&t, obj, type_checker);
    if t.is_class() && crate::ls::mig::m5q_3::contains_non_public_properties(&properties) {
        return Vec::new();
    }
    if has_completions_type {
        return properties
            .into_iter()
            .filter(|member| {
                if member.declarations.is_empty() {
                    return true;
                }
                member
                    .declarations
                    .iter()
                    .any(|decl| decl.parent().is_some_and(|p| !Arc::ptr_eq(&p, obj)))
            })
            .collect();
    }
    properties
}

pub fn parse_triple_slash_directive_fragment(text: &str) -> (String, String, String, bool) {
    let empty = String::new();
    if !text.starts_with("///") {
        return (empty.clone(), empty.clone(), empty.clone(), false);
    }

    let mut rest = &text[3..];
    rest = rest.trim_start_matches(|c: char| tsox_core::stringutil::is_white_space_like(c));

    if !rest.starts_with("<reference") {
        return (empty.clone(), empty.clone(), empty.clone(), false);
    }
    rest = &rest["<reference".len()..];

    if rest.is_empty() || !tsox_core::stringutil::is_white_space_like(rest.chars().next().unwrap()) {
        return (empty.clone(), empty.clone(), empty.clone(), false);
    }
    rest = rest.trim_start_matches(|c: char| tsox_core::stringutil::is_white_space_like(c));

    let kind;
    if rest.starts_with("path") {
        kind = "path".to_string();
        rest = &rest["path".len()..];
    } else if rest.starts_with("types") {
        kind = "types".to_string();
        rest = &rest["types".len()..];
    } else {
        return (empty.clone(), empty.clone(), empty.clone(), false);
    }

    rest = rest.trim_start_matches(|c: char| tsox_core::stringutil::is_white_space_like(c));
    if !rest.starts_with('=') {
        return (empty.clone(), empty.clone(), empty.clone(), false);
    }
    rest = &rest[1..];

    rest = rest.trim_start_matches(|c: char| tsox_core::stringutil::is_white_space_like(c));
    if rest.is_empty() || (rest.as_bytes()[0] != b'\'' && rest.as_bytes()[0] != b'"') {
        return (empty.clone(), empty.clone(), empty.clone(), false);
    }
    rest = &rest[1..];

    if rest.contains('\'') || rest.contains('"') {
        return (empty.clone(), empty.clone(), empty.clone(), false);
    }
    let to_complete = rest.to_string();
    let prefix = text[..text.len() - to_complete.len()].to_string();
    (prefix, kind, to_complete, true)
}
