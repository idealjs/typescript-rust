#![allow(unused_imports)]

use std::sync::Arc;

use crate::checker::services::*;
use tsox_frontend::ast::{CheckFlags, Symbol, SymbolFlags};

impl Checker {
    /// Go getCandidateForOverloadFailure：全重载不可适用且无泛型重载时给
    /// 联合签名（createUnionOfSignaturesForOverloadFailure）；含泛型重载
    /// 退最长候选
    pub(crate) fn candidate_for_overload_failure(
        &mut self,
        node: &Arc<Node>,
        signatures: &[Arc<Signature>],
        args: &Arc<tsox_frontend::ast::NodeList>,
    ) -> Option<Arc<Signature>> {
        if signatures.is_empty() {
            return None;
        }
        if signatures.len() > 1 && signatures.iter().all(|s| s.type_parameters.is_empty()) {
            return Some(self.create_union_of_signatures_for_overload_failure(signatures));
        }
        // Go pickLongestCandidateSignature：参数最多的候选
        let mut best = 0usize;
        let mut best_len = 0usize;
        for (idx, sig) in signatures.iter().enumerate() {
            let len = sig.parameters.len();
            if len > best_len {
                best = idx;
                best_len = len;
            }
        }
        let _ = (node, args);
        Some(Arc::clone(&signatures[best]))
    }

    /// 参数位 = 各重载对位类型的并集；rest 位 = 元素并集的数组；返回 =
    /// 各重载返回的交集
    fn create_union_of_signatures_for_overload_failure(
        &mut self,
        candidates: &[Arc<Signature>],
    ) -> Arc<Signature> {
        let non_rest_count = |s: &Arc<Signature>| {
            if s.has_rest_parameter() {
                s.parameters.len().saturating_sub(1)
            } else {
                s.parameters.len()
            }
        };
        let max_non_rest = candidates.iter().map(non_rest_count).max().unwrap_or(0);

        let mut parameters: Vec<Arc<Symbol>> = Vec::with_capacity(max_non_rest);
        for i in 0..max_non_rest {
            let mut types: Vec<Arc<Type>> = Vec::new();
            for sig in candidates {
                if let Some(t) = self.overload_failure_type_at_position(sig, i) {
                    types.push(t);
                }
            }
            let union = self.get_union_type(types);
            parameters.push(self.synthetic_parameter_symbol_for_overload_failure(i, union));
        }

        let mut flags =
            SignatureFlags::IsSignatureCandidateForOverloadFailure;
        let rest_symbols: Vec<&Arc<Symbol>> = candidates
            .iter()
            .filter(|s| s.has_rest_parameter())
            .map(|s| s.parameters.last().unwrap())
            .collect();
        if !rest_symbols.is_empty() {
            let mut element_types: Vec<Arc<Type>> = Vec::new();
            for sig in candidates {
                if sig.has_rest_parameter()
                    && let Some(elem) = self.effective_rest_element_type(sig)
                {
                    element_types.push(elem);
                }
            }
            let element_union = self.get_union_type(element_types);
            let array = self.create_array_type(element_union);
            let rest = self.synthetic_rest_symbol_for_overload_failure(&rest_symbols, array);
            parameters.push(rest);
            flags |= SignatureFlags::HasRestParameter;
        }

        if candidates.iter().any(|s| s.flags.contains(SignatureFlags::HasLiteralTypes)) {
            flags |= SignatureFlags::HasLiteralTypes;
        }

        let mut returns: Vec<Arc<Type>> = Vec::new();
        for sig in candidates {
            if let Some(rt) = self.get_return_type_of_signature(sig) {
                returns.push(rt);
            }
        }
        let intersection = self.get_intersection_type(returns);

        let min_argument_count = candidates
            .iter()
            .map(|s| s.min_argument_count)
            .min()
            .unwrap_or(0);

        let mut combined = Signature::new();
        combined.flags = flags;
        combined.declaration = candidates.first().and_then(|s| s.declaration.clone());
        combined.parameters = parameters;
        combined.min_argument_count = min_argument_count;
        let _ = combined.resolved_return_type.set(intersection);
        Arc::new(combined)
    }

    /// Go tryGetTypeAtPosition：rest 位给元素类型（...B[] 位上的联合
    /// 成员是 B）
    fn overload_failure_type_at_position(
        &mut self,
        sig: &Arc<Signature>,
        pos: usize,
    ) -> Option<Arc<Type>> {
        let non_rest = if sig.has_rest_parameter() {
            sig.parameters.len().saturating_sub(1)
        } else {
            sig.parameters.len()
        };
        if pos < non_rest {
            return Some(self.get_type_of_symbol(&sig.parameters[pos]));
        }
        if sig.has_rest_parameter() {
            let rest = self.get_type_of_symbol(sig.parameters.last()?);
            return Some(self.get_array_element_type_of(&rest).unwrap_or(rest));
        }
        None
    }

    fn effective_rest_element_type(&mut self, sig: &Arc<Signature>) -> Option<Arc<Type>> {
        let rest_param = sig.parameters.last()?;
        let rest_type = self.get_type_of_symbol(rest_param);
        self.get_array_element_type_of(&rest_type)
    }

