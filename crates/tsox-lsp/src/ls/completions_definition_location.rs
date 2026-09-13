use std::sync::Arc;

use super::completions_context::{ScanToken, line_of_position};
use tsox_frontend::ast::{
    Node, SyntaxKind, is_class_element, is_class_like, is_declaration, is_function_like_kind,
    is_keyword_kind, is_type_element, node_name,
};

/// Go AST 的 token 自带 Parent；此处以「包住 token 跨度的最深节点」近似
pub(super) fn token_parent(root: &Arc<Node>, tok: &ScanToken) -> Option<Arc<Node>> {
    let mut current = Arc::clone(root);
    loop {
        let mut next: Option<Arc<Node>> = None;
        tsox_frontend::ast::for_each_child(&current, |child| {
            if child.loc.pos() <= tok.pos && tok.end <= child.loc.end() {
                next = Some(Arc::clone(child));
                true
            } else {
                false
            }
        });
        match next {
            Some(child) => current = child,
            None => return Some(current),
        }
    }
}

pub(super) fn is_completion_list_blocker(
    context_token: Option<&ScanToken>,
    previous_token: Option<&ScanToken>,
    containing_token: Option<&ScanToken>,
    location: &Arc<Node>,
    text: &str,
    position: usize,
    root: &Arc<Node>,
) -> bool {
    let literal_token = context_token.or(containing_token);
    if let Some(tok) = literal_token
        && is_in_string_or_regular_expression_or_template(tok, text, position)
    {
        return true;
    }
    let Some(context) = context_token else {
        return false;
    };
    super::completions_definition_solely::is_solely_identifier_definition_location(
        context,
        previous_token,
        text,
        position,
        root,
    )
        || is_dot_of_numeric_literal(context, text)
        || context.kind == SyntaxKind::BigIntLiteral
        || is_in_jsx_text(context, location, root)
}

fn is_in_string_or_regular_expression_or_template(tok: &ScanToken, text: &str, position: usize) -> bool {
    let is_string = matches!(
        tok.kind,
        SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral
    );
    let is_regex = tok.kind == SyntaxKind::RegularExpressionLiteral;
    if !is_string && !is_regex {
        return false;
    }
    if tok.pos < position && position < tok.end {
        return true;
    }
    position == tok.end && (is_unterminated_literal(tok, text) || is_regex)
}

fn is_unterminated_literal(tok: &ScanToken, text: &str) -> bool {
    let raw = &text[tok.pos.min(text.len())..tok.end.min(text.len())];
    let (quote, body) = match raw.chars().next() {
        Some(quote @ ('"' | '\'' | '`')) => (quote, &raw[1..]),
        _ => return false,
    };
    !body.ends_with(quote)
}

fn is_dot_of_numeric_literal(tok: &ScanToken, text: &str) -> bool {
    tok.kind == SyntaxKind::NumericLiteral && text[tok.pos..tok.end].ends_with('.')
}

fn is_in_jsx_text(context: &ScanToken, location: &Arc<Node>, root: &Arc<Node>) -> bool {
    if context.kind == SyntaxKind::JsxText {
        return true;
    }
    if context.kind == SyntaxKind::GreaterThanToken
        && let Some(parent) = token_parent(root, context)
    {
        if Arc::ptr_eq(&parent, location) && is_jsx_opening_like_element(&parent) {
            return false;
        }
        if parent.kind == SyntaxKind::JsxOpeningElement {
            return location
                .parent()
                .as_ref()
                .is_none_or(|lp| lp.kind != SyntaxKind::JsxOpeningElement);
        }
        if matches!(
            parent.kind,
            SyntaxKind::JsxClosingElement | SyntaxKind::JsxSelfClosingElement
        ) && let Some(grand) = parent.parent()
        {
            return grand.kind == SyntaxKind::JsxElement;
        }
    }
    false
}

fn is_jsx_opening_like_element(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::JsxOpeningElement | SyntaxKind::JsxSelfClosingElement
    )
}

pub(super) fn same_token(a: &ScanToken, b: Option<&ScanToken>) -> bool {
    b.is_some_and(|b| b.pos == a.pos && b.end == a.end)
}


/// Go AST 的 token 直接以声明为 Parent；我们 AST 的标识符/关键字 token
/// 会包一层叶子节点，此处跳过该包装层
pub(super) fn go_token_parent(root: &Arc<Node>, tok: &ScanToken) -> Option<Arc<Node>> {
    let deepest = token_parent(root, tok)?;
    if matches!(deepest.kind, SyntaxKind::Identifier | SyntaxKind::PrivateIdentifier)
        || is_keyword_kind(deepest.kind)
    {
        return deepest.parent().or(Some(deepest));
    }
    Some(deepest)
}

pub(super) fn is_from_object_type_declaration(tok: &ScanToken, root: &Arc<Node>) -> bool {
    let Some(parent) = go_token_parent(root, tok) else {
        return false;
    };
    let Some(grand) = parent.parent() else {
        return false;
    };
    (is_class_element(&parent) || is_type_element(&parent))
        && (is_class_like(&grand)
            || matches!(
                grand.kind,
                SyntaxKind::InterfaceDeclaration | SyntaxKind::TypeLiteral
            ))
}

pub(super) fn is_constructor_parameter_completion(tok: &ScanToken, root: &Arc<Node>) -> bool {
    let Some(parent) = go_token_parent(root, tok) else {
        return false;
    };
    let Some(grand) = parent.parent() else {
        return false;
    };
    parent.kind == SyntaxKind::Parameter
        && grand.kind == SyntaxKind::Constructor
        && (is_parameter_property_modifier(tok.kind) || is_declaration_name(tok, root))
}

pub(super) fn is_currently_editing_node(tok: &ScanToken, position: usize) -> bool {
    tok.pos <= position && position <= tok.end
}

pub(super) fn is_parameter_property_modifier(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::PublicKeyword
            | SyntaxKind::PrivateKeyword
            | SyntaxKind::ProtectedKeyword
            | SyntaxKind::ReadonlyKeyword
    )
}

fn is_class_member_modifier(kind: SyntaxKind) -> bool {
    is_parameter_property_modifier(kind)
        || matches!(
            kind,
            SyntaxKind::StaticKeyword | SyntaxKind::OverrideKeyword | SyntaxKind::AccessorKeyword
        )
}

pub(super) fn is_class_member_completion_keyword(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::AbstractKeyword
            | SyntaxKind::AccessorKeyword
            | SyntaxKind::ConstructorKeyword
            | SyntaxKind::GetKeyword
            | SyntaxKind::SetKeyword
            | SyntaxKind::AsyncKeyword
            | SyntaxKind::DeclareKeyword
            | SyntaxKind::OverrideKeyword
    ) || is_class_member_modifier(kind)
}

pub(super) fn is_declaration_name(tok: &ScanToken, root: &Arc<Node>) -> bool {
    let Some(deepest) = token_parent(root, tok) else {
        return false;
    };
    if deepest.kind != SyntaxKind::Identifier {
        return false;
    }
    let Some(p) = deepest.parent() else {
        return false;
    };
    is_declaration(&p) && node_name(&p).is_some_and(|n| Arc::ptr_eq(n, &deepest))
}

