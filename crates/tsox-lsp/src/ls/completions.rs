#![allow(dead_code)]

pub(crate) use crate::ls::completions_helpers::*;
pub(crate) use crate::ls::{
    completions_context, completions_object_like, completions_object_like_types,
};
#[allow(unused_imports)]
pub(crate) use crate::ls::completions_members::*;

pub(crate) use std::sync::Arc;

pub(crate) use crate::lsp::lsproto_lsp::DocumentUri;
pub(crate) use crate::lsp::lsproto_lsp::Position;
pub(crate) use tsox_frontend::ast::SourceFile;
pub(crate) use tsox_frontend::ast::Symbol;
pub(crate) use tsox_frontend::ast::SymbolFlags;

pub(crate) use super::language_service::LanguageService;
pub(crate) use super::types::{
    CompletionContext, CompletionItem, CompletionItemData, CompletionList,
};

pub const ERR_NEEDS_AUTO_IMPORTS: &str = "completion list needs auto imports";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    None,
    Global,
    PropertyAccess,
    Member,
    String,
    Import,
    ObjectLiteralMember,
    JsDocTagName,
    JsDocTag,
    JsDocParameterName,
}

pub struct CompletionDataData {
    pub symbols: Vec<Arc<Symbol>>,
    pub completion_kind: CompletionKind,
    pub is_in_snippet_scope: bool,
}

impl LanguageService {
    pub fn provide_completion(
        &self,
        document_uri: &DocumentUri,
        position: Position,
        _context: &CompletionContext,
    ) -> CompletionList {
        let (_program, source_file) = self.get_program_and_file(document_uri);
        let offset = lsp_position_to_offset(&source_file.line_map, &position);
        match self.get_completions_at_position(&source_file, offset, None, false) {
            Ok(list) => ensure_item_data(&source_file.file_name, offset, list),
            Err(_) => CompletionList::default(),
        }
    }

