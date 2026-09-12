//! Go span.go 头部的 findEnclosingNode/getScanStartPosition/
//! getOwnOrInheritedDelta 与 indent.go 的 GetIndentationForNode。

use std::sync::Arc;

use tsox_core::core::text::TextRange;

use crate::ast::node::Node;
use crate::ast::SourceFile;
use crate::format::FormatCodeSettings;

use super::indenter::should_indent_child_node;
use super::util;

/// Go findEnclosingNode：完全包含给定 range 的最小节点。
pub(crate) fn find_enclosing_node(r: TextRange, file: &SourceFile) -> Arc<Node> {
    fn find(n: &Arc<Node>, r: TextRange, file: &SourceFile) -> Arc<Node> {
        let mut candidate: Option<Arc<Node>> = None;
        crate::ast::node_data_generated::for_each_child(n, |c| {
            if c.flags.contains(crate::ast::NodeFlags::Reparsed) {
                return false;
            }
            if r.contained_by(&util::with_token_start(file, c)) {
                candidate = Some(Arc::clone(c));
                return true;
            }
            false
        });
        // Go：候选存在即递归；否则 n 自身就是包含节点
        match candidate {
            Some(c) => find(&c, r, file),
            None => Arc::clone(n),
        }
    }
    find(&file.node, r, file)
}

/// Go getScanStartPosition：range 起点可能落在注释里，scanner 从
/// 前	token 的末尾起扫。
pub(crate) fn get_scan_start_position(
    enclosing_node: &Arc<Node>,
    original_range: TextRange,
    file: &SourceFile,
) -> usize {
    let start = util::with_token_start(file, enclosing_node).pos();
    if start == original_range.pos() && enclosing_node.end() == original_range.end() {
        return start;
    }

    let preceding_token =
        crate::astnav::find_preceding_token(&file.node, original_range.pos());
    let Some(preceding_token) = preceding_token else {
        return enclosing_node.pos();
    };

    if preceding_token.end() >= original_range.pos() {
        return enclosing_node.pos();
    }

    preceding_token.end()
}

/// Go getOwnOrInheritedDelta
pub(crate) fn get_own_or_inherited_delta(
    n: &Arc<Node>,
    options: &FormatCodeSettings,
    file: &SourceFile,
) -> i64 {
    let mut previous_line: i64 = -1;
    let mut child: Option<Arc<Node>> = None;
    let mut current = Some(Arc::clone(n));
    while let Some(node) = current {
        let line =
            util::line_of_position(file, util::with_token_start(file, &node).pos());
        if previous_line != -1 && line as i64 != previous_line {
            break;
        }

        if should_indent_child_node(options, &node, child.as_ref(), Some(file), false) {
            return options.editor_settings.indent_size as i64;
        }

        previous_line = line as i64;
        child = Some(Arc::clone(&node));
        current = node.parent();
    }
    0
}

/// Go GetIndentationForNode：span 入口的 initialIndentation。
pub(crate) fn get_indentation_for_node(
    n: &Arc<Node>,
    ignore_actual_indentation_range: &TextRange,
    file: &SourceFile,
    options: &FormatCodeSettings,
) -> i64 {
    let token_pos = util::token_pos_of_node(file, n);
    let (startline, startpos) = util::line_and_byte_offset_of_position(file, token_pos);
    get_indentation_for_node_worker(
        n,
        startline,
        startpos,
        Some(ignore_actual_indentation_range),
        0,
        file,
        false,
        options,
    )
}

