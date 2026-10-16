#![allow(unused_imports)]

use crate::checker::mig::wc3::r25k2_defs::create_instantiated_symbol_table_opt;
use crate::checker::mig::w9a::new_type_mapper;
use crate::checker::relater_probing::*;

impl Checker {
    pub fn get_permissive_instantiation(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_permissive_instantiation"); 
        let key = t.id;
        if let Some(cached) = self.probe_cache_permissive.get(&key) {
            return Arc::clone(cached);
        }
        let result = self.instantiate_probing(t, ProbeMode::Permissive);
        self.probe_cache_permissive.insert(key, Arc::clone(&result));
        result
    }

    pub fn get_restrictive_instantiation(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_restrictive_instantiation"); 
        let key = t.id;
        if let Some(cached) = self.probe_cache_restrictive.get(&key) {
            return Arc::clone(cached);
        }
        let result = self.instantiate_probing(t, ProbeMode::Restrictive);
        self.probe_cache_restrictive
            .insert(key, Arc::clone(&result));
        result
    }

    pub(crate) fn instantiate_probing(&mut self, t: &Arc<Type>, mode: ProbeMode) -> Arc<Type> { ::tsox_core::fntrace::enter("instantiate_probing"); 
        // Go instantiateType 先解析结构化成员（惰性签名在此填充）
        if t.flags.contains(TypeFlags::Object)
            && t.as_object()
                .is_some_and(|o| o.structured.signatures.is_empty())
        {
            let _ = self.get_signatures_of_type(t, crate::checker::SignatureKind::Call);
        }
        match &t.data {
            TypeData::TypeParameter(_) => match mode {
                // Go permissiveMapper：类型参数 → wildcard（Any 旗标，与一切双向可赋值）
                ProbeMode::Permissive => self.get_any_type(),
                ProbeMode::Restrictive => {
                    let tp = match &t.data {
                        TypeData::TypeParameter(tp) => tp,
                        _ => unreachable!(),
                    };
                    if tp.constraint.is_none() {
                        return Arc::clone(t);
                    }
                    let mut rebuilt = Type::new(
                        t.flags,
                        TypeData::TypeParameter(TypeParameterData {
                            constrained: ConstrainedTypeData::default(),
                            constraint: None,
                            target: tp.target.clone(),
                            mapper: tp.mapper.clone(),
                            is_this_type: tp.is_this_type,
                            resolved_default_type: std::sync::RwLock::new(None),
                        }),
                    );
                    rebuilt.symbol = t.symbol.clone();
                    rebuilt.object_flags = t.object_flags;
                    Arc::new(rebuilt)
                }
            },
            TypeData::IndexedAccess(ia) => {
                let new_object = ia
                    .object_type
                    .as_ref()
                    .map(|o| self.instantiate_probing(o, mode));
                let new_index = ia
                    .index_type
                    .as_ref()
                    .map(|i| self.instantiate_probing(i, mode));
                let changed = new_object.as_ref().zip(ia.object_type.as_ref()).is_some_and(
                    |(n, o)| !Arc::ptr_eq(n, o),
                ) || new_index
                    .as_ref()
                    .zip(ia.index_type.as_ref())
                    .is_some_and(|(n, o)| !Arc::ptr_eq(n, o));
                if !changed {
                    return Arc::clone(t);
                }
                let object = new_object.or_else(|| ia.object_type.clone());
                let index = new_index.or_else(|| ia.index_type.clone());
                // Go 探测走 instantiateTypeWorker IndexedAccess 臂（checker.go
                // :23577）→ getIndexedAccessTypeEx 全引擎（checker.go:28532），
                // 内部 shouldDeferIndexedAccessType（checker.go:28991）对泛型
                // 索引（含未归约 keyof 的 Index 旗标型）/泛型对象（含泛型映射）
                // 延迟驻留不归约——自造 TypeParameter|IndexedAccess|Conditional
                // 谓词漏掉这两类，探测语境急切归约会穿透在飞约束解析链
                // （crossProductUnionIntersectionRepeatedPropDeclarationsNoOOM1
                // 的 TLIndexedShapes[keyof TLIndexedShapes]）
                if let (Some(obj), Some(idx)) = (object.as_ref(), index.as_ref())
                    && !self.should_defer_indexed_access_type(obj, idx, None)
                {
                    return self.get_indexed_access_type(obj, idx);
                }
                let mut rebuilt = Type::new(
                    t.flags,
                    TypeData::IndexedAccess(crate::checker::types::IndexedAccessTypeData {
                        constrained: ConstrainedTypeData::default(),
                        object_type: object,
                        index_type: index,
                        access_flags: ia.access_flags,
                    }),
                );
                rebuilt.object_flags = t.object_flags;
                rebuilt.symbol = t.symbol.clone();
                Arc::new(rebuilt)
            }
            TypeData::Union(u) => {
                let types = u.union_or_intersection.types.clone();
                let new_types: Vec<Arc<Type>> = types
                    .iter()
                    .map(|c| self.instantiate_probing(c, mode))
                    .collect();
                if new_types
                    .iter()
                    .zip(types.iter())
                    .all(|(n, o)| Arc::ptr_eq(n, o))
                {
                    return Arc::clone(t);
                }
                self.get_union_type(new_types)
            }
            TypeData::Intersection(i) => {
                let types = i.union_or_intersection.types.clone();
                let new_types: Vec<Arc<Type>> = types
                    .iter()
                    .map(|c| self.instantiate_probing(c, mode))
                    .collect();
                if new_types
                    .iter()
                    .zip(types.iter())
                    .all(|(n, o)| Arc::ptr_eq(n, o))
                {
                    return Arc::clone(t);
                }
                self.get_intersection_type(new_types)
            }
            TypeData::Object(o) => {
                if o.type_arguments.is_empty() {
                    // Go getPermissive/RestrictiveInstantiation（checker.go:25944/25960）
                    // → instantiateTypeWorker（checker.go:23536-23544）→
                    // getObjectTypeInstantiation（checker.go:23620-23691）：带声明符号的
                    // 匿名对象按声明外层类型参数生成探测实例，探测与常规实例化同
                    // worker 同驻留域——探测须穿透对象字面量内的类型参数
                    //（{ type: T } permissive → wildcard、restrictive → 去约束克隆），
                    // 条件型 A extends { type: T } 的 deferral 才能按 Go 由
                    // restrictive 探测判定
                    // Go instantiateTypeWorker Object 臂（checker.go:23536-23544）
                    // 只对 objectFlags 含 Reference|Anonymous|Mapped 的对象下探
                    // getObjectTypeInstantiation（外层形参上溯住在其中，
                    // checker.go:23644-23646），其余对象（接口/类/枚举/无三旗标
                    // 的合成位）原样返回 t——探测不得经符号声明回溯激活；本分支
                    // 先前对任意带声明符号对象上溯，交集展开等合成位在 Go 不取
                    // 外层形参的位置被激活探测
                    if !t.object_flags.intersects(crate::checker::types::ObjectFlags::Reference)
                        && t.object_flags
                            .intersects(crate::checker::types::ObjectFlags::ANONYMOUS | crate::checker::types::ObjectFlags::MAPPED)
                        && let Some(declaration) =
                            t.symbol().and_then(|s| s.declarations.first().cloned())
                    {
                        let mut type_parameters =
                            self.get_outer_type_parameters(&declaration, true);
                        if t.symbol().is_some_and(|s| {
                            s.flags.intersects(
                                tsox_frontend::ast::SymbolFlags::Method
                                    | tsox_frontend::ast::SymbolFlags::TypeLiteral,
                            )
                        }) {
                            let decls =
                                t.symbol().map(|s| s.declarations.clone()).unwrap_or_default();
                            type_parameters.retain(|tp| {
                                decls.iter().any(|d| {
                                    self.is_type_parameter_possibly_referenced(tp, d)
                                })
                            });
                        }
                        if !type_parameters.is_empty() {
                            // Go mapTypeWithCompositeMapper(tp, t.Mapper(), m)：
                            // 已实例化对象先过自身 mapper 再探测；mapper 键取
                            // 自身 mapper 作用后的外层形参，worker 内部复合
                            //（m1 先行）后与 Go 同键
                            let object_mapper = t.mapper();
                            let mapped_params: Vec<Arc<Type>> = type_parameters
                                .iter()
                                .map(|tp| match &object_mapper {
                                    Some(m1) => m1.map(tp),
                                    None => Arc::clone(tp),
                                })
                                .collect();
                            let type_arguments: Vec<Arc<Type>> = mapped_params
                                .iter()
                                .map(|p| self.instantiate_probing(p, mode))
                                .collect();
                            let changed = type_arguments
                                .iter()
                                .zip(mapped_params.iter())
                                .any(|(a, p)| !Arc::ptr_eq(a, p));
                            if changed {
                                // Go 探测实例化与常规实例化同引擎同域
                                // （getObjectTypeInstantiation checker.go:23620-23691）：
                                // per-target 驻留 + 恒等播种使同键探测产物身份
                                // 稳定，成员表经 instantiateAnonymousType 惰性
                                // 定型；本分支原先绕过 worker 裸建新对象，
                                // 每次探测产出新身份且急切展开源成员表
                                let mapper = Arc::new(new_type_mapper(mapped_params, type_arguments));
                                return self.get_object_type_instantiation(
                                    t,
                                    Some(mapper.as_ref()),
                                    None,
                                );
                            }
                        }
                    }
                    // Go instantiateAnonymousType：匿名对象的签名/属性同样探测
                    // （restrictive 剥掉签名内类型参数的约束，使条件型探测正确 defer）
                    if !o.structured.signatures.is_empty() {
                        let mut changed = false;
                        let mut new_sigs: Vec<Arc<Signature>> =
                            Vec::with_capacity(o.structured.signatures.len());
                        for sig in &o.structured.signatures {
                            let mut new_params: Vec<Arc<Type>> = Vec::new();
                            let mut old_params: Vec<Arc<Type>> = Vec::new();
                            // Go instantiateSignature：按参数符号类型映射（rest 参数保持
                            // 数组形态，不折叠为位置元素）
                            for (i, param) in sig.parameters.iter().enumerate() {
                                let pt = match &sig.instantiated_parameter_types {
                                    Some(ov) => ov
                                        .get(i)
                                        .cloned()
                                        .unwrap_or_else(|| self.get_type_of_symbol(param)),
                                    None => self.get_type_of_symbol(param),
                                };
                                let np = self.instantiate_probing(&pt, mode);
                                old_params.push(Arc::clone(&pt));
                                new_params.push(np);
                            }
                            let new_return = self
                                .get_return_type_of_signature(sig)
                                .map(|rt| self.instantiate_probing(&rt, mode));
                            let params_changed = old_params
                                .iter()
                                .zip(new_params.iter())
                                .any(|(old, new)| !Arc::ptr_eq(old, new));
                            let return_changed = new_return.as_ref().is_some_and(|nr| {
                                self.get_return_type_of_signature(sig)
                                    .is_some_and(|old| !Arc::ptr_eq(nr, &old))
                            });
                            if !params_changed && !return_changed {
                                new_sigs.push(Arc::clone(sig));
                                continue;
                            }
                            changed = true;
                            let mut inst = Signature::new();
                            inst.flags = sig.flags;
                            inst.min_argument_count = sig.min_argument_count;
                            inst.resolved_min_argument_count =
                                sig.resolved_min_argument_count;
                            inst.declaration = sig.declaration.clone();
                            inst.target = Some(Arc::clone(sig));
                            inst.parameters = sig.parameters.clone();
                            inst.this_parameter = sig.this_parameter.clone();
                            inst.type_parameters = sig.type_parameters.clone();
                            inst.resolved_type_predicate = sig.resolved_type_predicate.clone();
                            inst.instantiated_parameter_types = Some(new_params);
                            if let Some(nr) = new_return {
                                let _ = inst.resolved_return_type.set(nr);
                            }
                            new_sigs.push(Arc::new(inst));
                        }
                        if changed {
                            let is_construct = o.structured.call_signature_count == 0;
                            return self.create_function_or_constructor_type(new_sigs, is_construct);
                        }
                    }
                    return Arc::clone(t);
                }
                let new_args: Vec<Arc<Type>> = o
                    .type_arguments
                    .iter()
                    .map(|a| self.instantiate_probing(a, mode))
                    .collect();
                if new_args
                    .iter()
                    .zip(o.type_arguments.iter())
                    .all(|(n, old)| Arc::ptr_eq(n, old))
                {
                    return Arc::clone(t);
                }
                if o.type_arguments.len() == 1 && self.is_array_type(t) {
                    let readonly = t
                        .object_flags
                        .contains(crate::checker::types::ObjectFlags::IsReadonlyArray);
                    return self.create_array_type_ex(Arc::clone(&new_args[0]), readonly);
                }
                let mut rebuilt = Type::new(
                    t.flags,
                    TypeData::Object(ObjectTypeData { node: None,
                        structured: StructuredTypeData::default(),
                        target: o.target.clone(),
                        mapper: None,
                        type_arguments: new_args,
                    }),
                );
                rebuilt.object_flags = t.object_flags;
                rebuilt.symbol = t.symbol.clone();
                Arc::new(rebuilt)
            }
            TypeData::Tuple(tup) => {
                let args: Vec<Arc<Type>> = tup
                    .element_infos
                    .iter()
                    .filter_map(|ei| ei.type_.clone())
                    .collect();
                if args.is_empty() {
                    return Arc::clone(t);
                }
                let new_elems: Vec<Arc<Type>> = args
                    .iter()
                    .map(|e| self.instantiate_probing(e, mode))
                    .collect();
                if new_elems
                    .iter()
                    .zip(args.iter())
                    .all(|(n, o)| Arc::ptr_eq(n, o))
                {
                    return Arc::clone(t);
                }
                self.create_tuple_type(new_elems)
            }
            TypeData::Conditional(ct) => {
                let (old_check, old_extends) =
                    match (ct.check_type.as_ref(), ct.extends_type.as_ref()) {
                        (Some(c), Some(e)) => (Arc::clone(c), Arc::clone(e)),
                        _ => return Arc::clone(t),
                    };
                let new_check = self.instantiate_probing(&old_check, mode);
                let new_extends = self.instantiate_probing(&old_extends, mode);
                if Arc::ptr_eq(&new_check, &old_check) && Arc::ptr_eq(&new_extends, &old_extends) {
                    return Arc::clone(t);
                }
                let mut rebuilt = Type::new(
                    t.flags,
                    TypeData::Conditional(ConditionalTypeData {
                        constrained: ConstrainedTypeData::default(),
                        root: ct.root.as_ref().map(|r| {
                            Box::new(ConditionalRoot {
                                node: r.node.clone(),
                                check_type: r.check_type.clone(),
                                extends_type: r.extends_type.clone(),
                                is_distributive: r.is_distributive,
                                check_type_parameter_symbol: r.check_type_parameter_symbol.clone(),
                                infer_type_parameters: r.infer_type_parameters.clone(),
                                outer_type_parameters: r.outer_type_parameters.clone(),
                                alias: None,
                                creation_scopes: r.creation_scopes.clone(),
                            })
                        }),
                        check_type: Some(new_check),
                        extends_type: Some(new_extends),
                        resolved_true_type: OnceLock::new(),
                        resolved_false_type: OnceLock::new(),
                        resolved_inferred_true_type: OnceLock::new(),
                        resolved_default_constraint: OnceLock::new(),
                        resolved_constraint_of_distributive: OnceLock::new(),
                        mapper: None,
                        combined_mapper: None,
                        creation_type_argument_stack: Vec::new(),
                    }),
                );
                rebuilt.symbol = t.symbol.clone();
                rebuilt.object_flags = t.object_flags;
                Arc::new(rebuilt)
            }
            _ => Arc::clone(t),
        }
    }

