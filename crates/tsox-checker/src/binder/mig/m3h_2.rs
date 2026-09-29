use super::m3h::{add_antecedent, get_exports, set_symbol_flags_or, INTERNAL_SYMBOL_NAME_ASSIGNMENT_DECLARATION};
use super::m4a_4::{is_logical_assignment_expression, get_parent_of_property_assignment, get_initializer_symbol};
use crate::binder::*;
use std::sync::Arc;
use tsox_frontend::ast::mig::m3f::get_symbol_id;
use tsox_frontend::ast::mig::m3f_2::has_dynamic_name;
use tsox_frontend::ast::mig::m3g::is_logical_expression;
use tsox_frontend::ast::mig::m3g_2::is_outermost_optional_chain;
use tsox_frontend::ast::mig::x6a::is_enum_const;

thread_local! {
    static PRE_SWITCH_CASE_FLOW: std::cell::RefCell<Option<Arc<FlowNode>>> = const { std::cell::RefCell::new(None) };
}

pub(crate) fn pre_switch_case_flow() -> Option<Arc<FlowNode>> {
    PRE_SWITCH_CASE_FLOW.with(|f| f.borrow().clone())
}

pub(crate) fn set_pre_switch_case_flow(flow: Option<Arc<FlowNode>>) {
    PRE_SWITCH_CASE_FLOW.with(|f| *f.borrow_mut() = flow);
}

fn finish_flow_label_node(b: &Binder, label: &Arc<FlowNode>) -> Arc<FlowNode> {
    if label.antecedents.is_empty() {
        return Arc::clone(b.unreachable_flow.as_ref().unwrap());
    }
    if label.antecedents.len() == 1 {
        return Arc::clone(&label.antecedents[0]);
    }
    Arc::clone(label)
}

pub(crate) fn set_clause_fallthrough_flow_node(
    b: &mut Binder,
    clause: &Arc<Node>,
    flow: Option<Arc<FlowNode>>,
) {
    if let Some(flow) = flow {
        b.symbol_map.set_flow_node(clause, flow);
    }
}

impl Binder {
    pub(crate) fn bind_case_block(&mut self, node: &Arc<Node>) {
        let switch_statement = node.parent().unwrap();
        let clauses: Vec<Arc<Node>> = match &node.data {
            NodeData::CaseBlock(d) => d.clauses.nodes.clone(),
            _ => Vec::new(),
        };
        let switch_expression_kind = switch_statement
            .expression()
            .map(|e| e.kind)
            .unwrap_or(SyntaxKind::Unknown);
        let is_narrowing_switch = switch_expression_kind == SyntaxKind::TrueKeyword
            || switch_statement
                .expression()
                .is_some_and(|e| self.is_narrowing_expression(e));
        let unreachable = self.unreachable_flow.clone().unwrap();
        let mut fallthrough_flow = Some(Arc::clone(&unreachable));
        let total = clauses.len();
        let mut i = 0;
        while i < total {
            let clause_start = i;
            while matches!(&clauses[i].data, NodeData::CaseOrDefaultClause(d) if d.statements.nodes.is_empty())
                && i + 1 < total
            {
                if Arc::ptr_eq(fallthrough_flow.as_ref().unwrap(), &unreachable) {
                    self.current_flow = pre_switch_case_flow();
                }
                self.bind(&clauses[i]);
                i += 1;
            }
            let pre_case_label = Arc::new(self.create_branch_label());
            let mut pre_case_flow = pre_switch_case_flow();
            if is_narrowing_switch {
                pre_case_flow = Some(self.create_flow_switch_clause(
                    pre_switch_case_flow().as_ref().unwrap(),
                    None,
                    &switch_statement,
                    clause_start,
                    i + 1,
                ));
            }
            add_antecedent(self, &pre_case_label, pre_case_flow.as_ref().unwrap());
            add_antecedent(self, &pre_case_label, fallthrough_flow.as_ref().unwrap());
            self.current_flow = Some(finish_flow_label_node(self, &pre_case_label));
            let clause = &clauses[i];
            self.bind(clause);
            fallthrough_flow = self.current_flow.clone();
            let reachable = self
                .current_flow
                .as_ref()
                .is_some_and(|f| !f.flags.contains(FlowFlags::UNREACHABLE));
            if reachable && i != total - 1 {
                let flow = self.current_flow.clone();
                set_clause_fallthrough_flow_node(self, clause, flow);
            }
            i += 1;
        }
    }