/// Go getIndentationForNodeWorker
pub(crate) fn get_indentation_for_node_worker(
    current: &Arc<Node>,
    current_start_line: usize,
    current_start_character: usize,
    ignore_actual_indentation_range: Option<&TextRange>,
    mut indentation_delta: i64,
    file: &SourceFile,
    is_next_child: bool,
    options: &FormatCodeSettings,
) -> i64 {
    
    let mut current = Arc::clone(current);
    let mut current_start_line = current_start_line;
    let mut current_start_character = current_start_character;
    let mut parent = current.parent();

    while let Some(p) = parent {
        let use_actual_indentation = match ignore_actual_indentation_range {
            None => true,
            Some(range) => {
                let start = util::token_pos_of_node(file, &current);
                start < range.pos() || start > range.end()
            }
        };

        let (containing_list_or_parent_start_line, containing_list_or_parent_start_character) =
            get_containing_list_or_parent_start(&p, &current, file);
        let parent_and_child_share_line = containing_list_or_parent_start_line == current_start_line
            || super::indenter::child_starts_on_same_line_with_else_in_if_statement(
                &p, &current, current_start_line, file,
            );

        if use_actual_indentation {
            // 当前列表项缩进优先
            let container_list = super::lists::get_containing_list(&current, file);
            let first_list_child: Option<Arc<Node>> = container_list
                .as_ref()
                .and_then(|(_, list)| list.nodes.first().cloned());
            let mut list_indents_child = false;
            if let Some(first) = &first_list_child {
                let list_line = util::line_of_position(
                    file,
                    util::token_pos_of_node(file, first),
                );
                list_indents_child = list_line > containing_list_or_parent_start_line;
            }
            let actual_indentation = get_actual_indentation_for_list_item(
                &current,
                file,
                options,
                list_indents_child,
            );
            if actual_indentation != -1 {
                return actual_indentation + indentation_delta;
            }

            let actual_indentation = get_actual_indentation_for_node(
                &current,
                &p,
                current_start_line,
                current_start_character,
                parent_and_child_share_line,
                file,
                options,
            );
            if actual_indentation != -1 {
                return actual_indentation + indentation_delta;
            }
        }

        if should_indent_child_node(options, &p, Some(&current), Some(file), is_next_child)
            && !parent_and_child_share_line
        {
            indentation_delta += options.editor_settings.indent_size as i64;
        }

        let use_true_start = is_argument_and_start_line_overlaps_expression_being_called(
            &p, &current, current_start_line, file,
        );

        let next = p.clone();
        parent = next.parent();
        current = next;

        if use_true_start {
            let pos = util::token_pos_of_node(file, &current);
            let (line, ch) = util::line_and_byte_offset_of_position(file, pos);
            current_start_line = line;
            current_start_character = ch;
        } else {
            current_start_line = containing_list_or_parent_start_line;
            current_start_character = containing_list_or_parent_start_character;
        }
    }

    indentation_delta + options.editor_settings.base_indent_size as i64
}

/// Go getContainingListOrParentStart
fn get_containing_list_or_parent_start(
    parent: &Arc<Node>,
    child: &Arc<Node>,
    file: &SourceFile,
) -> (usize, usize) {
    let start_pos = match super::lists::get_containing_list(child, file) {
        Some((_, list)) => list.loc.pos(),
        None => util::token_pos_of_node(file, parent),
    };
    util::line_and_byte_offset_of_position(file, start_pos)
}

/// Go getActualIndentationForNode
fn get_actual_indentation_for_node(
    current: &Arc<Node>,
    parent: &Arc<Node>,
    current_line: usize,
    current_char: usize,
    parent_and_child_share_line: bool,
    file: &SourceFile,
    options: &FormatCodeSettings,
) -> i64 {
    use crate::ast::SyntaxKind;
    let is_declaration = matches!(
        current.kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::VariableStatement
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ExportAssignment
            | SyntaxKind::NamespaceExportDeclaration
            | SyntaxKind::MissingDeclaration
    );
    let is_statement = !is_declaration
        && matches!(
            current.kind,
            SyntaxKind::BreakStatement
                | SyntaxKind::ContinueStatement
                | SyntaxKind::DebuggerStatement
                | SyntaxKind::DoStatement
                | SyntaxKind::ExpressionStatement
                | SyntaxKind::EmptyStatement
                | SyntaxKind::ForInStatement
                | SyntaxKind::ForOfStatement
                | SyntaxKind::ForStatement
                | SyntaxKind::IfStatement
                | SyntaxKind::LabeledStatement
                | SyntaxKind::ReturnStatement
                | SyntaxKind::SwitchStatement
                | SyntaxKind::ThrowStatement
                | SyntaxKind::TryStatement
                | SyntaxKind::VariableStatement
                | SyntaxKind::WhileStatement
                | SyntaxKind::WithStatement
                | SyntaxKind::Block
        );
    let use_actual_indentation =
        (is_declaration || is_statement) && (parent.kind == SyntaxKind::SourceFile || !parent_and_child_share_line);
    if !use_actual_indentation {
        return -1;
    }
    find_column_for_first_non_whitespace_character_in_line(current_line, current_char, file, options)
}

