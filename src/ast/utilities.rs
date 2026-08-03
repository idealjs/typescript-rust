#![allow(dead_code)]

//! AST utility functions ported from `internal/ast/utilities.go`.
//!
//! This module provides the composite node-type predicates (e.g.
//! `is_expression`, `is_statement`, `is_class_like`), tree-walking
//! helpers (`find_ancestor`, `get_source_file_of_node`), and node-state
//! checks (`node_is_missing`, `node_is_synthesized`) that are NOT
//! already emitted by the code generator in `node_data_generated.rs`.
//!
//! The generated file already contains:
//! - Simple per-kind predicates (`is_identifier`, `is_class_declaration`, …)
//! - Kind-class predicates (`is_literal_kind`, `is_token_kind`, …)
//! - Operator predicates (`is_assignment_operator`, `is_binary_operator`, …)
//! - `for_each_child`
//!
//! Only functions that aggregate kinds, walk parents, or provide
//! higher-level logic belong here.

use super::SyntaxKind;
use super::node::{Node, NodeList, SourceFile};
use super::node_data_generated::*;
use super::node_flags::{ModifierFlags, NodeFlags};
use crate::core::text::TextRange;
use std::sync::Arc;

// ────────────────────────────────────────────────────────────────────────────
// Node state checks
// ────────────────────────────────────────────────────────────────────────────

/// Determines if a node is missing (zero-width range and not EOF).
///
/// Mirrors `ast.NodeIsMissing` in Go.
pub fn node_is_missing(node: Option<&Arc<Node>>) -> bool {
    match node {
        None => true,
        Some(n) => n.pos() == n.end() && (n.pos() as i32) >= 0 && n.kind != SyntaxKind::EndOfFile,
    }
}

/// Determines if a node is present.
///
/// Mirrors `ast.NodeIsPresent` in Go.
pub fn node_is_present(node: Option<&Arc<Node>>) -> bool {
    !node_is_missing(node)
}

/// Determines if a node contains synthetic positions.
///
/// Mirrors `ast.NodeIsSynthesized` in Go.
pub fn node_is_synthesized(node: &Node) -> bool {
    position_is_synthesized(node.pos()) || position_is_synthesized(node.end())
}

/// Whether a position is synthetic (negative).
///
/// Mirrors `ast.PositionIsSynthesized` in Go.
pub fn position_is_synthesized(pos: usize) -> bool {
    (pos as i32) < 0
}

/// Whether a range is synthesized.
///
/// Mirrors `ast.RangeIsSynthesized` in Go.
pub fn range_is_synthesized(loc: TextRange) -> bool {
    position_is_synthesized(loc.pos()) || position_is_synthesized(loc.end())
}

// ────────────────────────────────────────────────────────────────────────────
// Token / operator predicates
// ────────────────────────────────────────────────────────────────────────────

/// Whether a token kind is a compound assignment operator (`+=`, etc.).
///
/// Mirrors `ast.IsCompoundAssignment` in Go.
pub fn is_compound_assignment(token: SyntaxKind) -> bool {
    (token as i16) >= (SyntaxKind::PlusEqualsToken as i16)
        && (token as i16) <= (SyntaxKind::CaretEqualsToken as i16)
}

/// Whether a token kind is `||` or `&&`.
///
/// Mirrors `ast.IsLogicalBinaryOperator` in Go.
pub fn is_logical_binary_operator(token: SyntaxKind) -> bool {
    token == SyntaxKind::BarBarToken || token == SyntaxKind::AmpersandAmpersandToken
}

/// Whether a token kind is `||`, `&&`, or `??`.
///
/// Mirrors `ast.IsLogicalOrCoalescingBinaryOperator` in Go.
pub fn is_logical_or_coalescing_binary_operator(token: SyntaxKind) -> bool {
    is_logical_binary_operator(token) || token == SyntaxKind::QuestionQuestionToken
}

// Note: `is_logical_or_coalescing_assignment_operator` is already
// generated in `node_data_generated.rs`.

// ────────────────────────────────────────────────────────────────────────────
// Expression classification
// ────────────────────────────────────────────────────────────────────────────

/// Skips past partially-emitted expressions.
///
/// Mirrors `ast.SkipPartiallyEmittedExpressions` in Go.
pub fn skip_partially_emitted_expressions_arc(node: &Arc<Node>) -> Arc<Node> {
    let mut current = Arc::clone(node);
    while is_partially_emitted_expression(&current) {
        if let Some(inner) = current.expression() {
            current = Arc::clone(inner);
        } else {
            break;
        }
    }
    current
}

fn is_left_hand_side_expression_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::PropertyAccessExpression
            | SyntaxKind::ElementAccessExpression
            | SyntaxKind::NewExpression
            | SyntaxKind::CallExpression
            | SyntaxKind::JsxElement
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxFragment
            | SyntaxKind::TaggedTemplateExpression
            | SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::ClassExpression
            | SyntaxKind::FunctionExpression
            | SyntaxKind::Identifier
            | SyntaxKind::PrivateIdentifier
            | SyntaxKind::RegularExpressionLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::StringLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::TemplateExpression
            | SyntaxKind::FalseKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::ThisKeyword
            | SyntaxKind::TrueKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::NonNullExpression
            | SyntaxKind::ExpressionWithTypeArguments
            | SyntaxKind::MetaProperty
            | SyntaxKind::ImportKeyword
            | SyntaxKind::MissingDeclaration
    )
}

/// Whether a node is a `LeftHandSideExpression`.
///
/// Mirrors `ast.IsLeftHandSideExpression` in Go.
pub fn is_left_hand_side_expression(node: &Node) -> bool {
    is_left_hand_side_expression_kind(skip_partially_emitted_expressions_kind(node))
}

/// Whether a kind is a unary-expression kind.
///
/// Mirrors `ast.isUnaryExpressionKind` in Go.
fn is_unary_expression_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::PrefixUnaryExpression
            | SyntaxKind::PostfixUnaryExpression
            | SyntaxKind::DeleteExpression
            | SyntaxKind::TypeOfExpression
            | SyntaxKind::VoidExpression
            | SyntaxKind::AwaitExpression
            | SyntaxKind::TypeAssertionExpression
    ) || is_left_hand_side_expression_kind(kind)
}

