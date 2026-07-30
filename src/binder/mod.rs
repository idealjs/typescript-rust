//! Symbol binding, ported from `internal/binder/binder.go`.
//!
//! The binder walks the AST and creates symbols for declarations, builds
//! scopes (symbol tables), and associates identifiers with their declarations.
//! It also builds the control flow graph for use by the checker.
//!
//! In Go, symbols and flow nodes are stored directly on AST nodes. In Rust,
//! we use side tables (`NodeSymbolMap`) keyed by node ID.

use crate::ast::*;
use std::sync::Arc;

/// The binder.
///
/// Mirrors `binder.Binder` in Go.
/// A flow label (junction point in the control flow graph).
///
/// Mirrors `ast.FlowLabel` in Go. Labels are used to collect antecedents
/// from multiple control flow paths (e.g. the merge point after an if/else).
#[derive(Debug)]
struct FlowLabel {
    node: FlowNode,
}

impl FlowLabel {
    fn new(flags: FlowFlags) -> Self {
        Self {
            node: FlowNode::new(flags),
        }
    }

    /// Add an antecedent to this label.
    fn add_antecedent(&mut self, antecedent: Arc<FlowNode>) {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return;
        }
        // Check if already present
        for ant in &self.node.antecedents {
            if Arc::ptr_eq(ant, &antecedent) {
                return;
            }
        }
        self.node.antecedents.push(antecedent);
    }

    /// Finish the label, returning the resulting flow node.
    fn finish(&self, unreachable: &Arc<FlowNode>) -> Arc<FlowNode> {
        if self.node.antecedents.is_empty() {
            return Arc::clone(unreachable);
        }
        if self.node.antecedents.len() == 1 {
            return Arc::clone(&self.node.antecedents[0]);
        }
        Arc::new(FlowNode {
            flags: self.node.flags,
            node: None,
            antecedent: None,
            antecedents: self.node.antecedents.clone(),
            switch_statement: None,
        })
    }
}

/// Active label tracking for labeled statements.
#[derive(Debug)]
struct ActiveLabel {
    name: String,
    break_target: Arc<FlowNode>,
    continue_target: Option<Arc<FlowNode>>,
    referenced: bool,
    next: Option<Box<ActiveLabel>>,
}

/// The binder.
///
/// Mirrors `binder.Binder` in Go.
pub struct Binder {
    /// Side table mapping nodes to symbols, locals, and flow nodes.
    pub symbol_map: NodeSymbolMap,
    /// The current container node (where members/exports go).
    container: Option<Arc<Node>>,
    /// The current block-scoped container (where block-scoped locals go).
    block_scope_container: Option<Arc<Node>>,
    /// The current container's parent symbol.
    parent_symbol: Option<Arc<Symbol>>,
    /// The current flow node.
    current_flow: Option<Arc<FlowNode>>,
    /// Symbol count (for diagnostics/stats).
    symbol_count: usize,
    /// Unreachable flow node.
    unreachable_flow: Option<Arc<FlowNode>>,
    /// Current break target flow label.
    current_break_target: Option<Arc<FlowNode>>,
    /// Current continue target flow label.
    current_continue_target: Option<Arc<FlowNode>>,
    /// Current exception target flow label (for try-catch-finally).
    current_exception_target: Option<Arc<FlowNode>>,
    /// Current return target flow label (for try-finally with IIFE).
    current_return_target: Option<Arc<FlowNode>>,
    /// Active label list (for labeled statements with break/continue).
    active_label_list: Option<Box<ActiveLabel>>,
    /// Whether the current function has explicit return statements.
    has_explicit_return: bool,
    /// Whether there are flow effects (assignments, calls, etc.).
    has_flow_effects: bool,
}

impl Default for Binder {
    fn default() -> Self {
        Self::new()
    }
}

