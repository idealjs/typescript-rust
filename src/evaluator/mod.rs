//! Constant expression evaluator, ported from `internal/evaluator/`.
//!
//! Evaluates constant expressions (numeric literals, string literals,
//! template expressions, binary/unary operations on constants) for use
//! in enum member initialization, const enum evaluation, etc.

use crate::ast::*;
use crate::jsnum::{Number, PseudoBigInt};
use std::sync::Arc;

/// The result of evaluating a constant expression.
#[derive(Clone, Debug, Default)]
pub struct EvalResult {
    pub value: Option<EvalValue>,
    pub is_syntactically_string: bool,
    pub resolved_other_files: bool,
    pub has_external_references: bool,
}

impl EvalResult {
    pub fn new(
        value: Option<EvalValue>,
        is_syntactically_string: bool,
        resolved_other_files: bool,
        has_external_references: bool,
    ) -> EvalResult {
        EvalResult {
            value,
            is_syntactically_string,
            resolved_other_files,
            has_external_references,
        }
    }

    pub fn none() -> EvalResult {
        EvalResult {
            value: None,
            is_syntactically_string: false,
            resolved_other_files: false,
            has_external_references: false,
        }
    }
}

/// A evaluated constant value.
#[derive(Clone, Debug)]
pub enum EvalValue {
    Number(Number),
    String(String),
    Bool(bool),
    BigInt(PseudoBigInt),
}