/// Go isArgumentAndStartLineOverlapsExpressionBeingCalled
fn is_argument_and_start_line_overlaps_expression_being_called(
    parent: &Arc<Node>,
    child: &Arc<Node>,
    child_start_line: usize,
    file: &SourceFile,
) -> bool {
    if parent.kind != crate::ast::SyntaxKind::CallExpression {
        return false;
    }
    let crate::ast::node_data_generated::NodeData::CallExpression(d) = &parent.data else {
        return false;
    };
    if !d.arguments.nodes.iter().any(|a| Arc::ptr_eq(a, child)) {
        return false;
    }
    let expression_end_line = util::line_of_position(file, d.expression.end());
    expression_end_line == child_start_line
}

/// Go getActualIndentationForListItem
pub(crate) fn get_actual_indentation_for_list_item(
    node: &Arc<Node>,
    file: &SourceFile,
    options: &FormatCodeSettings,
    list_indents_child: bool,
) -> i64 {
    use crate::ast::SyntaxKind;
    if let Some(parent) = node.parent() {
        if parent.kind == SyntaxKind::VariableDeclarationList {
            return -1;
        }
    }
    let Some((_, containing_list)) = super::lists::get_containing_list(node, file) else {
        return -1;
    };
    let index = containing_list
        .nodes
        .iter()
        .position(|n| Arc::ptr_eq(n, node));
    if let Some(index) = index {
        let result = derive_actual_indentation_from_list(&containing_list, index, file, options);
        if result != -1 {
            return result;
        }
    }
    let delta: i64 = if list_indents_child { options.editor_settings.indent_size as i64 } else { 0 };
    let res = get_actual_indentation_for_list_start_line(Some(&containing_list), file, options);
    if res == -1 {
        return delta;
    }
    res + delta
}

/// Go getActualIndentationForListStartLine
pub(crate) fn get_actual_indentation_for_list_start_line(
    list: Option<&Arc<crate::ast::node::NodeList>>,
    file: &SourceFile,
    options: &FormatCodeSettings,
) -> i64 {
    let Some(list) = list else { return -1 };
    let (line, char) = util::line_and_byte_offset_of_position(file, list.loc.pos());
    find_column_for_first_non_whitespace_character_in_line(line, char, file, options)
}

/// Go deriveActualIndentationFromList
fn derive_actual_indentation_from_list(
    list: &Arc<crate::ast::node::NodeList>,
    index: usize,
    file: &SourceFile,
    options: &FormatCodeSettings,
) -> i64 {
    let node = &list.nodes[index];
    let (mut line, mut char) = util::line_and_byte_offset_of_position(
        file,
        util::token_pos_of_node(file, node),
    );
    for i in (0..=index).rev() {
        let item = &list.nodes[i];
        if item.kind == crate::ast::SyntaxKind::CommaToken {
            continue;
        }
        let prev_end_line = util::line_of_position(file, item.end());
        if prev_end_line != line {
            return find_column_for_first_non_whitespace_character_in_line(
                line, char, file, options,
            );
        }
        let pos = util::token_pos_of_node(file, item);
        let (l, c) = util::line_and_byte_offset_of_position(file, pos);
        line = l;
        char = c;
    }
    -1
}

/// Go findColumnForFirstNonWhitespaceCharacterInLine
fn find_column_for_first_non_whitespace_character_in_line(
    line: usize,
    char: usize,
    file: &SourceFile,
    options: &FormatCodeSettings,
) -> i64 {
    let line_start = util::position_of_line_and_byte_offset(file, line, 0);
    util::find_first_non_whitespace_column(file, line_start, line_start + char, options.editor_settings.tab_size)
        as i64
}
