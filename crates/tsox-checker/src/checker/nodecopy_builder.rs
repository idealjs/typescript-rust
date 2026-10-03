use crate::checker::nodecopy_recovery::RecoveryBoundary;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use tsox_frontend::ast::Node;
use tsox_frontend::ast::Symbol;
use tsox_frontend::ast::SymbolFlags;
use tsox_frontend::ast::SyntaxKind;

use crate::checker::checker::Checker;
use crate::checker::symboltracker::{NodeBuilderContext, SharedNodeBuilderContext};
use crate::checker::types::Type;

#[derive(Default)]
pub struct NodeBuilderLinks {
    pub fake_scope_for_signature_declaration: Option<String>,
}

#[derive(Default)]
pub struct NodeBuilderSymbolLinks {}

pub struct NodeBuilderImpl<'a> {
    pub f: NodeFactoryStub,

    pub ch: &'a Checker,

    pub e: EmitContextStub,

    pub pc: PseudoCheckerStub,

    pub ctx: SharedNodeBuilderContext,

    pub id_to_symbol: HashMap<u64, Arc<Symbol>>,

    pub clone_binding_name_visitor: Option<crate::checker::mig::r27k_defs::NodeVisitor>,
}

#[derive(Default)]
pub struct NodeFactoryStub;

#[derive(Default)]
pub struct EmitContextStub;

#[derive(Default)]
pub struct PseudoCheckerStub;

impl EmitContextStub {
    pub fn set_original(&self, _node: &Arc<Node>, _original: Option<&Arc<Node>>) { ::tsox_core::fntrace::enter("set_original"); }

    pub fn add_emit_flags(&self, _node: &Arc<Node>, _flags: u32) { ::tsox_core::fntrace::enter("add_emit_flags"); }

    pub fn most_original(&self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("most_original"); 
        Arc::clone(node)
    }
}

impl<'a> NodeBuilderImpl<'a> {
    pub fn new(ch: &'a Checker, id_to_symbol: HashMap<u64, Arc<Symbol>>) -> Self { ::tsox_core::fntrace::enter("new"); 
        let ctx = Rc::new(RefCell::new(NodeBuilderContext::default()));
        NodeBuilderImpl {
            f: NodeFactoryStub,
            ch,
            e: EmitContextStub,
            pc: PseudoCheckerStub,
            ctx,
            id_to_symbol,
            clone_binding_name_visitor: None,
        }
    }

    pub fn reuse_node(&mut self, node: Option<&Arc<Node>>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("reuse_node"); 
        let node = node?;
        self.try_reuse_existing_node_helper(node)
    }

    pub fn try_js_type_node_to_type_node(&mut self, node: Option<&Arc<Node>>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("try_js_type_node_to_type_node"); 
        self.reuse_node(node)
    }

    pub fn reuse_name(&mut self, node: Option<&Arc<Node>>, _is_method: bool) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("reuse_name"); 
        let res = self.reuse_node(node)?;

