use super::Tracker;
use super::edit::NodeOptions;
use crate::ast::{Node, NodeList, SourceFile};
use crate::core::text::TextPos;
use std::sync::Arc;

impl Tracker {
    pub fn insert_node_in_list_after(
        &mut self,
        _source_file: &SourceFile,
        _after: &Arc<Node>,
        _new_node: &Arc<Node>,
        _containing_list: Option<&NodeList>,
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

    pub(super) fn insert_node_at_start_worker(
        &mut self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _new_element: &Arc<Node>,
    ) {
        todo!("insertNodeAtStartWorker")
    }

    pub(super) fn try_compute_indentation_for_new_member(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
    ) -> i32 {
        todo!("tryComputeIndentationForNewMember")
    }

    pub(super) fn try_compute_indentation_from_existing_members(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
    ) -> i32 {
        todo!("tryComputeIndentationFromExistingMembers")
    }

    pub(super) fn get_insert_node_after_options(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
    ) -> NodeOptions {
        todo!("getInsertNodeAfterOptions")
    }

    pub(super) fn get_options_for_insert_node_before(
        &self,
        _before: &Arc<Node>,
        _inserted: &Arc<Node>,
        _blank_line_between: bool,
    ) -> NodeOptions {
        todo!("getOptionsForInsertNodeBefore")
    }

    pub(super) fn get_insert_node_at_start_insert_options(
        &mut self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _indentation: i32,
    ) -> NodeOptions {
        todo!("getInsertNodeAtStartInsertOptions")
    }

    pub(crate) fn finish_nodes_with_insertions_at_start(&mut self) {}

    pub(crate) fn finish_delete_declarations(&mut self) {}

    pub(super) fn end_pos_for_insert_node_after(
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
