use crate::ast::node_flags::ModifierFlags;
use crate::ast::node_node::Node;
use std::sync::Arc;
use tsox_core::core::text::TextRange;

#[derive(Debug, Default, Clone)]
pub struct NodeList {
    pub loc: TextRange,
    pub nodes: Vec<Arc<Node>>,
}

impl NodeList {
    pub fn new(nodes: Vec<Arc<Node>>) -> Self { ::tsox_core::fntrace::enter("new"); 
        Self {
            loc: TextRange::undefined(),
            nodes,
        }
    }

    #[inline]
    pub fn pos(&self) -> usize { ::tsox_core::fntrace::enter("pos"); 
        self.loc.pos()
    }

    #[inline]
    pub fn end(&self) -> usize { ::tsox_core::fntrace::enter("end"); 
        self.loc.end()
    }

    pub fn has_trailing_comma(&self) -> bool { ::tsox_core::fntrace::enter("has_trailing_comma"); 
        if self.nodes.is_empty() {
            return false;
        }
        let last = self.nodes.last().unwrap();
        last.end() < self.end()
    }

    pub fn len(&self) -> usize { ::tsox_core::fntrace::enter("len"); 
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool { ::tsox_core::fntrace::enter("is_empty"); 
        self.nodes.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Arc<Node>> { ::tsox_core::fntrace::enter("iter"); 
        self.nodes.iter()
    }
}

#[derive(Debug, Default)]
pub struct ModifierList {
    pub list: NodeList,
    pub modifier_flags: ModifierFlags,
}

impl ModifierList {
    pub fn new(nodes: Vec<Arc<Node>>, flags: ModifierFlags) -> Self { ::tsox_core::fntrace::enter("new"); 
        Self {
            list: NodeList::new(nodes),
            modifier_flags: flags,
        }
    }

    pub fn flags(&self) -> ModifierFlags { ::tsox_core::fntrace::enter("flags"); 
        self.modifier_flags
    }
}

impl std::ops::Deref for ModifierList {
    type Target = NodeList;

    fn deref(&self) -> &Self::Target { ::tsox_core::fntrace::enter("deref"); 
        &self.list
    }
}
