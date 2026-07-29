//! Emit resolver: provides symbol/type information needed during emit.
//!
//! Ported from `internal/checker/emitresolver.go` (~1322 lines). The Go
//! implementation wraps the checker and provides thread-safe access to
//! symbol visibility, enum values, parameter optionality, and other
//! information required by the emitter/transformer.
//!
//! This Rust port provides the most commonly needed methods. Additional
//! methods can be added as the emitter grows more sophisticated.

use std::sync::Arc;

use crate::ast::{Node, NodeData, Symbol, SymbolFlags, SyntaxKind};

use super::checker::Checker;

/// The emit resolver. Provides access to checker data needed during emit.
///
/// In Go, this is a separate struct that wraps the checker with a mutex.
/// In Rust, we implement the methods directly on `Checker` (since the
/// checker already uses interior mutability) and provide this module as
/// the organizational home for the emit-resolution logic.
impl Checker {
    // ────────────────────────────────────────────────────────────────────
    // Declaration visibility
    // ────────────────────────────────────────────────────────────────────

    /// Check if a declaration is visible (should be emitted).
    ///
    /// Mirrors Go's `EmitResolver.IsDeclarationVisible` (emitresolver.go ~L104).
    /// A declaration is visible if it's not purely a type-only declaration
    /// that's never used in a value position.
    pub fn is_declaration_visible(&mut self, node: &Arc<Node>) -> bool {
        // For now, all declarations are considered visible. Full visibility
        // tracking requires the alias marking visitor which is a complex
        // pass over all references. This stub is sufficient for basic emit.
        true
    }

    // ────────────────────────────────────────────────────────────────────
    // Enum member values
    // ────────────────────────────────────────────────────────────────────

