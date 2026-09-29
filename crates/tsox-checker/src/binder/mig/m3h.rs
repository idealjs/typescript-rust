use super::m4a_4::{is_assignment_declaration, is_effective_module_declaration};
use crate::binder::*;
use std::sync::Arc;
use tsox_frontend::scanner::mig::m3i::get_source_text_of_node_from_source_file;
use tsox_frontend::ast::FlowFlags;

thread_local! {
    static LAST_CONTAINER: std::cell::RefCell<Option<Arc<Node>>> = const { std::cell::RefCell::new(None) };
}

pub(crate) const INTERNAL_SYMBOL_NAME_ASSIGNMENT_DECLARATION: &str = "\u{FE}assignment";

pub(crate) fn get_exports(symbol: &Arc<Symbol>) -> &mut SymbolTable {
    let ptr = Arc::as_ptr(symbol) as *mut Symbol;
    unsafe { &mut (*ptr).exports }
}

pub(crate) fn add_antecedent(b: &Binder, label: &Arc<FlowNode>, antecedent: &Arc<FlowNode>) {
    b.add_antecedent_to_flow(label, antecedent);
}

pub(crate) fn clone_active_label(label: &ActiveLabel) -> Box<ActiveLabel> {
    Box::new(ActiveLabel {
        name: label.name.clone(),
        break_target: Arc::clone(&label.break_target),
        continue_target: label.continue_target.clone(),
        referenced: label.referenced,
        next: None,
    })
}

impl ActiveLabel {
    pub(crate) fn break_target(&self) -> Arc<FlowNode> {
        Arc::clone(&self.break_target)
    }

    pub(crate) fn continue_target(&self) -> Option<Arc<FlowNode>> {
        self.continue_target.as_ref().map(Arc::clone)
    }

    pub(crate) fn set_referenced(&self) {
        let ptr = self as *const ActiveLabel as *mut ActiveLabel;
        unsafe {
            (*ptr).referenced = true;
        }
    }
}

pub(crate) fn set_symbol_flags_or(symbol: &Arc<Symbol>, flags: SymbolFlags) {
    let ptr = Arc::as_ptr(symbol) as *mut Symbol;
    unsafe {
        (*ptr).flags |= flags;
    }
}

pub(crate) fn remove_symbol_flags(symbol: &Arc<Symbol>, flags: SymbolFlags) {
    let ptr = Arc::as_ptr(symbol) as *mut Symbol;
    unsafe {
        (*ptr).flags = (*ptr).flags.difference(flags);
    }
}

pub(crate) fn push_declaration(symbol: &Arc<Symbol>, node: &Arc<Node>) {
    let ptr = Arc::as_ptr(symbol) as *mut Symbol;
    unsafe {
        (*ptr).declarations.push(Arc::clone(node));
    }
}

pub(crate) fn push_declaration_if_unique(symbol: &Arc<Symbol>, node: &Arc<Node>) {
    let ptr = Arc::as_ptr(symbol) as *mut Symbol;
    unsafe {
        if !(*ptr).declarations.iter().any(|d| Arc::ptr_eq(d, node)) {
            (*ptr).declarations.push(Arc::clone(node));
        }
    }
}

pub(crate) fn find_use_strict_prologue(
    source_file: &Arc<SourceFile>,
    statements: &[Arc<Node>],
) -> Option<Arc<Node>> {
    for statement in statements {
        if is_prologue_directive(statement) {
            if is_use_strict_prologue_directive(source_file, statement) {
                return Some(Arc::clone(statement));
            }
        } else {
            return None;
        }
    }
    None
}

fn is_use_strict_prologue_directive(
    source_file: &Arc<SourceFile>,
    node: &Arc<Node>,
) -> bool {
    let node_text =
        get_source_text_of_node_from_source_file(source_file, &node.expression().unwrap(), false);
    node_text == "\"use strict\"" || node_text == "'use strict'"
}

pub(crate) fn get_symbol_name_for_private_identifier(
    containing_class_symbol: &Arc<Symbol>,
    description: &str,
) -> String {
    format!(
        "{}#{}@{}",
        INTERNAL_SYMBOL_NAME_PREFIX,
        get_symbol_id(containing_class_symbol),
        description
    )
}

pub(crate) fn set_value_declaration(symbol: &Arc<Symbol>, node: &Arc<Node>) {
    let value_declaration = symbol.value_declaration.as_ref();
    let replace = match value_declaration {
        None => true,
        Some(vd) => {
            (is_assignment_declaration(vd) && !is_assignment_declaration(node))
                || (vd.kind != node.kind && is_effective_module_declaration(vd))
        }
    };
    if replace {
        let ptr = Arc::as_ptr(symbol) as *mut Symbol;
        unsafe {
            (*ptr).value_declaration = Some(Arc::clone(node));
        }
    }
}

pub(crate) fn get_parent_of_property_assignment(node: &Arc<Node>) -> Option<Arc<Node>> {
    let parent = node.parent()?;
    if parent.kind == SyntaxKind::PropertyAssignment {
        return Some(parent);
    }
    let grandparent = parent.parent()?;
    if grandparent.kind == SyntaxKind::PropertyAssignment {
        Some(grandparent)
    } else {
        None
    }
}

pub(crate) fn get_alias_target_symbol(initializer: &Arc<Node>) -> Option<Arc<Symbol>> {
    let _ = initializer;
    None
}

