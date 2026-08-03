//! Change tracker implementation — text-change computation and range
//! adjustment.
//!
//! Ported from `internal/ls/change/trackerimpl.go`. All functions depend on the
//! printer (`EmitContext`, `NodeFactory`, `ChangeTrackerWriter`), the format
//! engine, the scanner, and `lsconv::Converters`, none of which are ported yet;
//! bodies are stubbed (`todo!()`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, SourceFile};
use crate::lsp::lsproto::lsp::{Range, TextEdit};

use crate::ls::lsutil::format_code_options::{FormatCodeSettings, SemicolonPreference};
use crate::ls::lsutil::utilities::probably_uses_semicolons;

use super::tracker::{LeadingTriviaOption, NodeOptions, Tracker, TrailingTriviaOption};

impl Tracker {
    /// Computes the final `TextEdit` map from the accumulated edits.
    ///
    /// Mirrors `Tracker.getTextChangesFromChanges` in Go. The edit list is
    /// private to [`Tracker`]; this stub returns an empty map until the
    /// formatting/printer pipeline is ported.
    pub(crate) fn get_text_changes_from_changes(
        &self,
    ) -> std::collections::HashMap<String, Vec<TextEdit>> {
        // TODO: sort edits by range, verify no overlap, computeNewText per edit.
        std::collections::HashMap::new()
    }

    /// Computes the replacement text for a single edit.
    ///
    /// Mirrors `Tracker.computeNewText` in Go.
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

    /// Formats and returns the text of a node to be inserted.
    ///
    /// Mirrors `Tracker.getFormattedTextOfNode` in Go.
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

    /// Prints a node to unformatted text.
    ///
    /// Mirrors `Tracker.getNonformattedText` in Go.
    pub(crate) fn get_nonformatted_text(
        &mut self,
        _node: &Arc<Node>,
        _source_file: &SourceFile,
    ) -> (String, Arc<Node>) {
        todo!("getNonformattedText")
    }

    /// Computes the adjusted range for a node, accounting for trivia.
    ///
    /// Mirrors `Tracker.GetAdjustedRange` in Go.
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

    /// Computes the adjusted start position for trivia handling.
    ///
    /// Mirrors `Tracker.getAdjustedStartPosition` in Go.
    pub(crate) fn get_adjusted_start_position(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _leading_option: LeadingTriviaOption,
        _has_trailing_comment: bool,
    ) -> usize {
        todo!("getAdjustedStartPosition")
    }

    /// Returns the end position of a multiline trailing comment, if any.
    ///
    /// Mirrors `Tracker.getEndPositionOfMultilineTrailingComment` in Go.
    pub(crate) fn get_end_position_of_multiline_trailing_comment(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _trailing_opt: TrailingTriviaOption,
    ) -> usize {
        todo!("getEndPositionOfMultilineTrailingComment")
    }

    /// Computes the adjusted end position for trivia handling.
    ///
    /// Mirrors `Tracker.getAdjustedEndPosition` in Go.
    pub(crate) fn get_adjusted_end_position(
        &self,
        _source_file: &SourceFile,
        _node: &Arc<Node>,
        _trailing_option: TrailingTriviaOption,
    ) -> usize {
        todo!("getAdjustedEndPosition")
    }

    /// Returns the position at which to insert nodes at the top of the file.
    ///
    /// Mirrors `Tracker.getInsertionPositionAtSourceFileTop` in Go.
    pub(crate) fn get_insertion_position_at_source_file_top(
        &self,
        _source_file: &SourceFile,
    ) -> usize {
        todo!("getInsertionPositionAtSourceFileTop")
    }
}

/// Returns format settings adjusted for writing, auto-detecting semicolons.
///
/// Mirrors `getFormatCodeSettingsForWriting` in Go.
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
