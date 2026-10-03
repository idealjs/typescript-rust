#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{self, Node, NodeData, NodeList, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::astnav;

use crate::ls::types_symbols::{ParameterInformation, SignatureInformation};

pub const SIGNATURE_HELP_NODE_BUILDER_FLAGS: u32 = 0;

const ELEMENT_FLAGS_NON_REQUIRED_BITS: u32 = 0b1110;

fn m5w_nil_node() -> Arc<Node> { ::tsox_core::fntrace::enter("m5w_nil_node"); 
    Arc::new(Node::new(
        SyntaxKind::MissingDeclaration,
        NodeData::MissingDeclaration(tsox_frontend::ast::MissingDeclarationData {
            modifiers: None,
        }),
    ))
}

#[derive(Clone)]
pub struct M5wCallInvocation {
    pub node: Arc<Node>,
}

impl Default for M5wCallInvocation {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        M5wCallInvocation {
            node: m5w_nil_node(),
        }
    }
}

#[derive(Clone)]
pub struct M5wTypeArgsInvocation {
    pub called: Arc<Node>,
}

impl Default for M5wTypeArgsInvocation {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        M5wTypeArgsInvocation {
            called: m5w_nil_node(),
        }
    }
}

#[derive(Clone)]
pub struct M5wContextualInvocation {
    pub signature: Option<Arc<tsox_checker::checker::Signature>>,
    pub node: Arc<Node>,
    pub symbol: Option<Arc<Symbol>>,
}

impl Default for M5wContextualInvocation {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        M5wContextualInvocation {
            signature: None,
            node: m5w_nil_node(),
            symbol: None,
        }
    }
}

#[derive(Clone, Default)]
pub struct M5wInvocation {
    pub call_invocation: Option<M5wCallInvocation>,
    pub type_args_invocation: Option<M5wTypeArgsInvocation>,
    pub contextual_invocation: Option<M5wContextualInvocation>,
}

#[derive(Clone, Default)]
pub struct M5wArgumentListInfo {
    pub is_type_parameter_list: bool,
    pub invocation: M5wInvocation,
    pub arguments_span: TextRange,
    pub argument_index: usize,
    pub argument_count: usize,
}

pub struct M5wCandidateInfo {
    pub candidates: Vec<Arc<tsox_checker::checker::Signature>>,
    pub resolved_signature: Option<Arc<tsox_checker::checker::Signature>>,
}

pub struct M5wCandidateOrTypeInfo {
    pub candidate_info: Option<M5wCandidateInfo>,
    pub type_info: Option<Arc<Symbol>>,
}

pub struct M5wArgumentOrParameterListInfo {
    pub list: Arc<NodeList>,
    pub argument_index: usize,
    pub argument_count: usize,
    pub arguments_span: TextRange,
}

pub struct M5wArgumentOrParameterListAndIndex {
    pub list: Arc<NodeList>,
    pub argument_index: usize,
}

pub struct M5wContextualSignatureLocationInfo {
    pub contextual_type: Arc<tsox_checker::checker::Type>,
    pub argument_index: usize,
    pub argument_count: usize,
    pub arguments_span: TextRange,
}

pub struct M5wSignatureHelpItemInfo {
    pub is_variadic: bool,
    pub parameters: Vec<M5wSignatureHelpParameter>,
    pub writer: crate::ls::display_parts_writer::DisplayPartsWriter,
}

#[derive(Clone)]
pub struct M5wSignatureHelpParameter {
    pub parameter_info: ParameterInformation,
    pub is_variadic: bool,
}

pub fn get_enclosing_declaration_from_invocation(
    invocation: &M5wInvocation,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_enclosing_declaration_from_invocation"); 
    if let Some(call) = &invocation.call_invocation {
        return Some(call.node.clone());
    }
    if let Some(type_args) = &invocation.type_args_invocation {
        return Some(type_args.called.clone());
    }
    invocation.contextual_invocation.as_ref().map(|c| c.node.clone())
}

pub fn get_expression_from_invocation(
    argument_info: &M5wArgumentListInfo,
) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_expression_from_invocation"); 
    if let Some(call) = &argument_info.invocation.call_invocation {
        return Some(tsox_frontend::ast::mig::x4ast::get_invoked_expression(&call.node));
    }
    argument_info
        .invocation
        .type_args_invocation
        .as_ref()
        .map(|t| t.called.clone())
}

pub fn call_or_new_expression_expression(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("call_or_new_expression_expression"); 
    match &node.data {
        NodeData::CallExpression(d) => Some(d.expression.clone()),
        NodeData::NewExpression(d) => Some(d.expression.clone()),
        _ => None,
    }
}

pub fn call_or_new_expression_type_argument_list(node: &Arc<Node>) -> Option<Arc<NodeList>> { ::tsox_core::fntrace::enter("call_or_new_expression_type_argument_list"); 
    match &node.data {
        NodeData::CallExpression(d) => d.type_arguments.clone(),
        NodeData::NewExpression(d) => d.type_arguments.clone(),
        _ => None,
    }
}

