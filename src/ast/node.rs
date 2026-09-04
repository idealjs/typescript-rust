use super::SyntaxKind;
use super::node_data_generated::NodeData;
use super::node_flags::{ModifierFlags, NodeFlags};
use crate::core::text::TextRange;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_NODE_ID: AtomicU64 = AtomicU64::new(1);

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

    pub fn with_loc_flags(
        kind: SyntaxKind,
        data: NodeData,
        loc: TextRange,
        flags: NodeFlags,
    ) -> Self {
        Self {
            kind,
            flags,
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

    pub fn id(&self) -> u64 {
        let mut id = self.id.load(Ordering::Relaxed);
        if id == 0 {
            id = NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed);
            self.id.store(id, Ordering::Relaxed);
        }
        id
    }

    pub fn text(&self) -> &str {
        crate::ast::node_data_generated::node_text(self)
    }

    pub fn expression(&self) -> Option<&Arc<Node>> {
        crate::ast::node_data_generated::node_expression(self)
    }

    pub fn type_node(&self) -> Option<&Arc<Node>> {
        crate::ast::node_data_generated::node_type(self)
    }

    pub fn name(&self) -> Option<&Arc<Node>> {
        crate::ast::node_data_generated::node_name(self)
    }

    pub fn modifiers(&self) -> Option<&Arc<ModifierList>> {
        crate::ast::node::node_modifiers(self)
    }

    pub fn modifier_nodes(&self) -> &[Arc<Node>] {
        match self.modifiers() {
            Some(ml) => &ml.list.nodes,
            None => &[],
        }
    }

    pub fn syntactic_modifier_flags(&self) -> ModifierFlags {
        match self.modifiers() {
            Some(ml) => ml.modifier_flags,
            None => ModifierFlags::empty(),
        }
    }

    pub fn has_syntactic_modifier(&self, flags: ModifierFlags) -> bool {
        self.syntactic_modifier_flags().intersects(flags)
    }

    pub fn jsdoc(&self, file: &SourceFile) -> Vec<Arc<Node>> {
        if !self.flags.contains(NodeFlags::HasJSDoc) {
            return Vec::new();
        }
        if file.has_lazy_jsdoc() {
            file.resolve_jsdoc(self)
        } else {
            file.eager_jsdoc(self)
        }
    }
}

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

#[derive(Debug)]
pub struct SourceFile {
    pub node: Arc<Node>,
    pub file_name: String,
    pub text: String,
    pub line_map: LineMap,
    pub language_variant: LanguageVariant,
    pub script_kind: ScriptKind,

    pub comment_directives: Vec<crate::scanner::CommentDirective>,

    pub(crate) jsdoc_cache: std::sync::RwLock<std::collections::HashMap<u64, Vec<Arc<Node>>>>,

    pub(crate) has_lazy_jsdoc: bool,

    pub is_declaration_file: bool,

    pub imports: Vec<Arc<Node>>,

    pub module_augmentations: Vec<Arc<Node>>,

    pub ambient_module_names: Vec<String>,

    pub parse_error_spans: Vec<crate::core::text::TextRange>,

    pub external_module_indicator: Option<Arc<Node>>,

    pub common_js_module_indicator: Option<Arc<Node>>,

    pub uses_uri_style_node_core_modules: crate::core::tristate::Tristate,

    pub has_parse_diagnostics: bool,
}

impl SourceFile {

    pub fn id(&self) -> u64 {
        self.node.id()
    }

    pub fn set_jsdoc_cache(&self, cache: std::collections::HashMap<u64, Vec<Arc<Node>>>) {
        *self.jsdoc_cache.write().unwrap() = cache;
    }

    pub fn set_has_lazy_jsdoc(&mut self, lazy: bool) {
        self.has_lazy_jsdoc = lazy;
    }

    pub fn has_lazy_jsdoc(&self) -> bool {
        self.has_lazy_jsdoc
    }

