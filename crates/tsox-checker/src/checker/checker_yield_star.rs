#![allow(unused_imports)]

use crate::checker::checker::*;

impl Checker {
    /// Go getIterationTypeOfIterable(yieldStar, Return)：yield* 表达式的类型 =
    /// 操作数迭代器的 TReturn（next() 返回 IteratorResult 中 done 含 true 变体的 value）
    pub(crate) fn get_yield_star_return_type(
        &mut self,
        operand_type: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        if operand_type.flags.contains(TypeFlags::Any) {
            return None;
        }
        if operand_type.is_union() {
            let parts: Vec<Arc<Type>> = operand_type
                .types()?
                .iter()
                .filter_map(|c| self.get_yield_star_return_type(c))
                .collect();
            if parts.is_empty() {
                return None;
            }
            return Some(self.get_union_type(parts));
        }
        let iter_method = self.get_property_of_type(operand_type, "__@iterator")?;
        let iter_method_type = self.get_type_of_symbol(&iter_method);
        if iter_method_type.flags.contains(TypeFlags::Any) {
            return None;
        }
        let mut iterator_types: Vec<Arc<Type>> = Vec::new();
        for sig in self.get_signatures_of_type(&iter_method_type, SignatureKind::Call) {
            if self.get_min_argument_count(&sig) == 0
                && let Some(rt) = self.get_return_type_of_signature(&sig)
            {
                iterator_types.push(rt);
            }
        }
        if iterator_types.is_empty() {
            return None;
        }
        let mut returns: Vec<Arc<Type>> = Vec::new();
        for it in &iterator_types {
            let Some(next) = self.get_property_of_type(it, "next") else {
                continue;
            };
            let next_type = self.get_type_of_symbol(&next);
            if next_type.flags.contains(TypeFlags::Any) {
                continue;
            }
            for sig in self.get_signatures_of_type(&next_type, SignatureKind::Call) {
                if let Some(rt) = self.get_return_type_of_signature(&sig)
                    && let Some(value) = self.iterator_result_return_value(&rt)
                {
                    returns.push(value);
                }
            }
        }
        if returns.is_empty() {
            None
        } else {
            Some(self.get_union_type(returns))
        }
    }

    /// Go getIterationTypesOfIteratorResult：IteratorResult 联合中取
    /// done 含 true 变体（IteratorReturnResult）的 value
    fn iterator_result_return_value(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if t.is_union() {
            let parts: Vec<Arc<Type>> = t
                .types()?
                .iter()
                .filter_map(|c| self.iterator_result_return_value(c))
                .collect();
            if parts.is_empty() {
                return None;
            }
            return Some(self.get_union_type(parts));
        }
        // Go isIteratorResult(Return)：true 可赋给 done 才是返回变体
        //（done?: false 的 yield 变体不可命中）
        let is_return = match self.get_property_of_type(t, "done") {
            Some(done) => {
                let dt = self.get_type_of_symbol(&done);
                let true_t = self.true_type();
                self.is_type_assignable_to(&true_t, &dt)
            }
            // done 缺省视为 false（yield 变体）
            None => false,
        };
        if !is_return {
            return None;
        }
        let value = self.get_property_of_type(t, "value")?;
        Some(self.get_type_of_symbol(&value))
    }
}
