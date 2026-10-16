#![allow(unused_imports)]
use crate::checker::mig::m2a::r19k11_defs::*;
use tsox_frontend::ast::mig::w2::create_modifiers_from_modifier_flags;
use crate::checker::checker::Checker;

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use tsox_frontend::ast::{Node, Symbol, SymbolFlags};

#[path = "r17k8_factory_ext.rs"]
pub mod r17k8_factory_ext;
pub use r17k8_factory_ext::{replace_modifiers, NodeFactoryExt};

#[path = "r24k12_defs.rs"]
pub mod r24k12_defs;
pub use r24k12_defs::symbol_format_flags_to_node_builder_flags;

use crate::checker::mig::m2b::r22k6_defs::{node_modifier_flags, R22K6NodeFactoryExt};
use crate::checker::nodecopy_builder::EmitContextStub;
pub use crate::checker::nodecopy_builder::NodeBuilderImpl;
use crate::checker::symboltracker::SharedNodeBuilderContext;

use crate::checker::symboltracker::{
    NodeBuilderContext, NodeBuilderFlags, NodeBuilderInternalFlags, SymbolTracker,
    SymbolTrackerImpl,
};
use crate::checker::types::{Type, TypePredicate};

pub struct VerbosityContext {
    pub level: i32,
    pub max_truncation_length: usize,
    pub can_increase_verbosity: bool,
    pub truncated: bool,
}

impl Default for VerbosityContext {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        VerbosityContext {
            level: 0,
            max_truncation_length: 0,
            can_increase_verbosity: false,
            truncated: false,
        }
    }
}

pub struct NodeBuilder<'a> {
    pub ctx_stack: Vec<SharedNodeBuilderContext>,
    pub impl_: NodeBuilderImpl<'a>,
    pub verbosity: Option<VerbosityContext>,
}

pub fn new_emit_context() -> EmitContextStub { ::tsox_core::fntrace::enter("new_emit_context"); 
    EmitContextStub::default()
}

pub fn new_node_builder_ex<'a>(
    ch: &'a Checker,
    _e: EmitContextStub,
    id_to_symbol: HashMap<u64, Arc<Symbol>>,
) -> NodeBuilder<'a> { ::tsox_core::fntrace::enter("new_node_builder_ex"); 
    crate::checker::mig::m2c_5::r26k4_defs::set_builder_checker(ch);
    NodeBuilder {
        ctx_stack: Vec::new(),
        impl_: NodeBuilderImpl::new(ch, id_to_symbol),
        verbosity: None,
    }
}

impl<'a> NodeBuilder<'a> {
    pub fn emit_context(&self) -> &EmitContextStub { ::tsox_core::fntrace::enter("emit_context"); 
        &self.impl_.e
    }

    pub fn enter_context(
        &mut self,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        tracker: Option<Box<dyn SymbolTracker>>,
    ) { ::tsox_core::fntrace::enter("enter_context"); 
        let mut verbosity_level = -1;
        let mut max_truncation_length = 0;
        if let Some(verbosity) = &self.verbosity {
            verbosity_level = verbosity.level;
            max_truncation_length = verbosity.max_truncation_length;
        }
        self.ctx_stack.push(Rc::clone(&self.impl_.ctx));
        let mut ctx = NodeBuilderContext {
            flags,
            internal_flags,
            max_expansion_depth: verbosity_level,
            max_truncation_length,
            enclosing_declaration: enclosing_declaration.cloned(),
            enclosing_file: enclosing_declaration
                .and_then(|d| self.impl_.ch.get_source_file_of_node(d)),
            ..NodeBuilderContext::default()
        };
        self.impl_.ctx = Rc::new(std::cell::RefCell::new(ctx));
        let tracker = SymbolTrackerImpl::new(Rc::clone(&self.impl_.ctx), tracker);
        self.impl_.ctx.borrow_mut().tracker = Some(Box::new(tracker));
    }