    pub fn resolve_jsdoc(&self, node: &Node) -> Vec<Arc<Node>> {
        let node_id = node.id();

        {
            let cache = self.jsdoc_cache.read().unwrap();
            if let Some(jsdocs) = cache.get(&node_id) {
                return jsdocs.clone();
            }
        }

        let mut cache = self.jsdoc_cache.write().unwrap();
        if let Some(jsdocs) = cache.get(&node_id) {
            return jsdocs.clone();
        }
        let jsdocs = crate::parser::parse_jsdoc_for_node(self, node);
        cache.insert(node_id, jsdocs.clone());
        jsdocs
    }

    pub fn eager_jsdoc(&self, node: &Node) -> Vec<Arc<Node>> {
        let cache = self.jsdoc_cache.read().unwrap();
        cache.get(&node.id()).cloned().unwrap_or_default()
    }
}

#[derive(Debug, Default)]
pub struct LineMap {
    pub line_starts: Vec<u32>,
}

fn is_line_break(ch: char) -> bool {
    matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

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

                let s = &text[pos..];
                match s.chars().next() {
                    Some(ch) => {
                        pos += ch.len_utf8();
                        if is_line_break(ch) {
                            line_starts.push(pos as u32);
                        }
                    }
                    None => break,
                }
            }
        }

        Self { line_starts }
    }

    pub fn line_at(&self, offset: usize) -> usize {
        match self.line_starts.binary_search(&(offset as u32)) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        }
    }

    pub fn line_start(&self, offset: usize) -> usize {
        let line = self.line_at(offset);
        self.line_starts[line] as usize
    }

    pub fn utf16_column_at(&self, text: &str, offset: usize) -> usize {
        let line_start = self.line_start(offset);
        utf16_len(&text[line_start..offset])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LanguageVariant {
    #[default]
    Standard,
    Jsx,
}

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

        let lm = LineMap::from_text("abc\r\ndef\r\nghi");
        assert_eq!(lm.line_starts, vec![0, 5, 10]);
        assert_eq!(lm.line_at(0), 0);
        assert_eq!(lm.line_at(3), 0);
        assert_eq!(lm.line_at(5), 1);
        assert_eq!(lm.line_at(10), 2);
    }

    #[test]
    fn line_map_cr_only() {

        let lm = LineMap::from_text("abc\rdef");
        assert_eq!(lm.line_starts, vec![0, 4]);
        assert_eq!(lm.line_at(0), 0);
        assert_eq!(lm.line_at(4), 1);
    }

    #[test]
    fn line_map_unicode_line_separators() {

        let lm = LineMap::from_text("ab\u{2028}cd\u{2029}ef");
        assert_eq!(lm.line_starts.len(), 3);
        assert_eq!(lm.line_at(0), 0);

        assert_eq!(lm.line_at(5), 1);

        assert_eq!(lm.line_at(10), 2);
    }

    #[test]
    fn line_map_utf16_column_ascii() {
        let text = "abc\ndef";
        let lm = LineMap::from_text(text);

        assert_eq!(lm.utf16_column_at(text, 5), 1);

        assert_eq!(lm.utf16_column_at(text, 6), 2);
    }

    #[test]
    fn line_map_utf16_column_non_ascii() {

        let text = "café\ndef";
        let lm = LineMap::from_text(text);

        assert_eq!(lm.utf16_column_at(text, 3), 3);

        assert_eq!(lm.utf16_column_at(text, 5), 4);
    }

    #[test]
    fn line_map_utf16_column_emoji() {

        let text = "x🦀y";
        let lm = LineMap::from_text(text);

        assert_eq!(lm.utf16_column_at(text, 5), 3);
    }

    #[test]
    fn utf16_len_basic() {
        assert_eq!(utf16_len("abc"), 3);
        assert_eq!(utf16_len("café"), 4);
        assert_eq!(utf16_len("🦀"), 2);
        assert_eq!(utf16_len("x🦀y"), 4);
    }
}
