#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node::ModifierList;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node::NodeList;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;

pub struct TypedNode6<'a> {
    pub node: &'a Node,
}

impl<'a> TypedNode6<'a> {
    pub fn as_node(&self) -> &'a Node { ::tsox_core::fntrace::enter("as_node"); 
        self.node
    }
}

pub trait NodeAsExt {
    fn assert_kind(&self, kind: SyntaxKind) -> &Node;

    fn as_typed(&self) -> TypedNode6<'_>;


    fn as_identifier(&self) -> TypedNode6<'_> { ::tsox_core::fntrace::enter("as_identifier"); 
        self.assert_kind(SyntaxKind::Identifier);
        self.as_typed()
    }

    fn as_private_identifier(&self) -> TypedNode6<'_> { ::tsox_core::fntrace::enter("as_private_identifier"); 
        self.assert_kind(SyntaxKind::PrivateIdentifier);
        self.as_typed()
    }

    fn as_string_literal(&self) -> TypedNode6<'_> { ::tsox_core::fntrace::enter("as_string_literal"); 
        self.assert_kind(SyntaxKind::StringLiteral);
        self.as_typed()
    }

    fn as_qualified_name(&self) -> TypedNode6<'_> { ::tsox_core::fntrace::enter("as_qualified_name"); 
        self.assert_kind(SyntaxKind::QualifiedName);
        self.as_typed()
    }

    fn as_type_parameter_declaration(&self) -> TypedNode6<'_> { ::tsox_core::fntrace::enter("as_type_parameter_declaration"); 
        self.assert_kind(SyntaxKind::TypeParameter);
        self.as_typed()
    }

    fn as_meta_property(&self) -> TypedNode6<'_> { ::tsox_core::fntrace::enter("as_meta_property"); 
        self.assert_kind(SyntaxKind::MetaProperty);
        self.as_typed()
    }

    fn as_method_declaration(&self) -> TypedNode6<'_> { ::tsox_core::fntrace::enter("as_method_declaration"); 
        self.assert_kind(SyntaxKind::MethodDeclaration);
        self.as_typed()
    }

    fn as_method_signature_declaration(&self) -> TypedNode6<'_> { ::tsox_core::fntrace::enter("as_method_signature_declaration"); 
        self.assert_kind(SyntaxKind::MethodSignature);
        self.as_typed()
    }

    fn as_decorator(&self) -> TypedNode6<'_> { ::tsox_core::fntrace::enter("as_decorator"); 
        self.assert_kind(SyntaxKind::Decorator);
        self.as_typed()
    }

    fn as_module_block(&self) -> TypedNode6<'_> { ::tsox_core::fntrace::enter("as_module_block"); 
        self.assert_kind(SyntaxKind::ModuleBlock);
        self.as_typed()
    }

    fn as_module_declaration(&self) -> TypedNode6<'_> { ::tsox_core::fntrace::enter("as_module_declaration"); 
        self.assert_kind(SyntaxKind::ModuleDeclaration);
        self.as_typed()
    }

    fn as_external_module_reference(&self) -> TypedNode6<'_> { ::tsox_core::fntrace::enter("as_external_module_reference"); 
        self.assert_kind(SyntaxKind::ExternalModuleReference);
        self.as_typed()
    }

    fn as_namespace_export(&self) -> TypedNode6<'_> { ::tsox_core::fntrace::enter("as_namespace_export"); 
        self.assert_kind(SyntaxKind::NamespaceExport);
        self.as_typed()
    }

    fn as_named_exports(&self) -> TypedNode6<'_> { ::tsox_core::fntrace::enter("as_named_exports"); 
        self.assert_kind(SyntaxKind::NamedExports);
        self.as_typed()
    }
}

impl NodeAsExt for Node {
    fn assert_kind(&self, kind: SyntaxKind) -> &Node { ::tsox_core::fntrace::enter("assert_kind"); 
        debug_assert_eq!(self.kind, kind, "as typed node on wrong kind");
        self
    }

    fn as_typed(&self) -> TypedNode6<'_> { ::tsox_core::fntrace::enter("as_typed"); 
        TypedNode6 { node: self }
    }
}

