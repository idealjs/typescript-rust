use super::*;

impl Binder {
    pub(crate) fn new_symbol(&mut self, flags: SymbolFlags, name: impl Into<String>) -> Arc<Symbol> {
        self.symbol_count += 1;
        Arc::new(Symbol::new(flags, name))
    }

    pub(crate) fn declare_symbol(
        &mut self,
        node: &Arc<Node>,
        includes: SymbolFlags,
        _excludes: SymbolFlags,
    ) -> Arc<Symbol> {
        let name = self.get_declaration_name(node);

        let var_hoist_container: Option<Arc<Node>> =
            if Self::declaration_is_var(node) && self.parent_symbol.is_none() {
                self.container
                    .as_ref()
                    .filter(|c| is_var_container_kind(c.kind))
                    .cloned()
            } else {
                None
            };

        let is_module_member_container = self
            .container
            .as_ref()
            .is_some_and(|c| c.kind == SyntaxKind::ModuleDeclaration);

        let module_member_is_exported = |b: &Self, node: &Arc<Node>| -> bool {
            node.kind == SyntaxKind::ExportSpecifier
                || b.get_combined_modifier_flags(node)
                    .contains(ModifierFlags::Export)
        };
        let existing: Option<Arc<Symbol>> = if is_module_member_container
            && let Some(parent_sym) = &self.parent_symbol
        {
            let has_export = module_member_is_exported(self, node);
            let container_id = self.container.as_ref().unwrap().id();
            let locals_hit = || {
                self.symbol_map
                    .locals
                    .get(&container_id)
                    .and_then(|l| l.get(&name).cloned())
            };
            if includes.contains(SymbolFlags::Alias) {

                if has_export {
                    parent_sym.exports.get(&name).cloned()
                } else {
                    locals_hit()
                }
            } else if has_export {

                parent_sym
                    .exports
                    .get(&name)
                    .cloned()
                    .or_else(locals_hit)
            } else {

                locals_hit()
            }
        } else if let Some(parent_sym) = &self.parent_symbol {
            parent_sym
                .members
                .get(&name)
                .cloned()
                .or_else(|| parent_sym.exports.get(&name).cloned())
        } else if let Some(hoist) = &var_hoist_container {
            match hoist.kind {
                SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration => self
                    .symbol_map
                    .symbol_of(hoist)
                    .and_then(|sym| sym.members.get(&name).cloned()),

                _ => {
                    let container_id = hoist.id();
                    self.symbol_map
                        .locals
                        .get(&container_id)
                        .and_then(|locals| locals.get(&name).cloned())
                }
            }
        } else if let Some(block_container) = &self.block_scope_container {
            let container_id = block_container.id();
            self.symbol_map
                .locals
                .get(&container_id)
                .and_then(|locals| locals.get(&name).cloned())
        } else {
            None
        };

        let mut conflicted = false;

        if let Some(existing) = existing {

            let var_var_merge = Self::declaration_is_var(node)
                && existing.flags == SymbolFlags::BlockScopedVariable
                && existing.declarations.iter().all(|d| Self::declaration_is_var(d));

            let ns_var_merge = Self::declaration_is_var(node)
                && existing.flags.contains(SymbolFlags::ValueModule)
                && existing
                    .declarations
                    .iter()
                    .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
                    .all(|ns| !Self::ns_is_instantiated_static(ns));

            let var_ns_merge = node.kind == SyntaxKind::ModuleDeclaration
                && !Self::ns_is_instantiated_static(node)
                && existing.flags == SymbolFlags::BlockScopedVariable;

            let import_export_alias_merge = includes.contains(SymbolFlags::Alias)
                && existing.flags.contains(SymbolFlags::Alias)
                && {
                    let node_is_spec = node.kind == SyntaxKind::ExportSpecifier;
                    let existing_all_spec = existing.declarations.iter().all(|d| {
                        d.kind == SyntaxKind::ExportSpecifier
                    });
                    node_is_spec != existing_all_spec
                };
            if self.can_merge_symbols(existing.flags, includes)
                || var_var_merge
                || ns_var_merge
                || var_ns_merge
                || import_export_alias_merge
            {

                let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                unsafe {
                    (*existing_mut).declarations.push(Arc::clone(node));
                    (*existing_mut).flags |= includes;

                    if (*existing_mut).value_declaration.is_none()
                        && includes.intersects(SymbolFlags::VALUE)
                    {
                        (*existing_mut).value_declaration = Some(Arc::clone(node));
                    }
                }

                let ns_proto_loc: Option<crate::core::text::TextRange> = (|| {
                    let scan = |n: &Arc<Node>| -> Option<crate::core::text::TextRange> {
                        if n.kind != SyntaxKind::ModuleDeclaration {
                            return None;
                        }
                        let NodeData::ModuleDeclaration(md) = &n.data else {
                            return None;
                        };
                        let body = md.body.as_ref()?;
                        let mut hit: Option<crate::core::text::TextRange> = None;
                        crate::ast::node_data_generated::for_each_child(body, |stmt| {
                            if stmt.kind == SyntaxKind::VariableStatement {
                                if let NodeData::VariableStatement(vs) = &stmt.data {
                                    let NodeData::VariableDeclarationList(vdl) =
                                        &vs.declaration_list.data
                                    else {
                                        return false;
                                    };
                                    for decl in vdl.declarations.iter() {
                                        if decl
                                            .name()
                                            .is_some_and(|n| n.text() == "prototype")
                                        {
                                            hit = decl.name().map(|n| n.loc);
                                        }
                                    }
                                }
                            }
                            false
                        });
                        hit
                    };
                    if node.kind == SyntaxKind::ModuleDeclaration
                        && existing
                            .flags
                            .intersects(SymbolFlags::Class | SymbolFlags::Function)
                    {
                        return scan(node);
                    }
                    if matches!(node.kind, SyntaxKind::ClassDeclaration | SyntaxKind::FunctionDeclaration)
                        && existing.flags.contains(SymbolFlags::ValueModule)
                    {
                        for d in &existing.declarations {
                            if let Some(loc) = scan(d) {
                                return Some(loc);
                            }
                        }
                    }
                    None
                })();
                if let Some(loc) = ns_proto_loc {
                    let already = self
                        .symbol_map
                        .binder_diagnostics
                        .iter()
                        .any(|dd| dd.code == 2300 && dd.loc == loc);
                    if !already {
                        self.symbol_map.binder_diagnostics.push(Diagnostic::new(
                            self.current_source_file.clone(),
                            loc,
                            DUPLICATE_IDENTIFIER_0,
                            vec!["prototype".to_string()],
                        ));
                    }
                }

                if node.kind == SyntaxKind::EnumDeclaration
                    && existing.flags.intersects(SymbolFlags::ENUM)
                {
                    let NodeData::EnumDeclaration(new_ed) = &node.data else {
                        unreachable!()
                    };
                    let mut new_names: Vec<(String, crate::core::text::TextRange)> =
                        Vec::new();
                    for m in new_ed.members.iter() {
                        if let Some(n) = m.name() {
                            new_names.push((n.text().to_string(), n.loc));
                        }
                    }
                    for d in &existing.declarations {
                        if d.kind != SyntaxKind::EnumDeclaration || Arc::ptr_eq(d, node) {
                            continue;
                        }
                        let NodeData::EnumDeclaration(ed) = &d.data else {
                            continue;
                        };
                        for m in ed.members.iter() {
                            let Some(n) = m.name() else { continue };
                            if let Some((_, new_loc)) =
                                new_names.iter().find(|(name, _)| *name == n.text())
                            {
                                for loc in [*new_loc, n.loc] {
                                    let already = self
                                        .symbol_map
                                        .binder_diagnostics
                                        .iter()
                                        .any(|dd| dd.code == 2300 && dd.loc == loc);
                                    if !already {
                                        self.symbol_map.binder_diagnostics.push(
                                            Diagnostic::new(
                                                self.current_source_file.clone(),
                                                loc,
                                                DUPLICATE_IDENTIFIER_0,
                                                vec![n.text().to_string()],
                                            ),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
self.symbol_map.set_symbol(node, Arc::clone(&existing));

                if let Some(container) = &self.container {
                    let is_module_container = container.kind == SyntaxKind::SourceFile
                        || container.kind == SyntaxKind::ModuleDeclaration;
                    if is_module_container
                        && self
                            .get_combined_modifier_flags(node)
                            .contains(ModifierFlags::Export)
                    {
                        let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                        unsafe {
                            (*existing_mut).export_symbol = Some(Arc::clone(&existing));
                        }
                    }
                }
                return existing;
            }

            let both_block_scoped_var = existing.flags.contains(SymbolFlags::BlockScopedVariable)
                && includes.contains(SymbolFlags::BlockScopedVariable);
            if !name.is_empty() {

                let report_all = |b: &mut Self, message: &'static crate::diagnostics::Message| {

                    let push = |b: &mut Self, loc: crate::core::text::TextRange| {
                        if b
                            .symbol_map
                            .binder_diagnostics
                            .iter()
                            .any(|d| d.loc == loc && d.code == message.code)
                        {
                            return;
                        }
                        b.symbol_map.binder_diagnostics.push(Diagnostic::new(
                            b.current_source_file.clone(),
                            loc,
                            *message,
                            vec![name.clone()],
                        ));
                    };
                    for d in &existing.declarations {
                        let name_node = crate::ast::utilities::get_name_of_declaration(d)
                            .unwrap_or_else(|| Arc::clone(d));
                        push(b, name_node.loc);
                    }
                    let name_node = crate::ast::utilities::get_name_of_declaration(node)
                        .unwrap_or_else(|| Arc::clone(node));
                    push(b, name_node.loc);
                };
                if both_block_scoped_var {
                    if Self::is_let_or_const_declaration(node) {
                        report_all(
                            self,
                            &CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0,
                        );

                        conflicted = true;
                    }

                } else {

                    let member_flags = SymbolFlags::Property
                        .union(SymbolFlags::Method)
                        .union(SymbolFlags::GetAccessor)
                        .union(SymbolFlags::SetAccessor)
                        .union(SymbolFlags::EnumMember)
                        .union(SymbolFlags::FunctionScopedVariable)
                        .union(SymbolFlags::TypeParameter)
                        .union(SymbolFlags::Constructor)
                        .union(SymbolFlags::Signature);

                    let involves_namespace_export = node.kind
                        == SyntaxKind::NamespaceExportDeclaration
                        || existing
                            .declarations
                            .iter()
                            .any(|d| d.kind == SyntaxKind::NamespaceExportDeclaration);
                    if involves_namespace_export
                        || existing.flags.intersects(member_flags)
                        || includes.intersects(member_flags)
                    {

                    } else if existing.flags.intersects(SymbolFlags::ENUM)
                        != includes.intersects(SymbolFlags::ENUM)
                        && (existing.flags
                            .intersects(SymbolFlags::ENUM | SymbolFlags::Class)
                            || includes
                                .intersects(SymbolFlags::ENUM | SymbolFlags::Class))
                    {

                        report_all(
                            self,
                            &crate::diagnostics::messages_generated::
                                ENUM_DECLARATIONS_CAN_ONLY_MERGE_WITH_NAMESPACE_OR_OTHER_ENUM_DECLARATIONS,
                        );
                        conflicted = true;
                    } else {

                        report_all(self, &DUPLICATE_IDENTIFIER_0);
                        conflicted = true;
                    }
                }
            }
        }

        let symbol = self.new_symbol(includes, name.clone());

        {
            let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
            unsafe {
                (*symbol_mut).declarations.push(Arc::clone(node));

                if (*symbol_mut).value_declaration.is_none()
                    && includes.intersects(SymbolFlags::VALUE)
                {
                    (*symbol_mut).value_declaration = Some(Arc::clone(node));
                }
            }
        }

        if !conflicted && let Some(container) = &self.container {
            if container.kind == SyntaxKind::ModuleDeclaration {

                let has_export = module_member_is_exported(self, node);

                let alias_no_local = has_export
                    && matches!(
                        node.kind,
                        SyntaxKind::ExportSpecifier | SyntaxKind::ImportEqualsDeclaration
                    );
                if has_export {
                    if let Some(parent_sym) = &self.parent_symbol {
                        let parent_sym_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                        unsafe {
                            (*parent_sym_mut)
                                .exports
                                .insert(name.clone(), Arc::clone(&symbol));
                        }
                    }

                    if has_locals(container.kind) && !alias_no_local {
                        let locals = self
                            .symbol_map
                            .locals
                            .entry(container.id())
                            .or_insert_with(SymbolTable::new);
                        locals.insert(name.clone(), Arc::clone(&symbol));
                    }
                } else if has_locals(container.kind) {
                    let locals = self
                        .symbol_map
                        .locals
                        .entry(container.id())
                        .or_insert_with(SymbolTable::new);
                    locals.insert(name.clone(), Arc::clone(&symbol));
                }
            } else if let Some(parent_sym) = &self.parent_symbol {

                let parent_sym_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                unsafe {
                    (*parent_sym_mut)
                        .members
                        .insert(name.clone(), Arc::clone(&symbol));
                }
            } else if let Some(hoist) = &var_hoist_container {

                match hoist.kind {
                    SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration => {
                        if let Some(sym) = self.symbol_map.symbol_of(hoist) {
                            let sym_mut = Arc::as_ptr(&sym) as *mut Symbol;
                            unsafe {
                                (*sym_mut)
                                    .members
                                    .insert(name.clone(), Arc::clone(&symbol));
                            }
                        }
                    }
                    _ => {
                        let locals = self
                            .symbol_map
                            .locals
                            .entry(hoist.id())
                            .or_insert_with(SymbolTable::new);
                        locals.insert(name.clone(), Arc::clone(&symbol));
                    }
                }
            } else if let Some(block_container) = &self.block_scope_container {

                let container_id = block_container.id();
                let locals = self
                    .symbol_map
                    .locals
                    .entry(container_id)
                    .or_insert_with(SymbolTable::new);
                locals.insert(name.clone(), Arc::clone(&symbol));
            }
        }

        if let Some(container) = &self.container {
            let is_module_container = container.kind == SyntaxKind::SourceFile
                || container.kind == SyntaxKind::ModuleDeclaration;
            if is_module_container
                && self
                    .get_combined_modifier_flags(node)
                    .contains(ModifierFlags::Export)
            {
                let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                unsafe {
                    (*symbol_mut).export_symbol = Some(Arc::clone(&symbol));
                }
            }
        }

        self.symbol_map.set_symbol(node, Arc::clone(&symbol));

        symbol
    }

    pub(crate) fn declare_symbol_into(
        &mut self,
        node: &Arc<Node>,
        includes: SymbolFlags,
        _excludes: SymbolFlags,
        target: DeclareTarget,
    ) -> Arc<Symbol> {
        let name = self.get_declaration_name(node);

        let existing: Option<Arc<Symbol>> = match &target {
            DeclareTarget::Exports(parent_sym) => parent_sym.exports.get(&name).cloned(),
            DeclareTarget::Locals(container) => self
                .symbol_map
                .locals
                .get(&container.id())
                .and_then(|locals| locals.get(&name).cloned()),
        };

        if let Some(existing) = existing {
            if self.can_merge_symbols(existing.flags, includes) {
                let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                unsafe {
                    (*existing_mut).declarations.push(Arc::clone(node));
                    (*existing_mut).flags |= includes;
                    if (*existing_mut).value_declaration.is_none()
                        && includes.intersects(SymbolFlags::VALUE)
                    {
                        (*existing_mut).value_declaration = Some(Arc::clone(node));
                    }
                }
                self.symbol_map.set_symbol(node, Arc::clone(&existing));
                return existing;
            }

        }

        let symbol = self.new_symbol(includes, name.clone());
        {
            let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
            unsafe {
                (*symbol_mut).declarations.push(Arc::clone(node));
                if (*symbol_mut).value_declaration.is_none()
                    && includes.intersects(SymbolFlags::VALUE)
                {
                    (*symbol_mut).value_declaration = Some(Arc::clone(node));
                }
            }
        }

        match &target {
            DeclareTarget::Exports(parent_sym) => {
                let parent_mut = Arc::as_ptr(parent_sym) as *mut Symbol;
                unsafe {
                    (*parent_mut)
                        .exports
                        .insert(name.clone(), Arc::clone(&symbol));

                    let symbol_mut = Arc::as_ptr(&symbol) as *mut Symbol;
                    (*symbol_mut).parent = Some(Arc::clone(parent_sym));
                }
            }
            DeclareTarget::Locals(container) => {
                let locals = self
                    .symbol_map
                    .locals
                    .entry(container.id())
                    .or_insert_with(SymbolTable::new);
                locals.insert(name.clone(), Arc::clone(&symbol));
            }
        }

        self.symbol_map.set_symbol(node, Arc::clone(&symbol));
        symbol
    }

    fn ns_is_instantiated_static(ns: &Arc<Node>) -> bool {
        let NodeData::ModuleDeclaration(md) = &ns.data else {
            return false;
        };
        let Some(body) = &md.body else {
            return false;
        };
        let mut found = false;
        crate::ast::node_data_generated::for_each_child(body, |stmt| {
            match stmt.kind {
                SyntaxKind::InterfaceDeclaration
                | SyntaxKind::TypeAliasDeclaration
                | SyntaxKind::ImportDeclaration
                | SyntaxKind::ImportEqualsDeclaration
                | SyntaxKind::ExportDeclaration => {}
                _ => found = true,
            }
            false
        });
        found
    }
    pub(crate) fn can_merge_symbols(&self, existing_flags: SymbolFlags, new_flags: SymbolFlags) -> bool {

        let existing_alias = existing_flags.contains(SymbolFlags::Alias);
        let new_alias = new_flags.contains(SymbolFlags::Alias);
        if existing_alias || new_alias {
            return !(existing_alias && new_alias);
        }

        if existing_flags.contains(SymbolFlags::Interface)
            && new_flags.contains(SymbolFlags::Interface)
        {
            return true;
        }

        let existing_interface = existing_flags.contains(SymbolFlags::Interface);
        let new_interface = new_flags.contains(SymbolFlags::Interface);
        let existing_type_alias = existing_flags.contains(SymbolFlags::TypeAlias);
        let new_type_alias = new_flags.contains(SymbolFlags::TypeAlias);
        let class_side = SymbolFlags::Class;

        let enum_side =
            SymbolFlags::ENUM;
        if (existing_flags.intersects(enum_side) && new_interface)
            || (new_flags.intersects(enum_side) && existing_interface)
        {
            return false;
        }
        if (existing_interface && !new_interface && !new_type_alias)
            || (new_interface && !existing_interface && !existing_type_alias)
            || (existing_type_alias && !new_type_alias && !new_flags.intersects(class_side) && !new_interface)
            || (new_type_alias && !existing_type_alias && !existing_flags.intersects(class_side) && !existing_interface)
        {
            return true;
        }

        let existing_class = existing_flags.contains(SymbolFlags::Class);
        let new_class = new_flags.contains(SymbolFlags::Class);
        let existing_fn = existing_flags.contains(SymbolFlags::Function);
        let new_fn = new_flags.contains(SymbolFlags::Function);
        if (existing_class && new_fn) || (existing_fn && new_class) {
            return true;
        }

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

            let can_merge_with_ns = other_existing.contains(SymbolFlags::ValueModule)
                || other_existing.contains(SymbolFlags::Function)
                || other_existing.contains(SymbolFlags::Class)
                || other_existing.contains(SymbolFlags::RegularEnum)
                || other_existing.contains(SymbolFlags::ConstEnum)
                || other_existing.contains(SymbolFlags::Interface);
            if can_merge_with_ns {
                return true;
            }
        }

        if existing_flags.contains(SymbolFlags::Function)
            && new_flags.contains(SymbolFlags::Function)
        {
            return true;
        }

        if (existing_flags.contains(SymbolFlags::RegularEnum)
            || existing_flags.contains(SymbolFlags::ConstEnum))
            && (new_flags.contains(SymbolFlags::RegularEnum)
                || new_flags.contains(SymbolFlags::ConstEnum))
        {
            return true;
        }

        let type_param_existing =
            existing_flags.contains(SymbolFlags::TypeParameter);
        let type_param_new = new_flags.contains(SymbolFlags::TypeParameter);
        if (type_param_existing && !new_flags.intersects(SymbolFlags::TYPE))
            || (type_param_new && !existing_flags.intersects(SymbolFlags::TYPE))
        {
            return true;
        }
        false
    }

    fn is_let_or_const_declaration(node: &Arc<Node>) -> bool {
        if node.kind == SyntaxKind::VariableDeclaration {
            if let Some(parent) = node.parent.as_ref() {
                if parent.kind == SyntaxKind::VariableDeclarationList {
                    return parent.flags.intersects(NodeFlags::Let | NodeFlags::Const);
                }
            }
        }
        true
    }

    pub(crate) fn has_export_declarations(container: &Arc<Node>) -> bool {
        let statements: &[Arc<Node>] = match &container.data {
            crate::ast::NodeData::SourceFile(sf) => &sf.statements.nodes,
            crate::ast::NodeData::ModuleDeclaration(md) => {
                if let Some(body) = &md.body
                    && body.kind == SyntaxKind::ModuleBlock
                    && let crate::ast::NodeData::ModuleBlock(block) = &body.data
                {
                    &block.statements.nodes
                } else {
                    &[]
                }
            }
            _ => &[],
        };
        statements.iter().any(|s| {
            s.kind == SyntaxKind::ExportDeclaration || s.kind == SyntaxKind::ExportAssignment
        })
    }

    #[allow(dead_code)]
    fn is_var_declaration(node: &Arc<Node>) -> bool {
        if node.kind == SyntaxKind::VariableDeclaration {
            if let Some(parent) = node.parent.as_ref() {
                if parent.kind == SyntaxKind::VariableDeclarationList {
                    return !parent.flags.intersects(NodeFlags::Let | NodeFlags::Const);
                }
            }
        }
        false
    }

    fn declaration_is_var(node: &Arc<Node>) -> bool {
        let mut current = node;
        loop {
            match current.kind {
                SyntaxKind::VariableDeclaration => {
                    return if let Some(parent) = current.parent.as_ref() {
                        parent.kind == SyntaxKind::VariableDeclarationList
                            && !parent.flags.intersects(NodeFlags::Let | NodeFlags::Const)
                    } else {
                        false
                    };
                }
                SyntaxKind::BindingElement
                | SyntaxKind::ObjectBindingPattern
                | SyntaxKind::ArrayBindingPattern => {
                    match current.parent.as_ref() {
                        Some(parent) => current = parent,
                        None => return false,
                    }
                }
                _ => return false,
            }
        }
    }

    #[allow(dead_code)]
    fn symbol_is_var_declaration(symbol: &Arc<Symbol>) -> bool {
        let decl: Option<&Arc<Node>> = symbol
            .value_declaration
            .as_ref()
            .or_else(|| symbol.declarations.first());
        match decl {
            Some(node) => Self::is_var_declaration(node),
            None => false,
        }
    }

    fn get_combined_modifier_flags(&self, node: &Arc<Node>) -> ModifierFlags {
        let mut flags = node.syntactic_modifier_flags();
        if node.kind == SyntaxKind::VariableDeclaration {
            if let Some(parent) = &node.parent {
                if parent.kind == SyntaxKind::VariableDeclarationList {
                    flags |= parent.syntactic_modifier_flags();
                    if let Some(gp) = &parent.parent {
                        if gp.kind == SyntaxKind::VariableStatement {
                            flags |= gp.syntactic_modifier_flags();
                        }
                    }
                }
            }
        }
        flags
    }

    pub(crate) fn get_declaration_name(&self, node: &Arc<Node>) -> String {
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

            NodeData::ImportSpecifier(data) => self.node_text(&data.name),
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

            NodeData::ImportEqualsDeclaration(data) => self.node_text(&data.name),
            NodeData::NamespaceImport(data) => self.node_text(&data.name),

            NodeData::ExportSpecifier(data) => self.node_text(&data.name),
            NodeData::Identifier(data) => data.text.clone(),

            NodeData::ExportAssignment(data) => {
                if data.is_export_equals {
                    INTERNAL_SYMBOL_NAME_EXPORT_EQUALS.to_string()
                } else {
                    INTERNAL_SYMBOL_NAME_DEFAULT.to_string()
                }
            }

            NodeData::ExportDeclaration(_) => INTERNAL_SYMBOL_NAME_EXPORT_STAR.to_string(),

            NodeData::NamespaceExport(data) => self.node_text(&data.name),
            NodeData::NamespaceExportDeclaration(data) => self.node_text(&data.name),
            _ => String::new(),
        }
    }

    pub(crate) fn node_text(&self, node: &Arc<Node>) -> String {
        match &node.data {
            NodeData::Identifier(data) => data.text.clone(),

            NodeData::PrivateIdentifier(data) => data.text.clone(),
            NodeData::StringLiteral(data) => data.text.clone(),
            NodeData::NumericLiteral(data) => data.text.clone(),
            NodeData::NoSubstitutionTemplateLiteral(data) => data.text.clone(),
            NodeData::BigIntLiteral(data) => data.text.clone(),
            _ => String::new(),
        }
    }
}
