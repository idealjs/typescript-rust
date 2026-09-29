#![allow(dead_code, unused_imports, unused_variables)]

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::Node;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrackerEditKind {
    #[default]
    Remove,
    Text,
    ReplaceWithSingleNode,
    ReplaceWithMultipleNodes,
}

#[derive(Debug, Clone, Default)]
pub struct NodeOptions {
    pub prefix: String,
    pub suffix: String,
    pub indentation: Option<usize>,
    pub joiner: String,
    pub leading_trivia_option: LeadingTriviaOption,
    pub trailing_trivia_option: TrailingTriviaOption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LeadingTriviaOption {
    #[default]
    Include,
    Exclude,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrailingTriviaOption {
    #[default]
    Include,
    Exclude,
}

#[derive(Debug, Clone)]
pub struct TrackerEdit {
    pub kind: TrackerEditKind,
    pub text_range: TextRange,
    pub new_text: String,
    pub node: Option<Arc<Node>>,
    pub nodes: Vec<Arc<Node>>,
    pub options: NodeOptions,
}

impl Default for TrackerEdit {
    fn default() -> Self {
        TrackerEdit {
            kind: TrackerEditKind::default(),
            text_range: TextRange::new(0, 0),
            new_text: String::new(),
            node: None,
            nodes: Vec::new(),
            options: NodeOptions::default(),
        }
    }
}
