mod eval;
mod value;

pub use eval::evaluate_expression;
pub use value::{EvalResult, EvalValue, EvaluateEntity};

#[cfg(test)]
mod tests;
