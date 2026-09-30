use std::sync::Arc;

use crate::ast::diagnostic::Diagnostic;
use crate::ast::node::Node;
use crate::ast::node_data_generated::{NodeData, is_private_identifier};
use crate::ast::node_node_list::ModifierList;
use crate::ast::node_source_file::SourceFile;
use crate::ast::syntax_kind_generated::SyntaxKind;

#[derive(Clone, Copy, Default)]
pub struct NodeFactoryHooks {
    pub on_update: Option<fn(&Arc<Node>, &Arc<Node>)>,
    pub on_clone: Option<fn(&Arc<Node>, &Arc<Node>)>,
}

pub struct NodeFactory {
    pub hooks: NodeFactoryHooks,
    pub text_count: usize,
    pub node_count: usize,
}

impl NodeFactory {
    pub fn with_counters(hooks: NodeFactoryHooks, text_count: usize, node_count: usize) -> Self {
        Self { hooks, text_count, node_count }
    }

    pub fn new_source_file(
        &mut self,
        text: &str,
        statements: Arc<crate::ast::node::NodeList>,
        end_of_file_token: Arc<Node>,
    ) -> Arc<Node> {
        self.node_count += 1;
        Arc::new(Node::new(
            SyntaxKind::SourceFile,
            NodeData::SourceFile(crate::ast::node_data_generated::SourceFileData {
                statements,
                end_of_file_token,
                global_exports: None,
            }),
        ))
    }

    pub fn update_source_file(
        &mut self,
        node: &SourceFile,
        statements: Option<Arc<crate::ast::node::NodeList>>,
        end_of_file_token: Option<Arc<Node>>,
    ) -> Arc<Node> {
        update_source_file(self, node, statements, end_of_file_token)
    }
}

pub struct NodeVisitor {
    pub factory: NodeFactory,
}

impl NodeVisitor {
    pub fn visit_top_level_statements(
        &self,
        node: &Arc<Node>,
    ) -> Option<Arc<crate::ast::node::NodeList>> {
        statement_list(node).map(|l| Arc::clone(l))
    }

    pub fn visit_token(&self, token: &Arc<Node>) -> Option<Arc<Node>> {
        Some(Arc::clone(token))
    }
}


pub fn question_token(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::ParameterDeclaration(d) => d.question_token.as_ref(),
        NodeData::ConditionalExpression(d) => Some(&d.question_token),
        NodeData::MappedTypeNode(d) => d.question_token.as_ref(),
        NodeData::NamedTupleMember(d) => d.question_token.as_ref(),
        _ => postfix_token_fallback(node),
    }
}

fn postfix_token_fallback(node: &Node) -> Option<&Arc<Node>> {
    let postfix = match &node.data {
        NodeData::MethodDeclaration(d) => d.postfix_token.as_ref()?,
        NodeData::MethodSignatureDeclaration(d) => d.postfix_token.as_ref()?,
        NodeData::PropertySignatureDeclaration(d) => d.postfix_token.as_ref()?,
        NodeData::PropertyDeclaration(d) => d.postfix_token.as_ref()?,
        NodeData::PropertyAssignment(d) => d.postfix_token.as_ref()?,
        NodeData::ShorthandPropertyAssignment(d) => d.postfix_token.as_ref()?,
        _ => return None,
    };
    if postfix.kind == SyntaxKind::QuestionToken {
        Some(postfix)
    } else {
        None
    }
}

pub fn raw_text(node: &Node) -> &str {
    match &node.data {
        NodeData::TemplateHead(d) => &d.raw_text,
        NodeData::TemplateMiddle(d) => &d.raw_text,
        NodeData::TemplateTail(d) => &d.raw_text,
        _ => panic!("Unhandled case in Node.RawText"),
    }
}

pub fn symbol<'a>(
    node: &'a Node,
    map: &'a crate::ast::symbol_map::NodeSymbolMap,
) -> Option<&'a Arc<crate::ast::symbol::Symbol>> {
    match &node.data {
        NodeData::VariableDeclaration(_)
        | NodeData::ParameterDeclaration(_)
        | NodeData::BindingElement(_)
        | NodeData::PropertyDeclaration(_)
        | NodeData::PropertySignatureDeclaration(_)
        | NodeData::PropertyAssignment(_)
        | NodeData::ShorthandPropertyAssignment(_)
        | NodeData::EnumMember(_)
        | NodeData::FunctionDeclaration(_)
        | NodeData::ClassDeclaration(_)
        | NodeData::ClassExpression(_)
        | NodeData::InterfaceDeclaration(_)
        | NodeData::TypeAliasDeclaration(_)
        | NodeData::EnumDeclaration(_)
        | NodeData::ModuleDeclaration(_)
        | NodeData::ImportEqualsDeclaration(_)
        | NodeData::ImportClause(_)
        | NodeData::NamespaceImport(_)
        | NodeData::NamespaceExport(_)
        | NodeData::ExportSpecifier(_)
        | NodeData::ImportSpecifier(_)
        | NodeData::MethodDeclaration(_)
        | NodeData::MethodSignatureDeclaration(_)
        | NodeData::GetAccessorDeclaration(_)
        | NodeData::SetAccessorDeclaration(_)
        | NodeData::ConstructorDeclaration(_) => map.symbol_of(node),
        _ => None,
    }
}

