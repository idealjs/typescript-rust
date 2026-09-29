#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::lsp::lsproto_lsp::DocumentUri;
use crate::lsp::lsproto_lsp_basic::{Location, Position, Range};
use crate::ls::find_all_references::{EntryKind, RefOptions, ReferenceEntry, ReferenceUse};
use crate::ls::language_service::LanguageService;
use crate::ls::mig::m5s::{MultiDocumentHighlightsOrNull, SpanFeature, SpanFidelity};
use crate::ls::types_highlight::{DocumentHighlight, DocumentHighlightKind, MultiDocumentHighlight};
use tsox_compile::compiler::Program;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::astnav;

pub fn combine_multi_document_highlights(
    results: Vec<MultiDocumentHighlightsOrNull>,
) -> MultiDocumentHighlightsOrNull {
    let mut by_uri: HashMap<DocumentUri, usize> = HashMap::new();
    let mut seen: HashMap<DocumentUri, HashSet<Range>> = HashMap::new();
    let mut combined_documents: Vec<MultiDocumentHighlight> = Vec::new();
    for result in &results {
        let Some(documents) = &result.multi_document_highlights else {
            continue;
        };
        for document in documents {
            let index = *by_uri.entry(document.uri.clone()).or_insert_with(|| {
                combined_documents.push(MultiDocumentHighlight {
                    uri: document.uri.clone(),
                    highlights: Vec::new(),
                });
                combined_documents.len() - 1
            });
            let ranges = seen.entry(document.uri.clone()).or_default();
            for highlight in &document.highlights {
                if ranges.insert(highlight.range.clone()) {
                    combined_documents[index].highlights.push(highlight.clone());
                }
            }
        }
    }
    MultiDocumentHighlightsOrNull {
        multi_document_highlights: Some(combined_documents),
    }
}

impl LanguageService {
    pub fn to_document_highlight(
        &self,
        entry: &mut ReferenceEntry,
    ) -> (String, Option<DocumentHighlight>) {
        let file_name = entry.file_name.clone();

        let kind = DocumentHighlightKind::Read;
        let Some(lsp_range) =
            self.get_range_of_entry_for_feature(entry, SpanFeature::DocumentHighlights)
        else {
            return (file_name, None);
        };
        if entry.kind == EntryKind::Range {
            return (
                file_name,
                Some(DocumentHighlight {
                    range: lsp_range,
                    kind: Some(kind),
                }),
            );
        }

        let is_write = entry
            .node
            .as_ref()
            .map(|node| {
                tsox_frontend::ast::mig::m3b::is_write_access(node)
                    || node.kind == SyntaxKind::DefaultKeyword
            })
            .unwrap_or(false);
        let kind = if is_write {
            DocumentHighlightKind::Write
        } else {
            DocumentHighlightKind::Read
        };

        (
            file_name,
            Some(DocumentHighlight {
                range: lsp_range,
                kind: Some(kind),
            }),
        )
    }

    pub fn use_parent(
        &self,
        node: Option<&Arc<Node>>,
        node_test: impl Fn(&Arc<Node>) -> bool,
        get_nodes: impl Fn(&Arc<Node>, &Arc<SourceFile>) -> Vec<Arc<Node>>,
        source_file: &Arc<SourceFile>,
    ) -> Vec<DocumentHighlight> {
        if let Some(node) = node {
            if node_test(node) {
                return self.highlight_spans(&get_nodes(node, source_file), source_file);
            }
        }
        Vec::new()
    }

    pub fn highlight_spans(
        &self,
        nodes: &[Arc<Node>],
        source_file: &Arc<SourceFile>,
    ) -> Vec<DocumentHighlight> {
        if nodes.is_empty() {
            return Vec::new();
        }
        let mut highlights = Vec::new();
        let kind = DocumentHighlightKind::Read;
        for node in nodes {
            let (lsp_range, fidelity) = self.create_lsp_range_from_node_for_feature(
                node,
                source_file,
                SpanFeature::DocumentHighlights as u32,
            );
            if fidelity != 0 {
                highlights.push(DocumentHighlight {
                    range: lsp_range,
                    kind: Some(kind.clone()),
                });
            }
        }
        highlights
    }