    pub fn get_completions_at_position(
        &self,
        file: &Arc<SourceFile>,
        position: usize,
        _trigger_character: Option<&str>,
        _include_symbols: bool,
    ) -> Result<CompletionList, String> {
        let mut node = find_deepest_node(&file.node, position);
        // EOF 边界（未闭合串跨到文件尾）：offset==全部节点 end 时 deepest 退化
        // 为 SourceFile；按 Go preceding-token 语义回看 offset-1 定位 token
        if node.kind == tsox_frontend::ast::SyntaxKind::SourceFile && position > 0 {
            let boundary = find_deepest_node(&file.node, position - 1);
            // EOF 回看仅用于 token 恢复（未闭合串/点）；回看命中标识符等
            // 正常 token 时保持原位（extends 子句 EOF 补全仍走 scope 路径）
            if boundary.kind != tsox_frontend::ast::SyntaxKind::SourceFile {
                node = boundary;
            }
        }
        let jsx = completions_context::is_jsx_file(file);

        let mut checker = program_build_checker(&self.get_program());

        let comment_ranges = completions_context::comment_ranges(&file.text, jsx);
        if let Some(range) = completions_context::enclosing_comment(&comment_ranges, &file.text, position)
            && !completions_context::is_doc_comment(&range, &file.text)
        {
            return Ok(CompletionList::default());
        }

        // Go right-of-dot：位置在 '.'（或 '?.'）之后的成员补全，优先于全局
        // scope 符号（completions.go getTypeScriptMemberSymbols）；点后语境
        // 无成员即空列表，不回退 scope
        match member_symbols_after_dot(&mut checker, &node, position) {
            MemberDotResult::Dot(symbols) => {
                let items = symbols
                    .iter()
                    .filter(|s| !s.name.is_empty() && !s.name.starts_with('\u{FE}'))
                    .map(|s| symbol_to_completion_item(s))
                    .collect();
                return Ok(CompletionList {
                    is_incomplete: false,
                    items,
                });
            }
            MemberDotResult::NotDot => {}
        }

        // 字符串字面量位：形参约束的字面量集合 / 索引访问的属性名
        // （Go getStringLiteralCompletions）
        if let Some(labels) =
            crate::ls::string_completions::string_literal_completion_labels(&mut checker, &node, position)
        {
            let items = labels
                .into_iter()
                .map(|l| CompletionItem {
                    label: l,
                    kind: Some(12), // String kind
                    ..Default::default()
                })
                .collect();
            return Ok(CompletionList {
                is_incomplete: false,
                items,
            });
        }

        let (context_token, _previous_token) =
            completions_context::relevant_tokens(&file.text, jsx, 0, position);
        let containing_token = completions_context::token_containing(&file.text, jsx, position);

        // 模块说明符字符串（import/export/require/import() 实参）的路径补全
        if let Some(labels) = crate::ls::string_completions::relative_module_specifier_labels(
            &self.get_program(),
            file,
            &node,
            &file.text,
            position,
        ) {
            let items = labels
                .into_iter()
                .map(|l| CompletionItem {
                    label: l,
                    kind: Some(9), // Module kind
                    ..Default::default()
                })
                .collect();
            return Ok(CompletionList {
                is_incomplete: false,
                items,
            });
        }

        // 类型实参内类型字面量的成员补全（Go
        // tryGetObjectTypeLiteralInTypeArgumentCompletionSymbols）
        if let Some(symbols) = completions_object_like_types::type_literal_in_type_argument_completion(
            &mut checker,
            &node,
            &file.text,
            jsx,
            position,
        ) {
            let items = symbols
                .iter()
                .filter(|s| !s.name.is_empty() && !s.name.starts_with('\u{FE}'))
                .map(|s| symbol_to_completion_item(s))
                .collect();
            return Ok(CompletionList {
                is_incomplete: false,
                items,
            });
        }

        // Go isCompletionListBlocker：确定的无效补全位置直接空列表
        if completions_context::is_completion_list_blocker(
            context_token.as_ref(),
            containing_token.as_ref(),
            &file.text,
            position,
            &node,
        ) {
            return Ok(CompletionList::default());
        }

        if let Some(container) = completions_object_like::try_get_object_like_container(
            &node,
            &file.text,
            jsx,
            position,
        ) && let Some(symbols) = completions_object_like::object_like_completion(
            &mut checker,
            file,
            &container,
            &file.text,
            position,
        ) {
            let items = symbols
                .iter()
                .filter(|s| !s.name.is_empty() && !s.name.starts_with('\u{FE}'))
                .map(|s| symbol_to_completion_item(s))
                .collect();
            return Ok(CompletionList {
                is_incomplete: false,
                items,
            });
        }

        let meaning = SymbolFlags::VALUE
            .union(SymbolFlags::TYPE)
            .union(SymbolFlags::NAMESPACE);
        let mut symbols = checker.get_symbols_in_scope(&node, meaning);

        if symbols.is_empty() {
            symbols = collect_scope_symbols_fallback(&checker, file, &node);
        }

        let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();
        symbols.retain(|s| seen.insert(s.id()));

        let items: Vec<CompletionItem> = symbols
            .iter()
            .filter(|s| !s.name.is_empty() && !s.name.starts_with('\u{FE}'))
            .map(|s| symbol_to_completion_item(s))
            .collect();

        Ok(CompletionList {
            is_incomplete: false,
            items,
        })
    }

    pub fn get_completion_entry_details(
        &self,
        _file: &Arc<SourceFile>,
        _position: usize,
        _name: &str,
    ) -> Option<CompletionItem> {
        None
    }
}

pub fn ensure_item_data(file_name: &str, pos: usize, mut list: CompletionList) -> CompletionList {
    for item in &mut list.items {
        if item.data.is_none() {
            item.data = Some(CompletionItemData {
                file_name: file_name.to_string(),
                position: pos as i32,
                name: item.label.clone(),
            });
        }
    }
    list
}
