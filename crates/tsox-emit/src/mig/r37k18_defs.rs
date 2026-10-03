use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::visitor::NodeVisitor;

use crate::printer::NodeFactory;

pub trait R37K18NodeVisitorExt {
    fn visit_each_child(&mut self, node: &Arc<Node>) -> Arc<Node>;
}

impl R37K18NodeVisitorExt for NodeVisitor {
    fn visit_each_child(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_each_child"); 
        tsox_frontend::ast::mig::m3c::visit_each_child(node, &mut r37k18_m3c_visitor())
    }
}

fn r37k18_m3c_visitor() -> tsox_frontend::ast::mig::m3c::NodeVisitor { ::tsox_core::fntrace::enter("r37k18_m3c_visitor"); 
    tsox_frontend::ast::mig::m3c::NodeVisitor {
        factory: tsox_frontend::ast::mig::m3c::NodeFactory {
            hooks: tsox_frontend::ast::mig::m3c::NodeFactoryHooks::default(),
            text_count: 0,
            node_count: 0,
        },
    }
}

impl<'a> NodeFactory<'a> {
    pub fn new_global_method_call(
        &self,
        global_object_name: &str,
        method_name: &str,
        arguments_list: &[Arc<Node>],
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_global_method_call"); 
        self.new_method_call(
            &self.new_identifier(global_object_name),
            &self.new_identifier(method_name),
            arguments_list.to_vec(),
        )
    }

    pub fn new_big_int_literal(&self, text: &str, token_flags: i32) -> Arc<Node> { ::tsox_core::fntrace::enter("new_big_int_literal"); 
        Arc::new(Node::new(
            tsox_frontend::ast::SyntaxKind::BigIntLiteral,
            tsox_frontend::ast::NodeData::BigIntLiteral(
                tsox_frontend::ast::node_data_generated::BigIntLiteralData {
                    text: text.to_string(),
                    token_flags: token_flags as u32,
                },
            ),
        ))
    }
}
