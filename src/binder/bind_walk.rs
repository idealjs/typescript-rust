use super::*;

impl Binder {
    pub(crate) fn bind(&mut self, node: &Arc<Node>) {

        match node.kind {
            SyntaxKind::Identifier => {
                if let Some(flow) = &self.current_flow {
                    self.symbol_map.set_flow_node(node, Arc::clone(flow));
                }
                self.check_contextual_identifier(node);
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

        match node.kind {
            SyntaxKind::VariableDeclaration => {
                self.declare_symbol(node, SymbolFlags::BlockScopedVariable, SymbolFlags::VALUE);
            }
            SyntaxKind::VariableStatement => {

            }
            SyntaxKind::FunctionDeclaration => {
                self.declare_symbol(node, SymbolFlags::Function, SymbolFlags::VALUE);
            }
            SyntaxKind::FunctionExpression => {

                let name = match &node.data {
                    NodeData::FunctionExpression(data) => {
                        data.name.as_ref().map(|n| self.node_text(n))
                    }
                    _ => None,
                }
                .unwrap_or_else(|| INTERNAL_SYMBOL_NAME_FUNCTION.to_string());
                self.bind_anonymous_declaration(node, SymbolFlags::Function, &name);
            }
            SyntaxKind::ArrowFunction => {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::Function,
                    INTERNAL_SYMBOL_NAME_FUNCTION,
                );
            }
            SyntaxKind::ClassDeclaration => {
                let class_symbol = self.declare_symbol(
                    node,
                    SymbolFlags::Class,
                    SymbolFlags::VALUE | SymbolFlags::TYPE,
                );

                let prototype = Arc::new(Symbol::new(
                    SymbolFlags::Property | SymbolFlags::Prototype,
                    "prototype",
                ));
                let class_mut = Arc::as_ptr(&class_symbol) as *mut Symbol;
                unsafe {
                    (*class_mut).exports.insert("prototype", Arc::clone(&prototype));
                    let proto_mut = Arc::as_ptr(&prototype) as *mut Symbol;
                    (*proto_mut).parent = Some(Arc::clone(&class_symbol));
                }
            }
            SyntaxKind::ClassExpression => {
                let has_name = matches!(
                    &node.data,
                    NodeData::ClassExpression(data) if data.name.is_some()
                );
                if has_name {

                    self.bind_anonymous_declaration(
                        node,
                        SymbolFlags::Class,
                        INTERNAL_SYMBOL_NAME_CLASS,
                    );
                } else {
                    self.bind_anonymous_declaration(
                        node,
                        SymbolFlags::Class,
                        INTERNAL_SYMBOL_NAME_CLASS,
                    );
                }
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

                let dotted_name = match &node.data {
                    crate::ast::NodeData::ModuleDeclaration(md) => match md.name.kind {
                        SyntaxKind::Identifier => md.name.text().to_string(),
                        SyntaxKind::QualifiedName => {
                            fn qualified_text(n: &Arc<Node>) -> String {
                                match &n.data {
                                    crate::ast::NodeData::QualifiedName(q) => {
                                        format!("{}.{}", qualified_text(&q.left), q.right.text())
                                    }
                                    _ => n.text().to_string(),
                                }
                            }
                            qualified_text(&md.name)
                        }
                        _ => String::new(),
                    },
                    _ => String::new(),
                };
                if dotted_name.contains('.') {
                    let parts: Vec<&str> = dotted_name.split('.').collect();

                    let container = self.container.clone();
                    let parent_sym = self.parent_symbol.clone();
                    let mut table: Option<Arc<Symbol>> = None;
                    let mut locals_key: Option<u64> = None;
                    if let Some(ps) = &parent_sym {
                        table = Some(Arc::clone(ps));
                    } else if let Some(c) = &container {
                        locals_key = Some(c.id());
                    }
                    let mut current: Option<Arc<Symbol>> = None;
                    for part in &parts[..parts.len() - 1] {
                        let existing = current.as_ref().map_or_else(
                            || {
                                table.as_ref().and_then(|t| {
                                    t.members.get(*part).cloned().or_else(|| t.exports.get(*part).cloned())
                                }).or_else(|| {
                                    locals_key
                                        .and_then(|k| self.symbol_map.locals.get(&k))
                                        .and_then(|l| l.get(*part).cloned())
                                })
                            },
                            |cur| cur.exports.get(*part).cloned(),
                        );
                        let sym = match existing {
                            Some(s) if s.flags.contains(SymbolFlags::ValueModule) => s,
                            _ => {
                                let fresh = Arc::new(Symbol::new(
                                    SymbolFlags::ValueModule,
                                    part.to_string(),
                                ));
                                if let Some(cur) = &current {
                                    let cur_mut = Arc::as_ptr(cur) as *mut Symbol;
                                    unsafe {
                                        (*cur_mut).exports.insert(part.to_string(), Arc::clone(&fresh));
                                    }
                                } else if let Some(t) = &table {
                                    let t_mut = Arc::as_ptr(t) as *mut Symbol;
                                    unsafe {
                                        (*t_mut).members.insert(part.to_string(), Arc::clone(&fresh));
                                    }
                                } else if let Some(k) = locals_key {
                                    self.symbol_map
                                        .locals
                                        .entry(k)
                                        .or_default()
                                        .insert(part.to_string(), Arc::clone(&fresh));
                                }
                                fresh
                            }
                        };
                        current = Some(sym);
                    }

                    let last = parts[parts.len() - 1];
                    let symbol = Arc::new(Symbol::new(SymbolFlags::ValueModule, last.to_string()));
                    {
                        let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                        unsafe {
                            (*symbol_mut).declarations.push(Arc::clone(node));
                        }
                    }
                    match &current {
                        Some(cur) => {
                            let cur_mut = Arc::as_ptr(cur) as *mut Symbol;
                            unsafe {
                                (*cur_mut).exports.insert(last.to_string(), Arc::clone(&symbol));
                            }
                        }
                        None => {
                            if let Some(t) = &table {
                                let t_mut = Arc::as_ptr(t) as *mut Symbol;
                                unsafe {
                                    (*t_mut).members.insert(last.to_string(), Arc::clone(&symbol));
                                }
                            } else if let Some(k) = locals_key {
                                self.symbol_map
                                    .locals
                                    .entry(k)
                                    .or_default()
                                    .insert(last.to_string(), Arc::clone(&symbol));
                            }
                        }
                    }
                    self.symbol_map.set_symbol(node, Arc::clone(&symbol));
                } else {
                    self.declare_symbol(node, SymbolFlags::ValueModule, SymbolFlags::MODULE);
                }
            }
            SyntaxKind::Parameter => {

                let report_2371 = |b: &mut Self, loc: crate::core::text::TextRange| {
                    b.symbol_map.binder_diagnostics.push(Diagnostic::new(
                        b.current_source_file.clone(),
                        loc,
                        A_PARAMETER_INITIALIZER_IS_ONLY_ALLOWED_IN_A_FUNCTION_OR_CONSTRUCTOR_IMPLEMENTATION,
                        vec![],
                    ));
                };
                if let NodeData::ParameterDeclaration(pd) = &node.data
                    && let Some(parent) = node.parent.as_ref()
                    && !fn_like_body_present(parent)
                {
                    if pd.initializer.is_some() {
                        report_2371(self, node.loc);
                    } else {

                        let mut elements: Vec<&Arc<Node>> = Vec::new();
                        collect_binding_elements(&pd.name, &mut elements);
                        for el in elements {
                            if matches!(&el.data, NodeData::BindingElement(be) if be.initializer.is_some()) {
                                report_2371(self, el.loc);
                            }
                        }
                    }
                }
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

            SyntaxKind::ImportClause => {
                self.bind_import_clause(node);
            }

            SyntaxKind::ExportAssignment => {
                self.bind_export_assignment(node);
            }

            SyntaxKind::ExportDeclaration => {
                self.bind_export_declaration(node);
            }

            SyntaxKind::NamespaceExportDeclaration => {
                self.bind_namespace_export_declaration(node);
            }
            SyntaxKind::BindingElement => {
                self.declare_symbol(node, SymbolFlags::BlockScopedVariable, SymbolFlags::VALUE);
            }
            SyntaxKind::TypeParameter => {

                if let Some(list) = node.parent.as_ref()
                    && let Some(name) = node.name()
                    && name.kind == SyntaxKind::Identifier
                {
                    let mut dup = false;
                    crate::ast::node_data_generated::for_each_child(list, |sibling| {
                        if Arc::ptr_eq(sibling, node) {
                            return true;
                        }
                        if sibling.kind == SyntaxKind::TypeParameter
                            && sibling
                                .name()
                                .is_some_and(|sn| sn.text() == name.text())
                        {
                            dup = true;
                        }
                        false
                    });
                    if dup {
                        self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                            self.current_source_file.clone(),
                            name.loc,
                            DUPLICATE_IDENTIFIER_0,
                            vec![name.text().to_string()],
                        ));
                    }
                }
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

                self.bind_children(node);
                return;
            }
            SyntaxKind::VariableDeclaration | SyntaxKind::BindingElement => {

                self.bind_children(node);
                let has_initializer = match &node.data {
                    NodeData::VariableDeclaration(d) => d.initializer.is_some(),
                    NodeData::BindingElement(d) => d.initializer.is_some(),
                    _ => false,
                };
                if has_initializer || Self::is_in_for_in_or_of_head(node) {
                    self.bind_initialized_variable_flow(node);
                }
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

            }
            SyntaxKind::BinaryExpression => {

                self.bind_this_property_assignment(node);

                self.collect_expando_assignment(node);

                if matches!(&node.data, NodeData::BinaryExpression(bin)
                    if bin.operator_token.kind == SyntaxKind::EqualsToken
                        && matches!(
                            bin.left.kind,
                            SyntaxKind::ObjectLiteralExpression
                                | SyntaxKind::ArrayLiteralExpression
                        ))
                {
                    if let NodeData::BinaryExpression(bin) = &node.data {
                        let left = Arc::clone(&bin.left);
                        self.bind_assignment_target_flow(&left);
                    }
                }

                if let NodeData::BinaryExpression(bin) = &node.data {
                    let op = bin.operator_token.kind;

                    let parent_is_expr_stmt = node
                        .parent
                        .as_ref()
                        .is_some_and(|p| p.kind == SyntaxKind::ExpressionStatement);
                    if is_assignment_operator(op)
                        && matches!(bin.left.kind, SyntaxKind::Identifier)
                        && !parent_is_expr_stmt
                    {
                        let left = Arc::clone(&bin.left);
                        let right = Arc::clone(&bin.right);
                        self.bind(&left);
                        self.bind(&right);
                        if let Some(current) = self.current_flow.take() {
                            self.current_flow =
                                Some(self.create_flow_assignment(&current, node));
                        }
                        return;
                    }
                    if matches!(op, SyntaxKind::AmpersandAmpersandToken | SyntaxKind::BarBarToken)
                    {
                        let left = Arc::clone(&bin.left);
                        let right = Arc::clone(&bin.right);
                        self.bind(&left);
                        if let Some(current) = self.current_flow.take() {
                            let is_and = op == SyntaxKind::AmpersandAmpersandToken;

                            let rhs_flags = if is_and {
                                FlowFlags::TRUE_CONDITION
                            } else {
                                FlowFlags::FALSE_CONDITION
                            };
                            let keep_flags = if is_and {
                                FlowFlags::FALSE_CONDITION
                            } else {
                                FlowFlags::TRUE_CONDITION
                            };
                            let keep =
                                self.create_flow_condition(keep_flags, &current, &left);
                            let cond = self.create_flow_condition(rhs_flags, &current, &left);
                            self.current_flow = Some(cond);
                            self.bind(&right);

                            let after_right = self.current_flow.take();
                            let mut label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
                            label.add_antecedent(keep);
                            if let Some(ar) = after_right {
                                label.add_antecedent(ar);
                            }
                            self.current_flow = Some(
                                label.finish(self.unreachable_flow.as_ref().unwrap()),
                            );
                        } else {
                            self.bind(&right);
                        }
                        return;
                    }
                }
            }
            _ => {}
        }

        let container_flags = get_container_flags(node.kind);
        if node.kind == SyntaxKind::PropertyDeclaration
            && matches!(&node.data, NodeData::PropertyDeclaration(d) if d.initializer.is_some())
        {

            let prev_flow = self.current_flow.take();
            self.current_flow = Some(Arc::new(FlowNode::new(FlowFlags::START)));
            self.bind_children(node);
            self.current_flow = prev_flow;
        } else if container_flags != ContainerFlags::NONE {
            self.bind_container(node, container_flags);
        } else {
            self.bind_children(node);

            if node.kind == SyntaxKind::CallExpression {
                if let Some(current) = self.current_flow.take() {
                    let call_flow = self.create_flow_call(&current, node);
                    self.current_flow = Some(call_flow);
                }
            }
        }
    }

