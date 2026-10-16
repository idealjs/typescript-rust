#![allow(dead_code, unused_imports)]

use std::sync::Arc;

use tsox_checker::checker::mig::m2d::EmitResolver;
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

pub trait R39K03EmitResolverExt {
    fn set_referenced_import_declaration(&self, node: &Arc<Node>, import_ref: Option<Arc<Node>>);
}

impl R39K03EmitResolverExt for EmitResolver {
    fn set_referenced_import_declaration(&self, node: &Arc<Node>, import_ref: Option<Arc<Node>>) { ::tsox_core::fntrace::enter("set_referenced_import_declaration"); 
        tsox_checker::checker::mig::m2c_5::r26k4_defs::set_jsx_links_import_ref(node, import_ref);
    }
}

pub trait R39K03NodeExt {
    fn is_declaration_file(&self) -> bool;
    fn as_named_imports(&self) -> &tsox_frontend::ast::node_data_generated::NamedImportsData;
}

impl R39K03NodeExt for Node {
    fn is_declaration_file(&self) -> bool { ::tsox_core::fntrace::enter("is_declaration_file"); 
        let _ = self.kind == SyntaxKind::SourceFile;
        false
    }

    fn as_named_imports(&self) -> &tsox_frontend::ast::node_data_generated::NamedImportsData { ::tsox_core::fntrace::enter("as_named_imports"); 
        match &self.data {
            NodeData::NamedImports(d) => d,
            _ => panic!("AsNamedImports on wrong node kind"),
        }
    }
}
