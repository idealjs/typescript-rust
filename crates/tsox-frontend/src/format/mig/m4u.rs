use std::sync::Arc;

use crate::ast::node::Node;
use crate::ast::utilities_declarations::can_have_modifiers;
use crate::ast::utilities_navigation::get_name_of_declaration;
use crate::ast::NodeData;
use crate::ast::SyntaxKind;
use tsox_core::core::text::TextRange;

use crate::format::rule_context::{FormattingContext, Tristate};
use crate::format::scanner::FormattingScanner;
use crate::format::{FormatCodeSettings, SemicolonPreference, TextChange};
use crate::format::rule::TokenRange;

pub type OptionSelector = fn(&FormatCodeSettings) -> Tristate;
pub type BoolOptionSelector = fn(&FormatCodeSettings) -> bool;
pub type AnyOptionSelector<'a, T> = fn(&FormatCodeSettings) -> T;
pub type BoxedContextPredicate = Box<dyn Fn(&mut FormattingContext) -> bool>;

pub fn semicolon_option(options: &FormatCodeSettings) -> SemicolonPreference {
    options.semicolons
}

pub fn insert_space_after_opening_and_before_closing_nonempty_brackets_option(
    options: &FormatCodeSettings,
) -> Tristate {
    options.insert_space_after_opening_and_before_closing_nonempty_brackets
}

pub fn insert_space_after_opening_and_before_closing_nonempty_parenthesis_option(
    options: &FormatCodeSettings,
) -> Tristate {
    options.insert_space_after_opening_and_before_closing_nonempty_parenthesis
}

pub fn insert_space_after_opening_and_before_closing_template_string_braces_option(
    options: &FormatCodeSettings,
) -> Tristate {
    options.insert_space_after_opening_and_before_closing_template_string_braces
}

pub fn insert_space_after_semicolon_in_for_statements_option(
    options: &FormatCodeSettings,
) -> Tristate {
    options.insert_space_after_semicolon_in_for_statements
}

pub fn insert_space_after_type_assertion_option(options: &FormatCodeSettings) -> Tristate {
    options.insert_space_after_type_assertion
}

pub fn insert_space_before_and_after_binary_operators_option(
    options: &FormatCodeSettings,
) -> bool {
    options.insert_space_before_and_after_binary_operators
}

pub fn insert_space_before_function_parenthesis_option(
    options: &FormatCodeSettings,
) -> Tristate {
    options.insert_space_before_function_parenthesis
}

pub fn insert_space_before_type_annotation_option(options: &FormatCodeSettings) -> Tristate {
    options.insert_space_before_type_annotation
}

pub fn place_open_brace_on_new_line_for_control_blocks_option(
    options: &FormatCodeSettings,
) -> Tristate {
    options.place_open_brace_on_new_line_for_control_blocks
}

pub fn place_open_brace_on_new_line_for_functions_option(
    options: &FormatCodeSettings,
) -> Tristate {
    options.place_open_brace_on_new_line_for_functions
}

pub fn option_equals<T: PartialEq + Clone + 'static>(
    option_name: AnyOptionSelector<'static, T>,
    option_value: T,
) -> BoxedContextPredicate {
    Box::new(move |context: &mut FormattingContext| {
        option_name(&context.options) == option_value
    })
}

pub fn is_option_enabled(option_name: OptionSelector) -> BoxedContextPredicate {
    Box::new(move |context: &mut FormattingContext| {
        option_name(&context.options).is_true()
    })
}

pub fn is_option_disabled(option_name: OptionSelector) -> BoxedContextPredicate {
    Box::new(move |context: &mut FormattingContext| {
        option_name(&context.options).is_false()
    })
}

pub fn is_option_disabled_or_undefined(option_name: OptionSelector) -> BoxedContextPredicate {
    Box::new(move |context: &mut FormattingContext| {
        option_name(&context.options).is_false_or_unknown()
    })
}

pub fn is_option_disabled_or_undefined_or_tokens_on_same_line(
    option_name: OptionSelector,
) -> BoxedContextPredicate {
    Box::new(move |context: &mut FormattingContext| {
        option_name(&context.options).is_false_or_unknown() || context.tokens_are_on_same_line()
    })
}

pub fn is_option_enabled_or_undefined(option_name: OptionSelector) -> BoxedContextPredicate {
    Box::new(move |context: &mut FormattingContext| {
        option_name(&context.options).is_true_or_unknown()
    })
}

impl FormattingContext {
    pub fn is_function_call_context(&mut self) -> bool {
        self.context_node_kind() == SyntaxKind::CallExpression
    }

    pub fn is_new_context(&mut self) -> bool {
        self.context_node_kind() == SyntaxKind::NewExpression
    }