pub fn tag_name(node: &Node) -> &Arc<Node> {
    match &node.data {
        NodeData::JsxOpeningElement(d) => &d.tag_name,
        NodeData::JsxClosingElement(d) => &d.tag_name,
        NodeData::JsxSelfClosingElement(d) => &d.tag_name,
        NodeData::JSDocUnknownTag(d) => &d.tag_name,
        NodeData::JSDocAugmentsTag(d) => &d.tag_name,
        NodeData::JSDocImplementsTag(d) => &d.tag_name,
        NodeData::JSDocDeprecatedTag(d) => &d.tag_name,
        NodeData::JSDocPublicTag(d) => &d.tag_name,
        NodeData::JSDocPrivateTag(d) => &d.tag_name,
        NodeData::JSDocProtectedTag(d) => &d.tag_name,
        NodeData::JSDocReadonlyTag(d) => &d.tag_name,
        NodeData::JSDocOverrideTag(d) => &d.tag_name,
        NodeData::JSDocSatisfiesTag(d) => &d.tag_name,
        _ => panic!("Unhandled case in Node.TagName"),
    }
}

pub fn type_argument_list(node: &Node) -> Option<&Arc<crate::ast::node::NodeList>> {
    match &node.data {
        NodeData::CallExpression(d) => d.type_arguments.as_ref(),
        NodeData::NewExpression(d) => d.type_arguments.as_ref(),
        NodeData::TaggedTemplateExpression(d) => d.type_arguments.as_ref(),
        NodeData::TypeReferenceNode(d) => d.type_arguments.as_ref(),
        NodeData::ExpressionWithTypeArguments(d) => d.type_arguments.as_ref(),
        NodeData::ImportTypeNode(d) => d.type_arguments.as_ref(),
        NodeData::TypeQueryNode(d) => d.type_arguments.as_ref(),
        NodeData::JsxOpeningElement(d) => d.type_arguments.as_ref(),
        NodeData::JsxSelfClosingElement(d) => d.type_arguments.as_ref(),
        _ => panic!("Unhandled case in Node.TypeArgumentList"),
    }
}

pub fn type_arguments(node: &Node) -> &[Arc<Node>] {
    type_argument_list(node).map(|l| l.nodes.as_slice()).unwrap_or(&[])
}

pub fn type_parameter_list(node: &Node) -> Option<&Arc<crate::ast::node::NodeList>> {
    match &node.data {
        NodeData::ClassDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::ClassExpression(d) => d.type_parameters.as_ref(),
        NodeData::InterfaceDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::TypeAliasDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::JSDocTemplateTag(d) => Some(&d.type_parameters),
        NodeData::FunctionDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::FunctionExpression(d) => d.type_parameters.as_ref(),
        NodeData::ArrowFunction(d) => d.type_parameters.as_ref(),
        NodeData::MethodDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::MethodSignatureDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::ConstructorDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::GetAccessorDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::SetAccessorDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::CallSignatureDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::ConstructSignatureDeclaration(d) => d.type_parameters.as_ref(),
        NodeData::FunctionTypeNode(d) => d.type_parameters.as_ref(),
        NodeData::ConstructorTypeNode(d) => d.type_parameters.as_ref(),
        NodeData::JSDocSignature(d) => d.type_parameters.as_ref(),
        NodeData::IndexSignatureDeclaration(_) => None,
        _ => panic!("Unhandled case in Node.TypeParameterList"),
    }
}

pub fn type_parameters(node: &Node) -> &[Arc<Node>] {
    type_parameter_list(node).map(|l| l.nodes.as_slice()).unwrap_or(&[])
}

pub fn statement_list(node: &Node) -> Option<&Arc<crate::ast::node::NodeList>> {
    match &node.data {
        NodeData::SourceFile(d) => Some(&d.statements),
        NodeData::Block(d) => Some(&d.statements),
        NodeData::ModuleBlock(d) => Some(&d.statements),
        NodeData::CaseOrDefaultClause(d) => Some(&d.statements),
        _ => panic!("Unhandled case in Node.StatementList"),
    }
}

pub fn statements(node: &Node) -> &[Arc<Node>] {
    statement_list(node).map(|l| l.nodes.as_slice()).unwrap_or(&[])
}

