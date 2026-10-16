#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::mig::m3c::NodeVisitor;
use tsox_frontend::scanner::skip_trivia;

use tsox_core::stringutil::is_white_space_like;
use crate::mig::m4s::{
    get_default_indent_size, new_text_writer, EmitTextWriter, TextWriter,
};
use crate::printer::NodeFactory;

pub struct PrintHandlers<'a> {
    pub on_before_emit_node: Option<Box<dyn FnMut(Option<&Arc<Node>>) + 'a>>,
    pub on_after_emit_node: Option<Box<dyn FnMut(Option<&Arc<Node>>) + 'a>>,
    pub on_before_emit_node_list: Option<Box<dyn FnMut(Option<&NodeList>) + 'a>>,
    pub on_after_emit_node_list: Option<Box<dyn FnMut(Option<&NodeList>) + 'a>>,
    pub on_before_emit_token: Option<Box<dyn FnMut(Option<&Arc<Node>>) + 'a>>,
    pub on_after_emit_token: Option<Box<dyn FnMut(Option<&Arc<Node>>) + 'a>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TriviaPositionKey {
    Node(*const Node),
    NodeList(*const NodeList),
}

impl TriviaPositionKey {
    fn pos(&self) -> usize { ::tsox_core::fntrace::enter("pos"); 
        match self {
            TriviaPositionKey::Node(n) => unsafe { (**n).pos() },
            TriviaPositionKey::NodeList(l) => unsafe { (**l).pos() },
        }
    }

    fn end(&self) -> usize { ::tsox_core::fntrace::enter("end"); 
        match self {
            TriviaPositionKey::Node(n) => unsafe { (**n).end() },
            TriviaPositionKey::NodeList(l) => unsafe { (**l).end() },
        }
    }
}

pub struct ChangeTrackerWriter {
    pub writer: TextWriter,
    pub last_non_trivia_position: usize,
    pub pos: HashMap<TriviaPositionKey, usize>,
    pub end: HashMap<TriviaPositionKey, usize>,
}

pub fn new_change_tracker_writer(newline: &str, indent_size: i32) -> ChangeTrackerWriter { ::tsox_core::fntrace::enter("new_change_tracker_writer"); 
    let indent_size = if indent_size < 0 {
        get_default_indent_size()
    } else {
        indent_size as usize
    };
    let mut ctw = ChangeTrackerWriter {
        writer: new_text_writer(newline, indent_size),
        last_non_trivia_position: 0,
        pos: HashMap::new(),
        end: HashMap::new(),
    };
    ctw.writer.clear();
    ctw
}

