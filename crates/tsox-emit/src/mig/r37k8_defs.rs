#![allow(unused_imports, dead_code)]

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use tsox_frontend::ast::mig::m3g_3::{new_has_file_name, HasFileNameImpl};
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::node_data_generated::*;
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::is_prologue_directive;
use tsox_frontend::ast::{ModifierList, NodeList, TokenFlags};
use tsox_frontend::scanner::TOKEN_FLAGS_NONE;

use crate::mig::r33k6_shim::Visitor;
use crate::printer::generated_identifier_flags::{EmitContext, NodeFactory};

impl Clone for EmitContext {
    fn clone(&self) -> Self {
        let mut cloned = EmitContext::new();
        cloned.next_id = AtomicU32::new(self.next_id.load(Ordering::Relaxed));
        cloned
    }
}

pub fn empty_has_file_name() -> HasFileNameImpl {
    new_has_file_name(String::new(), tsox_core::tspath::Path(String::new()))
}

pub trait NodeAsR37k8Ext {
    fn as_source_file(&self) -> &SourceFileData;
    fn as_import_declaration(&self) -> &ImportDeclarationData;
    fn as_export_assignment(&self) -> &ExportAssignmentData;
    fn as_export_declaration(&self) -> &ExportDeclarationData;
    fn as_call_expression(&self) -> &CallExpressionData;
    fn as_string_literal(&self) -> &StringLiteralData;
    fn is_declaration_file(&self) -> bool;
}

macro_rules! as_data_r37k8 {
    ($name:ident, $variant:ident, $ty:ty) => {
        fn $name(&self) -> &$ty {
            match &self.data {
                NodeData::$variant(d) => d,
                _ => panic!(concat!("As", stringify!($variant), " on wrong node kind")),
            }
        }
    };
}

impl NodeAsR37k8Ext for Node {
    as_data_r37k8!(as_source_file, SourceFile, SourceFileData);
    as_data_r37k8!(
        as_import_declaration,
        ImportDeclaration,
        ImportDeclarationData
    );
    as_data_r37k8!(as_export_assignment, ExportAssignment, ExportAssignmentData);
    as_data_r37k8!(as_export_declaration, ExportDeclaration, ExportDeclarationData);
    as_data_r37k8!(as_call_expression, CallExpression, CallExpressionData);
    as_data_r37k8!(as_string_literal, StringLiteral, StringLiteralData);

    fn is_declaration_file(&self) -> bool {
        let _ = self.kind == SyntaxKind::SourceFile;
        false
    }
}

pub enum OptNode {
    None,
    Some(Arc<Node>),
}

impl From<Arc<Node>> for OptNode {
    fn from(node: Arc<Node>) -> Self {
        OptNode::Some(node)
    }
}

impl From<Option<Arc<Node>>> for OptNode {
    fn from(node: Option<Arc<Node>>) -> Self {
        match node {
            Some(n) => OptNode::Some(n),
            None => OptNode::None,
        }
    }
}

impl Visitor {
    pub fn visit_node(&mut self, node: impl Into<OptNode>) -> Option<Arc<Node>> {
        match node.into() {
            OptNode::None => None,
            OptNode::Some(n) => Some(n),
        }
    }

    pub fn visit_slice(&mut self, nodes: &[Arc<Node>]) -> Vec<Arc<Node>> {
        nodes.to_vec()
    }

    pub fn visit_each_child(&mut self, node: Arc<Node>) -> Option<Arc<Node>> {
        let _ = &node;
        Some(node)
    }
}

pub fn split_standard_prologue_r37k8(source: &[Arc<Node>]) -> (Vec<Arc<Node>>, Vec<Arc<Node>>) {
    let mut prologue = Vec::new();
    let mut rest = Vec::new();
    let mut in_prologue = true;
    for node in source {
        if in_prologue && is_prologue_directive(node) {
            prologue.push(node.clone());
        } else {
            in_prologue = false;
            rest.push(node.clone());
        }
    }
    (prologue, rest)
}

