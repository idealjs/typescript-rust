#![allow(unused_imports)]

use crate::checker::relater_probing::*;

impl Checker {
    pub(crate) fn substitute_infer_tuple(
        &mut self,
        t: &Arc<Type>,
        tup: &TupleTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("substitute_infer_tuple"); 
        let new_elems: Vec<Arc<Type>> = tup
            .element_infos
            .iter()
            .map(|ei| match &ei.type_ {
                Some(ty) => self.substitute_infer_type_parameters(ty, params, substitutions),
                None => self.error_type(),
            })
            .collect();

        let changed = tup
            .element_infos
            .iter()
            .zip(new_elems.iter())
            .any(|(ei, new_t)| match &ei.type_ {
                Some(old_t) => !Arc::ptr_eq(old_t, new_t),
                None => true,
            });
        if !changed {
            return Arc::clone(t);
        }
        let mut element_types: Vec<Arc<Type>> = Vec::new();
        let mut element_infos: Vec<TupleElementInfo> = Vec::new();
        for (ei, new_t) in tup.element_infos.iter().zip(new_elems.iter()) {
            if ei.flags.contains(ElementFlags::Variadic) {
                self.spread_variadic_tuple_element(
                    new_t,
                    ei,
                    &mut element_types,
                    &mut element_infos,
                );
            } else {
                element_types.push(Arc::clone(new_t));
                element_infos.push(TupleElementInfo {
                    label: ei.label.clone(),
                    flags: ei.flags,
                    labeled_declaration: ei.labeled_declaration.clone(),
                    type_: None,
                });
            }
        }
        self.create_tuple_type_ex(element_types, element_infos, tup.readonly)
    }

    pub(crate) fn substitute_infer_indexed_access(
        &mut self,
        t: &Arc<Type>,
        ia: &IndexedAccessTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("substitute_infer_indexed_access"); 
        let new_object = ia
            .object_type
            .as_ref()
            .map(|o| self.substitute_infer_type_parameters(o, params, substitutions));
        let new_index = ia
            .index_type
            .as_ref()
            .map(|idx| self.substitute_infer_type_parameters(idx, params, substitutions));
        let object_changed = new_object
            .as_ref()
            .zip(ia.object_type.as_ref())
            .map(|(new, old)| !Arc::ptr_eq(new, old))
            .unwrap_or(false);
        let index_changed = new_index
            .as_ref()
            .zip(ia.index_type.as_ref())
            .map(|(new, old)| !Arc::ptr_eq(new, old))
            .unwrap_or(false);
        if !object_changed && !index_changed {
            return Arc::clone(t);
        }
        let object_type = new_object.or_else(|| ia.object_type.clone());
        let index_type = new_index.or_else(|| ia.index_type.clone());
        // Go instantiateIndexedAccessType（instantiateTypeWorker checker.go:23577
        // → getIndexedAccessTypeEx checker.go:28532）内部以
        // shouldDeferIndexedAccessType（checker.go:28991）判延：泛型索引
        // （isGenericIndexType，含未归约 keyof 的 Index 旗标型）或泛型对象
        // （isGenericObjectType，含泛型映射/元组/型参）一律延迟驻留。此前自造
        // TypeParameter|IndexedAccess|Conditional 谓词漏掉 Index 旗标索引与泛型
        // 映射对象：crossProductUnionIntersectionRepeatedPropDeclarationsNoOOM1
        // 的 TLShape = TLIndexedShapes[keyof TLIndexedShapes]（keyof 索引在 Go
        // 侧因 isGenericIndexType 延迟）在探测/代入语境被急切归约，映射归约
        // 中途穿透在飞约束解析链
        if let (Some(obj), Some(idx)) = (object_type.as_ref(), index_type.as_ref())
            && !self.should_defer_indexed_access_type(obj, idx, None)
        {
            let resolved = self.get_indexed_access_type(obj, idx);
            if resolved.intrinsic_name() != Some("error") {
                return resolved;
            }
        }
        // 保持延迟的 IndexedAccess 按 (对象, 索引) 驻留，保证类型恒等
        // （推断信息表按恒等匹配，Go 依赖 interning）
        if let (Some(obj), Some(idx)) = (object_type.as_ref(), index_type.as_ref()) {
            let interned = self.deferred_indexed_access_ex(obj, idx, None);
            if interned.flags.contains(TypeFlags::IndexedAccess) {
                return interned;
            }
        }
        let mut rebuilt = Type::new(
            t.flags,
            TypeData::IndexedAccess(IndexedAccessTypeData {
                constrained: ConstrainedTypeData::default(),
                object_type,
                index_type,
                access_flags: ia.access_flags,
            }),
        );
        rebuilt.object_flags = t.object_flags;
        rebuilt.symbol = t.symbol.clone();
        Arc::new(rebuilt)
    }