    pub fn propagate_verbosity_out(&mut self) { ::tsox_core::fntrace::enter("propagate_verbosity_out"); 
        let can_increase = self.impl_.ctx.borrow().can_increase_expansion_depth;
        let truncated = self.impl_.ctx.borrow().expansion_truncated;
        if let Some(verbosity) = &mut self.verbosity {
            if can_increase {
                verbosity.can_increase_verbosity = true;
            }
            if truncated {
                verbosity.truncated = true;
            }
        }
    }

    pub fn pop_context(&mut self) { ::tsox_core::fntrace::enter("pop_context"); 
        if self.ctx_stack.is_empty() {
            self.impl_.ctx = Rc::new(std::cell::RefCell::new(NodeBuilderContext::default()));
        } else {
            self.impl_.ctx = self.ctx_stack.pop().unwrap();
        }
    }

    pub fn exit_context(&mut self, result: Option<Arc<Node>>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("exit_context"); 
        self.propagate_verbosity_out();
        self.exit_context_check();
        let encountered_error = self.impl_.ctx.borrow().encountered_error;
        let out = if encountered_error { None } else { result };
        self.pop_context();
        out
    }

    pub fn exit_context_slice(&mut self, result: Vec<Arc<Node>>) -> Option<Vec<Arc<Node>>> { ::tsox_core::fntrace::enter("exit_context_slice"); 
        self.propagate_verbosity_out();
        self.exit_context_check();
        let encountered_error = self.impl_.ctx.borrow().encountered_error;
        let out = if encountered_error { None } else { Some(result) };
        self.pop_context();
        out
    }

    pub fn exit_context_check(&mut self) { ::tsox_core::fntrace::enter("exit_context_check"); 
        let (truncating, no_truncation) = {
            let ctx = self.impl_.ctx.borrow();
            (ctx.truncating, ctx.flags.contains(NodeBuilderFlags::NoTruncation))
        };
        if truncating && no_truncation {
            if let Some(tracker) = self.impl_.ctx.borrow_mut().tracker.as_mut() {
                tracker.report_truncation_error();
            }
        }
    }
    pub fn symbol_to_expression(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        tracker: Option<Box<dyn SymbolTracker>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("symbol_to_expression"); 
        self.enter_context(enclosing_declaration, flags, internal_flags, tracker);
        let result = self.impl_.symbol_to_expression(symbol, meaning);
        self.exit_context(Some(result))
    }

    pub fn symbol_to_node(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        tracker: Option<Box<dyn SymbolTracker>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("symbol_to_node"); 
        self.enter_context(enclosing_declaration, flags, internal_flags, tracker);
        let result = self.impl_.symbol_to_node(symbol, meaning);
        self.exit_context(Some(result))
    }

    pub fn symbol_to_parameter_declaration(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        tracker: Option<Box<dyn SymbolTracker>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("symbol_to_parameter_declaration"); 
        self.enter_context(enclosing_declaration, flags, internal_flags, tracker);
        let result = self.impl_.symbol_to_parameter_declaration(symbol, false);
        self.exit_context(Some(result))
    }

    pub fn symbol_to_type_parameter_declarations(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        tracker: Option<Box<dyn SymbolTracker>>,
    ) -> Option<Vec<Arc<Node>>> { ::tsox_core::fntrace::enter("symbol_to_type_parameter_declarations"); 
        self.enter_context(enclosing_declaration, flags, internal_flags, tracker);
        let result = self.impl_.symbol_to_type_parameter_declarations(symbol);
        self.exit_context_slice(result.unwrap_or_default())
    }

    pub fn type_parameter_to_declaration(
        &mut self,
        parameter: &Arc<Type>,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        tracker: Option<Box<dyn SymbolTracker>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("type_parameter_to_declaration"); 
        self.enter_context(enclosing_declaration, flags, internal_flags, tracker);
        let result = self.impl_.type_parameter_to_declaration(parameter);
        self.exit_context(Some(result))
    }

