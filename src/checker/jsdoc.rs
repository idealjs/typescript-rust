//! JSDoc-related type checks.
//!
//! Ported from `internal/checker/jsdoc.go` in the Go implementation
//! (~100 lines). The only check currently in that file is
//! `checkUnmatchedJSDocParameters`, which verifies that every `@param`
//! JSDoc tag on a function corresponds to an actual parameter.
//!
//! ## Status
//!
//! JSDoc parsing itself (`internal/parser/jsdoc.go`, ~1355 lines) is not
//! yet ported — see P2.7 in `TODO.md`. As a result, no `JSDoc` nodes
//! are ever attached to declarations, so `get_all_jsdoc_tags` always
//! returns an empty slice and `check_unmatched_jsdoc_parameters` is a
//! no-op. The module is wired into the checker today so that once JSDoc
//! parsing lands, the check activates automatically.
//!
//! The independent helper `contains_arguments_reference` is fully
//! implemented: it walks a function body looking for `arguments`
//! identifier references, which does not depend on JSDoc parsing.

use std::sync::Arc;

use crate::ast::{is_binding_pattern, is_identifier, Node, NodeData, SyntaxKind};

use super::checker::Checker;

impl Checker {
    /// Check that every `@param` JSDoc tag on `node` matches an actual
    /// parameter of `node`.
    ///
    /// Mirrors Go's `checkUnmatchedJSDocParameters`. Emits:
    /// - TS8024 (`JSDoc_param_tag_has_name_0_but_there_is_no_parameter_with_that_name`)
    ///   when a `@param` tag's name doesn't match any parameter.
    /// - TS8029 (`..._It_would_match_arguments_if_it_had_an_array_type`)
    ///   when the unmatched `@param` could correspond to `arguments`
    ///   but its type isn't an array type.
    /// - TS8032 (`Qualified_name_0_is_not_allowed_without_a_leading_param_object_1`)
    ///   when a `@param` tag uses a qualified name like `obj.prop`
    ///   without a leading `param object`.
    ///
    /// Until JSDoc parsing (P2.7) lands, this is a no-op because
    /// `get_all_jsdoc_tags` returns an empty slice.
    pub fn check_unmatched_jsdoc_parameters(&mut self, node: &Arc<Node>) {
        let jsdoc_parameters = self.get_all_jsdoc_parameter_tags(node);
        if jsdoc_parameters.is_empty() {
            return;
        }

        let is_js = node.flags.contains(crate::ast::NodeFlags::JavaScriptFile);

        // Collect parameter names from the actual function signature.
        let parameters = match &node.data {
            NodeData::FunctionDeclaration(d) => &d.parameters,
            NodeData::FunctionExpression(d) => &d.parameters,
            NodeData::ArrowFunction(d) => &d.parameters,
            NodeData::MethodDeclaration(d) => &d.parameters,
            NodeData::ConstructorDeclaration(d) => &d.parameters,
            NodeData::GetAccessorDeclaration(d) => &d.parameters,
            NodeData::SetAccessorDeclaration(d) => &d.parameters,
            NodeData::CallSignatureDeclaration(d) => &d.parameters,
            NodeData::ConstructSignatureDeclaration(d) => &d.parameters,
            NodeData::MethodSignatureDeclaration(d) => &d.parameters,
            _ => return,
        };

        let mut param_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut excluded: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for (i, param) in parameters.iter().enumerate() {
            let name = match &param.data {
                NodeData::ParameterDeclaration(d) => &d.name,
                _ => continue,
            };
            if is_identifier(name) {
                param_names.insert(name.text().to_string());
            }
            if is_binding_pattern(name) {
                excluded.insert(i);
            }
        }

        if self.contains_arguments_reference(node) {
            // The function body references `arguments`: the last
            // `@param` tag is allowed to be unmatched if it has an
            // array-typed JSDoc annotation.
            if is_js {
                let last_idx = jsdoc_parameters.len() - 1;
                let last_tag = &jsdoc_parameters[last_idx];
                let (last_name, last_type_expr) = match &last_tag.data {
                    NodeData::JSDocParameterOrPropertyTag(d) => {
                        (&d.name, &d.type_expression)
                    }
                    _ => return,
                };
                if !is_identifier(last_name) {
                    return;
                }
                if excluded.contains(&last_idx) || param_names.contains(last_name.text()) {
                    return;
                }
                let Some(type_expr) = last_type_expr else { return };
                let type_node = match &type_expr.data {
                    NodeData::JSDocTypeExpression(d) => &d.type_node,
                    _ => return,
                };
                let tag_type = self.get_type_from_type_node(type_node);
                if self.is_array_type(&tag_type) {
                    return;
                }
                self.grammar_error_on_node_with_args(
                    last_name,
                    &crate::diagnostics::messages_generated::JSDOC_PARAM_TAG_HAS_NAME_0_BUT_THERE_IS_NO_PARAMETER_WITH_THAT_NAME_IT_WOULD_MATCH_ARGUMENTS_IF_IT_HAD_AN_ARRAY_TYPE,
                    &[last_name.text().to_string()],
                );
            }
        } else {
            for (index, tag) in jsdoc_parameters.iter().enumerate() {
                let (name, is_name_first) = match &tag.data {
                    NodeData::JSDocParameterOrPropertyTag(d) => (&d.name, d.is_name_first),
                    _ => continue,
                };
                if excluded.contains(&index)
                    || (is_identifier(name) && param_names.contains(name.text()))
                {
                    continue;
                }
                if name.kind == SyntaxKind::QualifiedName {
                    if is_js {
                        // TS8032: Qualified name not allowed without a
                        // leading param object.
                        let full = name.text().to_string();
                        let left = match &name.data {
                            NodeData::QualifiedName(d) => d.left.text().to_string(),
                            _ => String::new(),
                        };
                        self.grammar_error_on_node_with_args(
                            name,
                            &crate::diagnostics::messages_generated::QUALIFIED_NAME_0_IS_NOT_ALLOWED_WITHOUT_A_LEADING_PARAM_OBJECT_1,
                            &[full, left],
                        );
                    }
                } else if !is_name_first {
                    // TS8024 (suggestion in .js, error in .ts).
                    self.grammar_error_on_node_with_args(
                        name,
                        &crate::diagnostics::messages_generated::JSDOC_PARAM_TAG_HAS_NAME_0_BUT_THERE_IS_NO_PARAMETER_WITH_THAT_NAME,
                        &[name.text().to_string()],
                    );
                }
            }
        }
    }