/// Whether a node is a `UnaryExpression`.
///
/// Mirrors `ast.IsUnaryExpression` in Go.
pub fn is_unary_expression(node: &Node) -> bool {
    is_unary_expression_kind(skip_partially_emitted_expressions_kind(node))
}

/// Skip partially-emitted expressions and return the resulting kind.
fn skip_partially_emitted_expressions_kind(node: &Node) -> SyntaxKind {
    let mut current = node;
    loop {
        if !is_partially_emitted_expression(current) {
            return current.kind;
        }
        match current.expression() {
            Some(inner) => current = inner,
            None => return current.kind,
        }
    }
}

/// Whether a kind is an expression kind.
///
/// Mirrors `ast.isExpressionKind` in Go.
fn is_expression_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ConditionalExpression
            | SyntaxKind::YieldExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::BinaryExpression
            | SyntaxKind::SpreadElement
            | SyntaxKind::AsExpression
            | SyntaxKind::OmittedExpression
            | SyntaxKind::PartiallyEmittedExpression
            | SyntaxKind::SatisfiesExpression
    ) || is_unary_expression_kind(kind)
}

/// Whether a node is an expression.
///
/// Mirrors `ast.IsExpression` in Go.
pub fn is_expression(node: &Node) -> bool {
    is_expression_kind(skip_partially_emitted_expressions_kind(node))
}

/// Whether a node is a comma expression (`a, b`).
///
/// Mirrors `ast.IsCommaExpression` in Go.
pub fn is_comma_expression(node: &Node) -> bool {
    if let NodeData::BinaryExpression(d) = &node.data {
        return d.operator_token.kind == SyntaxKind::CommaToken;
    }
    false
}

/// Whether a node is `a ?? b`.
///
/// Mirrors `ast.IsNullishCoalesce` in Go.
pub fn is_nullish_coalesce(node: &Node) -> bool {
    if let NodeData::BinaryExpression(d) = &node.data {
        return d.operator_token.kind == SyntaxKind::QuestionQuestionToken;
    }
    false
}

/// Whether a node is a type assertion (`<T>x` or `x as T`).
///
/// Mirrors `ast.IsAssertionExpression` in Go.
pub fn is_assertion_expression(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::TypeAssertionExpression | SyntaxKind::AsExpression
    )
}

/// Whether a node is a `boolean` literal (`true` / `false`).
///
/// Mirrors `ast.IsBooleanLiteral` in Go.
pub fn is_boolean_literal(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword
    )
}

/// Whether a node is a literal expression (numeric, string, etc.).
///
/// Mirrors `ast.IsLiteralExpression` in Go.
pub fn is_literal_expression(node: &Node) -> bool {
    is_literal_kind(node.kind)
}

/// Whether a node is a string-literal-like (StringLiteral or
/// NoSubstitutionTemplateLiteral).
///
/// Mirrors `ast.IsStringLiteralLike` in Go.
pub fn is_string_literal_like(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral
    )
}

/// Whether a node is a string- or numeric-literal-like.
///
/// Mirrors `ast.IsStringOrNumericLiteralLike` in Go.
pub fn is_string_or_numeric_literal_like(node: &Node) -> bool {
    is_string_literal_like(node) || is_numeric_literal(node)
}

/// Whether a node is an access expression (property or element access).
///
/// Mirrors `ast.IsAccessExpression` in Go.
pub fn is_access_expression(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression
    )
}

/// Whether a node is part of an optional chain.
///
/// Mirrors `ast.IsOptionalChain` in Go.
pub fn is_optional_chain(node: &Node) -> bool {
    if node.flags.contains(NodeFlags::OptionalChain) {
        matches!(
            node.kind,
            SyntaxKind::PropertyAccessExpression
                | SyntaxKind::ElementAccessExpression
                | SyntaxKind::CallExpression
                | SyntaxKind::NonNullExpression
        )
    } else {
        false
    }
}

/// Whether a node is an assignment expression.
///
/// Mirrors `ast.IsAssignmentExpression` in Go.
pub fn is_assignment_expression(node: &Node, exclude_compound_assignment: bool) -> bool {
    if let NodeData::BinaryExpression(d) = &node.data {
        return (d.operator_token.kind == SyntaxKind::EqualsToken
            || (!exclude_compound_assignment && is_assignment_operator(d.operator_token.kind)))
            && is_left_hand_side_expression(&d.left);
    }
    false
}

// ────────────────────────────────────────────────────────────────────────────
// Statement classification
// ────────────────────────────────────────────────────────────────────────────

/// Whether a kind is a declaration-statement kind.
///
/// Mirrors `ast.isDeclarationStatementKind` in Go.
fn is_declaration_statement_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::MissingDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::JSImportDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::ExportAssignment
            | SyntaxKind::NamespaceExportDeclaration
    )
}

/// Whether a node is a DeclarationStatement.
///
/// Mirrors `ast.IsDeclarationStatement` in Go.
pub fn is_declaration_statement(node: &Node) -> bool {
    is_declaration_statement_kind(node.kind)
}

/// Whether a kind is a statement-but-not-declaration kind.
///
/// Mirrors `ast.isStatementKindButNotDeclarationKind` in Go.
fn is_statement_kind_but_not_declaration_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::BreakStatement
            | SyntaxKind::ContinueStatement
            | SyntaxKind::DebuggerStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::ExpressionStatement
            | SyntaxKind::EmptyStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::ForStatement
            | SyntaxKind::IfStatement
            | SyntaxKind::LabeledStatement
            | SyntaxKind::ReturnStatement
            | SyntaxKind::SwitchStatement
            | SyntaxKind::ThrowStatement
            | SyntaxKind::TryStatement
            | SyntaxKind::VariableStatement
            | SyntaxKind::WhileStatement
            | SyntaxKind::WithStatement
            | SyntaxKind::NotEmittedStatement
    )
}

