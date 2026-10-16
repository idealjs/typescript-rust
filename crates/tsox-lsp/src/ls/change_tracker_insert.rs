use crate::ls::change_tracker::Tracker;
use crate::ls::change_tracker_edit::NodeOptions;
use std::sync::Arc;
use tsox_core::core::text::TextPos;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::SourceFile;

#[allow(dead_code)]
impl Tracker {
    pub fn insert_node_in_list_after(
        &mut self,
        _source_file: &SourceFile,
        _after: &Arc<Node>,
        _new_node: &Arc<Node>,
        _containing_list: Option<&NodeList>,
    ) { ::tsox_core::fntrace::enter("insert_node_in_list_after"); 
        todo!("InsertNodeInListAfter")
    }

    pub fn insert_import_specifier_at_index(
        &mut self,
        _source_file: &SourceFile,
        _new_specifier: &Arc<Node>,
        _named_imports: &Arc<Node>,
        _index: usize,
    ) { ::tsox_core::fntrace::enter("insert_import_specifier_at_index"); 
        todo!("InsertImportSpecifierAtIndex")
    }

    pub fn insert_at_top_of_file(
        &mut self,
        _source_file: &SourceFile,
        _insert: &[Arc<Node>],
        _blank_line_between: bool,
    ) { ::tsox_core::fntrace::enter("insert_at_top_of_file"); 
        todo!("InsertAtTopOfFile")
    }

    pub fn insert_member_at_start(
        &mut self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _new_element: &Arc<Node>,
    ) { ::tsox_core::fntrace::enter("insert_member_at_start"); 
        todo!("InsertMemberAtStart")
    }

    pub(super) fn insert_node_at_start_worker(
        &mut self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _new_element: &Arc<Node>,
    ) { ::tsox_core::fntrace::enter("insert_node_at_start_worker"); 
        todo!("insertNodeAtStartWorker")
    }

    pub(super) fn try_compute_indentation_for_new_member(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
    ) -> i32 { ::tsox_core::fntrace::enter("try_compute_indentation_for_new_member"); 
        todo!("tryComputeIndentationForNewMember")
    }

    pub(super) fn try_compute_indentation_from_existing_members(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
    ) -> i32 { ::tsox_core::fntrace::enter("try_compute_indentation_from_existing_members"); 
        todo!("tryComputeIndentationFromExistingMembers")
    }

    pub(super) fn get_insert_node_after_options(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
    ) -> NodeOptions { ::tsox_core::fntrace::enter("get_insert_node_after_options"); 
        todo!("getInsertNodeAfterOptions")
    }

    pub(super) fn get_options_for_insert_node_before(
        &self,
        _before: &Arc<Node>,
        _inserted: &Arc<Node>,
        _blank_line_between: bool,
    ) -> NodeOptions { ::tsox_core::fntrace::enter("get_options_for_insert_node_before"); 
        todo!("getOptionsForInsertNodeBefore")
    }

    pub(super) fn get_insert_node_at_start_insert_options(
        &mut self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _indentation: i32,
    ) -> NodeOptions { ::tsox_core::fntrace::enter("get_insert_node_at_start_insert_options"); 
        todo!("getInsertNodeAtStartInsertOptions")
    }

    pub(crate) fn finish_nodes_with_insertions_at_start(&mut self) { ::tsox_core::fntrace::enter("finish_nodes_with_insertions_at_start"); }

    pub(crate) fn finish_delete_declarations(&mut self) { ::tsox_core::fntrace::enter("finish_delete_declarations"); }

    pub(super) fn end_pos_for_insert_node_after(
        &mut self,
        _source_file: &SourceFile,
        _after: &Arc<Node>,
        _new_node: &Arc<Node>,
    ) -> TextPos { ::tsox_core::fntrace::enter("end_pos_for_insert_node_after"); 
        todo!("endPosForInsertNodeAfter")
    }

    pub(crate) fn start_position_to_delete_node_in_list(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
    ) -> usize { ::tsox_core::fntrace::enter("start_position_to_delete_node_in_list"); 
        todo!("startPositionToDeleteNodeInList")
    }

    pub(crate) fn end_position_to_delete_node_in_list(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _prev_node: Option<&Arc<Node>>,
        _next_node: &Arc<Node>,
    ) -> usize { ::tsox_core::fntrace::enter("end_position_to_delete_node_in_list"); 
        todo!("endPositionToDeleteNodeInList")
    }
}