    pub fn is_semicolon_insertion_context(&mut self) -> bool {
        position_is_asi_candidate(
            self.current_token_span.loc.end(),
            self.current_token_parent.as_ref(),
            &self.source_file,
        )
    }
}

/// tsox-lsp::ls::lsutil_asi::position_is_asi_candidate 的本 crate 镜像
/// (lsp 侧当前为占位实现,返回 false;tsox-frontend 不能反向依赖 lsp)
fn position_is_asi_candidate(
    _pos: usize,
    _context: Option<&Arc<Node>>,
    _file: &Arc<crate::ast::SourceFile>,
) -> bool {
    false
}

pub fn starts_with_slash_token(t: SyntaxKind) -> bool {
    t == SyntaxKind::SlashToken || t == SyntaxKind::SlashEqualsToken
}

impl FormattingScanner {
    pub fn get_start_pos(&self) -> usize {
        self.get_token_full_start()
    }
}

pub const MASK_BIT_SIZE: usize = 5;
pub const MAP_ROW_LENGTH: usize = SyntaxKind::DeferKeyword as usize + 1;

pub fn get_rule_bucket_index(row: SyntaxKind, column: SyntaxKind) -> usize {
    debug_assert!(
        (row as usize) <= SyntaxKind::DeferKeyword as usize
            && (column as usize) <= SyntaxKind::DeferKeyword as usize
    );
    (row as usize) * MAP_ROW_LENGTH + column as usize
}

pub fn token_range_from_ex(prefix: &[SyntaxKind], tokens: &[SyntaxKind]) -> TokenRange {
    let mut all = Vec::with_capacity(prefix.len() + tokens.len());
    all.extend_from_slice(prefix);
    all.extend_from_slice(tokens);
    TokenRange { tokens: all, is_specific: true }
}

pub fn token_range_from_range(start: SyntaxKind, end: SyntaxKind) -> TokenRange {
    let mut tokens = Vec::with_capacity(end as usize - start as usize + 1);
    let mut token = start as i16;
    while token <= end as i16 {
        tokens.push(syntax_kind_from_i16(token));
        token += 1;
    }
    TokenRange { tokens, is_specific: true }
}

/// Go ast.Kind(token):SyntaxKind 为 #[repr(i16)] 连续枚举,
/// 仅用于 token 区间(start..=end 均为 token kind)
fn syntax_kind_from_i16(token: i16) -> SyntaxKind {
    debug_assert!(token >= 0 && token <= SyntaxKind::DeferKeyword as i16);
    unsafe { std::mem::transmute::<i16, SyntaxKind>(token) }
}

pub fn range_has_no_errors(_r: TextRange) -> bool {
    false
}

pub fn create_text_change_from_start_length(
    start: usize,
    length: usize,
    new_text: &str,
) -> TextChange {
    TextChange {
        pos: start,
        end: start + length,
        new_text: new_text.to_string(),
    }
}

pub fn get_first_non_decorator_token_of_node(node: &Arc<Node>) -> SyntaxKind {
    if can_have_modifiers(node) {
        let modifier_nodes: Vec<Arc<Node>> = node
            .modifiers()
            .map(|m| m.list.nodes.clone())
            .unwrap_or_default();
        let first_decorator = modifier_nodes
            .iter()
            .position(|n| n.kind == SyntaxKind::Decorator);
        let from = first_decorator.unwrap_or(modifier_nodes.len());
        if let Some(m) = modifier_nodes[from..]
            .iter()
            .find(|n| crate::ast::mig::m3g::is_modifier(n))
        {
            return m.kind;
        }
    }

    match node.kind {
        SyntaxKind::ClassDeclaration => return SyntaxKind::ClassKeyword,
        SyntaxKind::InterfaceDeclaration => return SyntaxKind::InterfaceKeyword,
        SyntaxKind::FunctionDeclaration => return SyntaxKind::FunctionKeyword,
        SyntaxKind::EnumDeclaration => return SyntaxKind::EnumDeclaration,
        SyntaxKind::GetAccessor => return SyntaxKind::GetKeyword,
        SyntaxKind::SetAccessor => return SyntaxKind::SetKeyword,
        SyntaxKind::MethodDeclaration => {
            if let NodeData::MethodDeclaration(d) = &node.data {
                if d.asterisk_token.is_some() {
                    return SyntaxKind::AsteriskToken;
                }
            }
            return name_kind_of_declaration(node);
        }
        SyntaxKind::PropertyDeclaration | SyntaxKind::Parameter => {
            return name_kind_of_declaration(node);
        }
        _ => {}
    }

    SyntaxKind::Unknown
}

fn name_kind_of_declaration(node: &Arc<Node>) -> SyntaxKind {
    match get_name_of_declaration(node) {
        Some(name) => name.kind,
        None => SyntaxKind::Unknown,
    }
}
