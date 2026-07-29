//! Core AST node types.
//!
//! Ported from `internal/ast/ast.go`. The Go implementation uses a
//! `nodeData` interface with hundreds of implementations and type
//! switches. In Rust we use an enum (`NodeData`) with one variant per
//! node kind, and pattern matching for accessors.
//!
//! The `NodeData` enum and per-node data structs are generated from
//! `_scripts/ast.json` by `_scripts/generate-rust-ast.ts`. See
//! `node_data_generated.rs`.

use super::SyntaxKind;
use super::node_data_generated::NodeData;
use super::node_flags::{ModifierFlags, NodeFlags};
use crate::core::text::TextRange;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// A unique ID assigned to each node for debugging and caching.
static NEXT_NODE_ID: AtomicU64 = AtomicU64::new(1);

/// The core AST node.
///
/// Mirrors `ast.Node` in Go. A node carries its kind, flags, source
/// location, parent pointer, and kind-specific data.
#[derive(Debug)]
pub struct Node {
    pub kind: SyntaxKind,
    pub flags: NodeFlags,
    pub loc: TextRange,
    id: AtomicU64,
    pub parent: Option<Arc<Node>>,
    pub data: NodeData,
}

impl Node {
    pub fn new(kind: SyntaxKind, data: NodeData) -> Self {
        Self {
            kind,
            flags: NodeFlags::empty(),
            loc: TextRange::undefined(),
            id: AtomicU64::new(0),
            parent: None,
            data,
        }
    }

    /// Create a node with a specific kind, data, and source location.
    pub fn with_loc(kind: SyntaxKind, data: NodeData, loc: TextRange) -> Self {
        Self {
            kind,
            flags: NodeFlags::empty(),
            loc,
            id: AtomicU64::new(0),
            parent: None,
            data,
        }
    }

    #[inline]
    pub fn pos(&self) -> usize {
        self.loc.pos()
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.loc.end()
    }

    /// A unique numeric ID for this node (lazily assigned).
    pub fn id(&self) -> u64 {
        let mut id = self.id.load(Ordering::Relaxed);
        if id == 0 {
            id = NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed);
            self.id.store(id, Ordering::Relaxed);
        }
        id
    }

    /// The text content of literal/identifier nodes.
    ///
    /// Mirrors `Node.Text()` in Go.
    pub fn text(&self) -> &str {
        crate::ast::node_data_generated::node_text(self)
    }

    /// The primary expression child of this node, if any.
    ///
    /// Mirrors `Node.Expression()` in Go.
    pub fn expression(&self) -> Option<&Arc<Node>> {
        crate::ast::node_data_generated::node_expression(self)
    }

    /// The type child of a type node, if any.
    ///
    /// Mirrors `Node.Type()` in Go.
    pub fn type_node(&self) -> Option<&Arc<Node>> {
        crate::ast::node_data_generated::node_type(self)
    }

    /// The name of a declaration, if any.
    ///
    /// Mirrors `Node.Name()` in Go.
    pub fn name(&self) -> Option<&Arc<Node>> {
        crate::ast::node_data_generated::node_name(self)
    }

    /// The modifier list of a declaration, if any.
    ///
    /// Mirrors `Node.Modifiers()` in Go.
    pub fn modifiers(&self) -> Option<&Arc<ModifierList>> {
        crate::ast::node::node_modifiers(self)
    }

    /// The modifier nodes (as a slice of `Arc<Node>`), if any.
    ///
    /// Mirrors `Node.ModifierNodes()` in Go.
    pub fn modifier_nodes(&self) -> &[Arc<Node>] {
        match self.modifiers() {
            Some(ml) => &ml.list.nodes,
            None => &[],
        }
    }

    /// Combined modifier flags (syntactic only; does not include JSDoc
    /// modifiers). Mirrors `ast.GetSyntacticModifierFlags` in Go.
    pub fn syntactic_modifier_flags(&self) -> ModifierFlags {
        match self.modifiers() {
            Some(ml) => ml.modifier_flags,
            None => ModifierFlags::empty(),
        }
    }

    /// Whether the node has the given syntactic modifier flag set.
    pub fn has_syntactic_modifier(&self, flags: ModifierFlags) -> bool {
        self.syntactic_modifier_flags().intersects(flags)
    }
}