pub fn statement(node: &Node) -> &Arc<Node> {
    match &node.data {
        NodeData::DoStatement(d) => &d.statement,
        NodeData::WhileStatement(d) => &d.statement,
        NodeData::ForStatement(d) => &d.statement,
        NodeData::ForInOrOfStatement(d) => &d.statement,
        NodeData::WithStatement(d) => &d.statement,
        NodeData::LabeledStatement(d) => &d.statement,
        _ => panic!("Unhandled case in Node.Statement"),
    }
}

pub fn type_expression(node: &Node) -> Option<&Arc<Node>> {
    match &node.data {
        NodeData::JSDocParameterOrPropertyTag(d) => d.type_expression.as_ref(),
        NodeData::JSDocReturnTag(d) => d.type_expression.as_ref(),
        NodeData::JSDocTypeTag(d) => Some(&d.type_expression),
        NodeData::JSDocTypedefTag(d) => d.type_expression.as_ref(),
        NodeData::JSDocCallbackTag(d) => Some(&d.type_expression),
        NodeData::JSDocSatisfiesTag(d) => Some(&d.type_expression),
        NodeData::JSDocThrowsTag(d) => d.type_expression.as_ref(),
        _ => None,
    }
}

pub fn visit_each_child(node: &Arc<Node>, v: &mut NodeVisitor) -> Arc<Node> {
    match &node.data {
        NodeData::SourceFile(d) => {
            let statements = v.visit_top_level_statements(node);
            let end_of_file_token = v.visit_token(&d.end_of_file_token);
            let new_statements = statements.unwrap_or_else(|| Arc::clone(&d.statements));
            let new_eof = end_of_file_token.unwrap_or_else(|| Arc::clone(&d.end_of_file_token));
            Arc::new(Node::new(
                SyntaxKind::SourceFile,
                NodeData::SourceFile(crate::ast::node_data_generated::SourceFileData {
                    statements: new_statements,
                    end_of_file_token: new_eof,
                    global_exports: None,
                }),
            ))
        }
        _ => visit_each_child_node_default(node, v),
    }
}

pub fn visit_each_child_node_default(node: &Arc<Node>, _v: &mut NodeVisitor) -> Arc<Node> {
    Arc::clone(node)
}

pub fn visit_each_child_source_file(file: &SourceFile, v: &mut NodeVisitor) -> Arc<Node> {
    let statements = v.visit_top_level_statements(&file.node);
    let end_of_file_token = v.visit_token(&file_end_of_file_token(file));
    v.factory.update_source_file(file, statements, end_of_file_token)
}

fn file_end_of_file_token(file: &SourceFile) -> Arc<Node> {
    match &file.node.data {
        NodeData::SourceFile(d) => Arc::clone(&d.end_of_file_token),
        _ => panic!("node is not a source file"),
    }
}

pub fn clone_node(
    updated: Arc<Node>,
    original: &Arc<Node>,
    hooks: &NodeFactoryHooks,
) -> Arc<Node> {
    if !Arc::ptr_eq(&updated, original) {
        let updated_mut = unsafe { unsafe_mutable(&updated) };
        updated_mut.flags = original.flags;
        updated_mut.loc = original.loc;
        if let Some(on_update) = hooks.on_update {
            on_update(&updated, original);
        }
        if let Some(on_clone) = hooks.on_clone {
            on_clone(&updated, original);
        }
    }
    updated
}

unsafe fn unsafe_mutable(node: &Arc<Node>) -> &mut Node {
    &mut *(Arc::as_ptr(node) as *mut Node)
}

pub fn text_count(f: &NodeFactory) -> usize {
    f.text_count
}

pub fn release_arenas(f: &mut NodeFactory) {
    *f = NodeFactory::with_counters(f.hooks.clone(), f.text_count, f.node_count);
}

pub fn update_source_file(
    f: &mut NodeFactory,
    node: &SourceFile,
    statements: Option<Arc<crate::ast::node::NodeList>>,
    end_of_file_token: Option<Arc<Node>>,
) -> Arc<Node> {
    let cur_statements = statement_list(&node.node);
    let cur_eof = file_end_of_file_token(node);
    let statements_changed = !option_list_same(&statements, cur_statements);
    let eof_changed = match &end_of_file_token {
        Some(t) => !Arc::ptr_eq(t, &cur_eof),
        None => false,
    };
    if statements_changed || eof_changed {
        let new_statements = statements.unwrap_or_else(|| Arc::clone(cur_statements.unwrap()));
        let new_eof = end_of_file_token.unwrap_or_else(|| Arc::clone(&cur_eof));
        let updated = f.new_source_file(&node.text, new_statements, new_eof);
        return updated;
    }
    Arc::clone(&node.node)
}

fn option_list_same(
    a: &Option<Arc<crate::ast::node::NodeList>>,
    b: Option<&Arc<crate::ast::node::NodeList>>,
) -> bool {
    match (a, b) {
        (Some(x), Some(y)) => Arc::ptr_eq(x, y),
        (None, None) => true,
        _ => false,
    }
}
