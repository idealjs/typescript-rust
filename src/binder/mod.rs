pub mod nameresolver;
pub mod referenceresolver;

use crate::ast::*;
use crate::diagnostics::messages_generated::{
    A_PARAMETER_INITIALIZER_IS_ONLY_ALLOWED_IN_A_FUNCTION_OR_CONSTRUCTOR_IMPLEMENTATION,
    CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0, DUPLICATE_IDENTIFIER_0,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_AT_THE_TOP_LEVEL_OF_A_MODULE,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_CLASS_DEFINITIONS_ARE_AUTOMATICALLY_IN_STRICT_MODE,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE,
    IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE,
};
use std::sync::Arc;

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

    fn add_antecedent(&mut self, antecedent: Arc<FlowNode>) {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return;
        }

        for ant in &self.node.antecedents {
            if Arc::ptr_eq(ant, &antecedent) {
                return;
            }
        }
        self.node.antecedents.push(antecedent);
    }

    fn finish_multi(&self, unreachable: &Arc<FlowNode>) -> Arc<FlowNode> {
        if self.node.antecedents.is_empty() {
            return Arc::clone(unreachable);
        }
        Arc::new(FlowNode {
            flags: self.node.flags,
            node: None,
            antecedent: None,
            antecedents: self.node.antecedents.clone(),
            switch_statement: None,
            clause_range: None,
            reduce_target: None,
        })
    }

    fn push_antecedent(node: &Arc<FlowNode>, ant: Arc<FlowNode>) {
        if ant.flags.contains(FlowFlags::UNREACHABLE) {
            return;
        }
        let ptr = Arc::as_ptr(node) as *mut FlowNode;
        unsafe {
            for existing in &(*ptr).antecedents {
                if Arc::ptr_eq(existing, &ant) {
                    return;
                }
            }
            (*ptr).antecedents.push(ant);
        }
    }

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
            clause_range: None,
            reduce_target: None,
        })
    }
}

#[derive(Debug)]
struct ActiveLabel {
    name: String,
    break_target: Arc<FlowNode>,
    continue_target: Option<Arc<FlowNode>>,
    referenced: bool,
    next: Option<Box<ActiveLabel>>,
}

pub struct Binder {

    pub symbol_map: NodeSymbolMap,

    current_source_file: Option<Arc<SourceFile>>,

    container: Option<Arc<Node>>,

    block_scope_container: Option<Arc<Node>>,

    this_container: Option<Arc<Node>>,

    parent_symbol: Option<Arc<Symbol>>,

    current_flow: Option<Arc<FlowNode>>,

    symbol_count: usize,

    expando_assignments: Vec<(Arc<Node>, Option<Arc<Node>>)>,

    unreachable_flow: Option<Arc<FlowNode>>,

    current_break_target: Option<Arc<FlowNode>>,

    current_continue_target: Option<Arc<FlowNode>>,

    current_exception_target: Option<Arc<FlowNode>>,

    current_return_target: Option<Arc<FlowNode>>,

    active_label_list: Option<Box<ActiveLabel>>,

    has_explicit_return: bool,

    has_flow_effects: bool,
}

impl Default for Binder {
    fn default() -> Self {
        Self::new()
    }
}

enum DeclareTarget {

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

    fn new_symbol(&mut self, flags: SymbolFlags, name: impl Into<String>) -> Arc<Symbol> {
        self.symbol_count += 1;
        Arc::new(Symbol::new(flags, name))
    }

