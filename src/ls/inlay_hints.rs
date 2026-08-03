//! Inlay hints provider (1:1 port of Go's `internal/ls/inlay_hints.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::node::LineMap;
use crate::ast::node_data_generated::for_each_child;
use crate::ast::{Node, NodeData};
use crate::checker::Checker;
use crate::lsp::lsproto::lsp::{DocumentUri, Position, Range};

use super::language_service::LanguageService;
use super::types::{InlayHint, InlayHintLabel};

/// LSP inlay hint kind: Type annotation.
const INLAY_HINT_KIND_TYPE: i32 = 1;
/// LSP inlay hint kind: Parameter name.
const INLAY_HINT_KIND_PARAMETER: i32 = 2;

impl LanguageService {
    /// Provide inlay hints for a range in a document.
    ///
    /// Mirrors `ProvideInlayHint`:
    /// 1. Get program + source file.
    /// 2. Walk the AST within the requested range.
    /// 3. For variable declarations without an explicit type, add a type
    ///    annotation hint.
    /// 4. For call-expression arguments, add parameter-name hints (when the
    ///    resolved signature is available).
    /// 5. Return the collected `InlayHint` list.
    pub fn provide_inlay_hints(&self, document_uri: &DocumentUri, range: Range) -> Vec<InlayHint> {
        let (program, source_file) = self.get_program_and_file(document_uri);
        let line_map = &source_file.line_map;

        // Convert the requested LSP range to byte offsets.
        let range_start = lsp_position_to_offset(line_map, &range.start);
        let range_end = lsp_position_to_offset(line_map, &range.end);

        let mut checker = program.build_checker();

        let mut hints = Vec::new();
        collect_inlay_hints(
            &source_file.node,
            &mut checker,
            range_start,
            range_end,
            line_map,
            &mut hints,
        );
        hints
    }

    /// Legacy single-argument entry point (kept for backward compatibility).
    pub fn provide_inlay_hint(&self, document_uri: &DocumentUri) -> Vec<InlayHint> {
        // Default to the entire file.
        let (_, source_file) = self.get_program_and_file(document_uri);
        let end = source_file.text.len();
        self.provide_inlay_hints(
            document_uri,
            Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: u32::MAX,
                    character: end as u32,
                },
            },
        )
    }
}

/// Recursively walk the AST collecting inlay hints for nodes that fall within
/// the `[range_start, range_end]` range.
fn collect_inlay_hints(
    node: &Arc<Node>,
    checker: &mut Checker,
    range_start: usize,
    range_end: usize,
    line_map: &LineMap,
    hints: &mut Vec<InlayHint>,
) {
    // Skip subtrees entirely outside the requested range.
    if node.end() < range_start || node.pos() > range_end {
        return;
    }

    // Process this node based on its kind.
    match &node.data {
        // Variable declarations without an explicit type annotation:
        //   const x = 42;  →  hint ": number"
        NodeData::VariableDeclaration(data) => {
            if data.type_node.is_none() {
                if let Some(ref initializer) = data.initializer {
                    let ty = checker.get_type_of_node(initializer);
                    let type_str = checker.type_to_string(&ty);
                    // Skip trivial intrinsic types that match common patterns.
                    if !type_str.is_empty() && !is_omit_type_hint(&data.name.text(), &type_str) {
                        // Place the hint at the end of the variable name.
                        let hint_pos = offset_to_position(line_map, data.name.end());
                        hints.push(InlayHint {
                            position: hint_pos,
                            label: InlayHintLabel {
                                string: Some(format!(": {}", type_str)),
                            },
                            kind: Some(INLAY_HINT_KIND_TYPE),
                            text_edits: None,
                            padding_left: Some(false),
                            padding_right: Some(false),
                        });
                    }
                }
            }
        }

        // Call expressions: add parameter-name hints for arguments.
        //   foo(42);  →  hint "arg:" before 42
        NodeData::CallExpression(data) => {
            add_parameter_name_hints(node, data, checker, line_map, hints);
        }

        _ => {}
    }

    // Recurse into children.
    for_each_child(node, |child| {
        collect_inlay_hints(child, checker, range_start, range_end, line_map, hints);
        false
    });
}