    pub(crate) fn substitute_infer_conditional(
        &mut self,
        t: &Arc<Type>,
        ct: &ConditionalTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("substitute_infer_conditional"); 
        let Some(old_check) = ct.check_type.clone() else {
            return Arc::clone(t);
        };
        let old_extends = ct.extends_type.clone();
        let new_check = self.substitute_infer_type_parameters(&old_check, params, substitutions);
        let new_extends = old_extends
            .as_ref()
            .map(|e| self.substitute_infer_type_parameters(e, params, substitutions));
        let check_changed = !Arc::ptr_eq(&new_check, &old_check);
        let extends_changed = new_extends
            .as_ref()
            .zip(old_extends.as_ref())
            .is_some_and(|(n, o)| !Arc::ptr_eq(n, o));
        if !check_changed && !extends_changed {
            return Arc::clone(t);
        }
        // Go getConditionalType 的判延门（checker.go:25768/25812）：
        // isDeferredType = isGenericType（checker.go:26338-26341，
        // getGenericObjectFlags != 0：InstantiableNonPrimitive 顶层位、
        // 泛型映射/变长元组、Index 位、泛型 string-like，并/交/替换递归成分）
        // ——映射型的迭代参数（{[K in ...]} 的 K）是映射自有形参，不构成
        // 顶层泛型位；具体约束的映射（代入后 {[K in keyof 具体型]: ...}）
        // 两判均 false 应解析。原深递归 contains 判定（含 mapped 的
        // m.type_parameter 成分）使任何含映射型的 check 恒判泛型
        // （fresh K 恒命中）而 rebuild deferred——
        // inferConditionalConstraintMappedMember 的 KeysWithoutStringIndex
        // 条件（check 为具体约束映射）停 deferred 呈自引用别名形态，check
        // 侧改顶层位判定。extends 侧维持跳过本条件型 infer 形参的深递归
        // contains：Go 的 extends 判延发生在解析循环内
        //（checker.go:25812 isDeferredType(inferredExtendsType)，inferences
        // 已代入消费 infer 形参后），代入窗并无此判定；裸含 infer 形参的
        // extends（[unknown, ...infer X, unknown]、`${infer R}` 等）在 Go
        // 实例化即解析（getConditionalTypeInstantiation checker.go:23798 →
        // getConditionalType 推断消费），顶层位判定把 infer 形参计作
        // Instantiable 顶层位恒 defer——sliceTupleTypeOutOfBounds 的
        // Middle<[1]>（应 never）与 templateLiteralInferenceSupplementary-
        // Split/templateLiteralMultiByteCharInference/
        // templateLiteralInferenceLoneSurrogate 停 deferred 呈自引用别名
        let is_generic_top = |checker: &mut Checker, ty: &Arc<Type>| {
            !checker.get_generic_object_flags(ty).is_empty()
        };
        let infer_params: Vec<Arc<Type>> = ct
            .root
            .as_ref()
            .map(|r| r.infer_type_parameters.clone())
            .unwrap_or_default();
        if is_generic_top(self, &new_check)
            || new_extends
                .as_ref()
                .is_some_and(|e| {
                    crate::checker::relater_type_params::type_contains_type_parameter_skipping(
                        e, &infer_params,
                    )
                })
        {
            return self.rebuild_deferred_conditional(
                t,
                ct,
                new_check,
                new_extends,
                params,
                substitutions,
            );
        }
        // Go getConditionalType：分支结果 = instantiateType(branchNode, trueMapper)，
        // 解析出的分支再用 mapper 实例化（T→实参）；嵌套别名引用在实例化中
        // 按 (symbol, args) 缓存递归收敛
        // Go getConditionalType：分支以 trueMapper（本层 params→substitutions）
        // 实例化；帧栈是全局的，嵌套实例化时外层帧会污染本层分支节点的解析，
        // 解析期间把栈替换为本层帧
        let saved_stack = std::mem::take(&mut self.type_argument_stack);
        let mut frames: Vec<HashMap<*const Symbol, Arc<Type>>> = ct
            .creation_type_argument_stack
            .iter()
            .map(|frame| {
                frame
                    .iter()
                    .map(|(k, v)| {
                        (
                            *k as *const Symbol,
                            self.substitute_infer_type_parameters(v, params, substitutions),
                        )
                    })
                    .collect()
            })
            .collect();
        let mut frame: HashMap<*const Symbol, Arc<Type>> = HashMap::new();
        for (i, p) in params.iter().enumerate() {
            if let Some(sym) = &p.symbol {
                frame.insert(
                    Arc::as_ptr(sym),
                    Arc::clone(&substitutions[i.min(substitutions.len() - 1)]),
                );
            }
        }
        if !frame.is_empty() {
            frames.push(frame);
        }
        self.type_argument_stack = frames;
        // Go getConditionalTypeInstantiation：distributive 条件型的 check 型
        // 被映射为 union 时按成分分发求值（A extends U ? X : Y | B extends U ?
        // X : Y），never 收敛为 never；成分不可判定时保持挂起
        let distribute = ct.root.as_ref().is_some_and(|r| r.is_distributive)
            && (new_check.flags.contains(TypeFlags::Never)
                || matches!(&new_check.data, TypeData::Union(_)));
        let resolved_branch = if new_check.flags.contains(TypeFlags::Never) && distribute {
            Some(self.never_type())
        } else if distribute
            && let TypeData::Union(u) = &new_check.data
            && let Some(check_sym) = ct
                .root
                .as_ref()
                .and_then(|r| r.check_type_parameter_symbol.clone())
        {
            let constituents = u.union_or_intersection.types.clone();
            let key = Arc::as_ptr(&check_sym) as *const tsox_frontend::ast::Symbol;
            let mut results: Vec<Arc<Type>> = Vec::with_capacity(constituents.len());
            let mut all_resolved = true;
            for constituent in constituents {
                let mut mapping = HashMap::new();
                mapping.insert(key, Arc::clone(&constituent));
                self.type_argument_stack.push(mapping);
                let r = self.resolve_conditional_type_with_check(
                    t,
                    Some(Arc::clone(&constituent)),
                    new_extends.clone(),
                );
                self.type_argument_stack.pop();
                match r {
                    Some(r) => results.push(r),
                    None => {
                        all_resolved = false;
                        break;
                    }
                }
            }
            if all_resolved {
                Some(self.get_union_type(results))
            } else {
                None
            }
        } else {
            self.resolve_conditional_type_with_check(
                t,
                Some(Arc::clone(&new_check)),
                new_extends.clone(),
            )
        };
        self.type_argument_stack = saved_stack;
        match resolved_branch {
            Some(branch) => self.substitute_infer_type_parameters(&branch, params, substitutions),
            None => self.rebuild_deferred_conditional(
                t,
                ct,
                new_check,
                new_extends,
                params,
                substitutions,
            ),
        }
    }

