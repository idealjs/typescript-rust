#![allow(invalid_reference_casting)]
#![allow(dead_code, unused_imports, unused_variables)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use tsox_frontend::ast::deep_clone_node;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated as ndg;
use tsox_frontend::ast::node_data_generated::NodeData;

use crate::printer::{EmitContext, NodeFactory};
use tsox_frontend::format::mig::m4o::EmitFlags;

use crate::mig::m4n_4::AssignedNameOptions;

thread_local! {
    static ORIGINAL_MAP: RefCell<HashMap<*const Node, Arc<Node>>> = RefCell::new(HashMap::new());
    static ASSIGNED_NAME_MAP: RefCell<HashMap<*const Node, Arc<Node>>> = RefCell::new(HashMap::new());
    static CLASS_THIS_MAP: RefCell<HashMap<*const Node, Arc<Node>>> = RefCell::new(HashMap::new());
    static TEXT_SOURCE_MAP: RefCell<HashMap<*const Node, Arc<Node>>> = RefCell::new(HashMap::new());
}

fn map_get(
    cell: &RefCell<HashMap<*const Node, Arc<Node>>>,
    node: &Arc<Node>,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("map_get"); 
    cell.borrow().get(&(Arc::as_ptr(node) as *const Node)).cloned()
}

fn map_set(
    cell: &RefCell<HashMap<*const Node, Arc<Node>>>,
    node: &Arc<Node>,
    value: &Arc<Node>,
) { ::tsox_core::fntrace::enter("map_set"); 
    cell.borrow_mut()
        .insert(Arc::as_ptr(node) as *const Node, Arc::clone(value));
}

fn map_remove(cell: &RefCell<HashMap<*const Node, Arc<Node>>>, node: &Arc<Node>) { ::tsox_core::fntrace::enter("map_remove"); 
    cell.borrow_mut().remove(&(Arc::as_ptr(node) as *const Node));
}

pub trait R39K12EmitContextMapsExt {
    fn r39k12_original_get(&self, node: &Arc<Node>) -> Option<Arc<Node>>;
    fn r39k12_original_set(&self, node: &Arc<Node>, original: &Arc<Node>);
    fn r39k12_original_remove(&self, node: &Arc<Node>);
    fn r39k12_assigned_name_get(&self, node: &Arc<Node>) -> Option<Arc<Node>>;
    fn r39k12_assigned_name_set(&mut self, node: &Arc<Node>, name: &Arc<Node>);
    fn r39k12_class_this_get(&self, node: &Arc<Node>) -> Option<Arc<Node>>;
    fn r39k12_class_this_set(&mut self, node: &Arc<Node>, class_this: &Arc<Node>);
    fn r39k12_text_source_get(&self, node: &Arc<Node>) -> Option<Arc<Node>>;
    fn r39k12_text_source_set(&mut self, node: &Arc<Node>, source: &Arc<Node>);
}

impl R39K12EmitContextMapsExt for EmitContext {
    fn r39k12_original_get(&self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("r39k12_original_get"); 
        ORIGINAL_MAP.with(|m| map_get(m, node))
    }

    fn r39k12_original_set(&self, node: &Arc<Node>, original: &Arc<Node>) { ::tsox_core::fntrace::enter("r39k12_original_set"); 
        ORIGINAL_MAP.with(|m| map_set(m, node, original));
    }

    fn r39k12_original_remove(&self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("r39k12_original_remove"); 
        ORIGINAL_MAP.with(|m| map_remove(m, node));
    }

    fn r39k12_assigned_name_get(&self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("r39k12_assigned_name_get"); 
        ASSIGNED_NAME_MAP.with(|m| map_get(m, node))
    }

    fn r39k12_assigned_name_set(&mut self, node: &Arc<Node>, name: &Arc<Node>) { ::tsox_core::fntrace::enter("r39k12_assigned_name_set"); 
        ASSIGNED_NAME_MAP.with(|m| map_set(m, node, name));
    }

    fn r39k12_class_this_get(&self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("r39k12_class_this_get"); 
        CLASS_THIS_MAP.with(|m| map_get(m, node))
    }

    fn r39k12_class_this_set(&mut self, node: &Arc<Node>, class_this: &Arc<Node>) { ::tsox_core::fntrace::enter("r39k12_class_this_set"); 
        CLASS_THIS_MAP.with(|m| map_set(m, node, class_this));
    }

    fn r39k12_text_source_get(&self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("r39k12_text_source_get"); 
        TEXT_SOURCE_MAP.with(|m| map_get(m, node))
    }

    fn r39k12_text_source_set(&mut self, node: &Arc<Node>, source: &Arc<Node>) { ::tsox_core::fntrace::enter("r39k12_text_source_set"); 
        TEXT_SOURCE_MAP.with(|m| map_set(m, node, source));
    }
}

impl NodeFactory<'_> {
    pub(crate) fn emit_context_mut(&self) -> &mut EmitContext { ::tsox_core::fntrace::enter("emit_context_mut"); 
        unsafe { &mut *(self.emit_context as *const EmitContext as *mut EmitContext) }
    }

    pub fn get_name(
        &self,
        node: &Arc<Node>,
        emit_flags: EmitFlags,
        opts: AssignedNameOptions,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("get_name"); 
        let node_name = tsox_frontend::ast::utilities::get_name_of_declaration(node);
        if let Some(node_name) = node_name {
            let mut emit_flags = emit_flags;
            let name = deep_clone_node(&node_name);
            if !opts.allow_comments {
                emit_flags |= EmitFlags::NO_COMMENTS;
            }
            if !opts.allow_source_maps {
                emit_flags |= EmitFlags::NO_SOURCE_MAP;
            }
            self.emit_context_mut().add_emit_flags(&name, emit_flags);
            return name;
        }
        self.generated_name_node(&self.new_generated_name_for_node(node))
    }
}