/// Get the modifier list of a node, if any.
///
/// Mirrors `Node.Modifiers()` in Go. This is a hand-written counterpart to
/// the generated accessors in `node_data_generated.rs`, since the generator
/// does not yet emit a modifiers accessor.
pub fn node_modifiers(node: &Node) -> Option<&Arc<ModifierList>> {
    use super::node_data_generated::*;
    match &node.data {
        NodeData::VariableStatement(d) => d.modifiers.as_ref(),
        NodeData::ParameterDeclaration(d) => d.modifiers.as_ref(),
        NodeData::MissingDeclaration(d) => d.modifiers.as_ref(),
        NodeData::FunctionDeclaration(d) => d.modifiers.as_ref(),
        NodeData::ClassDeclaration(d) => d.modifiers.as_ref(),
        NodeData::ClassExpression(d) => d.modifiers.as_ref(),
        NodeData::InterfaceDeclaration(d) => d.modifiers.as_ref(),
        NodeData::TypeAliasDeclaration(d) => d.modifiers.as_ref(),
        NodeData::EnumDeclaration(d) => d.modifiers.as_ref(),
        NodeData::ImportDeclaration(d) => d.modifiers.as_ref(),
        NodeData::ExportAssignment(d) => d.modifiers.as_ref(),
        NodeData::NamespaceExportDeclaration(d) => d.modifiers.as_ref(),
        NodeData::ConstructorDeclaration(d) => d.modifiers.as_ref(),
        NodeData::GetAccessorDeclaration(d) => d.modifiers.as_ref(),
        NodeData::SetAccessorDeclaration(d) => d.modifiers.as_ref(),
        NodeData::IndexSignatureDeclaration(d) => d.modifiers.as_ref(),
        NodeData::MethodSignatureDeclaration(d) => d.modifiers.as_ref(),
        NodeData::MethodDeclaration(d) => d.modifiers.as_ref(),
        NodeData::PropertySignatureDeclaration(d) => d.modifiers.as_ref(),
        NodeData::PropertyDeclaration(d) => d.modifiers.as_ref(),
        NodeData::ClassStaticBlockDeclaration(d) => d.modifiers.as_ref(),
        NodeData::BinaryExpression(d) => d.modifiers.as_ref(),
        NodeData::ArrowFunction(d) => d.modifiers.as_ref(),
        NodeData::FunctionExpression(d) => d.modifiers.as_ref(),
        NodeData::PropertyAssignment(d) => d.modifiers.as_ref(),
        NodeData::ShorthandPropertyAssignment(d) => d.modifiers.as_ref(),
        NodeData::ConstructorTypeNode(d) => d.modifiers.as_ref(),
        NodeData::ModuleDeclaration(d) => d.modifiers.as_ref(),
        NodeData::ImportEqualsDeclaration(d) => d.modifiers.as_ref(),
        NodeData::ExportDeclaration(d) => d.modifiers.as_ref(),
        NodeData::TypeParameterDeclaration(d) => d.modifiers.as_ref(),
        _ => None,
    }
}

/// A list of nodes, preserving source location.
///
/// Mirrors `ast.NodeList` in Go.
#[derive(Debug, Default)]
pub struct NodeList {
    pub loc: TextRange,
    pub nodes: Vec<Arc<Node>>,
}

impl NodeList {
    pub fn new(nodes: Vec<Arc<Node>>) -> Self {
        Self {
            loc: TextRange::undefined(),
            nodes,
        }
    }

    #[inline]
    pub fn pos(&self) -> usize {
        self.loc.pos()
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.loc.end()
    }

    pub fn has_trailing_comma(&self) -> bool {
        if self.nodes.is_empty() {
            return false;
        }
        let last = self.nodes.last().unwrap();
        last.end() < self.end()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Arc<Node>> {
        self.nodes.iter()
    }
}

/// A list of modifier tokens, with cached flags.
///
/// Mirrors `ast.ModifierList` in Go.
#[derive(Debug, Default)]
pub struct ModifierList {
    pub list: NodeList,
    pub modifier_flags: ModifierFlags,
}

impl ModifierList {
    pub fn new(nodes: Vec<Arc<Node>>, flags: ModifierFlags) -> Self {
        Self {
            list: NodeList::new(nodes),
            modifier_flags: flags,
        }
    }