    /// 类型参数符号等价（跨树副本）：同名且容器符号名相同（或共享容器指针）。
    /// lib 副本/多份符号实例下指针恒等失效，名字级等价保证替换帧仍可命中
    pub(crate) fn type_param_symbols_equivalent(
        &self,
        a: &Symbol,
        b: &Symbol,
    ) -> bool { ::tsox_core::fntrace::enter("type_param_symbols_equivalent"); 
        if std::ptr::eq(a as *const Symbol, b as *const Symbol) {
            return true;
        }
        if a.name != b.name {
            return false;
        }
        let symbol_map = self.program.symbol_map();
        let container_name = |s: &Symbol| -> Option<String> {
            let mut node = s.declarations.first()?.parent()?;
            for _ in 0..4 {
                if let Some(sym) = symbol_map.symbols.get(&node.id()) {
                    return Some(sym.name.clone());
                }
                node = node.parent()?;
            }
            None
        };
        match (container_name(a), container_name(b)) {
            (Some(x), Some(y)) => x == y,
            (None, None) => true,
            _ => false,
        }
    }

    /// 阴影判定同声明等价（Go typeParameterShadowsOtherTypeParameterInScope
    /// nodebuilderimpl.go:1524 `result != typeParameter.symbol` 的声明身份
    /// 语义：同声明恒同指针不阴影，跨声明即阴影）：指针相等或声明节点相同
    /// 才算同符号，实例化克隆（instantiate_symbol 携带原 declarations）与
    /// 多树副本仍判同声明，重载签名族同名型参（跨声明）判阴影
    pub(crate) fn type_param_symbols_same_declaration(
        &self,
        a: &Symbol,
        b: &Symbol,
    ) -> bool { ::tsox_core::fntrace::enter("type_param_symbols_same_declaration");
        if std::ptr::eq(a as *const Symbol, b as *const Symbol) {
            return true;
        }
        a.declarations
            .iter()
            .any(|da| b.declarations.iter().any(|db| da.id() == db.id()))
    }

    #[allow(dead_code)]
    pub(crate) fn type_param_symbols_share_container(
        &self,
        a: &Arc<Symbol>,
        b: &Arc<Symbol>,
    ) -> bool { ::tsox_core::fntrace::enter("type_param_symbols_share_container"); 
        let symbol_map = self.program.symbol_map();
        let container_of = |s: &Arc<Symbol>| -> Option<usize> {
            let mut node = s.declarations.first()?.parent()?;
            for _ in 0..4 {
                if let Some(sym) = symbol_map.symbols.get(&node.id()) {
                    return Some(Arc::as_ptr(sym) as *const Symbol as usize);
                }
                node = node.parent()?;
            }
            None
        };
        match (container_of(a), container_of(b)) {
            (Some(x), Some(y)) => x == y,
            _ => false,
        }
    }
}
