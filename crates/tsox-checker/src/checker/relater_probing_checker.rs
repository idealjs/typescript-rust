#![allow(unused_imports)]

use crate::checker::relater_probing::*;

impl Checker {
    pub fn resolve_conditional_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("resolve_conditional_type"); 
        let ct = match &t.data {
            TypeData::Conditional(ct) => ct,
            _ => return None,
        };

        let check_type = ct.check_type.clone()?;
        let _extends_type = ct.extends_type.clone()?;

        if check_type.flags.contains(TypeFlags::Any) && check_type.intrinsic_name() == Some("error")
        {
            return Some(Arc::clone(&check_type));
        }

        let (is_distributive, check_tp_symbol) = match ct.root.as_ref() {
            Some(root) => (
                root.is_distributive,
                root.check_type_parameter_symbol.clone(),
            ),
            None => (false, None),
        };
        if is_distributive && let Some(tp_symbol) = &check_tp_symbol {
            if check_type.flags.contains(TypeFlags::Never) {
                let never = self.never_type();
                if let TypeData::Conditional(ct2) = &t.data {
                    let _ = ct2.resolved_true_type.set(Arc::clone(&never));
                }
                return Some(never);
            }
            if let TypeData::Union(u) = &check_type.data {
                let constituents = u.union_or_intersection.types.clone();
                let key = Arc::as_ptr(tp_symbol) as *const tsox_frontend::ast::Symbol;
                let mut results: Vec<Arc<Type>> = Vec::with_capacity(constituents.len());
                for constituent in constituents {
                    let mut mapping = std::collections::HashMap::new();
                    mapping.insert(key, Arc::clone(&constituent));
                    self.type_argument_stack.push(mapping);
                    let r =
                        self.resolve_conditional_type_with_check(t, Some(Arc::clone(&constituent)), None);
                    self.type_argument_stack.pop();
                    results.push(r?);
                }
                let union = self.get_union_type(results);
                return Some(union);
            }
        }

