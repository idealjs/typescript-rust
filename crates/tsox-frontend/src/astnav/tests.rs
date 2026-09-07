use super::*;
use crate::parser::Parser;

#[test]
fn get_token_at_position_jsdoc_type_assertion() {
    let file_text = "function foo(x) {\n    const s = /**@type {string}*/(x)\n}";

    let position: usize = 52;
    let file = Parser::parse_source_file_text("/test.js", file_text.to_string());
    let token = get_touching_property_name(&file.node, position);
    assert!(token.is_some(), "Expected to get a token");
    let token = token.unwrap();
    assert!(
        token.kind == SyntaxKind::Identifier || token.kind == SyntaxKind::ParenthesizedExpression,
        "Expected identifier or parenthesized expression, got {:?}",
        token.kind
    );
}

#[test]
fn get_token_at_position_jsdoc_type_assertion_with_comment() {
    let file_text = "function foo(x) {\n    const s = /**@type {string}*/(x)  // comment\n}";
    let x_pos: usize = 52;
    let file = Parser::parse_source_file_text("/test.js", file_text.to_string());
    let token = get_touching_property_name(&file.node, x_pos);
    assert!(token.is_some(), "Expected to get a token");
}

#[test]
fn get_token_at_position_pointer_equality() {
    let file_text = "\n\t\t\tfunction foo() {\n\t\t\t\treturn 0;\n\t\t\t}";
    let file = Parser::parse_source_file_text("/file.ts", file_text.to_string());
    let t1 = get_token_at_position(&file.node, 0);
    let t2 = get_token_at_position(&file.node, 0);
    assert!(t1.is_some() && t2.is_some());
    assert!(
        Arc::ptr_eq(t1.as_ref().unwrap(), t2.as_ref().unwrap()),
        "Expected pointer-equal nodes for repeated calls"
    );
}

#[test]
fn get_token_at_position_baseline() {
    let file_text = "a.b";
    let file = Parser::parse_source_file_text("/f.ts", file_text.to_string());

    let pos: usize = 2;
    let token = get_token_at_position(&file.node, pos).expect("a token at position");
    assert!(
        token.pos() <= pos && pos < token.end(),
        "returned node must contain the position"
    );
    assert_eq!(token.kind, SyntaxKind::Identifier);
}

#[test]
fn get_touching_property_name_baseline() {
    let file_text = "foo.bar";
    let file = Parser::parse_source_file_text("/f.ts", file_text.to_string());

    let pos: usize = 4;
    let token = get_touching_property_name(&file.node, pos).expect("a token at position");
    assert!(
        token.pos() <= pos && pos < token.end(),
        "returned node must contain the position"
    );
    assert_eq!(token.kind, SyntaxKind::Identifier);
}

#[test]
fn find_preceding_token_baseline() {
    let file_text = "a - b";
    let file = Parser::parse_source_file_text("/f.ts", file_text.to_string());

    let token = find_preceding_token(&file.node, 4).expect("a preceding token");
    assert_eq!(token.kind, SyntaxKind::MinusToken, "Expected MinusToken");
}

#[test]
fn find_next_token_baseline() {
    let file_text = "a + b";
    let file = Parser::parse_source_file_text("/f.ts", file_text.to_string());

    let token = find_next_token(&file.node, 0).expect("a following token");
    assert_eq!(token.kind, SyntaxKind::PlusToken, "Expected PlusToken");
}

#[test]
fn find_preceding_token_after_comma_in_parameter_list() {
    let file_content = "takesCb((n, s, ))";
    let position: usize = 15;
    let file = Parser::parse_source_file_text("/file.ts", file_content.to_string());
    let token = find_preceding_token(&file.node, position);
    assert!(token.is_some(), "Expected a preceding token");
    assert_eq!(
        token.unwrap().kind,
        SyntaxKind::CommaToken,
        "Expected CommaToken"
    );
}

#[test]
fn find_preceding_token_after_dot_in_jsdoc() {
    let file_content = "a + b";
    let file = Parser::parse_source_file_text("/file.ts", file_content.to_string());

    let token = find_preceding_token(&file.node, 4);
    assert!(token.is_some(), "Expected a preceding token");
    assert_eq!(
        token.unwrap().kind,
        SyntaxKind::PlusToken,
        "Expected PlusToken"
    );
}
