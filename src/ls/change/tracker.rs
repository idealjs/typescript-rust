//! Change tracker — accumulates text edits.
//!
//! Ported from `internal/ls/change/tracker.go`. The `Tracker` accumulates
//! insertions, replacements, and deletions against source files and produces
//! `TextEdit` maps on `get_changes`. The full implementation depends on the
//! printer (`EmitContext`, `NodeFactory`, `ChangeTrackerWriter`), the format
//! engine, and `lsconv::Converters`, none of which are ported yet; method
//! bodies are stubbed (`todo!()`) and the struct omits those fields.

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

/// Text to insert before/after a new node, plus trivia options.
///
/// Mirrors `NodeOptions` in Go.
#[derive(Debug, Clone, Default)]
pub struct NodeOptions {
    /// Text to be inserted before the new node.
    pub prefix: String,
    /// Text to be inserted after the new node.
    pub suffix: String,
    /// Text of inserted node will be formatted with this indentation, otherwise
    /// indentation will be inferred from the old node.
    pub indentation: Option<i32>,
    /// Text of inserted node will be formatted with this delta, otherwise delta
    /// will be inferred from the new node kind.
    pub delta: Option<i32>,
    pub leading_trivia_option: LeadingTriviaOption,
    pub trailing_trivia_option: TrailingTriviaOption,
    pub joiner: String,
}

/// How leading trivia is handled when replacing/inserting a node.
///
/// Mirrors `LeadingTriviaOption` in Go.
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

/// How trailing trivia is handled when replacing/inserting a node.
///
/// Mirrors `TrailingTriviaOption` in Go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum TrailingTriviaOption {
    #[default]
    None = 0,
    Exclude = 1,
    ExcludeWhitespace = 2,
    Include = 3,
}

/// The kind of a queued tracker edit.
///
/// Mirrors `trackerEditKind` in Go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
enum TrackerEditKind {
    Text = 1,
    Remove = 2,
    ReplaceWithSingleNode = 3,
    ReplaceWithMultipleNodes = 4,
}

/// A single queued edit.
///
/// Mirrors `trackerEdit` in Go.
#[derive(Debug, Clone)]
struct TrackerEdit {
    kind: TrackerEditKind,
    range: Range,
    /// `kind == Text`.
    new_text: String,
    /// `kind == ReplaceWithSingleNode`.
    node: Option<Arc<Node>>,
    /// `kind == ReplaceWithMultipleNodes`.
    nodes: Vec<Arc<Node>>,
    options: NodeOptions,
}

/// State tracked for nodes that have had members inserted at their start.
///
/// Mirrors `nodesInsertedAtStartState` in Go.
#[derive(Debug, Clone)]
struct NodesInsertedAtStartState {
    node: Arc<Node>,
    #[allow(dead_code)]
    source_file_file_name: String,
}

/// A node queued for deletion.
///
/// Mirrors `deletedNode` in Go.
#[derive(Debug, Clone)]
pub struct DeletedNode {
    pub source_file_file_name: String,
    pub node: Arc<Node>,
}

/// Accumulates text edits across one or more source files.
///
/// Mirrors `Tracker` in Go. The Go struct embeds `*printer.EmitContext` and
/// `*ast.NodeFactory`; those are not yet ported, so they are omitted here. The
/// `changes` map is keyed by file name (a simplification of Go's
/// `*ast.SourceFile` keying).
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