pub(crate) fn get_initializer_symbol(symbol: Option<&Arc<Symbol>>) -> Option<Arc<Symbol>> {
    let symbol = symbol?;
    if symbol.flags.intersects(SymbolFlags::Alias) {
        let first_declaration = symbol.declarations.first()?;
        let target = first_declaration.initializer()?;
        return get_alias_target_symbol(target);
    }
    Some(Arc::clone(symbol))
}

impl Binder {
    pub(crate) fn add_declaration_to_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        node: &Arc<Node>,
        symbol_flags: SymbolFlags,
    ) {
        set_symbol_flags_or(symbol, symbol_flags);
        self.symbol_map.set_symbol(node, Arc::clone(symbol));
        push_declaration_if_unique(symbol, node);
        if symbol
            .flags
            .intersects(SymbolFlags::ConstEnumOnlyModule)
            && symbol.flags.intersects(
                SymbolFlags::Function
                    | SymbolFlags::Class
                    | SymbolFlags::RegularEnum,
            )
        {
            remove_symbol_flags(symbol, SymbolFlags::ConstEnumOnlyModule);
            self.not_const_enum_only_modules.insert(symbol.id());
        }
        if symbol_flags.intersects(SymbolFlags::VALUE) {
            set_value_declaration(symbol, node);
        }
    }

    pub(crate) fn add_diagnostic(&mut self, diagnostic: &Arc<Diagnostic>) {
        self.symbol_map
            .binder_diagnostics
            .push((**diagnostic).clone());
        if let Some(file) = &self.current_source_file {
            let mut file_diags = file.bind_diagnostics();
            file_diags.push(Arc::clone(diagnostic));
            file.set_bind_diagnostics(file_diags);
        }
    }

    pub(crate) fn add_late_bound_assignment_declaration_to_symbol(
        &mut self,
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) {
        let exports = get_exports(symbol);
        let assignment_symbol = match exports.get(INTERNAL_SYMBOL_NAME_ASSIGNMENT_DECLARATION) {
            Some(existing) => Arc::clone(existing),
            None => {
                let created =
                    self.new_symbol(SymbolFlags::empty(), INTERNAL_SYMBOL_NAME_ASSIGNMENT_DECLARATION);
                exports.insert(
                    INTERNAL_SYMBOL_NAME_ASSIGNMENT_DECLARATION.to_string(),
                    Arc::clone(&created),
                );
                created
            }
        };
        push_declaration(&assignment_symbol, node);
    }

    pub(crate) fn add_to_container_chain(&mut self, next: &Arc<Node>) {
        let _ = next;
        LAST_CONTAINER.with(|c| *c.borrow_mut() = Some(Arc::clone(next)));
    }

    pub(crate) fn bind_block_scoped_declaration(
        &mut self,
        node: &Arc<Node>,
        symbol_flags: SymbolFlags,
        symbol_excludes: SymbolFlags,
    ) {
        match self.block_scope_container.as_ref().map(|c| c.kind) {
            Some(SyntaxKind::ModuleDeclaration) => {
                self.declare_module_member(node, symbol_flags, symbol_excludes);
            }
            Some(SyntaxKind::SourceFile) => {
                let is_external_or_common_js = self
                    .current_source_file
                    .as_ref()
                    .map(|f| is_external_or_common_js_module(f))
                    .unwrap_or(false);
                if is_external_or_common_js {
                    self.declare_module_member(node, symbol_flags, symbol_excludes);
                } else {
                    self.declare_symbol(node, symbol_flags, symbol_excludes);
                }
            }
            _ => {
                self.declare_symbol(node, symbol_flags, symbol_excludes);
            }
        }
    }

    pub(crate) fn bind_break_or_continue_flow(&mut self, flow_label: Option<&Arc<FlowNode>>) {
        if let Some(flow_label) = flow_label {
            FlowLabel::push_antecedent(flow_label, Arc::clone(self.current_flow.as_ref().unwrap()));
            let current = self.current_flow.as_ref().unwrap();
            self.set_flow_node_referenced(current);
            self.current_flow = self.unreachable_flow.clone();
            self.has_flow_effects = true;
        }
    }

    pub(crate) fn bind_break_or_continue_statement(
        &mut self,
        label: Option<&Arc<Node>>,
        current_target: Option<Arc<FlowNode>>,
        get_target: impl Fn(&ActiveLabel) -> Option<Arc<FlowNode>>,
    ) {
        if let Some(label) = label {
            self.bind(label);
            let active_label = self.find_active_label_cloned(tsox_frontend::ast::Node::text(label));
            if let Some(mut active_label) = active_label {
                active_label.set_referenced();
                let target = get_target(&active_label);
                self.bind_break_or_continue_flow(target.as_ref());
            }
        } else {
            self.bind_break_or_continue_flow(current_target.as_ref());
        }
    }

    pub(crate) fn find_active_label_cloned(&self, name: &str) -> Option<Box<ActiveLabel>> {
        let mut label = self.active_label_list.as_ref();
        while let Some(current) = label {
            if current.name == name {
                return Some(clone_active_label(current));
            }
            label = current.next.as_ref();
        }
        None
    }

    pub(crate) fn bind_call_expression(&mut self, node: &Arc<Node>) {
        let indicator_none = self
            .current_source_file
            .as_ref()
            .is_none_or(|f| f.common_js_module_indicator.is_none());
        if indicator_none && is_require_call(node, false) {
            self.set_common_js_module_indicator(node);
        }
    }
}