/// Whether a node is a statement that is not also a declaration.
///
/// Mirrors `ast.IsStatementButNotDeclaration` in Go.
pub fn is_statement_but_not_declaration(node: &Node) -> bool {
    is_statement_kind_but_not_declaration_kind(node.kind)
}

/// Whether a node is the Block-like body of a function.
///
/// Mirrors `ast.IsFunctionBlock` in Go.
pub fn is_function_block(node: &Node) -> bool {
    if node.kind != SyntaxKind::Block {
        return false;
    }
    match &node.parent {
        Some(parent) => is_function_like(parent),
        None => false,
    }
}

/// Whether a node is a block statement (Block that is not a function body
/// or part of try/catch).
///
/// Mirrors `ast.isBlockStatement` in Go.
pub fn is_block_statement(node: &Node) -> bool {
    if node.kind != SyntaxKind::Block {
        return false;
    }
    if let Some(parent) = &node.parent {
        if parent.kind == SyntaxKind::TryStatement || parent.kind == SyntaxKind::CatchClause {
            return false;
        }
    }
    !is_function_block(node)
}

/// Whether a node is a Statement (declaration or non-declaration).
///
/// Mirrors `ast.IsStatement` in Go.
pub fn is_statement(node: &Node) -> bool {
    let kind = node.kind;
    is_statement_kind_but_not_declaration_kind(kind)
        || is_declaration_statement_kind(kind)
        || is_block_statement(node)
}

/// Whether a node is an iteration statement (for/while/do-while/for-in/for-of).
///
/// Mirrors `ast.IsIterationStatement` in Go.
pub fn is_iteration_statement(node: &Node, look_in_labeled_statements: bool) -> bool {
    match node.kind {
        SyntaxKind::ForStatement
        | SyntaxKind::ForInStatement
        | SyntaxKind::ForOfStatement
        | SyntaxKind::DoStatement
        | SyntaxKind::WhileStatement => true,
        SyntaxKind::LabeledStatement => {
            if look_in_labeled_statements {
                if let NodeData::LabeledStatement(d) = &node.data {
                    return is_iteration_statement(&d.statement, look_in_labeled_statements);
                }
            }
            false
        }
        _ => false,
    }
}

/// Whether a node is a prologue directive (`"use strict";`).
///
/// Mirrors `ast.IsPrologueDirective` in Go.
pub fn is_prologue_directive(node: &Node) -> bool {
    if node.kind != SyntaxKind::ExpressionStatement {
        return false;
    }
    match node.expression() {
        Some(expr) => expr.kind == SyntaxKind::StringLiteral,
        None => false,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Declaration classification
// ────────────────────────────────────────────────────────────────────────────

/// Whether a kind is a function-like declaration kind.
///
/// Mirrors `ast.isFunctionLikeDeclarationKind` in Go.
fn is_function_like_declaration_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
    )
}

/// Whether a node is a function-like declaration.
///
/// Mirrors `ast.IsFunctionLikeDeclaration` in Go.
pub fn is_function_like_declaration(node: &Node) -> bool {
    is_function_like_declaration_kind(node.kind)
}

/// Whether a kind is function-like (declarations + signatures).
///
/// Mirrors `ast.IsFunctionLikeKind` in Go.
pub fn is_function_like_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::MethodSignature
            | SyntaxKind::CallSignature
            | SyntaxKind::JSDocSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::IndexSignature
            | SyntaxKind::FunctionType
            | SyntaxKind::ConstructorType
    ) || is_function_like_declaration_kind(kind)
}

/// Whether a node is function- or signature-like.
///
/// Mirrors `ast.IsFunctionLike` in Go.
pub fn is_function_like(node: &Node) -> bool {
    is_function_like_kind(node.kind)
}

/// Whether a node is function-like or a class static block.
///
/// Mirrors `ast.IsFunctionLikeOrClassStaticBlockDeclaration` in Go.
pub fn is_function_like_or_class_static_block_declaration(node: &Node) -> bool {
    is_function_like(node) || is_class_static_block_declaration(node)
}

/// Whether a node is a function or source file.
///
/// Mirrors `ast.IsFunctionOrSourceFile` in Go.
pub fn is_function_or_source_file(node: &Node) -> bool {
    is_function_like(node) || is_source_file(node)
}

/// Whether a node is class-like (class declaration or expression).
///
/// Mirrors `ast.IsClassLike` in Go.
pub fn is_class_like(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
    )
}

/// Whether a node is class- or interface-like.
///
/// Mirrors `ast.IsClassOrInterfaceLike` in Go.
pub fn is_class_or_interface_like(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::InterfaceDeclaration
    )
}

/// Whether a node is a class element.
///
/// Mirrors `ast.IsClassElement` in Go.
pub fn is_class_element(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::Constructor
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::IndexSignature
            | SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::SemicolonClassElement
    )
}

/// Whether a node is a method or accessor.
///
/// Mirrors `ast.IsMethodOrAccessor` in Go.
pub fn is_method_or_accessor(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::MethodDeclaration | SyntaxKind::GetAccessor | SyntaxKind::SetAccessor
    )
}

/// Whether a node is a type element (interface member).
///
/// Mirrors `ast.IsTypeElement` in Go.
pub fn is_type_element(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ConstructSignature
            | SyntaxKind::CallSignature
            | SyntaxKind::PropertySignature
            | SyntaxKind::MethodSignature
            | SyntaxKind::IndexSignature
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::NotEmittedTypeElement
    )
}

/// Whether a node is an object-literal element.
///
/// Mirrors `ast.IsObjectLiteralElement` in Go.
pub fn is_object_literal_element(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::PropertyAssignment
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::SpreadAssignment
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
    )
}

/// Whether a node is an accessor (get or set).
///
/// Mirrors `ast.IsAccessor` in Go.
pub fn is_accessor(node: &Node) -> bool {
    matches!(node.kind, SyntaxKind::GetAccessor | SyntaxKind::SetAccessor)
}

