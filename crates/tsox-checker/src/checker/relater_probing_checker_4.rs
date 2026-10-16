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
        // Go instantiateTypeWithAlias 深度护栏（checker.go:23414）同款：Go 全部
        // 实例化经 instantiateTypeWithAlias 计数（depth==100 || count>=5M 报
        // TS2589 返 errorType），本仓并行 substitute 引擎此前在计数器外——
        // 自引用条件型（__Awaited 类）经 substitute→resolve→分支节点解析路
        // 每层重建产物为新指针，指针判环永不命中且两计数器均不增长，递归
        // 无界栈溢出（recursiveConditionalTypes/conditionalTypeDoesntSpinForever
        // worker signal 6）。本引擎并入同一计数器，超 Go 同款 100 层即恒等
        // 返回落 errorType 语义（不发新诊断，Go 同位 TS2589 处本侧落未代入
        // 形态）；正常推断深度几十层以内不受影响
        if self.instantiation_depth >= 100 || self.instantiation_count >= 5_000_000 {
            return Arc::clone(t);
        }
        if self.infer_subst_ancestor_stack.len() >= 100 {
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
        self.instantiation_depth += 1;
        self.instantiation_count += 1;
        let result = self.substitute_infer_type_parameters_inner(t, params, substitutions);
        self.instantiation_depth -= 1;
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

    /// Go instantiateTypeWorker union/intersection 臂的别名派生
    ///（checker.go:23576-23577）与 instantiateTypeAlias
    ///（checker.go:24091-24097）：源型自带 alias 时以替换后的实参重建
    /// TypeAlias（无条件重建，不判恒同），供联合/交集缓存键区分别名实例
    pub(crate) fn substituted_source_alias(
        &mut self,
        t: &Arc<Type>,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Option<crate::checker::types::TypeAlias> { ::tsox_core::fntrace::enter("substituted_source_alias");
        let alias = t.alias.as_ref()?;
        let new_args: Vec<Arc<Type>> = alias
            .type_arguments
            .iter()
            .map(|arg| self.substitute_infer_type_parameters(arg, params, substitutions))
            .collect();
        Some(crate::checker::types::TypeAlias::new(
            alias.symbol.clone(),
            new_args,
        ))
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
                let alias_inst = self.substituted_source_alias(t, params, substitutions);
                let result = self.get_union_type_ex(
                    new_types,
                    crate::checker::types::UnionReduction::Literal,
                    alias_inst.as_ref(),
                    None,
                );
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
                let alias_inst = self.substituted_source_alias(t, params, substitutions);
                let result = self.get_intersection_type_ex(
                    &new_types,
                    crate::checker::types::IntersectionFlags::None,
                    alias_inst.as_ref(),
                );
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