    pub fn flags(&self) -> ModifierFlags {
        self.modifier_flags
    }
}

impl std::ops::Deref for ModifierList {
    type Target = NodeList;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Node data variants — now generated by _scripts/generate-rust-ast.ts
// See node_data_generated.rs for the NodeData enum and all *Data structs.
// ────────────────────────────────────────────────────────────────────────────

/// A source file (the root of an AST).
///
/// Mirrors `ast.SourceFile` in Go.
#[derive(Debug)]
pub struct SourceFile {
    pub node: Arc<Node>,
    pub file_name: String,
    pub text: String,
    pub line_map: LineMap,
    pub language_variant: LanguageVariant,
    pub script_kind: ScriptKind,
    /// `@ts-expect-error` / `@ts-ignore` directives collected by the scanner.
    pub comment_directives: Vec<crate::scanner::CommentDirective>,
}

impl SourceFile {
    /// A unique numeric ID for this source file (delegates to its node).
    pub fn id(&self) -> u64 {
        self.node.id()
    }
}

/// A mapping from line number to byte offset in the source text.
///
/// Mirrors Go's `ComputeECMALineStarts`, handling ECMAScript line
/// terminators: LF (`\n`), CR (`\r`), CRLF (`\r\n`), LS (`\u2028`),
/// and PS (`\u2029`).
#[derive(Debug, Default)]
pub struct LineMap {
    pub line_starts: Vec<u32>,
}

/// Check if a character is an ECMAScript line terminator.
/// ES5 §7.3: LF, CR, LS (\u2028), PS (\u2029).
fn is_line_break(ch: char) -> bool {
    matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

/// Count the number of UTF-16 code units needed to represent a string.
/// Mirrors Go's `core.UTF16Len`. BMP characters → 1 unit, astral → 2 units.
pub fn utf16_len(s: &str) -> usize {
    let mut n = 0usize;
    for c in s.chars() {
        n += c.len_utf16();
    }
    n
}

impl LineMap {
    pub fn from_text(text: &str) -> Self {
        let mut line_starts = Vec::with_capacity(text.matches('\n').count() + 1);
        line_starts.push(0u32);

        let bytes = text.as_bytes();
        let text_len = bytes.len();
        let mut pos = 0usize;

        while pos < text_len {
            let b = bytes[pos];
            if b < 0x80 {
                // ASCII fast path
                pos += 1;
                if b == b'\r' {
                    if pos < text_len && bytes[pos] == b'\n' {
                        pos += 1;
                    }
                    line_starts.push(pos as u32);
                } else if b == b'\n' {
                    line_starts.push(pos as u32);
                }
            } else {
                // Non-ASCII: decode UTF-8 rune
                let s = &text[pos..];
                match s.chars().next() {
                    Some(ch) => {
                        pos += ch.len_utf8();
                        if is_line_break(ch) {
                            line_starts.push(pos as u32);
                        }
                    }
                    None => break, // shouldn't happen
                }
            }
        }

        Self { line_starts }
    }

    /// Get the line number (0-based) for a byte offset.
    pub fn line_at(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&(offset as u32)) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        }
    }

    /// Get the byte offset of the start of the line containing `offset`.
    pub fn line_start(&self, offset: usize) -> usize {
        let line = self.line_at(offset);
        self.line_starts[line] as usize
    }

    /// Get the UTF-16 column number (0-based) for a byte offset.
    /// Mirrors Go's `GetECMALineAndUTF16CharacterOfPosition` character computation.
    pub fn utf16_column_at(&self, text: &str, offset: usize) -> usize {
        let line_start = self.line_start(offset);
        utf16_len(&text[line_start..offset])
    }
}

/// Language variant (Standard or JSX).
///
/// Mirrors `core.LanguageVariant` in Go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LanguageVariant {
    #[default]
    Standard,
    Jsx,
}

/// Script kind (TS, JS, JSON, etc.).
///
/// Mirrors `core.ScriptKind` in Go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScriptKind {
    #[default]
    Unknown,
    Js,
    Jsx,
    Ts,
    Tsx,
    Json,
    External,
    Deferred,
}

#[cfg(test)]
mod tests {
    use super::super::node_data_generated::IdentifierData;
    use super::*;