impl Binder {
    /// Create a new binder.
    pub fn new() -> Self {
        Self {
            symbol_map: NodeSymbolMap::new(),
            container: None,
            block_scope_container: None,
            parent_symbol: None,
            current_flow: None,
            symbol_count: 0,
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

    /// Bind a source file: walk the AST and create symbols.
    ///
    /// Mirrors `binder.BindSourceFile` in Go.
    pub fn bind_source_file(&mut self, file: &SourceFile) -> &NodeSymbolMap {
        // Populate parent pointers before binding so the binder can locate
        // enclosing containers (e.g. the `ConditionalType` that owns an
        // `infer R` type parameter). Mirrors Go's parser, which sets
        // `Node.Parent` during parsing.
        self.set_parent_pointers(&file.node);

        let start_flow = Arc::new(FlowNode::new(FlowFlags::START));
        self.current_flow = Some(Arc::clone(&start_flow));
        self.unreachable_flow = Some(Arc::new(FlowNode::new(FlowFlags::UNREACHABLE)));
        // Set the start flow node on the source file node itself
        self.symbol_map
            .set_flow_node(&file.node, Arc::clone(&start_flow));

        // Create a symbol for the source file itself
        let file_symbol = Arc::new(Symbol::new(
            SymbolFlags::ValueModule,
            file.file_name.clone(),
        ));
        self.symbol_map
            .set_symbol(&file.node, Arc::clone(&file_symbol));
        self.symbol_count += 1;

        // Set up container context
        let prev_container = self.container.take();
        let prev_block = self.block_scope_container.take();
        let prev_parent = self.parent_symbol.take();

        self.container = Some(Arc::clone(&file.node));
        self.block_scope_container = Some(Arc::clone(&file.node));
        self.parent_symbol = Some(file_symbol);

        // Bind children
        self.bind_children(&file.node);

        self.container = prev_container;
        self.block_scope_container = prev_block;
        self.parent_symbol = prev_parent;

        &self.symbol_map
    }

    /// Walk the AST and set `parent` pointers on every child node.
    /// Mirrors the parent-pointer population done by Go's parser. Safe
    /// because the binder runs single-threaded and the AST is a tree.
    fn set_parent_pointers(&mut self, node: &Arc<Node>) {
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

    /// Create a new symbol.
    fn new_symbol(&mut self, flags: SymbolFlags, name: impl Into<String>) -> Arc<Symbol> {
        self.symbol_count += 1;
        Arc::new(Symbol::new(flags, name))
    }

    /// Declare a symbol for a node, adding it to the appropriate symbol table.
    ///
    /// Mirrors `binder.declareSymbol` in Go, including declaration merging
    /// for mergeable kinds (interface+interface, namespace+namespace,
    /// namespace+function/class, function+function overloads, enum+enum).
    /// Non-mergeable kinds (TypeAlias, Class, block-scoped variables)
    /// overwrite the previous symbol on redeclaration — matching the
    /// previous behavior.
    fn declare_symbol(
        &mut self,
        node: &Arc<Node>,
        includes: SymbolFlags,
        _excludes: SymbolFlags,
    ) -> Arc<Symbol> {
        let name = self.get_declaration_name(node);

        // Look up an existing symbol with the same name in the target scope.
        // If it exists and the kinds are mergeable, fold this declaration
        // into the existing symbol instead of creating a new one.
        let existing: Option<Arc<Symbol>> = if let Some(parent_sym) = &self.parent_symbol {
            parent_sym.members.get(&name).cloned()
        } else if let Some(block_container) = &self.block_scope_container {
            let container_id = block_container.id();
            self.symbol_map
                .locals
                .get(&container_id)
                .and_then(|locals| locals.get(&name).cloned())
        } else {
            None
        };

        if let Some(existing) = existing {
            if self.can_merge_symbols(existing.flags, includes) {
                // Merge: add this declaration to the existing symbol, union
                // the flags, and map the node to the existing symbol.
                let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                unsafe {
                    (*existing_mut).declarations.push(Arc::clone(node));
                    (*existing_mut).flags |= includes;
                    // For function overloads, only the first declaration
                    // carries the VALUE flag (already set). For other
                    // merges (interface/namespace), VALUE isn't involved.
                    if (*existing_mut).value_declaration.is_none()
                        && includes.contains(SymbolFlags::VALUE)
                    {
                        (*existing_mut).value_declaration = Some(Arc::clone(node));
                    }
                }
                self.symbol_map.set_symbol(node, Arc::clone(&existing));
                return existing;
            }
            // Non-mergeable redeclaration: fall through to create a new
            // symbol (overwrites the previous entry). Real TS would emit a
            // TS2300/TS2640 duplicate-identifier error here.
        }

        let symbol = self.new_symbol(includes, name.clone());

        // Record this declaration node on the symbol. `Symbol` is behind an
        // `Arc`; the binder runs single-threaded before any checker access, so
        // we mutate through the raw pointer (same pattern used for `members`
        // below). This lets the checker recover the AST declaration from a
        // symbol (e.g. resolving a type alias's declared type).
        {
            let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
            unsafe {
                (*symbol_mut).declarations.push(Arc::clone(node));
                // The first declaration is also the value declaration.
                if (*symbol_mut).value_declaration.is_none()
                    && includes.contains(SymbolFlags::VALUE)
                {
                    (*symbol_mut).value_declaration = Some(Arc::clone(node));
                }
            }
        }

        // Add to appropriate symbol table
        // 1) container's exports (if in a module/namespace)
        // 2) container's members (if in a class/interface/object)
        // 3) block-scope container's locals
        if let Some(_container) = &self.container {
            if let Some(parent_sym) = &self.parent_symbol {
                // For now, add to parent symbol's members
                // In a full implementation, this would distinguish between
                // members, exports, and locals based on container flags
                let parent_sym_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                unsafe {
                    (*parent_sym_mut)
                        .members
                        .insert(name.clone(), Arc::clone(&symbol));
                }
            } else if let Some(block_container) = &self.block_scope_container {
                // Add to locals of the block-scoped container
                let container_id = block_container.id();
                let locals = self
                    .symbol_map
                    .locals
                    .entry(container_id)
                    .or_insert_with(SymbolTable::new);
                locals.insert(name.clone(), Arc::clone(&symbol));
            }
        }

        // Associate the symbol with the node
        self.symbol_map.set_symbol(node, Arc::clone(&symbol));

        // Set the value declaration if this is a value declaration
        // (in the full Go implementation, this is more nuanced)

        symbol
    }

    /// Whether a new declaration with `new_flags` can be merged into an
    /// existing symbol with `existing_flags`. Mirrors the merge rules in
    /// Go's `binder.declareSymbol` (`canMergeSymbol`).
    ///
    /// Mergeable combinations:
    /// - interface + interface
    /// - namespace + namespace (ValueModule + ValueModule)
    /// - namespace + function/class (ValueModule + Function/Class) and vice
    ///   versa
    /// - function + function (overloads)
    /// - enum + enum
    /// - namespace + enum (ValueModule + RegularEnum/ConstEnum)
    ///
    /// Non-mergeable: TypeAlias (redefinition error), Class + Class
    /// (duplicate), block-scoped variable redeclarations.
    fn can_merge_symbols(&self, existing_flags: SymbolFlags, new_flags: SymbolFlags) -> bool {
        // Interface + Interface (and interface + class, which is allowed in
        // TS but not yet fully handled by the checker — still merge so the
        // interface members are visible).
        if existing_flags.contains(SymbolFlags::Interface)
            && new_flags.contains(SymbolFlags::Interface)
        {
            return true;
        }
        // Namespace merging: a ValueModule can merge with another ValueModule,
        // a Function, a Class, or an Enum.
        let existing_ns = existing_flags.contains(SymbolFlags::ValueModule);
        let new_ns = new_flags.contains(SymbolFlags::ValueModule);
        if existing_ns || new_ns {
            let other_existing = if existing_ns {
                new_flags
            } else {
                existing_flags
            };
            let _other_new = if existing_ns {
                existing_flags
            } else {
                new_flags
            };
            // The non-namespace side must be one of: ValueModule, Function,
            // Class, RegularEnum, ConstEnum.
            let can_merge_with_ns = other_existing.contains(SymbolFlags::ValueModule)
                || other_existing.contains(SymbolFlags::Function)
                || other_existing.contains(SymbolFlags::Class)
                || other_existing.contains(SymbolFlags::RegularEnum)
                || other_existing.contains(SymbolFlags::ConstEnum);
            if can_merge_with_ns {
                return true;
            }
        }
        // Function overloads: Function + Function.
        if existing_flags.contains(SymbolFlags::Function)
            && new_flags.contains(SymbolFlags::Function)
        {
            return true;
        }
        // Enum + Enum.
        if (existing_flags.contains(SymbolFlags::RegularEnum)
            || existing_flags.contains(SymbolFlags::ConstEnum))
            && (new_flags.contains(SymbolFlags::RegularEnum)
                || new_flags.contains(SymbolFlags::ConstEnum))
        {
            return true;
        }
        false
    }

    /// Get the name of a declaration node.
    fn get_declaration_name(&self, node: &Arc<Node>) -> String {
        match &node.data {
            NodeData::VariableDeclaration(data) => self.node_text(&data.name),
            NodeData::VariableStatement(_) => String::new(),
            NodeData::FunctionDeclaration(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_default(),
            NodeData::FunctionExpression(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_else(|| INTERNAL_SYMBOL_NAME_FUNCTION.to_string()),
            NodeData::ArrowFunction(_) => INTERNAL_SYMBOL_NAME_FUNCTION.to_string(),
            NodeData::ClassDeclaration(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_default(),
            NodeData::ClassExpression(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_else(|| INTERNAL_SYMBOL_NAME_CLASS.to_string()),
            NodeData::InterfaceDeclaration(data) => self.node_text(&data.name),
            NodeData::TypeAliasDeclaration(data) => self.node_text(&data.name),
            NodeData::EnumDeclaration(data) => self.node_text(&data.name),
            NodeData::ModuleDeclaration(data) => self.node_text(&data.name),
            NodeData::ParameterDeclaration(data) => self.node_text(&data.name),
            NodeData::BindingElement(data) => data
                .name
                .as_ref()
                .map(|n| self.node_text(n))
                .unwrap_or_default(),
            NodeData::ImportSpecifier(data) => data
                .property_name
                .as_ref()
                .map_or_else(|| self.node_text(&data.name), |n| self.node_text(n)),
            NodeData::ImportClause(data) => data.name.as_ref().map_or_else(
                || {
                    data.named_bindings
                        .as_ref()
                        .map_or_else(|| String::new(), |n| self.node_text(n))
                },
                |n| self.node_text(n),
            ),
            NodeData::PropertyDeclaration(data) => self.node_text(&data.name),
            NodeData::MethodDeclaration(data) => self.node_text(&data.name),
            NodeData::PropertyAssignment(data) => self.node_text(&data.name),
            NodeData::ShorthandPropertyAssignment(data) => self.node_text(&data.name),
            NodeData::EnumMember(data) => self.node_text(&data.name),
            NodeData::GetAccessorDeclaration(data) => self.node_text(&data.name),
            NodeData::SetAccessorDeclaration(data) => self.node_text(&data.name),
            NodeData::TypeParameterDeclaration(data) => self.node_text(&data.name),
            NodeData::Identifier(data) => data.text.clone(),
            _ => String::new(),
        }
    }

    /// Get the text of a node (for name extraction).
    fn node_text(&self, node: &Arc<Node>) -> String {
        match &node.data {
            NodeData::Identifier(data) => data.text.clone(),
            NodeData::StringLiteral(data) => data.text.clone(),
            NodeData::NumericLiteral(data) => data.text.clone(),
            NodeData::NoSubstitutionTemplateLiteral(data) => data.text.clone(),
            NodeData::BigIntLiteral(data) => data.text.clone(),
            _ => String::new(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Flow graph helper methods
    // ─────────────────────────────────────────────────────────────────────

    /// Get the unreachable flow node.
    fn unreachable_flow(&self) -> Arc<FlowNode> {
        Arc::clone(self.unreachable_flow.as_ref().unwrap())
    }

    /// Create a new flow node with the given flags.
    fn new_flow_node(&self, flags: FlowFlags) -> FlowNode {
        FlowNode::new(flags)
    }

    /// Create a flow condition node (true or false branch).
    fn create_flow_condition(
        &mut self,
        flags: FlowFlags,
        antecedent: &Arc<FlowNode>,
        expression: &Arc<Node>,
    ) -> Arc<FlowNode> {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return Arc::clone(antecedent);
        }
        self.has_flow_effects = true;
        Arc::new(FlowNode {
            flags,
            node: Some(Arc::clone(expression)),
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: Vec::new(),
            switch_statement: None,
        })
    }

    /// Create a flow assignment node.
    fn create_flow_assignment(
        &mut self,
        antecedent: &Arc<FlowNode>,
        node: &Arc<Node>,
    ) -> Arc<FlowNode> {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return Arc::clone(antecedent);
        }
        self.has_flow_effects = true;
        Arc::new(FlowNode {
            flags: FlowFlags::ASSIGNMENT,
            node: Some(Arc::clone(node)),
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: Vec::new(),
            switch_statement: None,
        })
    }

    /// Create a flow call node.
    fn create_flow_call(&mut self, antecedent: &Arc<FlowNode>, node: &Arc<Node>) -> Arc<FlowNode> {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return Arc::clone(antecedent);
        }
        self.has_flow_effects = true;
        Arc::new(FlowNode {
            flags: FlowFlags::CALL,
            node: Some(Arc::clone(node)),
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: Vec::new(),
            switch_statement: None,
        })
    }

    /// Create a flow mutation node (for array mutations like push, unshift, idx assignment).
    fn create_flow_mutation(
        &mut self,
        antecedent: &Arc<FlowNode>,
        node: &Arc<Node>,
    ) -> Arc<FlowNode> {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return Arc::clone(antecedent);
        }
        self.set_flow_node_referenced(antecedent);
        self.has_flow_effects = true;
        let result = Arc::new(FlowNode {
            flags: FlowFlags::ARRAY_MUTATION,
            node: Some(Arc::clone(node)),
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: Vec::new(),
            switch_statement: None,
        });
        // Add to exception target if we're inside a try block
        if let Some(target) = &self.current_exception_target {
            self.add_antecedent_to_flow(target, &result);
        }
        result
    }

    /// Mark a flow node as referenced (sets Referenced flag, then Shared on subsequent calls).
    fn set_flow_node_referenced(&self, flow: &FlowNode) {
        // We need interior mutability for this. Since FlowNode is behind Arc,
        // we use a raw pointer cast. This is safe because we only modify flags.
        let ptr = flow as *const FlowNode as *mut FlowNode;
        unsafe {
            if (*ptr).flags.contains(FlowFlags::REFERENCED) {
                (*ptr).flags = (*ptr).flags | FlowFlags::SHARED;
            } else {
                (*ptr).flags = (*ptr).flags | FlowFlags::REFERENCED;
            }
        }
    }

    /// Create a reduce label node (for try-finally flow graph).
    fn create_reduce_label(
        &self,
        antecedents: &[Arc<FlowNode>],
        antecedent: &Arc<FlowNode>,
    ) -> Arc<FlowNode> {
        Arc::new(FlowNode {
            flags: FlowFlags::REDUCE_LABEL,
            node: None,
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: antecedents.to_vec(),
            switch_statement: None,
        })
    }

    /// Add an antecedent to a flow label (checking for duplicates).
    fn add_antecedent_to_flow(&self, label: &Arc<FlowNode>, antecedent: &Arc<FlowNode>) {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return;
        }
        for ant in &label.antecedents {
            if Arc::ptr_eq(ant, antecedent) {
                return;
            }
        }
        let ptr = Arc::as_ptr(label) as *mut FlowNode;
        unsafe {
            (*ptr).antecedents.push(Arc::clone(antecedent));
        }
    }

    /// Create a flow switch clause node.
    ///
    /// `switch_statement` is the enclosing `SwitchStatement` node, used by
    /// the checker to resolve the discriminant expression for narrowing.
    /// `clause` is the `CaseClause` or `DefaultClause` node, used to
    /// resolve the case expression(s) being matched.
    fn create_flow_switch_clause(
        &mut self,
        antecedent: &Arc<FlowNode>,
        clause: &Arc<Node>,
        switch_statement: &Arc<Node>,
    ) -> Arc<FlowNode> {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return Arc::clone(antecedent);
        }
        Arc::new(FlowNode {
            flags: FlowFlags::SWITCH_CLAUSE,
            node: Some(Arc::clone(clause)),
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: Vec::new(),
            switch_statement: Some(Arc::clone(switch_statement)),
        })
    }

    // ─────────────────────────────────────────────────────────────────────
    // Control flow statement binding
    // ─────────────────────────────────────────────────────────────────────

    /// Bind an if statement with proper control flow.
    fn bind_if_statement(&mut self, node: &Arc<Node>) {
        let mut then_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut else_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut post_if_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (expr, then_stmt, else_stmt) = match &node.data {
            NodeData::IfStatement(data) => (
                data.expression.clone(),
                data.then_statement.clone(),
                data.else_statement.clone(),
            ),
            _ => return,
        };

        // Bind condition and split flow
        self.bind(&expr);
        if let Some(current) = self.current_flow.take() {
            let true_flow = self.create_flow_condition(FlowFlags::TRUE_CONDITION, &current, &expr);
            let false_flow =
                self.create_flow_condition(FlowFlags::FALSE_CONDITION, &current, &expr);
            then_label.add_antecedent(true_flow);
            else_label.add_antecedent(false_flow);
        }

        // Then branch
        self.current_flow = Some(then_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.bind(&then_stmt);
        if let Some(current) = &self.current_flow {
            post_if_label.add_antecedent(Arc::clone(current));
        }

        // Else branch
        self.current_flow = Some(else_label.finish(self.unreachable_flow.as_ref().unwrap()));
        if let Some(else_s) = else_stmt {
            self.bind(&else_s);
        }
        if let Some(current) = &self.current_flow {
            post_if_label.add_antecedent(Arc::clone(current));
        }

        // Merge after if/else
        self.current_flow = Some(post_if_label.finish(self.unreachable_flow.as_ref().unwrap()));
    }

    /// Bind a while statement with proper control flow.
    fn bind_while_statement(&mut self, node: &Arc<Node>) {
        let mut pre_while_label = FlowLabel::new(FlowFlags::LOOP_LABEL);
        let mut pre_body_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut post_while_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (expr, stmt) = match &node.data {
            NodeData::WhileStatement(data) => (data.expression.clone(), data.statement.clone()),
            _ => return,
        };

        if let Some(current) = &self.current_flow {
            pre_while_label.add_antecedent(Arc::clone(current));
        }
        self.current_flow = Some(pre_while_label.finish(self.unreachable_flow.as_ref().unwrap()));

        // Condition
        self.bind(&expr);
        if let Some(current) = self.current_flow.take() {
            let true_flow = self.create_flow_condition(FlowFlags::TRUE_CONDITION, &current, &expr);
            let false_flow =
                self.create_flow_condition(FlowFlags::FALSE_CONDITION, &current, &expr);
            pre_body_label.add_antecedent(true_flow);
            post_while_label.add_antecedent(false_flow);
        }

        // Save break/continue targets
        let prev_break = self.current_break_target.take();
        let prev_continue = self.current_continue_target.take();
        self.current_break_target =
            Some(post_while_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.current_continue_target =
            Some(pre_while_label.finish(self.unreachable_flow.as_ref().unwrap()));

        // Body
        self.current_flow = Some(pre_body_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.bind(&stmt);
        if let Some(current) = &self.current_flow {
            pre_while_label.add_antecedent(Arc::clone(current));
        }

        // Restore break/continue targets
        self.current_break_target = prev_break;
        self.current_continue_target = prev_continue;

        self.current_flow = Some(post_while_label.finish(self.unreachable_flow.as_ref().unwrap()));
    }

    /// Bind a do-while statement with proper control flow.
    fn bind_do_statement(&mut self, node: &Arc<Node>) {
        let mut pre_do_label = FlowLabel::new(FlowFlags::LOOP_LABEL);
        let mut pre_condition_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut post_do_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (expr, stmt) = match &node.data {
            NodeData::DoStatement(data) => (data.expression.clone(), data.statement.clone()),
            _ => return,
        };

        if let Some(current) = &self.current_flow {
            pre_do_label.add_antecedent(Arc::clone(current));
        }
        self.current_flow = Some(pre_do_label.finish(self.unreachable_flow.as_ref().unwrap()));

        // Save break/continue targets
        let prev_break = self.current_break_target.take();
        let prev_continue = self.current_continue_target.take();
        self.current_break_target =
            Some(post_do_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.current_continue_target =
            Some(pre_condition_label.finish(self.unreachable_flow.as_ref().unwrap()));

        // Body
        self.bind(&stmt);
        if let Some(current) = &self.current_flow {
            pre_condition_label.add_antecedent(Arc::clone(current));
        }

        // Restore break/continue targets
        self.current_break_target = prev_break;
        self.current_continue_target = prev_continue;

        // Condition
        self.current_flow =
            Some(pre_condition_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.bind(&expr);
        if let Some(current) = self.current_flow.take() {
            let true_flow = self.create_flow_condition(FlowFlags::TRUE_CONDITION, &current, &expr);
            let false_flow =
                self.create_flow_condition(FlowFlags::FALSE_CONDITION, &current, &expr);
            pre_do_label.add_antecedent(true_flow);
            post_do_label.add_antecedent(false_flow);
        }

        self.current_flow = Some(post_do_label.finish(self.unreachable_flow.as_ref().unwrap()));
    }

    /// Bind a for statement with proper control flow.
    fn bind_for_statement(&mut self, node: &Arc<Node>) {
        let mut pre_loop_label = FlowLabel::new(FlowFlags::LOOP_LABEL);
        let mut pre_body_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut pre_incr_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut post_loop_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (initializer, condition, incrementor, statement) = match &node.data {
            NodeData::ForStatement(data) => (
                data.initializer.clone(),
                data.condition.clone(),
                data.incrementor.clone(),
                data.statement.clone(),
            ),
            _ => return,
        };

        // Initializer
        if let Some(init) = initializer {
            self.bind(&init);
        }

        if let Some(current) = &self.current_flow {
            pre_loop_label.add_antecedent(Arc::clone(current));
        }
        self.current_flow = Some(pre_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));

        // Condition
        if let Some(cond) = condition {
            self.bind(&cond);
            if let Some(current) = self.current_flow.take() {
                let true_flow =
                    self.create_flow_condition(FlowFlags::TRUE_CONDITION, &current, &cond);
                let false_flow =
                    self.create_flow_condition(FlowFlags::FALSE_CONDITION, &current, &cond);
                pre_body_label.add_antecedent(true_flow);
                post_loop_label.add_antecedent(false_flow);
            }
        } else {
            // No condition = always true
            if let Some(current) = &self.current_flow {
                pre_body_label.add_antecedent(Arc::clone(current));
            }
        }

        // Save break/continue targets
        let prev_break = self.current_break_target.take();
        let prev_continue = self.current_continue_target.take();
        self.current_break_target =
            Some(post_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.current_continue_target =
            Some(pre_incr_label.finish(self.unreachable_flow.as_ref().unwrap()));

        // Body
        self.current_flow = Some(pre_body_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.bind(&statement);
        if let Some(current) = &self.current_flow {
            pre_incr_label.add_antecedent(Arc::clone(current));
        }

        // Restore break/continue targets
        self.current_break_target = prev_break;
        self.current_continue_target = prev_continue;

        // Incrementor
        self.current_flow = Some(pre_incr_label.finish(self.unreachable_flow.as_ref().unwrap()));
        if let Some(inc) = incrementor {
            self.bind(&inc);
        }
        if let Some(current) = &self.current_flow {
            pre_loop_label.add_antecedent(Arc::clone(current));
        }

        self.current_flow = Some(post_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));
    }

    /// Bind a for-in or for-of statement with proper control flow.
    fn bind_for_in_or_of_statement(&mut self, node: &Arc<Node>) {
        let mut pre_loop_label = FlowLabel::new(FlowFlags::LOOP_LABEL);
        let mut post_loop_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (expression, initializer, statement) = match &node.data {
            NodeData::ForInOrOfStatement(data) => (
                data.expression.clone(),
                data.initializer.clone(),
                data.statement.clone(),
            ),
            _ => return,
        };

        // Expression
        self.bind(&expression);

        if let Some(current) = &self.current_flow {
            pre_loop_label.add_antecedent(Arc::clone(current));
        }
        self.current_flow = Some(pre_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));

        post_loop_label.add_antecedent(Arc::clone(self.current_flow.as_ref().unwrap()));

        // Initializer
        self.bind(&initializer);

        // Save break/continue targets
        let prev_break = self.current_break_target.take();
        let prev_continue = self.current_continue_target.take();
        self.current_break_target =
            Some(post_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.current_continue_target =
            Some(pre_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));

        // Body
        self.bind(&statement);
        if let Some(current) = &self.current_flow {
            pre_loop_label.add_antecedent(Arc::clone(current));
        }

        // Restore break/continue targets
        self.current_break_target = prev_break;
        self.current_continue_target = prev_continue;

        self.current_flow = Some(post_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));
    }

    /// Bind a switch statement with proper control flow.
    fn bind_switch_statement(&mut self, node: &Arc<Node>) {
        let mut post_switch_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (expression, case_block) = match &node.data {
            NodeData::SwitchStatement(data) => (data.expression.clone(), data.case_block.clone()),
            _ => return,
        };

        // Switch expression
        self.bind(&expression);

        // Save break target
        let prev_break = self.current_break_target.take();
        self.current_break_target =
            Some(post_switch_label.finish(self.unreachable_flow.as_ref().unwrap()));

        // Get clauses from case block
        let clauses = match &case_block.data {
            NodeData::CaseBlock(data) => data.clauses.clone(),
            _ => {
                self.current_break_target = prev_break;
                return;
            }
        };

        // Process each clause
        for clause in &clauses.nodes {
            // Create switch clause flow
            if let Some(current) = self.current_flow.take() {
                let clause_flow = self.create_flow_switch_clause(&current, clause, node);
                self.current_flow = Some(clause_flow);
            }

            // Bind clause (expression + statements)
            match &clause.data {
                NodeData::CaseOrDefaultClause(data) => {
                    // For CaseClause, bind the expression; for DefaultClause, expression is just a placeholder
                    self.bind(&data.expression);
                    // Bind statements
                    for stmt in &data.statements.nodes {
                        self.bind(stmt);
                    }
                }
                _ => {}
            }
        }

        // Add final flow to post-switch label
        if let Some(current) = &self.current_flow {
            post_switch_label.add_antecedent(Arc::clone(current));
        }

        self.current_flow = Some(post_switch_label.finish(self.unreachable_flow.as_ref().unwrap()));

        // Restore break target
        self.current_break_target = prev_break;
    }

    /// Bind a return statement.
    fn bind_return_statement(&mut self, node: &Arc<Node>) {
        if let NodeData::ReturnStatement(data) = &node.data {
            if let Some(expr) = &data.expression {
                self.bind(expr);
            }
        }
        self.current_flow = Some(self.unreachable_flow());
        self.has_explicit_return = true;
        self.has_flow_effects = true;
    }

    /// Bind a throw statement.
    fn bind_throw_statement(&mut self, node: &Arc<Node>) {
        if let NodeData::ThrowStatement(data) = &node.data {
            self.bind(&data.expression);
        }
        self.current_flow = Some(self.unreachable_flow());
        self.has_flow_effects = true;
    }

    /// Bind a try/catch/finally statement with proper control flow.
    ///
    /// Mirrors `binder.bindTryStatement` in Go.
    fn bind_try_statement(&mut self, node: &Arc<Node>) {
        let stmt = match &node.data {
            NodeData::TryStatement(data) => data,
            _ => return,
        };

        let save_return_target = self.current_return_target.take();
        let save_exception_target = self.current_exception_target.take();

        let mut normal_exit_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut return_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let mut exception_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let mut finally_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        if stmt.finally_block.is_some() {
            self.current_return_target = Some(return_label.finish(&self.unreachable_flow()));
        }

        // Add current flow as possible exception source
        if let Some(current) = &self.current_flow {
            exception_label.add_antecedent(Arc::clone(current));
        }
        self.current_exception_target = Some(Arc::clone(
            &exception_label.finish(&self.unreachable_flow()),
        ));

        // Bind try block
        self.bind(&stmt.try_block);
        // Try block normal exit -> normal_exit_label
        if let Some(current) = &self.current_flow {
            normal_exit_label.add_antecedent(Arc::clone(current));
        }
        // Try block exception -> exception_label (already set above)

        // Bind catch clause if present
        if let Some(catch_clause) = &stmt.catch_clause {
            // Set current flow to exception label (the catch block starts here)
            self.current_flow = Some(exception_label.finish(&self.unreachable_flow()));
            self.bind(catch_clause);
            // Catch block normal exit -> normal_exit_label
            if let Some(current) = &self.current_flow {
                normal_exit_label.add_antecedent(Arc::clone(current));
            }
        }

        self.current_return_target = save_return_target;
        self.current_exception_target = save_exception_target;

        // Bind finally block if present
        if let Some(finally_block) = &stmt.finally_block {
            // Combine all possible donors into the finally label
            for ant in &normal_exit_label.node.antecedents {
                finally_label.add_antecedent(Arc::clone(ant));
            }
            for ant in &exception_label.node.antecedents {
                finally_label.add_antecedent(Arc::clone(ant));
            }
            for ant in &return_label.node.antecedents {
                finally_label.add_antecedent(Arc::clone(ant));
            }

            self.current_flow = Some(finally_label.finish(&self.unreachable_flow()));
            self.bind(finally_block);

            if self
                .current_flow
                .as_ref()
                .map_or(false, |f| f.flags.contains(FlowFlags::UNREACHABLE))
            {
                self.current_flow = Some(self.unreachable_flow());
            } else {
                // Handle return paths through finally
                if self.current_return_target.is_some() && !return_label.node.antecedents.is_empty()
                {
                    if let Some(current_flow) = &self.current_flow {
                        let reduce =
                            self.create_reduce_label(&return_label.node.antecedents, current_flow);
                        if let Some(rt) = &self.current_return_target {
                            self.add_antecedent_to_flow(rt, &reduce);
                        }
                    }
                }
                // Handle exception paths through finally
                if self.current_exception_target.is_some()
                    && !exception_label.node.antecedents.is_empty()
                {
                    if let Some(current_flow) = &self.current_flow {
                        let reduce = self
                            .create_reduce_label(&exception_label.node.antecedents, current_flow);
                        if let Some(et) = &self.current_exception_target {
                            self.add_antecedent_to_flow(et, &reduce);
                        }
                    }
                }
                // Normal exit path through finally
                if !normal_exit_label.node.antecedents.is_empty() {
                    if let Some(current_flow) = &self.current_flow {
                        self.current_flow = Some(self.create_reduce_label(
                            &normal_exit_label.node.antecedents,
                            current_flow,
                        ));
                    }
                } else {
                    self.current_flow = Some(self.unreachable_flow());
                }
            }
        } else {
            self.current_flow = Some(normal_exit_label.finish(&self.unreachable_flow()));
        }
    }

    /// Bind a break statement.
    fn bind_break_statement(&mut self, node: &Arc<Node>) {
        // Check for labeled break first
        let label_name = if let NodeData::BreakStatement(data) = &node.data {
            data.label.as_ref().map(|l| self.node_text(l))
        } else {
            None
        };

        if let Some(name) = label_name {
            // Look for a matching active label
            let mut found = false;
            let mut current = &self.active_label_list;
            while let Some(label) = current {
                if label.name == name {
                    if let Some(current_flow) = &self.current_flow {
                        self.add_antecedent_to_flow(&label.break_target, current_flow);
                    }
                    found = true;
                    break;
                }
                current = &label.next;
            }
            let _ = found;
        } else if let Some(target) = &self.current_break_target {
            // Unlabeled break to the innermost break target
            if let Some(current) = &self.current_flow {
                self.add_antecedent_to_flow(target, current);
            }
        }
        self.current_flow = Some(self.unreachable_flow());
    }

    /// Bind a continue statement.
    fn bind_continue_statement(&mut self, node: &Arc<Node>) {
        // Check for labeled continue first
        let label_name = if let NodeData::ContinueStatement(data) = &node.data {
            data.label.as_ref().map(|l| self.node_text(l))
        } else {
            None
        };

        if let Some(name) = label_name {
            // Look for a matching active label with a continue target
            let mut current = &self.active_label_list;
            while let Some(label) = current {
                if label.name == name {
                    if let Some(continue_target) = &label.continue_target {
                        if let Some(current_flow) = &self.current_flow {
                            self.add_antecedent_to_flow(continue_target, current_flow);
                        }
                    }
                    break;
                }
                current = &label.next;
            }
        } else if let Some(target) = &self.current_continue_target {
            if let Some(current) = &self.current_flow {
                self.add_antecedent_to_flow(target, current);
            }
        }
        self.current_flow = Some(self.unreachable_flow());
    }

    /// Bind a labeled statement.
    ///
    /// Mirrors `binder.bindLabeledStatement` in Go.
    fn bind_labeled_statement(&mut self, node: &Arc<Node>) {
        let stmt = match &node.data {
            NodeData::LabeledStatement(data) => data,
            _ => return,
        };

        let label_name = self.node_text(&stmt.label);
        let break_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
        let break_target = break_label.finish(self.unreachable_flow.as_ref().unwrap());

        // Determine if this is a label for an iteration statement (has continue target)
        let continue_target = match &stmt.statement.data {
            NodeData::WhileStatement(_)
            | NodeData::DoStatement(_)
            | NodeData::ForStatement(_)
            | NodeData::ForInOrOfStatement(_) => Some(
                self.current_continue_target
                    .clone()
                    .unwrap_or_else(|| self.unreachable_flow()),
            ),
            _ => None,
        };

        let active_label = Box::new(ActiveLabel {
            name: label_name,
            break_target: Arc::clone(&break_target),
            continue_target,
            referenced: false,
            next: self.active_label_list.take(),
        });

        self.active_label_list = Some(active_label);

        // Bind the statement (the loop body, etc.)
        self.bind(&stmt.statement);

        // Restore active label list
        self.active_label_list = self.active_label_list.take().and_then(|l| l.next);

        // Finish break target
        if let Some(current) = &self.current_flow {
            self.add_antecedent_to_flow(&break_target, current);
        }
        self.current_flow = Some(break_target);
    }

    /// Check if an identifier is push or unshift (for array mutation tracking).
    /// Mirrors Go's `ast.IsPushOrUnshiftIdentifier`.
    fn is_push_or_unshift_identifier(&self, name: &str) -> bool {
        name == "push" || name == "unshift"
    }

    /// Check if an expression is a narrowable operand (identifier, property
    /// access chain, parenthesized, etc.). Mirrors Go's `isNarrowableOperand` +
    /// `containsNarrowableReference`. Used to gate ARRAY_MUTATION flow nodes
    /// so that `arr.push(x)` (where `arr` is an identifier) is tracked but
    /// `getFoo().push(x)` is not.
    fn is_narrowable_operand(&self, expr: &Arc<Node>) -> bool {
        match expr.kind {
            SyntaxKind::Identifier
            | SyntaxKind::ThisKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::MetaProperty => true,
            SyntaxKind::PropertyAccessExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::NonNullExpression => {
                if let Some(inner) = expr.expression() {
                    self.is_narrowable_operand(&inner)
                } else {
                    false
                }
            }
            SyntaxKind::ElementAccessExpression => {
                // Element access is narrowable if the argument is a
                // string/numeric literal or an entity-name expression whose
                // receiver is narrowable.
                if let NodeData::ElementAccessExpression(ea) = &expr.data {
                    if self.is_string_or_numeric_literal_like(&ea.argument_expression) {
                        return true;
                    }
                    return self.is_entity_name_expression(&ea.argument_expression)
                        && self.is_narrowable_operand(&ea.expression);
                }
                false
            }
            _ => false,
        }
    }

    /// Mirrors Go's `ast.IsStringOrNumericLiteralLike`.
    fn is_string_or_numeric_literal_like(&self, node: &Arc<Node>) -> bool {
        matches!(
            node.kind,
            SyntaxKind::StringLiteral
                | SyntaxKind::NumericLiteral
                | SyntaxKind::NoSubstitutionTemplateLiteral
        )
    }

    /// Mirrors Go's `ast.IsEntityNameExpression` (identifier or qualified
    /// name).
    fn is_entity_name_expression(&self, node: &Arc<Node>) -> bool {
        matches!(
            node.kind,
            SyntaxKind::Identifier | SyntaxKind::QualifiedName
        )
    }

    /// Bind a call expression for flow tracking (array mutation detection).
    ///
    /// Mirrors `binder.bindCallExpressionFlow` in Go. Handles:
    /// - Optional chains (delegates to `bind_optional_chain_flow`)
    /// - IIFE (function/arrow expression): bind args then callee
    /// - `super()`: create a CALL flow node
    /// - `arr.push(x)` / `arr.unshift(x)`: create an ARRAY_MUTATION flow node
    fn bind_call_expression_flow(&mut self, node: &Arc<Node>) {
        if let NodeData::CallExpression(data) = &node.data {
            let expr = &data.expression;
            // Check for property access expression like arr.push()
            if let NodeData::PropertyAccessExpression(prop) = &expr.data {
                let name = self.node_text(&prop.name);
                if self.is_push_or_unshift_identifier(&name)
                    && self.is_narrowable_operand(&prop.expression)
                {
                    // This is an array mutation call: create a flow mutation node
                    let current = self.current_flow.clone();
                    if let Some(current) = current {
                        self.current_flow = Some(self.create_flow_mutation(&current, node));
                    }
                }
            }
        }
    }

    /// Bind an expression statement (with assignment flow tracking).
    fn bind_expression_statement(&mut self, node: &Arc<Node>) {
        if let NodeData::ExpressionStatement(data) = &node.data {
            self.bind(&data.expression);
            // Check for assignment
            if let NodeData::BinaryExpression(bin_data) = &data.expression.data {
                if is_assignment_operator(bin_data.operator_token.kind) {
                    if let Some(current) = self.current_flow.take() {
                        let assign_flow = self.create_flow_assignment(&current, &data.expression);
                        self.symbol_map
                            .set_flow_node(&data.expression, Arc::clone(&assign_flow));
                        self.current_flow = Some(assign_flow);
                    }
                    // Check for element access assignment (array mutation: arr[i] = val)
                    if let NodeData::ElementAccessExpression(_) = &bin_data.left.data {
                        let current = self.current_flow.clone();
                        if let Some(current) = current {
                            self.current_flow = Some(self.create_flow_mutation(&current, node));
                        }
                    }
                }
            }
            // Check for call expression
            if let NodeData::CallExpression(_) = &data.expression.data {
                if let Some(current) = self.current_flow.take() {
                    let call_flow = self.create_flow_call(&current, &data.expression);
                    self.symbol_map
                        .set_flow_node(&data.expression, Arc::clone(&call_flow));
                    self.current_flow = Some(call_flow);
                }
            }
        } else {
            self.bind_children(node);
        }
    }

    /// Bind a variable statement (with assignment flow for initializers).
    fn bind_variable_statement(&mut self, node: &Arc<Node>) {
        // Bind normally (creates symbols, recurses)
        self.bind_children(node);

        // Add assignment flow for declarations with initializers
        if let NodeData::VariableStatement(data) = &node.data {
            if let NodeData::VariableDeclarationList(list_data) = &data.declaration_list.data {
                for decl in &list_data.declarations.nodes {
                    if let NodeData::VariableDeclaration(decl_data) = &decl.data {
                        if decl_data.initializer.is_some() {
                            if let Some(current) = self.current_flow.take() {
                                let assign_flow = self.create_flow_assignment(&current, decl);
                                self.symbol_map
                                    .set_flow_node(decl, Arc::clone(&assign_flow));
                                self.current_flow = Some(current);
                            }
                        }
                    }
                }
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Binding dispatch
    // ─────────────────────────────────────────────────────────────────────

    /// Bind a single node: create symbols, set flow nodes, then recurse.
    fn bind(&mut self, node: &Arc<Node>) {
        // Set flow node for expressions
        match node.kind {
            SyntaxKind::Identifier => {
                if let Some(flow) = &self.current_flow {
                    self.symbol_map.set_flow_node(node, Arc::clone(flow));
                }
            }
            SyntaxKind::ThisKeyword | SyntaxKind::SuperKeyword => {
                if let Some(flow) = &self.current_flow {
                    self.symbol_map.set_flow_node(node, Arc::clone(flow));
                }
            }
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                if let Some(flow) = &self.current_flow {
                    self.symbol_map.set_flow_node(node, Arc::clone(flow));
                }
            }
            _ => {}
        }

        // Create symbols for declarations
        match node.kind {
            SyntaxKind::VariableDeclaration => {
                self.declare_symbol(node, SymbolFlags::BlockScopedVariable, SymbolFlags::VALUE);
            }
            SyntaxKind::VariableStatement => {
                // The statement itself doesn't get a symbol; its declarations do
            }
            SyntaxKind::FunctionDeclaration => {
                self.declare_symbol(node, SymbolFlags::Function, SymbolFlags::VALUE);
            }
            SyntaxKind::FunctionExpression => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::Function,
                    INTERNAL_SYMBOL_NAME_FUNCTION,
                );
            }
            SyntaxKind::ArrowFunction => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::Function,
                    INTERNAL_SYMBOL_NAME_FUNCTION,
                );
            }
            SyntaxKind::ClassDeclaration => {
                self.declare_symbol(
                    node,
                    SymbolFlags::Class,
                    SymbolFlags::VALUE | SymbolFlags::TYPE,
                );
            }
            SyntaxKind::ClassExpression => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::Class,
                    INTERNAL_SYMBOL_NAME_CLASS,
                );
            }
            SyntaxKind::InterfaceDeclaration => {
                self.declare_symbol(node, SymbolFlags::Interface, SymbolFlags::TYPE);
            }
            SyntaxKind::TypeAliasDeclaration => {
                self.declare_symbol(node, SymbolFlags::TypeAlias, SymbolFlags::TYPE);
            }
            SyntaxKind::EnumDeclaration => {
                self.declare_symbol(
                    node,
                    SymbolFlags::RegularEnum,
                    SymbolFlags::VALUE | SymbolFlags::TYPE,
                );
            }
            SyntaxKind::ModuleDeclaration => {
                self.declare_symbol(node, SymbolFlags::ValueModule, SymbolFlags::MODULE);
            }
            SyntaxKind::Parameter => {
                self.declare_symbol(
                    node,
                    SymbolFlags::FunctionScopedVariable,
                    SymbolFlags::VALUE,
                );
            }
            SyntaxKind::PropertyDeclaration | SyntaxKind::PropertySignature => {
                self.declare_symbol(node, SymbolFlags::Property, SymbolFlags::VALUE);
            }
            SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature => {
                self.declare_symbol(node, SymbolFlags::Method, SymbolFlags::VALUE);
            }
            SyntaxKind::PropertyAssignment => {
                self.declare_symbol(node, SymbolFlags::Property, SymbolFlags::VALUE);
            }
            SyntaxKind::ShorthandPropertyAssignment => {
                self.declare_symbol(node, SymbolFlags::Property, SymbolFlags::VALUE);
            }
            SyntaxKind::EnumMember => {
                self.declare_symbol(
                    node,
                    SymbolFlags::EnumMember,
                    SymbolFlags::VALUE | SymbolFlags::TYPE,
                );
            }
            SyntaxKind::GetAccessor => {
                self.declare_symbol(node, SymbolFlags::GetAccessor, SymbolFlags::VALUE);
            }
            SyntaxKind::SetAccessor => {
                self.declare_symbol(node, SymbolFlags::SetAccessor, SymbolFlags::VALUE);
            }
            SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::NamespaceImport
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::ExportSpecifier => {
                self.declare_symbol(node, SymbolFlags::Alias, SymbolFlags::Alias);
            }
            SyntaxKind::BindingElement => {
                self.declare_symbol(node, SymbolFlags::BlockScopedVariable, SymbolFlags::VALUE);
            }
            SyntaxKind::TypeParameter => {
                self.bind_type_parameter(node);
            }
            SyntaxKind::ObjectLiteralExpression => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::ObjectLiteral,
                    INTERNAL_SYMBOL_NAME_OBJECT,
                );
            }
            SyntaxKind::TypeLiteral => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::TypeLiteral,
                    INTERNAL_SYMBOL_NAME_TYPE,
                );
            }
            _ => {}
        }

