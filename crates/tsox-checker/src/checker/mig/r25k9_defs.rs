#![allow(unused_imports)]

#[allow(unused_imports)]
use crate::checker::*;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
#[allow(unused_imports)]
use std::sync::Arc;

pub(crate) use crate::checker::Checker;

use crate::checker::nodebuilder_type_format_flags_2::TypeFormatFlags;
use crate::checker::types::IterationTypeKind;
use tsox_frontend::ast::Symbol;

pub(crate) fn no_type_reduction_flags() -> TypeFormatFlags { ::tsox_core::fntrace::enter("no_type_reduction_flags"); 
    unsafe { std::mem::transmute::<u32, TypeFormatFlags>(1 << 29) }
}

impl Checker {
    pub fn get_global_promise_constructor_symbol_or_nil(&mut self) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_global_promise_constructor_symbol_or_nil"); 
        self.get_global_symbol("Promise", tsox_frontend::ast::SymbolFlags::VALUE, None)
    }

    pub fn is_iterator_result(&mut self, t: &Arc<Type>, kind: IterationTypeKind) -> bool { ::tsox_core::fntrace::enter("is_iterator_result"); 
        let done_type = match self.get_type_of_property_of_type(t, "done") {
            Some(d) => d,
            None => self.false_type(),
        };
        let probe = if matches!(kind, IterationTypeKind::YIELD) {
            self.false_type()
        } else {
            self.true_type()
        };
        self.is_type_assignable_to(&probe, &done_type)
    }

    pub fn get_inference_context_arc(
        &self,
        node: &tsox_frontend::ast::Node,
    ) -> Option<&Arc<crate::checker::inference_inference_key_2::InferenceContext>> { ::tsox_core::fntrace::enter("get_inference_context_arc");
        for info in self.inference_context_infos.iter().rev() {
            if let Some(context) = info.context.as_ref()
                && info
                    .node
                    .as_ref()
                    .is_some_and(|n| crate::checker::mig::m1b::is_node_descendant_of(node, n))
            {
                return Some(context);
            }
        }
        None
    }

    pub fn arg_inference_context_of(
        &self,
        node: &tsox_frontend::ast::Node,
    ) -> Option<*mut crate::checker::inference_inference_key_2::InferenceContext> { ::tsox_core::fntrace::enter("arg_inference_context_of");
        for (n, ctx) in self.arg_inference_contexts.iter().rev() {
            // Go isNodeDescendantOf（utilities.go:966-973）从 node 自身起比较
            //（node == ancestor 为 true）；m1b 版从 parent 起步排他。入栈节点
            // 即实参节点本身，嵌套调用节点 == 实参节点（layerEffect(Foo,
            // effectGen(...)) 的 arg1）时须自匹配，否则 getInferenceContext
            // 永不命中、checker.go:9947 的 outerMapper 不可达
            if std::ptr::eq(node, Arc::as_ptr(n))
                || crate::checker::mig::m1b::is_node_descendant_of(node, n)
            {
                return Some(*ctx);
            }
        }
        None
    }
}