    fn synthetic_parameter_symbol_for_overload_failure(
        &mut self,
        index: usize,
        t: Arc<Type>,
    ) -> Arc<Symbol> {
        let name = format!("arg{index}");
        let mut symbol = Symbol::new(SymbolFlags::Property, name);
        symbol.check_flags |= CheckFlags::SyntheticProperty;
        let arc = Arc::new(symbol);
        self.value_symbol_links.insert(
            &arc,
            crate::checker::types::ValueSymbolLinks {
                resolved_type: Some(t),
                ..Default::default()
            },
        );
        arc
    }

    fn synthetic_rest_symbol_for_overload_failure(
        &mut self,
        sources: &[&Arc<Symbol>],
        t: Arc<Type>,
    ) -> Arc<Symbol> {
        let mut symbol = Symbol::new(SymbolFlags::Property, "rest".to_string());
        symbol.check_flags |= CheckFlags::SyntheticProperty;
        for s in sources {
            for d in &s.declarations {
                if !symbol.declarations.iter().any(|x| Arc::ptr_eq(x, d)) {
                    symbol.declarations.push(Arc::clone(d));
                }
            }
        }
        let arc = Arc::new(symbol);
        self.value_symbol_links.insert(
            &arc,
            crate::checker::types::ValueSymbolLinks {
                resolved_type: Some(t),
                ..Default::default()
            },
        );
        arc
    }
}

impl Checker {
    /// Go getUnionSignatures 兜底分支（checker.go:21465 起）：各成分单签名、
    /// 无泛型、无有效 rest（定长 tuple rest 视作位置参数）时合并为单一
    /// 签名，参数位取交集（combineUnionOrIntersectionParameters，
    /// union 方向）、min 取 max、返回取并集。有效 rest（数组/非元组/
    /// 变长 tuple）与泛型签名维持逐成员拼接，避免无 composite 分布语义
    /// 的真交集误伤上下文敏感推断
    pub(crate) fn try_combine_union_call_signatures(
        &mut self,
        sigs: &[Arc<Signature>],
    ) -> Option<Arc<Signature>> {
        if sigs.len() < 2 {
            return None;
        }
        for s in sigs {
            if !s.type_parameters.is_empty() {
                return None;
            }
            if s.has_rest_parameter() {
                let rest_param = s.parameters.last()?;
                let rest_type = self.get_type_of_symbol(rest_param);
                let TypeData::Tuple(t) = &rest_type.data else {
                    return None;
                };
                if t.combined_flags.intersects(
                    ElementFlags::Variadic | ElementFlags::Rest,
                ) {
                    return None;
                }
            }
        }
        let mut positional_counts: Vec<usize> = Vec::with_capacity(sigs.len());
        for s in sigs {
            let mut count = s.parameters.len();
            if s.has_rest_parameter() {
                count -= 1;
                let rest_param = s.parameters.last().unwrap();
                let rest_type = self.get_type_of_symbol(rest_param);
                if let TypeData::Tuple(t) = &rest_type.data {
                    count += t.fixed_length;
                }
            }
            positional_counts.push(count);
        }
        let max_count = positional_counts.iter().copied().max().unwrap_or(0);
        let max_min = sigs.iter().map(|s| s.min_argument_count).max().unwrap_or(0);

        let mut parameters: Vec<Arc<Symbol>> = Vec::with_capacity(max_count);
        for i in 0..max_count {
            let mut types: Vec<Arc<Type>> = Vec::new();
            for sig in sigs {
                if let Some(t) = self.try_get_type_at_position(sig, i) {
                    types.push(t);
                }
            }
            let combined_type = if types.is_empty() {
                self.any_type()
            } else {
                self.get_intersection_type(types)
            };
            let optional = sigs
                .iter()
                .all(|s| i >= s.min_argument_count.max(0) as usize);
            let mut symbol = Symbol::new(
                SymbolFlags::Property
                    | if optional {
                        SymbolFlags::Optional
                    } else {
                        SymbolFlags::empty()
                    },
                format!("arg{i}"),
            );
            symbol.check_flags |= CheckFlags::SyntheticProperty;
            let arc = Arc::new(symbol);
            self.value_symbol_links.insert(
                &arc,
                crate::checker::types::ValueSymbolLinks {
                    resolved_type: Some(combined_type),
                    ..Default::default()
                },
            );
            parameters.push(arc);
        }

        let mut returns: Vec<Arc<Type>> = Vec::new();
        for sig in sigs {
            if let Some(rt) = self.get_return_type_of_signature(sig) {
                returns.push(rt);
            }
        }
        let return_type = if returns.is_empty() {
            None
        } else {
            Some(self.get_union_type(returns))
        };

        let mut combined = Signature::new();
        combined.flags = SignatureFlags::IsSignatureCandidateForOverloadFailure;
        combined.declaration = sigs.first().and_then(|s| s.declaration.clone());
        combined.parameters = parameters;
        combined.min_argument_count = max_min;
        if let Some(rt) = return_type {
            let _ = combined.resolved_return_type.set(rt);
        }
        Some(Arc::new(combined))
    }
}
