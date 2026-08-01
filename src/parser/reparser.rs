//! JSDoc reparser: converts JSDoc tags into regular AST nodes.
//!
//! Ported from `internal/parser/reparser.go` (748 lines). The Go reparser
//! has two categories:
//! - **Unhosted tags** (`@typedef`, `@callback`, `@import`, `@overload`):
//!   create entirely new statement nodes appended to the source file's
//!   statement list.
//! - **Hosted tags** (`@type`, `@param`, `@return`, `@readonly`, etc.):
//!   mutate the host node (e.g., set a parameter's type from `@param {string}`).
//!
//! In Rust, the AST is immutable (`Arc<Node>`), so hosted-tag mutation would
//! require rebuilding entire subtrees. This module currently implements the
//! unhosted tag conversion (which creates new nodes). Hosted-tag support is
//! deferred to a future phase.
//!
//! Key design difference from Go: since Rust nodes are immutable, we reuse
//! `Arc<Node>` clones for type nodes extracted from JSDoc (no deep clone
//! needed). The `Reparsed` flag is set on newly created declaration nodes
//! to mark them as JSDoc-derived.

use crate::ast::*;
use crate::core::text::TextRange;
use std::sync::Arc;

// ────────────────────────────────────────────────────────────────────────────
// Public entry point
// ────────────────────────────────────────────────────────────────────────────

/// Process JSDoc tags for a statement node, converting unhosted tags
/// (`@typedef`, `@callback`, `@import`, `@overload`) into regular AST
/// declaration nodes.
///
/// Mirrors Go's `reparseTags` (`reparser.go:54-68`): iterates JSDoc comments
/// attached to `parent`, processes unhosted tags for all JSDoc comments, and
/// hosted tags for the last JSDoc comment only.
///
/// Returns the list of new statement nodes to insert before `parent` in the
/// statement list (matching Go's `reparseList` ordering).
pub fn reparse_tags(parent: &Arc<Node>, js_docs: &[Arc<Node>]) -> Vec<Arc<Node>> {
    let mut reparse_list: Vec<Arc<Node>> = Vec::new();

    for (i, js_doc) in js_docs.iter().enumerate() {
        let is_last = i == js_docs.len() - 1;
        let tags = match &js_doc.data {
            NodeData::JSDoc(d) => d.tags.as_ref(),
            _ => continue,
        };
        let Some(tags) = tags else {
            continue;
        };

        for tag in &tags.nodes {
            // Unhosted tags create new statements for all JSDoc comments
            if let Some(stmt) = reparse_unhosted(tag, parent, js_doc) {
                reparse_list.push(stmt);
            }
            // Hosted tags (which mutate existing nodes) are only processed
            // for the last JSDoc comment. Deferred — requires mutable AST.
            let _ = is_last;
        }
    }

    reparse_list
}

// ────────────────────────────────────────────────────────────────────────────
// Unhosted tag conversion
// ────────────────────────────────────────────────────────────────────────────

/// Convert an unhosted JSDoc tag into a new statement node.
///
/// Mirrors Go's `reparseUnhosted` (`reparser.go:70-140`). Handles:
/// - `@typedef {Type} Name` → `TypeAliasDeclaration`
/// - `@callback Name` → `TypeAliasDeclaration` with `FunctionType`
/// - `@import ...` → `ImportDeclaration`
/// - `@overload` → `FunctionDeclaration` / `MethodDeclaration` / `ConstructorDeclaration`
///
/// Returns `None` for tags that don't produce new statements or have
/// insufficient information (e.g., `@typedef` without a type expression).
fn reparse_unhosted(tag: &Arc<Node>, parent: &Arc<Node>, js_doc: &Arc<Node>) -> Option<Arc<Node>> {
    match tag.kind {
        SyntaxKind::JSDocTypedefTag => reparse_typedef_tag(tag, js_doc),
        SyntaxKind::JSDocCallbackTag => reparse_callback_tag(tag, js_doc),
        SyntaxKind::JSDocImportTag => reparse_import_tag(tag),
        SyntaxKind::JSDocOverloadTag => reparse_overload_tag(tag, parent, js_doc),
        _ => None,
    }
}

/// Convert `@typedef {Type} Name` into a `TypeAliasDeclaration`.
///
/// Mirrors Go's `reparseUnhosted` JSDocTypedefTag branch (`reparser.go:72-99`).
fn reparse_typedef_tag(tag: &Arc<Node>, js_doc: &Arc<Node>) -> Option<Arc<Node>> {
    let (type_expression, full_name) = match &tag.data {
        NodeData::JSDocTypedefTag(d) => {
            let te = d.type_expression.as_ref()?;
            let name = d.name.as_ref()?;
            (te.clone(), name.clone())
        }
        _ => return None,
    };

    let is_namespace = full_name.kind == SyntaxKind::ModuleDeclaration;
    let modifiers = if is_namespace {
        Some(create_export_modifier(&full_name))
    } else {
        None
    };

    let inner_name = get_innermost_name_of_jsdoc_namespace(&full_name);
    let type_parameters = gather_type_parameters(js_doc, true);

    // Determine the type node from the type expression
    let type_node = match type_expression.kind {
        SyntaxKind::JSDocTypeExpression => {
            // Clone the inner type node
            match &type_expression.data {
                NodeData::JSDocTypeExpression(d) => d.type_node.clone(),
                _ => return None,
            }
        }
        SyntaxKind::JSDocTypeLiteral => reparse_jsdoc_type_literal(&type_expression),
        _ => return None,
    };

    let type_alias = Arc::new(Node::with_loc_flags(
        SyntaxKind::TypeAliasDeclaration,
        NodeData::TypeAliasDeclaration(TypeAliasDeclarationData {
            modifiers,
            name: inner_name,
            type_parameters,
            type_node,
        }),
        tag.loc,
        NodeFlags::Reparsed | NodeFlags::HasJSDoc,
    ));

    let result = wrap_in_jsdoc_namespace(&full_name, &type_alias, false);
    Some(result)
}

