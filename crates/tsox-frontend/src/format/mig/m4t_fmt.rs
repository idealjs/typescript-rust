use crate::ast::node::{Node, SourceFile, LanguageVariant};
use crate::ast::syntax_kind_generated::SyntaxKind;
use crate::ast::utilities_navigation::find_ancestor;
use crate::astnav::{get_start_of_node, get_token_at_position, find_next_token};
use crate::format::rule_context::FormatRequestKind;
use crate::format::indenter::should_indent_child_node;
use crate::format::indentation::{
    derive_actual_indentation_from_list, get_actual_indentation_for_list_item,
    get_indentation_for_node_worker,
};
use crate::format::lists::{get_list_by_range, Field};
use crate::format::util::{
    find_first_non_whitespace_character_and_column, find_first_non_whitespace_column,
    line_start_position_for_position,
};
use crate::format::{FormatCodeSettings, FormatContext, TextChange};
use crate::scanner::CommentRange;
use crate::scanner::is_jsx_line_break::CommentRangeKind;
use crate::scanner::{get_leading_comment_ranges, get_trailing_comment_ranges};
use crate::scanner::mig::x5a::get_token_pos_of_node;
use std::sync::Arc;
use tsox_core::core::text::TextRange;
use tsox_core::stringutil::is_white_space_like;

pub fn format_node_given_indentation(
    ctx: &FormatContext,
    node: &Arc<Node>,
    file: &Arc<SourceFile>,
    language_variant: LanguageVariant,
    initial_indentation: i64,
    delta: i64,
) -> Vec<TextChange> { ::tsox_core::fntrace::enter("format_node_given_indentation"); 
    let text_range = TextRange::new(node.pos(), node.end());
    crate::format::span::format_span(
        file,
        text_range,
        ctx.settings.clone(),
        &ctx.new_line_character,
        FormatRequestKind::FormatSelection,
        Box::new(|_r: TextRange| false),
        node.clone(),
        initial_indentation,
        delta,
        text_range.pos(),
    )
}

pub fn get_actual_indentation_for_list_item_before_comma(
    comma_token: &Arc<Node>,
    source_file: &SourceFile,
    options: &FormatCodeSettings,
) -> i64 { ::tsox_core::fntrace::enter("get_actual_indentation_for_list_item_before_comma"); 
    let parent = match comma_token.parent() {
        Some(p) => p,
        None => return -1,
    };
    let containing_list = get_list_by_range(
        get_token_pos_of_node(comma_token, source_file, false),
        comma_token.end(),
        &parent,
        source_file,
    );
    let Some((_, containing_list)) = containing_list else {
        return -1;
    };
    let comma_index = containing_list
        .nodes
        .iter()
        .position(|n| Arc::ptr_eq(n, comma_token));
    if let Some(comma_index) = comma_index {
        if comma_index > 0 {
            return derive_actual_indentation_from_list(&containing_list, comma_index - 1, source_file, options);
        }
    }
    -1
}

pub fn get_comment_indent(
    source_file: &SourceFile,
    position: usize,
    options: &FormatCodeSettings,
    enclosing_comment_range: &CommentRange,
) -> i64 { ::tsox_core::fntrace::enter("get_comment_indent"); 
    let tab_size = options.editor_settings.tab_size;
    let previous_line = get_ecma_line_of_position(source_file, position) as i64 - 1;
    let comment_start_line = get_ecma_line_of_position(source_file, enclosing_comment_range.pos) as i64;

    debug_assert!(comment_start_line >= 0);

    let line_starts = get_ecma_line_starts(source_file);
    if previous_line <= comment_start_line {
        return find_first_non_whitespace_column(
            source_file,
            line_starts[comment_start_line as usize],
            position,
            tab_size,
        ) as i64;
    }

    let start_position_of_line = line_starts[previous_line as usize];
    let (character, column) = find_first_non_whitespace_character_and_column(
        source_file,
        start_position_of_line,
        position,
        tab_size,
    );

    if column == 0 {
        return column as i64;
    }

    let first_non_whitespace_character_code = source_file.text.as_bytes()
        [start_position_of_line + character];
    if first_non_whitespace_character_code == b'*' {
        return column as i64 - 1;
    }
    column as i64
}

