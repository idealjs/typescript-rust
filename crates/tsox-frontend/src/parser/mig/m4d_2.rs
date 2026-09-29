use crate::ast::node_node_list::NodeList;
use crate::ast::{
    is_identifier, is_non_null_expression, Node, NodeData, NodeFlags, SourceFile, SyntaxKind,
};
use crate::parser::parsing_context::Parser;
use crate::scanner::mig::m4d_2::is_valid_identifier;
use crate::scanner::skip_trivia;
use std::sync::Arc;
use tsox_core::core::mig::m3j::ScriptKind;
use tsox_core::core::text::TextRange;
use tsox_core::diagnostics;

impl Parser {
    pub(crate) fn skip_range_trivia(&self, text_range: TextRange) -> TextRange {
        TextRange::new(
            skip_trivia(self.scanner.text(), text_range.pos()),
            text_range.end(),
        )
    }
}

impl Parser {
    pub(crate) fn reparse_jsdoc_comment(&mut self, node: &Arc<Node>, tag: &Arc<Node>) {
        let (comment_loc, comment_nodes) = match &tag.data {
            NodeData::JSDoc(d) => (d.comment.loc, d.comment.nodes.clone()),
            NodeData::JSDocParameterOrPropertyTag(d) => match &d.comment {
                Some(comment) => (comment.loc, comment.nodes.clone()),
                None => return,
            },
            _ => return,
        };
        let comment = Arc::new(NodeList {
            loc: comment_loc,
            nodes: comment_nodes,
        });
        let prop_jsdoc = Arc::new(Node::with_loc(
            SyntaxKind::JSDoc,
            NodeData::JSDoc(crate::ast::JSDocData {
                comment,
                tags: None,
            }),
            comment_loc,
        ));
        prop_jsdoc.set_parent(node);
    }

    pub(crate) fn make_new_cast(
        &self,
        t: &Arc<Node>,
        e: &Arc<Node>,
        is_assertion: bool,
    ) -> Arc<Node> {
        let (kind, data) = if is_assertion {
            (
                SyntaxKind::AsExpression,
                NodeData::AsExpression(crate::ast::AsExpressionData {
                    expression: e.clone(),
                    type_node: t.clone(),
                }),
            )
        } else {
            (
                SyntaxKind::SatisfiesExpression,
                NodeData::SatisfiesExpression(crate::ast::SatisfiesExpressionData {
                    expression: e.clone(),
                    type_node: t.clone(),
                }),
            )
        };
        Arc::new(Node::with_loc(kind, data, TextRange::new(e.pos(), e.end())))
    }
}
fn is_double_quoted_string(node: &Arc<Node>) -> bool {
    crate::ast::is_string_literal(node)
        && matches!(
            &node.data,
            NodeData::StringLiteral(d)
                if d.token_flags & crate::scanner::TOKEN_FLAGS_SINGLE_QUOTE == 0
        )
}

pub(crate) fn skip_to(text: &str, pos: usize, s: &str) -> Option<usize> {
    if pos >= text.len() {
        return None;
    }
    text[pos..].find(s).map(|i| pos + i)
}

pub(crate) fn type_has_arrow_function_blocking_parse_error(node: &Arc<Node>) -> bool {
    match node.kind {
        SyntaxKind::TypeReference => match &node.data {
            NodeData::TypeReferenceNode(d) => crate::astnav::is_missing_node(&d.type_name),
            _ => false,
        },
        SyntaxKind::FunctionType | SyntaxKind::ConstructorType => {
            crate::parser::mig::w7p::is_missing_node_list(
                crate::ast::mig::m3b::parameter_list(node).map(|l| &**l),
            ) || node
                .type_node()
                .map(type_has_arrow_function_blocking_parse_error)
                .unwrap_or(false)
        }
        SyntaxKind::ParenthesizedType => node
            .type_node()
            .map(type_has_arrow_function_blocking_parse_error)
            .unwrap_or(false),
        _ => false,
    }
}