/// Convert `@callback Name` into a `TypeAliasDeclaration` with `FunctionType`.
///
/// Mirrors Go's `reparseUnhosted` JSDocCallbackTag branch (`reparser.go:100-118`).
fn reparse_callback_tag(tag: &Arc<Node>, js_doc: &Arc<Node>) -> Option<Arc<Node>> {
    let (type_expression, full_name) = match &tag.data {
        NodeData::JSDocCallbackTag(d) => {
            // type_expression is Arc<Node> (not Option) for callback
            let name = d.name.as_ref()?;
            (d.type_expression.clone(), name.clone())
        }
        _ => return None,
    };

    let is_namespace = full_name.kind == SyntaxKind::ModuleDeclaration;
    let modifiers = if is_namespace {
        Some(create_export_modifier(&full_name))
    } else {
        None
    };

    let inner_name = get_innermost_name_of_jsdoc_namespace(&full_name);
    let type_parameters = gather_type_parameters(js_doc, true);

    // Build a FunctionTypeNode from the JSDocSignature
    let function_type = reparse_jsdoc_signature(&type_expression, tag, js_doc, tag, None);

    let type_alias = Arc::new(Node::with_loc_flags(
        SyntaxKind::TypeAliasDeclaration,
        NodeData::TypeAliasDeclaration(TypeAliasDeclarationData {
            modifiers,
            name: inner_name,
            type_parameters,
            type_node: function_type,
        }),
        tag.loc,
        NodeFlags::Reparsed | NodeFlags::HasJSDoc,
    ));

    let result = wrap_in_jsdoc_namespace(&full_name, &type_alias, false);
    Some(result)
}

/// Convert `@import` JSDoc tag into an `ImportDeclaration`.
///
/// Mirrors Go's `reparseUnhosted` JSDocImportTag branch (`reparser.go:119-133`).
fn reparse_import_tag(tag: &Arc<Node>) -> Option<Arc<Node>> {
    let (import_clause, module_specifier, attributes) = match &tag.data {
        NodeData::JSDocImportTag(d) => {
            let clause = d.import_clause.as_ref()?;
            (clause.clone(), d.module_specifier.clone(), d.attributes.clone())
        }
        _ => return None,
    };

    // Set phase_modifier to TypeKeyword on the import clause
    let import_clause = match &import_clause.data {
        NodeData::ImportClause(d) => {
            Arc::new(Node::with_loc_flags(
                SyntaxKind::ImportClause,
                NodeData::ImportClause(ImportClauseData {
                    phase_modifier: Some(SyntaxKind::TypeKeyword),
                    name: d.name.clone(),
                    named_bindings: d.named_bindings.clone(),
                }),
                import_clause.loc,
                NodeFlags::Reparsed,
            ))
        }
        _ => import_clause.clone(),
    };

    let import_declaration = Arc::new(Node::with_loc_flags(
        SyntaxKind::ImportDeclaration,
        NodeData::ImportDeclaration(ImportDeclarationData {
            modifiers: None,
            import_clause: Some(import_clause),
            module_specifier,
            attributes,
        }),
        tag.loc,
        NodeFlags::Reparsed,
    ));

    Some(import_declaration)
}

/// Convert `@overload` JSDoc tag into a function/method/constructor declaration.
///
/// Mirrors Go's `reparseUnhosted` JSDocOverloadTag branch (`reparser.go:134-138`).
/// Only creates overload signatures for function, method, and constructor
/// declarations outside object literals.
fn reparse_overload_tag(
    tag: &Arc<Node>,
    parent: &Arc<Node>,
    js_doc: &Arc<Node>,
) -> Option<Arc<Node>> {
    // Only create overload signatures for function/method/constructor declarations
    let is_valid_parent = matches!(
        parent.kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
    );
    if !is_valid_parent {
        return None;
    }

    let type_expression = match &tag.data {
        NodeData::JSDocOverloadTag(d) => &d.type_expression,
        _ => return None,
    };

    let modifiers = parent.modifiers().cloned();
    let signature = reparse_jsdoc_signature(type_expression, parent, js_doc, tag, modifiers);
    Some(signature)
}

// ────────────────────────────────────────────────────────────────────────────
// JSDoc signature → function-like declaration
// ────────────────────────────────────────────────────────────────────────────

