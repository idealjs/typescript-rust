#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::format::mig::m4o_2::WriteKind;

use crate::mig::m4p::Printer;
use crate::mig::m4p_5::r36k12_defs::LF_NAMED_IMPORTS_OR_EXPORTS_ELEMENTS;

impl Printer {
    pub fn emit_module_export_name(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_module_export_name"); 
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_name(node),
            SyntaxKind::StringLiteral => self.emit_string_literal(node),
            _ => panic!("unexpected ModuleExportName: {:?}", node.kind),
        }
    }

    pub fn emit_module_reference(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_module_reference"); 
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_reference(node),
            SyntaxKind::QualifiedName => self.emit_qualified_name(node),
            SyntaxKind::ExternalModuleReference => self.emit_external_module_reference(node),
            _ => panic!("unhandled ModuleReference: {:?}", node.kind),
        }
    }

    pub fn emit_qualified_name(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_qualified_name"); 
        let state = self.enter_node(node);
        let (left, right) = match &node.data {
            NodeData::QualifiedName(d) => (d.left.clone(), d.right.clone()),
            _ => panic!("unexpected QualifiedName: {:?}", node.kind),
        };
        self.emit_entity_name(&left);
        self.write_punctuation(".");
        self.emit_member_name(&right);
        self.exit_node(node, state);
    }

    pub fn emit_member_name(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_member_name"); 
        match node.kind {
            SyntaxKind::Identifier => self.emit_identifier_name(node),
            SyntaxKind::PrivateIdentifier => self.emit_private_identifier(node),
            _ => panic!("unexpected MemberName: {:?}", node.kind),
        }
    }

    pub fn emit_namespace_import(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_namespace_import"); 
        let state = self.enter_node(node);
        let name = match &node.data {
            NodeData::NamespaceImport(d) => d.name.clone(),
            _ => panic!("unexpected NamespaceImport: {:?}", node.kind),
        };
        let pos = self.emit_token(SyntaxKind::AsteriskToken, node.pos(), WriteKind::Punctuation, node);
        self.write_space();
        self.emit_token(SyntaxKind::AsKeyword, pos, WriteKind::Keyword, node);
        self.write_space();
        self.emit_binding_identifier(&name);
        self.exit_node(node, state);
    }

    pub fn emit_named_imports(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_named_imports"); 
        let state = self.enter_node(node);
        let elements = match &node.data {
            NodeData::NamedImports(d) => d.elements.clone(),
            _ => panic!("unexpected NamedImports: {:?}", node.kind),
        };
        self.write_punctuation("{");
        self.emit_list(
            Self::emit_import_specifier_node,
            node,
            &elements,
            LF_NAMED_IMPORTS_OR_EXPORTS_ELEMENTS,
        );
        self.write_punctuation("}");
        self.exit_node(node, state);
    }

    pub fn emit_named_import_bindings(&mut self, node: Option<&Arc<Node>>) { ::tsox_core::fntrace::enter("emit_named_import_bindings"); 
        let Some(node) = node else {
            return;
        };
        match node.kind {
            SyntaxKind::NamespaceImport => self.emit_namespace_import(node),
            SyntaxKind::NamedImports => self.emit_named_imports(node),
            _ => panic!("unhandled NamedImportBindings: {:?}", node.kind),
        }
    }

    pub fn emit_namespace_export(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_namespace_export"); 
        let state = self.enter_node(node);
        let name = match &node.data {
            NodeData::NamespaceExport(d) => d.name.clone(),
            _ => panic!("unexpected NamespaceExport: {:?}", node.kind),
        };
        let pos = self.emit_token(SyntaxKind::AsteriskToken, node.pos(), WriteKind::Punctuation, node);
        self.write_space();
        self.emit_token(SyntaxKind::AsKeyword, pos, WriteKind::Keyword, node);
        self.write_space();
        self.emit_module_export_name(&name);
        self.exit_node(node, state);
    }

    pub fn emit_named_exports(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_named_exports"); 
        let state = self.enter_node(node);
        let elements = match &node.data {
            NodeData::NamedExports(d) => d.elements.clone(),
            _ => panic!("unexpected NamedExports: {:?}", node.kind),
        };
        self.write_punctuation("{");
        self.emit_list(
            Self::emit_export_specifier_node,
            node,
            &elements,
            LF_NAMED_IMPORTS_OR_EXPORTS_ELEMENTS,
        );
        self.write_punctuation("}");
        self.exit_node(node, state);
    }

    pub fn emit_named_export_bindings(&mut self, node: &Arc<Node>) { ::tsox_core::fntrace::enter("emit_named_export_bindings"); 
        match node.kind {
            SyntaxKind::NamespaceExport => self.emit_namespace_export(node),
            SyntaxKind::NamedExports => self.emit_named_exports(node),
            _ => panic!("unhandled NamedExportBindings: {:?}", node.kind),
        }
    }
}
