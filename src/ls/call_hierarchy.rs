#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::Node;
use crate::lsp::lsproto::lsp::{DocumentUri, Location, Position, Range};

use super::language_service::LanguageService;

pub type CallHierarchyDeclaration = Arc<Node>;

#[derive(Debug, Clone, Default)]
pub struct CallHierarchyIncomingCall {
    pub from: Location,
    pub from_ranges: Vec<Range>,
}

#[derive(Debug, Clone, Default)]
pub struct CallHierarchyOutgoingCall {
    pub to: Location,
    pub from_ranges: Vec<Range>,
}

impl LanguageService {

    pub fn prepare_call_hierarchy(
        &self,
        _document_uri: &DocumentUri,
        _position: Position,
    ) -> Vec<CallHierarchyDeclaration> {

        Vec::new()
    }

    pub fn provide_call_hierarchy_incoming_calls(
        &self,
        _document_uri: &DocumentUri,
        _position: Position,
    ) -> Vec<CallHierarchyIncomingCall> {

        Vec::new()
    }

    pub fn provide_call_hierarchy_outgoing_calls(
        &self,
        _document_uri: &DocumentUri,
        _position: Position,
    ) -> Vec<CallHierarchyOutgoingCall> {

        Vec::new()
    }
}

pub fn is_named_expression(_node: &Arc<Node>) -> bool {

    false
}

pub fn is_variable_like(_node: &Arc<Node>) -> bool {

    false
}

pub fn is_possible_call_hierarchy_declaration(_node: &Arc<Node>) -> bool {

    false
}