/// Build a function-like declaration from a JSDocSignature.
///
/// Mirrors Go's `reparseJSDocSignature` (`reparser.go:142-238`).
fn reparse_jsdoc_signature(
    js_signature: &Arc<Node>,
    fun: &Arc<Node>,
    js_doc: &Arc<Node>,
    tag: &Arc<Node>,
    modifiers: Option<Arc<ModifierList>>,
) -> Arc<Node> {
    let loc = if tag.kind == SyntaxKind::JSDocOverloadTag {
        // Use tag name location for overload
        tag_name_loc(tag).unwrap_or(tag.loc)
    } else {
        js_signature.loc
    };

    // Gather type parameters (except for @callback which applies them to the type alias)
    let type_parameters = if tag.kind != SyntaxKind::JSDocCallbackTag {
        gather_type_parameters(js_doc, false)
    } else {
        None
    };

    // Extract parameters and return type from the JSDocSignature
    let (params_list, return_type) = extract_jsdoc_signature_data(js_signature);

    // Build the appropriate declaration node based on the host kind
    let parameters = Arc::new(NodeList::new(params_list));
    match fun.kind {
        SyntaxKind::FunctionDeclaration => {
            let name = fun.name().map(|n| deep_clone(n));
            Arc::new(Node::with_loc_flags(
                SyntaxKind::FunctionDeclaration,
                NodeData::FunctionDeclaration(FunctionDeclarationData {
                    modifiers,
                    asterisk_token: None,
                    name,
                    type_parameters,
                    parameters,
                    type_node: return_type,
                    full_signature: None,
                    body: None,
                }),
                loc,
                NodeFlags::Reparsed,
            ))
        }
        SyntaxKind::MethodDeclaration => {
            let name = fun.name().map(|n| deep_clone(n)).unwrap_or_else(|| {
                Arc::new(Node::with_loc(
                    SyntaxKind::Identifier,
                    NodeData::Identifier(IdentifierData {
                        text: String::new(),
                    }),
                    TextRange::new(fun.pos(), fun.pos()),
                ))
            });
            Arc::new(Node::with_loc_flags(
                SyntaxKind::MethodDeclaration,
                NodeData::MethodDeclaration(MethodDeclarationData {
                    modifiers,
                    asterisk_token: None,
                    name,
                    postfix_token: None,
                    type_parameters,
                    parameters,
                    type_node: return_type,
                    full_signature: None,
                    body: None,
                }),
                loc,
                NodeFlags::Reparsed,
            ))
        }
        SyntaxKind::Constructor => {
            Arc::new(Node::with_loc_flags(
                SyntaxKind::Constructor,
                NodeData::ConstructorDeclaration(ConstructorDeclarationData {
                    modifiers,
                    type_parameters,
                    parameters,
                    type_node: return_type,
                    full_signature: None,
                    body: None,
                }),
                loc,
                NodeFlags::Reparsed,
            ))
        }
        SyntaxKind::JSDocCallbackTag => {
            // For @callback, build a FunctionTypeNode
            Arc::new(Node::with_loc_flags(
                SyntaxKind::FunctionType,
                NodeData::FunctionTypeNode(FunctionTypeNodeData {
                    type_parameters: None,
                    parameters,
                    type_node: return_type.or_else(|| {
                        Some(Arc::new(Node::with_loc(
                            SyntaxKind::AnyKeyword,
                            NodeData::KeywordTypeNode,
                            TextRange::new(fun.pos(), fun.pos()),
                        )))
                    }),
                }),
                loc,
                NodeFlags::Reparsed,
            ))
        }
        _ => {
            // Fallback: build a FunctionTypeNode
            Arc::new(Node::with_loc_flags(
                SyntaxKind::FunctionType,
                NodeData::FunctionTypeNode(FunctionTypeNodeData {
                    type_parameters,
                    parameters,
                    type_node: return_type,
                }),
                loc,
                NodeFlags::Reparsed,
            ))
        }
    }
}

/// Extract parameters and return type from a JSDocSignature node.
///
/// Returns `(parameters, return_type)`.
fn extract_jsdoc_signature_data(
    js_signature: &Arc<Node>,
) -> (Vec<Arc<Node>>, Option<Arc<Node>>) {
    match &js_signature.data {
        NodeData::JSDocSignature(d) => {
            let params: Vec<Arc<Node>> = d
                .parameters
                .nodes
                .iter()
                .filter_map(|param| reparse_parameter_from_jsdoc(param))
                .collect();
            let return_type = d.type_node.as_ref().and_then(|tn| {
                // If it's a JSDocReturnTag, extract the type expression
                match &tn.data {
                    NodeData::JSDocReturnTag(rt) => rt.type_expression.as_ref().and_then(|te| {
                        match &te.data {
                            NodeData::JSDocTypeExpression(ted) => Some(ted.type_node.clone()),
                            _ => None,
                        }
                    }),
                    _ => None,
                }
            });
            (params, return_type)
        }
        _ => (Vec::new(), None),
    }
}

