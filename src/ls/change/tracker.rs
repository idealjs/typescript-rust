#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::{Node, SourceFile};
use crate::core::compiler_options::CompilerOptions;
use crate::core::text::TextPos;
use crate::core::text::TextRange;
use crate::ls::lsconv::converters::Converters;
use crate::lsp::lsproto::lsp::{Position, Range, TextEdit};

use crate::ls::lsutil::format_code_options::FormatCodeSettings;

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
enum TrackerEditKind {
    Text = 1,
    Remove = 2,
    ReplaceWithSingleNode = 3,
    ReplaceWithMultipleNodes = 4,
}

#[derive(Debug, Clone)]
pub struct TrackerEdit {
    kind: TrackerEditKind,
    range: Range,

    new_text: String,

    node: Option<Arc<Node>>,

    nodes: Vec<Arc<Node>>,
    options: NodeOptions,
}

#[derive(Debug, Clone)]
pub struct NodesInsertedAtStartState {
    node: Arc<Node>,
    #[allow(dead_code)]
    source_file_file_name: String,
}

#[derive(Debug, Clone)]
pub struct DeletedNode {
    pub source_file_file_name: String,
    pub node: Arc<Node>,
}

pub struct Tracker {
    format_settings: FormatCodeSettings,
    new_line: String,
    converters: Option<Box<Converters>>,
    changes: HashMap<String, Vec<TrackerEdit>>,
    deleted_nodes: Vec<DeletedNode>,
    nodes_with_insertions_at_start: HashMap<u64, NodesInsertedAtStartState>,
}

impl std::fmt::Debug for Tracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tracker")
            .field("new_line", &self.new_line)
            .field("deleted_nodes", &self.deleted_nodes.len())
            .finish()
    }
}

pub fn new_tracker(
    _compiler_options: &CompilerOptions,
    format_options: FormatCodeSettings,
    converters: Option<Box<Converters>>,
) -> Tracker {

    Tracker {
        format_settings: format_options,
        new_line: "\n".to_string(),
        converters,
        changes: HashMap::new(),
        deleted_nodes: Vec::new(),
        nodes_with_insertions_at_start: HashMap::new(),
    }
}

impl Tracker {

    pub fn get_changes(&mut self) -> HashMap<String, Vec<TextEdit>> {

        HashMap::new()
    }

    pub fn replace_node(
        &mut self,
        _source_file: &SourceFile,
        _old_node: &Arc<Node>,
        _new_node: &Arc<Node>,
        _options: Option<&NodeOptions>,
    ) {
        todo!("ReplaceNode")
    }

    pub fn replace_node_with_nodes(
        &mut self,
        _source_file: &SourceFile,
        _old_node: &Arc<Node>,
        _new_nodes: &[Arc<Node>],
        _options: Option<&NodeOptions>,
    ) {
        todo!("ReplaceNodeWithNodes")
    }

    pub fn replace_range(
        &mut self,
        source_file: &SourceFile,
        lsproto_range: Range,
        _new_node: &Arc<Node>,
        options: NodeOptions,
    ) {
        self.push_edit(
            source_file.file_name.clone(),
            TrackerEdit {
                kind: TrackerEditKind::ReplaceWithSingleNode,
                range: lsproto_range,
                new_text: String::new(),
                node: None,
                nodes: Vec::new(),
                options,
            },
        );
    }

    pub fn replace_range_with_text(
        &mut self,
        source_file: &SourceFile,
        lsproto_range: Range,
        text: String,
    ) {
        self.push_edit(
            source_file.file_name.clone(),
            TrackerEdit {
                kind: TrackerEditKind::Text,
                range: lsproto_range,
                new_text: text,
                node: None,
                nodes: Vec::new(),
                options: NodeOptions::default(),
            },
        );
    }

    pub fn replace_range_with_nodes(
        &mut self,
        source_file: &SourceFile,
        lsproto_range: Range,
        new_nodes: &[Arc<Node>],
        options: NodeOptions,
    ) {
        if new_nodes.len() == 1 {
            self.replace_range(source_file, lsproto_range, &new_nodes[0], options);
            return;
        }
        self.push_edit(
            source_file.file_name.clone(),
            TrackerEdit {
                kind: TrackerEditKind::ReplaceWithMultipleNodes,
                range: lsproto_range,
                new_text: String::new(),
                node: None,
                nodes: new_nodes.to_vec(),
                options,
            },
        );
    }

