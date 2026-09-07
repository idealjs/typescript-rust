use std::sync::Arc;

use crate::ast::node_data_generated::for_each_child;
use crate::ast::{Node, SyntaxKind};

use crate::ast::{Symbol, SymbolFlags};
use crate::checker::Checker;

use super::SemanticToken;
use super::token_modifier;
use super::token_type;

pub fn classify_symbol(symbol: &Symbol, _meaning: u32) -> (u32, bool) {
    let flags = symbol.flags;
    if flags.contains(SymbolFlags::TypeParameter) {
        return (token_type::TYPE_PARAMETER, true);
    }
    if flags.contains(SymbolFlags::Class) {
        return (token_type::CLASS, true);
    }
    if flags.contains(SymbolFlags::Interface) {
        return (token_type::INTERFACE, true);
    }
    if flags.contains(SymbolFlags::TypeAlias) {
        return (token_type::TYPE, true);
    }
    if flags.contains(SymbolFlags::ENUM) {
        return (token_type::ENUM, true);
    }
    if flags.contains(SymbolFlags::EnumMember) {
        return (token_type::ENUM_MEMBER, true);
    }
    if flags.contains(SymbolFlags::Function) {
        return (token_type::FUNCTION, true);
    }
    if flags.contains(SymbolFlags::Method) {
        return (token_type::METHOD, true);
    }
    if flags.contains(SymbolFlags::Constructor) {
        return (token_type::FUNCTION, true);
    }
    if flags.intersects(SymbolFlags::GetAccessor | SymbolFlags::SetAccessor) {
        return (token_type::PROPERTY, true);
    }
    if flags.contains(SymbolFlags::Property) {
        return (token_type::PROPERTY, true);
    }
    if flags.intersects(SymbolFlags::ValueModule | SymbolFlags::NamespaceModule) {
        return (token_type::NAMESPACE, true);
    }
    if flags.contains(SymbolFlags::VARIABLE) {
        return (token_type::VARIABLE, true);
    }
    if flags.contains(SymbolFlags::Alias) {
        return (token_type::VARIABLE, false);
    }
    (token_type::INVALID, false)
}

pub fn token_from_declaration_mapping(kind: crate::ast::SyntaxKind) -> u32 {
    use crate::ast::SyntaxKind;
    match kind {
        SyntaxKind::VariableDeclaration => token_type::VARIABLE,
        SyntaxKind::Parameter => token_type::PARAMETER,
        SyntaxKind::PropertyDeclaration => token_type::PROPERTY,
        SyntaxKind::ModuleDeclaration => token_type::NAMESPACE,
        SyntaxKind::EnumDeclaration => token_type::ENUM,
        SyntaxKind::EnumMember => token_type::ENUM_MEMBER,
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => token_type::CLASS,
        SyntaxKind::MethodDeclaration => token_type::METHOD,
        SyntaxKind::FunctionDeclaration | SyntaxKind::FunctionExpression => token_type::FUNCTION,
        SyntaxKind::MethodSignature => token_type::METHOD,
        SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => token_type::PROPERTY,
        SyntaxKind::PropertySignature => token_type::PROPERTY,
        SyntaxKind::InterfaceDeclaration => token_type::INTERFACE,
        SyntaxKind::TypeAliasDeclaration => token_type::TYPE,
        SyntaxKind::TypeParameter => token_type::TYPE_PARAMETER,
        SyntaxKind::PropertyAssignment | SyntaxKind::ShorthandPropertyAssignment => {
            token_type::PROPERTY
        }
        _ => token_type::INVALID,
    }
}

pub(super) fn collect_tokens(
    checker: &Checker,
    node: &Arc<Node>,
    span_start: usize,
    span_end: usize,
    tokens: &mut Vec<SemanticToken>,
) {
    if node.end() <= span_start || node.pos() >= span_end {
        return;
    }

    if let Some(token) = classify_node_token(checker, node) {
        if token.pos >= span_start && token.end <= span_end {
            tokens.push(token);
        }
    }

    for_each_child(node, |child| {
        collect_tokens(checker, child, span_start, span_end, tokens);
        false
    });
}

fn classify_node_token(checker: &Checker, node: &Arc<Node>) -> Option<SemanticToken> {
    let kind = node.kind;

    match kind {
        SyntaxKind::NumericLiteral | SyntaxKind::BigIntLiteral => {
            return Some(SemanticToken {
                token_type: token_type::NUMBER,
                token_modifier: 0,
                pos: node.pos(),
                end: node.end(),
            });
        }
        SyntaxKind::StringLiteral
        | SyntaxKind::NoSubstitutionTemplateLiteral
        | SyntaxKind::TemplateHead
        | SyntaxKind::TemplateMiddle
        | SyntaxKind::TemplateTail => {
            return Some(SemanticToken {
                token_type: token_type::STRING,
                token_modifier: 0,
                pos: node.pos(),
                end: node.end(),
            });
        }
        SyntaxKind::RegularExpressionLiteral => {
            return Some(SemanticToken {
                token_type: token_type::REGEXP,
                token_modifier: 0,
                pos: node.pos(),
                end: node.end(),
            });
        }
        _ => {}
    }

    if is_keyword_kind(kind) {
        return Some(SemanticToken {
            token_type: token_type::KEYWORD,
            token_modifier: 0,
            pos: node.pos(),
            end: node.end(),
        });
    }

    if kind == SyntaxKind::Identifier {
        if let Some(parent) = node.parent.as_ref() {
            let decl_type = token_from_declaration_mapping(parent.kind);
            if decl_type != token_type::INVALID {
                let mut modifier = 0u32;

                if is_name_of_declaration(node, parent) {
                    modifier |= token_modifier::DECLARATION | token_modifier::DEFINITION;
                }
                return Some(SemanticToken {
                    token_type: decl_type,
                    token_modifier: modifier,
                    pos: node.pos(),
                    end: node.end(),
                });
            }
        }

        if let Some(symbol) = checker.get_symbol_at_location(node) {
            let (token_type_val, is_declaration) = classify_symbol(&symbol, 0);
            if token_type_val != token_type::INVALID {
                let mut modifier = 0u32;
                if is_declaration {
                    modifier |= token_modifier::DECLARATION;
                }
                return Some(SemanticToken {
                    token_type: token_type_val,
                    token_modifier: modifier,
                    pos: node.pos(),
                    end: node.end(),
                });
            }
        }

        return Some(SemanticToken {
            token_type: token_type::VARIABLE,
            token_modifier: 0,
            pos: node.pos(),
            end: node.end(),
        });
    }

    None
}

pub(super) fn is_keyword_kind(kind: SyntaxKind) -> bool {
    (kind as i16) >= (SyntaxKind::BreakKeyword as i16)
        && (kind as i16) <= (SyntaxKind::DeferKeyword as i16)
}

pub(super) fn is_name_of_declaration(name: &Arc<Node>, declaration: &Arc<Node>) -> bool {
    declaration
        .name()
        .map(|n| Arc::ptr_eq(n, name))
        .unwrap_or(false)
}
