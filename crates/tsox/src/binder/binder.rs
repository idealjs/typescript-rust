use super::*;

pub struct Binder {
    pub symbol_map: NodeSymbolMap,

    pub(crate) current_source_file: Option<Arc<SourceFile>>,

    pub(crate) container: Option<Arc<Node>>,

    pub(crate) block_scope_container: Option<Arc<Node>>,

    pub(crate) this_container: Option<Arc<Node>>,

    pub(crate) parent_symbol: Option<Arc<Symbol>>,

    pub(crate) current_flow: Option<Arc<FlowNode>>,

    pub(crate) symbol_count: usize,

    pub(crate) expando_assignments: Vec<(Arc<Node>, Option<Arc<Node>>)>,

    pub(crate) unreachable_flow: Option<Arc<FlowNode>>,

    pub(crate) current_break_target: Option<Arc<FlowNode>>,

    pub(crate) current_continue_target: Option<Arc<FlowNode>>,

    pub(crate) current_exception_target: Option<Arc<FlowNode>>,

    pub(crate) current_return_target: Option<Arc<FlowNode>>,

    pub(crate) active_label_list: Option<Box<ActiveLabel>>,

    pub(crate) has_explicit_return: bool,

    pub(crate) has_flow_effects: bool,
}

impl Default for Binder {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) enum DeclareTarget {
    Exports(Arc<Symbol>),

    Locals(Arc<Node>),
}

impl Binder {
    pub fn new() -> Self {
        Self {
            symbol_map: NodeSymbolMap::new(),
            current_source_file: None,
            container: None,
            block_scope_container: None,
            this_container: None,
            parent_symbol: None,
            current_flow: None,
            symbol_count: 0,
            expando_assignments: Vec::new(),
            unreachable_flow: None,
            current_break_target: None,
            current_continue_target: None,
            current_exception_target: None,
            current_return_target: None,
            active_label_list: None,
            has_explicit_return: false,
            has_flow_effects: false,
        }
    }

    pub fn bind_source_file(&mut self, file: &Arc<SourceFile>) -> &NodeSymbolMap {
        self.current_source_file = Some(Arc::clone(file));

        self.set_parent_pointers(&file.node);

        let start_flow = Arc::new(FlowNode::new(FlowFlags::START));
        self.current_flow = Some(Arc::clone(&start_flow));
        self.unreachable_flow = Some(Arc::new(FlowNode::new(FlowFlags::UNREACHABLE)));

        self.symbol_map
            .set_flow_node(&file.node, Arc::clone(&start_flow));

        let file_symbol = Arc::new(Symbol::new(
            SymbolFlags::ValueModule,
            file.file_name.clone(),
        ));
        {
            let file_symbol_mut = Arc::as_ptr(&file_symbol) as *mut Symbol;
            unsafe {
                (*file_symbol_mut).declarations.push(Arc::clone(&file.node));
                (*file_symbol_mut).value_declaration = Some(Arc::clone(&file.node));
            }
        }
        self.symbol_map
            .set_symbol(&file.node, Arc::clone(&file_symbol));
        self.symbol_count += 1;

        let prev_container = self.container.take();
        let prev_block = self.block_scope_container.take();
        let prev_parent = self.parent_symbol.take();

        self.container = Some(Arc::clone(&file.node));
        self.block_scope_container = Some(Arc::clone(&file.node));
        self.parent_symbol = Some(file_symbol);

        self.bind_children(&file.node);

        self.process_expando_assignments();

        self.container = prev_container;
        self.block_scope_container = prev_block;
        self.parent_symbol = prev_parent;

        &self.symbol_map
    }

    pub(crate) fn set_parent_pointers(&mut self, node: &Arc<Node>) {
        use crate::ast::node_data_generated::for_each_child;
        let mut children: Vec<Arc<Node>> = Vec::new();
        for_each_child(node, |child| {
            children.push(Arc::clone(child));
            false
        });
        let parent_clone = Arc::clone(node);
        for child in &children {
            let child_mut = Arc::as_ptr(child) as *mut Node;
            unsafe {
                (*child_mut).parent = Some(Arc::clone(&parent_clone));
            }
            self.set_parent_pointers(child);
        }
    }
}

pub fn bind_source_file(file: &Arc<SourceFile>) -> NodeSymbolMap {
    let mut binder = Binder::new();
    binder.bind_source_file(file);
    std::mem::take(&mut binder.symbol_map)
}
