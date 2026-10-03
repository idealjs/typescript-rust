#![allow(dead_code)]

use std::sync::Arc;

use crate::lsp::lsproto_lsp::DocumentUri;
use crate::lsp::lsproto_lsp::Location;
use crate::lsp::lsproto_lsp::Position;
use crate::lsp::lsproto_lsp::Range;
use tsox_frontend::ast::Node;

use super::language_service::LanguageService;

pub type CallHierarchyDeclaration = Arc<Node>;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
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
    ) -> Vec<CallHierarchyDeclaration> { ::tsox_core::fntrace::enter("prepare_call_hierarchy"); 
        Vec::new()
    }

    pub fn provide_call_hierarchy_incoming_calls(
        &self,
        _document_uri: &DocumentUri,
        _position: Position,
    ) -> Vec<CallHierarchyIncomingCall> { ::tsox_core::fntrace::enter("provide_call_hierarchy_incoming_calls"); 
        Vec::new()
    }

    pub fn provide_call_hierarchy_outgoing_calls(
        &self,
        _document_uri: &DocumentUri,
        _position: Position,
    ) -> Vec<CallHierarchyOutgoingCall> { ::tsox_core::fntrace::enter("provide_call_hierarchy_outgoing_calls"); 
        Vec::new()
    }
}

pub fn is_named_expression(_node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_named_expression"); 
    false
}

pub fn is_variable_like(_node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_variable_like"); 
    false
}

pub fn is_possible_call_hierarchy_declaration(_node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_possible_call_hierarchy_declaration"); 
    false
}