pub fn split_custom_prologue_r37k8(source: &[Arc<Node>]) -> (Vec<Arc<Node>>, Vec<Arc<Node>>) {
    let mut custom = Vec::new();
    let mut rest = Vec::new();
    let mut in_prologue = true;
    for node in source {
        if in_prologue {
            let is_custom = matches!(&node.data, NodeData::VariableStatement(_))
                && is_prologue_directive(node);
            if is_custom {
                custom.push(node.clone());
                continue;
            }
        }
        in_prologue = false;
        rest.push(node.clone());
    }
    (custom, rest)
}

impl<'a> NodeFactory<'a> {
    pub fn new_import_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        import_clause: Option<Arc<Node>>,
        module_specifier: Arc<Node>,
        attributes: Option<Arc<Node>>,
    ) -> Arc<Node> {
        let _ = modifiers;
        Arc::new(Node::new(
            SyntaxKind::ImportDeclaration,
            NodeData::ImportDeclaration(ImportDeclarationData {
                modifiers: None,
                import_clause,
                module_specifier,
                attributes,
            }),
        ))
    }

    pub fn new_import_clause(
        &self,
        is_type_only: SyntaxKind,
        name: Option<Arc<Node>>,
        named_bindings: Option<Arc<Node>>,
    ) -> Arc<Node> {
        let _ = is_type_only;
        Arc::new(Node::new(
            SyntaxKind::ImportClause,
            NodeData::ImportClause(ImportClauseData {
                phase_modifier: None,
                name,
                named_bindings,
            }),
        ))
    }

    pub fn new_namespace_import(&self, name: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::NamespaceImport,
            NodeData::NamespaceImport(NamespaceImportData { name }),
        ))
    }

    pub fn new_meta_property(&self, keyword_token: SyntaxKind, name: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::MetaProperty,
            NodeData::MetaProperty(MetaPropertyData {
                keyword_token,
                name,
            }),
        ))
    }

    pub fn new_true_expression(&self) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TrueKeyword,
            NodeData::Identifier(IdentifierData {
                text: "true".to_string(),
            }),
        ))
    }

    pub fn new_get_accessor_declaration(
        &self,
        name: Arc<Node>,
        parameters: Arc<NodeList>,
        body: Option<Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::GetAccessor,
            NodeData::GetAccessorDeclaration(GetAccessorDeclarationData {
                modifiers: None,
                name,
                type_parameters: None,
                parameters,
                type_node: None,
                full_signature: None,
                body,
            }),
        ))
    }

    pub fn new_run_initializers_helper(&self, initializers: Arc<NodeList>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::CallExpression,
            NodeData::CallExpression(CallExpressionData {
                expression: Arc::new(Node::new(
                    SyntaxKind::Identifier,
                    NodeData::Identifier(IdentifierData {
                        text: "__runInitializers".to_string(),
                    }),
                )),
                question_dot_token: None,
                type_arguments: None,
                arguments: Arc::new(NodeList {
                    loc: initializers.loc,
                    nodes: initializers.nodes.clone(),
                }),
            }),
        ))
    }

    pub fn new_rewrite_relative_import_extensions_helper(
        &self,
        expression: Arc<Node>,
        _use_js_extension: bool,
    ) -> Arc<Node> {
        expression
    }

    pub fn update_call_expression(
        &self,
        node: &Arc<Node>,
        expression: Option<Arc<Node>>,
        _question_dot_token: Option<Arc<Node>>,
        _type_arguments: Option<Arc<Node>>,
        arguments: Arc<NodeList>,
        _flags: NodeFlags,
    ) -> Arc<Node> {
        let call = node.as_call_expression();
        Arc::new(Node::new(
            SyntaxKind::CallExpression,
            NodeData::CallExpression(CallExpressionData {
                expression: expression.unwrap_or_else(|| call.expression.clone()),
                question_dot_token: call.question_dot_token.clone(),
                type_arguments: call.type_arguments.clone(),
                arguments,
            }),
        ))
    }
}