pub fn get_candidate_or_type_info(
    info: &M5wArgumentListInfo,
    c: &mut Checker,
    source_file: &Arc<SourceFile>,
    starting_token: &Arc<Node>,
    only_use_syntactic_owners: bool,
) -> Option<M5wCandidateOrTypeInfo> { ::tsox_core::fntrace::enter("get_candidate_or_type_info"); 
    if let Some(call) = &info.invocation.call_invocation {
        if only_use_syntactic_owners
            && !is_syntactic_owner(starting_token, &call.node, source_file)
        {
            return None;
        }
        let (resolved_signature, candidates) = c.get_resolved_signature_for_signature_help(
            &call.node,
            info.argument_count as i32,
        );
        if candidates.is_empty() {
            return None;
        }
        return Some(M5wCandidateOrTypeInfo {
            candidate_info: Some(M5wCandidateInfo {
                candidates,
                resolved_signature,
            }),
            type_info: None,
        });
    }
    if let Some(type_args) = &info.invocation.type_args_invocation {
        let called = type_args.called.clone();
        let container = if ast::is_identifier(&called) {
            called.parent()?
        } else {
            called.clone()
        };
        if only_use_syntactic_owners
            && !contains_preceding_token(starting_token, source_file, &container)
        {
            return None;
        }
        let candidates =
            super::m5x_5::get_possible_generic_signatures(&called, info.argument_count, c);
        if !candidates.is_empty() {
            let resolved = candidates[0].clone();
            return Some(M5wCandidateOrTypeInfo {
                candidate_info: Some(M5wCandidateInfo {
                    candidates,
                    resolved_signature: Some(resolved),
                }),
                type_info: None,
            });
        }
        if let Some(symbol) = c.get_symbol_at_location(&called) {
            return Some(M5wCandidateOrTypeInfo {
                candidate_info: None,
                type_info: Some(symbol),
            });
        }
        return None;
    }
    if let Some(contextual) = &info.invocation.contextual_invocation {
        let signature = contextual.signature.clone()?;
        return Some(M5wCandidateOrTypeInfo {
            candidate_info: Some(M5wCandidateInfo {
                candidates: vec![signature.clone()],
                resolved_signature: Some(signature),
            }),
            type_info: None,
        });
    }
    None
}

pub fn is_syntactic_owner(
    starting_token: &Arc<Node>,
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
) -> bool { ::tsox_core::fntrace::enter("is_syntactic_owner"); 
    if !tsox_frontend::ast::mig::m3f_4::is_call_or_new_expression(node) {
        return false;
    }
    let invocation_children =
        get_children_from_non_js_doc_node(node, source_file);
    match starting_token.kind {
        SyntaxKind::OpenParenToken | SyntaxKind::CommaToken => {
            invocation_children.iter().any(|child| Arc::ptr_eq(child, starting_token))
        }
        SyntaxKind::LessThanToken => {
            let expression = call_or_new_expression_expression(node);
            expression
                .map(|expr| contains_preceding_token(starting_token, source_file, &expr))
                .unwrap_or(false)
        }
        _ => false,
    }
}

pub fn contains_preceding_token(
    starting_token: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    container: &Arc<Node>,
) -> bool { ::tsox_core::fntrace::enter("contains_preceding_token"); 
    let pos = starting_token.pos();
    let mut current_parent = starting_token.parent();
    while let Some(parent) = current_parent {
        if let Some(preceding_token) =
            astnav::find_preceding_token(&source_file.node, pos)
        {
            return super::m5x_3::range_contains_range(container.loc, preceding_token.loc);
        }
        current_parent = parent.parent();
    }
    false
}

pub fn get_children_from_non_js_doc_node(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_children_from_non_js_doc_node"); 
    let mut child_nodes: Vec<Arc<Node>> = Vec::new();
    ast::for_each_child(node, |child: &Arc<Node>| {
        child_nodes.push(Arc::clone(child));
        false
    });
    if child_nodes.is_empty() {
        return Vec::new();
    }
    let mut children: Vec<Arc<Node>> = Vec::new();
    let mut pos = node.pos();
    for child in &child_nodes {
        while pos < child.pos() {
            let scanner = tsox_frontend::scanner::mig::m4d_2::get_scanner_for_source_file(source_file, pos);
            let token = scanner.token();
            let token_full_start = scanner.full_start_pos();
            let token_end = scanner.token_end();
            children.push(tsox_frontend::ast::mig::m3b_2::get_or_create_token(
                source_file,
                token,
                token_full_start,
                token_end,
                node,
                scanner.token_flags(),
            ));
            pos = token_end;
        }
        children.push(Arc::clone(child));
        pos = child.end();
    }
    while pos < node.end() {
        let scanner = tsox_frontend::scanner::mig::m4d_2::get_scanner_for_source_file(source_file, pos);
        let token = scanner.token();
        let token_full_start = scanner.full_start_pos();
        let token_end = scanner.token_end();
        children.push(tsox_frontend::ast::mig::m3b_2::get_or_create_token(
            source_file,
            token,
            token_full_start,
            token_end,
            node,
            scanner.token_flags(),
        ));
        pos = token_end;
    }
    children
}