/// Whether a node is a module or enum declaration.
///
/// Mirrors `ast.IsModuleOrEnumDeclaration` in Go.
pub fn is_module_or_enum_declaration(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ModuleDeclaration | SyntaxKind::EnumDeclaration
    )
}

/// Whether a node is a function expression or arrow function.
///
/// Mirrors `ast.IsFunctionExpressionOrArrowFunction` in Go.
pub fn is_function_expression_or_arrow_function(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction
    )
}

/// Whether a node is a JSX child.
///
/// Mirrors `ast.IsJsxChild` in Go.
pub fn is_jsx_child(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::JsxElement
            | SyntaxKind::JsxExpression
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxText
            | SyntaxKind::JsxFragment
    )
}

/// Whether a node is a JSX attribute-like (attribute or spread attribute).
///
/// Mirrors `ast.IsJsxAttributeLike` in Go.
pub fn is_jsx_attribute_like(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::JsxAttribute | SyntaxKind::JsxSpreadAttribute
    )
}

/// Whether a node is an import or export specifier.
///
/// Mirrors `ast.IsImportOrExportSpecifier` in Go.
pub fn is_import_or_export_specifier(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ImportSpecifier | SyntaxKind::ExportSpecifier
    )
}

/// Whether a node is a break or continue statement.
///
/// Mirrors `ast.IsBreakOrContinueStatement` in Go.
pub fn is_break_or_continue_statement(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::BreakStatement | SyntaxKind::ContinueStatement
    )
}

/// Whether a node is a property access or qualified name.
///
/// Mirrors `ast.IsPropertyAccessOrQualifiedName` in Go.
pub fn is_property_access_or_qualified_name(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::PropertyAccessExpression | SyntaxKind::QualifiedName
    )
}

// ────────────────────────────────────────────────────────────────────────────
// Name classification
// ────────────────────────────────────────────────────────────────────────────

/// Whether a node is a property-name literal (identifier, string, template,
/// numeric).
///
/// Mirrors `ast.IsPropertyNameLiteral` in Go.
pub fn is_property_name_literal(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::Identifier
            | SyntaxKind::StringLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::NumericLiteral
    )
}

/// Whether a node is a member name (identifier or private identifier).
///
/// Mirrors `ast.IsMemberName` in Go.
pub fn is_member_name(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::Identifier | SyntaxKind::PrivateIdentifier
    )
}

/// Whether a node is an entity name (identifier or qualified name).
///
/// Mirrors `ast.IsEntityName` in Go.
pub fn is_entity_name(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::Identifier | SyntaxKind::QualifiedName
    )
}

/// Whether a node is a property name.
///
/// Mirrors `ast.IsPropertyName` in Go.
pub fn is_property_name_node(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::Identifier
            | SyntaxKind::PrivateIdentifier
            | SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::ComputedPropertyName
    )
}

/// Whether a node is an entity-name expression (identifier or chained
/// property access of identifiers).
///
/// Mirrors `ast.IsEntityNameExpression` in Go.
pub fn is_entity_name_expression(node: &Node) -> bool {
    is_identifier(node)
        || (is_property_access_expression(node) && {
            if let NodeData::PropertyAccessExpression(d) = &node.data {
                is_identifier(&d.name) && is_entity_name_expression(&d.expression)
            } else {
                false
            }
        })
}

/// Whether a node is a modifier or decorator.
///
/// Mirrors `ast.IsModifierLike` in Go.
pub fn is_modifier_like(node: &Node) -> bool {
    is_modifier_kind(node.kind) || is_decorator(node)
}

// ────────────────────────────────────────────────────────────────────────────
// Type node classification
// ────────────────────────────────────────────────────────────────────────────

/// Whether a kind is a type-node kind.
///
/// Mirrors `ast.IsTypeNodeKind` in Go.
pub fn is_type_node_kind(kind: SyntaxKind) -> bool {
    if matches!(
        kind,
        SyntaxKind::AnyKeyword
            | SyntaxKind::UnknownKeyword
            | SyntaxKind::NumberKeyword
            | SyntaxKind::BigIntKeyword
            | SyntaxKind::ObjectKeyword
            | SyntaxKind::BooleanKeyword
            | SyntaxKind::StringKeyword
            | SyntaxKind::SymbolKeyword
            | SyntaxKind::VoidKeyword
            | SyntaxKind::UndefinedKeyword
            | SyntaxKind::NeverKeyword
            | SyntaxKind::IntrinsicKeyword
            | SyntaxKind::ExpressionWithTypeArguments
            | SyntaxKind::JSDocAllType
            | SyntaxKind::JSDocNullableType
            | SyntaxKind::JSDocNonNullableType
            | SyntaxKind::JSDocOptionalType
            | SyntaxKind::JSDocVariadicType
    ) {
        return true;
    }
    // FirstTypeNode .. LastTypeNode range
    (kind as i16) >= (SyntaxKind::TypePredicate as i16)
        && (kind as i16) <= (SyntaxKind::ImportType as i16)
}

/// Whether a node is a type node.
///
/// Mirrors `ast.IsTypeNode` in Go.
pub fn is_type_node(node: &Node) -> bool {
    is_type_node_kind(node.kind)
}

/// Whether a node is a JSDoc kind.
///
/// Mirrors `ast.IsJSDocKind` in Go.
pub fn is_jsdoc_kind(kind: SyntaxKind) -> bool {
    (kind as i16) >= (SyntaxKind::JSDocTypeExpression as i16)
        && (kind as i16) <= (SyntaxKind::JSDocImportTag as i16)
}

/// Whether a node is a JSDoc node.
///
/// Mirrors `ast.IsJSDocNode` in Go.
pub fn is_jsdoc_node(node: &Node) -> bool {
    is_jsdoc_kind(node.kind)
}

/// Whether a node is a JSDoc tag.
///
/// Mirrors `ast.IsJSDocTag` in Go.
pub fn is_jsdoc_tag(node: &Node) -> bool {
    (node.kind as i16) >= (SyntaxKind::JSDocUnknownTag as i16)
        && (node.kind as i16) <= (SyntaxKind::JSDocImportTag as i16)
}

