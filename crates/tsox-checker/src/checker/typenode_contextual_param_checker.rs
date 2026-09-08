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
        // 无注解 this 参数：取上下文签名 this 参数类型（Go getTypeForVariableLikeDeclaration 的 InternalSymbolNameThis 分支）
        if is_this_param {
            if let Some(ctx_this) = ctx_sig.this_parameter.clone() {
                return Some(self.get_type_of_symbol(&ctx_this));
            }
            return Some(self.get_void_type());
        }
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
            let fixed = n.saturating_sub(usize::from(ctx_rest));
            // Go getRestTypeAtPosition：rest 源参数收集上下文签名中从 si 起的剩余参数；
            // 上下文 rest 参数落在收集范围内时以 labeled rest 元素（...name: T）收尾
            let mut elems: Vec<Arc<Type>> = Vec::new();
            let mut names: Vec<String> = Vec::new();
            let mut rest_elem: Option<Arc<Type>> = None;
            if ctx_rest && si >= fixed {
                let rt = self
                    .signature_instantiated_param_type(ctx_sig, n - 1)
                    .unwrap_or_else(|| self.get_type_of_symbol(&ctx_params[n - 1]));
                return Some(rt);
            }
            for j in si..n {
                if ctx_rest && j == fixed {
                    let rt = self
                        .signature_instantiated_param_type(ctx_sig, n - 1)
                        .unwrap_or_else(|| {
                            self.get_type_of_symbol(&ctx_params[n - 1])
                        });
                    rest_elem = Some(rt);
                    break;
                }
                elems.push(
                    self.signature_instantiated_param_type(ctx_sig, j)
                        .unwrap_or_else(|| self.get_type_of_symbol(&ctx_params[j])),
                );
                names.push(ctx_params[j].name.clone());
            }
            if let Some(rt) = rest_elem {
                if elems.is_empty() {
                    return Some(rt);
                }
                let label = ctx_params[fixed].name.clone();
                let mut all_elems = elems;
                all_elems.push(rt);
                let mut all_names = names;
                all_names.push(label);
                let tuple = self.create_tuple_type_named(all_elems, all_names);
                // 末元素标记 rest 形态（...a2: string[]），借 combined flags 表达
                if let crate::checker::types::TypeData::Tuple(tup) = &tuple.data {
                    let tuple = Arc::new(crate::checker::types::Type {
                        flags: tuple.flags,
                        object_flags: crate::checker::types::ObjectFlags::Tuple,
                        id: crate::checker::types::next_type_id(),
                        symbol: None,
                        alias: None,
                        data: crate::checker::types::TypeData::Tuple(
                            crate::checker::types::TupleTypeData {
                                element_infos: tup
                                    .element_infos
                                    .iter()
                                    .enumerate()
                                    .map(|(i, ei)| crate::checker::types::TupleElementInfo {
                                        label: ei.label.clone(),
                                        flags: if i + 1 == tup.element_infos.len() {
                                            crate::checker::types::ElementFlags::Variadic
                                        } else {
                                            ei.flags
                                        },
                                        type_: ei.type_.clone(),
                                        labeled_declaration: ei.labeled_declaration.clone(),
                                    })
                                    .collect(),
                                min_length: tup.min_length.saturating_sub(1),
                                fixed_length: tup.fixed_length.saturating_sub(1),
                                combined_flags: tup.combined_flags,
                                readonly: tup.readonly,
                                interface_data: Default::default(),
                            },
                        ),
                    });
                    return Some(tuple);
                }
                return Some(tuple);
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
            let fixed = n.saturating_sub(usize::from(ctx_rest));
            if si < fixed {
                return Some(
                    self.signature_instantiated_param_type(ctx_sig, si)
                        .unwrap_or_else(|| self.get_type_of_symbol(&ctx_params[si])),
                );
            }
            // Go getRestTypeAtPosition：非 rest 参数越过固定参数落在 rest 区间，取 rest 元素类型
            if ctx_rest && n > 0 {
                let rest_sym = ctx_params.last().expect("ctx_rest implies n>0");
                let rt = self
                    .signature_instantiated_param_type(ctx_sig, n - 1)
                    .unwrap_or_else(|| self.get_type_of_symbol(rest_sym));
                return Some(self.get_array_element_type(&rt));
            }
            None
        }
    }
}