/// Construct a new tracker.
///
/// Mirrors `NewTracker` in Go.
pub fn new_tracker(
    _compiler_options: &CompilerOptions,
    format_options: FormatCodeSettings,
    converters: Option<Box<Converters>>,
) -> Tracker {
    // TODO: wire up printer.EmitContext + NodeFactory + format context.
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
    /// Returns the accumulated text edits.
    ///
    /// Note: after calling this, the tracker should be discarded.
    ///
    /// Mirrors `Tracker.GetChanges` in Go.
    pub fn get_changes(&mut self) -> HashMap<String, Vec<TextEdit>> {
        // TODO: finish_delete_declarations + finish_nodes_with_insertions_at_start +
        // get_text_changes_from_changes.
        HashMap::new()
    }

    /// Mirrors `Tracker.ReplaceNode` in Go.
    pub fn replace_node(
        &mut self,
        _source_file: &SourceFile,
        _old_node: &Arc<Node>,
        _new_node: &Arc<Node>,
        _options: Option<&NodeOptions>,
    ) {
        todo!("ReplaceNode")
    }

    /// Mirrors `Tracker.ReplaceNodeWithNodes` in Go.
    pub fn replace_node_with_nodes(
        &mut self,
        _source_file: &SourceFile,
        _old_node: &Arc<Node>,
        _new_nodes: &[Arc<Node>],
        _options: Option<&NodeOptions>,
    ) {
        todo!("ReplaceNodeWithNodes")
    }

    /// Mirrors `Tracker.ReplaceRange` in Go.
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

    /// Mirrors `Tracker.ReplaceRangeWithText` in Go.
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

    /// Mirrors `Tracker.ReplaceRangeWithNodes` in Go.
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

    /// Mirrors `Tracker.InsertText` in Go.
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

    /// Mirrors `Tracker.InsertNodeAt` in Go.
    pub fn insert_node_at(
        &mut self,
        _source_file: &SourceFile,
        _pos: TextPos,
        _new_node: &Arc<Node>,
        _options: NodeOptions,
    ) {
        todo!("InsertNodeAt")
    }

    /// Mirrors `Tracker.InsertNodesAt` in Go.
    pub fn insert_nodes_at(
        &mut self,
        _source_file: &SourceFile,
        _pos: TextPos,
        _new_nodes: &[Arc<Node>],
        _options: NodeOptions,
    ) {
        todo!("InsertNodesAt")
    }

    /// Mirrors `Tracker.InsertNodeAfter` in Go.
    pub fn insert_node_after(
        &mut self,
        _source_file: &SourceFile,
        _after: &Arc<Node>,
        _new_node: &Arc<Node>,
    ) {
        todo!("InsertNodeAfter")
    }

    /// Mirrors `Tracker.InsertNodesAfter` in Go.
    pub fn insert_nodes_after(
        &mut self,
        _source_file: &SourceFile,
        _after: &Arc<Node>,
        _new_nodes: &[Arc<Node>],
    ) {
        todo!("InsertNodesAfter")
    }

    /// Mirrors `Tracker.InsertNodeBefore` in Go.
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

    /// Inserts a type annotation after the appropriate position on a node.
    ///
    /// Mirrors `Tracker.TryInsertTypeAnnotation` in Go.
    pub fn try_insert_type_annotation(
        &mut self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _type_node: &Arc<Node>,
    ) -> bool {
        todo!("TryInsertTypeAnnotation")
    }

    /// Wraps the parameters of a paren-less arrow function in `(` and `)`.
    ///
    /// Mirrors `Tracker.ParenthesizeArrowParameters` in Go.
    pub fn parenthesize_arrow_parameters(
        &mut self,
        _source_file: &SourceFile,
        _arrow_func: &Arc<Node>,
    ) {
        todo!("ParenthesizeArrowParameters")
    }

    /// Inserts a modifier token (like `type`) before a node with a trailing space.
    ///
    /// Mirrors `Tracker.InsertModifierBefore` in Go.
    pub fn insert_modifier_before(
        &mut self,
        _source_file: &SourceFile,
        _modifier: crate::ast::SyntaxKind,
        _before: &Arc<Node>,
    ) {
        todo!("InsertModifierBefore")
    }

    /// Queues a node for deletion with smart handling of list items, imports, etc.
    ///
    /// Mirrors `Tracker.Delete` in Go.
    pub fn delete(&mut self, source_file: &SourceFile, node: &Arc<Node>) {
        self.deleted_nodes.push(DeletedNode {
            source_file_file_name: source_file.file_name.clone(),
            node: Arc::clone(node),
        });
    }

    /// Deletes a text range from the source file.
    ///
    /// Mirrors `Tracker.DeleteRange` in Go.
    pub fn delete_range(&mut self, source_file: &SourceFile, text_range: TextRange) {
        let lsp_range = self.text_range_to_lsp(source_file, text_range);
        self.replace_range_with_text(source_file, lsp_range, String::new());
    }

    /// Deletes a node immediately with specified trivia options.
    ///
    /// Mirrors `Tracker.DeleteNode` in Go.
    pub fn delete_node(
        &mut self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _leading_trivia: LeadingTriviaOption,
        _trailing_trivia: TrailingTriviaOption,
    ) {
        todo!("DeleteNode")
    }

    /// Deletes a range of nodes with specified trivia options.
    ///
    /// Mirrors `Tracker.DeleteNodeRange` in Go.
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

    // Internal accessor used by delete.rs and tracker_impl.rs.
    pub(crate) fn changes(&self) -> &HashMap<String, Vec<TrackerEdit>> {
        &self.changes
    }
    pub(crate) fn changes_mut(&mut self) -> &mut HashMap<String, Vec<TrackerEdit>> {
        &mut self.changes
    }
    /// Append an edit to the per-file edit list.
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

    /// Convert a compiler text range to an LSP range.
    ///
    /// Mirrors the converters usage in Go. Stubbed until `Converters` is wired
    /// into the tracker; returns a default range.
    fn text_range_to_lsp(&self, _source_file: &SourceFile, _text_range: TextRange) -> Range {
        // TODO: self.converters.to_lsp_range(...)
        Range::default()
    }
}

