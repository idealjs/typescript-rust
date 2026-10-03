#![allow(unused_imports)]
#![allow(dead_code)]

use crate::checker::checker::*;
use crate::checker::flow_flow_max_depth::{FlowQuery, FlowRef};
use crate::checker::types_type_id::TYPE_FLAGS_NULLABLE;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tsox_frontend::ast::{Diagnostic, NodeData};
use tsox_frontend::ast::NodeList;
use tsox_frontend::ast::SyntaxKind;

thread_local! {
    static CONTEXT_FREE_TYPES: RefCell<HashMap<usize, Arc<Type>>> = RefCell::new(HashMap::new());
}

pub(crate) fn context_free_types_get(node: &Arc<Node>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("context_free_types_get"); 
    CONTEXT_FREE_TYPES.with(|m| m.borrow().get(&(Arc::as_ptr(node) as *const () as usize)).cloned())
}

pub(crate) fn context_free_types_insert(node: &Arc<Node>, t: &Arc<Type>) { ::tsox_core::fntrace::enter("context_free_types_insert"); 
    CONTEXT_FREE_TYPES.with(|m| {
        m.borrow_mut()
            .insert(Arc::as_ptr(node) as *const () as usize, Arc::clone(t))
    });
}

thread_local! {
    static CONTEXTUAL_BINDING_PATTERNS: RefCell<Vec<Arc<Node>>> = RefCell::new(Vec::new());
}

pub(crate) fn contextual_binding_patterns_contains_declaration(decl: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("contextual_binding_patterns_contains_declaration"); 
    CONTEXTUAL_BINDING_PATTERNS.with(|v| {
        v.borrow()
            .iter()
            .any(|p| p.parent().map(|dp| Arc::ptr_eq(&dp, decl)).unwrap_or(false))
    })
}

impl Checker {
    pub(crate) fn non_inferrable_any_type(&mut self) -> Arc<Type> { ::tsox_core::fntrace::enter("non_inferrable_any_type"); 
        let mut t = self.new_object_type(ObjectFlags::Anonymous, None);
        if let Some(t_mut) = Arc::get_mut(&mut t) {
            t_mut.object_flags |= ObjectFlags::NonInferrableType;
        }
        t
    }

    pub(crate) fn resolving_signature(&self) -> Arc<Signature> { ::tsox_core::fntrace::enter("resolving_signature"); 
        Arc::clone(self.resolving_signature.get_or_init(|| {
            let mut sig = Signature::new();
            sig.flags = SignatureFlags::empty();
            sig.min_argument_count = 0;
            sig.resolved_min_argument_count = -1;
            sig.resolved_return_type = OnceLock::from(self.any_type());
            Arc::new(sig)
        }))
    }

    pub(crate) fn create_promise_return_type(&mut self, _node: &Arc<Node>, promised: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("create_promise_return_type"); 
        self.create_promise_return_type_for(promised)
    }

    pub(crate) fn get_global_import_type_checked(&mut self, name: &str) -> Arc<Type> { ::tsox_core::fntrace::enter("get_global_import_type_checked"); 
        match self.get_global_symbol(name, SymbolFlags::TYPE, None) {
            Some(symbol) => self.get_type_of_symbol(&symbol),
            None => self.empty_object_type(),
        }
    }

    pub(crate) fn get_global_import_meta_type(&mut self) -> Arc<Type> { ::tsox_core::fntrace::enter("get_global_import_meta_type"); 
        self.get_global_import_type_checked("ImportMeta")
    }

    pub(crate) fn get_global_import_call_options_type_checked(&mut self) -> Arc<Type> { ::tsox_core::fntrace::enter("get_global_import_call_options_type_checked"); 
        self.get_global_import_type_checked("ImportCallOptions")
    }

    pub(crate) fn get_global_import_attributes_type_checked(&mut self) -> Arc<Type> { ::tsox_core::fntrace::enter("get_global_import_attributes_type_checked"); 
        self.get_global_import_type_checked("ImportAttributes")
    }

