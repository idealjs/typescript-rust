//! Grammar checks: syntactic-level diagnostics emitted during semantic
//! checking.
//!
//! Ported from `internal/checker/grammarchecks.go` in the Go
//! implementation. These checks validate modifier ordering, parameter
//! lists, break/continue targets, variable declaration rules, etc.
//!
//! Unlike the Go version (2200+ lines), this module currently implements
//! the most commonly-encountered subset. Additional checks are added
//! incrementally.

use std::sync::Arc;

use crate::ast::{
    is_class_declaration, is_class_expression, is_jsx_namespaced_name, is_module_block,
    is_source_file, ModifierFlags, Node, NodeData, NodeFlags, SyntaxKind,
};
use crate::diagnostics::messages_generated::*;
use crate::diagnostics::Message;
use crate::scanner::token_to_string;

use super::checker::Checker;

// ────────────────────────────────────────────────────────────────────────────
// Grammar error helpers
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    /// Emit a grammar error on the given node.
    ///
    /// Mirrors Go's `grammarErrorOnNode`.
    pub(crate) fn grammar_error_on_node(&mut self, node: &Arc<Node>, message: &Message) -> bool {
        self.grammar_error_on_node_with_args(node, message, &[])
    }

    /// Emit a grammar error on the given node with formatted arguments.
    pub(crate) fn grammar_error_on_node_with_args(
        &mut self,
        node: &Arc<Node>,
        message: &Message,
        args: &[String],
    ) -> bool {
        let file = self.current_file.clone();
        let diagnostic = crate::ast::Diagnostic::new(file, node.loc, *message, args.to_vec());
        self.diagnostics.add(diagnostic);
        true
    }

    /// Emit a grammar error at a specific position range.
    ///
    /// Mirrors Go's `grammarErrorAtPos`.
    fn grammar_error_at_pos(
        &mut self,
        node_for_file: &Arc<Node>,
        start: usize,
        length: usize,
        message: &Message,
    ) -> bool {
        let file = self.current_file.clone();
        let loc = crate::core::text::TextRange::new(start, start + length);
        let diagnostic =
            crate::ast::Diagnostic::new(file, loc, *message, Vec::new());
        self.diagnostics.add(diagnostic);
        true
    }

    /// Emit a grammar error on the first token of a node.
    ///
    /// Mirrors Go's `grammarErrorOnFirstToken`. Since we don't have a
    /// `GetRangeOfTokenAtPosition` utility yet, this falls back to the
    /// node's own location.
    fn grammar_error_on_first_token(&mut self, node: &Arc<Node>, message: &Message) -> bool {
        self.grammar_error_on_node(node, message)
    }

    // ─────────────────────────────────────────────────────────────────────
    // checkGrammarModifiers
    // ─────────────────────────────────────────────────────────────────────

    /// Check modifier ordering and validity on a declaration.
    ///
    /// Mirrors Go's `checkGrammarModifiers` (grammarchecks.go ~L213).
    /// Validates that modifiers appear in the correct order and that
    /// combinations are legal. Key rules:
    ///
    /// - Accessibility (`public`/`protected`/`private`) appears at most once
    /// - `static` appears at most once
    /// - `override` must precede `readonly`, `accessor`, `async`
    /// - `accessor` only on property declarations
    /// - `readonly` only on properties/index signatures/parameters
    /// - `abstract` only on classes, methods, properties, accessors
    /// - `declare` (ambient) cannot combine with `async`/`override`
    /// - `export`/`default` ordering
    pub fn check_grammar_modifiers(&mut self, node: &Arc<Node>) -> bool {
        let modifiers = match node.modifiers() {
            Some(ml) => Arc::clone(ml),
            None => return false,
        };

        // `this` parameter: no decorators or modifiers.
        if is_this_parameter(node) {
            return self.grammar_error_on_first_token(
                node,
                &NEITHER_DECORATORS_NOR_MODIFIERS_MAY_BE_APPLIED_TO_THIS_PARAMETERS,
            );
        }

        let block_scope_kind = if is_variable_statement(node) {
            if let NodeData::VariableStatement(data) = &node.data {
                data.declaration_list.flags & NodeFlags::BlockScoped
            } else {
                NodeFlags::empty()
            }
        } else {
            NodeFlags::empty()
        };

        let mut flags = ModifierFlags::empty();
        let mut last_static: Option<Arc<Node>> = None;
        let mut last_override: Option<Arc<Node>> = None;
        let mut last_async: Option<Arc<Node>> = None;
        let mut last_declare: Option<Arc<Node>> = None;

        for modifier in &modifiers.list.nodes {
            if modifier.kind == SyntaxKind::Decorator {
                // Decorators: skip for now (decorator validation is complex).
                flags |= ModifierFlags::Decorator;
                continue;
            }

            // Check context restrictions for non-readonly modifiers.
            if modifier.kind != SyntaxKind::ReadonlyKeyword {
                if node.kind == SyntaxKind::PropertySignature
                    || node.kind == SyntaxKind::MethodSignature
                {
                    let text = token_to_string(modifier.kind);
                    return self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_A_TYPE_MEMBER,
                        &[text.to_string()],
                    );
                }
                if node.kind == SyntaxKind::IndexSignature {
                    let text = token_to_string(modifier.kind);
                    return self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_AN_INDEX_SIGNATURE,
                        &[text.to_string()],
                    );
                }
            }

            // Type parameter restrictions.
            if modifier.kind != SyntaxKind::InKeyword
                && modifier.kind != SyntaxKind::OutKeyword
                && modifier.kind != SyntaxKind::ConstKeyword
            {
                if node.kind == SyntaxKind::TypeParameter {
                    let text = token_to_string(modifier.kind);
                    return self.grammar_error_on_node_with_args(
                        modifier,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_A_TYPE_PARAMETER,
                        &[text.to_string()],
                    );
                }
            }

            match modifier.kind {
                SyntaxKind::ConstKeyword => {
                    if node.kind != SyntaxKind::EnumDeclaration
                        && node.kind != SyntaxKind::TypeParameter
                    {
                        return self.grammar_error_on_node_with_args(
                            node,
                            &A_CLASS_MEMBER_CANNOT_HAVE_THE_0_KEYWORD,
                            &["const".to_string()],
                        );
                    }
                }
                SyntaxKind::OverrideKeyword => {
                    if flags.contains(ModifierFlags::Override) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_ALREADY_SEEN,
                            &["override".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Ambient) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_BE_USED_WITH_1_MODIFIER,
                            &["override".to_string(), "declare".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Readonly)
                        && !modifier.flags.contains(NodeFlags::Reparsed)
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                            &["override".to_string(), "readonly".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Accessor)
                        && !modifier.flags.contains(NodeFlags::Reparsed)
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                            &["override".to_string(), "accessor".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Async)
                        && !modifier.flags.contains(NodeFlags::Reparsed)
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                            &["override".to_string(), "async".to_string()],
                        );
                    }
                    flags |= ModifierFlags::Override;
                    last_override = Some(Arc::clone(modifier));
                }
                SyntaxKind::PublicKeyword
                | SyntaxKind::ProtectedKeyword
                | SyntaxKind::PrivateKeyword => {
                    let text = visibility_to_string(modifier.kind);
                    if flags.contains(ModifierFlags::AccessibilityModifier) {
                        return self.grammar_error_on_node(
                            modifier,
                            &ACCESSIBILITY_MODIFIER_ALREADY_SEEN,
                        );
                    } else if flags.contains(ModifierFlags::Override)
                        && !modifier.flags.contains(NodeFlags::Reparsed)
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                            &[text.to_string(), "override".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Static)
                        && !modifier.flags.contains(NodeFlags::Reparsed)
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                            &[text.to_string(), "static".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Accessor)
                        && !modifier.flags.contains(NodeFlags::Reparsed)
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                            &[text.to_string(), "accessor".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Readonly)
                        && !modifier.flags.contains(NodeFlags::Reparsed)
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                            &[text.to_string(), "readonly".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Async)
                        && !modifier.flags.contains(NodeFlags::Reparsed)
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                            &[text.to_string(), "async".to_string()],
                        );
                    } else if is_parent_module_block_or_source_file(node) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_APPEAR_ON_A_MODULE_OR_NAMESPACE_ELEMENT,
                            &[text.to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Abstract) {
                        if modifier.kind == SyntaxKind::PrivateKeyword {
                            return self.grammar_error_on_node_with_args(
                                modifier,
                                &X_0_MODIFIER_CANNOT_BE_USED_WITH_1_MODIFIER,
                                &[text.to_string(), "abstract".to_string()],
                            );
                        } else if !modifier.flags.contains(NodeFlags::Reparsed) {
                            return self.grammar_error_on_node_with_args(
                                modifier,
                                &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                                &[text.to_string(), "abstract".to_string()],
                            );
                        }
                    }
                    flags |= modifier_to_flag(modifier.kind);
                }
                SyntaxKind::StaticKeyword => {
                    if flags.contains(ModifierFlags::Static) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_ALREADY_SEEN,
                            &["static".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Readonly)
                        && !modifier.flags.contains(NodeFlags::Reparsed)
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                            &["static".to_string(), "readonly".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Async)
                        && !modifier.flags.contains(NodeFlags::Reparsed)
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                            &["static".to_string(), "async".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Accessor)
                        && !modifier.flags.contains(NodeFlags::Reparsed)
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                            &["static".to_string(), "accessor".to_string()],
                        );
                    } else if is_parent_module_block_or_source_file(node) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_APPEAR_ON_A_MODULE_OR_NAMESPACE_ELEMENT,
                            &["static".to_string()],
                        );
                    } else if node.kind == SyntaxKind::Parameter {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_APPEAR_ON_A_PARAMETER,
                            &["static".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Abstract) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_BE_USED_WITH_1_MODIFIER,
                            &["static".to_string(), "abstract".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Override)
                        && !modifier.flags.contains(NodeFlags::Reparsed)
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                            &["static".to_string(), "override".to_string()],
                        );
                    }
                    flags |= ModifierFlags::Static;
                    last_static = Some(Arc::clone(modifier));
                }
                SyntaxKind::AccessorKeyword => {
                    if flags.contains(ModifierFlags::Accessor) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_ALREADY_SEEN,
                            &["accessor".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Readonly) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_BE_USED_WITH_1_MODIFIER,
                            &["accessor".to_string(), "readonly".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Ambient) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_BE_USED_WITH_1_MODIFIER,
                            &["accessor".to_string(), "declare".to_string()],
                        );
                    } else if node.kind != SyntaxKind::PropertyDeclaration {
                        return self.grammar_error_on_node(
                            modifier,
                            &X_ACCESSOR_MODIFIER_CAN_ONLY_APPEAR_ON_A_PROPERTY_DECLARATION,
                        );
                    }
                    flags |= ModifierFlags::Accessor;
                }
                SyntaxKind::ReadonlyKeyword => {
                    if flags.contains(ModifierFlags::Readonly) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_ALREADY_SEEN,
                            &["readonly".to_string()],
                        );
                    } else if node.kind != SyntaxKind::PropertyDeclaration
                        && node.kind != SyntaxKind::PropertySignature
                        && node.kind != SyntaxKind::IndexSignature
                        && node.kind != SyntaxKind::Parameter
                    {
                        return self.grammar_error_on_node(
                            modifier,
                            &X_READONLY_MODIFIER_CAN_ONLY_APPEAR_ON_A_PROPERTY_DECLARATION_OR_INDEX_SIGNATURE,
                        );
                    } else if flags.contains(ModifierFlags::Accessor) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_BE_USED_WITH_1_MODIFIER,
                            &["readonly".to_string(), "accessor".to_string()],
                        );
                    }
                    flags |= ModifierFlags::Readonly;
                }
                SyntaxKind::ExportKeyword => {
                    if flags.contains(ModifierFlags::Export) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_ALREADY_SEEN,
                            &["export".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Ambient)
                        && !modifier.flags.contains(NodeFlags::Reparsed)
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                            &["export".to_string(), "declare".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Abstract)
                        && !modifier.flags.contains(NodeFlags::Reparsed)
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                            &["export".to_string(), "abstract".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Async)
                        && !modifier.flags.contains(NodeFlags::Reparsed)
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                            &["export".to_string(), "async".to_string()],
                        );
                    } else if is_parent_class_like(node) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_APPEAR_ON_CLASS_ELEMENTS_OF_THIS_KIND,
                            &["export".to_string()],
                        );
                    } else if node.kind == SyntaxKind::Parameter {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_APPEAR_ON_A_PARAMETER,
                            &["export".to_string()],
                        );
                    }
                    flags |= ModifierFlags::Export;
                }
                SyntaxKind::DefaultKeyword => {
                    if block_scope_kind == NodeFlags::Using {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_APPEAR_ON_A_USING_DECLARATION,
                            &["default".to_string()],
                        );
                    } else if !flags.contains(ModifierFlags::Export)
                        && !modifier.flags.contains(NodeFlags::Reparsed)
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                            &["export".to_string(), "default".to_string()],
                        );
                    }
                    flags |= ModifierFlags::Default;
                }
                SyntaxKind::DeclareKeyword => {
                    if flags.contains(ModifierFlags::Ambient) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_ALREADY_SEEN,
                            &["declare".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Async) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_BE_USED_IN_AN_AMBIENT_CONTEXT,
                            &["async".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Override) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_BE_USED_IN_AN_AMBIENT_CONTEXT,
                            &["override".to_string()],
                        );
                    } else if is_parent_class_like(node)
                        && node.kind != SyntaxKind::PropertyDeclaration
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_APPEAR_ON_CLASS_ELEMENTS_OF_THIS_KIND,
                            &["declare".to_string()],
                        );
                    } else if node.kind == SyntaxKind::Parameter {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_APPEAR_ON_A_PARAMETER,
                            &["declare".to_string()],
                        );
                    }
                    flags |= ModifierFlags::Ambient;
                    last_declare = Some(Arc::clone(modifier));
                }
                SyntaxKind::AbstractKeyword => {
                    if flags.contains(ModifierFlags::Abstract) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_ALREADY_SEEN,
                            &["abstract".to_string()],
                        );
                    }
                    if node.kind != SyntaxKind::ClassDeclaration
                        && node.kind != SyntaxKind::ConstructorType
                    {
                        if node.kind != SyntaxKind::MethodDeclaration
                            && node.kind != SyntaxKind::PropertyDeclaration
                            && node.kind != SyntaxKind::GetAccessor
                            && node.kind != SyntaxKind::SetAccessor
                        {
                            return self.grammar_error_on_node(
                                modifier,
                                &X_ABSTRACT_MODIFIER_CAN_ONLY_APPEAR_ON_A_CLASS_METHOD_OR_PROPERTY_DECLARATION,
                            );
                        }
                        // Must be within an abstract class.
                        let parent_is_abstract_class = node
                            .parent
                            .as_ref()
                            .map(|p| {
                                p.kind == SyntaxKind::ClassDeclaration
                                    && p.has_syntactic_modifier(ModifierFlags::Abstract)
                            })
                            .unwrap_or(false);
                        if !parent_is_abstract_class {
                            let message = if node.kind == SyntaxKind::PropertyDeclaration {
                                &ABSTRACT_PROPERTIES_CAN_ONLY_APPEAR_WITHIN_AN_ABSTRACT_CLASS
                            } else {
                                &ABSTRACT_METHODS_CAN_ONLY_APPEAR_WITHIN_AN_ABSTRACT_CLASS
                            };
                            return self.grammar_error_on_node(modifier, message);
                        }
                        if flags.contains(ModifierFlags::Static) {
                            return self.grammar_error_on_node_with_args(
                                modifier,
                                &X_0_MODIFIER_CANNOT_BE_USED_WITH_1_MODIFIER,
                                &["static".to_string(), "abstract".to_string()],
                            );
                        }
                        if flags.contains(ModifierFlags::Private) {
                            return self.grammar_error_on_node_with_args(
                                modifier,
                                &X_0_MODIFIER_CANNOT_BE_USED_WITH_1_MODIFIER,
                                &["private".to_string(), "abstract".to_string()],
                            );
                        }
                    }
                    flags |= ModifierFlags::Abstract;
                }
                SyntaxKind::AsyncKeyword => {
                    if flags.contains(ModifierFlags::Async) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_ALREADY_SEEN,
                            &["async".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Ambient)
                        || node
                            .parent
                            .as_ref()
                            .map(|p| p.flags.contains(NodeFlags::Ambient))
                            .unwrap_or(false)
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_BE_USED_IN_AN_AMBIENT_CONTEXT,
                            &["async".to_string()],
                        );
                    } else if node.kind == SyntaxKind::Parameter {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_APPEAR_ON_A_PARAMETER,
                            &["async".to_string()],
                        );
                    } else if flags.contains(ModifierFlags::Abstract) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CANNOT_BE_USED_WITH_1_MODIFIER,
                            &["async".to_string(), "abstract".to_string()],
                        );
                    }
                    flags |= ModifierFlags::Async;
                    last_async = Some(Arc::clone(modifier));
                }
                SyntaxKind::InKeyword | SyntaxKind::OutKeyword => {
                    let in_out_flag = if modifier.kind == SyntaxKind::InKeyword {
                        ModifierFlags::In
                    } else {
                        ModifierFlags::Out
                    };
                    let in_out_text = if modifier.kind == SyntaxKind::InKeyword {
                        "in"
                    } else {
                        "out"
                    };
                    if node.kind != SyntaxKind::TypeParameter {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_CAN_ONLY_APPEAR_ON_A_TYPE_PARAMETER_OF_A_CLASS_INTERFACE_OR_TYPE_ALIAS,
                            &[in_out_text.to_string()],
                        );
                    }
                    if flags.contains(in_out_flag) {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_ALREADY_SEEN,
                            &[in_out_text.to_string()],
                        );
                    }
                    if in_out_flag.contains(ModifierFlags::In)
                        && flags.contains(ModifierFlags::Out)
                    {
                        return self.grammar_error_on_node_with_args(
                            modifier,
                            &X_0_MODIFIER_MUST_PRECEDE_1_MODIFIER,
                            &["in".to_string(), "out".to_string()],
                        );
                    }
                    flags |= in_out_flag;
                }
                _ => {}
            }
        }

        // Post-loop checks for constructors.
        if node.kind == SyntaxKind::Constructor {
            if flags.contains(ModifierFlags::Static) {
                if let Some(last_static) = &last_static {
                    return self.grammar_error_on_node_with_args(
                        last_static,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_A_CONSTRUCTOR_DECLARATION,
                        &["static".to_string()],
                    );
                }
            }
            if flags.contains(ModifierFlags::Override) {
                if let Some(last_override) = &last_override {
                    return self.grammar_error_on_node_with_args(
                        last_override,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_A_CONSTRUCTOR_DECLARATION,
                        &["override".to_string()],
                    );
                }
            }
            if flags.contains(ModifierFlags::Async) {
                if let Some(last_async) = &last_async {
                    return self.grammar_error_on_node_with_args(
                        last_async,
                        &X_0_MODIFIER_CANNOT_APPEAR_ON_A_CONSTRUCTOR_DECLARATION,
                        &["async".to_string()],
                    );
                }
            }
            return false;
        }

        // `declare` on import declarations.
        if (node.kind == SyntaxKind::ImportDeclaration
            || node.kind == SyntaxKind::ImportEqualsDeclaration)
            && flags.contains(ModifierFlags::Ambient)
        {
            if let Some(last_declare) = &last_declare {
                return self.grammar_error_on_node_with_args(
                    last_declare,
                    &A_0_MODIFIER_CANNOT_BE_USED_WITH_AN_IMPORT_DECLARATION,
                    &["declare".to_string()],
                );
            }
        }

        // Async modifier on non-function-like nodes.
        if flags.contains(ModifierFlags::Async) {
            if let Some(last_async_node) = &last_async {
                match node.kind {
                    SyntaxKind::MethodDeclaration
                    | SyntaxKind::FunctionDeclaration
                    | SyntaxKind::FunctionExpression
                    | SyntaxKind::ArrowFunction => {}
                    _ => {
                        return self.grammar_error_on_node_with_args(
                            last_async_node,
                            &X_0_MODIFIER_CANNOT_BE_USED_HERE,
                            &["async".to_string()],
                        );
                    }
                }
            }
        }

        false
    }

    // ─────────────────────────────────────────────────────────────────────
    // checkGrammarBreakOrContinueStatement
    // ─────────────────────────────────────────────────────────────────────

    /// Validate that `break`/`continue` targets are valid.
    ///
    /// Mirrors Go's `checkGrammarBreakOrContinueStatement`.
    /// - `break` must be within an enclosing iteration or switch statement
    ///   (or target a label).
    /// - `continue` must be within an enclosing iteration statement
    ///   (or target a label on an iteration statement).
    ///
    /// Since parent pointers are not set on nodes in the Rust port, this
    /// uses the checker's `break_continue_context_stack` which is pushed/
    /// popped as the checker enters loops, switches, functions, and labeled
    /// statements.
    pub fn check_grammar_break_or_continue_statement(&mut self, node: &Arc<Node>) -> bool {
        let target_label = match &node.data {
            NodeData::BreakStatement(data) => data.label.as_ref(),
            NodeData::ContinueStatement(data) => data.label.as_ref(),
            _ => None,
        };
        let target_label_text = target_label.map(|l| l.text().to_string());
        let is_break = node.kind == SyntaxKind::BreakStatement;

        // Walk the context stack from innermost to outermost.
        for ctx in self.break_continue_context_stack.iter().rev() {
            match ctx.kind {
                super::checker::BreakContinueContextKind::Function => {
                    // Cannot cross function boundary.
                    return self.grammar_error_on_node(
                        node,
                        &JUMP_TARGET_CANNOT_CROSS_FUNCTION_BOUNDARY,
                    );
                }
                super::checker::BreakContinueContextKind::Labeled => {
                    if let Some(label_text) = &target_label_text {
                        if ctx.label.as_deref() == Some(label_text.as_str()) {
                            // Found matching label.
                            if !is_break && !ctx.is_iteration {
                                // continue can only target iteration statements.
                                return self.grammar_error_on_node(
                                    node,
                                    &A_CONTINUE_STATEMENT_CAN_ONLY_JUMP_TO_A_LABEL_OF_AN_ENCLOSING_ITERATION_STATEMENT,
                                );
                            }
                            return false;
                        }
                    }
                }
                super::checker::BreakContinueContextKind::Loop => {
                    if target_label.is_none() {
                        // Unlabeled break or continue within iteration — OK.
                        return false;
                    }
                }
                super::checker::BreakContinueContextKind::Switch => {
                    if is_break && target_label.is_none() {
                        // Unlabeled break within switch — OK.
                        return false;
                    }
                }
            }
        }

        // No valid target found.
        let message = if target_label.is_some() {
            if is_break {
                &A_BREAK_STATEMENT_CAN_ONLY_JUMP_TO_A_LABEL_OF_AN_ENCLOSING_STATEMENT
            } else {
                &A_CONTINUE_STATEMENT_CAN_ONLY_JUMP_TO_A_LABEL_OF_AN_ENCLOSING_ITERATION_STATEMENT
            }
        } else if is_break {
            &A_BREAK_STATEMENT_CAN_ONLY_BE_USED_WITHIN_AN_ENCLOSING_ITERATION_OR_SWITCH_STATEMENT
        } else {
            &A_CONTINUE_STATEMENT_CAN_ONLY_BE_USED_WITHIN_AN_ENCLOSING_ITERATION_STATEMENT
        };
        self.grammar_error_on_node(node, message)
    }

    // ─────────────────────────────────────────────────────────────────────
    // checkGrammarVariableDeclarationList
    // ─────────────────────────────────────────────────────────────────────

    /// Validate a variable declaration list.
    ///
    /// Mirrors Go's `checkGrammarVariableDeclarationList`. Checks:
    /// - Non-empty declaration list.
    /// - `using`/`await using` not in `for-in`.
    /// - `using`/`await using` not in ambient context.
    pub fn check_grammar_variable_declaration_list(&mut self, node: &Arc<Node>) -> bool {
        let data = match &node.data {
            NodeData::VariableDeclarationList(data) => data,
            _ => return false,
        };

        let declarations = &data.declarations;
        if declarations.is_empty() {
            return self.grammar_error_at_pos(
                node,
                declarations.pos(),
                declarations.end() - declarations.pos(),
                &VARIABLE_DECLARATION_LIST_CANNOT_BE_EMPTY,
            );
        }

        let block_scope_flags = node.flags & NodeFlags::BlockScoped;
        if block_scope_flags == NodeFlags::Using || block_scope_flags == NodeFlags::AwaitUsing {
            // `using` in `for-in` is not allowed.
            if let Some(parent) = &node.parent {
                if parent.kind == SyntaxKind::ForInStatement {
                    let message = if block_scope_flags == NodeFlags::Using {
                        &THE_LEFT_HAND_SIDE_OF_A_FOR_IN_STATEMENT_CANNOT_BE_A_USING_DECLARATION
                    } else {
                        &THE_LEFT_HAND_SIDE_OF_A_FOR_IN_STATEMENT_CANNOT_BE_AN_AWAIT_USING_DECLARATION
                    };
                    return self.grammar_error_on_node(node, message);
                }
            }
            // `using` in ambient context.
            if node.flags.contains(NodeFlags::Ambient) {
                let message = if block_scope_flags == NodeFlags::Using {
                    &X_USING_DECLARATIONS_ARE_NOT_ALLOWED_IN_AMBIENT_CONTEXTS
                } else {
                    &X_AWAIT_USING_DECLARATIONS_ARE_NOT_ALLOWED_IN_AMBIENT_CONTEXTS
                };
                return self.grammar_error_on_node(node, message);
            }
        }

        // Check individual declarations.
        for decl in declarations.iter() {
            if self.check_grammar_variable_declaration(decl) {
                return true;
            }
        }

        false
    }

    /// Validate a single variable declaration.
    ///
    /// Mirrors Go's `checkGrammarVariableDeclaration`.
    pub fn check_grammar_variable_declaration(&mut self, node: &Arc<Node>) -> bool {
        let data = match &node.data {
            NodeData::VariableDeclaration(data) => data,
            _ => return false,
        };

        let node_flags = node.flags;
        let block_scope_kind = node_flags & NodeFlags::BlockScoped;

        // Destructuring with using/await using.
        if is_binding_pattern(&data.name) {
            match block_scope_kind {
                NodeFlags::AwaitUsing => {
                    return self.grammar_error_on_node_with_args(
                        node,
                        &X_0_DECLARATIONS_MAY_NOT_HAVE_BINDING_PATTERNS,
                        &["await using".to_string()],
                    );
                }
                NodeFlags::Using => {
                    return self.grammar_error_on_node_with_args(
                        node,
                        &X_0_DECLARATIONS_MAY_NOT_HAVE_BINDING_PATTERNS,
                        &["using".to_string()],
                    );
                }
                _ => {}
            }
        }

        // Check if we're in a for-in/for-of (skip initializer checks).
        let in_for_in_or_of = node
            .parent
            .as_ref()
            .and_then(|p| p.parent.clone())
            .map(|grandparent| {
                grandparent.kind == SyntaxKind::ForInStatement
                    || grandparent.kind == SyntaxKind::ForOfStatement
            })
            .unwrap_or(false);

        if !in_for_in_or_of {
            if data.initializer.is_none() {
                // Destructuring must have initializer (unless nested).
                if is_binding_pattern(&data.name) {
                    let parent_is_binding_pattern = node
                        .parent
                        .as_ref()
                        .map(|p| is_binding_pattern(p))
                        .unwrap_or(false);
                    if !parent_is_binding_pattern {
                        return self.grammar_error_on_node(
                            node,
                            &A_DESTRUCTURING_DECLARATION_MUST_HAVE_AN_INITIALIZER,
                        );
                    }
                }
                // const/using must be initialized.
                match block_scope_kind {
                    NodeFlags::AwaitUsing => {
                        return self.grammar_error_on_node_with_args(
                            node,
                            &X_0_DECLARATIONS_MUST_BE_INITIALIZED,
                            &["await using".to_string()],
                        );
                    }
                    NodeFlags::Using => {
                        return self.grammar_error_on_node_with_args(
                            node,
                            &X_0_DECLARATIONS_MUST_BE_INITIALIZED,
                            &["using".to_string()],
                        );
                    }
                    NodeFlags::Const => {
                        return self.grammar_error_on_node_with_args(
                            node,
                            &X_0_DECLARATIONS_MUST_BE_INITIALIZED,
                            &["const".to_string()],
                        );
                    }
                    _ => {}
                }
            }
        }

        // Definite assignment assertion (`!`).
        if let Some(excl_token) = &data.exclamation_token {
            let parent_kind = node
                .parent
                .as_ref()
                .and_then(|p| p.parent.as_ref())
                .map(|gp| gp.kind);
            let in_variable_statement = parent_kind == Some(SyntaxKind::VariableStatement);
            let has_type = data.type_node.is_some();
            let has_initializer = data.initializer.is_some();
            let is_ambient = node_flags.contains(NodeFlags::Ambient);

            if !in_variable_statement || !has_type || has_initializer || is_ambient {
                let message = if has_initializer {
                    &DECLARATIONS_WITH_INITIALIZERS_CANNOT_ALSO_HAVE_DEFINITE_ASSIGNMENT_ASSERTIONS
                } else if !has_type {
                    &DECLARATIONS_WITH_DEFINITE_ASSIGNMENT_ASSERTIONS_MUST_ALSO_HAVE_TYPE_ANNOTATIONS
                } else {
                    &A_DEFINITE_ASSIGNMENT_ASSERTION_IS_NOT_PERMITTED_IN_THIS_CONTEXT
                };
                return self.grammar_error_on_node(excl_token, message);
            }
        }

        false
    }

    // ─────────────────────────────────────────────────────────────────────
    // checkGrammarParameterList
    // ─────────────────────────────────────────────────────────────────────

    /// Validate a parameter list.
    ///
    /// Mirrors Go's `checkGrammarParameterList`. Checks:
    /// - Rest parameter must be last.
    /// - Required parameter cannot follow optional.
    /// - Rest parameter cannot be optional or have an initializer.
    pub fn check_grammar_parameter_list(&mut self, parameters: &crate::ast::NodeList) -> bool {
        let mut seen_optional = false;
        let count = parameters.nodes.len();

        for (i, param_node) in parameters.nodes.iter().enumerate() {
            let param = match &param_node.data {
                NodeData::ParameterDeclaration(data) => data,
                _ => continue,
            };

            if param.dot_dot_dot_token.is_some() {
                // Rest parameter.
                if i != count - 1 {
                    if let Some(rest_token) = &param.dot_dot_dot_token {
                        let _ = self.grammar_error_on_node(
                            rest_token,
                            &A_REST_PARAMETER_MUST_BE_LAST_IN_A_PARAMETER_LIST,
                        );
                        return true;
                    }
                }
                if param.question_token.is_some() {
                    if let Some(q) = &param.question_token {
                        let _ = self.grammar_error_on_node(q, &A_REST_PARAMETER_CANNOT_BE_OPTIONAL);
                        return true;
                    }
                }
                if param.initializer.is_some() {
                    if let Some(name) = param_node.name() {
                        let _ = self.grammar_error_on_node(
                            name,
                            &A_REST_PARAMETER_CANNOT_HAVE_AN_INITIALIZER,
                        );
                        return true;
                    }
                }
            } else if is_optional_declaration(param_node) {
                seen_optional = true;
                // `?` + initializer is invalid.
                if param.question_token.is_some()
                    && !param
                        .question_token
                        .as_ref()
                        .map(|q| q.flags.contains(NodeFlags::Reparsed))
                        .unwrap_or(false)
                    && param.initializer.is_some()
                {
                    if let Some(name) = param_node.name() {
                        let _ = self.grammar_error_on_node(
                            name,
                            &PARAMETER_CANNOT_HAVE_QUESTION_MARK_AND_INITIALIZER,
                        );
                        return true;
                    }
                }
            } else if seen_optional && param.initializer.is_none() {
                if let Some(name) = param_node.name() {
                    let _ = self.grammar_error_on_node(
                        name,
                        &A_REQUIRED_PARAMETER_CANNOT_FOLLOW_AN_OPTIONAL_PARAMETER,
                    );
                    return true;
                }
            }
        }

        false
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Helper functions
// ────────────────────────────────────────────────────────────────────────────

/// Check if a node is a `this` parameter.
fn is_this_parameter(node: &Arc<Node>) -> bool {
    if node.kind != SyntaxKind::Parameter {
        return false;
    }
    match &node.data {
        NodeData::ParameterDeclaration(data) => {
            matches!(&data.name.data, NodeData::Identifier(id) if id.text == "this")
        }
        _ => false,
    }
}

/// Check if a node is a variable statement.
fn is_variable_statement(node: &Arc<Node>) -> bool {
    node.kind == SyntaxKind::VariableStatement
}

/// Check if a node's parent is a module block or source file.
fn is_parent_module_block_or_source_file(node: &Arc<Node>) -> bool {
    match &node.parent {
        Some(parent) => is_module_block(parent) || is_source_file(parent),
        None => false,
    }
}

/// Check if a node's parent is a class-like declaration.
fn is_parent_class_like(node: &Arc<Node>) -> bool {
    match &node.parent {
        Some(parent) => is_class_declaration(parent) || is_class_expression(parent),
        None => false,
    }
}

/// Check if a node is an iteration statement (for/while/do-while/for-in/for-of).
fn is_iteration_statement(node: &Arc<Node>, look_in_labeled: bool) -> bool {
    match node.kind {
        SyntaxKind::ForStatement
        | SyntaxKind::ForInStatement
        | SyntaxKind::ForOfStatement
        | SyntaxKind::WhileStatement
        | SyntaxKind::DoStatement => true,
        SyntaxKind::LabeledStatement if look_in_labeled => {
            if let NodeData::LabeledStatement(data) = &node.data {
                is_iteration_statement(&data.statement, false)
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Check if a node is a function-like declaration or class static block.
fn is_function_like_or_class_static_block(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::ClassStaticBlockDeclaration
    )
}

/// Check if a parameter declaration is optional (`?` token or initializer).
fn is_optional_declaration(node: &Arc<Node>) -> bool {
    if node.kind != SyntaxKind::Parameter {
        return false;
    }
    match &node.data {
        NodeData::ParameterDeclaration(data) => {
            data.question_token.is_some() || data.initializer.is_some()
        }
        _ => false,
    }
}

/// Check if a node is a binding pattern.
fn is_binding_pattern(node: &Arc<Node>) -> bool {
    matches!(
        node.kind,
        SyntaxKind::ObjectBindingPattern | SyntaxKind::ArrayBindingPattern
    )
}

/// Convert a modifier kind to its visibility string.
fn visibility_to_string(kind: SyntaxKind) -> &'static str {
    match kind {
        SyntaxKind::PublicKeyword => "public",
        SyntaxKind::ProtectedKeyword => "protected",
        SyntaxKind::PrivateKeyword => "private",
        _ => "",
    }
}

/// Convert a modifier kind to its flag.
fn modifier_to_flag(kind: SyntaxKind) -> ModifierFlags {
    match kind {
        SyntaxKind::PublicKeyword => ModifierFlags::Public,
        SyntaxKind::ProtectedKeyword => ModifierFlags::Protected,
        SyntaxKind::PrivateKeyword => ModifierFlags::Private,
        SyntaxKind::StaticKeyword => ModifierFlags::Static,
        SyntaxKind::ReadonlyKeyword => ModifierFlags::Readonly,
        SyntaxKind::OverrideKeyword => ModifierFlags::Override,
        SyntaxKind::ExportKeyword => ModifierFlags::Export,
        SyntaxKind::AbstractKeyword => ModifierFlags::Abstract,
        SyntaxKind::DeclareKeyword => ModifierFlags::Ambient,
        SyntaxKind::AccessorKeyword => ModifierFlags::Accessor,
        SyntaxKind::AsyncKeyword => ModifierFlags::Async,
        SyntaxKind::DefaultKeyword => ModifierFlags::Default,
        SyntaxKind::ConstKeyword => ModifierFlags::Const,
        SyntaxKind::InKeyword => ModifierFlags::In,
        SyntaxKind::OutKeyword => ModifierFlags::Out,
        _ => ModifierFlags::empty(),
    }
}

// ────────────────────────────────────────────────────────────────────────────
// JSX grammar checks
// ────────────────────────────────────────────────────────────────────────────

impl Checker {
    /// Validate a JSX element's tag name and attribute list.
    ///
    /// Mirrors Go's `checkGrammarJsxElement`:
    /// - Validate the tag name (namespace-name rules).
    /// - Validate any explicit type arguments (JSX elements cannot have
    ///   type arguments).
    /// - Reject duplicate attribute names (TS17001).
    /// - Reject empty JSX attribute expressions (TS17000).
    pub fn check_grammar_jsx_element(&mut self, node: &Arc<Node>) -> bool {
        let tag_name = match super::jsx::jsx_tag_name(node) {
            Some(t) => t,
            None => return false,
        };

        if self.check_grammar_jsx_name(&tag_name) {
            return true;
        }

        // Type arguments are not allowed on JSX elements.
        let type_args: Option<Vec<Arc<Node>>> = match &node.data {
            NodeData::JsxOpeningElement(data) => {
                data.type_arguments.as_ref().map(|l| l.iter().cloned().collect())
            }
            NodeData::JsxSelfClosingElement(data) => {
                data.type_arguments.as_ref().map(|l| l.iter().cloned().collect())
            }
            _ => None,
        };
        if let Some(args) = type_args {
            if !args.is_empty() {
                // TS2558: Expected 0 type arguments, got N.
                // Reuse the generic message; the Go implementation uses
                // diagnostics.Expected_0_type_arguments_but_got_1.
                let count = args.len().to_string();
                return self.grammar_error_on_node_with_args(
                    node,
                    &EXPECTED_0_TYPE_ARGUMENTS_BUT_GOT_1,
                    &["0".to_string(), count],
                );
            }
        }

        // Check for duplicate attribute names and empty JSX expressions.
        let attrs = match super::jsx::jsx_attributes(node) {
            Some(a) => a,
            None => return false,
        };
        let properties: Vec<Arc<Node>> = match &attrs.data {
            NodeData::JsxAttributes(data) => data.properties.iter().cloned().collect(),
            _ => return false,
        };

        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for attr in &properties {
            if attr.kind == SyntaxKind::JsxSpreadAttribute {
                continue;
            }
            let (name_node, initializer) = match &attr.data {
                NodeData::JsxAttribute(d) => (Arc::clone(&d.name), d.initializer.clone()),
                _ => continue,
            };
            let text = name_node.text().to_string();
            if !seen.insert(text.clone()) {
                return self.grammar_error_on_node(&name_node, &JSX_ELEMENTS_CANNOT_HAVE_MULTIPLE_ATTRIBUTES_WITH_THE_SAME_NAME);
            }
            if let Some(init) = initializer {
                if init.kind == SyntaxKind::JsxExpression {
                    if let NodeData::JsxExpression(d) = &init.data {
                        if d.expression.is_none() {
                            return self.grammar_error_on_node(
                                &init,
                                &JSX_ATTRIBUTES_MUST_ONLY_BE_ASSIGNED_A_NON_EMPTY_EXPRESSION,
                            );
                        }
                    }
                }
            }
        }

        false
    }

    /// Validate a JSX tag name expression.
    ///
    /// Mirrors Go's `checkGrammarJsxName`:
    /// - TS2633: JSX property access expressions cannot include JSX
    ///   namespace names.
    /// - TS2639: React components cannot include JSX namespace names
    ///   (when JSX transform is enabled and the namespace is not
    ///   intrinsic).
    pub fn check_grammar_jsx_name(&mut self, node: &Arc<Node>) -> bool {
        // Property access whose expression is a JSX namespaced name:
        //   <foo:bar.baz />  — invalid.
        if node.kind == SyntaxKind::PropertyAccessExpression {
            if let NodeData::PropertyAccessExpression(data) = &node.data {
                let expr = &data.expression;
                if is_jsx_namespaced_name(expr) {
                    return self.grammar_error_on_node(
                        expr,
                        &JSX_PROPERTY_ACCESS_EXPRESSIONS_CANNOT_INCLUDE_JSX_NAMESPACE_NAMES,
                    );
                }
            }
        }
        // JSX namespaced name used as a React component when JSX
        // transform is enabled and the namespace isn't an intrinsic.
        if is_jsx_namespaced_name(node)
            && self.is_jsx_transform_enabled()
        {
            let namespace_text = match &node.data {
                NodeData::JsxNamespacedName(data) => data.namespace.text().to_string(),
                _ => String::new(),
            };
            if !super::jsx::is_intrinsic_jsx_name(&namespace_text) {
                return self.grammar_error_on_node(
                    node,
                    &REACT_COMPONENTS_CANNOT_INCLUDE_JSX_NAMESPACE_NAMES,
                );
            }
        }
        false
    }

    /// Validate a JSX expression (`{...}`).
    ///
    /// Mirrors Go's `checkGrammarJsxExpression`:
    /// - TS18007: JSX expressions may not use the comma operator.
    pub fn check_grammar_jsx_expression(&mut self, node: &Arc<Node>) -> bool {
        let expr = match &node.data {
            NodeData::JsxExpression(data) => &data.expression,
            _ => return false,
        };
        let Some(expr) = expr else { return false };
        // A comma sequence is a BinaryExpression with a CommaToken.
        if is_comma_sequence(expr) {
            return self.grammar_error_on_node(
                expr,
                &JSX_EXPRESSIONS_MAY_NOT_USE_THE_COMMA_OPERATOR_DID_YOU_MEAN_TO_WRITE_AN_ARRAY,
            );
        }
        false
    }

    /// Whether JSX emit/transform is enabled (i.e. `--jsx` is not `None`).
    fn is_jsx_transform_enabled(&self) -> bool {
        self.compiler_options.jsx != crate::core::compiler_options::JsxEmit::None
    }
}

/// Whether `node` is a comma-sequence expression (`a, b`).
///
/// Mirrors Go's `ast.IsCommaSequence`.
fn is_comma_sequence(node: &Arc<Node>) -> bool {
    if node.kind != SyntaxKind::BinaryExpression {
        return false;
    }
    match &node.data {
        NodeData::BinaryExpression(data) => data.operator_token.kind == SyntaxKind::CommaToken,
        _ => false,
    }
}