        Some(res)
    }

    pub fn reuse_type_node(&mut self, node: Option<&Arc<Node>>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("reuse_type_node"); 
        let node = node?;
        let r = self.reuse_node(Some(node));
        if let Some(ref r) = r {
            let ctx = self.ctx.borrow();
            if ctx.max_expansion_depth >= 0 && !ctx.can_increase_expansion_depth {
                drop(ctx);
                self.walk_node_for_expandability(node);
            }
            return r.clone().into();
        }

        None
    }

    pub fn walk_node_for_expandability(&mut self, _node: &Arc<Node>) { ::tsox_core::fntrace::enter("walk_node_for_expandability"); 
        let can_increase = self.ctx.borrow().can_increase_expansion_depth;
        if can_increase {
            return;
        }
    }

    pub fn create_recovery_boundary(&mut self) -> Rc<RefCell<RecoveryBoundary>> { ::tsox_core::fntrace::enter("create_recovery_boundary"); 
        let ctx = self.ctx.borrow();
        let bound = RecoveryBoundary {
            ctx: Rc::clone(&self.ctx),
            had_error: false,
            deferred_reports: Vec::new(),
            old_tracker: None,
            old_tracked_symbols: ctx.tracked_symbols.clone(),
            tracked_symbols: Vec::new(),
            old_encountered_error: ctx.encountered_error,
            old_approximate_length: ctx.approximate_length,
        };
        drop(ctx);

        Rc::new(RefCell::new(bound))
    }

    pub fn finalize_boundary(&mut self, bound: &Rc<RefCell<RecoveryBoundary>>) -> bool { ::tsox_core::fntrace::enter("finalize_boundary"); 
        let mut ctx = self.ctx.borrow_mut();

        ctx.encountered_error = bound.borrow().old_encountered_error;
        ctx.approximate_length = bound.borrow().old_approximate_length;
        drop(ctx);

        let had_error = bound.borrow().had_error;
        if had_error {
            return false;
        }

        let tracked = bound.borrow().tracked_symbols.clone();
        let mut ctx = self.ctx.borrow_mut();
        if let Some(ref mut tracker) = ctx.tracker {
            for a in &tracked {
                tracker.track_symbol(&a.symbol, a.enclosing_declaration.as_ref(), a.meaning);
            }
        }
        true
    }

    pub fn try_reuse_existing_node_helper(&mut self, _existing: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("try_reuse_existing_node_helper"); 
        let bound = self.create_recovery_boundary();

        self.finalize_boundary(&bound);
        None
    }

    pub fn get_module_specifier_override(&mut self, _parent: &Arc<Node>, _lit: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("get_module_specifier_override"); 
        String::new()
    }

    pub fn rewrite_module_specifier(&mut self, parent: &Arc<Node>, lit: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("rewrite_module_specifier"); 
        let new_name = self.get_module_specifier_override(parent, lit);
        if new_name.is_empty() {
            return Arc::clone(lit);
        }

        Arc::clone(lit)
    }

    pub fn get_enclosing_declaration_ignoring_fake_scope(&self) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_enclosing_declaration_ignoring_fake_scope"); 
        let enc = self.ctx.borrow().enclosing_declaration.clone();

        enc
    }

    pub fn set_text_range(&self, node: Arc<Node>, _range_node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("set_text_range"); 
        node
    }

    pub fn new_identifier(&mut self, _text: &str, symbol: Option<&Arc<Symbol>>) -> Arc<Node> { ::tsox_core::fntrace::enter("new_identifier"); 
        let node = Node::new(SyntaxKind::Identifier, tsox_frontend::ast::NodeData::Token);
        if let Some(sym) = symbol {
            let _ = sym;
        }
        Arc::new(node)
    }

    pub fn get_synthesized_deep_clone(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_synthesized_deep_clone"); 
        Some(Arc::clone(node))
    }

    pub fn get_synthesized_deep_clones(&mut self, nodes: &[Arc<Node>]) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_synthesized_deep_clones"); 
        nodes
            .iter()
            .filter_map(|n| self.get_synthesized_deep_clone(n))
            .collect()
    }

    pub fn deep_clone_node(&self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("deep_clone_node"); 
        Arc::clone(node)
    }

    pub fn get_type_from_type_node(
        &mut self,
        _node: &Arc<Node>,
        _ignore_errors: bool,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_type_from_type_node"); 
        None
    }

    pub fn type_to_type_node(&mut self, _t: &Arc<Type>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("type_to_type_node"); 
        None
    }

    pub fn serialize_type_name(
        &mut self,
        _node: &Arc<Node>,
        _is_type_query: bool,
        _type_arguments: Option<&[Arc<Node>]>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("serialize_type_name"); 
        None
    }

    pub fn can_reuse_existing_js_type_node(
        &mut self,
        _node: &Arc<Node>,
        _t: Option<&Arc<Type>>,
    ) -> bool { ::tsox_core::fntrace::enter("can_reuse_existing_js_type_node"); 
        true
    }

    pub fn check_type_expandability(&mut self, _t: &Arc<Type>) { ::tsox_core::fntrace::enter("check_type_expandability"); }

    pub fn enter_new_scope(
        &mut self,
        _node: &Arc<Node>,
        _params: Option<Vec<Arc<Symbol>>>,
        _type_params: Option<Vec<Arc<Type>>>,
        _arg1: Option<()>,
        _arg2: Option<()>,
    ) -> Box<dyn FnOnce()> { ::tsox_core::fntrace::enter("enter_new_scope"); 
        Box::new(|| {})
    }

    pub fn type_parameter_to_name(&mut self, _t: &Arc<Type>) -> Arc<Node> { ::tsox_core::fntrace::enter("type_parameter_to_name"); 
        let node = Node::new(SyntaxKind::Identifier, tsox_frontend::ast::NodeData::Token);
        Arc::new(node)
    }

    pub fn try_get_resolved_symbol_from_type_node(
        &mut self,
        _node: &Arc<Node>,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("try_get_resolved_symbol_from_type_node"); 
        None
    }

    pub fn lookup_symbol_chain(
        &mut self,
        symbol: &Arc<Symbol>,
        _meaning: SymbolFlags,
        _use_only_external_aliasing: bool,
    ) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("lookup_symbol_chain"); 
        vec![Arc::clone(symbol)]
    }

    pub fn get_specifier_for_module_symbol(&mut self, _symbol: &Arc<Symbol>, _mode: u32) -> String { ::tsox_core::fntrace::enter("get_specifier_for_module_symbol"); 
        String::new()
    }
}