/// Whether a node is a JSDoc link-like.
///
/// Mirrors `ast.IsJSDocLinkLike` in Go.
pub fn is_jsdoc_link_like(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::JSDocLink | SyntaxKind::JSDocLinkCode | SyntaxKind::JSDocLinkPlain
    )
}

/// Whether a node is a non-whitespace token.
///
/// Mirrors `ast.IsNonWhitespaceToken` in Go.
pub fn is_non_whitespace_token(node: &Node) -> bool {
    is_token_kind(node.kind) && !is_whitespace_only_jsx_text(node)
}

/// Whether a node is whitespace-only JSX text.
///
/// Mirrors `ast.IsWhitespaceOnlyJsxText` in Go.
pub fn is_whitespace_only_jsx_text(node: &Node) -> bool {
    if let NodeData::JsxText(d) = &node.data {
        return d.contains_only_trivia_white_spaces;
    }
    false
}

// ────────────────────────────────────────────────────────────────────────────
// Declaration checks
// ────────────────────────────────────────────────────────────────────────────

/// Whether a node is a declaration.
///
/// Mirrors `ast.IsDeclaration` in Go.
pub fn is_declaration(node: &Node) -> bool {
    if node.kind == SyntaxKind::TypeParameter {
        return node.parent.is_some();
    }
    is_declaration_node(node)
}

/// Whether a node is a declaration node (by kind).
///
/// Mirrors `ast.IsDeclarationNode` in Go.
pub fn is_declaration_node(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::VariableDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::BindingElement
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::SpreadAssignment
            | SyntaxKind::EnumMember
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::IndexSignature
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::ImportClause
            | SyntaxKind::NamespaceImport
            | SyntaxKind::NamespaceExport
            | SyntaxKind::NamespaceExportDeclaration
            | SyntaxKind::ExportAssignment
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::MissingDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::JSImportDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::JsxAttribute
            | SyntaxKind::JsxSpreadAttribute
            | SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::TypeParameter
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::NamedTupleMember
    )
}

/// Whether a node can have a symbol.
///
/// Mirrors `ast.CanHaveSymbol` in Go.
pub fn can_have_symbol(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ArrowFunction
            | SyntaxKind::BinaryExpression
            | SyntaxKind::BindingElement
            | SyntaxKind::CallExpression
            | SyntaxKind::CallSignature
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::ConstructorType
            | SyntaxKind::ConstructSignature
            | SyntaxKind::ElementAccessExpression
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::EnumMember
            | SyntaxKind::ExportAssignment
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::FunctionType
            | SyntaxKind::GetAccessor
            | SyntaxKind::ImportClause
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::IndexSignature
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::JsxAttribute
            | SyntaxKind::JsxAttributes
            | SyntaxKind::JsxSpreadAttribute
            | SyntaxKind::MappedType
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::NamedTupleMember
            | SyntaxKind::NamespaceExport
            | SyntaxKind::NamespaceExportDeclaration
            | SyntaxKind::NamespaceImport
            | SyntaxKind::NewExpression
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::Parameter
            | SyntaxKind::PropertyAccessExpression
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::SetAccessor
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::SourceFile
            | SyntaxKind::SpreadAssignment
            | SyntaxKind::StringLiteral
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::TypeLiteral
            | SyntaxKind::TypeParameter
            | SyntaxKind::VariableDeclaration
    )
}

/// Whether a node can have modifiers.
///
/// Mirrors `ast.CanHaveModifiers` in Go.
pub fn can_have_modifiers(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::TypeParameter
            | SyntaxKind::Parameter
            | SyntaxKind::PropertySignature
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::IndexSignature
            | SyntaxKind::ConstructorType
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::ClassExpression
            | SyntaxKind::VariableStatement
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::JSImportDeclaration
            | SyntaxKind::ExportAssignment
            | SyntaxKind::ExportDeclaration
    )
}

/// Whether a node can have decorators.
///
/// Mirrors `ast.CanHaveDecorators` in Go.
pub fn can_have_decorators(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::Parameter
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::ClassExpression
            | SyntaxKind::ClassDeclaration
    )
}

// ────────────────────────────────────────────────────────────────────────────
// Modifier helpers
// ────────────────────────────────────────────────────────────────────────────

/// Whether a node has a specific syntactic modifier flag.
///
/// Mirrors `ast.HasSyntacticModifier` in Go.
pub fn has_syntactic_modifier(node: &Node, flags: ModifierFlags) -> bool {
    node.syntactic_modifier_flags().intersects(flags)
}

/// Whether a node has the `accessor` modifier.
///
/// Mirrors `ast.HasAccessorModifier` in Go.
pub fn has_accessor_modifier(node: &Node) -> bool {
    has_syntactic_modifier(node, ModifierFlags::Accessor)
}

/// Whether a node has the `static` modifier.
///
/// Mirrors `ast.HasStaticModifier` in Go.
pub fn has_static_modifier(node: &Node) -> bool {
    has_syntactic_modifier(node, ModifierFlags::Static)
}

/// Whether a node is static (has the modifier or is a static block).
///
/// Mirrors `ast.IsStatic` in Go.
pub fn is_static(node: &Node) -> bool {
    (is_class_element(node) && has_static_modifier(node)) || is_class_static_block_declaration(node)
}

/// Maps a modifier token kind to its `ModifierFlags`.
///
/// Mirrors `ast.ModifierToFlag` in Go.
pub fn modifier_to_flag(token: SyntaxKind) -> ModifierFlags {
    match token {
        SyntaxKind::StaticKeyword => ModifierFlags::Static,
        SyntaxKind::PublicKeyword => ModifierFlags::Public,
        SyntaxKind::ProtectedKeyword => ModifierFlags::Protected,
        SyntaxKind::PrivateKeyword => ModifierFlags::Private,
        SyntaxKind::AbstractKeyword => ModifierFlags::Abstract,
        SyntaxKind::AccessorKeyword => ModifierFlags::Accessor,
        SyntaxKind::ExportKeyword => ModifierFlags::Export,
        SyntaxKind::DeclareKeyword => ModifierFlags::Ambient,
        SyntaxKind::ConstKeyword => ModifierFlags::Const,
        SyntaxKind::DefaultKeyword => ModifierFlags::Default,
        SyntaxKind::AsyncKeyword => ModifierFlags::Async,
        SyntaxKind::ReadonlyKeyword => ModifierFlags::Readonly,
        SyntaxKind::OverrideKeyword => ModifierFlags::Override,
        SyntaxKind::InKeyword => ModifierFlags::In,
        SyntaxKind::OutKeyword => ModifierFlags::Out,
        SyntaxKind::Decorator => ModifierFlags::Decorator,
        _ => ModifierFlags::empty(),
    }
}