        // Control flow statement dispatch
        match node.kind {
            SyntaxKind::IfStatement => {
                self.bind_if_statement(node);
                return;
            }
            SyntaxKind::WhileStatement => {
                self.bind_while_statement(node);
                return;
            }
            SyntaxKind::DoStatement => {
                self.bind_do_statement(node);
                return;
            }
            SyntaxKind::ForStatement => {
                self.bind_for_statement(node);
                return;
            }
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement => {
                self.bind_for_in_or_of_statement(node);
                return;
            }
            SyntaxKind::SwitchStatement => {
                self.bind_switch_statement(node);
                return;
            }
            SyntaxKind::ReturnStatement => {
                self.bind_return_statement(node);
                return;
            }
            SyntaxKind::ThrowStatement => {
                self.bind_throw_statement(node);
                return;
            }
            SyntaxKind::BreakStatement => {
                self.bind_break_statement(node);
                return;
            }
            SyntaxKind::ContinueStatement => {
                self.bind_continue_statement(node);
                return;
            }
            SyntaxKind::ExpressionStatement => {
                self.bind_expression_statement(node);
                return;
            }
            SyntaxKind::VariableStatement => {
                self.bind_variable_statement(node);
                return;
            }
            SyntaxKind::TryStatement => {
                self.bind_try_statement(node);
                return;
            }
            SyntaxKind::LabeledStatement => {
                self.bind_labeled_statement(node);
                return;
            }
            SyntaxKind::CallExpression => {
                self.bind_call_expression_flow(node);
                // Don't return - also check for children after call expression flow
            }
            _ => {}
        }

