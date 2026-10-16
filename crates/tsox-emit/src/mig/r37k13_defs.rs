#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::mig::m3c;
use tsox_frontend::ast::mig::m3c::{NodeFactory as M3cNodeFactory, NodeFactoryHooks as M3cNodeFactoryHooks};
use tsox_frontend::ast::node::Node;
use tsox_frontend::format::mig::m4o::{new_node_factory, EmitContext, EmitFlags, NodeFactory};

use crate::mig::m4f_3::ClassFieldsTransformer as M4f3ClassFieldsTransformer;
use crate::mig::m4g::r33k7_defs::ReferenceResolver;
use crate::mig::m4g::ClassFieldsTransformer;

pub struct K13Visitor {
    inner: tsox_frontend::ast::mig::m3c::NodeVisitor,
}

impl Default for K13Visitor {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        Self {
            inner: tsox_frontend::ast::mig::m3c::NodeVisitor {
                factory: M3cNodeFactory {
                    hooks: M3cNodeFactoryHooks::default(),
                    text_count: 0,
                    node_count: 0,
                },
            },
        }
    }
}

impl K13Visitor {
    pub fn visit_each_child(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_each_child"); 
        m3c::visit_each_child(node, &mut self.inner)
    }

    pub fn visit_node(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_node"); 
        m3c_visit_node_r37k13(node)
    }

    pub fn visit_node_opt(&mut self, node: Option<&Arc<Node>>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_node_opt"); 
        node.cloned()
    }
}

pub trait ClassFieldsTransformerR37k13 {
    fn emit_context(&self) -> crate::printer::EmitContext;
    fn factory(&self) -> NodeFactory;
    fn visitor(&mut self) -> K13Visitor;
    fn resolver(&self) -> Arc<dyn ReferenceResolver>;
    fn modifier_visitor(&mut self) -> K13Visitor;
    fn discarded_value_visitor(&mut self) -> K13Visitor;
    fn heritage_clause_visitor(&mut self) -> K13Visitor;
    fn assignment_target_visitor(&mut self) -> K13Visitor;
    fn class_element_visitor(&mut self) -> K13Visitor;
    fn accessor_field_result_visitor(&mut self) -> K13Visitor;
    fn substitution_visitor(&mut self) -> K13Visitor;
    fn visit_pre_or_postfix_unary_expression(&mut self, node: &Arc<Node>, discarded: bool) -> Arc<Node>;
    fn visit_assignment_pattern(&mut self, node: &Arc<Node>) -> Arc<Node>;
    fn wrap_private_identifier_for_destructuring_target(&mut self, node: &Arc<Node>) -> Arc<Node>;
    fn visit_invalid_super_property(&mut self, node: &Arc<Node>) -> Arc<Node>;
    fn visit_expression_with_type_arguments_in_heritage_clause(&mut self, node: &Arc<Node>) -> Arc<Node>;
}

impl ClassFieldsTransformerR37k13 for ClassFieldsTransformer {
    fn emit_context(&self) -> crate::printer::EmitContext { ::tsox_core::fntrace::enter("emit_context"); 
        crate::printer::EmitContext::new()
    }

    fn factory(&self) -> NodeFactory { ::tsox_core::fntrace::enter("factory"); 
        new_node_factory(EmitContext::default())
    }

    fn visitor(&mut self) -> K13Visitor { ::tsox_core::fntrace::enter("visitor"); 
        K13Visitor::default()
    }

    fn resolver(&self) -> Arc<dyn ReferenceResolver> { ::tsox_core::fntrace::enter("resolver"); 
        self.resolver.clone()
    }

    fn modifier_visitor(&mut self) -> K13Visitor { ::tsox_core::fntrace::enter("modifier_visitor"); 
        K13Visitor::default()
    }

    fn discarded_value_visitor(&mut self) -> K13Visitor { ::tsox_core::fntrace::enter("discarded_value_visitor"); 
        K13Visitor::default()
    }

    fn heritage_clause_visitor(&mut self) -> K13Visitor { ::tsox_core::fntrace::enter("heritage_clause_visitor"); 
        K13Visitor::default()
    }

    fn assignment_target_visitor(&mut self) -> K13Visitor { ::tsox_core::fntrace::enter("assignment_target_visitor"); 
        K13Visitor::default()
    }

    fn class_element_visitor(&mut self) -> K13Visitor { ::tsox_core::fntrace::enter("class_element_visitor"); 
        K13Visitor::default()
    }

    fn accessor_field_result_visitor(&mut self) -> K13Visitor { ::tsox_core::fntrace::enter("accessor_field_result_visitor"); 
        K13Visitor::default()
    }

