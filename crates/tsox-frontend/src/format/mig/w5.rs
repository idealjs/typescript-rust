#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;

use crate::ast::{Node, NodeList, SourceFile, SyntaxKind};
use crate::format::lists::get_visual_list_range;
use super::m4u::MASK_BIT_SIZE;

use crate::ast::mig::m3b::parameter_list;
use crate::ast::mig::m3c::{type_argument_list, type_parameter_list};
use crate::ast::mig::x1a::argument_list;

const FORMAT_NEWLINE_KEY: &str = "formatNewline";

pub fn get_list(
    list: Option<&NodeList>,
    r: TextRange,
    node: &Arc<Node>,
    source_file: &SourceFile,
) -> Option<NodeList> {
    let list = list?;
    if r.contained_by(&get_visual_list_range(node, list.loc, source_file)) {
        return Some(NodeList {
            loc: list.loc,
            nodes: list.nodes.clone(),
        });
    }
    None
}

pub fn get_new_line_or_default_from_context(ctx: &crate::format::FormatContext) -> String {
    if !ctx.new_line_character.is_empty() {
        return ctx.new_line_character.clone();
    }
    let _ = FORMAT_NEWLINE_KEY;
    "\n".to_string()
}

pub fn get_open_token_for_list(node: &Arc<Node>, list: &NodeList) -> SyntaxKind {
    match node.kind {
        SyntaxKind::Constructor
        | SyntaxKind::FunctionDeclaration
        | SyntaxKind::FunctionExpression
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::MethodSignature
        | SyntaxKind::ArrowFunction
        | SyntaxKind::CallSignature
        | SyntaxKind::ConstructSignature
        | SyntaxKind::FunctionType
        | SyntaxKind::ConstructorType
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor => {
            if ptr_eq_opt(type_parameter_list(node).map(Arc::as_ref), list) {
                return SyntaxKind::LessThanToken;
            } else if ptr_eq_opt(parameter_list(node).map(Arc::as_ref), list) {
                return SyntaxKind::OpenParenToken;
            }
        }
        SyntaxKind::CallExpression | SyntaxKind::NewExpression => {
            if ptr_eq_opt(type_argument_list(node).map(Arc::as_ref), list) {
                return SyntaxKind::LessThanToken;
            } else if ptr_eq_opt(argument_list(node), list) {
                return SyntaxKind::OpenParenToken;
            }
        }
        SyntaxKind::ClassDeclaration
        | SyntaxKind::ClassExpression
        | SyntaxKind::InterfaceDeclaration
        | SyntaxKind::TypeAliasDeclaration => {
            if ptr_eq_opt(type_parameter_list(node).map(Arc::as_ref), list) {
                return SyntaxKind::LessThanToken;
            }
        }
        SyntaxKind::TypeReference
        | SyntaxKind::TaggedTemplateExpression
        | SyntaxKind::TypeQuery
        | SyntaxKind::ExpressionWithTypeArguments
        | SyntaxKind::ImportType => {
            if ptr_eq_opt(type_argument_list(node).map(Arc::as_ref), list) {
                return SyntaxKind::LessThanToken;
            }
        }
        SyntaxKind::TypeLiteral => return SyntaxKind::OpenBraceToken,
        _ => {}
    }
    SyntaxKind::Unknown
}

fn ptr_eq_opt(a: Option<&NodeList>, b: &NodeList) -> bool {
    match a {
        Some(a) => std::ptr::eq(a, b),
        None => false,
    }
}

pub fn get_rule_insertion_index(index_bitmap: u32, mask_position: usize) -> usize {
    let mask: u32 = (1 << MASK_BIT_SIZE) - 1;
    let mut index = 0usize;
    let mut index_bitmap = index_bitmap;
    let mut pos = 0usize;
    while pos <= mask_position {
        index += (index_bitmap & mask) as usize;
        index_bitmap >>= MASK_BIT_SIZE;
        pos += MASK_BIT_SIZE;
    }
    index
}
