#![allow(dead_code)]

use crate::ls::change_tracker::*;

pub struct Tracker {
    pub(super) format_settings: FormatCodeSettings,
    pub(crate) new_line: String,
    pub(crate) converters: Option<Box<Converters>>,
    pub(super) changes: HashMap<String, Vec<TrackerEdit>>,
    pub(super) deleted_nodes: Vec<DeletedNode>,
    pub(super) nodes_with_insertions_at_start: HashMap<u64, NodesInsertedAtStartState>,
    pub(crate) unmappable_files: std::collections::HashSet<String>,
}

impl std::fmt::Debug for Tracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { ::tsox_core::fntrace::enter("fmt"); 
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
) -> Tracker { ::tsox_core::fntrace::enter("new_tracker"); 
    Tracker {
        format_settings: format_options,
        new_line: "\n".to_string(),
        converters,
        changes: HashMap::new(),
        deleted_nodes: Vec::new(),
        nodes_with_insertions_at_start: HashMap::new(),
        unmappable_files: std::collections::HashSet::new(),
    }
}

impl Tracker {
    pub fn get_changes(&mut self) -> HashMap<String, Vec<TextEdit>> { ::tsox_core::fntrace::enter("get_changes"); 
        self.get_changes_with_unmappable().0
    }

    pub fn get_changes_with_unmappable(&mut self) -> (HashMap<String, Vec<TextEdit>>, Vec<String>) { ::tsox_core::fntrace::enter("get_changes_with_unmappable"); 
        let mut changes = self.get_text_changes_from_changes();
        let mut unmappable: Vec<String> =
            self.unmappable_files.iter().cloned().collect();
        unmappable.sort();
        for file_name in &unmappable {
            changes.remove(file_name);
        }
        (changes, unmappable)
    }

    pub fn replace_node(
        &mut self,
        _source_file: &SourceFile,
        _old_node: &Arc<Node>,
        _new_node: &Arc<Node>,
        _options: Option<&NodeOptions>,
    ) { ::tsox_core::fntrace::enter("replace_node"); 
        todo!("ReplaceNode")
    }

    pub fn replace_node_with_nodes(
        &mut self,
        _source_file: &SourceFile,
        _old_node: &Arc<Node>,
        _new_nodes: &[Arc<Node>],
        _options: Option<&NodeOptions>,
    ) { ::tsox_core::fntrace::enter("replace_node_with_nodes"); 
        todo!("ReplaceNodeWithNodes")
    }