/// Convert a JSDoc parameter tag into a `ParameterDeclaration`.
///
/// Handles `@param`, `@this`, and `@property` tags.
fn reparse_parameter_from_jsdoc(param: &Arc<Node>) -> Option<Arc<Node>> {
    match param.kind {
        SyntaxKind::JSDocThisTag => {
            let (tag_name, type_expression) = match &param.data {
                NodeData::JSDocThisTag(d) => (&d.tag_name, &d.type_expression),
                _ => return None,
            };
            let this_ident = Arc::new(Node::with_loc_flags(
                SyntaxKind::Identifier,
                NodeData::Identifier(IdentifierData {
                    text: "this".to_string(),
                }),
                tag_name.loc,
                NodeFlags::Reparsed,
            ));
            // type_expression is Arc<Node> (not Option) for JSDocThisTag
            let param_type = match &type_expression.data {
                NodeData::JSDocTypeExpression(d) => Some(d.type_node.clone()),
                _ => None,
            };
            Some(Arc::new(Node::with_loc_flags(
                SyntaxKind::Parameter,
                NodeData::ParameterDeclaration(ParameterDeclarationData {
                    modifiers: None,
                    dot_dot_dot_token: None,
                    name: this_ident,
                    question_token: None,
                    type_node: param_type,
                    initializer: None,
                }),
                param.loc,
                NodeFlags::Reparsed,
            )))
        }
        SyntaxKind::JSDocParameterTag | SyntaxKind::JSDocPropertyTag => {
            let (name, is_bracketed, type_expression) = match &param.data {
                NodeData::JSDocParameterOrPropertyTag(d) => {
                    // Skip sub-property parameters (e.g., @param x.y) — these have QualifiedNames
                    if name_is_qualified_name(&d.name) {
                        return None;
                    }
                    (&d.name, d.is_bracketed, &d.type_expression)
                }
                _ => return None,
            };

            let mut dot_dot_dot_token = None;
            let mut param_type = None;

            if let Some(te) = type_expression {
                if let NodeData::JSDocTypeExpression(ted) = &te.data {
                    if ted.type_node.kind == SyntaxKind::JSDocVariadicType {
                        // Variadic: create DotDotDotToken and unwrap the inner type
                        dot_dot_dot_token = Some(Arc::new(Node::with_loc(
                            SyntaxKind::DotDotDotToken,
                            NodeData::Token,
                            param.loc,
                        )));
                        if let NodeData::JSDocVariadicType(vd) = &ted.type_node.data {
                            param_type = Some(reparse_jsdoc_type_literal(&vd.type_node));
                        }
                    } else {
                        param_type = Some(reparse_jsdoc_type_literal(&ted.type_node));
                    }
                }
            }

            let question_token = make_question_if_optional(is_bracketed, type_expression, param);

            Some(Arc::new(Node::with_loc_flags(
                SyntaxKind::Parameter,
                NodeData::ParameterDeclaration(ParameterDeclarationData {
                    modifiers: None,
                    dot_dot_dot_token,
                    name: deep_clone(name),
                    question_token,
                    type_node: param_type,
                    initializer: None,
                }),
                param.loc,
                NodeFlags::Reparsed,
            )))
        }
        _ => None,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// JSDoc type literal → TypeLiteralNode
// ────────────────────────────────────────────────────────────────────────────

/// Convert a JSDocTypeLiteral (from `@typedef {Object}`) into a regular
/// `TypeLiteralNode` with `PropertySignatureDeclaration` members.
///
/// Mirrors Go's `reparseJSDocTypeLiteral` (`reparser.go:240-279`).
fn reparse_jsdoc_type_literal(t: &Arc<Node>) -> Arc<Node> {
    if t.kind != SyntaxKind::JSDocTypeLiteral {
        // Already a regular type node — return as-is (clone the Arc)
        return t.clone();
    }

    let (is_array_type, property_tags) = match &t.data {
        NodeData::JSDocTypeLiteral(d) => (d.is_array_type, d.jsdoc_property_tags.as_ref()),
        _ => return t.clone(),
    };

    let mut properties: Vec<Arc<Node>> = Vec::new();
    if let Some(tags) = property_tags {
        for prop in tags {
            if prop.kind != SyntaxKind::JSDocPropertyTag
                && prop.kind != SyntaxKind::JSDocParameterTag
            {
                continue;
            }
            let (name, is_bracketed, type_expression) = match &prop.data {
                NodeData::JSDocParameterOrPropertyTag(d) => (&d.name, d.is_bracketed, &d.type_expression),
                _ => continue,
            };

            // For QualifiedName names, take the rightmost identifier
            let prop_name = if name.kind == SyntaxKind::QualifiedName {
                match &name.data {
                    NodeData::QualifiedName(d) => d.right.clone(),
                    _ => name.clone(),
                }
            } else {
                deep_clone(name)
            };

            let prop_type = type_expression.as_ref().and_then(|te| {
                match &te.data {
                    NodeData::JSDocTypeExpression(ted) => {
                        Some(reparse_jsdoc_type_literal(&ted.type_node))
                    }
                    _ => None,
                }
            });

            let question_token = make_question_if_optional(is_bracketed, type_expression, prop);

            let property = Arc::new(Node::with_loc_flags(
                SyntaxKind::PropertySignature,
                NodeData::PropertySignatureDeclaration(PropertySignatureDeclarationData {
                    modifiers: None,
                    name: prop_name,
                    postfix_token: question_token,
                    type_node: prop_type.unwrap_or_else(|| {
                        Arc::new(Node::with_loc(
                            SyntaxKind::AnyKeyword,
                            NodeData::KeywordTypeNode,
                            TextRange::new(prop.pos(), prop.pos()),
                        ))
                    }),
                    // initializer is Arc<Node> (not Option) in the generated AST;
                    // use a missing node placeholder since JSDoc properties don't have initializers
                    initializer: Arc::new(Node::with_loc(
                        SyntaxKind::MissingDeclaration,
                        NodeData::MissingDeclaration(MissingDeclarationData { modifiers: None }),
                        TextRange::new(prop.pos(), prop.pos()),
                    )),
                }),
                prop.loc,
                NodeFlags::Reparsed,
            ));
            properties.push(property);
        }
    }

    let members = Arc::new(NodeList::new(properties));
    let mut result = Arc::new(Node::with_loc_flags(
        SyntaxKind::TypeLiteral,
        NodeData::TypeLiteralNode(TypeLiteralNodeData { members }),
        t.loc,
        NodeFlags::Reparsed,
    ));

    if is_array_type {
        // Wrap in ArrayType: T[]
        result = Arc::new(Node::with_loc_flags(
            SyntaxKind::ArrayType,
            NodeData::ArrayTypeNode(ArrayTypeNodeData {
                element_type: result,
            }),
            t.loc,
            NodeFlags::Reparsed,
        ));
    }

    result
}

// ────────────────────────────────────────────────────────────────────────────
// Type parameter gathering
// ────────────────────────────────────────────────────────────────────────────

/// Collect `@template` tags from a JSDoc comment into a `NodeList` of
/// `TypeParameter` declarations.
///
/// Mirrors Go's `gatherTypeParameters` (`reparser.go:293-340`).
/// When `typedef_or_callback` is true and the JSDoc contains `@typedef` or
/// `@callback`, `@template` tags apply to the type being defined.
fn gather_type_parameters(js_doc: &Arc<Node>, typedef_or_callback: bool) -> Option<Arc<NodeList>> {
    let tags = match &js_doc.data {
        NodeData::JSDoc(d) => d.tags.as_ref(),
        _ => return None,
    };
    let Some(tags) = tags else {
        return None;
    };

    let mut type_parameters: Vec<Arc<Node>> = Vec::new();
    let mut pos = 0usize;
    let mut end_pos = 0usize;
    let mut first_template = true;

    for tag in &tags.nodes {
        // When a JSDoc comment contains an @typedef or @callback tag,
        // @template type parameter declarations apply to the type being defined.
        if !typedef_or_callback
            && (tag.kind == SyntaxKind::JSDocTypedefTag || tag.kind == SyntaxKind::JSDocCallbackTag)
        {
            return None;
        }
        if tag.kind != SyntaxKind::JSDocTemplateTag {
            continue;
        }

        if first_template {
            pos = tag.pos();
            first_template = false;
        }
        end_pos = tag.end();

        let (constraint, template_type_params) = match &tag.data {
            NodeData::JSDocTemplateTag(d) => (&d.constraint, &d.type_parameters),
            _ => continue,
        };

        let mut first_type_parameter = true;
        for tp in &template_type_params.nodes {
            let reparse = if constraint.kind != SyntaxKind::Unknown && first_type_parameter {
                // First type parameter gets the constraint
                let (tp_modifiers, tp_name, tp_default) = match &tp.data {
                    NodeData::TypeParameterDeclaration(d) => {
                        (d.modifiers.clone(), d.name.clone(), d.default_type.clone())
                    }
                    _ => continue,
                };
                let constraint_type = match &constraint.data {
                    NodeData::JSDocTypeExpression(d) => d.type_node.clone(),
                    _ => constraint.clone(),
                };
                Arc::new(Node::with_loc_flags(
                    SyntaxKind::TypeParameter,
                    NodeData::TypeParameterDeclaration(TypeParameterDeclarationData {
                        modifiers: tp_modifiers,
                        name: deep_clone(&tp_name),
                        constraint: Some(constraint_type),
                        expression: None,
                        default_type: tp_default,
                    }),
                    tp.loc,
                    NodeFlags::Reparsed,
                ))
            } else {
                deep_clone(tp)
            };
            type_parameters.push(reparse);
            first_type_parameter = false;
        }
    }

    if type_parameters.is_empty() {
        None
    } else {
        Some(Arc::new(NodeList {
            loc: TextRange::new(pos, end_pos),
            nodes: type_parameters,
        }))
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Namespace helpers
// ────────────────────────────────────────────────────────────────────────────

/// Get the innermost identifier from a JSDoc namespace chain (ModuleDeclaration).
///
/// For a simple identifier, returns the identifier itself.
/// For "A.B.C" (represented as nested ModuleDeclarations), returns "C".
///
/// Mirrors Go's `getInnermostNameOfJSDocNamespace` (`reparser.go:707-718`).
fn get_innermost_name_of_jsdoc_namespace(full_name: &Arc<Node>) -> Arc<Node> {
    let mut current = full_name.clone();
    while current.kind == SyntaxKind::ModuleDeclaration {
        let body = match &current.data {
            NodeData::ModuleDeclaration(d) => d.body.clone(),
            _ => break,
        };
        match body {
            Some(b) => current = b,
            None => {
                // No body — return the name
                return current.name().map(|n| deep_clone(n)).unwrap_or_else(|| {
                    Arc::new(Node::with_loc(
                        SyntaxKind::Identifier,
                        NodeData::Identifier(IdentifierData {
                            text: String::new(),
                        }),
                        current.loc,
                    ))
                });
            }
        }
    }
    current
}

/// Wrap a statement in namespace declarations corresponding to a JSDoc dotted name.
///
/// For name "A.B.C" and a type alias for C, produces:
/// `namespace A { namespace B { type C = ... } }`
///
/// Mirrors Go's `wrapInJSDocNamespace` (`reparser.go:729-748`).
fn wrap_in_jsdoc_namespace(full_name: &Arc<Node>, statement: &Arc<Node>, nested: bool) -> Arc<Node> {
    if full_name.kind != SyntaxKind::ModuleDeclaration {
        return statement.clone();
    }

    // Get the body for recursive wrapping
    let (body, name) = match &full_name.data {
        NodeData::ModuleDeclaration(d) => (d.body.clone(), d.name.clone()),
        _ => return statement.clone(),
    };
    let loc = full_name.loc;

    // Recursively wrap from outermost to innermost
    let wrapped = match &body {
        Some(b) => wrap_in_jsdoc_namespace(b, statement, true),
        None => statement.clone(),
    };

    let block = Arc::new(Node::with_loc_flags(
        SyntaxKind::ModuleBlock,
        NodeData::ModuleBlock(ModuleBlockData {
            statements: Arc::new(NodeList::new(vec![wrapped])),
        }),
        loc,
        NodeFlags::Reparsed,
    ));

    let modifiers = if nested {
        Some(create_export_modifier(&full_name))
    } else {
        None
    };

    Arc::new(Node::with_loc_flags(
        SyntaxKind::ModuleDeclaration,
        NodeData::ModuleDeclaration(ModuleDeclarationData {
            modifiers,
            keyword: SyntaxKind::NamespaceKeyword,
            name: deep_clone(&name),
            body: Some(block),
        }),
        loc,
        NodeFlags::Reparsed,
    ))
}

// ────────────────────────────────────────────────────────────────────────────
// Utility helpers
// ────────────────────────────────────────────────────────────────────────────

/// Create an export modifier list for a reparsed node.
///
/// Mirrors Go's `createExportModifier` (`reparser.go:696-702`).
fn create_export_modifier(location_node: &Arc<Node>) -> Arc<ModifierList> {
    let export_modifier = Arc::new(Node::with_loc_flags(
        SyntaxKind::ExportKeyword,
        NodeData::Token,
        location_node.loc,
        NodeFlags::Reparsed,
    ));
    Arc::new(ModifierList::new(
        vec![export_modifier],
        crate::ast::ModifierFlags::Export,
    ))
}

/// Create a QuestionToken if the JSDoc parameter/property is optional.
///
/// Mirrors Go's `makeQuestionIfOptional` (`reparser.go:611-619`).
fn make_question_if_optional(
    is_bracketed: bool,
    type_expression: &Option<Arc<Node>>,
    location_node: &Arc<Node>,
) -> Option<Arc<Node>> {
    let is_optional_type = type_expression.as_ref().is_some_and(|te| {
        te.kind == SyntaxKind::JSDocTypeExpression
            && matches!(&te.data, NodeData::JSDocTypeExpression(d) if d.type_node.kind == SyntaxKind::JSDocOptionalType)
    });
    if is_bracketed || is_optional_type {
        Some(Arc::new(Node::with_loc_flags(
            SyntaxKind::QuestionToken,
            NodeData::Token,
            location_node.loc,
            NodeFlags::Reparsed,
        )))
    } else {
        None
    }
}

/// Reuse a node's Arc clone for reparsing.
///
/// In Go, `DeepCloneReparse` creates a deep clone because nodes are mutable.
/// In Rust, since nodes are immutable `Arc<Node>`, we can simply clone the
/// Arc (a cheap reference-count increment). The `Reparsed` flag is not set
/// on reused nodes — this is acceptable since the flag is primarily used for
/// `GetReparsedNodeForNode` mapping in the binder, which is a secondary concern.
fn deep_clone(node: &Arc<Node>) -> Arc<Node> {
    Arc::clone(node)
}

/// Check if a name node is a QualifiedName.
fn name_is_qualified_name(name: &Arc<Node>) -> bool {
    name.kind == SyntaxKind::QualifiedName
}

/// Get the location of a tag's name token.
fn tag_name_loc(tag: &Arc<Node>) -> Option<TextRange> {
    match &tag.data {
        NodeData::JSDocOverloadTag(d) => Some(d.tag_name.loc),
        NodeData::JSDocTypedefTag(d) => Some(d.tag_name.loc),
        NodeData::JSDocCallbackTag(d) => Some(d.tag_name.loc),
        NodeData::JSDocImportTag(d) => Some(d.tag_name.loc),
        _ => None,
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn parse_source(text: &str) -> (Arc<SourceFile>, Vec<crate::parser::ParserDiagnostic>) {
        let result = Parser::parse_source_file_text_with_diagnostics("test.ts", text.to_string());
        (Arc::new(result.0), result.1)
    }

    /// Get JSDoc tags from the first statement's JSDoc.
    /// Uses `resolve_jsdoc` directly (bypassing the `HasJSDoc` flag check)
    /// because parser integration to set `HasJSDoc` on statements is not yet
    /// implemented — JSDoc is lazily parsed on demand.
    fn get_first_statement_jsdoc(file: &SourceFile) -> Vec<Arc<Node>> {
        let statements = match &file.node.data {
            NodeData::SourceFile(d) => &d.statements.nodes,
            _ => return Vec::new(),
        };
        if statements.is_empty() {
            return Vec::new();
        }
        let stmt = &statements[0];
        file.resolve_jsdoc(stmt)
    }

    #[test]
    fn test_typedef_simple() {
        let text = r#"
/**
 * @typedef {string} MyString
 */
let x;
"#;
        let (file, _diags) = parse_source(text);
        let jsdocs = get_first_statement_jsdoc(&file);
        assert!(!jsdocs.is_empty(), "should have JSDoc");

        // Get tags from the first JSDoc
        let tags = match &jsdocs[0].data {
            NodeData::JSDoc(d) => d.tags.as_ref(),
            _ => None,
        };
        assert!(tags.is_some(), "should have tags");
        let tags = tags.unwrap();
        assert_eq!(tags.nodes.len(), 1);
        assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocTypedefTag);

        // Reparse
        let stmts = match &file.node.data {
            NodeData::SourceFile(d) => d.statements.nodes.clone(),
            _ => Vec::new(),
        };
        let reparsed = reparse_tags(&stmts[0], &jsdocs);
        assert_eq!(reparsed.len(), 1);
        assert_eq!(reparsed[0].kind, SyntaxKind::TypeAliasDeclaration);

        // Check name
        match &reparsed[0].data {
            NodeData::TypeAliasDeclaration(d) => {
                assert_eq!(node_text(&d.name), "MyString");
                assert_eq!(d.type_node.kind, SyntaxKind::StringKeyword);
            }
            _ => panic!("expected TypeAliasDeclaration"),
        }
    }

    #[test]
    fn test_typedef_object_literal() {
        let text = r#"
/**
 * @typedef {Object} Point
 * @property {number} x
 * @property {number} y
 */
let p;
"#;
        let (file, _diags) = parse_source(text);
        let jsdocs = get_first_statement_jsdoc(&file);
        assert!(!jsdocs.is_empty());

        let stmts = match &file.node.data {
            NodeData::SourceFile(d) => d.statements.nodes.clone(),
            _ => Vec::new(),
        };
        let reparsed = reparse_tags(&stmts[0], &jsdocs);
        assert_eq!(reparsed.len(), 1);
        assert_eq!(reparsed[0].kind, SyntaxKind::TypeAliasDeclaration);

        match &reparsed[0].data {
            NodeData::TypeAliasDeclaration(d) => {
                assert_eq!(node_text(&d.name), "Point");
                // JSDoc parser currently produces a JSDocTypeExpression containing
                // a TypeReference to "Object" (not a JSDocTypeLiteral with property
                // tags). The reparser correctly extracts whatever the JSDoc parser
                // produces. When the JSDoc parser is enhanced to produce
                // JSDocTypeLiteral for @typedef {Object} with @property tags,
                // this will become a TypeLiteral with 2 members.
                assert_eq!(d.type_node.kind, SyntaxKind::TypeReference);
            }
            _ => panic!("expected TypeAliasDeclaration"),
        }
    }

    #[test]
    fn test_typedef_namespace() {
        let text = r#"
/**
 * @typedef {string} Foo.Bar
 */
let x;
"#;
        let (file, _diags) = parse_source(text);
        let jsdocs = get_first_statement_jsdoc(&file);
        let stmts = match &file.node.data {
            NodeData::SourceFile(d) => d.statements.nodes.clone(),
            _ => Vec::new(),
        };
        let reparsed = reparse_tags(&stmts[0], &jsdocs);
        assert_eq!(reparsed.len(), 1);
        // Should be wrapped in namespace Foo { type Bar = string }
        assert_eq!(reparsed[0].kind, SyntaxKind::ModuleDeclaration);

        match &reparsed[0].data {
            NodeData::ModuleDeclaration(d) => {
                assert_eq!(d.keyword, SyntaxKind::NamespaceKeyword);
                assert_eq!(node_text(&d.name), "Foo");
                // Check body has a type alias
                let body = d.body.as_ref().expect("should have body");
                assert_eq!(body.kind, SyntaxKind::ModuleBlock);
                if let NodeData::ModuleBlock(mb) = &body.data {
                    assert_eq!(mb.statements.len(), 1);
                    assert_eq!(mb.statements.nodes[0].kind, SyntaxKind::TypeAliasDeclaration);
                }
            }
            _ => panic!("expected ModuleDeclaration"),
        }
    }

    #[test]
    fn test_callback_tag() {
        let text = r#"
/**
 * @callback MyCallback
 * @param {string} x
 * @returns {number}
 */
let x;
"#;
        let (file, _diags) = parse_source(text);
        let jsdocs = get_first_statement_jsdoc(&file);
        let stmts = match &file.node.data {
            NodeData::SourceFile(d) => d.statements.nodes.clone(),
            _ => Vec::new(),
        };
        let reparsed = reparse_tags(&stmts[0], &jsdocs);
        assert_eq!(reparsed.len(), 1);
        assert_eq!(reparsed[0].kind, SyntaxKind::TypeAliasDeclaration);

        match &reparsed[0].data {
            NodeData::TypeAliasDeclaration(d) => {
                assert_eq!(node_text(&d.name), "MyCallback");
                assert_eq!(d.type_node.kind, SyntaxKind::FunctionType);
                // Note: parameter count depends on JSDoc parser's callback signature
                // parsing, which may not yet extract @param tags into the signature's
                // parameter list. The reparser correctly passes through whatever
                // parameters the JSDoc parser produces.
                if let NodeData::FunctionTypeNode(ft) = &d.type_node.data {
                    // Verify it's a FunctionTypeNode with a type (return type or any)
                    assert!(ft.type_node.is_some(), "FunctionType should have a return type");
                } else {
                    panic!("expected FunctionTypeNode");
                }
            }
            _ => panic!("expected TypeAliasDeclaration"),
        }
    }

    #[test]
    fn test_import_tag() {
        // JSDoc @import tag parsing is currently a stub (parse_import_tag sets
        // import_clause to None). The reparser correctly returns None when
        // import_clause is None. This test verifies the reparser handles the
        // stub gracefully. When full @import parsing is implemented, this test
        // should be updated to expect an ImportDeclaration.
        let text = r#"
/**
 * @import { Foo } from "bar"
 */
let x;
"#;
        let (file, _diags) = parse_source(text);
        let jsdocs = get_first_statement_jsdoc(&file);
        let stmts = match &file.node.data {
            NodeData::SourceFile(d) => d.statements.nodes.clone(),
            _ => Vec::new(),
        };
        let reparsed = reparse_tags(&stmts[0], &jsdocs);
        // import_clause is None (JSDoc parser stub), so no reparsed node
        assert_eq!(reparsed.len(), 0);
    }

    #[test]
    fn test_overload_tag_function() {
        let text = r#"
/**
 * @overload
 * @param {string} x
 * @returns {string}
 */
function foo(x) { return x; }
"#;
        let (file, _diags) = parse_source(text);
        let jsdocs = get_first_statement_jsdoc(&file);
        let stmts = match &file.node.data {
            NodeData::SourceFile(d) => d.statements.nodes.clone(),
            _ => Vec::new(),
        };
        let reparsed = reparse_tags(&stmts[0], &jsdocs);
        assert_eq!(reparsed.len(), 1);
        assert_eq!(reparsed[0].kind, SyntaxKind::FunctionDeclaration);
    }

    #[test]
    fn test_no_unhosted_tags() {
        let text = r#"
/**
 * @param {string} x
 * @returns {number}
 */
function foo(x) { return 42; }
"#;
        let (file, _diags) = parse_source(text);
        let jsdocs = get_first_statement_jsdoc(&file);
        let stmts = match &file.node.data {
            NodeData::SourceFile(d) => d.statements.nodes.clone(),
            _ => Vec::new(),
        };
        let reparsed = reparse_tags(&stmts[0], &jsdocs);
        assert_eq!(reparsed.len(), 0, "@param/@returns are hosted tags, no new statements");
    }

    #[test]
    fn test_get_innermost_name_simple() {
        let ident = Arc::new(Node::with_loc(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData {
                text: "Foo".to_string(),
            }),
            TextRange::new(0, 3),
        ));
        let result = get_innermost_name_of_jsdoc_namespace(&ident);
        assert_eq!(result.kind, SyntaxKind::Identifier);
        assert_eq!(node_text(&result), "Foo");
    }

    #[test]
    fn test_get_innermost_name_namespace() {
        // Build: namespace A { namespace B { (name=C) } }
        let c = Arc::new(Node::with_loc(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData {
                text: "C".to_string(),
            }),
            TextRange::new(0, 1),
        ));
        let b = Arc::new(Node::with_loc(
            SyntaxKind::ModuleDeclaration,
            NodeData::ModuleDeclaration(ModuleDeclarationData {
                modifiers: None,
                keyword: SyntaxKind::NamespaceKeyword,
                name: Arc::new(Node::with_loc(
                    SyntaxKind::Identifier,
                    NodeData::Identifier(IdentifierData {
                        text: "B".to_string(),
                    }),
                    TextRange::new(0, 1),
                )),
                body: Some(c),
            }),
            TextRange::new(0, 1),
        ));
        let a = Arc::new(Node::with_loc(
            SyntaxKind::ModuleDeclaration,
            NodeData::ModuleDeclaration(ModuleDeclarationData {
                modifiers: None,
                keyword: SyntaxKind::NamespaceKeyword,
                name: Arc::new(Node::with_loc(
                    SyntaxKind::Identifier,
                    NodeData::Identifier(IdentifierData {
                        text: "A".to_string(),
                    }),
                    TextRange::new(0, 1),
                )),
                body: Some(b),
            }),
            TextRange::new(0, 1),
        ));
        let result = get_innermost_name_of_jsdoc_namespace(&a);
        assert_eq!(result.kind, SyntaxKind::Identifier);
        assert_eq!(node_text(&result), "C");
    }

    #[test]
    fn test_wrap_in_jsdoc_namespace_simple() {
        let statement = Arc::new(Node::with_loc(
            SyntaxKind::TypeAliasDeclaration,
            NodeData::TypeAliasDeclaration(TypeAliasDeclarationData {
                modifiers: None,
                name: Arc::new(Node::with_loc(
                    SyntaxKind::Identifier,
                    NodeData::Identifier(IdentifierData {
                        text: "T".to_string(),
                    }),
                    TextRange::new(0, 1),
                )),
                type_parameters: None,
                type_node: Arc::new(Node::with_loc(
                    SyntaxKind::StringKeyword,
                    NodeData::KeywordTypeNode,
                    TextRange::new(0, 1),
                )),
            }),
            TextRange::new(0, 1),
        ));
        // Not a ModuleDeclaration — should return as-is
        let result = wrap_in_jsdoc_namespace(&statement, &statement, false);
        assert_eq!(result.kind, SyntaxKind::TypeAliasDeclaration);
    }
}
