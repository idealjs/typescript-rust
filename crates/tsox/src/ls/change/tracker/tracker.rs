#![allow(dead_code)]

use super::*;

pub struct Tracker {
    pub(super) format_settings: FormatCodeSettings,
    pub(super) new_line: String,
    pub(super) converters: Option<Box<Converters>>,
    pub(super) changes: HashMap<String, Vec<TrackerEdit>>,
    pub(super) deleted_nodes: Vec<DeletedNode>,
    pub(super) nodes_with_insertions_at_start: HashMap<u64, NodesInsertedAtStartState>,
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

    pub(super) fn push_edit(&mut self, file_name: String, edit: TrackerEdit) {
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

    pub(super) fn text_range_to_lsp(
        &self,
        _source_file: &SourceFile,
        _text_range: TextRange,
    ) -> Range {
        Range::default()
    }
}