    /// Collect all `@param` JSDoc tags attached to `node`.
    ///
    /// Mirrors Go's `getAllJSDocTags` filtered to `KindJSDocParameterTag`.
    /// Returns an empty slice until JSDoc parsing (P2.7) is implemented,
    /// since no `JSDoc` nodes are attached to declarations today.
    fn get_all_jsdoc_parameter_tags(&self, _node: &Arc<Node>) -> Vec<Arc<Node>> {
        // TODO(P2.7): once JSDoc parsing is implemented, walk
        // `node.js_doc` (or the equivalent side table) and collect
        // JSDocParameterTag / JSDocPropertyTag nodes whose name is
        // non-empty.
        Vec::new()
    }

    /// Whether the body of `node` (a function-like declaration)
    /// references the `arguments` identifier.
    ///
    /// Mirrors Go's `containsArgumentsReference`. Walks the function
    /// body looking for `Identifier` nodes with text `"arguments"`.
    /// Skips nested function-like declarations (their `arguments`
    /// references are scoped to the inner function).
    pub fn contains_arguments_reference(&self, node: &Arc<Node>) -> bool {
        let body: Option<Arc<Node>> = match &node.data {
            NodeData::FunctionDeclaration(d) => d.body.as_ref().map(Arc::clone),
            NodeData::FunctionExpression(d) => Some(Arc::clone(&d.body)),
            NodeData::MethodDeclaration(d) => d.body.as_ref().map(Arc::clone),
            NodeData::ConstructorDeclaration(d) => d.body.as_ref().map(Arc::clone),
            NodeData::GetAccessorDeclaration(d) => d.body.as_ref().map(Arc::clone),
            NodeData::SetAccessorDeclaration(d) => d.body.as_ref().map(Arc::clone),
            _ => None,
        };
        let Some(body) = body else { return false };
        // Arrow functions don't have their own `arguments`; they use the
        // enclosing function's. Walk the body but stop at nested non-arrow
        // function boundaries.
        let mut found = false;
        self.walk_for_arguments(&body, &mut found);
        found
    }

    /// Recursive walker for `contains_arguments_reference`.
    fn walk_for_arguments(&self, node: &Arc<Node>, found: &mut bool) {
        if *found {
            return;
        }
        // Stop at nested function boundaries (except arrows, which inherit
        // the enclosing `arguments`).
        match node.kind {
            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::ClassStaticBlockDeclaration => {
                // Don't recurse into the nested function's body.
                return;
            }
            _ => {}
        }
        if node.kind == SyntaxKind::Identifier {
            if let NodeData::Identifier(d) = &node.data {
                if d.text == "arguments" {
                    *found = true;
                    return;
                }
            }
        }
        crate::ast::node_data_generated::for_each_child(node, |child| {
            self.walk_for_arguments(child, found);
            false
        });
    }
}