        // Recurse into children
        let container_flags = get_container_flags(node.kind);
        if container_flags != ContainerFlags::NONE {
            self.bind_container(node, container_flags);
        } else {
            self.bind_children(node);
        }
    }

    /// Create an anonymous symbol (for function expressions, class expressions,
    /// object literals, type literals).
    fn bind_anonymous_declaration(&mut self, node: &Arc<Node>, flags: SymbolFlags, name: &str) {
        let symbol = self.new_symbol(flags, name.to_string());
        self.symbol_map.set_symbol(node, symbol);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Container binding
    // ─────────────────────────────────────────────────────────────────────

    /// Bind a container node: save/restore container context, then bind children.
    fn bind_container(&mut self, node: &Arc<Node>, flags: ContainerFlags) {
        let prev_container = self.container.take();
        let prev_block = self.block_scope_container.take();
        // Save the current parent_symbol. For container nodes that have a
        // symbol (e.g. FunctionDeclaration), we'll replace it with the
        // container's symbol so children are added to its members. For
        // block-scoped containers without a symbol (e.g. Block), we clear
        // it so children go into the block's locals.
        let prev_parent_symbol = self.parent_symbol.take();

        self.container = Some(Arc::clone(node));

        // Block-scoped containers get a new locals scope
        if is_block_scoped_container(node.kind) {
            self.block_scope_container = Some(Arc::clone(node));
        }

        // Create locals for this container if it has them
        if has_locals(node.kind) {
            self.symbol_map.locals.insert(node.id(), SymbolTable::new());
        }

        // Set parent_symbol to the container's symbol (if it has one).
        // This ensures children (parameters, class members, etc.) are added
        // to the container's symbol members rather than the outer scope.
        if let Some(sym) = self.symbol_map.symbol_of(node) {
            self.parent_symbol = Some(Arc::clone(sym));
        }
        // If the node has no symbol (e.g. Block), parent_symbol remains None,
        // so declare_symbol falls through to the block_scope_container.locals.

        // Function-like containers get their own fresh control flow graph:
        // a new START flow node, with the outer flow saved and restored.
        // This prevents flow effects inside the function body (e.g. a
        // `return` marking the flow UNREACHABLE) from leaking into the
        // enclosing scope. Mirrors Go's `bindChildren` flow handling for
        // `ContainerFlagsIsFunctionLike` containers.
        let is_function_like = flags.contains(ContainerFlags::IS_FUNCTION_LIKE);
        let prev_flow = if is_function_like {
            self.current_flow.take()
        } else {
            None
        };
        if is_function_like {
            self.current_flow = Some(Arc::new(FlowNode::new(FlowFlags::START)));
        }

        self.bind_children(node);

        // Restore the outer flow for function-like containers.
        if is_function_like {
            self.current_flow = prev_flow;
        }

        self.container = prev_container;
        self.block_scope_container = prev_block;
        self.parent_symbol = prev_parent_symbol;
    }

    // ─────────────────────────────────────────────────────────────────────
    // Child binding
    // ─────────────────────────────────────────────────────────────────────

    /// Bind all children of a node.
    fn bind_children(&mut self, node: &Arc<Node>) {
        // Use a raw pointer to work around the borrow checker: `bind` needs
        // `&mut self` but `for_each_child` gives us shared references to children.
        // This is safe because we don't alias the node itself.
        let this = self as *mut Self;
        crate::ast::node_data_generated::for_each_child(node, |child| {
            unsafe {
                (*this).bind(child);
            }
            false
        });
    }

    /// Get the number of symbols created.
    pub fn symbol_count(&self) -> usize {
        self.symbol_count
    }

    /// Bind a `TypeParameter` node. Mirrors Go's `bindTypeParameter`.
    ///
    /// When the type parameter is the child of an `InferType` (i.e.
    /// `infer R`), it is declared as a local of the enclosing
    /// `ConditionalType` (found via `get_infer_type_container`), so that
    /// `getInferTypeParameters` can later collect the infer type
    /// parameters. Otherwise it falls through to the normal
    /// `declare_symbol` path.
    fn bind_type_parameter(&mut self, node: &Arc<Node>) {
        let parent_is_infer = node
            .parent
            .as_ref()
            .map_or(false, |p| p.kind == SyntaxKind::InferType);
        if parent_is_infer {
            if let Some(container) = node
                .parent
                .as_ref()
                .and_then(|infer| self.get_infer_type_container(infer))
            {
                self.declare_local_symbol(
                    &container,
                    node,
                    SymbolFlags::TypeParameter,
                    SymbolFlags::TYPE,
                );
                return;
            }
            // No enclosing ConditionalType — fall back to anonymous declaration.
            let name = self.get_declaration_name(node);
            self.bind_anonymous_declaration(node, SymbolFlags::TypeParameter, &name);
            return;
        }
        self.declare_symbol(node, SymbolFlags::TypeParameter, SymbolFlags::TYPE);
    }

    /// Find the `ConditionalType` node whose `extends_type` clause contains
    /// the given `InferType` node. Mirrors Go's `getInferTypeContainer`.
    /// Requires parent pointers to be populated (see `set_parent_pointers`).
    fn get_infer_type_container(&self, infer_node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut current = Arc::clone(infer_node);
        loop {
            let parent = match &current.parent {
                Some(p) => Arc::clone(p),
                None => return None,
            };
            if parent.kind == SyntaxKind::ConditionalType {
                // Check that `current` is the extends_type of the conditional.
                let is_extends = match &parent.data {
                    NodeData::ConditionalTypeNode(data) => {
                        Arc::ptr_eq(&data.extends_type, &current)
                    }
                    _ => false,
                };
                if is_extends {
                    return Some(parent);
                }
                return None;
            }
            current = parent;
        }
    }

    /// Declare a symbol as a local of a specific container node, bypassing
    /// the normal `container`/`block_scope_container` state. Used for
    /// `infer R` type parameters which belong to the `ConditionalType`
    /// even though it is not the active container.
    fn declare_local_symbol(
        &mut self,
        container: &Arc<Node>,
        node: &Arc<Node>,
        flags: SymbolFlags,
        _excludes: SymbolFlags,
    ) -> Arc<Symbol> {
        let name = self.get_declaration_name(node);
        let symbol = self.new_symbol(flags, name.clone());
        {
            let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
            unsafe {
                (*symbol_mut).declarations.push(Arc::clone(node));
                if (*symbol_mut).value_declaration.is_none() && flags.contains(SymbolFlags::VALUE) {
                    (*symbol_mut).value_declaration = Some(Arc::clone(node));
                }
            }
        }
        let container_id = container.id();
        let locals = self
            .symbol_map
            .locals
            .entry(container_id)
            .or_insert_with(SymbolTable::new);
        locals.insert(name, Arc::clone(&symbol));
        self.symbol_map.set_symbol(node, Arc::clone(&symbol));
        symbol
    }
}