    #[test]
    fn identifier_node() {
        let node = Node::new(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData {
                text: "foo".to_string(),
            }),
        );
        assert_eq!(node.kind, SyntaxKind::Identifier);
        assert_eq!(node.text(), "foo");
    }

    #[test]
    fn node_ids_are_unique() {
        let n1 = Node::new(SyntaxKind::Unknown, NodeData::Token);
        let n2 = Node::new(SyntaxKind::Unknown, NodeData::Token);
        assert_ne!(n1.id(), n2.id());
    }

    #[test]
    fn line_map_basic() {
        let lm = LineMap::from_text("abc\ndef\nghi");
        assert_eq!(lm.line_starts, vec![0, 4, 8]);
        assert_eq!(lm.line_at(0), 0);
        assert_eq!(lm.line_at(3), 0);
        assert_eq!(lm.line_at(4), 1);
        assert_eq!(lm.line_at(7), 1);
        assert_eq!(lm.line_at(8), 2);
    }

    #[test]
    fn line_map_crlf() {
        // \r\n should produce one line start, not two.
        let lm = LineMap::from_text("abc\r\ndef\r\nghi");
        assert_eq!(lm.line_starts, vec![0, 5, 10]);
        assert_eq!(lm.line_at(0), 0);
        assert_eq!(lm.line_at(3), 0);
        assert_eq!(lm.line_at(5), 1);
        assert_eq!(lm.line_at(10), 2);
    }

    #[test]
    fn line_map_cr_only() {
        // Bare \r is also a line terminator.
        let lm = LineMap::from_text("abc\rdef");
        assert_eq!(lm.line_starts, vec![0, 4]);
        assert_eq!(lm.line_at(0), 0);
        assert_eq!(lm.line_at(4), 1);
    }

    #[test]
    fn line_map_unicode_line_separators() {
        // \u2028 (LS) and \u2029 (PS) are ECMAScript line terminators.
        let lm = LineMap::from_text("ab\u{2028}cd\u{2029}ef");
        assert_eq!(lm.line_starts.len(), 3);
        assert_eq!(lm.line_at(0), 0);
        // \u2028 is 3 bytes in UTF-8, so "ab\u{2028}" is 5 bytes
        assert_eq!(lm.line_at(5), 1);
        // \u2029 is also 3 bytes, so "ab\u{2028}cd\u{2029}" is 10 bytes
        assert_eq!(lm.line_at(10), 2);
    }

    #[test]
    fn line_map_utf16_column_ascii() {
        let text = "abc\ndef";
        let lm = LineMap::from_text(text);
        // On line 1, position 5 ('e') should be column 1
        assert_eq!(lm.utf16_column_at(text, 5), 1);
        // On line 1, position 6 ('f') should be column 2
        assert_eq!(lm.utf16_column_at(text, 6), 2);
    }

    #[test]
    fn line_map_utf16_column_non_ascii() {
        // 'é' is 2 bytes in UTF-8 but 1 UTF-16 code unit (BMP).
        let text = "café\ndef";
        let lm = LineMap::from_text(text);
        // c=byte0, a=byte1, f=byte2, é=bytes3-4, \n=byte5
        // 'é' at byte offset 3 → UTF-16 column 3 (c=0, a=1, f=2, é=3)
        assert_eq!(lm.utf16_column_at(text, 3), 3);
        // Position after 'é' at byte offset 5 → UTF-16 column 4
        assert_eq!(lm.utf16_column_at(text, 5), 4);
    }

    #[test]
    fn line_map_utf16_column_emoji() {
        // '🦀' (crab emoji) is 4 bytes in UTF-8 but 2 UTF-16 code units (surrogate pair).
        let text = "x🦀y";
        let lm = LineMap::from_text(text);
        // 'y' is at byte offset 5, UTF-16 column 3 (x=0, 🦀=1-2, y=3)
        assert_eq!(lm.utf16_column_at(text, 5), 3);
    }

    #[test]
    fn utf16_len_basic() {
        assert_eq!(utf16_len("abc"), 3);
        assert_eq!(utf16_len("café"), 4); // é is BMP
        assert_eq!(utf16_len("🦀"), 2); // emoji is astral → 2 UTF-16 units
        assert_eq!(utf16_len("x🦀y"), 4); // 1 + 2 + 1
    }
}
