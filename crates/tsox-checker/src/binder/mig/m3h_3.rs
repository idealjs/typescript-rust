use crate::binder::*;
use std::sync::Arc;
use tsox_frontend::ast::mig::m3f_3::is_async_function;
use tsox_frontend::ast::mig::m3b::parameters;
use tsox_frontend::ast::mig::m3g_2::{
    is_object_literal_or_class_expression_method_or_accessor, is_parameter_property_declaration,
};
use tsox_frontend::ast::{Node, NodeData};

thread_local! {
    static EMIT_FLAGS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

fn add_async_emit_flag() {
    EMIT_FLAGS.with(|f| f.set(f.get() | tsox_frontend::ast::NodeFlags::HasAsyncFunctions.bits()));
}

fn clone_flow_node(n: &tsox_frontend::ast::FlowNode) -> tsox_frontend::ast::FlowNode {
    tsox_frontend::ast::FlowNode {
        flags: n.flags,
        node: n.node.clone(),
        antecedent: n.antecedent.clone(),
        antecedents: n.antecedents.clone(),
        switch_statement: n.switch_statement.clone(),
        clause_range: n.clause_range.clone(),
        reduce_target: n.reduce_target.clone(),
    }
}

impl Binder {
    pub(crate) fn bind_function_declaration(&mut self, node: &Arc<Node>) {
        let is_declaration_file = self
            .current_source_file
            .as_ref()
            .is_some_and(|f| f.is_declaration_file);
        if !is_declaration_file
            && !node.flags.contains(tsox_frontend::ast::NodeFlags::Ambient)
            && is_async_function(node)
        {
            add_async_emit_flag();
        }
        self.check_strict_mode_function_name(node);
        self.bind_block_scoped_declaration(
            node,
            SymbolFlags::Function,
            SymbolFlags::FunctionExcludes,
        );
    }

    pub(crate) fn bind_function_expression(&mut self, node: &Arc<Node>) {
        let is_declaration_file = self
            .current_source_file
            .as_ref()
            .is_some_and(|f| f.is_declaration_file);
        if !is_declaration_file
            && !node.flags.contains(tsox_frontend::ast::NodeFlags::Ambient)
            && is_async_function(node)
        {
            add_async_emit_flag();
        }
        if let Some(flow) = self.current_flow.as_ref() {
            self.symbol_map.set_flow_node(node, Arc::clone(flow));
        }
        let mut binding_name = INTERNAL_SYMBOL_NAME_FUNCTION.to_string();
        if is_function_expression(node) && node.name().is_some() {
            self.check_strict_mode_function_name(node);
            binding_name = node.name().unwrap().text().to_string();
        }
        self.bind_anonymous_declaration(node, SymbolFlags::Function, &binding_name);
    }

    pub(crate) fn bind_function_or_constructor_type(&mut self, node: &Arc<Node>) {
        let declaration_name = self.get_declaration_name(node);
        let symbol = self.new_symbol(SymbolFlags::Signature, &declaration_name);
        self.add_declaration_to_symbol(&symbol, node, SymbolFlags::Signature);
        let type_literal_symbol = self.new_symbol(SymbolFlags::TypeLiteral, INTERNAL_SYMBOL_NAME_TYPE);
        self.add_declaration_to_symbol(&type_literal_symbol, node, SymbolFlags::TypeLiteral);
        {
            let ptr = Arc::as_ptr(&type_literal_symbol) as *mut Symbol;
            unsafe {
                (*ptr).members.insert(symbol.name.clone(), Arc::clone(&symbol));
            }
        }
    }

    pub(crate) fn bind_iterative_statement(
        &mut self,
        node: &Arc<Node>,
        break_target: &Arc<FlowLabel>,
        continue_target: &Arc<FlowLabel>,
    ) {
        let save_break_target = self.current_break_target.clone();
        let save_continue_target = self.current_continue_target.clone();
        self.current_break_target = Some(Arc::new(clone_flow_node(&break_target.node)));
        self.current_continue_target = Some(Arc::new(clone_flow_node(&continue_target.node)));
        self.bind(node);
        self.current_break_target = save_break_target;
        self.current_continue_target = save_continue_target;
    }

    pub(crate) fn bind_jsx_attribute(
        &mut self,
        node: &Arc<Node>,
        symbol_flags: SymbolFlags,
        symbol_excludes: SymbolFlags,
    ) {
        self.declare_symbol_and_add_to_symbol_table(node, symbol_flags, symbol_excludes);
    }

    pub(crate) fn bind_jsx_attributes(&mut self, node: &Arc<Node>) {
        self.bind_anonymous_declaration(
            node,
            SymbolFlags::ObjectLiteral,
            INTERNAL_SYMBOL_NAME_JSX_ATTRIBUTES,
        );
    }

    pub(crate) fn bind_modifiers(&mut self, modifiers: Option<&Arc<ModifierList>>) {
        if let Some(modifiers) = modifiers {
            self.bind_each(&modifiers.nodes);
        }
    }

    pub(crate) fn bind_node_list(&mut self, node_list: Option<&Arc<NodeList>>) {
        if let Some(node_list) = node_list {
            self.bind_each(&node_list.nodes);
        }
    }

    pub(crate) fn bind_parameter(&mut self, node: &Arc<Node>) {
        let decl_name = node.name().cloned();
        let question_token = match &node.data {
            NodeData::ParameterDeclaration(d) => d.question_token.clone(),
            _ => None,
        };
        if !node.flags.contains(tsox_frontend::ast::NodeFlags::Ambient) {
            if let Some(name) = &decl_name {
                self.check_strict_mode_eval_or_arguments(node, Some(name));
            }
        }
        if decl_name.as_ref().is_some_and(|n| is_binding_pattern(n)) {
            let parent = node.parent().unwrap();
            let index = parameters(&parent)
                .iter()
                .position(|p| Arc::ptr_eq(p, node))
                .unwrap_or(0);
            self.bind_anonymous_declaration(
                node,
                SymbolFlags::FunctionScopedVariable,
                &format!("__{index}"),
            );
        } else {
            self.declare_symbol_and_add_to_symbol_table(
                node,
                SymbolFlags::FunctionScopedVariable,
                SymbolFlags::ParameterExcludes,
            );
        }
        let parent = node.parent().unwrap();
        if is_parameter_property_declaration(node, &parent) {
            let mut flags = SymbolFlags::Property;
            if question_token.is_some() {
                flags |= SymbolFlags::Optional;
            }
            self.declare_symbol(
                node,
                flags,
                SymbolFlags::PropertyExcludes,
            );
        }
    }

    pub(crate) fn bind_property_or_method_or_accessor(
        &mut self,
        node: &Arc<Node>,
        symbol_flags: SymbolFlags,
        symbol_excludes: SymbolFlags,
    ) {
        let is_declaration_file = self
            .current_source_file
            .as_ref()
            .is_some_and(|f| f.is_declaration_file);
        if !is_declaration_file
            && !node.flags.contains(tsox_frontend::ast::NodeFlags::Ambient)
            && is_async_function(node)
        {
            add_async_emit_flag();
        }
        if self.current_flow.is_some() && is_object_literal_or_class_expression_method_or_accessor(node)
        {
            self.symbol_map.set_flow_node(node, Arc::clone(self.current_flow.as_ref().unwrap()));
        }
        if tsox_frontend::ast::mig::m3f_2::has_dynamic_name(node) {
            self.bind_anonymous_declaration(node, symbol_flags, INTERNAL_SYMBOL_NAME_COMPUTED);
        } else {
            self.declare_symbol_and_add_to_symbol_table(node, symbol_flags, symbol_excludes);
        }
    }

    pub(crate) fn lookup_entity_option(
        &self,
        node: &Option<Arc<Node>>,
        container: Option<&Arc<Node>>,
    ) -> Option<Arc<Symbol>> {
        match (node, container) {
            (Some(node), Some(container)) => self.lookup_entity(node, container),
            _ => None,
        }
    }
}