// ---------------------------------------------------------------------------
// Additional `tracker.go` methods and free functions (stubbed).
// ---------------------------------------------------------------------------

impl Tracker {
    /// Mirrors `Tracker.InsertNodeInListAfter` in Go.
    pub fn insert_node_in_list_after(
        &mut self,
        _source_file: &SourceFile,
        _after: &Arc<Node>,
        _new_node: &Arc<Node>,
        _containing_list: Option<&crate::ast::NodeList>,
    ) {
        todo!("InsertNodeInListAfter")
    }

    /// Mirrors `Tracker.InsertImportSpecifierAtIndex` in Go.
    pub fn insert_import_specifier_at_index(
        &mut self,
        _source_file: &SourceFile,
        _new_specifier: &Arc<Node>,
        _named_imports: &Arc<Node>,
        _index: usize,
    ) {
        todo!("InsertImportSpecifierAtIndex")
    }

    /// Mirrors `Tracker.InsertAtTopOfFile` in Go.
    pub fn insert_at_top_of_file(
        &mut self,
        _source_file: &SourceFile,
        _insert: &[Arc<Node>],
        _blank_line_between: bool,
    ) {
        todo!("InsertAtTopOfFile")
    }

    /// Mirrors `Tracker.InsertMemberAtStart` in Go.
    pub fn insert_member_at_start(
        &mut self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _new_element: &Arc<Node>,
    ) {
        todo!("InsertMemberAtStart")
    }

    /// Mirrors `Tracker.insertNodeAtStartWorker` in Go.
    fn insert_node_at_start_worker(
        &mut self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _new_element: &Arc<Node>,
    ) {
        todo!("insertNodeAtStartWorker")
    }

    /// Mirrors `Tracker.tryComputeIndentationForNewMember` in Go.
    fn try_compute_indentation_for_new_member(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
    ) -> i32 {
        todo!("tryComputeIndentationForNewMember")
    }

    /// Mirrors `Tracker.tryComputeIndentationFromExistingMembers` in Go.
    fn try_compute_indentation_from_existing_members(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
    ) -> i32 {
        todo!("tryComputeIndentationFromExistingMembers")
    }

    /// Mirrors `Tracker.getInsertNodeAfterOptions` in Go.
    fn get_insert_node_after_options(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
    ) -> NodeOptions {
        todo!("getInsertNodeAfterOptions")
    }

