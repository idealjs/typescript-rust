#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_flags::{NodeFlags, NodeFlags as NF};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

use crate::mig::m4q::r33k12_defs::RuntimeSyntaxTransformer;
use crate::mig::m4n_4::NameOptions;
use tsox_frontend::ast::mig::m3g_2::is_parameter_property_declaration;
use crate::mig::m4q::r33k12_defs::{EmitFlags, NodeFactory};

impl RuntimeSyntaxTransformer {
    pub(crate) fn get_namespace_container_name(&self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("get_namespace_container_name"); 
        let factory = self.factory();
        factory.generated_name_node(&factory.new_generated_name_for_node(node))
    }

    pub(crate) fn get_namespace_qualified_property(
        &self,
        ns: Arc<Node>,
        name: Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("get_namespace_qualified_property"); 
        self.factory().get_namespace_member_name(
            &ns,
            &name,
            NameOptions {
                allow_comments: false,
                allow_source_maps: true,
            },
        )
    }

    pub(crate) fn get_namespace_qualified_element(
        &mut self,
        ns: Arc<Node>,
        expression: Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("get_namespace_qualified_element"); 
        let qualified_name = self.emit_context().factory().new_element_access_expression(
            &ns,
            None,
            &expression,
            NodeFlags::empty(),
        );
        self.emit_context()
            .assign_comment_and_source_map_ranges(&qualified_name, &expression);
        qualified_name
    }

    pub(crate) fn get_parameter_properties(
        &self,
        constructor: Option<&Arc<Node>>,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_parameter_properties"); 
        let mut parameter_properties = Vec::new();
        if let Some(constructor) = constructor {
            for parameter in tsox_frontend::ast::mig::m3b::parameters(constructor) {
                if is_parameter_property_declaration(parameter, constructor) {
                    parameter_properties.push(Arc::clone(parameter));
                }
            }
        }
        parameter_properties
    }
}
