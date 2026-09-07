use super::namespace::{get_innermost_name_of_jsdoc_namespace, wrap_in_jsdoc_namespace};
use crate::ast::*;
use crate::core::text::TextRange;
use std::sync::Arc;

use super::*;
use crate::parser::Parser;

pub(crate) fn parse_source(text: &str) -> (Arc<SourceFile>, Vec<crate::parser::ParserDiagnostic>) {
    let result = Parser::parse_source_file_text_with_diagnostics("test.ts", text.to_string());
    (Arc::new(result.0), result.1)
}

pub(crate) fn get_first_statement_jsdoc(file: &SourceFile) -> Vec<Arc<Node>> {
    let statements = match &file.node.data {
        NodeData::SourceFile(d) => &d.statements.nodes,
        _ => return Vec::new(),
    };
    if statements.is_empty() {
        return Vec::new();
    }

    let stmt = statements.last().unwrap();
    file.resolve_jsdoc(stmt)
}

#[test]
pub(crate) fn test_typedef_simple() {
    let text = r#"
/**
 * @typedef {string} MyString
 */
let x;
"#;
    let (file, _diags) = parse_source(text);
    let jsdocs = get_first_statement_jsdoc(&file);
    assert!(!jsdocs.is_empty(), "should have JSDoc");

    let tags = match &jsdocs[0].data {
        NodeData::JSDoc(d) => d.tags.as_ref(),
        _ => None,
    };
    assert!(tags.is_some(), "should have tags");
    let tags = tags.unwrap();
    assert_eq!(tags.nodes.len(), 1);
    assert_eq!(tags.nodes[0].kind, SyntaxKind::JSDocTypedefTag);

    let stmts = match &file.node.data {
        NodeData::SourceFile(d) => d.statements.nodes.clone(),
        _ => Vec::new(),
    };
    let reparsed = reparse_tags(&stmts[0], &jsdocs);
    assert_eq!(reparsed.len(), 1);
    assert_eq!(reparsed[0].kind, SyntaxKind::TypeAliasDeclaration);

    match &reparsed[0].data {
        NodeData::TypeAliasDeclaration(d) => {
            assert_eq!(node_text(&d.name), "MyString");
            assert_eq!(d.type_node.kind, SyntaxKind::StringKeyword);
        }
        _ => panic!("expected TypeAliasDeclaration"),
    }
}

#[test]
pub(crate) fn test_typedef_object_literal() {
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

            assert_eq!(d.type_node.kind, SyntaxKind::TypeReference);
        }
        _ => panic!("expected TypeAliasDeclaration"),
    }
}

#[test]
pub(crate) fn test_typedef_namespace() {
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

    assert_eq!(reparsed[0].kind, SyntaxKind::ModuleDeclaration);

    match &reparsed[0].data {
        NodeData::ModuleDeclaration(d) => {
            assert_eq!(d.keyword, SyntaxKind::NamespaceKeyword);
            assert_eq!(node_text(&d.name), "Foo");

            let body = d.body.as_ref().expect("should have body");
            assert_eq!(body.kind, SyntaxKind::ModuleBlock);
            if let NodeData::ModuleBlock(mb) = &body.data {
                assert_eq!(mb.statements.len(), 1);
                assert_eq!(
                    mb.statements.nodes[0].kind,
                    SyntaxKind::TypeAliasDeclaration
                );
            }
        }
        _ => panic!("expected ModuleDeclaration"),
    }
}

#[test]
pub(crate) fn test_callback_tag() {
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

            if let NodeData::FunctionTypeNode(ft) = &d.type_node.data {
                assert!(
                    ft.type_node.is_some(),
                    "FunctionType should have a return type"
                );
            } else {
                panic!("expected FunctionTypeNode");
            }
        }
        _ => panic!("expected TypeAliasDeclaration"),
    }
}

#[test]
pub(crate) fn test_import_tag() {
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

    assert_eq!(reparsed.len(), 0);
}