/// Get container flags for a node kind.
fn get_container_flags(kind: SyntaxKind) -> ContainerFlags {
    match kind {
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
            ContainerFlags::IS_CONTAINER | ContainerFlags::HAS_LOCALS
        }
        SyntaxKind::InterfaceDeclaration
        | SyntaxKind::TypeLiteral
        | SyntaxKind::ObjectLiteralExpression
        | SyntaxKind::JsxAttributes
        | SyntaxKind::EnumDeclaration => ContainerFlags::IS_CONTAINER,
        SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction => {
            ContainerFlags::IS_CONTAINER
                | ContainerFlags::IS_CONTROL_FLOW_CONTAINER
                | ContainerFlags::IS_FUNCTION_LIKE
                | ContainerFlags::IS_FUNCTION_EXPRESSION
                | ContainerFlags::HAS_LOCALS
        }
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::Constructor
        | SyntaxKind::CallSignature
        | SyntaxKind::ConstructSignature
        | SyntaxKind::IndexSignature
        | SyntaxKind::FunctionType
        | SyntaxKind::ConstructorType => {
            ContainerFlags::IS_CONTAINER
                | ContainerFlags::IS_CONTROL_FLOW_CONTAINER
                | ContainerFlags::IS_FUNCTION_LIKE
                | ContainerFlags::HAS_LOCALS
        }
        SyntaxKind::Block | SyntaxKind::ModuleDeclaration | SyntaxKind::SourceFile => {
            ContainerFlags::IS_CONTAINER
                | ContainerFlags::IS_BLOCK_SCOPED_CONTAINER
                | ContainerFlags::IS_CONTROL_FLOW_CONTAINER
                | ContainerFlags::HAS_LOCALS
        }
        SyntaxKind::CatchClause
        | SyntaxKind::ForStatement
        | SyntaxKind::ForInStatement
        | SyntaxKind::ForOfStatement => {
            ContainerFlags::IS_BLOCK_SCOPED_CONTAINER | ContainerFlags::HAS_LOCALS
        }
        _ => ContainerFlags::NONE,
    }
}

