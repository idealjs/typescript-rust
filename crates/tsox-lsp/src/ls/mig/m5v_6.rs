#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::mig::m3b_2::NodeFactory;
use tsox_frontend::ast::node_data_generated::{MappedTypeNodeData, NodeData, SyntaxListData};
use tsox_frontend::ast::{self, Node, SourceFile, SyntaxKind};

use crate::lsp::lsproto_lsp::Range;

pub trait M5v6NodeExt {
    fn as_mapped_type_node(&self) -> &MappedTypeNodeData;
}

impl M5v6NodeExt for Node {
    fn as_mapped_type_node(&self) -> &MappedTypeNodeData {
        match &self.data {
            NodeData::MappedTypeNode(d) => d,
            _ => panic!("AsMappedTypeNode on wrong node kind"),
        }
    }
}

pub trait M5v6FactoryExt {
    fn new_syntax_list(&self, children: &[Arc<Node>]) -> Node;
}

impl M5v6FactoryExt for NodeFactory {
    fn new_syntax_list(&self, children: &[Arc<Node>]) -> Node {
        Node::new(
            SyntaxKind::SyntaxList,
            NodeData::SyntaxList(SyntaxListData {
                children: children.to_vec(),
            }),
        )
    }
}

pub const MAX_SELECTION_RANGE_DEPTH: usize = 1000;

pub struct M5vSelectionRange {
    pub range: Range,
    pub parent: Option<Box<M5vSelectionRange>>,
}

pub struct SelectionRangeBuilder {
    pub ranges: Vec<Range>,
    pub oldest_index: usize,
}

pub fn new_selection_range_builder(capacity: usize) -> SelectionRangeBuilder {
    SelectionRangeBuilder {
        ranges: Vec::with_capacity(capacity),
        oldest_index: 0,
    }
}

impl SelectionRangeBuilder {
    pub fn push(&mut self, selection_range: Range) {
        if self.ranges.len() < self.ranges.capacity() {
            self.ranges.push(selection_range);
            return;
        }

        if self.ranges.is_empty() {
            return;
        }
        self.ranges[self.oldest_index] = selection_range;
        self.oldest_index = (self.oldest_index + 1) % self.ranges.len();
    }

    pub fn build(
        &self,
        mut result: Option<Box<M5vSelectionRange>>,
    ) -> Option<Box<M5vSelectionRange>> {
        for i in 0..self.ranges.len() {
            let index = (self.oldest_index + i) % self.ranges.len();
            result = Some(Box::new(M5vSelectionRange {
                range: self.ranges[index].clone(),
                parent: result,
            }));
        }
        result
    }
}

pub fn get_selection_children(
    factory: &NodeFactory,
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
) -> Vec<Arc<Node>> {
    if !ast::is_mapped_type_node(node) {
        return crate::ls::utilities::get_children_from_non_jsdoc_node(node, source_file);
    }

    let children = crate::ls::utilities::get_children_from_non_jsdoc_node(node, source_file);
    if children.len() < 2 {
        return children;
    }

    let open_brace_token = Arc::clone(&children[0]);
    let close_brace_token = Arc::clone(&children[children.len() - 1]);
    if open_brace_token.kind != SyntaxKind::OpenBraceToken
        || close_brace_token.kind != SyntaxKind::CloseBraceToken
    {
        return children;
    }

    let mapped_type = node.as_mapped_type_node();
    let inner = &children[1..children.len() - 1];

    let grouped_with_plus_minus_tokens = group_children(factory, inner, &|child: &Arc<Node>| {
        mapped_type
            .readonly_token
            .as_ref()
            .map_or(false, |t| Arc::ptr_eq(t, child))
            || child.kind == SyntaxKind::ReadonlyKeyword
            || mapped_type
                .question_token
                .as_ref()
                .map_or(false, |t| Arc::ptr_eq(t, child))
            || child.kind == SyntaxKind::QuestionToken
    });

    let grouped_with_brackets = group_children(
        factory,
        &grouped_with_plus_minus_tokens,
        &|child: &Arc<Node>| {
            child.kind == SyntaxKind::OpenBracketToken
                || child.kind == SyntaxKind::TypeParameter
                || child.kind == SyntaxKind::CloseBracketToken
        },
    );

    vec![
        open_brace_token,
        create_syntax_list(
            factory,
            &split_children(factory, &grouped_with_brackets, &|child: &Arc<Node>| {
                child.kind == SyntaxKind::ColonToken
            }, false),
        ),
        close_brace_token,
    ]
}

pub fn group_children(
    factory: &NodeFactory,
    children: &[Arc<Node>],
    group_on: &dyn Fn(&Arc<Node>) -> bool,
) -> Vec<Arc<Node>> {
    let mut result: Vec<Arc<Node>> = Vec::new();
    let mut group: Vec<Arc<Node>> = Vec::new();
    for child in children {
        if group_on(child) {
            group.push(Arc::clone(child));
        } else {
            if !group.is_empty() {
                result.push(create_syntax_list(factory, &group));
                group = Vec::new();
            }
            result.push(Arc::clone(child));
        }
    }
    if !group.is_empty() {
        result.push(create_syntax_list(factory, &group));
    }
    result
}

pub fn split_children(
    factory: &NodeFactory,
    children: &[Arc<Node>],
    pivot_on: &dyn Fn(&Arc<Node>) -> bool,
    separate_trailing_semicolon: bool,
) -> Vec<Arc<Node>> {
    if children.len() < 2 {
        return children.to_vec();
    }

    let mut split_token_index: isize = -1;
    for (i, child) in children.iter().enumerate() {
        if pivot_on(child) {
            split_token_index = i as isize;
            break;
        }
    }
    if split_token_index == -1 {
        return children.to_vec();
    }
    let split_token_index = split_token_index as usize;

    let left_children = &children[..split_token_index];
    let split_token = Arc::clone(&children[split_token_index]);
    let last_token = Arc::clone(&children[children.len() - 1]);
    let separate_last_token =
        separate_trailing_semicolon && last_token.kind == SyntaxKind::SemicolonToken;
    let mut right_end = children.len();
    if separate_last_token {
        right_end -= 1;
    }
    let right_children = &children[split_token_index + 1..right_end];

    let mut result: Vec<Arc<Node>> = Vec::with_capacity(4);
    if !left_children.is_empty() {
        result.push(create_syntax_list(factory, left_children));
    }
    result.push(split_token);
    if !right_children.is_empty() {
        result.push(create_syntax_list(factory, right_children));
    }
    if separate_last_token {
        result.push(last_token);
    }
    result
}

pub fn create_syntax_list(factory: &NodeFactory, children: &[Arc<Node>]) -> Arc<Node> {
    let mut list = factory.new_syntax_list(children);
    list.loc = TextRange::new(
        children[0].pos(),
        children[children.len() - 1].end(),
    );
    Arc::new(list)
}
