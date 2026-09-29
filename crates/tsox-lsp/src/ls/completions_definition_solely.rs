use std::sync::Arc;

use tsox_frontend::ast::{
    Node, SyntaxKind, is_class_like,
    is_function_like_kind, is_keyword_kind, node_name,
};

use super::completions_context::{ScanToken, line_of_position};
use super::completions_definition_location::{
    go_token_parent, is_class_member_completion_keyword, is_constructor_parameter_completion,
    is_currently_editing_node, is_from_object_type_declaration, is_declaration_name,
    is_parameter_property_modifier, same_token, token_parent,
};

/// Go isSolelyIdentifierDefinitionLocation：当前编辑位必然是新标识符定义位
pub(super) fn is_solely_identifier_definition_location(
    context: &ScanToken,
    previous_token: Option<&ScanToken>,
    text: &str,
    position: usize,
    root: &Arc<Node>,
) -> bool {
    let Some(parent) = go_token_parent(root, context) else {
        return false;
    };
    let kind = parent.kind;

    if context.kind == SyntaxKind::Identifier {
        if matches!(kind, SyntaxKind::ImportSpecifier | SyntaxKind::ExportSpecifier)
            && node_name(&parent).is_some_and(|n| {
                n.loc.pos() == context.pos && n.loc.end() == context.end
            })
            && &text[context.pos..context.end] == "type"
        {
            return false;
        }
        if find_ancestor_var_declaration(&parent).is_some()
            && line_of_position(text, context.end) < line_of_position(text, position)
        {
            return false;
        }
    }

    match context.kind {
        SyntaxKind::CommaToken => {
            return matches!(
                kind,
                SyntaxKind::VariableDeclaration
                    | SyntaxKind::VariableDeclarationList
                    | SyntaxKind::VariableStatement
                    | SyntaxKind::EnumDeclaration
                    | SyntaxKind::InterfaceDeclaration
                    | SyntaxKind::ArrayBindingPattern
                    | SyntaxKind::TypeAliasDeclaration
            ) || is_function_like_but_not_constructor(kind)
                || (is_class_like(&parent) && class_type_params_reach(&parent, context.pos));
        }
        SyntaxKind::DotToken => return kind == SyntaxKind::ArrayBindingPattern,
        SyntaxKind::ColonToken => return kind == SyntaxKind::BindingElement,
        SyntaxKind::OpenBracketToken => return kind == SyntaxKind::ArrayBindingPattern,
        SyntaxKind::OpenParenToken => {
            return kind == SyntaxKind::CatchClause || is_function_like_but_not_constructor(kind);
        }
        SyntaxKind::OpenBraceToken => return kind == SyntaxKind::EnumDeclaration,
        SyntaxKind::LessThanToken => {
            return matches!(
                kind,
                SyntaxKind::ClassDeclaration
                    | SyntaxKind::ClassExpression
                    | SyntaxKind::InterfaceDeclaration
                    | SyntaxKind::TypeAliasDeclaration
            ) || is_function_like_kind(kind);
        }
        SyntaxKind::StaticKeyword => {
            return kind == SyntaxKind::PropertyDeclaration
                && parent.parent().as_ref().is_none_or(|p| !is_class_like(p));
        }
        SyntaxKind::DotDotDotToken => {
            return kind == SyntaxKind::Parameter
                || parent
                    .parent()
                    .as_ref()
                    .is_some_and(|p| p.kind == SyntaxKind::ArrayBindingPattern);
        }
        SyntaxKind::PublicKeyword | SyntaxKind::PrivateKeyword | SyntaxKind::ProtectedKeyword => {
            return kind == SyntaxKind::Parameter
                && parent
                    .parent()
                    .as_ref()
                    .is_none_or(|p| p.kind != SyntaxKind::Constructor);
        }
        SyntaxKind::AsKeyword => {
            return matches!(
                kind,
                SyntaxKind::ImportSpecifier
                    | SyntaxKind::ExportSpecifier
                    | SyntaxKind::NamespaceImport
            );
        }
        SyntaxKind::GetKeyword | SyntaxKind::SetKeyword => {
            return !is_from_object_type_declaration(context, root);
        }
        SyntaxKind::ClassKeyword
        | SyntaxKind::EnumKeyword
        | SyntaxKind::InterfaceKeyword
        | SyntaxKind::FunctionKeyword
        | SyntaxKind::VarKeyword
        | SyntaxKind::ImportKeyword
        | SyntaxKind::LetKeyword
        | SyntaxKind::ConstKeyword
        | SyntaxKind::InferKeyword => return true,
        SyntaxKind::TypeKeyword => return kind != SyntaxKind::ImportSpecifier,
        SyntaxKind::AsteriskToken => {
            return is_function_like_kind(kind) && kind != SyntaxKind::MethodDeclaration;
        }
        _ => {}
    }

    let token_kind = if is_keyword_kind(context.kind) {
        context.kind
    } else {
        SyntaxKind::Unknown
    };
    if is_class_member_completion_keyword(token_kind) && is_from_object_type_declaration(context, root)
    {
        return false;
    }
    if is_constructor_parameter_completion(context, root)
        && (context.kind != SyntaxKind::Identifier
            || is_parameter_property_modifier(token_kind)
            || is_currently_editing_node(context, position))
    {
        return false;
    }
    match token_kind {
        SyntaxKind::AbstractKeyword
        | SyntaxKind::ClassKeyword
        | SyntaxKind::DeclareKeyword
        | SyntaxKind::EnumKeyword
        | SyntaxKind::FunctionKeyword
        | SyntaxKind::InterfaceKeyword
        | SyntaxKind::LetKeyword
        | SyntaxKind::PrivateKeyword
        | SyntaxKind::ProtectedKeyword
        | SyntaxKind::PublicKeyword
        | SyntaxKind::StaticKeyword
        | SyntaxKind::VarKeyword => return true,
        SyntaxKind::AsyncKeyword => return kind == SyntaxKind::PropertyDeclaration,
        _ => {}
    }

    if find_ancestor(&parent, &|n: &Arc<Node>| is_class_like(n)).is_some()
        && same_token(context, previous_token)
        && is_previous_property_declaration_terminated(context, text, position)
    {
        return false;
    }
    if let (Some(pd), Some(prev)) = (
        find_ancestor(&parent, &|n: &Arc<Node>| n.kind == SyntaxKind::PropertyDeclaration),
        previous_token,
    ) && !same_token(context, previous_token)
    {
        let prev_class_like = token_parent(root, prev)
            .and_then(|t| t.parent())
            .and_then(|t| t.parent())
            .is_some_and(|p| is_class_like(&p));
        if prev_class_like && position <= prev.end {
            if is_previous_property_declaration_terminated(context, text, prev.end) {
                return false;
            }
            if context.kind != SyntaxKind::EqualsToken
                && (is_initialized_property(&pd) || property_has_type(&pd))
            {
                return true;
            }
        }
    }
    if token_kind == SyntaxKind::ConstKeyword {
        return true;
    }
    is_declaration_name(context, root)
        && kind != SyntaxKind::ShorthandPropertyAssignment
        && kind != SyntaxKind::JsxAttribute
        && !((is_class_like(&parent)
            || matches!(
                kind,
                SyntaxKind::InterfaceDeclaration | SyntaxKind::TypeParameter
            ))
            && (!same_token(context, previous_token)
                || previous_token.is_some_and(|p| position > p.end)))
}

