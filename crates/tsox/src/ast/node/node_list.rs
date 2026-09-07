use super::node::Node;
use crate::ast::node_flags::ModifierFlags;
use crate::core::text::TextRange;
use std::sync::Arc;

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