/// Computes combined modifier flags for a list of modifier nodes.
///
/// Mirrors `ast.ModifiersToFlags` in Go.
pub fn modifiers_to_flags(modifiers: &[Arc<Node>]) -> ModifierFlags {
    let mut flags = ModifierFlags::empty();
    for modifier in modifiers {
        flags |= modifier_to_flag(modifier.kind);
    }
    flags
}

// ────────────────────────────────────────────────────────────────────────────
// Tree walking
// ────────────────────────────────────────────────────────────────────────────

/// Walks up the parents of a node to find an ancestor matching the callback.
///
/// Mirrors `ast.FindAncestor` in Go.
pub fn find_ancestor<F>(node: &Arc<Node>, callback: F) -> Option<Arc<Node>>
where
    F: Fn(&Node) -> bool,
{
    let mut current: Option<&Arc<Node>> = Some(node);
    while let Some(n) = current {
        if callback(n) {
            return Some(Arc::clone(n));
        }
        current = n.parent.as_ref();
    }
    None
}

/// Walks up the parents of a node to find an ancestor of a specific kind.
///
/// Mirrors `ast.FindAncestorKind` in Go.
pub fn find_ancestor_kind(node: &Arc<Node>, kind: SyntaxKind) -> Option<Arc<Node>> {
    find_ancestor(node, |n| n.kind == kind)
}

/// Walks up the parents of a node to find the containing SourceFile.
///
/// Mirrors `ast.GetSourceFileOfNode` in Go.
pub fn get_source_file_of_node(node: &Arc<Node>) -> Option<Arc<Node>> {
    find_ancestor_kind(node, SyntaxKind::SourceFile)
}

/// Whether a node is a descendant of another node.
///
/// Mirrors `ast.IsNodeDescendantOf` in Go.
pub fn is_node_descendant_of(node: &Arc<Node>, ancestor: &Arc<Node>) -> bool {
    let mut current: Option<&Arc<Node>> = Some(node);
    while let Some(n) = current {
        if Arc::ptr_eq(n, ancestor) {
            return true;
        }
        current = n.parent.as_ref();
    }
    false
}

// ────────────────────────────────────────────────────────────────────────────
// Node accessors
// ────────────────────────────────────────────────────────────────────────────

/// Gets the root declaration by walking up binding elements.
///
/// Mirrors `ast.GetRootDeclaration` in Go.
pub fn get_root_declaration(node: &Arc<Node>) -> Arc<Node> {
    let mut current = Arc::clone(node);
    while current.kind == SyntaxKind::BindingElement {
        match &current.parent {
            Some(parent) => match &parent.parent {
                Some(grandparent) => {
                    current = Arc::clone(grandparent);
                }
                None => break,
            },
            None => break,
        }
    }
    current
}

/// Gets combined modifier flags by walking up variable declaration chains.
///
/// Mirrors `ast.GetCombinedModifierFlags` in Go.
pub fn get_combined_modifier_flags(node: &Arc<Node>) -> ModifierFlags {
    get_combined_flags(node, |n| n.syntactic_modifier_flags())
}

/// Gets combined node flags by walking up variable declaration chains.
///
/// Mirrors `ast.GetCombinedNodeFlags` in Go.
pub fn get_combined_node_flags(node: &Arc<Node>) -> NodeFlags {
    get_combined_flags(node, |n| n.flags)
}

fn get_combined_flags<F, T: std::ops::BitOr<Output = T>>(node: &Arc<Node>, get_flags: F) -> T
where
    F: Fn(&Node) -> T,
{
    let root = get_root_declaration(node);
    let mut flags = get_flags(&root);
    let mut current = if root.kind == SyntaxKind::VariableDeclaration {
        root.parent.clone()
    } else {
        None
    };
    if let Some(parent) = &current {
        if parent.kind == SyntaxKind::VariableDeclarationList {
            flags = flags | get_flags(parent);
            current = parent.parent.clone();
        }
    }
    if let Some(parent) = &current {
        if parent.kind == SyntaxKind::VariableStatement {
            flags = flags | get_flags(parent);
        }
    }
    flags
}

/// Gets the name of a declaration.
///
/// Mirrors `ast.GetNameOfDeclaration` in Go.
pub fn get_name_of_declaration(declaration: &Arc<Node>) -> Option<Arc<Node>> {
    let non_assigned = get_non_assigned_name_of_declaration(declaration);
    if non_assigned.is_some() {
        return non_assigned;
    }
    if is_function_expression(declaration)
        || is_arrow_function(declaration)
        || is_class_expression(declaration)
    {
        return get_assigned_name(declaration);
    }
    None
}

fn get_non_assigned_name_of_declaration(declaration: &Arc<Node>) -> Option<Arc<Node>> {
    match declaration.kind {
        SyntaxKind::ExportAssignment => {
            if let Some(expr) = declaration.expression() {
                if is_identifier(expr) {
                    return Some(Arc::clone(expr));
                }
            }
            None
        }
        _ => declaration.name().map(Arc::clone),
    }
}