    pub(crate) fn bind_case_or_default_clause(&mut self, node: &Arc<Node>) {
        let clause_expression: Option<Arc<Node>> = match &node.data {
            NodeData::CaseOrDefaultClause(d) if node.kind == SyntaxKind::CaseClause => {
                Some(Arc::clone(&d.expression))
            }
            _ => None,
        };
        let statements: Vec<Arc<Node>> = match &node.data {
            NodeData::CaseOrDefaultClause(d) => d.statements.nodes.clone(),
            _ => Vec::new(),
        };
        if let Some(expression) = clause_expression {
            let save_current_flow = self.current_flow.clone();
            self.current_flow = pre_switch_case_flow();
            self.bind(&expression);
            self.current_flow = save_current_flow;
        }
        self.bind_each(&statements);
    }

    pub(crate) fn bind_class_like_declaration(&mut self, node: &Arc<Node>) {
        let name = node.name().cloned();
        match node.kind {
            SyntaxKind::ClassDeclaration => {
                self.bind_block_scoped_declaration(
                    node,
                    SymbolFlags::Class,
                    SymbolFlags::ClassExcludes,
                );
            }
            SyntaxKind::ClassExpression => {
                let name_text = name
                    .as_ref()
                    .map(|n| n.text().to_string())
                    .unwrap_or_else(|| INTERNAL_SYMBOL_NAME_CLASS.to_string());
                self.bind_anonymous_declaration(node, SymbolFlags::Class, &name_text);
            }
            _ => {}
        }
        let symbol = self
            .symbol_map
            .symbol_of(node)
            .map(Arc::clone)
            .unwrap_or_else(|| self.new_symbol(SymbolFlags::empty(), INTERNAL_SYMBOL_NAME_CLASS));
        let prototype_symbol =
            self.new_symbol(SymbolFlags::Property.union(SymbolFlags::Prototype), "prototype");
        let exports = get_exports(&symbol);
        if let Some(symbol_export) = exports.get(&prototype_symbol.name) {
            let decl = symbol_export.declarations.first().unwrap().clone();
            self.error_on_node(&decl, &DUPLICATE_IDENTIFIER_0, &[prototype_symbol.name.clone()]);
        }
        exports.insert(prototype_symbol.name.clone(), Arc::clone(&prototype_symbol));
        prototype_symbol.set_parent(&symbol);
    }
    pub(crate) fn bind_common_js_type_exports(&mut self, module_symbol: &Arc<Symbol>) {
        let module_exports = get_exports(module_symbol);
        if let Some(export_equals) = module_exports
            .get(INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
            .cloned()
        {
            let export_equals_exports = get_exports(&export_equals);
            for symbol in module_exports.entries.values() {
                if symbol.name != INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
                    && symbol
                        .flags
                        .intersects(SymbolFlags::TYPE.union(SymbolFlags::NAMESPACE))
                {
                    export_equals_exports.insert(symbol.name.clone(), Arc::clone(symbol));
                    set_symbol_flags_or(&export_equals, SymbolFlags::NamespaceModule);
                }
            }
        }
    }

    pub(crate) fn bind_condition(
        &mut self,
        node: Option<&Arc<Node>>,
        true_target: &Arc<FlowNode>,
        false_target: &Arc<FlowNode>,
    ) {
        if let Some(node) = node {
            self.bind(node);
        }
        let plain_condition = match node {
            None => true,
            Some(n) => {
                !is_logical_assignment_expression(n)
                    && !is_logical_expression(n)
                    && !(is_optional_chain(n) && is_outermost_optional_chain(n))
            }
        };
        if plain_condition {
            if let (Some(n), Some(antecedent)) = (node, self.current_flow.clone()) {
                let true_condition = self.create_flow_condition(
                    FlowFlags::TRUE_CONDITION,
                    &antecedent,
                    n,
                );
                add_antecedent(self, true_target, &true_condition);
                let false_condition = self.create_flow_condition(
                    FlowFlags::FALSE_CONDITION,
                    &antecedent,
                    n,
                );
                add_antecedent(self, false_target, &false_condition);
            }
        }
    }

    pub(crate) fn bind_deferred_expando_assignment(&mut self, node: &Arc<Node>) {
        let parent = get_parent_of_property_assignment(node);
        let mut symbol = self.lookup_entity_option(&parent, self.block_scope_container.as_ref());
        if symbol.is_none() {
            symbol = self.lookup_entity_option(&parent, self.container.as_ref());
        }
        if let Some(symbol) = get_initializer_symbol(symbol.as_ref()) {
            if has_dynamic_name(node) {
                self.bind_anonymous_declaration(
                    node,
                    SymbolFlags::Property.union(SymbolFlags::Assignment),
                    INTERNAL_SYMBOL_NAME_COMPUTED,
                );
                self.add_late_bound_assignment_declaration_to_symbol(node, &symbol);
            } else {
                let exports = get_exports(&symbol);
                let declaration_name = self.get_declaration_name(node);
                let existing_is_assignment = exports
                    .get(&declaration_name)
                    .is_none_or(|existing| existing.flags.intersects(SymbolFlags::Assignment));
                if existing_is_assignment {
                    self.declare_symbol(
                        node,
                        SymbolFlags::Property.union(SymbolFlags::Assignment),
                        SymbolFlags::PropertyExcludes,
                    );
                }
            }
        }
    }

    pub(crate) fn bind_deferred_expando_assignments(&mut self) {
        let assignments = std::mem::take(&mut self.expando_assignments);
        for info in &assignments {
            self.container = info.container.clone();
            self.block_scope_container = info.block_scope_container.clone();
            self.bind_deferred_expando_assignment(&info.node);
        }
        self.expando_assignments = assignments;
    }

    pub(crate) fn bind_each(&mut self, nodes: &[Arc<Node>]) {
        for node in nodes {
            self.bind(node);
        }
    }

    pub(crate) fn bind_each_child(&mut self, node: &Arc<Node>) {
        for_each_child(node, |n| {
            self.bind(n);
            false
        });
    }

    pub(crate) fn bind_each_statement_functions_first(&mut self, statements: &Arc<NodeList>) {
        for node in &statements.nodes {
            if node.kind == SyntaxKind::FunctionDeclaration {
                self.bind(node);
            }
        }
        for node in &statements.nodes {
            if node.kind != SyntaxKind::FunctionDeclaration {
                self.bind(node);
            }
        }
    }

    pub(crate) fn bind_enum_declaration(&mut self, node: &Arc<Node>) {
        if is_enum_const(node) {
            self.bind_block_scoped_declaration(
                node,
                SymbolFlags::ConstEnum,
                SymbolFlags::ConstEnumExcludes,
            );
        } else {
            self.bind_block_scoped_declaration(
                node,
                SymbolFlags::RegularEnum,
                SymbolFlags::RegularEnumExcludes,
            );
        }
    }

    pub(crate) fn bind_expando_property_assignment(&mut self, node: &Arc<Node>) {
        self.expando_assignments.push(ExpandoAssignmentInfo {
            node: Arc::clone(node),
            container: self.container.clone(),
            block_scope_container: self.block_scope_container.clone(),
        });
    }
}