    pub fn type_predicate_to_type_predicate_node(
        &mut self,
        predicate: &TypePredicate,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        tracker: Option<Box<dyn SymbolTracker>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("type_predicate_to_type_predicate_node"); 
        self.enter_context(enclosing_declaration, flags, internal_flags, tracker);
        let result = self.impl_.type_predicate_to_type_predicate_node(predicate);
        self.exit_context(Some(result))
    }
    pub fn try_js_type_node_to_type_node(
        &mut self,
        node: &Arc<Node>,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: NodeBuilderFlags,
        internal_flags: NodeBuilderInternalFlags,
        tracker: Option<Box<dyn SymbolTracker>>,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("try_js_type_node_to_type_node"); 
        self.enter_context(enclosing_declaration, flags, internal_flags, tracker);
        let result = self.impl_.try_js_type_node_to_type_node(Some(node));
        self.exit_context(result)
    }
}

pub fn simplify_class_declaration(
    f: &crate::checker::nodecopy_builder::NodeFactoryStub,
    class_decl: Arc<Node>,
    symbol: &Arc<Symbol>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("simplify_class_declaration"); 
    let class_declarations: Vec<&Arc<Node>> = symbol
        .declarations
        .iter()
        .filter(|d| tsox_frontend::ast::is_class_like(d))
        .collect();
    let original_class_decl = class_declarations.first().copied().unwrap_or(&class_decl);
    let modifiers = tsox_frontend::ast::ModifierFlags::Export
        | tsox_frontend::ast::ModifierFlags::Ambient;
    let modifiers = node_modifier_flags(original_class_decl) - modifiers;
    let is_anonymous = tsox_frontend::ast::is_class_expression(original_class_decl);
    let class_decl = if is_anonymous {
        let cd = r24k12_defs::as_class_declaration(&class_decl);
        f.update_class_declaration(
            &class_decl,
            class_decl.modifiers().cloned(),
            None,
            cd.type_parameters.clone(),
            cd.heritage_clauses.clone(),
            cd.members.clone(),
        )
    } else {
        class_decl
    };
    replace_modifiers(
        f,
        &class_decl,
        f.new_modifier_list(create_modifiers_from_modifier_flags(
            modifiers,
            |k| f.new_modifier(k),
        )),
    )
}

pub fn simplify_modifiers(
    f: &crate::checker::nodecopy_builder::NodeFactoryStub,
    new_decl: Arc<Node>,
    is_decl_kind: fn(&Arc<Node>) -> bool,
    symbol: &Arc<Symbol>,
) -> Arc<Node> { ::tsox_core::fntrace::enter("simplify_modifiers"); 
    let decls: Vec<&Arc<Node>> = symbol
        .declarations
        .iter()
        .filter(|d| is_decl_kind(d))
        .collect();
    let decl_with_modifiers = decls.first().copied().unwrap_or(&new_decl);
    let modifiers = tsox_frontend::ast::ModifierFlags::Export
        | tsox_frontend::ast::ModifierFlags::Ambient;
    let modifiers = node_modifier_flags(decl_with_modifiers) - modifiers;
    replace_modifiers(
        f,
        &new_decl,
        f.new_modifier_list(create_modifiers_from_modifier_flags(
            modifiers,
            |k| f.new_modifier(k),
        )),
    )
}

impl Checker {
    pub fn get_node_builder(&mut self) -> (NodeBuilder<'_>, impl FnOnce() + use<'_>) { ::tsox_core::fntrace::enter("get_node_builder"); 
        let builder = self.get_node_builder_ex(HashMap::new());
        (builder, || {})
    }

    pub fn get_node_builder_ex(
        &mut self,
        id_to_symbol: HashMap<u64, Arc<Symbol>>,
    ) -> NodeBuilder<'_> { ::tsox_core::fntrace::enter("get_node_builder_ex"); 
        new_node_builder_ex(self, new_emit_context(), id_to_symbol)
    }
}