    pub(crate) fn get_flow_type_of_reference_ex(
        &mut self,
        reference: &Arc<Node>,
        declared_type: &Arc<Type>,
        narrowable_type: Option<&Arc<Type>>,
        container: Option<&Arc<Node>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_flow_type_of_reference_ex"); 
        if self.flow_analysis_disabled {
            return self.error_type();
        }
        let Some(flow) = self
            .program
            .symbol_map()
            .flow_node_of(reference)
            .map(Arc::clone)
        else {
            return Arc::clone(declared_type);
        };
        let initial_type = narrowable_type.unwrap_or(declared_type);
        let target = FlowRef::Node(Arc::clone(reference));
        let mut query = FlowQuery {
            reference: Some(Arc::clone(reference)),
            flow_container: container.map(Arc::clone),
            ..FlowQuery::default()
        };
        let evolved_type =
            self.type_at_flow_node(declared_type, initial_type, &flow, &target, 0, &mut query);
        let result_type = if evolved_type.object_flags.contains(ObjectFlags::EvolvingArray)
            && self.is_evolving_array_operation_target(reference)
        {
            self.auto_array_type()
        } else {
            self.finalize_evolving_array_type(&evolved_type)
        };
        let unreachable_never = Arc::ptr_eq(&result_type, &self.unreachable_never_type);
        let non_null_never = reference
            .parent()
            .is_some_and(|p| p.kind == SyntaxKind::NonNullExpression)
            && !result_type.flags.contains(TypeFlags::Never)
            && self.type_is_never_after_removing_nullable(&result_type);
        if unreachable_never || non_null_never {
            return Arc::clone(declared_type);
        }
        result_type
    }

    pub(crate) fn check_await_expression(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("check_await_expression"); 
        let expression = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
        let t = self.check_expression_ex(&expression, CheckMode::Normal);
        self.get_awaited_type(&t).unwrap_or_else(|| Arc::clone(&t))
    }

    pub(crate) fn check_spread_expression(&mut self, node: &Arc<Node>, _check_mode: CheckMode) -> Arc<Type> { ::tsox_core::fntrace::enter("check_spread_expression"); 
        let expression = node.expression().cloned().unwrap_or_else(|| Arc::clone(node));
        let t = self.check_expression_ex(&expression, CheckMode::Normal);
        self.get_non_nullable_type_if_needed(&t)
    }

    pub(crate) fn check_return_expression(
        &mut self,
        _node: &Arc<Node>,
        return_type: &Arc<Type>,
        expression: &Arc<Node>,
        _body: &Arc<Node>,
        expression_type: &Arc<Type>,
        _is_async: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("check_return_expression"); 
        self.check_type_assignable_to_and_optionally_elaborate(
            expression_type,
            return_type,
            Some(expression),
            None,
            None,
            None,
        );
        Arc::clone(expression_type)
    }

    pub(crate) fn check_decorators(&mut self, _node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_decorators"); }

    pub(crate) fn check_if_statement(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("check_if_statement"); 
        if self.check_grammar_statement_in_ambient_context(node) {
            return;
        }
        let (expression, then_statement, else_statement) = match &node.data {
            NodeData::IfStatement(data) => (
                data.expression.clone(),
                data.then_statement.clone(),
                data.else_statement.clone(),
            ),
            _ => return,
        };
        let _expr_type = self.check_truthiness_expression(&expression, CheckMode::Normal);
        self.check_source_element(&then_statement);
        if let Some(else_statement) = else_statement {
            self.check_source_element(&else_statement);
        }
    }

    pub(crate) fn add_related_info(&mut self, _node: &Arc<Node>, _related: Diagnostic) { ::tsox_core::fntrace::enter("add_related_info"); }

    pub(crate) fn add_related_info_to_diagnostic(&mut self, _diagnostic: &Diagnostic, _related: Diagnostic) { ::tsox_core::fntrace::enter("add_related_info_to_diagnostic"); }

    pub(crate) fn check_type_parameters(&mut self, type_parameters: Option<&NodeList>) { ::tsox_core::fntrace::enter("check_type_parameters"); 
        if let Some(type_parameters) = type_parameters {
            for type_parameter in type_parameters.iter() {
                self.check_node_deferred(&type_parameter);
            }
        }
    }

    pub(crate) fn check_object_type_for_duplicate_declarations(&mut self, _node: &Arc<Node>, _is_interface: bool) { ::tsox_core::fntrace::enter("check_object_type_for_duplicate_declarations"); }
}