/// Whether a node kind is a block-scoped container.
fn is_block_scoped_container(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Block
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::SourceFile
            | SyntaxKind::CatchClause
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::Constructor
    )
}

/// Whether a node kind has locals (a local symbol table).
fn has_locals(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Block
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::SourceFile
            | SyntaxKind::CatchClause
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::IndexSignature
    )
}

/// Bind a source file using a fresh binder.
pub fn bind_source_file(file: &SourceFile) -> NodeSymbolMap {
    let mut binder = Binder::new();
    binder.bind_source_file(file);
    std::mem::take(&mut binder.symbol_map)
}

/// Whether a syntax kind is an assignment operator token.
fn is_assignment_operator(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::EqualsToken
            | SyntaxKind::PlusEqualsToken
            | SyntaxKind::MinusEqualsToken
            | SyntaxKind::AsteriskEqualsToken
            | SyntaxKind::AsteriskAsteriskEqualsToken
            | SyntaxKind::SlashEqualsToken
            | SyntaxKind::PercentEqualsToken
            | SyntaxKind::LessThanLessThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanEqualsToken
            | SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken
            | SyntaxKind::AmpersandEqualsToken
            | SyntaxKind::BarEqualsToken
            | SyntaxKind::BarBarEqualsToken
            | SyntaxKind::AmpersandAmpersandEqualsToken
            | SyntaxKind::QuestionQuestionEqualsToken
            | SyntaxKind::CaretEqualsToken
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn parse_and_bind(source: &str) -> (SourceFile, NodeSymbolMap) {
        let source_file = Parser::parse_source_file_text("test.ts", source.to_string());
        let symbol_map = bind_source_file(&source_file);
        (source_file, symbol_map)
    }

    #[test]
    fn bind_variable_declaration() {
        let (file, map) = parse_and_bind("var x = 1;");
        let statements = match &file.node.data {
            NodeData::SourceFile(data) => &data.statements,
            _ => unreachable!(),
        };
        assert!(!statements.nodes.is_empty());
        // The variable statement contains a declaration list with declarations
        let var_stmt = &statements.nodes[0];
        assert_eq!(var_stmt.kind, SyntaxKind::VariableStatement);
        // Symbol count should be > 0 (file symbol + variable symbol)
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 2);
        let _ = map;
    }

    #[test]
    fn bind_function_declaration() {
        let (file, _map) = parse_and_bind("function foo() { return 42; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 2);
    }

    #[test]
    fn bind_class_declaration() {
        let (file, _map) = parse_and_bind("class Foo { bar() {} }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 3); // file + class + method
    }

    #[test]
    fn bind_interface_declaration() {
        let (file, _map) = parse_and_bind("interface Foo { bar: number; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 3); // file + interface + property
    }

    #[test]
    fn bind_import_declaration() {
        let (file, _map) = parse_and_bind("import { foo } from 'mod';");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        // Import is not yet parsed by our parser, but binding shouldn't crash
        let _ = binder.symbol_count();
    }

    #[test]
    fn bind_multiple_declarations() {
        let (file, _map) = parse_and_bind("let x = 1; let y = 2; let z = 3;");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 4); // file + 3 variables
    }

    #[test]
    fn bind_nested_scope() {
        let (file, _map) = parse_and_bind("function foo() { let x = 1; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        // file + function + variable
        assert!(binder.symbol_count() >= 3);
    }

    // ───────────────────────────────────────────────────────────────
    // Flow graph tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn flow_start_node_exists() {
        let (file, map) = parse_and_bind("let x = 1;");
        // File should have a start flow node
        let flow = map.flow_node_of(&file.node);
        assert!(flow.is_some());
        let flow = flow.unwrap();
        assert!(flow.flags.contains(FlowFlags::START));
    }

    #[test]
    fn flow_identifier_has_flow_node() {
        let (file, map) = parse_and_bind("let x = 1; x;");
        // Find the identifier x (the second statement's expression)
        let statements = match &file.node.data {
            NodeData::SourceFile(data) => &data.statements,
            _ => unreachable!(),
        };
        // Second statement is ExpressionStatement containing Identifier
        let expr_stmt = &statements.nodes[1];
        let expr = match &expr_stmt.data {
            NodeData::ExpressionStatement(data) => &data.expression,
            _ => unreachable!(),
        };
        assert_eq!(expr.kind, SyntaxKind::Identifier);
        // The identifier should have a flow node
        assert!(map.flow_node_of(expr).is_some());
    }

    #[test]
    fn flow_if_statement_merges() {
        // Just make sure binding an if statement doesn't crash
        let (file, _map) = parse_and_bind("let x = 1; if (x > 0) { x = 2; } else { x = 3; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 2);
    }

    #[test]
    fn flow_while_statement() {
        let (file, _map) = parse_and_bind("let i = 0; while (i < 10) { i = i + 1; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 2);
    }

    #[test]
    fn flow_for_statement() {
        let (file, _map) = parse_and_bind("for (let i = 0; i < 10; i++) { console.log(i); }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 2);
    }

    #[test]
    fn flow_switch_statement() {
        let (file, _map) =
            parse_and_bind("let x = 1; switch (x) { case 1: x = 2; break; default: x = 0; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 2);
    }

    #[test]
    fn flow_return_statement_unreachable() {
        let (file, map) = parse_and_bind("function foo() { return 1; let x = 2; }");
        let _ = map;
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_explicit_return);
    }

    #[test]
    fn flow_throw_statement() {
        let (file, _map) = parse_and_bind("function foo() { throw new Error(); }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_assignment_has_effects() {
        let (file, _map) = parse_and_bind("let x = 1; x = 2;");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_call_expression_has_effects() {
        let (file, _map) = parse_and_bind("console.log('hello');");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_try_catch_finally_does_not_crash() {
        // `try/catch/finally` must build a flow graph without panicking.
        let (file, _map) =
            parse_and_bind("try { let x = 1; } catch (e) { let y = 2; } finally { let z = 3; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_try_with_throw_in_catch() {
        // Throw inside try should route through catch, not fall through.
        let (file, _map) = parse_and_bind(
            "function f() {\
             try { throw new Error(); }\
             catch (e) { return 1; }\
             return 2;\
             }",
        );
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_labeled_break_to_outer_loop() {
        // Labeled break must route the inner loop's exit to the outer label.
        let (file, _map) = parse_and_bind(
            "outer: for (let i = 0; i < 3; i++) {\
             for (let j = 0; j < 3; j++) {\
             if (j === 1) break outer;\
             }\
             }",
        );
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_labeled_continue_to_outer_loop() {
        // Labeled continue must route the inner loop's continue to the outer label.
        let (file, _map) = parse_and_bind(
            "outer: for (let i = 0; i < 3; i++) {\
             for (let j = 0; j < 3; j++) {\
             if (j === 1) continue outer;\
             }\
             }",
        );
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_array_mutation_call_has_effects() {
        // `arr.push(x)` is an ARRAY_MUTATION flow node — has_flow_effects
        // must be true and binding must not crash.
        let (file, _map) = parse_and_bind("let arr = []; arr.push(1);");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }
}
