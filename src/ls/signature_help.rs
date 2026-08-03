//! Signature help provider (1:1 port of Go's `internal/ls/signaturehelp.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, Symbol};
use crate::checker::{Checker, Signature};
use crate::compiler::Program;
use crate::lsp::lsproto::lsp::{DocumentUri, Position};

use super::language_service::LanguageService;
use super::types::{SignatureHelp, SignatureHelpContext};

/// A call invocation.
pub struct CallInvocation {
    pub node: Arc<Node>,
}

/// A type-args invocation.
pub struct TypeArgsInvocation {
    pub called: Arc<Node>,
}

/// A contextual invocation.
pub struct ContextualInvocation {
    pub signature: Arc<Signature>,
    pub node: Arc<Node>,
    pub symbol: Option<Arc<Symbol>>,
}

/// An invocation (one of call, type-args, or contextual).
pub enum Invocation {
    Call(CallInvocation),
    TypeArgs(TypeArgsInvocation),
    Contextual(ContextualInvocation),
}

impl LanguageService {
    /// Provide signature help for a position.
    ///
    /// Mirrors `ProvideSignatureHelp`.
    pub fn provide_signature_help(
        &self,
        _document_uri: &DocumentUri,
        _position: Position,
        _context: &SignatureHelpContext,
    ) -> Option<SignatureHelp> {
        // TODO: requires checker + AST navigation
        None
    }

    /// Get signature help items.
    ///
    /// Mirrors `GetSignatureHelpItems`.
    pub fn get_signature_help_items(
        &self,
        _position: usize,
        _program: &Program,
        _source_file: &Arc<crate::ast::SourceFile>,
        _context: &SignatureHelpContext,
    ) -> Option<SignatureHelp> {
        // TODO: requires checker.GetResolvedSignature + argument list analysis
        None
    }
}
