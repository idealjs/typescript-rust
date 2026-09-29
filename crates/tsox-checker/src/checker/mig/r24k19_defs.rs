use crate::checker::inference_inference_key_2::InferenceContext;
use crate::checker::types::Type;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use tsox_frontend::ast::Node;

pub struct IntraExpressionInferenceSite {
    pub node: Arc<Node>,
    pub t: Arc<Type>,
}

thread_local! {
    static INTRA_EXPRESSION_INFERENCE_SITES: RefCell<HashMap<usize, Vec<IntraExpressionInferenceSite>>> =
        RefCell::new(HashMap::new());
}

fn context_key(n: &InferenceContext) -> usize {
    n as *const InferenceContext as usize
}

pub fn add_intra_expression_inference_site_to(n: &InferenceContext, node: Arc<Node>, t: Arc<Type>) {
    let key = context_key(n);
    INTRA_EXPRESSION_INFERENCE_SITES.with(|sites| {
        sites.borrow_mut().entry(key).or_default().push(IntraExpressionInferenceSite { node, t });
    });
}

pub fn take_intra_expression_inference_sites(n: &InferenceContext) -> Vec<IntraExpressionInferenceSite> {
    let key = context_key(n);
    INTRA_EXPRESSION_INFERENCE_SITES.with(|sites| sites.borrow_mut().remove(&key).unwrap_or_default())
}