fn get_assigned_name(node: &Arc<Node>) -> Option<Arc<Node>> {
    let parent = node.parent.as_ref()?;
    match parent.kind {
        SyntaxKind::PropertyAssignment => parent.name().map(Arc::clone),
        SyntaxKind::BindingElement => parent.name().map(Arc::clone),
        SyntaxKind::VariableDeclaration => {
            if let NodeData::VariableDeclaration(d) = &parent.data {
                if is_identifier(&d.name) {
                    return Some(Arc::clone(&d.name));
                }
            }
            None
        }
        _ => None,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// JS / file-context helpers
// ────────────────────────────────────────────────────────────────────────────

/// Whether a node was parsed in a JavaScript file.
///
/// Mirrors `ast.IsInJSFile` in Go.
pub fn is_in_js_file(node: &Node) -> bool {
    node.flags.contains(NodeFlags::JavaScriptFile)
}

/// Whether a node was parsed in a JSON file.
///
/// Mirrors `ast.IsInJsonFile` in Go.
pub fn is_in_json_file(node: &Node) -> bool {
    node.flags.contains(NodeFlags::JsonFile)
}

/// Whether a source file is JavaScript.
///
/// Mirrors `ast.IsSourceFileJS` in Go.
pub fn is_source_file_js(file: &SourceFile) -> bool {
    file.script_kind == super::node::ScriptKind::Js
        || file.script_kind == super::node::ScriptKind::Jsx
}

/// Whether a source file is JSON.
///
/// Mirrors `ast.IsJsonSourceFile` in Go.
pub fn is_json_source_file(file: &SourceFile) -> bool {
    file.script_kind == super::node::ScriptKind::Json
}

/// Whether a source file is an external module.
///
/// Mirrors `ast.IsExternalModule` in Go.
pub fn is_external_module(file: &SourceFile) -> bool {
    file.external_module_indicator.is_some()
}

/// Whether a source file is an external or CommonJS module.
///
/// Mirrors `ast.IsExternalOrCommonJSModule` in Go.
pub fn is_external_or_common_js_module(file: &SourceFile) -> bool {
    file.external_module_indicator.is_some() || file.common_js_module_indicator.is_some()
}

// ────────────────────────────────────────────────────────────────────────────
// Heritage / class helpers
// ────────────────────────────────────────────────────────────────────────────

/// Gets the heritage clauses of a class or interface.
///
/// Mirrors `ast.getHeritageClauses` in Go.
pub fn get_heritage_clauses(node: &Arc<Node>) -> Option<&Arc<NodeList>> {
    match &node.data {
        NodeData::ClassDeclaration(d) => d.heritage_clauses.as_ref(),
        NodeData::ClassExpression(d) => d.heritage_clauses.as_ref(),
        NodeData::InterfaceDeclaration(d) => d.heritage_clauses.as_ref(),
        _ => None,
    }
}

/// Gets a specific heritage clause (extends or implements).
///
/// Mirrors `ast.GetHeritageClause` in Go.
pub fn get_heritage_clause(node: &Arc<Node>, kind: SyntaxKind) -> Option<Arc<Node>> {
    if let Some(clauses) = get_heritage_clauses(node) {
        for clause in &clauses.nodes {
            if let NodeData::HeritageClause(d) = &clause.data {
                if d.token == kind {
                    return Some(Arc::clone(clause));
                }
            }
        }
    }
    None
}

/// Gets the extends heritage clause elements.
///
/// Mirrors `ast.GetExtendsHeritageClauseElements` in Go.
pub fn get_extends_heritage_clause_elements(node: &Arc<Node>) -> Vec<Arc<Node>> {
    get_heritage_elements(node, SyntaxKind::ExtendsKeyword)
}

/// Gets the implements heritage clause elements.
///
/// Mirrors `ast.GetImplementsHeritageClauseElements` in Go.
pub fn get_implements_heritage_clause_elements(node: &Arc<Node>) -> Vec<Arc<Node>> {
    get_heritage_elements(node, SyntaxKind::ImplementsKeyword)
}

fn get_heritage_elements(node: &Arc<Node>, kind: SyntaxKind) -> Vec<Arc<Node>> {
    match get_heritage_clause(node, kind) {
        Some(clause) => {
            if let NodeData::HeritageClause(d) = &clause.data {
                return d.types.nodes.clone();
            }
            Vec::new()
        }
        None => Vec::new(),
    }
}

/// Gets the first extends heritage clause element.
///
/// Mirrors `ast.GetExtendsHeritageClauseElement` in Go.
pub fn get_extends_heritage_clause_element(node: &Arc<Node>) -> Option<Arc<Node>> {
    get_extends_heritage_clause_elements(node)
        .into_iter()
        .next()
}

/// Gets the containing class declaration by walking up parents.
///
/// Mirrors `ast.GetContainingClass` in Go.
pub fn get_containing_class(node: &Arc<Node>) -> Option<Arc<Node>> {
    let parent = node.parent.as_ref()?;
    find_ancestor(parent, is_class_like)
}

// ────────────────────────────────────────────────────────────────────────────
// Module helpers
// ────────────────────────────────────────────────────────────────────────────

/// Whether a module declaration has a string-literal name.
///
/// Mirrors `ast.IsModuleWithStringLiteralName` in Go.
pub fn is_module_with_string_literal_name(node: &Node) -> bool {
    is_module_declaration(node)
        && node
            .name()
            .map(|n| n.kind == SyntaxKind::StringLiteral)
            .unwrap_or(false)
}

/// Whether a module declaration is an ambient module.
///
/// Mirrors `ast.IsAmbientModule` in Go.
pub fn is_ambient_module(node: &Node) -> bool {
    if !is_module_declaration(node) {
        return false;
    }
    match &node.data {
        NodeData::ModuleDeclaration(d) => {
            d.name.kind == SyntaxKind::StringLiteral || is_global_scope_augmentation(node)
        }
        _ => false,
    }
}

/// Whether a module declaration is a global scope augmentation.
///
/// Mirrors `ast.IsGlobalScopeAugmentation` in Go.
pub fn is_global_scope_augmentation(node: &Node) -> bool {
    if !is_module_declaration(node) {
        return false;
    }
    if let NodeData::ModuleDeclaration(d) = &node.data {
        return d.keyword == SyntaxKind::GlobalKeyword;
    }
    false
}

// ────────────────────────────────────────────────────────────────────────────
// Misc utilities
// ────────────────────────────────────────────────────────────────────────────

/// Whether an ambient module symbol name is quoted.
///
/// Mirrors `ast.IsAmbientModuleSymbolName` in Go.
pub fn is_ambient_module_symbol_name(s: &str) -> bool {
    s.starts_with('"') && s.ends_with('"')
}

/// Whether a node is `void 0`.
///
/// Mirrors `ast.IsVoidZero` in Go.
pub fn is_void_zero(node: &Node) -> bool {
    if !is_void_expression(node) {
        return false;
    }
    match node.expression() {
        Some(expr) => is_numeric_literal(expr) && expr.text() == "0",
        None => false,
    }
}

/// Whether a node is the identifier `exports`.
///
/// Mirrors `ast.IsExportsIdentifier` in Go.
pub fn is_exports_identifier(node: &Node) -> bool {
    is_identifier(node) && node.text() == "exports"
}

/// Whether a node is the identifier `module`.
///
/// Mirrors `ast.IsModuleIdentifier` in Go.
pub fn is_module_identifier(node: &Node) -> bool {
    is_identifier(node) && node.text() == "module"
}

/// Whether a node is the identifier `this`.
///
/// Mirrors `ast.IsThisIdentifier` in Go.
pub fn is_this_identifier(node: Option<&Node>) -> bool {
    match node {
        Some(n) => is_identifier(n) && n.text() == "this",
        None => false,
    }
}

/// Whether a node is a `super()` call.
///
/// Mirrors `ast.IsSuperCall` in Go.
pub fn is_super_call(node: &Node) -> bool {
    if !is_call_expression(node) {
        return false;
    }
    match node.expression() {
        Some(expr) => expr.kind == SyntaxKind::SuperKeyword,
        None => false,
    }
}

/// Whether a node is an `import()` call.
///
/// Mirrors `ast.IsImportCall` in Go.
pub fn is_import_call(node: &Node) -> bool {
    if !is_call_expression(node) {
        return false;
    }
    match node.expression() {
        Some(expr) => expr.kind == SyntaxKind::ImportKeyword,
        None => false,
    }
}

/// Whether a node is an `instanceof` expression.
///
/// Mirrors `ast.IsInstanceOfExpression` in Go.
pub fn is_instance_of_expression(node: &Node) -> bool {
    if let NodeData::BinaryExpression(d) = &node.data {
        return d.operator_token.kind == SyntaxKind::InstanceOfKeyword;
    }
    false
}

/// Whether a node is any import or re-export.
///
/// Mirrors `ast.IsAnyImportOrReExport` in Go.
pub fn is_any_import_or_re_export(node: &Node) -> bool {
    is_import_node(node) || is_export_declaration(node)
}

/// Whether a node is an import (declaration, equals, or JS re-parsed).
///
/// Mirrors `ast.IsImportNode` in Go.
pub fn is_import_node(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ImportDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::JSImportDeclaration
    )
}

/// Whether a node is genuine import syntax (excludes JS re-parsed).
///
/// Mirrors `ast.IsAnyImportSyntax` in Go.
pub fn is_any_import_syntax(node: &Node) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ImportDeclaration | SyntaxKind::ImportEqualsDeclaration
    )
}

