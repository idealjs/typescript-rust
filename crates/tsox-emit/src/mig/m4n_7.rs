#![allow(unused_imports)]
#![allow(dead_code)]

use std::sync::Arc;

use super::m4q::r33k12_defs::Set;
use super::m4q::r33k12_defs::{concatenate, splice};
use tsox_frontend::ast::node::Node;
use tsox_frontend::ast::is_prologue_directive;

use crate::printer::EmitContext;
use tsox_frontend::format::mig::m4o_2::EmitHelper;

impl EmitContext {
    pub fn merge_environment_inner(
        &mut self,
        statements: &[Arc<Node>],
        declarations: &[Arc<Node>],
    ) -> (Vec<Arc<Node>>, bool) { ::tsox_core::fntrace::enter("merge_environment_inner"); 
        if declarations.is_empty() {
            return (statements.to_vec(), false);
        }

        let mut changed = false;

        let left_standard_prologue_end =
            span_end(statements, |n| is_prologue_directive(n), 0);
        let left_hoisted_functions_end =
            span_end(statements, |n| self.is_hoisted_function(n), left_standard_prologue_end);
        let left_hoisted_variables_end = span_end(
            statements,
            |n| self.is_hoisted_variable_statement(n),
            left_hoisted_functions_end,
        );

        let right_standard_prologue_end =
            span_end(declarations, |n| is_prologue_directive(n), 0);
        let right_hoisted_functions_end =
            span_end(declarations, |n| self.is_hoisted_function(n), right_standard_prologue_end);
        let right_hoisted_variables_end = span_end(
            declarations,
            |n| self.is_hoisted_variable_statement(n),
            right_hoisted_functions_end,
        );
        let right_custom_prologue_end =
            span_end(declarations, |n| self.is_custom_prologue(n), right_hoisted_variables_end);
        if right_custom_prologue_end != declarations.len() {
            panic!("Expected declarations to be valid standard or custom prologues");
        }

        let mut left = statements.to_vec();

        if right_custom_prologue_end > right_hoisted_variables_end {
            left = splice(
                &left,
                left_hoisted_variables_end as isize,
                0,
                &declarations[right_hoisted_variables_end..right_custom_prologue_end],
            );
            changed = true;
        }

        if right_hoisted_variables_end > right_hoisted_functions_end {
            left = splice(
                &left,
                left_hoisted_functions_end as isize,
                0,
                &declarations[right_hoisted_functions_end..right_hoisted_variables_end],
            );
            changed = true;
        }

        if right_hoisted_functions_end > right_standard_prologue_end {
            left = splice(
                &left,
                left_standard_prologue_end as isize,
                0,
                &declarations[right_standard_prologue_end..right_hoisted_functions_end],
            );
            changed = true;
        }

        if right_standard_prologue_end > 0 {
            if left_standard_prologue_end == 0 {
                left = splice(&left, 0, 0, &declarations[..right_standard_prologue_end]);
                changed = true;
            } else {
                let mut left_prologues = Set::new();
                for i in 0..left_standard_prologue_end {
                    left_prologues.add(
                        statements[i]
                            .expression()
                            .map(|e| e.text().to_string())
                            .unwrap(),
                    );
                }
                for i in (0..right_standard_prologue_end).rev() {
                    let right_prologue = &declarations[i];
                    if !left_prologues.has(
                        &right_prologue
                            .expression()
                            .map(|e| e.text().to_string())
                            .unwrap(),
                    ) {
                        left = concatenate(&[Arc::clone(right_prologue)][..], &left);
                        changed = true;
                    }
                }
            }
        }

        (left, changed)
    }
}

pub fn append_if_unique_vec(
    helpers: &[&'static EmitHelper],
    helper: &'static EmitHelper,
) -> Vec<&'static EmitHelper> { ::tsox_core::fntrace::enter("append_if_unique_vec"); 
    let mut result = helpers.to_vec();
    if !result.iter().any(|h| std::ptr::eq(*h, helper)) {
        result.push(helper);
    }
    result
}

pub(crate) fn span_end(
    nodes: &[Arc<Node>],
    predicate: impl Fn(&Arc<Node>) -> bool,
    start: usize,
) -> usize { ::tsox_core::fntrace::enter("span_end"); 
    let mut i = start;
    while i < nodes.len() && predicate(&nodes[i]) {
        i += 1;
    }
    i
}