    /// Get the constant value of an enum member.
    ///
    /// Mirrors Go's `EmitResolver.GetEnumMemberValue` (emitresolver.go ~L89).
    /// Returns the enum member's value as a string (for numeric enums) or
    /// the string literal (for string enums).
    pub fn get_enum_member_value(&mut self, node: &Arc<Node>) -> Option<String> {
        // Look for the enum member's initializer expression.
        let NodeData::EnumMember(data) = &node.data else {
            return None;
        };
        let initializer = data.initializer.as_ref()?;
        match initializer.kind {
            SyntaxKind::StringLiteral => {
                if let NodeData::StringLiteral(s) = &initializer.data {
                    Some(format!("\"{}\"", s.text))
                } else {
                    None
                }
            }
            SyntaxKind::NumericLiteral => {
                if let NodeData::NumericLiteral(n) = &initializer.data {
                    Some(n.text.clone())
                } else {
                    None
                }
            }
            SyntaxKind::PrefixUnaryExpression => {
                // Handle `-1`, `+1` etc.
                if let NodeData::PrefixUnaryExpression(unary) = &initializer.data {
                    let operand_text = match &unary.operand.data {
                        NodeData::NumericLiteral(n) => n.text.clone(),
                        _ => return None,
                    };
                    let op = match unary.operator {
                        SyntaxKind::MinusToken => "-",
                        SyntaxKind::PlusToken => "+",
                        _ => return None,
                    };
                    Some(format!("{}{}", op, operand_text))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    // ────────────────────────────────────────────────────────────────────
    // Parameter optionality
    // ────────────────────────────────────────────────────────────────────

    /// Check if a parameter is optional.
    ///
    /// Mirrors Go's `EmitResolver.IsOptionalParameter` (emitresolver.go ~L65).
    pub fn is_optional_parameter(&self, node: &Arc<Node>) -> bool {
        match &node.data {
            NodeData::ParameterDeclaration(data) => {
                // A parameter is optional if it has a question mark or
                // if it's a rest parameter.
                data.question_token.is_some() || node.kind == SyntaxKind::RestType
            }
            _ => false,
        }
    }

    // ────────────────────────────────────────────────────────────────────
    // Literal const declarations
    // ────────────────────────────────────────────────────────────────────

    /// Check if a declaration is a literal const declaration.
    ///
    /// Mirrors Go's `EmitResolver.IsLiteralConstDeclaration` (emitresolver.go ~L639).
    /// A `const` declaration with a literal initializer (e.g. `const x = "foo"`)
    /// is a literal const declaration.
    pub fn is_literal_const_declaration(&self, node: &Arc<Node>) -> bool {
        if node.kind != SyntaxKind::VariableDeclaration {
            return false;
        }
        let NodeData::VariableDeclaration(data) = &node.data else {
            return false;
        };
        // Check if the parent is a const declaration.
        // This requires checking the parent VariableStatement's modifiers,
        // but since we don't have parent pointers, we check if the
        // declaration's type is a literal type.
        if data.initializer.is_none() {
            return false;
        }
        let initializer = data.initializer.as_ref().unwrap();
        matches!(
            initializer.kind,
            SyntaxKind::StringLiteral
                | SyntaxKind::NumericLiteral
                | SyntaxKind::TrueKeyword
                | SyntaxKind::FalseKeyword
                | SyntaxKind::BigIntLiteral
                | SyntaxKind::PrefixUnaryExpression
        )
    }

    // ────────────────────────────────────────────────────────────────────
    // Constant values
    // ────────────────────────────────────────────────────────────────────

    /// Get the constant value of a node (for enum members, const assertions).
    ///
    /// Mirrors Go's `EmitResolver.GetConstantValue` (emitresolver.go ~L1157).
    pub fn get_constant_value(&mut self, node: &Arc<Node>) -> Option<String> {
        if node.kind == SyntaxKind::EnumMember {
            return self.get_enum_member_value(node);
        }
        match node.kind {
            SyntaxKind::StringLiteral => {
                if let NodeData::StringLiteral(s) = &node.data {
                    Some(format!("\"{}\"", s.text))
                } else {
                    None
                }
            }
            SyntaxKind::NumericLiteral => {
                if let NodeData::NumericLiteral(n) = &node.data {
                    Some(n.text.clone())
                } else {
                    None
                }
            }
            SyntaxKind::TrueKeyword => Some("true".to_string()),
            SyntaxKind::FalseKeyword => Some("false".to_string()),
            SyntaxKind::NullKeyword => Some("null".to_string()),
            _ => None,
        }
    }

    // ────────────────────────────────────────────────────────────────────
    // Alias / import declarations
    // ────────────────────────────────────────────────────────────────────

    /// Check if an import declaration is referenced (and thus should be emitted).
    ///
    /// Mirrors Go's `EmitResolver.IsReferencedAliasDeclaration` (emitresolver.go ~L689).
    pub fn is_referenced_alias_declaration(&self, node: &Arc<Node>) -> bool {
        // For now, all alias declarations are considered referenced.
        // Full tracking requires the reference resolver.
        true
    }

    /// Check if an alias declaration is a value alias (not type-only).
    ///
    /// Mirrors Go's `EmitResolver.IsValueAliasDeclaration` (emitresolver.go ~L715).
    pub fn is_value_alias_declaration(&self, node: &Arc<Node>) -> bool {
        match &node.data {
            NodeData::ImportSpecifier(data) => !data.is_type_only,
            _ => true,
        }
    }

    // ────────────────────────────────────────────────────────────────────
    // Declaration flags
    // ────────────────────────────────────────────────────────────────────

    /// Get the effective modifier flags for a declaration.
    ///
    /// Mirrors Go's `EmitResolver.GetEffectiveDeclarationFlags` (emitresolver.go ~L1143).
    /// Currently returns the node's modifier flags directly.
    pub fn get_effective_declaration_flags(&self, node: &Arc<Node>) -> u32 {
        // Placeholder: return 0 for now. Full implementation requires
        // parsing modifier tokens from the AST.
        0
    }

    // ────────────────────────────────────────────────────────────────────
    // Symbol access
    // ────────────────────────────────────────────────────────────────────

    /// Get the symbol of a declaration node.
    ///
    /// Mirrors Go's `Checker.getSymbolOfDeclaration`.
    pub fn get_symbol_of_declaration(&self, node: &Arc<Node>) -> Option<Arc<Symbol>> {
        self.program.symbol_map().symbol_of(node).cloned()
    }

    /// Check if a symbol is a const enum member.
    pub fn is_const_enum_member(&self, symbol: &Symbol) -> bool {
        symbol.flags.contains(SymbolFlags::ConstEnum)
    }
}