impl ChangeTrackerWriter {
    pub fn get_print_handlers(&mut self) -> PrintHandlers<'_> { ::tsox_core::fntrace::enter("get_print_handlers"); 
        let ct: *mut ChangeTrackerWriter = self;
        PrintHandlers {
            on_before_emit_node: Some(Box::new(move |node_opt: Option<&Arc<Node>>| {
                if let Some(node) = node_opt {
                    unsafe { (*ct).set_pos_node(node) };
                }
            })),
            on_after_emit_node: Some(Box::new(move |node_opt: Option<&Arc<Node>>| {
                if let Some(node) = node_opt {
                    unsafe { (*ct).set_end_node(node) };
                }
            })),
            on_before_emit_node_list: Some(Box::new(move |nodes_opt: Option<&NodeList>| {
                if let Some(nodes) = nodes_opt {
                    unsafe { (*ct).set_pos_node_list(nodes) };
                }
            })),
            on_after_emit_node_list: Some(Box::new(move |nodes_opt: Option<&NodeList>| {
                if let Some(nodes) = nodes_opt {
                    unsafe { (*ct).set_end_node_list(nodes) };
                }
            })),
            on_before_emit_token: Some(Box::new(move |node_opt: Option<&Arc<Node>>| {
                if let Some(node) = node_opt {
                    unsafe { (*ct).set_pos_node(node) };
                }
            })),
            on_after_emit_token: Some(Box::new(move |node_opt: Option<&Arc<Node>>| {
                if let Some(node) = node_opt {
                    unsafe { (*ct).set_end_node(node) };
                }
            })),
        }
    }

    fn set_pos(&mut self, node: TriviaPositionKey) { ::tsox_core::fntrace::enter("set_pos"); 
        self.pos.insert(node, self.last_non_trivia_position);
    }

    fn set_end(&mut self, node: TriviaPositionKey) { ::tsox_core::fntrace::enter("set_end"); 
        self.end.insert(node, self.last_non_trivia_position);
    }

    fn set_pos_node(&mut self, node: &Node) { ::tsox_core::fntrace::enter("set_pos_node"); 
        self.set_pos(TriviaPositionKey::Node(node as *const Node));
    }

    fn set_end_node(&mut self, node: &Node) { ::tsox_core::fntrace::enter("set_end_node"); 
        self.set_end(TriviaPositionKey::Node(node as *const Node));
    }

    fn set_pos_node_list(&mut self, nodes: &NodeList) { ::tsox_core::fntrace::enter("set_pos_node_list"); 
        self.set_pos(TriviaPositionKey::NodeList(nodes as *const NodeList));
    }

    fn set_end_node_list(&mut self, nodes: &NodeList) { ::tsox_core::fntrace::enter("set_end_node_list"); 
        self.set_end(TriviaPositionKey::NodeList(nodes as *const NodeList));
    }

    fn get_pos(&self, node: TriviaPositionKey) -> usize { ::tsox_core::fntrace::enter("get_pos"); 
        self.pos[&node]
    }

    fn get_end(&self, node: TriviaPositionKey) -> usize { ::tsox_core::fntrace::enter("get_end"); 
        self.end[&node]
    }

    pub fn set_last_non_trivia_position(&mut self, s: &str, force: bool) { ::tsox_core::fntrace::enter("set_last_non_trivia_position"); 
        if force || skip_trivia(s, 0) != s.len() {
            self.last_non_trivia_position = self.writer.get_text_pos();
            let mut pos = s.len();
            for r in s.chars().rev() {
                if is_white_space_like(r) {
                    pos -= r.len_utf8();
                } else {
                    break;
                }
            }
            self.last_non_trivia_position -= s.len() - pos;
        }
    }

    pub fn assign_positions_to_node(
        &mut self,
        node: Option<&Arc<Node>>,
        _factory: &NodeFactory,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("assign_positions_to_node"); 
        let mut visitor = NodeVisitor {
            factory: tsox_frontend::ast::mig::m3c::NodeFactory::with_counters(
                tsox_frontend::ast::mig::m3c::NodeFactoryHooks::default(),
                0,
                0,
            ),
        };
        self.assign_positions_to_node_worker(node, &mut visitor)
    }

    fn assign_positions_to_node_worker(
        &mut self,
        node: Option<&Arc<Node>>,
        v: &mut NodeVisitor,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("assign_positions_to_node_worker"); 
        let node = node?;
        let visited = tsox_frontend::ast::mig::m3c::visit_each_child(node, v);
        let mut new_node = if Arc::ptr_eq(&visited, node) {
            crate::mig::m4g::r33k7_defs::clone_node(&visited, &v.factory)
        } else {
            visited
        };
        tsox_frontend::ast::node_data_generated::for_each_child(&new_node, |child| {
            child.set_parent(&new_node);
            true
        });
        let loc = TextRange::new(
            self.get_pos(TriviaPositionKey::Node(node.as_ref())),
            self.get_end(TriviaPositionKey::Node(node.as_ref())),
        );
        if let Some(n) = Arc::get_mut(&mut new_node) {
            n.loc = loc;
        } else {
            unsafe { (*(Arc::as_ptr(&new_node) as *mut Node)).loc = loc };
        }
        Some(new_node)
    }

    fn assign_positions_to_node_array(
        &mut self,
        nodes: Option<&NodeList>,
        v: &mut NodeVisitor,
    ) -> Option<NodeList> { ::tsox_core::fntrace::enter("assign_positions_to_node_array"); 
        let nodes = nodes?;
        let mut changed = false;
        let visited: Vec<Arc<Node>> = nodes
            .nodes
            .iter()
            .map(|n| match self.assign_positions_to_node_worker(Some(n), v) {
                Some(updated) if !Arc::ptr_eq(&updated, n) => {
                    changed = true;
                    updated
                }
                _ => Arc::clone(n),
            })
            .collect();
        if !changed {
            return None;
        }
        let mut node_array = NodeList::new(visited);
        node_array.loc = nodes.loc;
        node_array.loc = TextRange::new(
            self.get_pos(TriviaPositionKey::NodeList(nodes)) as usize,
            self.get_end(TriviaPositionKey::NodeList(nodes)) as usize,
        );
        Some(node_array)
    }
}
