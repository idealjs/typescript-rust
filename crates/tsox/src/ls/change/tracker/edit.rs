use crate::ast::Node;
use crate::lsp::lsproto::lsp::Range;
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct NodeOptions {
    pub prefix: String,

    pub suffix: String,

    pub indentation: Option<i32>,

    pub delta: Option<i32>,
    pub leading_trivia_option: LeadingTriviaOption,
    pub trailing_trivia_option: TrailingTriviaOption,
    pub joiner: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum LeadingTriviaOption {
    #[default]
    None = 0,
    Exclude = 1,
    IncludeAll = 2,
    JSDoc = 3,
    StartLine = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum TrailingTriviaOption {
    #[default]
    None = 0,
    Exclude = 1,
    ExcludeWhitespace = 2,
    Include = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub(super) enum TrackerEditKind {
    Text = 1,
    Remove = 2,
    ReplaceWithSingleNode = 3,
    ReplaceWithMultipleNodes = 4,
}

#[derive(Debug, Clone)]
pub struct TrackerEdit {
    pub(super) kind: TrackerEditKind,
    pub(super) range: Range,

    pub(super) new_text: String,

    pub(super) node: Option<Arc<Node>>,

    pub(super) nodes: Vec<Arc<Node>>,
    pub(super) options: NodeOptions,
}

#[derive(Debug, Clone)]
pub struct NodesInsertedAtStartState {
    pub(super) node: Arc<Node>,
    #[allow(dead_code)]
    pub(super) source_file_file_name: String,
}

#[derive(Debug, Clone)]
pub struct DeletedNode {
    pub source_file_file_name: String,
    pub node: Arc<Node>,
}
