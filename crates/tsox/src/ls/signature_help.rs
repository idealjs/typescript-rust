#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::node::LineMap;
use crate::ast::node_data_generated::for_each_child;
use crate::ast::{Node, NodeData, SourceFile, Symbol, SyntaxKind};
use crate::checker::{Checker, Signature};
use crate::compiler::Program;
use crate::lsp::lsproto::lsp::{DocumentUri, Position};

use super::language_service::LanguageService;
use super::types::{
    ParameterInformation, SignatureHelp, SignatureHelpContext, SignatureInformation,
};

pub struct CallInvocation {
    pub node: Arc<Node>,
}

pub struct TypeArgsInvocation {
    pub called: Arc<Node>,
}

pub struct ContextualInvocation {
    pub signature: Arc<Signature>,
    pub node: Arc<Node>,
    pub symbol: Option<Arc<Symbol>>,
}

pub enum Invocation {
    Call(CallInvocation),
    TypeArgs(TypeArgsInvocation),
    Contextual(ContextualInvocation),
}

impl LanguageService {
    pub fn provide_signature_help(
        &self,
        document_uri: &DocumentUri,
        position: Position,
        _context: &SignatureHelpContext,
    ) -> Option<SignatureHelp> {
        let (program, source_file) = self.get_program_and_file(document_uri);
        let offset = lsp_position_to_offset(&source_file.line_map, &position);
        self.get_signature_help_items(offset, &program, &source_file, _context)
    }

    pub fn get_signature_help_items(
        &self,
        position: usize,
        program: &Arc<Program>,
        source_file: &Arc<SourceFile>,
        _context: &SignatureHelpContext,
    ) -> Option<SignatureHelp> {
        let node = find_deepest_node(&source_file.node, position);

        let (call_node, argument_index) = find_enclosing_call_and_argument_index(&node, position)?;

        let mut checker = program.build_checker();

        let argument_count = (argument_index + 1) as i32;
        let (resolved, candidates) =
            checker.get_resolved_signature_for_signature_help(&call_node, argument_count);

        let signature_list: Vec<Arc<Signature>> = if candidates.is_empty() {
            resolved.iter().cloned().collect()
        } else {
            candidates
        };

        if signature_list.is_empty() {
            return None;
        }

        let signatures: Vec<SignatureInformation> = signature_list
            .iter()
            .map(|sig| signature_to_info(&checker, sig))
            .collect();

        Some(SignatureHelp {
            signatures,
            active_signature: Some(0),
            active_parameter: Some(argument_index as u32),
        })
    }
}

fn find_enclosing_call_and_argument_index(
    node: &Arc<Node>,
    position: usize,
) -> Option<(Arc<Node>, usize)> {
    let mut current = Arc::clone(node);
    loop {
        match current.kind {
            SyntaxKind::CallExpression => {
                let args = get_arguments(&current);
                let idx = count_arguments_before_position(&args, position);
                return Some((Arc::clone(&current), idx));
            }
            SyntaxKind::NewExpression => {
                let args = get_arguments(&current);
                let idx = count_arguments_before_position(&args, position);
                return Some((Arc::clone(&current), idx));
            }
            _ => {
                current = current.parent.clone()?;
            }
        }
    }
}

fn get_arguments(node: &Arc<Node>) -> Vec<Arc<Node>> {
    match &node.data {
        NodeData::CallExpression(d) => d.arguments.nodes.clone(),
        NodeData::NewExpression(d) => d
            .arguments
            .as_ref()
            .map(|a| a.nodes.clone())
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn count_arguments_before_position(args: &[Arc<Node>], position: usize) -> usize {
    let mut index = 0;
    for arg in args {
        if arg.pos() <= position {
            index += 1;
        } else {
            break;
        }
    }

    if index > 0 && args.last().map(|a| position > a.end()).unwrap_or(false) {
        index -= 1;
    }
    index
}

fn signature_to_info(checker: &Checker, sig: &Arc<Signature>) -> SignatureInformation {
    let params = &sig.parameters;
    let param_labels: Vec<String> = params
        .iter()
        .map(|p| {
            if p.name.is_empty() {
                format!(
                    "arg{}",
                    params.iter().position(|q| Arc::ptr_eq(p, q)).unwrap_or(0)
                )
            } else {
                p.name.clone()
            }
        })
        .collect();

    let name = sig
        .declaration
        .as_ref()
        .and_then(|d| d.name())
        .map(|n| n.text().to_string())
        .unwrap_or_else(|| "".to_string());

    let label = if name.is_empty() {
        format!("({})", param_labels.join(", "))
    } else {
        format!("{}({})", name, param_labels.join(", "))
    };

    let parameters: Vec<ParameterInformation> = param_labels
        .into_iter()
        .map(|label| ParameterInformation {
            label,
            documentation: None,
        })
        .collect();

    let _ = checker;
    SignatureInformation {
        label,
        documentation: None,
        parameters,
    }
}

fn find_deepest_node(node: &Arc<Node>, offset: usize) -> Arc<Node> {
    let mut deepest = Arc::clone(node);
    loop {
        let current = Arc::clone(&deepest);
        let mut next: Option<Arc<Node>> = None;
        for_each_child(&current, |child| {
            if child.pos() <= offset && offset < child.end() {
                next = Some(Arc::clone(child));
                true
            } else {
                false
            }
        });
        match next {
            Some(child) => deepest = child,
            None => break,
        }
    }
    deepest
}

fn lsp_position_to_offset(line_map: &LineMap, position: &Position) -> usize {
    let line = position.line as usize;
    let character = position.character as usize;
    let line_start = line_map.line_starts.get(line).copied().unwrap_or(0) as usize;
    line_start + character
}
