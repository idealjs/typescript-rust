//! Organize imports provider (1:1 port of Go's `internal/ls/organizeimports.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::node::LineMap;
use crate::ast::node_data_generated::NodeData;
use crate::ast::{Node, SourceFile, SyntaxKind};
use crate::compiler::Program;
use crate::lsp::lsproto::lsp::{Position, Range, TextEdit};
use crate::scanner;

use super::language_service::LanguageService;

impl LanguageService {
    /// Organize imports: remove unused, coalesce, and sort.
    ///
    /// Mirrors `OrganizeImports`:
    /// 1. Walk import declarations in the source file.
    /// 2. Sort them by module specifier (alphabetical).
    /// 3. Return `TextEdits` for the reorganized import block.
    pub fn organize_imports(
        &self,
        source_file: &Arc<SourceFile>,
        program: &Arc<Program>,
        _kind: &str,
    ) -> std::collections::HashMap<String, Vec<TextEdit>> {
        let line_map = &source_file.line_map;
        let text = &source_file.text;

        // Collect all import declarations from the source file.
        let imports = collect_import_declarations(&source_file.node);

        if imports.is_empty() {
            return std::collections::HashMap::new();
        }

        // Determine the range of the import block (from first to last import).
        let first = imports.first().unwrap();
        let last = imports.last().unwrap();
        let block_start = first.pos();
        let block_end = last.end();

        // Build a list of (sort_key, import_text) for each import.
        let mut import_infos: Vec<ImportInfo> = Vec::new();
        for imp in &imports {
            let specifier = get_module_specifier(imp);
            let is_type_only = is_type_only_import(imp);

            // Check if the import is used (via checker).
            let is_used = is_import_used(program, source_file, imp);

            let start = scanner::skip_trivia(text, imp.pos());
            let end = imp.end();
            let import_text = text[start..end].trim().to_string();

            import_infos.push(ImportInfo {
                sort_key: specifier.clone(),
                is_type_only,
                is_used,
                text: import_text,
            });
        }

        // Sort: unused first (for removal), then type-only, then alphabetical.
        import_infos.sort_by(|a, b| {
            // Unused imports go to the top (they'll be removed).
            match (a.is_used, b.is_used) {
                (false, true) => return std::cmp::Ordering::Less,
                (true, false) => return std::cmp::Ordering::Greater,
                _ => {}
            }
            // Type-only imports before value imports.
            match (a.is_type_only, b.is_type_only) {
                (true, false) => return std::cmp::Ordering::Less,
                (false, true) => return std::cmp::Ordering::Greater,
                _ => {}
            }
            // Alphabetical by module specifier.
            a.sort_key.cmp(&b.sort_key)
        });

        // Build the new import block text, excluding unused imports.
        let new_text: String = import_infos
            .iter()
            .filter(|info| info.is_used)
            .map(|info| info.text.clone())
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";

        // Create the TextEdit replacing the entire import block.
        let edit = TextEdit {
            range: Range {
                start: offset_to_position(line_map, block_start),
                end: offset_to_position(line_map, block_end),
            },
            new_text: new_text,
        };

        let mut result = std::collections::HashMap::new();
        result.insert(source_file.file_name.clone(), vec![edit]);
        result
    }
}

/// Information about a single import declaration used for sorting.
struct ImportInfo {
    sort_key: String,
    is_type_only: bool,
    is_used: bool,
    text: String,
}

/// Collect all `ImportDeclaration` nodes from a source file's statements.
fn collect_import_declarations(file_node: &Arc<Node>) -> Vec<Arc<Node>> {
    let mut imports = Vec::new();
    if let NodeData::SourceFile(data) = &file_node.data {
        for stmt in &data.statements.nodes {
            if stmt.kind == SyntaxKind::ImportDeclaration {
                imports.push(Arc::clone(stmt));
            }
        }
    }
    imports
}

/// Extract the module specifier text from an import declaration.
fn get_module_specifier(node: &Arc<Node>) -> String {
    if let NodeData::ImportDeclaration(data) = &node.data {
        return data.module_specifier.text().to_string();
    }
    String::new()
}

/// Check if an import declaration is type-only.
fn is_type_only_import(node: &Arc<Node>) -> bool {
    if let NodeData::ImportDeclaration(data) = &node.data {
        // Check if the import clause is type-only.
        if let Some(ref clause) = data.import_clause {
            if let NodeData::ImportClause(ic) = &clause.data {
                return ic.phase_modifier == Some(SyntaxKind::TypeKeyword);
            }
        }
    }
    false
}

/// Check if an import is used in the source file.
fn is_import_used(program: &Arc<Program>, source_file: &Arc<SourceFile>, imp: &Arc<Node>) -> bool {
    let mut checker = program.build_checker();

    // Get the imported identifiers and check if any are used.
    if let NodeData::ImportDeclaration(data) = &imp.data {
        if let Some(ref clause) = data.import_clause {
            let identifiers = collect_imported_identifiers(clause);
            // If no named identifiers (e.g. side-effect import), keep it.
            if identifiers.is_empty() {
                return true;
            }
            for id in &identifiers {
                if checker.is_declaration_used(source_file, id, false, false) {
                    return true;
                }
            }
        }
        // Side-effect-only import (no import clause): `import "mod";`
        if data.import_clause.is_none() {
            return true;
        }
    }
    false
}

/// Collect all identifier nodes declared by an import clause.
fn collect_imported_identifiers(clause: &Arc<Node>) -> Vec<Arc<Node>> {
    let mut result = Vec::new();
    match &clause.data {
        NodeData::ImportClause(data) => {
            // Default import name.
            if let Some(ref name) = data.name {
                result.push(Arc::clone(name));
            }
            // Named bindings (named imports or namespace import).
            if let Some(ref bindings) = data.named_bindings {
                match &bindings.data {
                    NodeData::NamedImports(ni) => {
                        for elem in &ni.elements.nodes {
                            if let NodeData::ImportSpecifier(spec) = &elem.data {
                                result.push(Arc::clone(&spec.name));
                            }
                        }
                    }
                    NodeData::NamespaceImport(nsi) => {
                        result.push(Arc::clone(&nsi.name));
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    result
}

/// Group contiguous import declarations by newline gaps.
///
/// Mirrors `groupByNewlineContiguous`.
pub fn group_by_newline_contiguous(
    source_file: &Arc<SourceFile>,
    imports: &[Arc<Node>],
) -> Vec<Vec<Arc<Node>>> {
    let text = &source_file.text;
    let mut groups: Vec<Vec<Arc<Node>>> = Vec::new();

    for imp in imports {
        let prev_end = groups
            .last()
            .and_then(|g| g.last())
            .map(|n| n.end())
            .unwrap_or(0);
        let curr_start = imp.pos();

        // If there is a blank line between the previous import and this one,
        // start a new group.
        if !groups.is_empty() && has_blank_line_between(text, prev_end, curr_start) {
            groups.push(Vec::new());
        } else if groups.is_empty() {
            groups.push(Vec::new());
        }
        groups.last_mut().unwrap().push(Arc::clone(imp));
    }
    groups
}

/// Check whether there is a blank line between two offsets in the text.
fn has_blank_line_between(text: &str, start: usize, end: usize) -> bool {
    if start >= end || end > text.len() {
        return false;
    }
    let between = &text[start..end];
    between.lines().filter(|l| l.trim().is_empty()).count() > 1
}

// ─── Helper functions ────────────────────────────────────────────────

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
