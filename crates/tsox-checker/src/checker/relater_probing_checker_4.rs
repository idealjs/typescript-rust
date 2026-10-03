#![allow(unused_imports)]

use crate::checker::relater_probing::*;

pub(crate) struct InferSubstMemoFrame {
    mapping: Vec<u32>,
    results: HashMap<u32, Arc<Type>>,
}

fn infer_mapping_key(params: &[Arc<Type>], substitutions: &[Arc<Type>]) -> Vec<u32> { ::tsox_core::fntrace::enter("infer_mapping_key"); 
    let mut key = Vec::with_capacity(params.len() + substitutions.len());
    key.extend(params.iter().map(|p| p.id));
    key.extend(substitutions.iter().map(|s| s.id));
    key
}

impl Checker {
    /// 推断替换的参数匹配：指针恒等，或同为类型参数且符号等价
    ///（实例化语境下 shell 内的参数实例与签名参数列表非同一 Arc）
    pub(crate) fn infer_param_matches(&self, p: &Arc<Type>, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("infer_param_matches"); 
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
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("substitute_infer_type_parameters"); 
        if params.is_empty() || substitutions.is_empty() {
            return Arc::clone(t);
        }
        let t_ptr = Arc::as_ptr(t) as usize;
        if self.infer_subst_ancestor_stack.len() > 1000
            && self.infer_subst_ancestor_stack.contains(&t_ptr)
        {
            return Arc::clone(t);
        }
        let mapping = infer_mapping_key(params, substitutions);
        let reuse = self
            .infer_subst_memo
            .last()
            .is_some_and(|f| f.mapping == mapping);
        if reuse
            && let Some(result) = self
                .infer_subst_memo
                .last()
                .and_then(|f| f.results.get(&t.id))
        {
            return Arc::clone(result);
        }
        if !reuse {
            self.infer_subst_memo.push(InferSubstMemoFrame {
                mapping: mapping.clone(),
                results: HashMap::new(),
            });
        }
        self.infer_subst_ancestor_stack.push(t_ptr);
        let result = self.substitute_infer_type_parameters_inner(t, params, substitutions);
        self.infer_subst_ancestor_stack.pop();
        if self
            .infer_subst_memo
            .last()
            .is_some_and(|f| f.mapping == mapping)
        {
            self.infer_subst_memo
                .last_mut()
                .expect("length checked above")
                .results
                .entry(t.id)
                .or_insert_with(|| Arc::clone(&result));
        }
        if !reuse {
            self.infer_subst_memo.pop();
        }
        result
    }

    /// Go instantiateAnonymousType（checker.go:22800）：alias 为空时以
    /// instantiateTypeAlias(t.alias, m) 补齐，实参经同一替换更新
    pub(crate) fn attach_substituted_alias(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) { ::tsox_core::fntrace::enter("attach_substituted_alias"); 
        let Some(alias) = source.alias.as_ref() else {
            return;
        };
        let new_args: Vec<Arc<Type>> = alias
            .type_arguments
            .iter()
            .map(|a| self.substitute_infer_type_parameters(a, params, substitutions))
            .collect();
        let ptr = Arc::as_ptr(target) as *mut crate::checker::types::Type;
        unsafe {
            if (*ptr).alias.is_none() {
                (*ptr).alias = Some(Box::new(crate::checker::types::TypeAlias::new(
                    alias.symbol.clone(),
                    new_args,
                )));
            }
        }
    }

    fn substitute_infer_type_parameters_inner(
        &mut self,
        t: &Arc<Type>,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("substitute_infer_type_parameters_inner"); 

        for (i, p) in params.iter().enumerate() {
            if if self.erase_signature_strict {
                Arc::ptr_eq(p, t) || p.id == t.id
            } else {
                self.infer_param_matches(p, t)
            } {
                // Go newTypeMapper：参数/实参按下标配对，缺位（实参短于参数）
                // 映射为恒等，不得回退末位实参（否则 P 会错代成 T 的实参）
                return if i < substitutions.len() {
                    Arc::clone(&substitutions[i])
                } else {
                    Arc::clone(t)
                };
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
                let result = self.get_union_type(new_types);
                self.attach_substituted_alias(t, &result, params, substitutions);
                result
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
                let result = self.get_intersection_type(new_types);
                self.attach_substituted_alias(t, &result, params, substitutions);
                result
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
            TypeData::TemplateLiteral(tl) => {
                self.substitute_infer_template_literal(t, tl, params, substitutions)
            }
            _ => Arc::clone(t),
        }
    }

    fn substitute_infer_template_literal(
        &mut self,
        t: &Arc<Type>,
        tl: &TemplateLiteralTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("substitute_infer_template_literal"); 
        let new_types: Vec<Arc<Type>> = tl
            .types
            .iter()
            .map(|inner| self.substitute_infer_type_parameters(inner, params, substitutions))
            .collect();
        let changed = tl
            .types
            .iter()
            .zip(new_types.iter())
            .any(|(old, new)| !Arc::ptr_eq(old, new));
        if !changed {
            return Arc::clone(t);
        }
        Arc::new(Type::new(
            TypeFlags::TemplateLiteral,
            TypeData::TemplateLiteral(TemplateLiteralTypeData {
                constrained: ConstrainedTypeData::default(),
                texts: tl.texts.clone(),
                types: new_types,
            }),
        ))
    }
}
