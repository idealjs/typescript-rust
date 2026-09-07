use crate::ast::Node;
use crate::jsnum::{Number, PseudoBigInt};
use std::sync::Arc;

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

#[derive(Clone, Debug, PartialEq)]
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

pub type EvaluateEntity = fn(&Arc<Node>, Option<&Arc<Node>>) -> EvalResult;