    pub fn get_from_all_declarations(
        &self,
        node_test: impl Fn(&Arc<Node>) -> bool,
        keywords: &[SyntaxKind],
        node: &Arc<Node>,
        source_file: &Arc<SourceFile>,
    ) -> Vec<DocumentHighlight> {
        self.use_parent(
            node.parent().as_ref(),
            &node_test,
            |decl, sf| {
                let mut symbol_decls = Vec::new();
                if ast::can_have_symbol(decl) {
                    if let Some(symbol) =
                        tsox_checker::checker::mig::m1a::r19k2_defs::symbol_of_node(decl)
                    {
                        for d in symbol.declarations.iter() {
                            if node_test(d) {
                                'outer: for c in
                                    crate::ls::utilities::get_children_from_non_jsdoc_node(d, sf)
                                {
                                    for k in keywords {
                                        if c.kind == *k {
                                            symbol_decls.push(c);
                                            break 'outer;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                symbol_decls
            },
            source_file,
        )
    }

    pub fn get_if_else_occurrences(
        &self,
        if_statement: &Arc<Node>,
        source_file: &Arc<SourceFile>,
    ) -> Vec<DocumentHighlight> {
        let keywords = get_if_else_keywords(if_statement.clone(), source_file);
        let kind = DocumentHighlightKind::Read;
        let mut highlights = Vec::new();

        let mut i = 0;
        while i < keywords.len() {
            if keywords[i].kind == SyntaxKind::ElseKeyword && i < keywords.len() - 1 {
                let else_keyword = &keywords[i];
                let if_keyword = &keywords[i + 1];
                let mut should_combine = true;

                let mut if_token_start =
                    tsox_frontend::scanner::mig::x5a::get_token_pos_of_node(if_keyword, source_file, false)
                        as i64;
                if if_token_start < 0 {
                    if_token_start = if_keyword.pos() as i64;
                }
                let text = &source_file.text;
                let mut j = if_token_start - 1;
                while j >= else_keyword.end() as i64 {
                    if !tsox_core::stringutil::is_white_space_single_line(
                        text.as_bytes()[j as usize] as char,
                    ) {
                        should_combine = false;
                        break;
                    }
                    j -= 1;
                }
                if should_combine {
                    let (lsp_range, fidelity) = self.m5x_create_lsp_range_from_bounds(
                        tsox_frontend::scanner::skip_trivia(
                            &source_file.text,
                            else_keyword.pos(),
                        ),
                        if_keyword.end(),
                        source_file,
                    );
                    if fidelity != 0 {
                        highlights.push(DocumentHighlight {
                            range: lsp_range,
                            kind: Some(kind.clone()),
                        });
                    }
                    i += 1;
                    i += 1;
                    continue;
                }
            }
            let (lsp_range, fidelity) = self.create_lsp_range_from_node_for_feature(
                &keywords[i],
                source_file,
                SpanFeature::DocumentHighlights as u32,
            );
            if fidelity != 0 {
                highlights.push(DocumentHighlight {
                    range: lsp_range,
                    kind: Some(kind.clone()),
                });
            }
            i += 1;
        }
        highlights
    }
}

pub fn get_if_else_keywords(
    mut if_statement: Arc<Node>,
    source_file: &Arc<SourceFile>,
) -> Vec<Arc<Node>> {
    while let Some(parenting_if) = if_statement.parent() {
        if !ast::is_if_statement(&parenting_if) {
            break;
        }
        let else_statement = match &parenting_if.data {
            ast::NodeData::IfStatement(data) => data.else_statement.clone(),
            _ => None,
        };
        if !else_statement
            .map(|e| Arc::ptr_eq(&e, &if_statement))
            .unwrap_or(false)
        {
            break;
        }
        if_statement = parenting_if;
    }

    let mut keywords: Vec<Arc<Node>> = Vec::new();

    loop {
        let children = crate::ls::utilities::get_children_from_non_jsdoc_node(&if_statement, source_file);
        if !children.is_empty() && children[0].kind == SyntaxKind::IfKeyword {
            keywords.push(children[0].clone());
        }
        for i in (0..children.len()).rev() {
            if children[i].kind == SyntaxKind::ElseKeyword {
                keywords.push(children[i].clone());
                break;
            }
        }
        let else_statement = match &if_statement.data {
            ast::NodeData::IfStatement(data) => data.else_statement.clone(),
            _ => None,
        };
        match else_statement {
            Some(else_statement) if ast::is_if_statement(&else_statement) => {
                if_statement = else_statement;
            }
            _ => break,
        }
    }
    keywords
}

pub fn get_return_occurrences(node: &Arc<Node>, source_file: &Arc<SourceFile>) -> Vec<Arc<Node>> {
    let func_node = match node.parent().and_then(|p| ast::find_ancestor(&p, ast::is_function_like))
    {
        Some(f) => f,
        None => return Vec::new(),
    };

    let mut keywords: Vec<Arc<Node>> = Vec::new();
    if let Some(body) = func_node.body() {
        ast::mig::m3e_4::for_each_return_statement(body, |ret| {
            if let Some(keyword) = astnav::find_child_of_kind(ret, SyntaxKind::ReturnKeyword) {
                keywords.push(keyword);
            }
            false
        });

        let throw_statements = aggregate_owned_throw_statements(body, source_file);
        for throw in throw_statements {
            if let Some(keyword) = astnav::find_child_of_kind(&throw, SyntaxKind::ThrowKeyword) {
                keywords.push(keyword);
            }
        }
    }
    keywords
}

pub fn aggregate_owned_throw_statements(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
) -> Vec<Arc<Node>> {
    if ast::is_throw_statement(node) {
        return vec![node.clone()];
    }
    if ast::is_try_statement(node) {
        let (try_block, catch_clause, finally_block) = match &node.data {
            ast::NodeData::TryStatement(data) => (
                Some(Arc::clone(&data.try_block)),
                data.catch_clause.clone(),
                data.finally_block.clone(),
            ),
            _ => (None, None, None),
        };

        let mut result = Vec::new();
        if let Some(catch_clause) = catch_clause {
            result = aggregate_owned_throw_statements(&catch_clause, source_file);
        } else if let Some(try_block) = try_block {
            result = aggregate_owned_throw_statements(&try_block, source_file);
        }
        if let Some(finally_block) = finally_block {
            result.extend(aggregate_owned_throw_statements(&finally_block, source_file));
        }
        return result;
    }
    if ast::is_function_like(node) {
        return Vec::new();
    }
    flat_map_children(node, source_file, |child, sf| {
        aggregate_owned_throw_statements(child, sf)
    })
}

pub fn flat_map_children<T>(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    mut cb: impl FnMut(&Arc<Node>, &Arc<SourceFile>) -> Vec<T>,
) -> Vec<T> {
    let mut result = Vec::new();
    tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
        let value = cb(child, source_file);
        result.extend(value);
        false
    });
    result
}

pub fn get_throw_occurrences(node: &Arc<Node>, source_file: &Arc<SourceFile>) -> Vec<Arc<Node>> {
    let owner = match get_throw_statement_owner(node) {
        Some(owner) => owner,
        None => return Vec::new(),
    };

    let mut keywords: Vec<Arc<Node>> = Vec::new();

    let throw_statements = aggregate_owned_throw_statements(&owner, source_file);
    for throw in throw_statements {
        if let Some(keyword) = astnav::find_child_of_kind(&throw, SyntaxKind::ThrowKeyword) {
            keywords.push(keyword);
        }
    }

    if ast::is_function_block(&owner) {
        ast::mig::m3e_4::for_each_return_statement(&owner, |ret| {
            if let Some(keyword) = astnav::find_child_of_kind(ret, SyntaxKind::ReturnKeyword) {
                keywords.push(keyword);
            }
            false
        });
    }

    keywords
}

pub fn get_throw_statement_owner(throw_statement: &Arc<Node>) -> Option<Arc<Node>> {
    let mut child = throw_statement.clone();
    while let Some(parent) = child.parent() {
        if ast::is_function_block(&parent) || parent.kind == SyntaxKind::SourceFile {
            return Some(parent);
        }

        if ast::is_try_statement(&parent) {
            let (try_block, catch_clause) = match &parent.data {
                ast::NodeData::TryStatement(data) => (
                    Some(Arc::clone(&data.try_block)),
                    data.catch_clause.clone(),
                ),
                _ => (None, None),
            };
            if try_block
                .map(|tb| Arc::ptr_eq(&tb, &child))
                .unwrap_or(false)
                && catch_clause.is_some()
            {
                return Some(child);
            }
        }

        child = parent;
    }
    None
}

pub fn get_try_catch_finally_occurrences(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
) -> Vec<Arc<Node>> {
    let mut keywords: Vec<Arc<Node>> = Vec::new();
    let token = crate::ls::lsutil_children::get_first_token(node, source_file);
    if let Some(token) = &token {
        if token.kind == SyntaxKind::TryKeyword {
            keywords.push(token.clone());
        }
    }

    let (catch_clause, finally_block) = match &node.data {
        ast::NodeData::TryStatement(data) => {
            (data.catch_clause.clone(), data.finally_block.clone())
        }
        _ => (None, None),
    };
    if catch_clause.is_some() {
        if let Some(catch_token) = astnav::find_child_of_kind(node, SyntaxKind::CatchKeyword) {
            keywords.push(catch_token);
        }
    }

    if finally_block.is_some() {
        if let Some(finally_keyword) = astnav::find_child_of_kind(node, SyntaxKind::FinallyKeyword)
        {
            keywords.push(finally_keyword);
        }
    }

    keywords
}

pub fn get_switch_case_default_occurrences(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
) -> Vec<Arc<Node>> {
    let mut keywords: Vec<Arc<Node>> = Vec::new();
    if let Some(token) = crate::ls::lsutil_children::get_first_token(node, source_file) {
        if token.kind == SyntaxKind::SwitchKeyword {
            keywords.push(token);
        }
    }

    let case_block = match &node.data {
        ast::NodeData::SwitchStatement(data) => Some(Arc::clone(&data.case_block)),
        _ => None,
    };
    if let Some(case_block) = case_block {
        let clauses: Vec<Arc<Node>> = match &case_block.data {
            ast::NodeData::CaseBlock(data) => data.clauses.nodes.clone(),
            _ => Vec::new(),
        };
        for clause in clauses {
            if let Some(clause_token) =
                crate::ls::lsutil_children::get_first_token(&clause, source_file)
            {
                if clause_token.kind == SyntaxKind::CaseKeyword
                    || clause_token.kind == SyntaxKind::DefaultKeyword
                {
                    keywords.push(clause_token.clone());
                }
            }

            let break_and_continue_statements =
                aggregate_all_break_and_continue_statements(&clause, source_file);
            for statement in break_and_continue_statements {
                if statement.kind == SyntaxKind::BreakStatement
                    && owns_break_or_continue_statement(node, &statement)
                {
                    if let Some(token) =
                        crate::ls::lsutil_children::get_first_token(&statement, source_file)
                    {
                        keywords.push(token);
                    }
                }
            }
        }
    }

    keywords
}

pub fn aggregate_all_break_and_continue_statements(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
) -> Vec<Arc<Node>> {
    if ast::is_break_or_continue_statement(node) {
        return vec![node.clone()];
    }
    if ast::is_function_like(node) {
        return Vec::new();
    }
    flat_map_children(node, source_file, |child, sf| {
        aggregate_all_break_and_continue_statements(child, sf)
    })
}

pub fn owns_break_or_continue_statement(owner: &Arc<Node>, statement: &Arc<Node>) -> bool {
    match get_break_or_continue_owner(statement) {
        Some(actual_owner) => Arc::ptr_eq(&actual_owner, owner),
        None => false,
    }
}

pub fn get_break_or_continue_owner(statement: &Arc<Node>) -> Option<Arc<Node>> {
    ast::mig::m3e_4::find_ancestor_or_quit(Some(statement), |node| {
        match node.kind {
            SyntaxKind::SwitchStatement => {
                if statement.kind == SyntaxKind::ContinueStatement {
                    return ast::mig::m3e_4::FindAncestorResult::False;
                }
                fallthrough_check(statement, node)
            }
            SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::WhileStatement
            | SyntaxKind::DoStatement => fallthrough_check(statement, node),
            _ => {
                if ast::is_function_like(node) {
                    return ast::mig::m3e_4::FindAncestorResult::Quit;
                }
                ast::mig::m3e_4::FindAncestorResult::False
            }
        }
    })
}

fn fallthrough_check(statement: &Arc<Node>, node: &Arc<Node>) -> ast::mig::m3e_4::FindAncestorResult {
    let label = tsox_frontend::ast::mig::m3b::label(statement);
    if label.is_none()
        || label.is_some_and(|l| is_labeled_by(node, l.text()))
    {
        return ast::mig::m3e_4::FindAncestorResult::True;
    }
    ast::mig::m3e_4::FindAncestorResult::False
}

pub fn is_labeled_by(node: &Arc<Node>, label_name: &str) -> bool {
    ast::mig::m3e_4::find_ancestor_or_quit(node.parent().as_ref(), |owner| {
        if !ast::is_labeled_statement(owner) {
            return ast::mig::m3e_4::FindAncestorResult::Quit;
        }
        if let Some(label) = tsox_frontend::ast::mig::m3b::label(owner) {
            if label.text() == label_name {
                return ast::mig::m3e_4::FindAncestorResult::True;
            }
        }
        ast::mig::m3e_4::FindAncestorResult::False
    })
    .is_some()
}

pub fn get_break_or_continue_statement_occurrences(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
) -> Vec<Arc<Node>> {
    if let Some(owner) = get_break_or_continue_owner(node) {
        match owner.kind {
            SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::WhileStatement => return get_loop_break_continue_occurrences(&owner, source_file),
            SyntaxKind::SwitchStatement => return get_switch_case_default_occurrences(&owner, source_file),
            _ => {}
        }
    }
    Vec::new()
}

pub fn get_loop_break_continue_occurrences(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
) -> Vec<Arc<Node>> {
    let mut keywords: Vec<Arc<Node>> = Vec::new();

    if let Some(token) = crate::ls::lsutil_children::get_first_token(node, source_file) {
        if token.kind == SyntaxKind::ForKeyword
            || token.kind == SyntaxKind::DoKeyword
            || token.kind == SyntaxKind::WhileKeyword
        {
            keywords.push(token);
            if node.kind == SyntaxKind::DoStatement {
                let loop_tokens = crate::ls::utilities::get_children_from_non_jsdoc_node(node, source_file);
                for i in (0..loop_tokens.len()).rev() {
                    if loop_tokens[i].kind == SyntaxKind::WhileKeyword {
                        keywords.push(loop_tokens[i].clone());
                        break;
                    }
                }
            }
        }
    }

    let break_and_continue_statements = aggregate_all_break_and_continue_statements(node, source_file);
    for statement in break_and_continue_statements {
        if let Some(token) = crate::ls::lsutil_children::get_first_token(&statement, source_file) {
            if owns_break_or_continue_statement(node, &statement)
                && (token.kind == SyntaxKind::BreakKeyword
                    || token.kind == SyntaxKind::ContinueKeyword)
            {
                keywords.push(token);
            }
        }
    }

    keywords
}

pub fn get_async_and_await_occurrences(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
) -> Vec<Arc<Node>> {
    let fun = match ast::mig::x4ast::get_containing_function(node) {
        Some(fun) => fun,
        None => return Vec::new(),
    };

    let mut keywords: Vec<Arc<Node>> = Vec::new();

    for modifier in fun.modifier_nodes() {
        if modifier.kind == SyntaxKind::AsyncKeyword {
            keywords.push(Arc::clone(modifier));
        }
    }

    tsox_frontend::ast::node_data_generated::for_each_child(&fun, |child| {
        traverse_without_crossing_function(child, source_file, &mut |child| {
            if ast::is_await_expression(child) {
                if let Some(token) = crate::ls::lsutil_children::get_first_token(child, source_file) {
                    if token.kind == SyntaxKind::AwaitKeyword {
                        keywords.push(token);
                    }
                }
            }
        });
        false
    });

    keywords
}

pub fn get_yield_occurrences(node: &Arc<Node>, source_file: &Arc<SourceFile>) -> Vec<Arc<Node>> {
    let parent_func = match node.parent().and_then(|p| ast::find_ancestor(&p, ast::is_function_like))
    {
        Some(f) => f,
        None => return Vec::new(),
    };

    let mut keywords: Vec<Arc<Node>> = Vec::new();

    tsox_frontend::ast::node_data_generated::for_each_child(&parent_func, |child| {
        traverse_without_crossing_function(child, source_file, &mut |child| {
            if ast::is_yield_expression(child) {
                if let Some(token) = crate::ls::lsutil_children::get_first_token(child, source_file) {
                    if token.kind == SyntaxKind::YieldKeyword {
                        keywords.push(token);
                    }
                }
            }
        });
        false
    });

    keywords
}

pub fn traverse_without_crossing_function(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    cb: &mut impl FnMut(&Arc<Node>),
) {
    cb(node);
    if !ast::is_function_like(node)
        && !ast::is_class_like(node)
        && !ast::is_interface_declaration(node)
        && !ast::is_module_declaration(node)
        && !ast::is_type_alias_declaration(node)
        && !ast::is_type_node(node)
    {
        tsox_frontend::ast::node_data_generated::for_each_child(node, |child| {
            traverse_without_crossing_function(child, source_file, cb);
            false
        });
    }
}

pub fn get_modifier_occurrences(
    kind: SyntaxKind,
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
) -> Vec<Arc<Node>> {
    let mut result = Vec::new();

    let nodes_to_search = get_nodes_to_search_for_modifier(node, ast::modifier_to_flag(kind));
    for n in nodes_to_search {
        if let Some(modifier) = find_modifier(&n, kind) {
            result.push(modifier);
        }
    }
    result
}

pub fn get_nodes_to_search_for_modifier(
    declaration: &Arc<Node>,
    modifier_flag: ast::ModifierFlags,
) -> Vec<Arc<Node>> {
    let container = declaration.parent();
    let Some(container) = container else {
        return Vec::new();
    };

    match container.kind {
        SyntaxKind::ModuleBlock
        | SyntaxKind::SourceFile
        | SyntaxKind::Block
        | SyntaxKind::CaseClause
        | SyntaxKind::DefaultClause => {
            if modifier_flag.contains(ast::ModifierFlags::Abstract) && ast::is_class_declaration(declaration) {
                let mut result: Vec<Arc<Node>> = tsox_frontend::ast::mig::m3b::members(declaration).to_vec();
                result.push(declaration.clone());
                result
            } else {
                tsox_frontend::ast::mig::m3c::statements(&container).to_vec()
            }
        }
        SyntaxKind::Constructor | SyntaxKind::MethodDeclaration | SyntaxKind::FunctionDeclaration => {
            let mut result: Vec<Arc<Node>> = tsox_frontend::ast::mig::m3b::parameters(&container).to_vec();
            if container
                .parent()
                .as_deref()
                .map(ast::is_class_like)
                .unwrap_or(false)
            {
                let container_parent = container.parent().unwrap();
                result.extend(tsox_frontend::ast::mig::m3b::members(&container_parent).to_vec());
            }
            result
        }
        SyntaxKind::ClassDeclaration
        | SyntaxKind::ClassExpression
        | SyntaxKind::InterfaceDeclaration
        | SyntaxKind::TypeLiteral => {
            let nodes = tsox_frontend::ast::mig::m3b::members(&container).to_vec();
            let mut result: Vec<Arc<Node>> = nodes.clone();
            if modifier_flag
                .intersects(ast::ModifierFlags::AccessibilityModifier | ast::ModifierFlags::Readonly)
            {
                let mut constructor: Option<Arc<Node>> = None;
                for member in &nodes {
                    if ast::is_constructor_declaration(member) {
                        constructor = Some(member.clone());
                        break;
                    }
                }
                if let Some(constructor) = constructor {
                    result.extend(tsox_frontend::ast::mig::m3b::parameters(&constructor).to_vec());
                }
            } else if modifier_flag.contains(ast::ModifierFlags::Abstract) {
                result.push(container.clone());
            }
            result
        }
        _ => Vec::new(),
    }
}

pub fn find_modifier(node: &Arc<Node>, kind: SyntaxKind) -> Option<Arc<Node>> {
    for modifier in node.modifier_nodes() {
        if modifier.kind == kind {
            return Some(Arc::clone(modifier));
        }
    }
    None
}