pub fn get_containing_argument_info(
    node: &Arc<Node>,
    source_file: &Arc<SourceFile>,
    checker: &mut Checker,
    is_manually_invoked: bool,
    position: usize,
) -> Option<M5wArgumentListInfo> { ::tsox_core::fntrace::enter("get_containing_argument_info"); 
    let mut first_argument_info: Option<M5wArgumentListInfo> = None;
    let mut current = Some(node.clone());
    while let Some(n) = current {
        if ast::is_source_file(&n) {
            break;
        }
        if !is_manually_invoked && ast::is_block(&n) {
            break;
        }
        if let Some(argument_info) =
            get_immediately_containing_argument_or_contextual_parameter_info(
                &n,
                position,
                source_file,
                checker,
            )
        {
            if argument_info.invocation.contextual_invocation.is_some() {
                return Some(argument_info);
            }
            if first_argument_info.is_none() {
                first_argument_info = Some(argument_info.clone());
            }
            if argument_info.arguments_span.end() == position {
                return Some(argument_info);
            }
            if argument_info.arguments_span.contains(position) {
                return Some(argument_info);
            }
        }
        current = n.parent();
    }
    first_argument_info
}

pub fn get_immediately_containing_argument_or_contextual_parameter_info(
    node: &Arc<Node>,
    position: usize,
    source_file: &Arc<SourceFile>,
    checker: &mut Checker,
) -> Option<M5wArgumentListInfo> { ::tsox_core::fntrace::enter("get_immediately_containing_argument_or_contextual_parameter_info"); 
    if let Some(result) = super::m5w_2::try_get_parameter_info(node, source_file, checker) {
        return Some(result);
    }
    super::m5w_2::get_immediately_containing_argument_info(node, position, source_file, checker)
}

pub fn get_spread_element_count(node: &Node, c: &mut Checker) -> usize { ::tsox_core::fntrace::enter("get_spread_element_count"); 
    let expression = match &node.data {
        NodeData::SpreadElement(d) => d.expression.clone(),
        _ => return 0,
    };
    let spread_type = c.get_type_at_location(&expression);
    if !tsox_checker::checker::utilities::is_tuple_type(&spread_type)
    {
        return 0;
    }
    let Some(tuple_type) = spread_type.target().and_then(|t| t.as_tuple_type()) else {
        return 0;
    };
    let element_flags = tuple_type.element_flags();
    let fixed_length = tuple_type.fixed_length;
    if fixed_length == 0 {
        return 0;
    }
    let first_optional_index = element_flags
        .iter()
        .position(|f| (f.bits() & ELEMENT_FLAGS_NON_REQUIRED_BITS) == 0);
    match first_optional_index {
        Some(index) => index,
        None => fixed_length,
    }
}

pub fn get_token_from_node_list(
    node_list: Option<&Arc<NodeList>>,
    node_list_parent: Option<&Arc<Node>>,
    source_file: &Arc<SourceFile>,
) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_token_from_node_list"); 
    let (Some(node_list), Some(node_list_parent)) = (node_list, node_list_parent) else {
        return Vec::new();
    };
    let mut left = node_list.pos();
    let mut node_list_index = 0;
    let mut tokens: Vec<Arc<Node>> = Vec::new();
    while left < node_list.end() {
        if node_list.nodes.len() > node_list_index
            && left == node_list.nodes[node_list_index].pos()
        {
            tokens.push(Arc::clone(&node_list.nodes[node_list_index]));
            left = node_list.nodes[node_list_index].end();
            node_list_index += 1;
        } else {
            let scanner = tsox_frontend::scanner::mig::m4d_2::get_scanner_for_source_file(source_file, left);
            let token = scanner.token();
            let token_full_start = scanner.full_start_pos();
            let token_end = scanner.token_end();
            tokens.push(tsox_frontend::ast::mig::m3b_2::get_or_create_token(
                source_file,
                token,
                token_full_start,
                token_end,
                node_list_parent,
                scanner.token_flags(),
            ));
            left = token_end;
        }
    }
    tokens
}

pub fn get_highest_binary(b: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("get_highest_binary"); 
    if let Some(parent) = b.parent() {
        if let NodeData::BinaryExpression(_) = &parent.data {
            return get_highest_binary(&parent);
        }
    }
    Arc::clone(b)
}

pub fn count_binary_expression_parameters(b: &Node) -> usize { ::tsox_core::fntrace::enter("count_binary_expression_parameters"); 
    if let NodeData::BinaryExpression(d) = &b.data {
        if let NodeData::BinaryExpression(_) = &d.left.data {
            return count_binary_expression_parameters(&d.left) + 1;
        }
    }
    2
}