        self.resolve_conditional_type_with_check(t, None, None)
    }

    pub(crate) fn resolve_conditional_type_with_check(
        &mut self,
        t: &Arc<Type>,
        check_override: Option<Arc<Type>>,
        extends_override: Option<Arc<Type>>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("resolve_conditional_type_with_check"); 
        let ct = match &t.data {
            TypeData::Conditional(ct) => ct,
            _ => return None,
        };

        let check_type = match check_override {
            Some(ref c) => Arc::clone(c),
            None => ct.check_type.clone()?,
        };
        let cond_node = ct.root.as_ref().and_then(|r| r.node.clone());
        let extends_type = if let Some(e) = extends_override {
            e
        } else {
            let stored = ct.extends_type.clone()?;
            match &check_override {
                Some(over) if !Arc::ptr_eq(&stored, over) => match ct.check_type.clone() {
                    Some(check) => {
                        self.substitute_infer_type_parameters(&stored, &[check], &[Arc::clone(over)])
                    }
                    None => stored,
                },
                _ => stored,
            }
        };

        // Go isDeferredType（checker.go:25917-25920）→ isGenericType →
        // getGenericObjectFlags（checker.go:26353-26376）：mapped 型的泛型性
        // 由 isGenericMappedType（checker.go:26384-26399）判定——约束为泛型
        // 索引（keyof X 且 X 含型参）或 as 名型实例化后泛型；迭代 fresh
        // 型参与模板体内的型参引用不计。type_contains_type_parameter 的
        // Mapped 分支含 m.type_parameter（fresh 迭代型参恒为 TypeParameter
        // 形态），实参代入后的具体 mapped（约束 keyof {…} 已具体化）仍被
        // 误判含型参，本守卫恒挂起走 rebuild_deferred_conditional——
        // KeysWithoutStringIndex<{…}> 类别名（inferConditionalConstraint
        // MappedMember）实例化产物呈 deferred 壳挂 alias，声明名位渲染
        // 别名（>test : test），Go 同位经 getConditionalType 解析臂
        // （checker.go:25846/25866 instantiateType(branch) 无 alias）落
        // never；非 mapped 形态（裸型参/Index(T)/泛型 IndexedAccess 等）
        // 维持原谓词，与 Go isGenericType 语义一致
        let check_is_deferred = match &check_type.data {
            TypeData::Mapped(_) => self.is_generic_mapped_type_by_constraint(&check_type),
            _ => type_contains_type_parameter(&check_type),
        };
        if check_is_deferred {
            return None;
        }

        let infer_params: Vec<Arc<Type>> = ct
            .root
            .as_ref()
            .map(|r| r.infer_type_parameters.clone())
            .unwrap_or_default();
        let inferences: Vec<InferenceInfo> = infer_params
            .iter()
            .map(|p| InferenceInfo::new(Arc::clone(p)))
            .collect();
        let mut context = InferenceContext::new(inferences);

        if !infer_params.is_empty() {
            self.infer_types(
                &mut context.inferences,
                Some(Arc::clone(&check_type)),
                Some(Arc::clone(&extends_type)),
                InferencePriority::NoConstraints | InferencePriority::AlwaysStrict,
                false,
            );

            let inferred = self.get_inferred_types(&context);
            for inf in &inferred {
                if inf.flags.contains(TypeFlags::Any) && inf.intrinsic_name() == Some("error") {
                    return None;
                }
            }
        }

        let inferred_extends = if !infer_params.is_empty() {
            let inferred = self.get_inferred_types(&context);
            self.substitute_infer_type_parameters(&extends_type, &infer_params, &inferred)
        } else {
            Arc::clone(&extends_type)
        };
        let extends_any_or_unknown = inferred_extends
            .flags
            .intersects(TypeFlags::Any | TypeFlags::Unknown);
        let check_is_any = check_type.flags.contains(TypeFlags::Any);
        let definitely_false = if extends_any_or_unknown {
            false
        } else if check_is_any {
            true
        } else {
            let permissive_check = self.get_permissive_instantiation(&check_type);
            let permissive_extends = self.get_permissive_instantiation(&inferred_extends);
            let was_silent = self.silence_relation_chain();
            let r = !self.is_type_assignable_to(&permissive_check, &permissive_extends);
            self.restore_relation_chain(was_silent);
            r
        };
        let take_true = if !definitely_false {
            let definitely_true = if extends_any_or_unknown {
                true
            } else {
                let restrictive_check = self.get_restrictive_instantiation(&check_type);
                let restrictive_extends = self.get_restrictive_instantiation(&inferred_extends);
                let was_silent = self.silence_relation_chain();
                let r = self.is_type_assignable_to(&restrictive_check, &restrictive_extends);
                self.restore_relation_chain(was_silent);
                r
            };
            if !definitely_true {
                return None;
            }
            true
        } else {
            false
        };

        let include_true_branch = take_true == false && check_is_any;

        let (cond_node, branch_node) = match ct
            .root
            .as_ref()
            .and_then(|r| r.node.as_ref())
            .and_then(|n| match &n.data {
                NodeData::ConditionalTypeNode(data) => {
                    let branch = if take_true {
                        Arc::clone(&data.true_type)
                    } else {
                        Arc::clone(&data.false_type)
                    };
                    Some((Arc::clone(n), branch))
                }
                _ => None,
            }) {
            Some(pair) => pair,
            None => return None,
        };

        if take_true {
            self.push_scope(&cond_node);
        }

        let creation_scopes: Vec<u64> = ct
            .root
            .as_ref()
            .map(|r| r.creation_scopes.clone())
            .unwrap_or_default();
        let mut common = 0usize;
        while common < creation_scopes.len()
            && common < self.scope_stack.len()
            && creation_scopes[common] == self.scope_stack[common]
        {
            common += 1;
        }
        let scopes_pushed = creation_scopes.len() - common;
        self.scope_stack
            .extend_from_slice(&creation_scopes[common..]);

        let mut merged_creation: HashMap<usize, Arc<Type>> = HashMap::new();
        for frame in ct.creation_type_argument_stack.iter() {
            for (k, v) in frame {
                merged_creation.insert(*k, Arc::clone(v));
            }
        }
        for map in self.type_argument_stack.iter() {
            for k in map.keys() {
                merged_creation.remove(&(*k as usize));
            }
        }
        let pushes_creation = !merged_creation.is_empty();
        if pushes_creation {
            self.type_argument_stack.push(
                merged_creation
                    .into_iter()
                    .map(|(k, v)| ((k as *const Symbol), v))
                    .collect(),
            );
        }
        let branch = self.resolve_conditional_branch_node(t, &branch_node);
        if pushes_creation {
            self.type_argument_stack.pop();
        }
        if scopes_pushed > 0 {
            self.scope_stack
                .truncate(self.scope_stack.len() - scopes_pushed);
        }
        if take_true {
            self.pop_scope();
        }
        let resolved = if !infer_params.is_empty() {
            let inferred = self.get_inferred_types(&context);
            self.substitute_infer_type_parameters(&branch, &infer_params, &inferred)
        } else {
            Arc::clone(&branch)
        };

        let resolved = if include_true_branch
            && let Some(true_branch) = self.get_forced_branch_type_of_conditional_type(t, true)
        {
            let true_branch = if !infer_params.is_empty() {
                let inferred = self.get_inferred_types(&context);
                self.substitute_infer_type_parameters(&true_branch, &infer_params, &inferred)
            } else {
                true_branch
            };
            self.get_union_type(vec![true_branch, Arc::clone(&resolved)])
        } else {
            resolved
        };

        if let TypeData::Conditional(ct2) = &t.data {
            let cell = if take_true {
                &ct2.resolved_true_type
            } else {
                &ct2.resolved_false_type
            };
            let _ = cell.set(Arc::clone(&resolved));
        }
        Some(resolved)
    }
}
