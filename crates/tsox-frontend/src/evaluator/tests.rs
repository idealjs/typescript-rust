use super::*;
use crate::ast::*;
use crate::parser::Parser;

fn eval_expr(source: &str) -> EvalResult {
    let file = Parser::parse_source_file_text("test.ts", source.to_string());
    let stmts = match &file.node.data {
        NodeData::SourceFile(d) => &d.statements,
        _ => unreachable!(),
    };
    assert!(!stmts.nodes.is_empty());
    let expr = match &stmts.nodes[0].data {
        NodeData::ExpressionStatement(d) => &d.expression,
        _ => unreachable!(),
    };
    evaluate_expression(expr, None, |_, _| EvalResult::none())
}

#[test]
fn eval_numeric_literal() {
    let result = eval_expr("42;");
    match result.value {
        Some(EvalValue::Number(n)) => assert_eq!(n.0, 42.0),
        _ => panic!("Expected number"),
    }
}

#[test]
fn eval_string_literal() {
    let result = eval_expr("\"hello\";");
    match result.value {
        Some(EvalValue::String(s)) => assert_eq!(s, "hello"),
        _ => panic!("Expected string"),
    }
}

#[test]
fn eval_binary_add() {
    let result = eval_expr("1 + 2;");
    match result.value {
        Some(EvalValue::Number(n)) => assert_eq!(n.0, 3.0),
        _ => panic!("Expected number"),
    }
}

#[test]
fn eval_binary_multiply() {
    let result = eval_expr("3 * 4;");
    match result.value {
        Some(EvalValue::Number(n)) => assert_eq!(n.0, 12.0),
        _ => panic!("Expected number"),
    }
}

#[test]
fn eval_unary_minus() {
    let result = eval_expr("-5;");
    match result.value {
        Some(EvalValue::Number(n)) => assert_eq!(n.0, -5.0),
        _ => panic!("Expected number"),
    }
}

#[test]
fn eval_string_concat() {
    let result = eval_expr("\"a\" + \"b\";");
    match result.value {
        Some(EvalValue::String(s)) => assert_eq!(s, "ab"),
        _ => panic!("Expected string"),
    }
}