    fn bind_anonymous_declaration(&mut self, node: &Arc<Node>, flags: SymbolFlags, name: &str) {
        let symbol = self.new_symbol(flags, name.to_string());
        self.symbol_map.set_symbol(node, symbol);
    }

    fn bind_import_clause(&mut self, node: &Arc<Node>) {
        let has_name = matches!(&node.data, NodeData::ImportClause(data) if data.name.is_some());
        if !has_name {
            return;
        }
        if let Some(container) = &self.container {
            self.declare_symbol_into(
                node,
                SymbolFlags::Alias,
                SymbolFlags::AliasExcludes,
                DeclareTarget::Locals(Arc::clone(container)),
            );
        }
    }

    fn bind_export_assignment(&mut self, node: &Arc<Node>) {
        let (is_export_equals, expr_kind) = match &node.data {
            NodeData::ExportAssignment(data) => (data.is_export_equals, data.expression.kind),
            _ => return,
        };
        let parent_sym = match self.parent_symbol.clone() {
            Some(s) => s,
            None => {

                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::VALUE,
                    &self.get_declaration_name(node),
                );
                return;
            }
        };

        let is_alias = matches!(
            expr_kind,
            SyntaxKind::Identifier | SyntaxKind::QualifiedName | SyntaxKind::ClassExpression
        );
        let flags = if is_alias {
            SymbolFlags::Alias
        } else {
            SymbolFlags::Property
        };
        let symbol = self.declare_symbol_into(
            node,
            flags,
            SymbolFlags::all(),
            DeclareTarget::Exports(parent_sym),
        );
        if is_export_equals {

            let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
            unsafe {
                (*symbol_mut).value_declaration = Some(Arc::clone(node));
            }
        }
    }

    fn bind_export_declaration(&mut self, node: &Arc<Node>) {
        let export_clause: Option<Arc<Node>> = match &node.data {
            NodeData::ExportDeclaration(data) => data.export_clause.clone(),
            _ => return,
        };
        let parent_sym = match self.parent_symbol.clone() {
            Some(s) => s,
            None => {

                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::ExportStar,
                    &self.get_declaration_name(node),
                );
                return;
            }
        };
        match &export_clause {
            None => {

                self.declare_symbol_into(
                    node,
                    SymbolFlags::ExportStar,
                    SymbolFlags::None,
                    DeclareTarget::Exports(parent_sym),
                );
            }
            Some(clause) if clause.kind == SyntaxKind::NamespaceExport => {

                let name = self.get_declaration_name(clause);
                let merged_with_members = parent_sym
                    .members
                    .get(&name)
                    .cloned()
                    .filter(|existing| self.can_merge_symbols(existing.flags, SymbolFlags::Alias))
                    .map(|existing| {
                        let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                        unsafe {
                            (*existing_mut).declarations.push(Arc::clone(clause));
                            (*existing_mut).flags |= SymbolFlags::Alias;
                        }
                        existing
                    });
                if let Some(merged) = merged_with_members {
                    let parent_mut = Arc::as_ptr(&parent_sym) as *mut Symbol;
                    unsafe {
                        (*parent_mut).exports.insert(name, merged.clone());
                    }
                    self.symbol_map.set_symbol(clause, merged.clone());
                    return;
                }
                self.declare_symbol_into(
                    clause,
                    SymbolFlags::Alias,
                    SymbolFlags::AliasExcludes,
                    DeclareTarget::Exports(parent_sym),
                );
            }
            _ => {

            }
        }
    }

    fn bind_namespace_export_declaration(&mut self, node: &Arc<Node>) {
        let parent_sym = match self.parent_symbol.clone() {
            Some(s) => s,
            None => return,
        };
        self.declare_symbol_into(
            node,
            SymbolFlags::Alias,
            SymbolFlags::AliasExcludes,
            DeclareTarget::Exports(parent_sym),
        );
    }

    fn bind_container(&mut self, node: &Arc<Node>, flags: ContainerFlags) {

        let prev_container = self.container.clone();
        let prev_block = self.block_scope_container.take();

        let prev_this_container = self.this_container.take();

        let prev_parent_symbol = self.parent_symbol.take();

        let block_only = is_block_only_container(node.kind);
        if flags.contains(ContainerFlags::IS_CONTAINER) && !block_only {
            self.container = Some(Arc::clone(node));
            self.block_scope_container = Some(Arc::clone(node));
        } else {

            self.block_scope_container = Some(Arc::clone(node));
        }

        if flags.contains(ContainerFlags::IS_THIS_CONTAINER) {
            self.this_container = Some(Arc::clone(node));
        }

        if has_locals(node.kind) {
            self.symbol_map.locals.insert(node.id(), SymbolTable::new());

            if node.kind == SyntaxKind::ClassExpression
                && let NodeData::ClassExpression(data) = &node.data
                && let Some(name_node) = data.name.as_ref()
            {
                let name = name_node.text().to_string();
                let sym = self.new_symbol(SymbolFlags::Class, name.clone());
                let sym_mut = Arc::as_ptr(&sym) as *mut Symbol;
                unsafe {
                    (*sym_mut).declarations.push(Arc::clone(node));
                    (*sym_mut).value_declaration = Some(Arc::clone(node));
                }
                self.symbol_map
                    .locals
                    .entry(node.id())
                    .or_insert_with(SymbolTable::new)
                    .insert(name, Arc::clone(&sym));
            }
        }

        if let Some(sym) = self.symbol_map.symbol_of(node) {
            self.parent_symbol = Some(Arc::clone(sym));
        }

        let is_function_like = flags.contains(ContainerFlags::IS_FUNCTION_LIKE);
        let prev_flow = if is_function_like {
            self.current_flow.take()
        } else {
            None
        };
        if is_function_like {
            self.current_flow = Some(Arc::new(FlowNode::new(FlowFlags::START)));
        }

        if node.kind == SyntaxKind::FunctionExpression {
            let sym_and_name = self
                .symbol_map
                .symbol_of(node)
                .map(|sym| (Arc::clone(&sym), sym.name.clone()));
            if let Some((sym, sym_name)) = sym_and_name {
                if sym_name != INTERNAL_SYMBOL_NAME_FUNCTION {
                    if let Some(locals) = self.symbol_map.locals.get_mut(&node.id()) {
                        locals.insert(sym_name, sym);
                    }
                }
            }
        }

        self.bind_children(node);

        if is_function_like {
            self.current_flow = prev_flow;
        }

        self.container = prev_container;
        self.block_scope_container = prev_block;
        self.this_container = prev_this_container;
        self.parent_symbol = prev_parent_symbol;
    }

    pub(crate) fn bind_children(&mut self, node: &Arc<Node>) {

        let this = self as *mut Self;
        crate::ast::node_data_generated::for_each_child(node, |child| {
            unsafe {
                (*this).bind(child);
            }
            false
        });
    }

    pub fn symbol_count(&self) -> usize {
        self.symbol_count
    }

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

            let name = self.get_declaration_name(node);
            self.bind_anonymous_declaration(node, SymbolFlags::TypeParameter, &name);
            return;
        }
        self.declare_symbol(node, SymbolFlags::TypeParameter, SymbolFlags::TYPE);
    }

    fn get_infer_type_container(&self, infer_node: &Arc<Node>) -> Option<Arc<Node>> {
        let mut current = Arc::clone(infer_node);
        loop {
            let parent = match &current.parent {
                Some(p) => Arc::clone(p),
                None => return None,
            };
            if parent.kind == SyntaxKind::ConditionalType {

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
                if (*symbol_mut).value_declaration.is_none() && flags.intersects(SymbolFlags::VALUE)
                {
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