/// Add parameter-name inlay hints for the arguments of a call expression.
fn add_parameter_name_hints(
    _call_node: &Arc<Node>,
    data: &crate::ast::node_data_generated::CallExpressionData,
    checker: &mut Checker,
    line_map: &LineMap,
    hints: &mut Vec<InlayHint>,
) {
    // Resolve the signature for this call expression.
    let (signature, _candidate_signatures) =
        checker.get_resolved_signature_for_signature_help(_call_node, data.arguments.len() as i32);

    let signature = match signature {
        Some(sig) => sig,
        None => return,
    };

    let parameters = &signature.parameters;
    for (i, arg) in data.arguments.nodes.iter().enumerate() {
        if i >= parameters.len() {
            break;
        }
        let param = &parameters[i];
        let param_name = &param.name;

        // Skip if the argument already matches the parameter name (e.g. a
        // property-access or identifier with the same text).
        if arg.text() == param_name.as_str() {
            continue;
        }

        let hint_pos = offset_to_position(line_map, arg.pos());
        hints.push(InlayHint {
            position: hint_pos,
            label: InlayHintLabel {
                string: Some(format!("{}:", param_name)),
            },
            kind: Some(INLAY_HINT_KIND_PARAMETER),
            text_edits: None,
            padding_left: Some(false),
            padding_right: Some(true),
        });
    }
}

/// Whether a type hint should be omitted for a given variable name / type pair.
///
/// For example, `const s = "hello"` with inferred type `string` is obvious
/// enough that we skip the hint — mirroring the `whenTypeMatchesName` option.
fn is_omit_type_hint(name: &str, type_str: &str) -> bool {
    let lower_name = name.to_ascii_lowercase();
    let lower_type = type_str.to_ascii_lowercase();
    matches!(
        (&lower_name[..], &lower_type[..]),
        ("s", "string")
            | ("str", "string")
            | ("n", "number")
            | ("num", "number")
            | ("b", "boolean")
            | ("bool", "boolean")
            | ("i", "number")
            | ("j", "number")
            | ("x", "number")
            | ("y", "number")
    )
}

/// State used during inlay hint collection.
pub struct InlayHintState<'a> {
    pub span: crate::core::text::TextRange,
    pub preferences: &'a crate::ls::lsutil::InlayHintsPreferences,
    pub quote_preference: crate::ls::lsutil::QuotePreference,
    pub result: Vec<InlayHint>,
}

/// Check if any inlay hint is enabled.
pub fn is_any_inlay_hint_enabled(prefs: &crate::ls::lsutil::InlayHintsPreferences) -> bool {
    prefs.include_inlay_variable_type_hints.is_true_or_unknown()
        || prefs.include_inlay_function_parameter_type_hints.is_true()
        || prefs
            .include_inlay_function_like_return_type_hints
            .is_true()
        || prefs
            .include_inlay_property_declaration_type_hints
            .is_true()
        || prefs.include_inlay_enum_member_value_hints.is_true()
        || !matches!(
            prefs.include_inlay_parameter_name_hints,
            crate::ls::lsutil::IncludeInlayParameterNameHints::None
        )
}

// ─── Helper functions ────────────────────────────────────────────────

/// Convert an LSP `Position` to a byte offset within a line map.
fn lsp_position_to_offset(line_map: &LineMap, position: &Position) -> usize {
    let line = position.line as usize;
    let character = position.character as usize;
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    line_start + character
}

/// Convert a byte offset to an LSP `Position`.
fn offset_to_position(line_map: &LineMap, offset: usize) -> Position {
    let line = line_of_offset(line_map, offset);
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    Position {
        line: line as u32,
        character: offset.saturating_sub(line_start) as u32,
    }
}

/// Binary search for the line number of a byte offset.
fn line_of_offset(line_map: &LineMap, offset: usize) -> usize {
    match line_map.line_starts.binary_search(&(offset as u32)) {
        Ok(idx) => idx,
        Err(idx) => idx.saturating_sub(1),
    }
}