pub(crate) fn find_matching_parameter(
    fun: &Arc<Node>,
    parameter_tag: &Arc<Node>,
    js_doc: &Arc<Node>,
) -> Option<(Arc<Node>, bool)> {
    let mut tag_index: isize = -1;
    let mut param_count: isize = -1;
    if let NodeData::JSDoc(d) = &js_doc.data {
        if let Some(tags) = &d.tags {
            for tag in tags.nodes.iter() {
                if tag.kind == SyntaxKind::JSDocParameterTag {
                    param_count += 1;
                    if Arc::ptr_eq(tag, parameter_tag) {
                        tag_index = param_count;
                        break;
                    }
                }
            }
        }
    }
    for (parameter_index, parameter) in
        crate::ast::mig::m3b::parameters(fun).iter().enumerate()
    {
        if let Some(name) = parameter.name() {
            if let Some(tag_name) = parameter_tag.name() {
                if name.kind == SyntaxKind::Identifier
                    && tag_name.kind == SyntaxKind::Identifier
                    && ((name.text() == tag_name.text())
                        || (parameter_index as isize == tag_index && tag_name.text().is_empty()))
                {
                    return Some((parameter.clone(), true));
                }
            }
        } else if parameter_index as isize == tag_index {
            return Some((parameter.clone(), true));
        }
    }
    None
}

pub(crate) fn skip_satisfies_expressions(node: Option<&Arc<Node>>) -> Option<Arc<Node>> {
    let mut node = node.cloned();
    while let Some(n) = node.clone() {
        if n.kind != SyntaxKind::SatisfiesExpression {
            break;
        }
        node = n.expression().cloned();
    }
    node
}

pub(crate) fn get_function_like_host(host: &Arc<Node>) -> Option<Arc<Node>> {
    let mut fun: Option<Arc<Node>> = Some(host.clone());
    match host.kind {
        SyntaxKind::VariableStatement => {
            if let NodeData::VariableStatement(d) = &host.data {
                if let NodeData::VariableDeclarationList(list) = &d.declaration_list.data {
                    let nodes = &list.declarations.nodes;
                    if !nodes.is_empty() {
                        fun = nodes[0].initializer().cloned();
                    }
                }
            }
        }
        SyntaxKind::PropertyAssignment | SyntaxKind::PropertyDeclaration => {
            fun = host.initializer().cloned();
        }
        SyntaxKind::ExportAssignment | SyntaxKind::ReturnStatement => {
            fun = host.expression().cloned();
        }
        SyntaxKind::ExpressionStatement => {
            fun = Some(crate::ast::mig::m3f::get_right_most_assigned_expression(
                host.expression()?,
            ));
        }
        _ => {}
    }
    let fun = skip_satisfies_expressions(fun.as_ref())?;
    if crate::ast::is_function_like(&fun) {
        return Some(fun);
    }
    None
}

#[derive(Debug, Clone)]
pub(crate) struct ClassLikeBase {
    pub type_parameters: Option<Arc<NodeList>>,
    pub heritage_clauses: Option<Arc<NodeList>>,
    pub members: Arc<NodeList>,
}

pub(crate) fn get_class_like_data(parent: &Arc<Node>) -> Option<ClassLikeBase> {
    match &parent.data {
        NodeData::ClassDeclaration(d) => Some(ClassLikeBase {
            type_parameters: d.type_parameters.clone(),
            heritage_clauses: d.heritage_clauses.clone(),
            members: d.members.clone(),
        }),
        NodeData::ClassExpression(d) => Some(ClassLikeBase {
            type_parameters: d.type_parameters.clone(),
            heritage_clauses: d.heritage_clauses.clone(),
            members: d.members.clone(),
        }),
        _ => None,
    }
}

pub(crate) fn get_language_variant(script_kind: ScriptKind) -> crate::ast::LanguageVariant {
    match script_kind {
        ScriptKind::Tsx | ScriptKind::Jsx | ScriptKind::Js | ScriptKind::Json => {
            crate::ast::LanguageVariant::Jsx
        }
        _ => crate::ast::LanguageVariant::Standard,
    }
}

pub(crate) fn is_keyword_or_punctuation(token: SyntaxKind) -> bool {
    crate::ast::is_keyword_kind(token) || crate::ast::is_punctuation_kind(token)
}