    pub fn replace_range(
        &mut self,
        source_file: &SourceFile,
        lsproto_range: Range,
        _new_node: &Arc<Node>,
        options: NodeOptions,
    ) { ::tsox_core::fntrace::enter("replace_range"); 
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
    ) { ::tsox_core::fntrace::enter("replace_range_with_text"); 
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
    ) { ::tsox_core::fntrace::enter("replace_range_with_nodes"); 
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

    pub fn insert_text(&mut self, source_file: &SourceFile, pos: Position, text: String) { ::tsox_core::fntrace::enter("insert_text"); 
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
    ) { ::tsox_core::fntrace::enter("insert_node_at"); 
        todo!("InsertNodeAt")
    }

    pub fn insert_nodes_at(
        &mut self,
        _source_file: &SourceFile,
        _pos: TextPos,
        _new_nodes: &[Arc<Node>],
        _options: NodeOptions,
    ) { ::tsox_core::fntrace::enter("insert_nodes_at"); 
        todo!("InsertNodesAt")
    }

    pub fn insert_node_after(
        &mut self,
        _source_file: &SourceFile,
        _after: &Arc<Node>,
        _new_node: &Arc<Node>,
    ) { ::tsox_core::fntrace::enter("insert_node_after"); 
        todo!("InsertNodeAfter")
    }

    pub fn insert_nodes_after(
        &mut self,
        _source_file: &SourceFile,
        _after: &Arc<Node>,
        _new_nodes: &[Arc<Node>],
    ) { ::tsox_core::fntrace::enter("insert_nodes_after"); 
        todo!("InsertNodesAfter")
    }

    pub fn insert_node_before(
        &mut self,
        _source_file: &SourceFile,
        _before: &Arc<Node>,
        _new_node: &Arc<Node>,
        _blank_line_between: bool,
        _leading_trivia_option: LeadingTriviaOption,
    ) { ::tsox_core::fntrace::enter("insert_node_before"); 
        todo!("InsertNodeBefore")
    }

    pub fn try_insert_type_annotation(
        &mut self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _type_node: &Arc<Node>,
    ) -> bool { ::tsox_core::fntrace::enter("try_insert_type_annotation"); 
        todo!("TryInsertTypeAnnotation")
    }

    pub fn parenthesize_arrow_parameters(
        &mut self,
        _source_file: &SourceFile,
        _arrow_func: &Arc<Node>,
    ) { ::tsox_core::fntrace::enter("parenthesize_arrow_parameters"); 
        todo!("ParenthesizeArrowParameters")
    }

    pub fn insert_modifier_before(
        &mut self,
        _source_file: &SourceFile,
        _modifier: tsox_frontend::ast::SyntaxKind,
        _before: &Arc<Node>,
    ) { ::tsox_core::fntrace::enter("insert_modifier_before"); 
        todo!("InsertModifierBefore")
    }

    pub fn delete(&mut self, source_file: &SourceFile, node: &Arc<Node>) { ::tsox_core::fntrace::enter("delete"); 
        self.deleted_nodes.push(DeletedNode {
            source_file_file_name: source_file.file_name.clone(),
            node: Arc::clone(node),
        });
    }

    pub fn delete_range(&mut self, source_file: &SourceFile, text_range: TextRange) { ::tsox_core::fntrace::enter("delete_range"); 
        let lsp_range = self.text_range_to_lsp(source_file, text_range);
        self.replace_range_with_text(source_file, lsp_range, String::new());
    }

    pub fn delete_node(
        &mut self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _leading_trivia: LeadingTriviaOption,
        _trailing_trivia: TrailingTriviaOption,
    ) { ::tsox_core::fntrace::enter("delete_node"); 
        todo!("DeleteNode")
    }

    pub fn delete_node_range(
        &mut self,
        _source_file: &SourceFile,
        _start_node: &Arc<Node>,
        _end_node: &Arc<Node>,
        _leading_trivia: LeadingTriviaOption,
        _trailing_trivia: TrailingTriviaOption,
    ) { ::tsox_core::fntrace::enter("delete_node_range"); 
        todo!("DeleteNodeRange")
    }

    pub(crate) fn changes(&self) -> &HashMap<String, Vec<TrackerEdit>> { ::tsox_core::fntrace::enter("changes"); 
        &self.changes
    }
    pub(crate) fn changes_mut(&mut self) -> &mut HashMap<String, Vec<TrackerEdit>> { ::tsox_core::fntrace::enter("changes_mut"); 
        &mut self.changes
    }

    pub(super) fn push_edit(&mut self, file_name: String, edit: TrackerEdit) { ::tsox_core::fntrace::enter("push_edit"); 
        self.changes.entry(file_name).or_default().push(edit);
    }
    pub(crate) fn deleted_nodes_mut(&mut self) -> &mut Vec<DeletedNode> { ::tsox_core::fntrace::enter("deleted_nodes_mut"); 
        &mut self.deleted_nodes
    }
    #[allow(dead_code)]
    pub(crate) fn format_settings(&self) -> &FormatCodeSettings { ::tsox_core::fntrace::enter("format_settings"); 
        &self.format_settings
    }
    #[allow(dead_code)]
    pub(crate) fn new_line(&self) -> &str { ::tsox_core::fntrace::enter("new_line"); 
        &self.new_line
    }
    #[allow(dead_code)]
    pub(crate) fn nodes_with_insertions_at_start_mut(
        &mut self,
    ) -> &mut HashMap<u64, NodesInsertedAtStartState> { ::tsox_core::fntrace::enter("nodes_with_insertions_at_start_mut"); 
        &mut self.nodes_with_insertions_at_start
    }

    pub(super) fn text_range_to_lsp(
        &self,
        _source_file: &SourceFile,
        _text_range: TextRange,
    ) -> Range { ::tsox_core::fntrace::enter("text_range_to_lsp"); 
        Range::default()
    }
}