impl<'a> TypedNode6<'a> {
    pub fn name(&self) -> &'a Node { ::tsox_core::fntrace::enter("name"); 
        match &self.node.data {
            NodeData::TypeParameterDeclaration(d) => &d.name,
            NodeData::MetaProperty(d) => &d.name,
            NodeData::MethodDeclaration(d) => &d.name,
            NodeData::MethodSignatureDeclaration(d) => &d.name,
            NodeData::NamespaceExport(d) => &d.name,
            _ => panic!("name() on {:?}", self.node.kind),
        }
    }

    pub fn constraint(&self) -> Option<&'a Arc<Node>> { ::tsox_core::fntrace::enter("constraint"); 
        match &self.node.data {
            NodeData::TypeParameterDeclaration(d) => d.constraint.as_ref(),
            _ => panic!("constraint() on {:?}", self.node.kind),
        }
    }

    pub fn keyword_token(&self) -> SyntaxKind { ::tsox_core::fntrace::enter("keyword_token"); 
        match &self.node.data {
            NodeData::MetaProperty(d) => d.keyword_token,
            NodeData::ModuleDeclaration(d) => d.keyword,
            _ => panic!("keywordToken() on {:?}", self.node.kind),
        }
    }

    pub fn modifiers(&self) -> Option<&'a ModifierList> { ::tsox_core::fntrace::enter("modifiers"); 
        self.node.modifiers().map(|m| &**m)
    }

    pub fn asterisk_token(&self) -> Option<&'a Node> { ::tsox_core::fntrace::enter("asterisk_token"); 
        match &self.node.data {
            NodeData::MethodDeclaration(d) => d.asterisk_token.as_deref(),
            NodeData::FunctionDeclaration(d) => d.asterisk_token.as_deref(),
            NodeData::FunctionExpression(d) => d.asterisk_token.as_deref(),
            _ => panic!("asteriskToken() on {:?}", self.node.kind),
        }
    }

    pub fn postfix_token(&self) -> Option<&'a Node> { ::tsox_core::fntrace::enter("postfix_token"); 
        match &self.node.data {
            NodeData::MethodDeclaration(d) => d.postfix_token.as_deref(),
            NodeData::MethodSignatureDeclaration(d) => d.postfix_token.as_deref(),
            _ => panic!("postfixToken() on {:?}", self.node.kind),
        }
    }

    pub fn body(&self) -> Option<&'a Node> { ::tsox_core::fntrace::enter("body"); 
        match &self.node.data {
            NodeData::MethodDeclaration(d) => d.body.as_deref(),
            NodeData::ModuleDeclaration(d) => d.body.as_deref(),
            _ => panic!("body() on {:?}", self.node.kind),
        }
    }

    pub fn statements(&self) -> &'a NodeList { ::tsox_core::fntrace::enter("statements"); 
        match &self.node.data {
            NodeData::ModuleBlock(d) => &d.statements,
            _ => panic!("statements() on {:?}", self.node.kind),
        }
    }

    pub fn keyword(&self) -> SyntaxKind { ::tsox_core::fntrace::enter("keyword"); 
        match &self.node.data {
            NodeData::ModuleDeclaration(d) => d.keyword,
            _ => panic!("keyword() on {:?}", self.node.kind),
        }
    }

    pub fn attributes(&self) -> Option<&'a Node> { ::tsox_core::fntrace::enter("attributes"); 
        match &self.node.data {
            NodeData::ModuleDeclaration(d) => d.attributes.as_deref(),
            _ => panic!("attributes() on {:?}", self.node.kind),
        }
    }

    pub fn elements(&self) -> &'a NodeList { ::tsox_core::fntrace::enter("elements"); 
        match &self.node.data {
            NodeData::NamedExports(d) => &d.elements,
            _ => panic!("elements() on {:?}", self.node.kind),
        }
    }

    pub fn expression(&self) -> &'a Node { ::tsox_core::fntrace::enter("expression"); 
        match &self.node.data {
            NodeData::Decorator(d) => &d.expression,
            _ => panic!("expression() on {:?}", self.node.kind),
        }
    }
}

pub trait NodeAccessorExt6 {
    fn asterisk_token6(&self) -> Option<&Arc<Node>>;
    fn heritage_token6(&self) -> SyntaxKind;
    fn heritage_types6(&self) -> &NodeList;
    fn heritage_clauses6(&self) -> Option<&NodeList>;
    fn members6(&self) -> &NodeList;
    fn decorators6(&self) -> &NodeList;
    fn tag_name6(&self) -> &Arc<Node>;
    fn jsx_attributes6(&self) -> &Arc<Node>;
}

impl NodeAccessorExt6 for Node {
    fn asterisk_token6(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("asterisk_token6"); 
        match &self.data {
            NodeData::MethodDeclaration(d) => d.asterisk_token.as_ref(),
            NodeData::FunctionDeclaration(d) => d.asterisk_token.as_ref(),
            NodeData::FunctionExpression(d) => d.asterisk_token.as_ref(),
            _ => None,
        }
    }

    fn heritage_token6(&self) -> SyntaxKind { ::tsox_core::fntrace::enter("heritage_token6"); 
        match &self.data {
            NodeData::HeritageClause(d) => d.token,
            _ => panic!("token() on {:?}", self.kind),
        }
    }

    fn heritage_types6(&self) -> &NodeList { ::tsox_core::fntrace::enter("heritage_types6"); 
        match &self.data {
            NodeData::HeritageClause(d) => &d.types,
            _ => panic!("types() on {:?}", self.kind),
        }
    }

    fn heritage_clauses6(&self) -> Option<&NodeList> { ::tsox_core::fntrace::enter("heritage_clauses6"); 
        match &self.data {
            NodeData::ClassDeclaration(d) => d.heritage_clauses.as_deref(),
            NodeData::ClassExpression(d) => d.heritage_clauses.as_deref(),
            _ => panic!("heritageClauses() on {:?}", self.kind),
        }
    }

    fn members6(&self) -> &NodeList { ::tsox_core::fntrace::enter("members6"); 
        match &self.data {
            NodeData::ClassDeclaration(d) => &d.members,
            NodeData::ClassExpression(d) => &d.members,
            _ => panic!("members() on {:?}", self.kind),
        }
    }

    fn decorators6(&self) -> &NodeList { ::tsox_core::fntrace::enter("decorators6"); 
        match &self.data {
            NodeData::ParameterDeclaration(d) => match &d.modifiers {
                Some(m) => &m.list,
                None => panic!("decorators() on parameter without modifiers"),
            },
            _ => panic!("decorators() on {:?}", self.kind),
        }
    }

    fn tag_name6(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("tag_name6"); 
        match &self.data {
            NodeData::JsxSelfClosingElement(d) => &d.tag_name,
            NodeData::JsxOpeningElement(d) => &d.tag_name,
            _ => panic!("tagName() on {:?}", self.kind),
        }
    }

    fn jsx_attributes6(&self) -> &Arc<Node> { ::tsox_core::fntrace::enter("jsx_attributes6"); 
        match &self.data {
            NodeData::JsxSelfClosingElement(d) => &d.attributes,
            NodeData::JsxOpeningElement(d) => &d.attributes,
            _ => panic!("attributes() on {:?}", self.kind),
        }
    }
}
