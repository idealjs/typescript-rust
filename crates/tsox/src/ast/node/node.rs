use super::node_list::ModifierList;
use super::source_file::SourceFile;
use crate::ast::node_data_generated::NodeData;
use crate::ast::node_flags::{ModifierFlags, NodeFlags};
use crate::ast::syntax_kind_generated::SyntaxKind;
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
    use crate::ast::node_data_generated::*;
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