pub fn get_leading_comment_ranges_of_node(
    node: &Arc<Node>,
    file: &SourceFile,
) -> Vec<CommentRange> { ::tsox_core::fntrace::enter("get_leading_comment_ranges_of_node"); 
    if node.kind == SyntaxKind::JsxText {
        return Vec::new();
    }
    get_leading_comment_ranges(&file.text, node.pos())
}

pub fn get_range_of_enclosing_comment(
    source_file: &SourceFile,
    position: usize,
    preceding_token: Option<&Arc<Node>>,
) -> Option<CommentRange> { ::tsox_core::fntrace::enter("get_range_of_enclosing_comment"); 
    let token_at_position = get_token_at_position(&source_file.node, position)?;
    let mut token_at_position = token_at_position;
    let jsdoc = find_ancestor(&token_at_position, |n| n.kind == SyntaxKind::JSDoc);
    if let Some(jsdoc) = jsdoc {
        if let Some(parent) = jsdoc.parent() {
            token_at_position = parent;
        }
    }
    let token_start = get_start_of_node(&token_at_position, source_file, false);
    if token_start <= position && position < token_at_position.end() {
        return None;
    }

    let trailing_ranges_of_previous_token: Vec<CommentRange> = match preceding_token {
        Some(preceding_token) => {
            get_trailing_comment_ranges(&source_file.text, preceding_token.end())
        }
        None => Vec::new(),
    };
    let leading_ranges_of_next_token =
        get_leading_comment_ranges_of_node(&token_at_position, source_file);
    let mut comment_ranges = trailing_ranges_of_previous_token;
    comment_ranges.extend(leading_ranges_of_next_token);
    for comment_range in comment_ranges {
        let is_exclusive = comment_range.pos < position && position < comment_range.end;
        if is_exclusive
            || position == comment_range.end
                && (comment_range.kind == CommentRangeKind::SingleLine
                    || position == source_file.text.len())
        {
            return Some(comment_range);
        }
    }
    None
}