impl EvalValue {
    pub fn to_string(&self) -> String {
        match self {
            EvalValue::String(s) => s.clone(),
            EvalValue::Number(n) => n.to_string(),
            EvalValue::Bool(b) => {
                if *b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            EvalValue::BigInt(b) => b.to_string(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            EvalValue::String(s) => !s.is_empty(),
            EvalValue::Number(n) => n.0 != 0.0 && !n.is_nan(),
            EvalValue::Bool(b) => *b,
            EvalValue::BigInt(b) => !b.is_zero(),
        }
    }
}

/// A function that evaluates an entity name expression (identifier or property access)
/// to a constant value. This is provided by the caller (typically the checker).
pub type EvaluateEntity = fn(&Arc<Node>, Option<&Arc<Node>>) -> EvalResult;

/// Evaluate a constant expression.
///
/// `evaluate_entity` is called when an identifier or property access is encountered.
/// It should resolve the entity to a constant value if possible.
pub fn evaluate_expression(
    expr: &Arc<Node>,
    location: Option<&Arc<Node>>,
    evaluate_entity: EvaluateEntity,
) -> EvalResult {
    match expr.kind {
        SyntaxKind::PrefixUnaryExpression => {
            if let NodeData::PrefixUnaryExpression(data) = &expr.data {
                let result = evaluate_expression(&data.operand, location, evaluate_entity);
                let mut is_syntactically_string = false;
                let resolved_other_files = result.resolved_other_files;
                let has_external_references = result.has_external_references;
                if let Some(EvalValue::Number(value)) = &result.value {
                    match data.operator {
                        SyntaxKind::PlusToken => {
                            return EvalResult::new(
                                Some(EvalValue::Number(*value)),
                                is_syntactically_string,
                                resolved_other_files,
                                has_external_references,
                            );
                        }
                        SyntaxKind::MinusToken => {
                            return EvalResult::new(
                                Some(EvalValue::Number(-*value)),
                                is_syntactically_string,
                                resolved_other_files,
                                has_external_references,
                            );
                        }
                        SyntaxKind::TildeToken => {
                            return EvalResult::new(
                                Some(EvalValue::Number(value.bitwise_not())),
                                is_syntactically_string,
                                resolved_other_files,
                                has_external_references,
                            );
                        }
                        _ => {}
                    }
                }
                is_syntactically_string = result.is_syntactically_string;
                return EvalResult::new(
                    None,
                    is_syntactically_string,
                    resolved_other_files,
                    has_external_references,
                );
            }
            EvalResult::none()
        }
        SyntaxKind::BinaryExpression => {
            if let NodeData::BinaryExpression(data) = &expr.data {
                let left = evaluate_expression(&data.left, location, evaluate_entity);
                let right = evaluate_expression(&data.right, location, evaluate_entity);
                let operator = data.operator_token.kind;
                let is_syntactically_string = (left.is_syntactically_string
                    || right.is_syntactically_string)
                    && operator == SyntaxKind::PlusToken;
                let resolved_other_files = left.resolved_other_files || right.resolved_other_files;
                let has_external_references =
                    left.has_external_references || right.has_external_references;

                if let (Some(EvalValue::Number(left_num)), Some(EvalValue::Number(right_num))) =
                    (&left.value, &right.value)
                {
                    let result = match operator {
                        SyntaxKind::BarToken => {
                            Some(EvalValue::Number(left_num.bitwise_or(*right_num)))
                        }
                        SyntaxKind::AmpersandToken => {
                            Some(EvalValue::Number(left_num.bitwise_and(*right_num)))
                        }
                        SyntaxKind::GreaterThanGreaterThanToken => {
                            Some(EvalValue::Number(left_num.signed_right_shift(*right_num)))
                        }
                        SyntaxKind::GreaterThanGreaterThanGreaterThanToken => {
                            Some(EvalValue::Number(left_num.unsigned_right_shift(*right_num)))
                        }
                        SyntaxKind::LessThanLessThanToken => {
                            Some(EvalValue::Number(left_num.left_shift(*right_num)))
                        }
                        SyntaxKind::CaretToken => {
                            Some(EvalValue::Number(left_num.bitwise_xor(*right_num)))
                        }
                        SyntaxKind::AsteriskToken => {
                            Some(EvalValue::Number(*left_num * *right_num))
                        }
                        SyntaxKind::SlashToken => Some(EvalValue::Number(*left_num / *right_num)),
                        SyntaxKind::PlusToken => Some(EvalValue::Number(*left_num + *right_num)),
                        SyntaxKind::MinusToken => Some(EvalValue::Number(*left_num - *right_num)),
                        SyntaxKind::PercentToken => {
                            Some(EvalValue::Number(left_num.remainder(*right_num)))
                        }
                        SyntaxKind::AsteriskAsteriskToken => {
                            Some(EvalValue::Number(left_num.exponentiate(*right_num)))
                        }
                        _ => None,
                    };
                    if let Some(v) = result {
                        return EvalResult::new(
                            Some(v),
                            is_syntactically_string,
                            resolved_other_files,
                            has_external_references,
                        );
                    }
                }

                // String concatenation
                if operator == SyntaxKind::PlusToken {
                    let left_str = left.value.as_ref().map(|v| match v {
                        EvalValue::String(s) => Some(s.clone()),
                        EvalValue::Number(n) => Some(n.to_string()),
                        _ => None,
                    });
                    let right_str = right.value.as_ref().map(|v| match v {
                        EvalValue::String(s) => Some(s.clone()),
                        EvalValue::Number(n) => Some(n.to_string()),
                        _ => None,
                    });
                    if let (Some(Some(ls)), Some(Some(rs))) = (left_str, right_str) {
                        return EvalResult::new(
                            Some(EvalValue::String(format!("{}{}", ls, rs))),
                            is_syntactically_string,
                            resolved_other_files,
                            has_external_references,
                        );
                    }
                }

                return EvalResult::new(
                    None,
                    is_syntactically_string,
                    resolved_other_files,
                    has_external_references,
                );
            }
            EvalResult::none()
        }
        SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral => {
            let text = match &expr.data {
                NodeData::StringLiteral(d) => d.text.clone(),
                NodeData::NoSubstitutionTemplateLiteral(d) => d.text.clone(),
                _ => String::new(),
            };
            EvalResult::new(Some(EvalValue::String(text)), true, false, false)
        }
        SyntaxKind::NumericLiteral => {
            if let NodeData::NumericLiteral(data) = &expr.data {
                let num = Number::from_string(&data.text);
                EvalResult::new(Some(EvalValue::Number(num)), false, false, false)
            } else {
                EvalResult::none()
            }
        }
        SyntaxKind::BigIntLiteral => {
            if let NodeData::BigIntLiteral(data) = &expr.data {
                let big = PseudoBigInt::parse(&data.text);
                EvalResult::new(Some(EvalValue::BigInt(big)), false, false, false)
            } else {
                EvalResult::none()
            }
        }
        SyntaxKind::TrueKeyword => {
            EvalResult::new(Some(EvalValue::Bool(true)), false, false, false)
        }
        SyntaxKind::FalseKeyword => {
            EvalResult::new(Some(EvalValue::Bool(false)), false, false, false)
        }
        SyntaxKind::TemplateExpression => {
            if let NodeData::TemplateExpression(data) = &expr.data {
                evaluate_template_expression(expr, data, location, evaluate_entity)
            } else {
                EvalResult::none()
            }
        }
        SyntaxKind::Identifier => evaluate_entity(expr, location),
        SyntaxKind::PropertyAccessExpression | SyntaxKind::ElementAccessExpression => {
            // Check if the expression is an entity name expression
            let is_entity = match &expr.data {
                NodeData::PropertyAccessExpression(data) => {
                    is_entity_name_expression(&data.expression)
                }
                NodeData::ElementAccessExpression(data) => {
                    is_entity_name_expression(&data.expression)
                }
                _ => false,
            };
            if is_entity {
                evaluate_entity(expr, location)
            } else {
                EvalResult::none()
            }
        }
        _ => EvalResult::none(),
    }
}

fn evaluate_template_expression(
    _expr: &Arc<Node>,
    data: &TemplateExpressionData,
    location: Option<&Arc<Node>>,
    evaluate_entity: EvaluateEntity,
) -> EvalResult {
    let head_text = match &data.head.data {
        NodeData::TemplateHead(d) => d.text.clone(),
        _ => String::new(),
    };
    let mut sb = head_text;
    let mut resolved_other_files = false;
    let mut has_external_references = false;

    for span in data.template_spans.iter() {
        let span_data = match &span.data {
            NodeData::TemplateSpan(d) => d,
            _ => continue,
        };
        let span_result = evaluate_expression(&span_data.expression, location, evaluate_entity);
        match &span_result.value {
            None => {
                return EvalResult::new(None, true, false, false);
            }
            Some(v) => {
                sb.push_str(&v.to_string());
            }
        }
        // Append the literal part of the span
        match &span_data.literal.data {
            NodeData::TemplateMiddle(d) => sb.push_str(&d.text),
            NodeData::TemplateTail(d) => sb.push_str(&d.text),
            _ => {}
        }
        resolved_other_files |= span_result.resolved_other_files;
        has_external_references |= span_result.has_external_references;
    }

    EvalResult::new(
        Some(EvalValue::String(sb)),
        true,
        resolved_other_files,
        has_external_references,
    )
}

/// Whether a node is an entity name expression (identifier or property access of entity names).
fn is_entity_name_expression(node: &Arc<Node>) -> bool {
    match node.kind {
        SyntaxKind::Identifier => true,
        SyntaxKind::PropertyAccessExpression => {
            if let NodeData::PropertyAccessExpression(data) = &node.data {
                is_entity_name_expression(&data.expression)
            } else {
                false
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