/// Whether a node is a `?` token.
///
/// Mirrors `ast.IsQuestionToken` in Go.
pub fn is_question_token(node: Option<&Node>) -> bool {
    match node {
        Some(n) => n.kind == SyntaxKind::QuestionToken,
        None => false,
    }
}

/// Whether a node is a JSX tag name.
///
/// Mirrors `ast.IsJsxTagName` in Go. Checks whether `node` is the
/// `tagName` of an opening/closing/self-closing JSX element.
pub fn is_jsx_tag_name(node: &Arc<Node>) -> bool {
    let parent = match &node.parent {
        Some(p) => p,
        None => return false,
    };
    match parent.kind {
        SyntaxKind::JsxOpeningElement
        | SyntaxKind::JsxClosingElement
        | SyntaxKind::JsxSelfClosingElement => match &parent.data {
            NodeData::JsxOpeningElement(d) => Arc::ptr_eq(&d.tag_name, node),
            NodeData::JsxClosingElement(d) => Arc::ptr_eq(&d.tag_name, node),
            NodeData::JsxSelfClosingElement(d) => Arc::ptr_eq(&d.tag_name, node),
            _ => false,
        },
        _ => false,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Position helpers (text-related)
// ────────────────────────────────────────────────────────────────────────────

/// Gets the line and character (0-based, UTF-16) of a position in the
/// source file.
///
/// Mirrors `ast.GetLineAndCharacterOfPosition` in Go.
pub fn get_line_and_character_of_position(file: &SourceFile, position: usize) -> (usize, usize) {
    let line = file.line_map.line_at(position);
    let character = file.line_map.utf16_column_at(&file.text, position);
    (line, character)
}

/// Gets the position of a line and character (0-based, UTF-16) in the
/// source file.
///
/// Mirrors `ast.GetPositionOfLineAndCharacter` in Go.
pub fn get_position_of_line_and_character(
    file: &SourceFile,
    line: usize,
    character: usize,
) -> usize {
    if line >= file.line_map.line_starts.len() {
        return file.text.len();
    }
    let line_start = file.line_map.line_starts[line] as usize;
    // Walk UTF-16 code units from line start
    let mut col = 0usize;
    let bytes = file.text.as_bytes();
    let mut pos = line_start;
    while pos < file.text.len() && col < character {
        let b = bytes[pos];
        if b < 0x80 {
            pos += 1;
            col += 1;
        } else {
            let s = &file.text[pos..];
            match s.chars().next() {
                Some(ch) => {
                    pos += ch.len_utf8();
                    col += ch.len_utf16();
                }
                None => break,
            }
        }
    }
    pos
}

/// Gets the source file of a node, panicking if not found.
///
/// Convenience wrapper for `get_source_file_of_node`.
pub fn source_file_of_node_or_panic(node: &Arc<Node>) -> Arc<Node> {
    get_source_file_of_node(node)
        .unwrap_or_else(|| panic!("get_source_file_of_node: node is not contained in a SourceFile"))
}
