#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, SourceFile};
use crate::lsp::lsproto::lsp::{Range, TextEdit};

use crate::ls::lsutil::format_code_options::{FormatCodeSettings, SemicolonPreference};
use crate::ls::lsutil::utilities::probably_uses_semicolons;

use super::tracker::{LeadingTriviaOption, NodeOptions, Tracker, TrailingTriviaOption};

impl Tracker {

    pub(crate) fn get_text_changes_from_changes(
        &self,
    ) -> std::collections::HashMap<String, Vec<TextEdit>> {

        std::collections::HashMap::new()
    }

    pub(crate) fn compute_new_text(
        &self,
        _change_kind: i32,
        _range: Range,
        _new_text: &str,
        _node: Option<&Arc<Node>>,
        _nodes: &[Arc<Node>],
        _options: &NodeOptions,
        _target_source_file: &SourceFile,
        _source_file: &SourceFile,
    ) -> String {
        todo!("computeNewText")
    }

    pub(crate) fn get_formatted_text_of_node(
        &self,
        _node_in: &Arc<Node>,
        _target_source_file: &SourceFile,
        _source_file: &SourceFile,
        _pos: usize,
        _options: &NodeOptions,
    ) -> String {
        todo!("getFormattedTextOfNode")
    }

    pub(crate) fn get_nonformatted_text(
        &mut self,
        _node: &Arc<Node>,
        _source_file: &SourceFile,
    ) -> (String, Arc<Node>) {
        todo!("getNonformattedText")
    }

    pub fn get_adjusted_range(
        &self,
        _source_file: &SourceFile,
        _start_node: &Arc<Node>,
        _end_node: &Arc<Node>,
        _leading_option: LeadingTriviaOption,
        _trailing_option: TrailingTriviaOption,
    ) -> Range {
        todo!("GetAdjustedRange")
    }

    pub(crate) fn get_adjusted_start_position(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _leading_option: LeadingTriviaOption,
        _has_trailing_comment: bool,
    ) -> usize {
        todo!("getAdjustedStartPosition")
    }

    pub(crate) fn get_end_position_of_multiline_trailing_comment(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _trailing_opt: TrailingTriviaOption,
    ) -> usize {
        todo!("getEndPositionOfMultilineTrailingComment")
    }

    pub(crate) fn get_adjusted_end_position(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _trailing_option: TrailingTriviaOption,
    ) -> usize {
        todo!("getAdjustedEndPosition")
    }

    pub(crate) fn get_insertion_position_at_source_file_top(
        &self,
        _source_file: &SourceFile,
    ) -> usize {
        todo!("getInsertionPositionAtSourceFileTop")
    }
}

pub fn get_format_code_settings_for_writing(
    mut options: FormatCodeSettings,
    source_file: &SourceFile,
) -> FormatCodeSettings {
    let should_auto_detect_semicolon_preference = options.semicolons == SemicolonPreference::Ignore;
    let should_remove_semicolons = options.semicolons == SemicolonPreference::Remove
        || (should_auto_detect_semicolon_preference && !probably_uses_semicolons(source_file));
    if should_remove_semicolons {
        options.semicolons = SemicolonPreference::Remove;
    }
    options
}
