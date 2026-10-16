use crate::scanner::{TokenFlags, TOKEN_FLAGS_CONTAINS_SEPARATOR, TOKEN_FLAGS_IS_INVALID, TOKEN_FLAGS_SINGLE_QUOTE};
use crate::ast::node::{Node, SourceFile};
use crate::ast::node_data_generated::NodeData;
use crate::ast::syntax_kind_generated::SyntaxKind;
use crate::ast::mig::m3g_3::is_unterminated_literal;
use crate::ast::utilities_statements::is_prologue_directive;
use crate::ast::utilities_synthesized::node_is_synthesized;
use crate::format::mig::m4t_2::{
    escape_string_worker, GetLiteralTextFlags, QuoteChar, GET_LITERAL_TEXT_FLAGS_ALLOW_NUMERIC_SEPARATOR,
    GET_LITERAL_TEXT_FLAGS_TERMINATE_UNTERMINATED_LITERALS,
};
use crate::scanner::mig::m3i::get_source_text_of_node_from_source_file;
use std::sync::Arc;

pub fn can_use_original_text(node: &Arc<Node>, flags: GetLiteralTextFlags) -> bool { ::tsox_core::fntrace::enter("can_use_original_text"); 
    if node_is_synthesized(node)
        || node.parent().is_none()
        || flags & GET_LITERAL_TEXT_FLAGS_TERMINATE_UNTERMINATED_LITERALS != 0
            && is_unterminated_literal(node)
    {
        return false;
    }

    if node.kind == SyntaxKind::NumericLiteral {
        let NodeData::NumericLiteral(d) = &node.data else {
            return false;
        };
        let token_flags: TokenFlags = d.token_flags;
        if token_flags & TOKEN_FLAGS_IS_INVALID != 0 {
            return false;
        }
        if token_flags & TOKEN_FLAGS_CONTAINS_SEPARATOR != 0 {
            return flags & GET_LITERAL_TEXT_FLAGS_ALLOW_NUMERIC_SEPARATOR != 0;
        }
    }

    node.kind != SyntaxKind::BigIntLiteral
}

pub fn get_literal_text(
    node: &Arc<Node>,
    source_file: Option<&Arc<SourceFile>>,
    flags: GetLiteralTextFlags,
) -> String { ::tsox_core::fntrace::enter("get_literal_text"); 
    if source_file.is_some() && can_use_original_text(node, flags) {
        return get_source_text_of_node_from_source_file(source_file.unwrap(), node, false);
    }

    match node.kind {
        SyntaxKind::StringLiteral => {
            let mut b = String::new();
            let quote_char = match &node.data {
                NodeData::StringLiteral(d) if d.token_flags & TOKEN_FLAGS_SINGLE_QUOTE != 0 => {
                    QuoteChar::SingleQuote
                }
                _ => QuoteChar::DoubleQuote,
            };

            let text = node.text();

            b.reserve(text.len() + 2);
            b.push(quote_char.as_char());
            escape_string_worker(text, quote_char, flags, &mut b);
            b.push(quote_char.as_char());
            b
        }
        SyntaxKind::NoSubstitutionTemplateLiteral
        | SyntaxKind::TemplateHead
        | SyntaxKind::TemplateMiddle
        | SyntaxKind::TemplateTail => {
            let mut b = String::new();
            let text = node.text();
            let raw_text = template_literal_like_raw_text(node);
            let raw = !raw_text.is_empty() || text.is_empty();

            let text_len = if raw { raw_text.len() } else { text.len() };

            match node.kind {
                SyntaxKind::NoSubstitutionTemplateLiteral => {
                    b.reserve(2 + text_len);
                    b.push('`');
                }
                SyntaxKind::TemplateHead => {
                    b.reserve(3 + text_len);
                    b.push('`');
                }
                SyntaxKind::TemplateMiddle => {
                    b.reserve(3 + text_len);
                    b.push('}');
                }
                SyntaxKind::TemplateTail => {
                    b.reserve(2 + text_len);
                    b.push('}');
                }
                _ => {}
            }

            if !raw_text.is_empty() || text.is_empty() {
                b.push_str(&raw_text);
            } else {
                escape_string_worker(text, QuoteChar::Backtick, flags, &mut b);
            }

            match node.kind {
                SyntaxKind::NoSubstitutionTemplateLiteral => b.push('`'),
                SyntaxKind::TemplateHead => b.push_str("${"),
                SyntaxKind::TemplateMiddle => b.push_str("${"),
                SyntaxKind::TemplateTail => b.push('`'),
                _ => {}
            }
            b
        }
        SyntaxKind::NumericLiteral | SyntaxKind::BigIntLiteral => node.text().to_string(),
        SyntaxKind::RegularExpressionLiteral => {
            if flags & GET_LITERAL_TEXT_FLAGS_TERMINATE_UNTERMINATED_LITERALS != 0
                && is_unterminated_literal(node)
            {
                let text = node.text();
                let mut b = String::new();
                if !text.is_empty() && text.as_bytes()[text.len() - 1] == b'\\' {
                    b.reserve(2 + text.len());
                    b.push_str(text);
                    b.push_str(" /");
                } else {
                    b.reserve(1 + text.len());
                    b.push_str(text);
                    b.push('/');
                }
                return b;
            }
            node.text().to_string()
        }
        _ => panic!("Unsupported LiteralLikeNode"),
    }
}

pub fn is_not_prologue_directive(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_not_prologue_directive"); 
    !is_prologue_directive(node)
}

/// Go TemplateLiteralLikeData().RawText;NoSubstitutionTemplateLiteralData
/// 尚无 raw_text 字段,返回空串(等价回退到 cooked text 分支)
fn template_literal_like_raw_text(node: &Node) -> &str { ::tsox_core::fntrace::enter("template_literal_like_raw_text"); 
    match &node.data {
        NodeData::TemplateHead(d) => &d.raw_text,
        NodeData::TemplateMiddle(d) => &d.raw_text,
        NodeData::TemplateTail(d) => &d.raw_text,
        NodeData::NoSubstitutionTemplateLiteral(_) => "",
        _ => "",
    }
}
