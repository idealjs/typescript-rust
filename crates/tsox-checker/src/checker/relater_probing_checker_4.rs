#![allow(unused_imports)]

use crate::checker::relater_probing::*;

impl Checker {
    /// 推断替换的参数匹配：指针恒等，或同为类型参数且符号等价
    ///（实例化语境下 shell 内的参数实例与签名参数列表非同一 Arc）
    pub(crate) fn infer_param_matches(&self, p: &Arc<Type>, t: &Arc<Type>) -> bool {
        if Arc::ptr_eq(p, t) {
            return true;
        }
        p.is_type_parameter()
            && t.is_type_parameter()
            && p.symbol
                .as_ref()
                .zip(t.symbol.as_ref())
                .is_some_and(|(ps, ts)| {
                    Arc::ptr_eq(ps, ts)
                        || (ps.name == ts.name && self.type_param_symbols_equivalent(ps, ts))
                })
    }

    pub fn substitute_infer_type_parameters(
        &mut self,
        t: &Arc<Type>,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> {
        if params.is_empty() || substitutions.is_empty() {
            return Arc::clone(t);
        }
        let t_ptr = Arc::as_ptr(t) as usize;
        if self.infer_subst_ancestor_stack.len() > 1000
            && self.infer_subst_ancestor_stack.contains(&t_ptr)
        {
            return Arc::clone(t);
        }
        self.infer_subst_ancestor_stack.push(t_ptr);
        let result = self.substitute_infer_type_parameters_inner(t, params, substitutions);
        self.infer_subst_ancestor_stack.pop();
        result
    }

    fn substitute_infer_type_parameters_inner(
        &mut self,
        t: &Arc<Type>,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> {

        for (i, p) in params.iter().enumerate() {
            if self.infer_param_matches(p, t) {
                return Arc::clone(&substitutions[i.min(substitutions.len() - 1)]);
            }
        }

        // Go instantiateTypeWorker(checker.go:22562)：别名实参重实例化只适用于
        // 惰性壳（Object，类型变量仅存于 alias 实参）；Conditional/Union/
        // IndexedAccess 等结构内直接持有类型变量，走下方结构替换分支
        //（Conditional 经 rebuild 携带代入后的 alias）
        if let TypeData::Object(_) = &t.data
            && let Some(alias) = t.alias.as_ref()
            && let Some(alias_sym) = alias.symbol.clone()
            && alias_sym
                .flags
                .intersects(tsox_frontend::ast::SymbolFlags::TypeAlias)
            && !alias.type_arguments.is_empty()
        {
            let (tp_symbols, _) = self.collect_alias_type_params_and_body(&alias_sym);
            let self_form = alias.type_arguments.iter().all(|a| {
                a.is_type_parameter()
                    && a.symbol.as_ref().is_some_and(|s| {
                        tp_symbols.iter().any(|tp| Arc::ptr_eq(tp, s))
                    })
            });
            if !self_form {
                let sym_key = Arc::as_ptr(&alias_sym) as *const tsox_frontend::ast::Symbol as usize;
                let same_symbol_depth = self
                    .alias_type_instantiation_stack
                    .iter()
                    .filter(|(k, _)| *k == sym_key)
                    .count();
                if same_symbol_depth >= crate::checker::checker::ALIAS_SELF_INSTANTIATION_DEPTH {
                    return Arc::clone(t);
                }
                let new_args: Vec<Arc<Type>> = alias
                    .type_arguments
                    .iter()
                    .map(|a| {
                        self.substitute_infer_type_parameters(a, params, substitutions)
                    })
                    .collect();
                let changed = alias
                    .type_arguments
                    .iter()
                    .zip(new_args.iter())
                    .any(|(old, new)| !Arc::ptr_eq(old, new));
                if changed {
                    return self.instantiate_alias_from_types(&alias_sym, new_args);
                }
            }
        }

        match &t.data {
            TypeData::Substitution(sub) => {
                let new_base = sub
                    .base_type
                    .as_ref()
                    .map(|b| self.substitute_infer_type_parameters(b, params, substitutions));
                let changed = new_base
                    .as_ref()
                    .zip(sub.base_type.as_ref())
                    .is_some_and(|(n, o)| !Arc::ptr_eq(n, o));
                if !changed {
                    return Arc::clone(t);
                }
                let mut rebuilt = Type::new(
                    t.flags,
                    TypeData::Substitution(SubstitutionTypeData {
                        constrained: ConstrainedTypeData::default(),
                        base_type: new_base.or_else(|| sub.base_type.clone()),
                        constraint: sub.constraint.clone(),
                    }),
                );
                rebuilt.object_flags = t.object_flags;
                rebuilt.symbol = t.symbol.clone();
                Arc::new(rebuilt)
            }
            TypeData::Union(u) => {
                let new_types: Vec<Arc<Type>> = u
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|inner| {
                        self.substitute_infer_type_parameters(inner, params, substitutions)
                    })
                    .collect();
                self.get_union_type(new_types)
            }
            TypeData::Intersection(i) => {
                let new_types: Vec<Arc<Type>> = i
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|inner| {
                        self.substitute_infer_type_parameters(inner, params, substitutions)
                    })
                    .collect();
                self.get_intersection_type(new_types)
            }
            TypeData::Object(o) => self.substitute_infer_object(t, o, params, substitutions),
            TypeData::Tuple(tup) => self.substitute_infer_tuple(t, tup, params, substitutions),
            TypeData::IndexedAccess(ia) => {
                self.substitute_infer_indexed_access(t, ia, params, substitutions)
            }
            TypeData::Conditional(ct) => {
                self.substitute_infer_conditional(t, ct, params, substitutions)
            }
            TypeData::Index(idx) => self.substitute_infer_index(t, idx, params, substitutions),
            TypeData::Mapped(m) => self.substitute_infer_mapped(t, m, params, substitutions),
            _ => Arc::clone(t),
        }
    }
}