    /// 挂起条件型代入后仍含类型参数（别名体 T→TActor 这类外层形参）：Go
    /// instantiateType 对 deferred conditional 实例化 check/extends 并携带
    /// mapper；这里重建携带代入映射的新挂起条件，映射并入
    /// creation_type_argument_stack 供后续分支解析时回放
    fn rebuild_deferred_conditional(
        &mut self,
        t: &Arc<Type>,
        ct: &ConditionalTypeData,
        new_check: Arc<Type>,
        new_extends: Option<Arc<Type>>,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("rebuild_deferred_conditional"); 
        let mut creation_stack: Vec<HashMap<usize, Arc<Type>>> = ct
            .creation_type_argument_stack
            .iter()
            .map(|frame| {
                frame
                    .iter()
                    .map(|(k, v)| {
                        (
                            *k,
                            self.substitute_infer_type_parameters(v, params, substitutions),
                        )
                    })
                    .collect()
            })
            .collect();
        let mut frame: HashMap<usize, Arc<Type>> = HashMap::new();
        for (i, p) in params.iter().enumerate() {
            if let Some(sym) = &p.symbol {
                frame.insert(
                    Arc::as_ptr(sym) as usize,
                    Arc::clone(&substitutions[i.min(substitutions.len() - 1)]),
                );
            }
        }
        if !frame.is_empty() {
            creation_stack.push(frame);
        }
        let substituted_alias: Option<Box<crate::checker::types::TypeAlias>> = t.alias.as_ref().map(|a| {
            let new_args: Vec<Arc<Type>> = a
                .type_arguments
                .iter()
                .map(|arg| self.substitute_infer_type_parameters(arg, params, substitutions))
                .collect();
            Box::new(crate::checker::types::TypeAlias::new(
                a.symbol.clone(),
                new_args,
            ))
        });
        // Go getConditionalTypeInstantiation（checker.go:23798-23827）把条件型
        // 实例化驻留在 root.instantiations，键 = getConditionalTypeKey（outer
        // 形参经组合 mapper 映射后的实参, 调用点 alias, forConstraint）。键中
        // alias 是**调用点**传参：源级别名引用路径 getTypeAliasInstantiation
        // （checker.go:25021）对普通引用传 newAlias=nil，worker 裸
        // instantiateType（checker.go:23402）同样以 nil 入键——两路径同实参
        // 共享同一条目；实例自身 alias 是构造产物（checker.go:25940-25944
        // result.alias = instantiateTypeAlias(root.alias, mapper)），不参与键。
        // 本臂此前键带 substituted_alias（实例自身 alias，如 MR[condX]），worker
        // 以 nil 键按同实参查 miss → 同 (root, 实参) 双实例，
        // identicalGenericConditionalsWithInferRelated 的 MappedResult 实例两引擎
        // 各取其一，跨实例比较呈 Two different types with this name。键改恒 nil
        // 后两引擎同域同条目（recursiveReverseMappedType 的 Recur 条件成员
        // t.alias 本为 None，键不变）
        //
        // Go 未命中时 newMapper = newTypeMapper(outer, typeArguments)
        // （checker.go:23816）——规范化 [outer→实参]，deferred 产物携带
        // newMapper（newConditionalType），后续 worker 再实例化
        // combineTypeMappers(t.mapper, m) 与分支解析（getTrue/FalseType
        // FromConditionalType 的 d.mapper）都按规范化 mapper 施加。携带组合
        // mapper（ct.mapper ∘ params→substitutions）会把非 outer 形参也映射进
        // 分支（Go 只映射 outer），复合序偏离 Go
        let frame_mapper: Option<Arc<crate::checker::TypeMapper>> =
            if params.is_empty() || params.len() != substitutions.len() {
                None
            } else {
                Some(Arc::new(crate::checker::mig::w9a::new_type_mapper(
                    params.to_vec(),
                    substitutions.to_vec(),
                )))
            };
        let combined_mapper = match &frame_mapper {
            Some(fm) => self.combine_type_mappers(ct.mapper.as_ref(), Some(fm)),
            None => ct.mapper.clone(),
        };
        let cache_frame: Option<(usize, u64, Arc<crate::checker::TypeMapper>)> =
            ct.root.as_deref().and_then(|root| {
                if root.outer_type_parameters.is_empty() {
                    return None;
                }
                let mapper = combined_mapper.as_ref()?;
                // typeArguments 双层计算：先过 combined mapper（ct.mapper 链的
                // sources 是 root.outer 形参自身，指针命中），再过本引擎符号
                // 容错代入。root.outer_type_parameters（get_declared_type_of_
                // type_parameter，declared_type_links）与 params（get_type_
                // parameter_from_symbol，type_alias_links）是同符号的**不同
                // Arc 实例**，TypeMapper 按 Arc::ptr_eq 匹配对后者落空——
                // 单走 mapper.map 会让 type_arguments 恒等坍缩为 outer 参数
                // 自身，键撞声明期恒等播种（nil alias 同域）误返声明态条件型，
                // 代入全部丢失（Exclude<T, U> 应为 Exclude<keyof T, never>）。
                // 第二层必须是**浅层单参数替换**（对齐 Go core.Map(outer,
                // mapper.Map) checker.go:23803：实参表逐参代入、结果原样取用，
                // 不递归入结构）——深层 substitute_infer 会穿入实参内的条件型
                // 重新进入本函数（自引用条件型每层重建产物为新指针，
                // infer_subst_ancestor_stack 按指针判环永不命中），
                // superPropertyAccess/superElementAccess 栈溢出崩溃由此而来
                let type_arguments: Vec<Arc<Type>> = root
                    .outer_type_parameters
                    .iter()
                    .map(|p| {
                        let mapped = mapper.map(p);
                        // CompositeTypeMapper.Map（mapper.go:321-327）：m1 命中
                        // （t1 != t）时结果为深实例化产物原样取用；m1 落空走
                        // m2.Map(t) 浅层单参替换——本仓 m2（fm，sources=
                        // type_alias_links 实例）对 outer（declared_links
                        // 实例）指针落空，等价补位 = 符号容错浅层代入
                        if Arc::ptr_eq(&mapped, p) {
                            match params
                                .iter()
                                .position(|q| self.infer_param_matches(q, p))
                            {
                                Some(i) if i < substitutions.len() => {
                                    Arc::clone(&substitutions[i])
                                }
                                _ => mapped,
                            }
                        } else {
                            mapped
                        }
                    })
                    .collect();
                let key = crate::checker::mig::m1d_2::get_conditional_type_key(
                    &type_arguments,
                    &None::<Box<crate::checker::types::TypeAlias>>,
                    false,
                );
                let canonical_mapper = Arc::new(crate::checker::mig::w9a::new_type_mapper(
                    root.outer_type_parameters.clone(),
                    type_arguments,
                ));
                Some((root as *const _ as usize, key, canonical_mapper))
            });
        if let Some((root_key, key, _)) = &cache_frame
            && let Some(hit) =
                crate::checker::mig::wc1b::r25k3_defs::conditional_type_instantiations_get(
                    *root_key, *key,
                )
        {
            return hit;
        }
        // check/extends 取本引擎代入产物（new_check/new_extends）：Go 的
        // instantiateType(root.checkType, newMapper)（checker.go:25753-25754）
        // 依赖 root.checkType 与 newMapper 的 sources（outer 形参）为同一对象；
        // 本仓 root.checkType（type_alias_links 实例）与 outer（declared_
        // type_links 实例）分叉，canonical（sources=outer）对其指针落空，
        // 再生是恒等 no-op 会回退未代入形态——new_check/new_extends 经符号
        // 容错代入，与 Go 的 [outer→实参] 施加语义等价
        let (recheck, reextends, canonical_mapper) = match &cache_frame {
            Some((_, _, canonical)) => (new_check, new_extends, Some(Arc::clone(canonical))),
            None => (new_check, new_extends, combined_mapper),
        };
        let mut rebuilt = Type::new(
            t.flags,
            TypeData::Conditional(ConditionalTypeData {
                constrained: ConstrainedTypeData::default(),
                root: ct.root.clone(),
                check_type: Some(recheck),
                extends_type: reextends,
                resolved_true_type: OnceLock::new(),
                resolved_false_type: OnceLock::new(),
                resolved_inferred_true_type: OnceLock::new(),
                resolved_default_constraint: OnceLock::new(),
                resolved_constraint_of_distributive: OnceLock::new(),
                mapper: canonical_mapper,
                combined_mapper: ct.combined_mapper.clone(),
                creation_type_argument_stack: creation_stack,
            }),
        );
        rebuilt.symbol = t.symbol.clone();
        rebuilt.alias = substituted_alias;
        rebuilt.object_flags = t.object_flags;
        let rebuilt = Arc::new(rebuilt);
        if let Some((root_key, key, _)) = cache_frame {
            crate::checker::mig::wc1b::r25k3_defs::conditional_type_instantiations_insert(
                root_key,
                key,
                Arc::clone(&rebuilt),
            );
        }
        rebuilt
    }
}