pub fn get_block_indent(
    source_file: &SourceFile,
    position: usize,
    options: &FormatCodeSettings,
) -> i64 { ::tsox_core::fntrace::enter("get_block_indent"); 
    let mut current = position;
    while current > 0 {
        let ch = source_file.text[current..].chars().next().unwrap_or('\u{FFFD}');
        let size = ch.len_utf8();
        if !is_white_space_like(ch) {
            break;
        }
        current -= size;
    }

    let line_start = line_start_position_for_position(source_file, current);
    find_first_non_whitespace_column(
        source_file,
        line_start,
        current,
        options.editor_settings.tab_size,
    ) as i64
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NextTokenKind {
    Unknown,
    OpenBrace,
    CloseBrace,
}

pub fn next_token_is_curly_brace_on_same_line_as_cursor(
    preceding_token: &Arc<Node>,
    current: &Arc<Node>,
    line_at_position: usize,
    source_file: &SourceFile,
) -> NextTokenKind { ::tsox_core::fntrace::enter("next_token_is_curly_brace_on_same_line_as_cursor"); 
    let next_token = find_next_token(&source_file.node, current.end());
    let Some(next_token) = next_token else {
        return NextTokenKind::Unknown;
    };

    if next_token.kind == SyntaxKind::OpenBraceToken {
        return NextTokenKind::OpenBrace;
    } else if next_token.kind == SyntaxKind::CloseBraceToken {
        let next_token_start_line = get_start_line_for_node(&next_token, source_file);
        if line_at_position == next_token_start_line {
            return NextTokenKind::CloseBrace;
        }
        return NextTokenKind::Unknown;
    }

    NextTokenKind::Unknown
}

fn position_belongs_to_node(node: &Arc<Node>, position: usize, source_file: &SourceFile) -> bool { ::tsox_core::fntrace::enter("position_belongs_to_node"); 
    get_start_of_node(node, source_file, true) <= position && position < node.end()
}

pub fn get_smart_indent(
    source_file: &SourceFile,
    position: usize,
    preceding_token: &Arc<Node>,
    line_at_position: usize,
    assume_new_line_before_close_brace: bool,
    options: &FormatCodeSettings,
) -> i64 { ::tsox_core::fntrace::enter("get_smart_indent"); 
    let mut previous: Option<Arc<Node>> = None;
    let mut current = Some(preceding_token.clone());

    while let Some(cur) = current.clone() {
        if position_belongs_to_node(&cur, position, source_file)
            && should_indent_child_node(options, &cur, previous.as_ref(), Some(source_file), true)
        {
            let (current_start_line, current_start_char) =
                get_start_line_and_character_for_node(&cur, source_file);
            let ntk = next_token_is_curly_brace_on_same_line_as_cursor(
                preceding_token,
                &cur,
                line_at_position,
                source_file,
            );
            let mut indentation_delta = 0;
            if ntk != NextTokenKind::Unknown {
                if assume_new_line_before_close_brace && ntk == NextTokenKind::CloseBrace {
                    indentation_delta = options.editor_settings.indent_size as i64;
                }
            } else if line_at_position != current_start_line {
                indentation_delta = options.editor_settings.indent_size as i64;
            }
            return get_indentation_for_node_worker(
                &cur,
                current_start_line,
                current_start_char,
                None,
                indentation_delta,
                source_file,
                true,
                options,
            );
        }

        let actual_indentation =
            get_actual_indentation_for_list_item(&cur, source_file, options, true);
        if actual_indentation != -1 {
            return actual_indentation;
        }

        previous = Some(cur.clone());
        current = cur.parent();
    }
    options.editor_settings.base_indent_size as i64
}

pub fn get_start_line_and_character_for_node(
    n: &Arc<Node>,
    source_file: &SourceFile,
) -> (usize, usize) { ::tsox_core::fntrace::enter("get_start_line_and_character_for_node"); 
    get_ecma_line_and_byte_offset_of_position(
        source_file,
        get_token_pos_of_node(n, source_file, false),
    )
}

pub fn get_start_line_for_node(n: &Arc<Node>, source_file: &SourceFile) -> usize { ::tsox_core::fntrace::enter("get_start_line_for_node"); 
    get_ecma_line_of_position(source_file, get_token_pos_of_node(n, source_file, false))
}

pub fn get_list_by_position(
    pos: usize,
    node: Option<&Arc<Node>>,
    source_file: &SourceFile,
) -> Option<(Field, Arc<crate::ast::node::NodeList>)> { ::tsox_core::fntrace::enter("get_list_by_position"); 
    let node = node?;
    get_list_by_range(pos, pos, node, source_file)
}

fn get_ecma_line_of_position(source_file: &SourceFile, position: usize) -> usize { ::tsox_core::fntrace::enter("get_ecma_line_of_position"); 
    source_file.line_map.line_at(position)
}

fn get_ecma_line_starts(source_file: &SourceFile) -> Vec<usize> { ::tsox_core::fntrace::enter("get_ecma_line_starts"); 
    source_file
        .line_map
        .line_starts
        .iter()
        .map(|&s| s as usize)
        .collect()
}

fn get_ecma_line_and_byte_offset_of_position(
    source_file: &SourceFile,
    position: usize,
) -> (usize, usize) { ::tsox_core::fntrace::enter("get_ecma_line_and_byte_offset_of_position"); 
    let line = source_file.line_map.line_at(position);
    let line_start = source_file.line_map.line_starts[line] as usize;
    (line, position - line_start)
}
