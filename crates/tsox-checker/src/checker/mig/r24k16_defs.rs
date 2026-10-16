#![allow(unused_imports)]

use std::cell::RefCell;
use std::sync::Arc;

use crate::checker::checker::*;
use crate::checker::checker_checker_checker::Checker;
use tsox_frontend::ast::node_data_generated::{is_parenthesized_type_node, is_type_operator_node};
use tsox_frontend::ast::{Node, Symbol, SyntaxKind};

use crate::checker::mig::m2a::r19k11_defs::R19K11NodeExt;

thread_local! {
    static CONTEXTUAL_BINDING_PATTERNS: RefCell<Vec<Arc<Node>>> = const { RefCell::new(Vec::new()) };
}

pub fn reset_contextual_binding_patterns_r24k16() {
    CONTEXTUAL_BINDING_PATTERNS.with(|v| v.borrow_mut().clear());
}

pub(crate) fn contextual_binding_patterns_push(node: Arc<Node>) { ::tsox_core::fntrace::enter("contextual_binding_patterns_push"); 
    CONTEXTUAL_BINDING_PATTERNS.with(|s| s.borrow_mut().push(node));
}

pub(crate) fn contextual_binding_patterns_pop() { ::tsox_core::fntrace::enter("contextual_binding_patterns_pop"); 
    CONTEXTUAL_BINDING_PATTERNS.with(|s| {
        s.borrow_mut().pop();
    });
}

pub(crate) fn pattern_for_type_insert(t: Arc<Type>, node: Arc<Node>) { ::tsox_core::fntrace::enter("pattern_for_type_insert");
    crate::checker::mig::wc1c::r24k17_defs::set_pattern_for_type(&t, node);
}

impl Checker {
    pub(crate) fn is_function_object_type_fwd(&mut self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_function_object_type_fwd"); 
        if let Some(sym) = self.get_global_type_by_name("Function")
            && let Some(target) = sym.target().cloned()
        {
            return self.has_base_type(t, &target) || Arc::ptr_eq(t, &sym);
        }
        false
    }

    pub(crate) fn get_alias_symbol_for_type_node(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_alias_symbol_for_type_node"); 
        let mut host = node.parent()?;
        loop {
            let next = if is_parenthesized_type_node(&host)
                || (is_type_operator_node(&host)
                    && matches!(
                        &host.data,
                        tsox_frontend::ast::NodeData::TypeOperatorNode(d)
                            if d.operator == SyntaxKind::ReadonlyKeyword
                    ))
            {
                host.parent()
            } else {
                None
            };
            match next {
                Some(h) => host = h,
                None => break,
            }
        }
        if host.kind == SyntaxKind::TypeAliasDeclaration {
            return self.get_symbol_of_declaration(&host);
        }
        None
    }
}