fn is_function_like_but_not_constructor(kind: SyntaxKind) -> bool {
    is_function_like_kind(kind) && kind != SyntaxKind::Constructor
}

fn find_ancestor(node: &Arc<Node>, predicate: &dyn Fn(&Arc<Node>) -> bool) -> Option<Arc<Node>> {
    let mut current = Some(Arc::clone(node));
    while let Some(n) = current {
        if predicate(&n) {
            return Some(n);
        }
        current = n.parent();
    }
    None
}

fn find_ancestor_var_declaration(node: &Arc<Node>) -> Option<Arc<Node>> {
    find_ancestor(node, &|n: &Arc<Node>| n.kind == SyntaxKind::VariableDeclaration)
}

fn class_type_params_reach(class_like: &Arc<Node>, pos: usize) -> bool {
    let type_params = match &class_like.data {
        tsox_frontend::ast::NodeData::ClassDeclaration(d) => &d.type_parameters,
        tsox_frontend::ast::NodeData::ClassExpression(d) => &d.type_parameters,
        _ => return false,
    };
    type_params
        .as_ref()
        .is_some_and(|tp| tp.loc.end() >= pos)
}

fn is_previous_property_declaration_terminated(
    context: &ScanToken,
    text: &str,
    position: usize,
) -> bool {
    context.kind != SyntaxKind::EqualsToken
        && (context.kind == SyntaxKind::SemicolonToken
            || line_of_position(text, context.end) != line_of_position(text, position))
}

fn is_initialized_property(pd: &Arc<Node>) -> bool {
    matches!(&pd.data, tsox_frontend::ast::NodeData::PropertyDeclaration(d) if d.initializer.is_some())
}

fn property_has_type(pd: &Arc<Node>) -> bool {
    matches!(&pd.data, tsox_frontend::ast::NodeData::PropertyDeclaration(d) if d.type_node.is_some())
}