    fn declare_symbol(
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

    fn declare_symbol_into(
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
    fn can_merge_symbols(&self, existing_flags: SymbolFlags, new_flags: SymbolFlags) -> bool {

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

    fn node_text(&self, node: &Arc<Node>) -> String {
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

    fn unreachable_flow(&self) -> Arc<FlowNode> {
        Arc::clone(self.unreachable_flow.as_ref().unwrap())
    }

    #[allow(dead_code)]
    fn new_flow_node(&self, flags: FlowFlags) -> FlowNode {
        FlowNode::new(flags)
    }

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
            clause_range: None,
            reduce_target: None,
        })
    }

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
            clause_range: None,
            reduce_target: None,
        })
    }

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
            clause_range: None,
            reduce_target: None,
        })
    }

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
            clause_range: None,
            reduce_target: None,
        });

        if let Some(target) = &self.current_exception_target {
            self.add_antecedent_to_flow(target, &result);
        }
        result
    }

    fn set_flow_node_referenced(&self, flow: &FlowNode) {

        let ptr = flow as *const FlowNode as *mut FlowNode;
        unsafe {
            if (*ptr).flags.contains(FlowFlags::REFERENCED) {
                (*ptr).flags = (*ptr).flags | FlowFlags::SHARED;
            } else {
                (*ptr).flags = (*ptr).flags | FlowFlags::REFERENCED;
            }
        }
    }

    fn create_reduce_label(
        &self,
        target: &Arc<FlowNode>,
        antecedents: &[Arc<FlowNode>],
        antecedent: &Arc<FlowNode>,
    ) -> Arc<FlowNode> {
        Arc::new(FlowNode {
            flags: FlowFlags::REDUCE_LABEL,
            node: None,
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: antecedents.to_vec(),
            switch_statement: None,
            clause_range: None,
            reduce_target: Some(Arc::clone(target)),
        })
    }

    fn new_flow_accumulator() -> Arc<FlowNode> {
        Arc::new(FlowNode {
            flags: FlowFlags::BRANCH_LABEL,
            node: None,
            antecedent: None,
            antecedents: Vec::new(),
            switch_statement: None,
            clause_range: None,
            reduce_target: None,
        })
    }

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

        self.set_flow_node_referenced(antecedent);
    }

    fn create_flow_switch_clause(
        &mut self,
        antecedent: &Arc<FlowNode>,
        clause: Option<&Arc<Node>>,
        switch_statement: &Arc<Node>,
        clause_start: usize,
        clause_end: usize,
    ) -> Arc<FlowNode> {
        if antecedent.flags.contains(FlowFlags::UNREACHABLE) {
            return Arc::clone(antecedent);
        }
        Arc::new(FlowNode {
            flags: FlowFlags::SWITCH_CLAUSE,
            node: clause.map(Arc::clone),
            antecedent: Some(Arc::clone(antecedent)),
            antecedents: Vec::new(),
            switch_statement: Some(Arc::clone(switch_statement)),
            clause_range: Some((clause_start, clause_end)),
            reduce_target: None,
        })
    }

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

        self.bind(&expr);
        if let Some(current) = self.current_flow.take() {
            let true_flow = self.create_flow_condition(FlowFlags::TRUE_CONDITION, &current, &expr);
            let false_flow =
                self.create_flow_condition(FlowFlags::FALSE_CONDITION, &current, &expr);
            then_label.add_antecedent(true_flow);
            else_label.add_antecedent(false_flow);
        }

        self.current_flow = Some(then_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.bind(&then_stmt);
        if let Some(current) = &self.current_flow {
            post_if_label.add_antecedent(Arc::clone(current));
        }

        self.current_flow = Some(else_label.finish(self.unreachable_flow.as_ref().unwrap()));
        if let Some(else_s) = else_stmt {
            self.bind(&else_s);
        }
        if let Some(current) = &self.current_flow {
            post_if_label.add_antecedent(Arc::clone(current));
        }

        self.current_flow = Some(post_if_label.finish(self.unreachable_flow.as_ref().unwrap()));
    }

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

        let loop_head = pre_while_label.finish_multi(self.unreachable_flow.as_ref().unwrap());
        self.current_flow = Some(Arc::clone(&loop_head));

        self.bind(&expr);
        if let Some(current) = self.current_flow.take() {
            let true_flow = self.create_flow_condition(FlowFlags::TRUE_CONDITION, &current, &expr);
            let false_flow =
                self.create_flow_condition(FlowFlags::FALSE_CONDITION, &current, &expr);
            pre_body_label.add_antecedent(true_flow);
            post_while_label.add_antecedent(false_flow);
        }

        let prev_break = self.current_break_target.take();
        let prev_continue = self.current_continue_target.take();
        let break_acc = Self::new_flow_accumulator();
        let continue_acc = Self::new_flow_accumulator();
        self.current_break_target = Some(Arc::clone(&break_acc));
        self.current_continue_target = Some(Arc::clone(&continue_acc));

        self.set_continue_target(node, &continue_acc);

        self.current_flow = Some(pre_body_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.bind(&stmt);
        if let Some(current) = &self.current_flow {
            FlowLabel::push_antecedent(&loop_head, Arc::clone(current));
        }

        for ant in &continue_acc.antecedents {
            FlowLabel::push_antecedent(&loop_head, Arc::clone(ant));
        }

        for ant in &break_acc.antecedents {
            post_while_label.add_antecedent(Arc::clone(ant));
        }

        self.current_break_target = prev_break;
        self.current_continue_target = prev_continue;

        self.current_flow = Some(post_while_label.finish(self.unreachable_flow.as_ref().unwrap()));
    }

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

        let prev_break = self.current_break_target.take();
        let prev_continue = self.current_continue_target.take();
        let break_acc = Self::new_flow_accumulator();
        let continue_acc = Self::new_flow_accumulator();
        self.current_break_target = Some(Arc::clone(&break_acc));
        self.current_continue_target = Some(Arc::clone(&continue_acc));

        self.set_continue_target(node, &continue_acc);

        self.bind(&stmt);
        if let Some(current) = &self.current_flow {
            pre_condition_label.add_antecedent(Arc::clone(current));
        }

        for ant in &continue_acc.antecedents {
            pre_condition_label.add_antecedent(Arc::clone(ant));
        }

        self.current_break_target = prev_break;
        self.current_continue_target = prev_continue;

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

        for ant in &break_acc.antecedents {
            post_do_label.add_antecedent(Arc::clone(ant));
        }

        self.current_flow = Some(post_do_label.finish(self.unreachable_flow.as_ref().unwrap()));
    }

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

        let prev_block = self.block_scope_container.take();
        let prev_parent = self.parent_symbol.take();
        self.block_scope_container = Some(Arc::clone(node));
        self.symbol_map
            .locals
            .entry(node.id())
            .or_insert_with(SymbolTable::new);

        if let Some(init) = initializer {
            self.bind(&init);
        }

        if let Some(current) = &self.current_flow {
            pre_loop_label.add_antecedent(Arc::clone(current));
        }
        self.current_flow = Some(pre_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));

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

            if let Some(current) = &self.current_flow {
                pre_body_label.add_antecedent(Arc::clone(current));
            }
        }

        let prev_break = self.current_break_target.take();
        let prev_continue = self.current_continue_target.take();
        let break_acc = Self::new_flow_accumulator();
        let continue_acc = Self::new_flow_accumulator();
        self.current_break_target = Some(Arc::clone(&break_acc));
        self.current_continue_target = Some(Arc::clone(&continue_acc));

        self.set_continue_target(node, &continue_acc);

        self.current_flow = Some(pre_body_label.finish(self.unreachable_flow.as_ref().unwrap()));
        self.bind(&statement);
        if let Some(current) = &self.current_flow {
            pre_incr_label.add_antecedent(Arc::clone(current));
        }

        for ant in &continue_acc.antecedents {
            pre_incr_label.add_antecedent(Arc::clone(ant));
        }

        self.current_break_target = prev_break;
        self.current_continue_target = prev_continue;

        self.current_flow = Some(pre_incr_label.finish(self.unreachable_flow.as_ref().unwrap()));
        if let Some(inc) = incrementor {
            self.bind(&inc);
        }
        if let Some(current) = &self.current_flow {
            pre_loop_label.add_antecedent(Arc::clone(current));
        }

        for ant in &break_acc.antecedents {
            post_loop_label.add_antecedent(Arc::clone(ant));
        }

        self.current_flow = Some(post_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));

        self.block_scope_container = prev_block;
        self.parent_symbol = prev_parent;
    }

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

        let prev_block = self.block_scope_container.take();
        let prev_parent = self.parent_symbol.take();
        self.block_scope_container = Some(Arc::clone(node));
        self.symbol_map
            .locals
            .entry(node.id())
            .or_insert_with(SymbolTable::new);

        self.bind(&expression);

        if let Some(current) = &self.current_flow {
            pre_loop_label.add_antecedent(Arc::clone(current));
        }
        self.current_flow = Some(pre_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));

        post_loop_label.add_antecedent(Arc::clone(self.current_flow.as_ref().unwrap()));

        self.bind(&initializer);

        if initializer.kind != SyntaxKind::VariableDeclarationList {
            self.bind_assignment_target_flow(&initializer);
        }

        let prev_break = self.current_break_target.take();
        let prev_continue = self.current_continue_target.take();
        let break_acc = Self::new_flow_accumulator();
        let continue_acc = Self::new_flow_accumulator();
        self.current_break_target = Some(Arc::clone(&break_acc));
        self.current_continue_target = Some(Arc::clone(&continue_acc));

        self.set_continue_target(node, &continue_acc);

        self.bind(&statement);
        if let Some(current) = &self.current_flow {
            pre_loop_label.add_antecedent(Arc::clone(current));
        }

        for ant in &continue_acc.antecedents {
            pre_loop_label.add_antecedent(Arc::clone(ant));
        }

        for ant in &break_acc.antecedents {
            post_loop_label.add_antecedent(Arc::clone(ant));
        }

        self.current_break_target = prev_break;
        self.current_continue_target = prev_continue;

        self.current_flow = Some(post_loop_label.finish(self.unreachable_flow.as_ref().unwrap()));

        self.block_scope_container = prev_block;
        self.parent_symbol = prev_parent;
    }

    fn bind_switch_statement(&mut self, node: &Arc<Node>) {
        let mut post_switch_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);

        let (expression, case_block) = match &node.data {
            NodeData::SwitchStatement(data) => (data.expression.clone(), data.case_block.clone()),
            _ => return,
        };

        self.bind(&expression);

        let prev_break = self.current_break_target.take();
        let break_acc = Self::new_flow_accumulator();
        self.current_break_target = Some(Arc::clone(&break_acc));

        let clauses = match &case_block.data {
            NodeData::CaseBlock(data) => data.clauses.clone(),
            _ => {
                self.current_break_target = prev_break;
                return;
            }
        };

        let prev_block = self.block_scope_container.take();
        let prev_parent = self.parent_symbol.take();
        self.block_scope_container = Some(Arc::clone(&case_block));
        self.symbol_map
            .locals
            .entry(case_block.id())
            .or_insert_with(SymbolTable::new);

        let entry_flow = self.current_flow.clone();
        let is_narrowing_switch = expression.kind == SyntaxKind::TrueKeyword
            || self.is_narrowing_expression(&expression);
        let mut fallthrough_flow: Option<Arc<FlowNode>> = None;
        let mut has_default = false;
        let clause_nodes = &clauses.nodes;
        let mut i = 0;
        while i < clause_nodes.len() {
            let clause_start = i;

            while clause_statements_empty(&clause_nodes[i]) && i + 1 < clause_nodes.len() {
                self.bind_case_clause(&clause_nodes[i], &entry_flow);
                i += 1;
            }
            let mut pre_case_label = FlowLabel::new(FlowFlags::BRANCH_LABEL);
            let pre_case_flow = if is_narrowing_switch {
                entry_flow.as_ref().map(|entry| {
                    self.create_flow_switch_clause(
                        entry,
                        Some(&clause_nodes[i]),
                        node,
                        clause_start,
                        i + 1,
                    )
                })
            } else {
                entry_flow.clone()
            };
            if let Some(f) = &pre_case_flow {
                pre_case_label.add_antecedent(Arc::clone(f));
            }
            if let Some(f) = &fallthrough_flow {
                pre_case_label.add_antecedent(Arc::clone(f));
            }
            self.current_flow =
                Some(pre_case_label.finish(self.unreachable_flow.as_ref().unwrap()));
            let clause = &clause_nodes[i];
            if clause.kind == SyntaxKind::DefaultClause {
                has_default = true;
            }
            self.bind_case_clause(clause, &entry_flow);
            fallthrough_flow = self.current_flow.clone();
            i += 1;
        }

        if let Some(current) = &self.current_flow {
            post_switch_label.add_antecedent(Arc::clone(current));
        }

        for ant in &break_acc.antecedents {
            post_switch_label.add_antecedent(Arc::clone(ant));
        }

        if !has_default {
            if let Some(entry) = &entry_flow {
                let bypass = self.create_flow_switch_clause(entry, None, node, 0, 0);
                post_switch_label.add_antecedent(bypass);
            }
        }

        self.current_flow = Some(post_switch_label.finish(self.unreachable_flow.as_ref().unwrap()));

        self.block_scope_container = prev_block;
        self.parent_symbol = prev_parent;

        self.current_break_target = prev_break;
    }

    fn bind_case_clause(&mut self, clause: &Arc<Node>, entry_flow: &Option<Arc<FlowNode>>) {
        let NodeData::CaseOrDefaultClause(data) = &clause.data else {
            return;
        };
        if clause.kind == SyntaxKind::CaseClause {
            let saved = self.current_flow.take();
            self.current_flow = entry_flow.clone();
            self.bind(&data.expression);
            self.current_flow = saved;
        }
        for stmt in &data.statements.nodes {
            self.bind(stmt);
        }
    }

    fn is_narrowing_expression(&self, expr: &Arc<Node>) -> bool {
        match expr.kind {
            SyntaxKind::Identifier | SyntaxKind::ThisKeyword => true,
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
                self.contains_narrowable_reference(expr)
            }
            SyntaxKind::CallExpression => self.has_narrowable_argument(expr),
            SyntaxKind::ParenthesizedExpression
            | SyntaxKind::NonNullExpression
            | SyntaxKind::TypeOfExpression => expr
                .expression()
                .map(|inner| self.is_narrowing_expression(inner))
                .unwrap_or(false),
            SyntaxKind::BinaryExpression => {
                let NodeData::BinaryExpression(bin) = &expr.data else {
                    return false;
                };
                self.is_narrowing_binary_expression(&bin.left, &bin.operator_token, &bin.right)
            }
            SyntaxKind::PrefixUnaryExpression => {
                let NodeData::PrefixUnaryExpression(un) = &expr.data else {
                    return false;
                };
                un.operator == SyntaxKind::ExclamationToken
                    && self.is_narrowing_expression(&un.operand)
            }
            _ => false,
        }
    }

    fn is_narrowing_binary_expression(
        &self,
        left: &Arc<Node>,
        operator: &Arc<Node>,
        right: &Arc<Node>,
    ) -> bool {
        match operator.kind {
            SyntaxKind::EqualsToken
            | SyntaxKind::BarBarEqualsToken
            | SyntaxKind::AmpersandAmpersandEqualsToken
            | SyntaxKind::QuestionQuestionEqualsToken => self.contains_narrowable_reference(left),
            SyntaxKind::EqualsEqualsToken
            | SyntaxKind::ExclamationEqualsToken
            | SyntaxKind::EqualsEqualsEqualsToken
            | SyntaxKind::ExclamationEqualsEqualsToken => {
                self.is_narrowable_operand(left)
                    || self.is_narrowable_operand(right)
                    || self.is_narrowing_typeof_operands(right, left)
                    || self.is_narrowing_typeof_operands(left, right)
                    || (Self::is_boolean_literal(right) && self.is_narrowing_expression(left))
                    || (Self::is_boolean_literal(left) && self.is_narrowing_expression(right))
            }
            SyntaxKind::InstanceOfKeyword => self.is_narrowable_operand(left),
            SyntaxKind::InKeyword => self.is_narrowing_expression(right),
            SyntaxKind::CommaToken => self.is_narrowing_expression(right),
            _ => false,
        }
    }

    fn is_boolean_literal(node: &Arc<Node>) -> bool {
        matches!(node.kind, SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword)
    }

    fn is_narrowable_operand(&self, expr: &Arc<Node>) -> bool {
        match expr.kind {
            SyntaxKind::ParenthesizedExpression => {
                expr.expression().map(|e| self.is_narrowable_operand(e)).unwrap_or(false)
            }
            SyntaxKind::BinaryExpression => {
                let NodeData::BinaryExpression(bin) = &expr.data else {
                    return false;
                };
                match bin.operator_token.kind {
                    SyntaxKind::EqualsToken => self.is_narrowable_operand(&bin.left),
                    SyntaxKind::CommaToken => self.is_narrowable_operand(&bin.right),
                    _ => self.contains_narrowable_reference(expr),
                }
            }
            _ => self.contains_narrowable_reference(expr),
        }
    }

    fn is_narrowing_typeof_operands(&self, expr1: &Arc<Node>, expr2: &Arc<Node>) -> bool {
        expr1.kind == SyntaxKind::TypeOfExpression
            && expr1
                .expression()
                .map(|e| self.is_narrowable_operand(e))
                .unwrap_or(false)
            && matches!(
                expr2.kind,
                SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral
            )
    }

    fn contains_narrowable_reference(&self, expr: &Arc<Node>) -> bool {
        if self.is_narrowable_reference(expr) {
            return true;
        }
        if expr.flags.contains(NodeFlags::OptionalChain) {
            if let Some(inner) = expr.expression() {
                if matches!(
                    expr.kind,
                    SyntaxKind::PropertyAccessExpression
                        | SyntaxKind::ElementAccessExpression
                        | SyntaxKind::CallExpression
                        | SyntaxKind::NonNullExpression
                ) {
                    return self.contains_narrowable_reference(inner);
                }
            }
        }
        false
    }

    fn is_narrowable_reference(&self, node: &Arc<Node>) -> bool {
        match node.kind {
            SyntaxKind::Identifier
            | SyntaxKind::ThisKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::MetaProperty => true,
            SyntaxKind::PropertyAccessExpression | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::NonNullExpression => {
                node.expression().map(|e| self.is_narrowable_reference(e)).unwrap_or(false)
            }
            SyntaxKind::ElementAccessExpression => {
                let NodeData::ElementAccessExpression(el) = &node.data else {
                    return false;
                };
                self.is_string_or_numeric_literal_like(&el.argument_expression)
                    || (self.is_entity_name_expression(&el.argument_expression)
                        && self.is_narrowable_reference(&el.expression))
            }
            SyntaxKind::BinaryExpression => {
                let NodeData::BinaryExpression(bin) = &node.data else {
                    return false;
                };
                (bin.operator_token.kind == SyntaxKind::CommaToken
                    && self.is_narrowable_reference(&bin.right))
                    || (is_assignment_operator(bin.operator_token.kind)
                        && crate::ast::utilities::is_left_hand_side_expression(&bin.left))
            }
            _ => false,
        }
    }

    fn has_narrowable_argument(&self, expr: &Arc<Node>) -> bool {
        let NodeData::CallExpression(call) = &expr.data else {
            return false;
        };
        call.arguments
            .nodes
            .iter()
            .any(|arg| self.contains_narrowable_reference(arg))
    }

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

    fn bind_throw_statement(&mut self, node: &Arc<Node>) {
        if let NodeData::ThrowStatement(data) = &node.data {
            self.bind(&data.expression);
        }
        self.current_flow = Some(self.unreachable_flow());
        self.has_flow_effects = true;
    }

    fn bind_try_statement(&mut self, node: &Arc<Node>) {
        let stmt = match &node.data {
            NodeData::TryStatement(data) => data,
            _ => return,
        };

        let save_return_target = self.current_return_target.take();
        let save_exception_target = self.current_exception_target.take();

        let normal_exit_label = Self::new_flow_accumulator();
        let return_label = Self::new_flow_accumulator();
        let mut exception_label = Self::new_flow_accumulator();

        if stmt.finally_block.is_some() {
            self.current_return_target = Some(Arc::clone(&return_label));
        }

        if let Some(current) = &self.current_flow {
            self.add_antecedent_to_flow(&exception_label, current);
        }
        self.current_exception_target = Some(Arc::clone(&exception_label));

        self.bind(&stmt.try_block);
        if let Some(current) = &self.current_flow {
            self.add_antecedent_to_flow(&normal_exit_label, current);
        }

        if let Some(catch_clause) = &stmt.catch_clause {
            self.current_flow = Some(Self::finish_flow_node(
                &exception_label,
                &self.unreachable_flow(),
            ));
            let catch_exception_label = Self::new_flow_accumulator();
            if let Some(current) = &self.current_flow {
                self.add_antecedent_to_flow(&catch_exception_label, current);
            }
            self.current_exception_target = Some(Arc::clone(&catch_exception_label));
            exception_label = catch_exception_label;
            self.bind(catch_clause);
            if let Some(current) = &self.current_flow {
                self.add_antecedent_to_flow(&normal_exit_label, current);
            }
        }

        self.current_return_target = save_return_target;
        self.current_exception_target = save_exception_target;

        if let Some(finally_block) = &stmt.finally_block {

            let finally_label = Self::new_flow_accumulator();
            for ant in normal_exit_label
                .antecedents
                .iter()
                .chain(exception_label.antecedents.iter())
                .chain(return_label.antecedents.iter())
            {
                self.add_antecedent_to_flow(&finally_label, ant);
            }
            let finally_node = Self::finish_flow_node(&finally_label, &self.unreachable_flow());
            self.current_flow = Some(Arc::clone(&finally_node));
            self.bind(finally_block);

            if self
                .current_flow
                .as_ref()
                .is_some_and(|f| f.flags.contains(FlowFlags::UNREACHABLE))
            {

                self.current_flow = Some(self.unreachable_flow());
            } else {
                let current_flow = self.current_flow.clone().expect("reachable flow");

                if self.current_return_target.is_some()
                    && !return_label.antecedents.is_empty()
                    && let Some(rt) = &self.current_return_target
                {
                    let reduce = self.create_reduce_label(
                        &finally_node,
                        &return_label.antecedents,
                        &current_flow,
                    );
                    self.add_antecedent_to_flow(rt, &reduce);
                }

                if self.current_exception_target.is_some()
                    && !exception_label.antecedents.is_empty()
                    && let Some(et) = &self.current_exception_target
                {
                    let reduce = self.create_reduce_label(
                        &finally_node,
                        &exception_label.antecedents,
                        &current_flow,
                    );
                    self.add_antecedent_to_flow(et, &reduce);
                }

                if !normal_exit_label.antecedents.is_empty() {
                    self.current_flow = Some(self.create_reduce_label(
                        &finally_node,
                        &normal_exit_label.antecedents,
                        &current_flow,
                    ));
                } else {
                    self.current_flow = Some(self.unreachable_flow());
                }
            }
        } else {
            self.current_flow = Some(Self::finish_flow_node(
                &normal_exit_label,
                &self.unreachable_flow(),
            ));
        }
    }

    fn finish_flow_node(
        node: &Arc<FlowNode>,
        unreachable: &Arc<FlowNode>,
    ) -> Arc<FlowNode> {
        if node.antecedents.is_empty() {
            return Arc::clone(unreachable);
        }
        if node.antecedents.len() == 1 {
            return Arc::clone(&node.antecedents[0]);
        }
        Arc::clone(node)
    }

    fn bind_break_statement(&mut self, node: &Arc<Node>) {

        let label_name = if let NodeData::BreakStatement(data) = &node.data {
            data.label.as_ref().map(|l| self.node_text(l))
        } else {
            None
        };

        if let Some(name) = label_name {

            let break_target = {
                let mut current = &self.active_label_list;
                let mut found = None;
                while let Some(label) = current {
                    if label.name == name {
                        found = Some(Arc::clone(&label.break_target));
                        break;
                    }
                    current = &label.next;
                }
                found
            };
            if let Some(target) = break_target {
                if let Some(current_flow) = &self.current_flow {
                    self.add_antecedent_to_flow(&target, current_flow);
                }

                let mut current = &mut self.active_label_list;
                while let Some(label) = current {
                    if label.name == name {
                        label.referenced = true;
                        break;
                    }
                    current = &mut label.next;
                }
            }
        } else if let Some(target) = &self.current_break_target {

            if let Some(current) = &self.current_flow {
                self.add_antecedent_to_flow(target, current);
            }
        }
        self.current_flow = Some(self.unreachable_flow());
    }

    fn bind_continue_statement(&mut self, node: &Arc<Node>) {

        let label_name = if let NodeData::ContinueStatement(data) = &node.data {
            data.label.as_ref().map(|l| self.node_text(l))
        } else {
            None
        };

        if let Some(name) = label_name {

            let continue_target = {
                let mut current = &self.active_label_list;
                let mut found = None;
                while let Some(label) = current {
                    if label.name == name {
                        found = label.continue_target.clone();
                        break;
                    }
                    current = &label.next;
                }
                found
            };
            if let Some(ref target) = continue_target {
                if let Some(current_flow) = &self.current_flow {
                    self.add_antecedent_to_flow(&target, current_flow);
                }

                let mut current = &mut self.active_label_list;
                while let Some(label) = current {
                    if label.name == name {
                        label.referenced = true;
                        break;
                    }
                    current = &mut label.next;
                }
            }
        } else if let Some(target) = &self.current_continue_target {
            if let Some(current) = &self.current_flow {
                self.add_antecedent_to_flow(target, current);
            }
        }
        self.current_flow = Some(self.unreachable_flow());
    }

    fn set_continue_target(&mut self, loop_node: &Arc<Node>, target: &Arc<FlowNode>) {
        let mut node = Arc::clone(loop_node);
        let mut cursor = &mut self.active_label_list;
        loop {
            let Some(parent) = node.parent.clone() else { break };
            if parent.kind != SyntaxKind::LabeledStatement {
                break;
            }
            let Some(label) = cursor else { break };
            label.continue_target = Some(Arc::clone(target));
            node = parent;
            cursor = &mut label.next;
        }
    }

    fn bind_labeled_statement(&mut self, node: &Arc<Node>) {
        let stmt = match &node.data {
            NodeData::LabeledStatement(data) => data,
            _ => return,
        };

        let label_name = self.node_text(&stmt.label);

        let break_target = Self::new_flow_accumulator();

        let continue_target: Option<Arc<FlowNode>> = None;

        let active_label = Box::new(ActiveLabel {
            name: label_name,
            break_target: Arc::clone(&break_target),
            continue_target,
            referenced: false,
            next: self.active_label_list.take(),
        });

        self.active_label_list = Some(active_label);

        self.bind(&stmt.statement);

        let was_referenced = self
            .active_label_list
            .as_ref()
            .map_or(false, |l| l.referenced);

        self.active_label_list = self.active_label_list.take().and_then(|l| l.next);

        if !was_referenced {

            let label_ptr = Arc::as_ptr(&stmt.label) as *mut Node;
            unsafe {
                (*label_ptr).flags |= NodeFlags::Unreachable;
            }
        }

        if let Some(current) = &self.current_flow {
            self.add_antecedent_to_flow(&break_target, current);
        }
        self.current_flow = if break_target.antecedents.is_empty() {
            Some(self.unreachable_flow())
        } else {
            Some(break_target)
        };
    }

    fn is_push_or_unshift_identifier(&self, name: &str) -> bool {
        name == "push" || name == "unshift"
    }

    fn is_mutation_tracked_reference(&self, expr: &Arc<Node>) -> bool {
        match expr.kind {
            SyntaxKind::Identifier
            | SyntaxKind::ThisKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::MetaProperty => true,
            SyntaxKind::PropertyAccessExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::NonNullExpression => {
                if let Some(inner) = expr.expression() {
                    self.is_mutation_tracked_reference(&inner)
                } else {
                    false
                }
            }
            SyntaxKind::ElementAccessExpression => {

                if let NodeData::ElementAccessExpression(ea) = &expr.data {
                    if self.is_string_or_numeric_literal_like(&ea.argument_expression) {
                        return true;
                    }
                    return self.is_entity_name_expression(&ea.argument_expression)
                        && self.is_mutation_tracked_reference(&ea.expression);
                }
                false
            }
            _ => false,
        }
    }

    fn is_string_or_numeric_literal_like(&self, node: &Arc<Node>) -> bool {
        matches!(
            node.kind,
            SyntaxKind::StringLiteral
                | SyntaxKind::NumericLiteral
                | SyntaxKind::NoSubstitutionTemplateLiteral
        )
    }

    fn is_entity_name_expression(&self, node: &Arc<Node>) -> bool {
        matches!(
            node.kind,
            SyntaxKind::Identifier | SyntaxKind::QualifiedName
        )
    }

    fn bind_call_expression_flow(&mut self, node: &Arc<Node>) {
        if let NodeData::CallExpression(data) = &node.data {
            let expr = &data.expression;

            if let NodeData::PropertyAccessExpression(prop) = &expr.data {
                let name = self.node_text(&prop.name);
                if self.is_push_or_unshift_identifier(&name)
                    && self.is_mutation_tracked_reference(&prop.expression)
                {

                    let current = self.current_flow.clone();
                    if let Some(current) = current {
                        self.current_flow = Some(self.create_flow_mutation(&current, node));
                    }
                }
            }
        }
    }

    fn bind_this_property_assignment(&mut self, _node: &Arc<Node>) {

    }

    fn collect_expando_assignment(&mut self, node: &Arc<Node>) {
        let NodeData::BinaryExpression(bin) = &node.data else {
            return;
        };
        if bin.operator_token.kind != SyntaxKind::EqualsToken {
            return;
        }
        let base = match &bin.left.data {
            NodeData::PropertyAccessExpression(pae)
                if pae.expression.kind == SyntaxKind::Identifier
                    && pae.name.kind == SyntaxKind::Identifier =>
            {
                &pae.expression
            }
            NodeData::ElementAccessExpression(eae)
                if eae.expression.kind == SyntaxKind::Identifier =>
            {
                &eae.expression
            }
            _ => return,
        };

        let base_name = base.text();
        if matches!(base_name, "exports" | "module" | "globalThis") {
            return;
        }
        self.expando_assignments
            .push((Arc::clone(node), self.block_scope_container.clone()));
    }

    fn process_expando_assignments(&mut self) {
        let assignments = std::mem::take(&mut self.expando_assignments);
        for (node, scope_start) in assignments {
            let NodeData::BinaryExpression(bin) = &node.data else {
                continue;
            };
            let base = match &bin.left.data {
                NodeData::PropertyAccessExpression(pae) => &pae.expression,
                NodeData::ElementAccessExpression(eae) => &eae.expression,
                _ => continue,
            };
            let base_name = base.text();
            let mut target: Option<Arc<Symbol>> = None;
            let mut scope = scope_start;
            while let Some(sc) = scope {
                if let Some(sym) = self
                    .symbol_map
                    .locals
                    .get(&sc.id())
                    .and_then(|l| l.get(base_name))
                {
                    target = Some(Arc::clone(sym));
                    break;
                }

                if matches!(
                    sc.kind,
                    SyntaxKind::SourceFile | SyntaxKind::ModuleDeclaration
                ) && let Some(sym) = self.symbol_map.symbol_of(&sc)
                {
                    let hit = sym
                        .members
                        .get(base_name)
                        .or_else(|| sym.exports.get(base_name))
                        .cloned();
                    if let Some(h) = hit {
                        target = Some(h);
                        break;
                    }
                }
                scope = sc.parent.clone();
            }
            let Some(sym) = target else { continue };

            if !sym
                .value_declaration
                .as_ref()
                .is_some_and(|d| d.kind == SyntaxKind::FunctionDeclaration)
            {
                continue;
            }
            let member_name: Option<String> = match &bin.left.data {
                NodeData::PropertyAccessExpression(pae) => Some(pae.name.text().to_string()),
                NodeData::ElementAccessExpression(eae) => {
                    match &eae.argument_expression.data {
                        NodeData::StringLiteral(s) => Some(s.text.clone()),
                        NodeData::NumericLiteral(n) => Some(n.text.clone()),
                        _ => None,
                    }
                }
                _ => None,
            };
            match member_name {
                Some(mname) => {

                    let existing = sym
                        .exports
                        .get(&mname)
                        .or_else(|| sym.members.get(&mname))
                        .cloned()
                        .or_else(|| {
                            sym.declarations
                                .iter()
                                .filter(|d| d.kind == SyntaxKind::ModuleDeclaration)
                                .find_map(|md| {
                                    self.symbol_map
                                        .locals
                                        .get(&md.id())
                                        .and_then(|l| l.get(&mname))
                                        .cloned()
                                })
                        });
                    let eligible = existing.as_ref().map_or(true, |e| {
                        e.declarations
                            .iter()
                            .all(|d| d.kind == SyntaxKind::BinaryExpression)
                    });
                    if !eligible {
                        continue;
                    }
                    match existing {
                        Some(e) => {
                            let e_mut = Arc::as_ptr(&e) as *mut Symbol;
                            unsafe { (*e_mut).declarations.push(Arc::clone(&node)) };
                        }
                        None => {
                            let prop = self.new_symbol(SymbolFlags::Property, mname.clone());
                            let prop_mut = Arc::as_ptr(&prop) as *mut Symbol;
                            unsafe {
                                (*prop_mut).declarations.push(Arc::clone(&node));
                                (*prop_mut).parent = Some(Arc::clone(&sym));
                            }
                            let sym_mut = Arc::as_ptr(&sym) as *mut Symbol;
                            unsafe {
                                (*sym_mut).exports.insert(mname, prop);
                            }
                        }
                    }
                }
                None => {

                    let pseudo = sym
                        .exports
                        .get(crate::ast::INTERNAL_SYMBOL_NAME_ASSIGNMENT)
                        .cloned();
                    match pseudo {
                        Some(p) => {
                            let p_mut = Arc::as_ptr(&p) as *mut Symbol;
                            unsafe { (*p_mut).declarations.push(Arc::clone(&node)) };
                        }
                        None => {
                            let p = self.new_symbol(
                                SymbolFlags::empty(),
                                crate::ast::INTERNAL_SYMBOL_NAME_ASSIGNMENT.to_string(),
                            );
                            let p_mut = Arc::as_ptr(&p) as *mut Symbol;
                            unsafe {
                                (*p_mut).declarations.push(Arc::clone(&node));
                                (*p_mut).parent = Some(Arc::clone(&sym));
                            }
                            let sym_mut = Arc::as_ptr(&sym) as *mut Symbol;
                            unsafe {
                                (*sym_mut)
                                    .exports
                                    .insert(crate::ast::INTERNAL_SYMBOL_NAME_ASSIGNMENT.to_string(), p);
                            }
                        }
                    }
                }
            }
        }
    }

    fn bind_expression_statement(&mut self, node: &Arc<Node>) {
        if let NodeData::ExpressionStatement(data) = &node.data {
            self.bind(&data.expression);

            if let NodeData::BinaryExpression(bin_data) = &data.expression.data {
                if is_assignment_operator(bin_data.operator_token.kind) {
                    if let Some(current) = self.current_flow.take() {
                        let assign_flow = self.create_flow_assignment(&current, &data.expression);
                        self.symbol_map
                            .set_flow_node(&data.expression, Arc::clone(&assign_flow));
                        self.current_flow = Some(assign_flow);
                    }

                    if let NodeData::ElementAccessExpression(ea) = &bin_data.left.data {
                        if self.is_mutation_tracked_reference(&ea.expression) {
                            let current = self.current_flow.clone();
                            if let Some(current) = current {
                                self.current_flow =
                                    Some(self.create_flow_mutation(&current, &data.expression));
                            }
                        }
                    }
                }
            }

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

    fn is_in_for_in_or_of_head(node: &Arc<Node>) -> bool {
        let Some(parent) = &node.parent else {
            return false;
        };
        let Some(grandparent) = &parent.parent else {
            return false;
        };
        matches!(
            grandparent.kind,
            SyntaxKind::ForInStatement | SyntaxKind::ForOfStatement
        )
    }

    fn bind_assignment_target_flow(&mut self, node: &Arc<Node>) {
        match &node.data {
            NodeData::ArrayLiteralExpression(arr) => {
                for e in &arr.elements.nodes {
                    if e.kind == SyntaxKind::SpreadElement {
                        if let Some(inner) = e.expression() {
                            self.bind_assignment_target_flow(&inner);
                        }
                    } else {
                        self.bind_destructuring_target_flow(e);
                    }
                }
            }
            NodeData::ObjectLiteralExpression(obj) => {
                for p in &obj.properties.nodes {
                    match &p.data {
                        NodeData::PropertyAssignment(pa) => {
                            self.bind_destructuring_target_flow(&pa.initializer);
                        }
                        NodeData::ShorthandPropertyAssignment(sa) => {
                            self.bind_assignment_target_flow(&sa.name);
                        }
                        NodeData::SpreadAssignment(sp) => {
                            self.bind_assignment_target_flow(&sp.expression);
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                if self.is_mutation_tracked_reference(node)
                    && matches!(
                        node.kind,
                        SyntaxKind::Identifier
                            | SyntaxKind::PropertyAccessExpression
                            | SyntaxKind::ElementAccessExpression
                            | SyntaxKind::ParenthesizedExpression
                            | SyntaxKind::NonNullExpression
                            | SyntaxKind::ThisKeyword
                            | SyntaxKind::SuperKeyword
                            | SyntaxKind::MetaProperty
                    )
                {
                    if let Some(current) = self.current_flow.take() {
                        let assign_flow = self.create_flow_assignment(&current, node);
                        self.current_flow = Some(assign_flow);
                    }
                }
            }
        }
    }

    fn bind_destructuring_target_flow(&mut self, node: &Arc<Node>) {
        if let NodeData::BinaryExpression(bin) = &node.data {
            if bin.operator_token.kind == SyntaxKind::EqualsToken {
                self.bind_assignment_target_flow(&bin.left);
                return;
            }
        }
        self.bind_assignment_target_flow(node);
    }

    fn bind_initialized_variable_flow(&mut self, node: &Arc<Node>) {
        let name = match &node.data {
            NodeData::VariableDeclaration(d) => Some(Arc::clone(&d.name)),
            NodeData::BindingElement(d) => d.name.clone(),
            _ => None,
        };
        let Some(name) = name else { return };
        if matches!(
            name.kind,
            SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
        ) {
            if let NodeData::BindingPattern(pattern) = &name.data {
                for child in &pattern.elements.nodes {
                    self.bind_initialized_variable_flow(child);
                }
            }
            return;
        }
        if let Some(current) = self.current_flow.take() {
            let assign_flow = self.create_flow_assignment(&current, node);
            self.symbol_map.set_flow_node(node, Arc::clone(&assign_flow));
            self.current_flow = Some(assign_flow);
        }
    }

    fn check_contextual_identifier(&mut self, node: &Arc<Node>) {
        let Some(file) = self.current_source_file.clone() else {
            return;
        };
        if file.has_parse_diagnostics
            || node.flags.contains(NodeFlags::Ambient)
            || node.flags.contains(NodeFlags::JSDoc)
            || is_identifier_name(node)
            || file.is_declaration_file
        {
            return;
        }

        {
            let mut anc = node.parent.as_ref();
            while let Some(a) = anc {
                if a.has_syntactic_modifier(ModifierFlags::Ambient) {
                    return;
                }
                anc = a.parent.as_ref();
            }
        }
        let Some(kind) = crate::scanner::string_to_keyword(node.text()) else {
            return;
        };
        let is_future_reserved = matches!(
            kind,
            SyntaxKind::ImplementsKeyword
                | SyntaxKind::InterfaceKeyword
                | SyntaxKind::LetKeyword
                | SyntaxKind::PackageKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::ProtectedKeyword
                | SyntaxKind::PublicKeyword
                | SyntaxKind::StaticKeyword
                | SyntaxKind::YieldKeyword
        );
        let message = if is_future_reserved {
            if crate::ast::utilities::get_containing_class(node).is_some() {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_CLASS_DEFINITIONS_ARE_AUTOMATICALLY_IN_STRICT_MODE
            } else if file.external_module_indicator.is_some() {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE
            } else {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE
            }
        } else if kind == SyntaxKind::AwaitKeyword {
            if file.external_module_indicator.is_some()
                && crate::ast::utilities::is_in_top_level_context(node)
            {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_AT_THE_TOP_LEVEL_OF_A_MODULE
            } else if node.flags.contains(NodeFlags::AwaitContext) {
                IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE
            } else {
                return;
            }
        } else if kind == SyntaxKind::YieldKeyword && node.flags.contains(NodeFlags::YieldContext) {
            IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE
        } else {
            return;
        };
        self.symbol_map.binder_diagnostics.push(Diagnostic::new(
            Some(file),
            node.loc,
            message,
            vec![node.text().to_string()],
        ));
    }

    fn bind(&mut self, node: &Arc<Node>) {

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

    fn bind_children(&mut self, node: &Arc<Node>) {

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
                | ContainerFlags::IS_THIS_CONTAINER
        }
        SyntaxKind::FunctionDeclaration
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::Constructor => {
            ContainerFlags::IS_CONTAINER
                | ContainerFlags::IS_CONTROL_FLOW_CONTAINER
                | ContainerFlags::IS_FUNCTION_LIKE
                | ContainerFlags::HAS_LOCALS
                | ContainerFlags::IS_THIS_CONTAINER
        }

        SyntaxKind::MethodSignature
        | SyntaxKind::CallSignature
        | SyntaxKind::ConstructSignature
        | SyntaxKind::FunctionType
        | SyntaxKind::ConstructorType => {
            ContainerFlags::IS_CONTAINER
                | ContainerFlags::IS_CONTROL_FLOW_CONTAINER
                | ContainerFlags::IS_FUNCTION_LIKE
                | ContainerFlags::HAS_LOCALS
        }
        SyntaxKind::IndexSignature => {
            ContainerFlags::IS_CONTAINER | ContainerFlags::HAS_LOCALS
        }

        SyntaxKind::TypeAliasDeclaration | SyntaxKind::JSTypeAliasDeclaration | SyntaxKind::MappedType => {
            ContainerFlags::IS_CONTAINER | ContainerFlags::HAS_LOCALS
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

#[allow(dead_code)]
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

fn is_block_only_container(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Block
            | SyntaxKind::CatchClause
            | SyntaxKind::ForStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::CaseBlock
    )
}

fn is_var_container_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::SourceFile
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor
    )
}

fn collect_binding_elements<'a>(node: &'a Arc<Node>, out: &mut Vec<&'a Arc<Node>>) {
    if let NodeData::BindingPattern(pattern) = &node.data {
        for el in pattern.elements.iter() {
            out.push(el);
            let name = match &el.data {
                NodeData::BindingElement(be) => &be.name,
                _ => continue,
            };
            if let Some(name_node) = name
                && matches!(name_node.data, NodeData::BindingPattern(_))
            {
                collect_binding_elements(name_node, out);
            }
        }
    }
}

fn fn_like_body_present(parent: &Arc<Node>) -> bool {
    match &parent.data {
        NodeData::FunctionDeclaration(d) => d.body.is_some(),
        NodeData::MethodDeclaration(d) => d.body.is_some(),
        NodeData::ConstructorDeclaration(d) => d.body.is_some(),
        NodeData::GetAccessorDeclaration(d) => d.body.is_some(),
        NodeData::SetAccessorDeclaration(d) => d.body.is_some(),
        NodeData::FunctionExpression(_) | NodeData::ArrowFunction(_) => true,
        _ => false,
    }
}

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
            | SyntaxKind::MethodSignature
            | SyntaxKind::FunctionType
            | SyntaxKind::ConstructorType
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::MappedType
    )
}

pub fn bind_source_file(file: &Arc<SourceFile>) -> NodeSymbolMap {
    let mut binder = Binder::new();
    binder.bind_source_file(file);
    std::mem::take(&mut binder.symbol_map)
}

fn clause_statements_empty(clause: &Arc<Node>) -> bool {
    matches!(&clause.data, NodeData::CaseOrDefaultClause(d) if d.statements.nodes.is_empty())
}

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

    fn parse_and_bind(source: &str) -> (Arc<SourceFile>, NodeSymbolMap) {
        let source_file = Arc::new(Parser::parse_source_file_text("test.ts", source.to_string()));
        let symbol_map = bind_source_file(&Arc::clone(&source_file));
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

        let var_stmt = &statements.nodes[0];
        assert_eq!(var_stmt.kind, SyntaxKind::VariableStatement);

        let mut binder = Binder::new();
        binder.bind_source_file(&Arc::clone(&file));
        assert!(binder.symbol_count() >= 2);
        let _ = map;
    }

    #[test]
    fn bind_function_declaration() {
        let (file, _map) = parse_and_bind("function foo() { return 42; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&Arc::clone(&file));
        assert!(binder.symbol_count() >= 2);
    }

    #[test]
    fn bind_class_declaration() {
        let (file, _map) = parse_and_bind("class Foo { bar() {} }");
        let mut binder = Binder::new();
        binder.bind_source_file(&Arc::clone(&file));
        assert!(binder.symbol_count() >= 3);
    }

    #[test]
    fn bind_interface_declaration() {
        let (file, _map) = parse_and_bind("interface Foo { bar: number; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 3);
    }

    #[test]
    fn bind_import_declaration() {
        let (file, _map) = parse_and_bind("import { foo } from 'mod';");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);

        let _ = binder.symbol_count();
    }

    #[test]
    fn bind_multiple_declarations() {
        let (file, _map) = parse_and_bind("let x = 1; let y = 2; let z = 3;");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.symbol_count() >= 4);
    }

    #[test]
    fn bind_nested_scope() {
        let (file, _map) = parse_and_bind("function foo() { let x = 1; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);

        assert!(binder.symbol_count() >= 3);
    }

    #[test]
    fn flow_start_node_exists() {
        let (file, map) = parse_and_bind("let x = 1;");

        let flow = map.flow_node_of(&file.node);
        assert!(flow.is_some());
        let flow = flow.unwrap();
        assert!(flow.flags.contains(FlowFlags::START));
    }

    #[test]
    fn flow_identifier_has_flow_node() {
        let (file, map) = parse_and_bind("let x = 1; x;");

        let statements = match &file.node.data {
            NodeData::SourceFile(data) => &data.statements,
            _ => unreachable!(),
        };

        let expr_stmt = &statements.nodes[1];
        let expr = match &expr_stmt.data {
            NodeData::ExpressionStatement(data) => &data.expression,
            _ => unreachable!(),
        };
        assert_eq!(expr.kind, SyntaxKind::Identifier);

        assert!(map.flow_node_of(expr).is_some());
    }

    #[test]
    fn flow_if_statement_merges() {

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

        let (file, _map) =
            parse_and_bind("try { let x = 1; } catch (e) { let y = 2; } finally { let z = 3; }");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    #[test]
    fn flow_try_with_throw_in_catch() {

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

        let (file, _map) = parse_and_bind("let arr = []; arr.push(1);");
        let mut binder = Binder::new();
        binder.bind_source_file(&file);
        assert!(binder.has_flow_effects);
    }

    fn file_symbol<'a>(file: &'a SourceFile, map: &'a NodeSymbolMap) -> &'a Arc<Symbol> {
        map.symbols
            .get(&file.node.id())
            .expect("source file should have a symbol")
    }

    fn find_statement(file: &SourceFile, kind: SyntaxKind) -> Option<Arc<Node>> {
        let NodeData::SourceFile(data) = &file.node.data else {
            return None;
        };
        data.statements
            .nodes
            .iter()
            .find(|n| n.kind == kind)
            .cloned()
    }

    fn find_child(node: &Arc<Node>, kind: SyntaxKind) -> Option<Arc<Node>> {
        let mut found: Option<Arc<Node>> = None;
        crate::ast::node_data_generated::for_each_child(node, |child| {
            if child.kind == kind {
                found = Some(Arc::clone(child));
                true
            } else {
                false
            }
        });
        found
    }

    fn find_descendant(node: &Arc<Node>, kind: SyntaxKind) -> Option<Arc<Node>> {
        if node.kind == kind {
            return Some(Arc::clone(node));
        }
        let mut found: Option<Arc<Node>> = None;
        crate::ast::node_data_generated::for_each_child(node, |child| {
            if found.is_none() {
                found = find_descendant(child, kind);
            }
            found.is_some()
        });
        found
    }

    #[test]
    fn bind_export_default_expression_creates_default_export_symbol() {

        let (file, map) = parse_and_bind("export default 42;");
        let export_assignment =
            find_statement(&file, SyntaxKind::ExportAssignment).expect("export assignment");
        let sym = map.symbol_of(&export_assignment).expect("symbol");
        assert!(
            sym.flags.contains(SymbolFlags::Property),
            "expected Property flags, got {:?}",
            sym.flags
        );
        assert_eq!(sym.name, INTERNAL_SYMBOL_NAME_DEFAULT);
        let file_sym = file_symbol(&file, &map);
        let default_export = file_sym
            .exports
            .get(INTERNAL_SYMBOL_NAME_DEFAULT)
            .expect("default export in file exports");
        assert!(Arc::ptr_eq(default_export, sym));
    }

    #[test]
    fn bind_export_default_identifier_creates_alias() {

        let (file, map) = parse_and_bind("const foo = 1; export default foo;");
        let export_assignment =
            find_statement(&file, SyntaxKind::ExportAssignment).expect("export assignment");
        let sym = map.symbol_of(&export_assignment).expect("symbol");
        assert!(
            sym.flags.contains(SymbolFlags::Alias),
            "expected Alias flags, got {:?}",
            sym.flags
        );
        assert_eq!(sym.name, INTERNAL_SYMBOL_NAME_DEFAULT);
    }

    #[test]
    fn bind_export_equals_creates_export_equals_symbol() {

        let (file, map) = parse_and_bind("function x() {} export = x;");
        let export_assignment =
            find_statement(&file, SyntaxKind::ExportAssignment).expect("export assignment");
        let sym = map.symbol_of(&export_assignment).expect("symbol");
        assert!(sym.flags.contains(SymbolFlags::Alias));
        assert_eq!(sym.name, INTERNAL_SYMBOL_NAME_EXPORT_EQUALS);
        assert!(
            sym.value_declaration.is_some(),
            "export = should have a value declaration set"
        );
        let file_sym = file_symbol(&file, &map);
        assert!(
            file_sym
                .exports
                .get(INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
                .is_some()
        );
    }

    #[test]
    fn bind_export_star_creates_export_star_symbol() {

        let (file, map) = parse_and_bind("export * from \"mod\";");
        let export_decl =
            find_statement(&file, SyntaxKind::ExportDeclaration).expect("export declaration");
        let sym = map.symbol_of(&export_decl).expect("symbol");
        assert!(
            sym.flags.contains(SymbolFlags::ExportStar),
            "expected ExportStar flags, got {:?}",
            sym.flags
        );
        assert_eq!(sym.name, INTERNAL_SYMBOL_NAME_EXPORT_STAR);
        let file_sym = file_symbol(&file, &map);
        assert!(
            file_sym
                .exports
                .get(INTERNAL_SYMBOL_NAME_EXPORT_STAR)
                .is_some()
        );
    }

    #[test]
    fn bind_export_star_as_ns_creates_alias() {

        let (file, map) = parse_and_bind("export * as ns from \"mod\";");
        let export_decl =
            find_statement(&file, SyntaxKind::ExportDeclaration).expect("export declaration");
        let ns_clause =
            find_child(&export_decl, SyntaxKind::NamespaceExport).expect("NamespaceExport clause");
        let sym = map
            .symbol_of(&ns_clause)
            .expect("symbol on NamespaceExport clause");
        assert!(sym.flags.contains(SymbolFlags::Alias));
        assert_eq!(sym.name, "ns");
        let file_sym = file_symbol(&file, &map);
        let ns_export = file_sym.exports.get("ns").expect("ns export");
        assert!(Arc::ptr_eq(ns_export, sym));
    }

    #[test]
    fn bind_export_named_specifiers_does_not_duplicate() {

        let (file, map) = parse_and_bind("const a = 1; const b = 2; export { a, b };");
        let export_decl =
            find_statement(&file, SyntaxKind::ExportDeclaration).expect("export declaration");

        assert!(
            map.symbol_of(&export_decl).is_none(),
            "export {{ a, b }} should not create a symbol on the ExportDeclaration"
        );
    }

    #[test]
    fn bind_import_clause_default_import_creates_local_alias() {

        let (file, map) = parse_and_bind("import D from \"mod\";");
        let import_decl =
            find_statement(&file, SyntaxKind::ImportDeclaration).expect("import declaration");
        let clause = find_child(&import_decl, SyntaxKind::ImportClause).expect("import clause");
        let sym = map.symbol_of(&clause).expect("symbol on ImportClause");
        assert!(sym.flags.contains(SymbolFlags::Alias));
        assert_eq!(sym.name, "D");
        let locals = map.locals.get(&file.node.id()).expect("file locals table");
        let local_sym = locals.get("D").expect("D in file locals");
        assert!(Arc::ptr_eq(local_sym, sym));
        let file_sym = file_symbol(&file, &map);
        assert!(
            file_sym.exports.get("D").is_none(),
            "default import should not be in exports"
        );
    }

    #[test]
    fn bind_import_clause_without_name_is_noop() {

        let (file, map) = parse_and_bind("import { x } from \"mod\";");
        let import_decl =
            find_statement(&file, SyntaxKind::ImportDeclaration).expect("import declaration");
        let clause = find_child(&import_decl, SyntaxKind::ImportClause).expect("import clause");
        assert!(
            map.symbol_of(&clause).is_none(),
            "ImportClause without a name should not get a symbol"
        );
    }

    #[test]
    fn bind_exported_namespace_member_has_export_symbol_link() {

        let (file, map) = parse_and_bind("namespace N { export const x = 1; }");

        let ns = find_statement(&file, SyntaxKind::ModuleDeclaration).expect("namespace N");
        let ns_sym = map.symbol_of(&ns).expect("namespace symbol");
        let x_export = ns_sym.exports.get("x").expect("x in N's exports");
        assert!(
            x_export.export_symbol.is_some(),
            "exported namespace member should have export_symbol set"
        );
        assert!(Arc::ptr_eq(
            x_export.export_symbol.as_ref().unwrap(),
            x_export
        ));
    }

    #[test]
    fn bind_non_exported_namespace_member_has_no_export_symbol() {

        let (file, map) = parse_and_bind("namespace N { const x = 1; }");
        let ns = find_statement(&file, SyntaxKind::ModuleDeclaration).expect("namespace N");
        let ns_sym = map.symbol_of(&ns).expect("namespace symbol");
        assert!(
            ns_sym.exports.get("x").is_none(),
            "non-exported member should not be in exports"
        );

        let locals = map.locals.get(&ns.id()).expect("namespace locals table");
        let x_local = locals.get("x").expect("x in locals");
        assert!(
            x_local.export_symbol.is_none(),
            "non-exported member should not have export_symbol"
        );
    }

    #[test]
    fn bind_exported_top_level_member_has_export_symbol_link() {

        let (file, map) = parse_and_bind("export const x = 1;");
        let var_stmt =
            find_statement(&file, SyntaxKind::VariableStatement).expect("variable statement");

        let decl_list =
            find_child(&var_stmt, SyntaxKind::VariableDeclarationList).expect("declaration list");
        let var_decl =
            find_child(&decl_list, SyntaxKind::VariableDeclaration).expect("variable declaration");
        let sym = map.symbol_of(&var_decl).expect("symbol for x");
        assert!(
            sym.export_symbol.is_some(),
            "exported top-level member should have export_symbol set"
        );
        assert!(Arc::ptr_eq(sym.export_symbol.as_ref().unwrap(), sym));
    }

    #[test]
    fn bind_generic_alias_type_params_do_not_leak_into_file_members() {

        let (file, map) = parse_and_bind(
            "export type G<T> = { [P in T]: string };\nexport type T = G<\"a\">;\nexport const q = 1;",
        );
        let fsym = file_symbol(&file, &map);
        let t_in_file = fsym.members.get("T").or_else(|| fsym.exports.get("T"));
        let Some(t_sym) = t_in_file else {
            panic!("exported alias T should be reachable in the file symbol tables");
        };

        assert!(
            t_sym
                .declarations
                .iter()
                .all(|d| d.kind == SyntaxKind::TypeAliasDeclaration),
            "file-table T merged with a type parameter: flags={:?}",
            t_sym.flags
        );
        assert!(
            !t_sym.flags.intersects(SymbolFlags::TypeParameter),
            "exported alias T must not carry TypeParameter flags (got {:?})",
            t_sym.flags
        );

        let g_stmt = find_statement(&file, SyntaxKind::TypeAliasDeclaration).unwrap();
        let g_sym = map.symbol_of(&g_stmt).expect("symbol for G");
        assert!(
            g_sym.members.get("T").is_some(),
            "G's type parameter should live in the alias symbol's members"
        );
    }

    #[test]
    fn bind_mapped_type_param_in_node_locals() {

        let (file, map) = parse_and_bind("type M<K extends string> = { [P in K]: number };");
        let fsym = file_symbol(&file, &map);
        assert!(
            fsym.members.get("P").is_none() && fsym.exports.get("P").is_none(),
            "mapped-type P must not leak into the file symbol tables"
        );
        let mapped = find_descendant(&file.node, SyntaxKind::MappedType).expect("mapped type node");
        let locals = map
            .locals
            .get(&mapped.id())
            .expect("mapped type node should have locals");
        assert!(
            locals.get("P").is_some(),
            "P should be in the mapped node's locals"
        );
    }
}