    fn substitution_visitor(&mut self) -> K13Visitor { ::tsox_core::fntrace::enter("substitution_visitor"); 
        K13Visitor::default()
    }

    fn visit_pre_or_postfix_unary_expression(&mut self, node: &Arc<Node>, _discarded: bool) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_pre_or_postfix_unary_expression"); 
        self.visitor().visit_each_child(node)
    }

    fn visit_assignment_pattern(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_assignment_pattern"); 
        self.visitor().visit_each_child(node)
    }

    fn wrap_private_identifier_for_destructuring_target(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("wrap_private_identifier_for_destructuring_target"); 
        self.visitor().visit_each_child(node)
    }

    fn visit_invalid_super_property(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_invalid_super_property"); 
        self.visitor().visit_each_child(node)
    }

    fn visit_expression_with_type_arguments_in_heritage_clause(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_expression_with_type_arguments_in_heritage_clause"); 
        self.visitor().visit_each_child(node)
    }
}

pub trait M4f3ClassFieldsTransformerR37k13 {
    fn factory(&self) -> NodeFactory;
    fn visitor(&mut self) -> K13Visitor;
    fn member_contains_constructor_reference(&self, member: &Arc<Node>, class_decl: &Arc<Node>) -> bool;
}

impl M4f3ClassFieldsTransformerR37k13 for M4f3ClassFieldsTransformer<'_> {
    fn factory(&self) -> NodeFactory { ::tsox_core::fntrace::enter("factory"); 
        new_node_factory(EmitContext::default())
    }

    fn visitor(&mut self) -> K13Visitor { ::tsox_core::fntrace::enter("visitor"); 
        K13Visitor::default()
    }

    fn member_contains_constructor_reference(&self, _member: &Arc<Node>, _class_decl: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("member_contains_constructor_reference"); 
        false
    }
}

pub trait NodeFactoryR37k13 {
    fn new_temp_variable_r37k13(&self) -> Arc<Node>;
}

impl NodeFactoryR37k13 for NodeFactory {
    fn new_temp_variable_r37k13(&self) -> Arc<Node> { ::tsox_core::fntrace::enter("new_temp_variable_r37k13"); 
        self.new_identifier("")
    }
}

pub trait EmitContextR37k13 {
    fn read_emit_helpers(&self) -> Vec<Arc<tsox_frontend::format::mig::m4o_2::EmitHelper>>;
    fn set_original_r37k13(&self, node: &Arc<Node>, original: &Arc<Node>);
}

impl EmitContextR37k13 for crate::printer::EmitContext {
    fn read_emit_helpers(&self) -> Vec<Arc<tsox_frontend::format::mig::m4o_2::EmitHelper>> { ::tsox_core::fntrace::enter("read_emit_helpers"); 
        Vec::new()
    }

    fn set_original_r37k13(&self, _node: &Arc<Node>, _original: &Arc<Node>) { ::tsox_core::fntrace::enter("set_original_r37k13"); }
}

pub fn node_members_r37k13(node: &Arc<Node>) -> &[Arc<Node>] { ::tsox_core::fntrace::enter("node_members_r37k13"); 
    crate::mig::m4g::r33k7_defs::class_members(node)
}

pub trait NodeR37k13Ext {
    fn flags_r37k13(&self) -> u32;
    fn referenced_files_r37k13(&self) -> &[Arc<Node>];
    fn type_reference_directives_r37k13(&self) -> &[Arc<Node>];
    fn lib_reference_directives_r37k13(&self) -> &[Arc<Node>];
}

impl NodeR37k13Ext for Node {
    fn flags_r37k13(&self) -> u32 { ::tsox_core::fntrace::enter("flags_r37k13"); 
        0
    }

    fn referenced_files_r37k13(&self) -> &[Arc<Node>] { ::tsox_core::fntrace::enter("referenced_files_r37k13"); 
        &[]
    }

    fn type_reference_directives_r37k13(&self) -> &[Arc<Node>] { ::tsox_core::fntrace::enter("type_reference_directives_r37k13"); 
        &[]
    }

    fn lib_reference_directives_r37k13(&self) -> &[Arc<Node>] { ::tsox_core::fntrace::enter("lib_reference_directives_r37k13"); 
        &[]
    }
}

pub const EF_NO_NESTED_SOURCE_MAPS_R37K13: EmitFlags = EmitFlags::NO_NESTED_SOURCE_MAPS;

fn m3c_visit_node_r37k13(node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("m3c_visit_node_r37k13"); 
    node.clone()
}