    pub fn insert_text(&mut self, source_file: &SourceFile, pos: Position, text: String) {
        self.replace_range_with_text(
            source_file,
            Range {
                start: pos.clone(),
                end: pos,
            },
            text,
        );
    }

    pub fn insert_node_at(
        &mut self,
        _source_file: &SourceFile,
        _pos: TextPos,
        _new_node: &Arc<Node>,
        _options: NodeOptions,
    ) {
        todo!("InsertNodeAt")
    }

    pub fn insert_nodes_at(
        &mut self,
        _source_file: &SourceFile,
        _pos: TextPos,
        _new_nodes: &[Arc<Node>],
        _options: NodeOptions,
    ) {
        todo!("InsertNodesAt")
    }

    pub fn insert_node_after(
        &mut self,
        _source_file: &SourceFile,
        _after: &Arc<Node>,
        _new_node: &Arc<Node>,
    ) {
        todo!("InsertNodeAfter")
    }

    pub fn insert_nodes_after(
        &mut self,
        _source_file: &SourceFile,
        _after: &Arc<Node>,
        _new_nodes: &[Arc<Node>],
    ) {
        todo!("InsertNodesAfter")
    }

    pub fn insert_node_before(
        &mut self,
        _source_file: &SourceFile,
        _before: &Arc<Node>,
        _new_node: &Arc<Node>,
        _blank_line_between: bool,
        _leading_trivia_option: LeadingTriviaOption,
    ) {
        todo!("InsertNodeBefore")
    }

    pub fn try_insert_type_annotation(
        &mut self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _type_node: &Arc<Node>,
    ) -> bool {
        todo!("TryInsertTypeAnnotation")
    }

    pub fn parenthesize_arrow_parameters(
        &mut self,
        _source_file: &SourceFile,
        _arrow_func: &Arc<Node>,
    ) {
        todo!("ParenthesizeArrowParameters")
    }

    pub fn insert_modifier_before(
        &mut self,
        _source_file: &SourceFile,
        _modifier: crate::ast::SyntaxKind,
        _before: &Arc<Node>,
    ) {
        todo!("InsertModifierBefore")
    }

    pub fn delete(&mut self, source_file: &SourceFile, node: &Arc<Node>) {
        self.deleted_nodes.push(DeletedNode {
            source_file_file_name: source_file.file_name.clone(),
            node: Arc::clone(node),
        });
    }

    pub fn delete_range(&mut self, source_file: &SourceFile, text_range: TextRange) {
        let lsp_range = self.text_range_to_lsp(source_file, text_range);
        self.replace_range_with_text(source_file, lsp_range, String::new());
    }

    pub fn delete_node(
        &mut self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _leading_trivia: LeadingTriviaOption,
        _trailing_trivia: TrailingTriviaOption,
    ) {
        todo!("DeleteNode")
    }

    pub fn delete_node_range(
        &mut self,
        _source_file: &SourceFile,
        _start_node: &Arc<Node>,
        _end_node: &Arc<Node>,
        _leading_trivia: LeadingTriviaOption,
        _trailing_trivia: TrailingTriviaOption,
    ) {
        todo!("DeleteNodeRange")
    }

    pub(crate) fn changes(&self) -> &HashMap<String, Vec<TrackerEdit>> {
        &self.changes
    }
    pub(crate) fn changes_mut(&mut self) -> &mut HashMap<String, Vec<TrackerEdit>> {
        &mut self.changes
    }

    fn push_edit(&mut self, file_name: String, edit: TrackerEdit) {
        self.changes.entry(file_name).or_default().push(edit);
    }
    pub(crate) fn deleted_nodes_mut(&mut self) -> &mut Vec<DeletedNode> {
        &mut self.deleted_nodes
    }
    #[allow(dead_code)]
    pub(crate) fn format_settings(&self) -> &FormatCodeSettings {
        &self.format_settings
    }
    #[allow(dead_code)]
    pub(crate) fn new_line(&self) -> &str {
        &self.new_line
    }
    #[allow(dead_code)]
    pub(crate) fn nodes_with_insertions_at_start_mut(
        &mut self,
    ) -> &mut HashMap<u64, NodesInsertedAtStartState> {
        &mut self.nodes_with_insertions_at_start
    }

