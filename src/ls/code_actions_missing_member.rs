//! Missing member fixer code action
//! (1:1 port of Go's `internal/ls/codeactions_missingmemberfixer.go`).

#![allow(dead_code)]

use std::sync::Arc;

use crate::ast::{Node, Symbol};
use crate::checker::Checker;
use crate::compiler::Program;
use crate::ls::lsutil::UserPreferences;

use super::language_service::LanguageService;

/// Flags for preserving optionality when creating members.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreserveOptionalFlags(pub u32);

pub const PRESERVE_OPTIONAL_FLAGS_METHOD: u32 = 1 << 0;
pub const PRESERVE_OPTIONAL_FLAGS_PROPERTY: u32 = 1 << 1;
pub const PRESERVE_OPTIONAL_FLAGS_ALL: u32 =
    PRESERVE_OPTIONAL_FLAGS_METHOD | PRESERVE_OPTIONAL_FLAGS_PROPERTY;

/// The missing-member fixer.
pub struct MissingMemberFixer<'a> {
    pub type_checker: &'a Checker,
    pub program: &'a Program,
    pub preferences: &'a UserPreferences,
}

impl<'a> MissingMemberFixer<'a> {
    pub fn new(
        type_checker: &'a Checker,
        program: &'a Program,
        preferences: &'a UserPreferences,
    ) -> Self {
        MissingMemberFixer {
            type_checker,
            program,
            preferences,
        }
    }

    /// Create a member declaration from a symbol.
    ///
    /// Mirrors `createMemberFromSymbol`.
    pub fn create_member_from_symbol(
        &self,
        _symbol: &Arc<Symbol>,
        _enclosing_declaration: &Arc<Node>,
        _source_file: &Arc<crate::ast::SourceFile>,
        _preserve_optional: u32,
    ) -> Vec<Arc<Node>> {
        // TODO: requires nodebuilder + printer
        Vec::new()
    }
}

impl LanguageService {
    /// Create a missing-member fixer.
    pub fn new_missing_member_fixer<'a>(
        &'a self,
        _program: &'a Program,
        _type_checker: &'a Checker,
    ) -> MissingMemberFixer<'a> {
        MissingMemberFixer::new(_type_checker, _program, self.user_preferences())
    }
}
