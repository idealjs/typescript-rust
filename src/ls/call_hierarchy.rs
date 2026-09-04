//! Call hierarchy provider (1:1 port of Go's `internal/ls/callhierarchy.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::Node;
use crate::lsp::lsproto::lsp::{DocumentUri, Location, Position, Range};

use super::language_service::LanguageService;

/// A call hierarchy declaration is just a node.
pub type CallHierarchyDeclaration = Arc<Node>;

/// Incoming call item.
#[derive(Debug, Clone, Default)]
pub struct CallHierarchyIncomingCall {
    pub from: Location,
    pub from_ranges: Vec<Range>,
}

/// Outgoing call item.
#[derive(Debug, Clone, Default)]
pub struct CallHierarchyOutgoingCall {
    pub to: Location,
    pub from_ranges: Vec<Range>,
}

impl LanguageService {
    /// Prepare call hierarchy for a position.
    ///
    /// Mirrors `PrepareCallHierarchy`.
    pub fn prepare_call_hierarchy(
        &self,
        _document_uri: &DocumentUri,
        _position: Position,
    ) -> Vec<CallHierarchyDeclaration> {
        // TODO: requires astnav + checker
        Vec::new()
    }

    /// Provide incoming calls.
    ///
    /// Mirrors `ProvideCallHierarchyIncomingCalls`.
    pub fn provide_call_hierarchy_incoming_calls(
        &self,
        _document_uri: &DocumentUri,
        _position: Position,
    ) -> Vec<CallHierarchyIncomingCall> {
        // TODO: requires reference search
        Vec::new()
    }

    /// Provide outgoing calls.
    ///
    /// Mirrors `ProvideCallHierarchyOutgoingCalls`.
    pub fn provide_call_hierarchy_outgoing_calls(
        &self,
        _document_uri: &DocumentUri,
        _position: Position,
    ) -> Vec<CallHierarchyOutgoingCall> {
        // TODO: requires call-expression traversal
        Vec::new()
    }
}

/// Indicates whether a node is a named function or class expression.
///
/// Mirrors `isNamedExpression`.
pub fn is_named_expression(_node: &Arc<Node>) -> bool {
    // TODO: requires AST kind + name checks
    false
}

/// Indicates whether a node is a variable-like declaration.
///
/// Mirrors `isVariableLike`.
pub fn is_variable_like(_node: &Arc<Node>) -> bool {
    // TODO: requires AST kind checks
    false
}

/// Indicates whether a node could possibly be a call hierarchy declaration.
///
/// Mirrors `isPossibleCallHierarchyDeclaration`.
pub fn is_possible_call_hierarchy_declaration(_node: &Arc<Node>) -> bool {
    // TODO: requires AST kind checks
    false
}
