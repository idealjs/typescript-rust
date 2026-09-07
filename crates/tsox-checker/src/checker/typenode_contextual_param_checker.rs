#![allow(unused_imports)]

use crate::checker::typenode_contextual_param::*;

impl Checker {
    /// checker.go getContextuallyTypedParameterType 的参数位定型：
    /// index = 参数位 - 源 this 数；rest 参数走 getRestTypeAtPosition 语义
    pub(crate) fn contextual_param_type_at(
        &mut self,
        ctx_sig: &Arc<Signature>,
        parameters: &Arc<NodeList>,
        i: usize,
        param: &Arc<Node>,
        is_rest: bool,
        is_this_param: bool,
    ) -> Option<Arc<Type>> {
        let src_has_this = parameters.iter().next().is_some_and(|first| {
            matches!(&first.data, NodeData::ParameterDeclaration(fd)
                if matches!(&fd.name.data, NodeData::Identifier(id) if id.text == "this"))
        });
        let si = i - usize::from(!is_this_param && src_has_this && i > 0);
        let is_last = parameters
            .iter()
            .next_back()
            .is_some_and(|p| Arc::ptr_eq(p, param));
        if is_rest && is_last {
            let ctx_params = &ctx_sig.parameters;
            let n = ctx_params.len();
            let ctx_rest = n > 0
                && ctx_params.last().is_some_and(|p| {
                    p.declarations.iter().any(|d| {
                        matches!(&d.data, NodeData::ParameterDeclaration(pd)
                                if pd.dot_dot_dot_token.is_some())
                    })
                });
            let fixed = n - usize::from(ctx_rest);
            if ctx_rest && si >= fixed.saturating_sub(1) {
                return self
                    .signature_instantiated_param_type(ctx_sig, fixed - 1)
                    .or_else(|| Some(self.get_type_of_symbol(&ctx_params[fixed - 1])));
            }
            let mut elems: Vec<Arc<Type>> = Vec::new();
            let mut names: Vec<String> = Vec::new();
            for j in si..fixed {
                elems.push(
                    self.signature_instantiated_param_type(ctx_sig, j)
                        .unwrap_or_else(|| self.get_type_of_symbol(&ctx_params[j])),
                );
                names.push(ctx_params[j].name.clone());
            }
            Some(self.create_tuple_type_named(elems, names))
        } else {
            let ctx_params = &ctx_sig.parameters;
            let n = ctx_params.len();
            let ctx_rest = n > 0
                && ctx_params.last().is_some_and(|p| {
                    p.declarations.iter().any(|d| {
                        matches!(&d.data, NodeData::ParameterDeclaration(pd)
                                if pd.dot_dot_dot_token.is_some())
                    })
                });
            let fixed = n - usize::from(ctx_rest);
            if si < fixed {
                return Some(
                    self.signature_instantiated_param_type(ctx_sig, si)
                        .unwrap_or_else(|| self.get_type_of_symbol(&ctx_params[si])),
                );
            }
            None
        }
    }
}