    fn text_range_to_lsp(&self, _source_file: &SourceFile, _text_range: TextRange) -> Range {

        Range::default()
    }
}

impl Tracker {

    pub fn insert_node_in_list_after(
        &mut self,
        _source_file: &SourceFile,
        _after: &Arc<Node>,
        _new_node: &Arc<Node>,
        _containing_list: Option<&crate::ast::NodeList>,
    ) {
        todo!("InsertNodeInListAfter")
    }

    pub fn insert_import_specifier_at_index(
        &mut self,
        _source_file: &SourceFile,
        _new_specifier: &Arc<Node>,
        _named_imports: &Arc<Node>,
        _index: usize,
    ) {
        todo!("InsertImportSpecifierAtIndex")
    }

    pub fn insert_at_top_of_file(
        &mut self,
        _source_file: &SourceFile,
        _insert: &[Arc<Node>],
        _blank_line_between: bool,
    ) {
        todo!("InsertAtTopOfFile")
    }

    pub fn insert_member_at_start(
        &mut self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _new_element: &Arc<Node>,
    ) {
        todo!("InsertMemberAtStart")
    }

    fn insert_node_at_start_worker(
        &mut self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _new_element: &Arc<Node>,
    ) {
        todo!("insertNodeAtStartWorker")
    }

    fn try_compute_indentation_for_new_member(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
    ) -> i32 {
        todo!("tryComputeIndentationForNewMember")
    }

    fn try_compute_indentation_from_existing_members(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
    ) -> i32 {
        todo!("tryComputeIndentationFromExistingMembers")
    }

    fn get_insert_node_after_options(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
    ) -> NodeOptions {
        todo!("getInsertNodeAfterOptions")
    }

    fn get_options_for_insert_node_before(
        &self,
        _before: &Arc<Node>,
        _inserted: &Arc<Node>,
        _blank_line_between: bool,
    ) -> NodeOptions {
        todo!("getOptionsForInsertNodeBefore")
    }

    fn get_insert_node_at_start_insert_options(
        &mut self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _indentation: i32,
    ) -> NodeOptions {
        todo!("getInsertNodeAtStartInsertOptions")
    }

    pub(crate) fn finish_nodes_with_insertions_at_start(&mut self) {

    }

    pub(crate) fn finish_delete_declarations(&mut self) {

    }

    fn end_pos_for_insert_node_after(
        &mut self,
        _source_file: &SourceFile,
        _after: &Arc<Node>,
        _new_node: &Arc<Node>,
    ) -> TextPos {
        todo!("endPosForInsertNodeAfter")
    }

    pub(crate) fn start_position_to_delete_node_in_list(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
    ) -> usize {
        todo!("startPositionToDeleteNodeInList")
    }

    pub(crate) fn end_position_to_delete_node_in_list(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _prev_node: Option<&Arc<Node>>,
        _next_node: &Arc<Node>,
    ) -> usize {
        todo!("endPositionToDeleteNodeInList")
    }
}

#[allow(dead_code)]
pub fn need_semicolon_between(_a: &Arc<Node>, _b: &Arc<Node>) -> bool {

    false
}

#[allow(dead_code)]
pub fn is_separator(_node: &Arc<Node>, _candidate: Option<&Arc<Node>>) -> bool {

    false
}

pub fn range_contains_range_exclusive(outer: &Arc<Node>, inner: &Arc<Node>) -> bool {
    outer.pos() < inner.pos() && inner.end() < outer.end()
}

#[allow(dead_code)]
pub fn get_members_or_properties(_node: &Arc<Node>) -> Option<crate::ast::NodeList> {

    None
}

#[allow(dead_code)]
fn find_indentation_column(
    _text: &str,
    _line_start: usize,
    _member_start: usize,
    _tab_size: i32,
) -> i32 {

    0
}

#[allow(dead_code)]
fn advance_indentation_column(column: i32, ch: char, tab_size: i32) -> i32 {
    if ch == '\t' {
        column + tab_size - (column % tab_size)
    } else {
        column + 1
    }
}

#[allow(dead_code)]
pub fn has_comments_before_line_break(text: &str, start: usize) -> bool {
    for ch in text[start..].chars() {
        if !crate::stringutil::is_white_space_single_line(ch) {
            return ch == '/';
        }
    }
    false
}
