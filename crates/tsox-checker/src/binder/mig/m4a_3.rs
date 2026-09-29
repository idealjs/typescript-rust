use crate::binder::nameresolver::NameResolver;
use crate::binder::*;
use std::sync::Arc;
use super::m3h::get_initializer_symbol;
use super::r19k5_msg as msg;
use tsox_core::diagnostics::Message;
use crate::binder::bind_js_assignment_declarations::get_element_or_property_access_name;
use tsox_frontend::ast::mig::m3f_4::is_dotted_name;
use tsox_frontend::scanner::mig::m3i::declaration_name_to_string;
use tsox_frontend::ast::*;

impl Binder {
    pub(crate) fn find_active_label(&self, name: &str) -> Option<&ActiveLabel> {
        let mut label = self.active_label_list.as_deref();
        while let Some(l) = label {
            if l.name == name {
                return Some(l);
            }
            label = l.next.as_deref();
        }
        None
    }

    pub(crate) fn get_display_name(&self, node: &Arc<Node>) -> String {
        if let Some(name_node) = node.name() {
            return declaration_name_to_string(Some(&name_node));
        }
        let name = self.get_declaration_name(node);
        if name != "___missing" {
            return name;
        }
        "(Missing)".to_string()
    }

    pub(crate) fn get_strict_mode_block_scope_function_declaration_message(
        &self,
        node: &Arc<Node>,
    ) -> &'static Message {
        if get_containing_class(node).is_some() {
            &msg::Function_declarations_are_not_allowed_inside_blocks_in_strict_mode_when_targeting_ES5_Class_definitions_are_automatically_in_strict_mode
        } else if self
            .current_source_file
            .as_ref()
            .is_some_and(|f| f.external_module_indicator.is_some())
        {
            &msg::Function_declarations_are_not_allowed_inside_blocks_in_strict_mode_when_targeting_ES5_Modules_are_automatically_in_strict_mode
        } else {
            &msg::Function_declarations_are_not_allowed_inside_blocks_in_strict_mode_when_targeting_ES5
        }
    }

    pub(crate) fn get_strict_mode_identifier_message(&self, node: &Arc<Node>) -> &'static Message {
        if get_containing_class(node).is_some() {
            &msg::Identifier_expected_0_is_a_reserved_word_in_strict_mode_Class_definitions_are_automatically_in_strict_mode
        } else if self
            .current_source_file
            .as_ref()
            .is_some_and(|f| f.external_module_indicator.is_some())
        {
            &msg::Identifier_expected_0_is_a_reserved_word_in_strict_mode_Modules_are_automatically_in_strict_mode
        } else {
            &msg::Identifier_expected_0_is_a_reserved_word_in_strict_mode
        }
    }

    pub(crate) fn get_this_class_and_symbol_table(
        &self,
    ) -> (Option<Arc<Symbol>>, Option<SymbolTable>) {
        let this_container = match &self.this_container {
            Some(c) => c,
            None => return (None, None),
        };
        match this_container.kind {
            SyntaxKind::FunctionDeclaration | SyntaxKind::FunctionExpression => (None, None),
            SyntaxKind::Constructor
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::ClassStaticBlockDeclaration => {
                let class_symbol = this_container
                    .parent()
                    .and_then(|p| self.symbol_map.symbol_of(&p).cloned());
                match &class_symbol {
                    Some(sym) => {
                        let table = if is_static(this_container) {
                            sym.exports.clone()
                        } else {
                            sym.members.clone()
                        };
                        (Some(Arc::clone(sym)), Some(table))
                    }
                    None => (None, None),
                }
            }
            _ => (None, None),
        }
    }

    pub(crate) fn lookup_entity(
        &self,
        node: &Arc<Node>,
        container: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        if is_identifier(node) {
            return self.lookup_name(node.text(), container);
        }
        if node.expression().map(|e| e.kind) == Some(SyntaxKind::ThisKeyword) {
            let (_, symbol_table) = self.get_this_class_and_symbol_table();
            if let Some(table) = symbol_table
                && let Some(name) = get_element_or_property_access_name(node)
            {
                return table.get(name.text()).cloned();
            }
            return None;
        }
        let initializer_symbol = node
            .expression()
            .and_then(|expr| self.lookup_entity(&expr, container))
            .and_then(|s| get_initializer_symbol(Some(&s)));
        if let Some(symbol) = &initializer_symbol
            && let Some(name) = get_element_or_property_access_name(node)
        {
            return symbol.exports.get(name.text()).cloned();
        }
        None
    }

    pub(crate) fn lookup_name(&self, name: &str, container: &Arc<Node>) -> Option<Arc<Symbol>> {
        if let Some(locals) = self.symbol_map.locals_of(container) {
            if let Some(local) = locals.get(name) {
                return Some(local.export_symbol.clone().unwrap_or_else(|| Arc::clone(local)));
            }
        }
        if let Some(declaration) = self.symbol_map.symbol_of(container) {
            return declaration.exports.get(name).cloned();
        }
        None
    }

    pub(crate) fn maybe_bind_expression_flow_if_call(&mut self, node: &Arc<Node>) {
        if node.kind == SyntaxKind::CallExpression {
            let expression_kind = node.expression().map(|e| e.kind);
            if expression_kind != Some(SyntaxKind::SuperKeyword)
                && expression_kind.is_some_and(|_| node.expression().is_some_and(|e| is_dotted_name(&e)))
            {
                let current = self.current_flow.clone();
                self.current_flow = Some(self.create_flow_call(current.as_ref().unwrap(), node));
            }
        }
    }

    pub(crate) fn new_flow_node_ex(
        &self,
        flags: FlowFlags,
        node: Option<Arc<Node>>,
        antecedent: Option<Arc<FlowNode>>,
    ) -> FlowNode {
        let mut result = self.new_flow_node(flags);
        result.node = node;
        result.antecedent = antecedent;
        result
    }
}