    /// Mirrors `Tracker.getOptionsForInsertNodeBefore` in Go.
    fn get_options_for_insert_node_before(
        &self,
        _before: &Arc<Node>,
        _inserted: &Arc<Node>,
        _blank_line_between: bool,
    ) -> NodeOptions {
        todo!("getOptionsForInsertNodeBefore")
    }

    /// Mirrors `Tracker.getInsertNodeAtStartInsertOptions` in Go.
    fn get_insert_node_at_start_insert_options(
        &mut self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _indentation: i32,
    ) -> NodeOptions {
        todo!("getInsertNodeAtStartInsertOptions")
    }

    /// Mirrors `Tracker.finishNodesWithInsertionsAtStart` in Go.
    pub(crate) fn finish_nodes_with_insertions_at_start(&mut self) {
        // TODO: requires astnav.FindChildOfKind + converters.
    }

    /// Mirrors `Tracker.finishDeleteDeclarations` in Go.
    pub(crate) fn finish_delete_declarations(&mut self) {
        // Delegates to delete.rs::finish_delete_declarations once wired.
    }

    /// Mirrors `Tracker.endPosForInsertNodeAfter` in Go.
    fn end_pos_for_insert_node_after(
        &mut self,
        _source_file: &SourceFile,
        _after: &Arc<Node>,
        _new_node: &Arc<Node>,
    ) -> TextPos {
        todo!("endPosForInsertNodeAfter")
    }

    /// Mirrors `Tracker.startPositionToDeleteNodeInList` in Go.
    pub(crate) fn start_position_to_delete_node_in_list(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
    ) -> usize {
        todo!("startPositionToDeleteNodeInList")
    }

    /// Mirrors `Tracker.endPositionToDeleteNodeInList` in Go.
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

/// Whether two nodes need a semicolon between them.
///
/// Mirrors `needSemicolonBetween` in Go.
#[allow(dead_code)]
pub fn need_semicolon_between(_a: &Arc<Node>, _b: &Arc<Node>) -> bool {
    // TODO: requires ast.IsPropertySignatureDeclaration etc.
    false
}

/// Whether `candidate` is a list separator for `node`.
///
/// Mirrors `isSeparator` in Go.
#[allow(dead_code)]
pub fn is_separator(_node: &Arc<Node>, _candidate: Option<&Arc<Node>>) -> bool {
    // TODO: requires node.Parent + kind checks.
    false
}

/// Whether `outer` contains `inner` (exclusive on both ends).
///
/// Mirrors `rangeContainsRangeExclusive` in Go.
pub fn range_contains_range_exclusive(outer: &Arc<Node>, inner: &Arc<Node>) -> bool {
    outer.pos() < inner.pos() && inner.end() < outer.end()
}

/// Returns the member/property list of `node`.
///
/// Mirrors `getMembersOrProperties` in Go.
#[allow(dead_code)]
pub fn get_members_or_properties(_node: &Arc<Node>) -> Option<crate::ast::NodeList> {
    // TODO: requires node.MemberList() / PropertyList() accessors.
    None
}

/// Compute the indentation column of text between `line_start` and `member_start`.
///
/// Mirrors `findIndentationColumn` in Go.
#[allow(dead_code)]
fn find_indentation_column(
    _text: &str,
    _line_start: usize,
    _member_start: usize,
    _tab_size: i32,
) -> i32 {
    // TODO: requires stringutil whitespace helpers.
    0
}

/// Advance an indentation column by one character/tab.
///
/// Mirrors `advanceIndentationColumn` in Go.
#[allow(dead_code)]
fn advance_indentation_column(column: i32, ch: char, tab_size: i32) -> i32 {
    if ch == '\t' {
        column + tab_size - (column % tab_size)
    } else {
        column + 1
    }
}

/// Whether `text[start..]` has a comment before the next line break.
///
/// Mirrors `hasCommentsBeforeLineBreak` in Go.
#[allow(dead_code)]
pub fn has_comments_before_line_break(text: &str, start: usize) -> bool {
    for ch in text[start..].chars() {
        if !crate::stringutil::is_white_space_single_line(ch) {
            return ch == '/';
        }
    }
    false
}