#[test]
pub(crate) fn test_overload_tag_function() {
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
pub(crate) fn test_no_unhosted_tags() {
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
    assert_eq!(
        reparsed.len(),
        0,
        "@param/@returns are hosted tags, no new statements"
    );
}

#[test]
pub(crate) fn test_get_innermost_name_simple() {
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
pub(crate) fn test_get_innermost_name_namespace() {
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
pub(crate) fn test_wrap_in_jsdoc_namespace_simple() {
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

    let result = wrap_in_jsdoc_namespace(&statement, &statement, false);
    assert_eq!(result.kind, SyntaxKind::TypeAliasDeclaration);
}

#[test]
pub(crate) fn test_integration_typedef_prepended_to_statements() {
    let text = r#"
/**
 * @typedef {string} MyString
 */
let x;
"#;
    let (file, _diags) = parse_source(text);
    let statements = match &file.node.data {
        NodeData::SourceFile(d) => &d.statements.nodes,
        _ => panic!("expected SourceFile"),
    };

    assert_eq!(statements.len(), 2);
    assert_eq!(statements[0].kind, SyntaxKind::TypeAliasDeclaration);
    assert_eq!(statements[1].kind, SyntaxKind::VariableStatement);

    match &statements[0].data {
        NodeData::TypeAliasDeclaration(d) => {
            assert_eq!(node_text(&d.name), "MyString");
            assert_eq!(d.type_node.kind, SyntaxKind::StringKeyword);
        }
        _ => panic!("expected TypeAliasDeclaration"),
    }
}

#[test]
pub(crate) fn test_integration_typedef_namespace_prepended() {
    let text = r#"
/**
 * @typedef {string} Foo.Bar
 */
let x;
"#;
    let (file, _diags) = parse_source(text);
    let statements = match &file.node.data {
        NodeData::SourceFile(d) => &d.statements.nodes,
        _ => panic!("expected SourceFile"),
    };

    assert_eq!(statements.len(), 2);
    assert_eq!(statements[0].kind, SyntaxKind::ModuleDeclaration);
    assert_eq!(statements[1].kind, SyntaxKind::VariableStatement);
}

#[test]
pub(crate) fn test_integration_overload_prepended_to_function() {
    let text = r#"
/**
 * @overload
 * @param {string} x
 * @returns {string}
 */
function foo(x) { return x; }
"#;
    let (file, _diags) = parse_source(text);
    let statements = match &file.node.data {
        NodeData::SourceFile(d) => &d.statements.nodes,
        _ => panic!("expected SourceFile"),
    };

    assert_eq!(statements.len(), 2);
    assert_eq!(statements[0].kind, SyntaxKind::FunctionDeclaration);
    assert_eq!(statements[1].kind, SyntaxKind::FunctionDeclaration);

    assert!(statements[0].flags.contains(NodeFlags::Reparsed));
    match &statements[0].data {
        NodeData::FunctionDeclaration(d) => {
            assert!(d.body.is_none(), "overload signature should have no body");
        }
        _ => panic!("expected FunctionDeclaration"),
    }
}

#[test]
pub(crate) fn test_integration_no_jsdoc_unchanged() {
    let text = "let x = 1;\nlet y = 2;\n";
    let (file, _diags) = parse_source(text);
    let statements = match &file.node.data {
        NodeData::SourceFile(d) => &d.statements.nodes,
        _ => panic!("expected SourceFile"),
    };
    assert_eq!(statements.len(), 2, "no JSDoc, no reparsed nodes");
}

#[test]
pub(crate) fn test_integration_hosted_tags_only_unchanged() {
    let text = r#"
/**
 * @param {string} x
 * @returns {number}
 */
function foo(x) { return 42; }
"#;
    let (file, _diags) = parse_source(text);
    let statements = match &file.node.data {
        NodeData::SourceFile(d) => &d.statements.nodes,
        _ => panic!("expected SourceFile"),
    };
    assert_eq!(statements.len(), 1, "hosted tags only, no new statements");
}
