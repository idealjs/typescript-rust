#![allow(unused_imports)]

use crate::checker::inference::*;
use crate::checker::types::*;

impl Checker {
    /// Go inferToMappedType：目标 { [P in keyof T]: X } / { [P in K]: X } 的推理
    pub(crate) fn infer_to_mapped_type(
        &mut self,
        state: &mut InferenceState,
        source: &Arc<Type>,
        target: &Arc<Type>,
        constraint_type: Option<&Arc<Type>>,
    ) -> bool {
        let Some(constraint) = constraint_type else {
            return false;
        };
        if constraint.flags.intersects(TypeFlags::Union | TypeFlags::Intersection) {
            let types = constraint.types().map(|t| t.to_vec()).unwrap_or_default();
            let mut result = false;
            for t in types {
                result = self.infer_to_mapped_type(state, source, target, Some(&t)) || result;
            }
            return result;
        }
        if let TypeData::Index(idx) = &constraint.data {
            // { [P in keyof T]: X }（同态）：从 source 反推 T
            let tp = idx.target.clone();
            let Some(tp) = tp else {
                return false;
            };
            let info = state
                .inferences
                .iter()
                .position(|i| crate::checker::utilities::type_parameters_match(&i.type_parameter, &tp));
                        if let Some(i) = info {
                if !state.inferences[i].is_fixed {
                    let inferred =
                        self.infer_type_for_homomorphic_mapped_type(source, target, constraint);
                                        if let Some(inferred) = inferred {
                        let priority = if source
                            .object_flags
                            .contains(ObjectFlags::NonInferrableType)
                        {
                            InferencePriority::PartialHomomorphicMappedType
                        } else {
                            InferencePriority::HomomorphicMappedType
                        };
                        let tp = state.inferences[i].type_parameter.clone();
                        self.infer_with_priority(state, &inferred, &tp, priority);
                    }
                }
            }
            return true;
        }
        if constraint.flags.contains(TypeFlags::TypeParameter) {
            // { [P in K]: X }：keyof source → K，再沿 K 的约束递归
            let keys = self.get_index_type(source);
            self.infer_with_priority(
                state,
                &keys,
                constraint,
                InferencePriority::MappedTypeConstraint,
            );
            if let Some(extended) = self.get_constraint_of_type_parameter(constraint) {
                if self.infer_to_mapped_type(state, source, target, Some(&extended)) {
                    return true;
                }
            }
            // 无处可推：属性类型并集 → 模板 X
            let mut prop_types: Vec<Arc<Type>> = source
                .as_structured()
                .map(|s| s.properties.iter().map(|p| self.get_type_of_symbol(p)).collect())
                .unwrap_or_default();
            if let Some(s) = source.as_structured() {
                for info in &s.index_infos {
                    if let Some(v) = &info.value_type {
                        prop_types.push(Arc::clone(v));
                    }
                }
            }
            if let Some(template) = self.get_template_type_from_mapped_type(target) {
                let union = self.get_union_type(prop_types);
                self.infer_from_types(state, &union, &template);
            }
            return true;
        }
        false
    }

    /// Go inferTypeForHomomorphicMappedType：构造 source 的反向映射型
    pub(crate) fn infer_type_for_homomorphic_mapped_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        constraint: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let key = (source.id, target.id, constraint.id);
        if let Some(cached) = self.reverse_mapped_cache.get(&key) {
            return cached.clone();
        }
        let t = self.create_reverse_mapped_type(source, target, constraint);
        self.reverse_mapped_cache.insert(key, t.clone());
        t
    }

    /// Go createReverseMappedType：数组/元组逐元素反向映射，对象构建惰性 ReverseMapped 型
    fn create_reverse_mapped_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        constraint: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        // Go：源有字符串索引签名，或（经 apparent 成员的）属性非空且部分可推断
        let source = &self.reify_interface_shell(source);
        let apparent = self.get_apparent_type(source);
        let has_index = apparent
            .as_structured()
            .is_some_and(|s| {
                s.index_infos.iter().any(|info| {
                    info.key_type
                        .as_ref()
                        .is_some_and(|k| k.flags.contains(TypeFlags::String))
                })
            });
        let has_props = apparent
            .as_structured()
            .is_some_and(|s| !s.properties.is_empty());
                if !has_index && !has_props {
            return None;
        }
        if self.is_array_type(source) {
            let elem = self
                .get_type_arguments(source)
                .into_iter()
                .next()?;
            let element = self.infer_reverse_mapped_type(&elem, target, constraint)?;
            return Some(self.create_array_type(element));
        }
        if source.object_flags.contains(ObjectFlags::Tuple) {
            let elems: Vec<Arc<Type>> = source
                .as_structured()
                .map(|s| s.properties.iter().map(|p| self.get_type_of_symbol(p)).collect())
                .unwrap_or_default();
            let mut new_elems = Vec::with_capacity(elems.len());
            for e in elems {
                new_elems.push(self.infer_reverse_mapped_type(&e, target, constraint)?);
            }
            return Some(self.create_tuple_type(new_elems));
        }
        let mut reversed = Type::new(
            TypeFlags::Object,
            TypeData::ReverseMapped(ReverseMappedTypeData {
                object: ObjectTypeData::default(),
                source: Some(Arc::clone(source)),
                mapped_type: Some(Arc::clone(target)),
                constraint_type: Some(Arc::clone(constraint)),
            }),
        );
        reversed.object_flags = ObjectFlags::Anonymous | ObjectFlags::ReverseMapped;
        let reversed = Arc::new(reversed);
        self.resolve_reverse_mapped_type_members(&reversed);
        Some(reversed)
    }

    /// Go inferReverseMappedType：按 T[P] ← X 推断 source 属性形态
    pub(crate) fn infer_reverse_mapped_type(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        constraint: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let key = (source.id, target.id, constraint.id);
        let cached = self.reverse_mapped_cache.get(&key).cloned();

        // Go inferReverseMappedType：仅当源与目标都深层嵌套（ExpandingFlagsBoth）才跳过；
        // 目标每层为新鲜实例时链持续生长，终止由显示层省略承担
        self.reverse_mapped_depth.push(source.id);
        self.reverse_mapped_target_depth.push(target.id);
        let source_deep = self
            .reverse_mapped_depth
            .iter()
            .filter(|&&id| id == source.id)
            .count()
            > 2;
        let target_deep = self
            .reverse_mapped_target_depth
            .iter()
            .filter(|&&id| id == target.id)
            .count()
            > 2;
        let too_deep = (source_deep && target_deep) || self.reverse_mapped_depth.len() > 16;
        self.template_resolution_letway = false;
        let t = if too_deep {
            None
        } else {
            self.infer_reverse_mapped_type_worker(source, target, constraint)
        };
        self.reverse_mapped_depth.pop();
        self.reverse_mapped_target_depth.pop();
        // 模板让位（外层解析在途）不算失败结果，不缓存，待外层完成后重试
        if !self.template_resolution_letway {
            self.reverse_mapped_cache.insert(key, t.clone());
        }
        t
    }

    fn infer_reverse_mapped_type_worker(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        constraint: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let TypeData::Index(idx) = &constraint.data else {
            return None;
        };
        let tp_of_mapped = self.get_type_parameter_from_mapped_type(target)?;
        let obj = idx.target.clone()?;
        // Go getIndexedAccessType(T, P)：泛型载体（类型参数/索引访问/条件型）保持延迟 T[P]
        // 按 (对象, 索引) 驻留保证类型恒等
        let generic_object = obj.flags.intersects(
            TypeFlags::TypeParameter | TypeFlags::IndexedAccess | TypeFlags::Conditional,
        ) || matches!(&obj.data, TypeData::IndexedAccess(_));
        let key_type = if generic_object {
            self.deferred_indexed_access(&obj, &tp_of_mapped)
        } else {
            self.get_indexed_access_type(&obj, &tp_of_mapped)
        };
        let template = self.get_template_type_from_mapped_type(target)?;
        let mut infos = vec![InferenceInfo::new(Arc::clone(&key_type))];
        self.infer_types(
            &mut infos,
            Some(Arc::clone(source)),
            Some(template),
            InferencePriority::None,
            false,
        );
        let t = self
            .get_type_from_inference(&infos[0])
            .unwrap_or_else(|| self.get_unknown_type());
        let widened = self.get_widened_type(&t);
        Some(widened)
    }

    /// Go resolveReverseMappedTypeMembers：source 的每个属性生成反向映射符号
    /// （类型经 reverse links 惰性求值）
    pub(crate) fn resolve_reverse_mapped_type_members(&mut self, t: &Arc<Type>) {
        let TypeData::ReverseMapped(r) = &t.data else {
            return;
        };
        let source = r.source.clone();
        let mapped = r.mapped_type.clone();
        let constraint = r.constraint_type.clone();
        let (Some(source), Some(mapped), Some(constraint)) = (source, mapped, constraint) else {
            return;
        };
        let apparent = self.get_apparent_type(&source);
        let props: Vec<Arc<Symbol>> = apparent
            .as_structured()
            .map(|s| s.properties.clone())
            .unwrap_or_default();
        // Go resolveReverseMappedTypeMembers：约束目标是 T[K_1]（对象/索引均类型参数）时
        // 归一为 T（replaceIndexedAccess），使各层 links 三元组恒等、缓存收敛自引用
        // Go resolveReverseMappedTypeMembers：源的字符串索引签名同样反向映射
        let mut index_infos: Vec<Arc<crate::checker::types::IndexInfo>> = Vec::new();
        if let Some(src_info) = apparent.as_structured().and_then(|s| {
            s.index_infos.iter().find(|info| {
                info.key_type
                    .as_ref()
                    .is_some_and(|k| k.flags.contains(TypeFlags::String))
            })
        }) {
            let value_type = src_info.value_type.clone().unwrap_or_else(|| self.get_unknown_type());
            let reversed_value = self
                .infer_reverse_mapped_type(&value_type, &mapped, &constraint)
                .unwrap_or_else(|| self.get_unknown_type());
            index_infos.push(Arc::new(crate::checker::types::IndexInfo {
                key_type: Some(self.string_type()),
                value_type: Some(reversed_value),
                is_readonly: src_info.is_readonly,
                declaration: None,
                index_symbol: None,
                components: Vec::new(),
            }));
        }
        let (mapped, constraint) =
            self.normalize_reverse_mapped_links(mapped, constraint);
        let mut members = SymbolTable::new();
        let mut new_props: Vec<Arc<Symbol>> = Vec::with_capacity(props.len());
        for prop in props {
            let mut sym = Symbol::new(prop.flags, prop.name.clone());
            sym.check_flags = tsox_frontend::ast::CheckFlags::ReverseMapped;
            sym.declarations = prop.declarations.clone();
            let sym = Arc::new(sym);
            let raw_prop_type = self.get_type_of_symbol(&prop);
            let property_type = self.reify_interface_shell(&raw_prop_type);
                        self.reverse_mapped_symbol_links.insert(
                &sym,
                ReverseMappedSymbolLinks {
                    property_type: Some(property_type),
                    mapped_type: Some(Arc::clone(&mapped)),
                    constraint_type: Some(Arc::clone(&constraint)),
                },
            );
            members.insert(prop.name.clone(), Arc::clone(&sym));
            new_props.push(sym);
        }
        let ptr = Arc::as_ptr(t) as *mut Type;
        unsafe {
            if let TypeData::ReverseMapped(r) = &mut (*ptr).data {
                r.object.structured.members = members;
                r.object.structured.properties = new_props;
                r.object.structured.index_infos = index_infos;
            }
        }
    }

    /// {[K2 in keyof T[K]]: X} 与 {[K2 in keyof T]: X} 反向映射等价：
    /// 把 links 的 (mapped, constraint) 归一到 T 形态（Go replaceIndexedAccess 语义）。
    /// 每层重新替换产生新实例（链不收敛，终止由显示层省略承担，对齐 Go）
    fn normalize_reverse_mapped_links(
        &mut self,
        mapped: Arc<Type>,
        constraint: Arc<Type>,
    ) -> (Arc<Type>, Arc<Type>) {
        let TypeData::Index(idx) = &constraint.data else {
            return (mapped, constraint);
        };
        let Some(ct) = idx.target.clone() else {
            return (mapped, constraint);
        };
        let TypeData::IndexedAccess(ia) = &ct.data else {
            return (mapped, constraint);
        };
        let (Some(obj), Some(index)) = (ia.object_type.clone(), ia.index_type.clone()) else {
            return (mapped, constraint);
        };
        if !obj.flags.contains(TypeFlags::TypeParameter)
            || !index.flags.contains(TypeFlags::TypeParameter)
        {
            return (mapped, constraint);
        }
        let key_of_t = self.get_index_type(&obj);
        let normalized = self.substitute_infer_type_parameters(&mapped, &[ct], &[obj]);
        (normalized, key_of_t)
    }
}

impl Checker {
    /// 自引用接口解析期返回的空成员壳：栈上无该符号解析时重解析完整成员并回填壳，
    /// 使所有持有壳的引用收敛（Go 声明类型一次性缓存 + 惰性成员的等价补救）
    pub(crate) fn reify_interface_shell(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let Some(sym) = t.symbol.clone() else {
            return Arc::clone(t);
        };
        if !sym.flags.contains(tsox_frontend::ast::SymbolFlags::Interface) {
            return Arc::clone(t);
        }
        let already_full = t.as_structured().is_some_and(|s| {
            !s.properties.is_empty()
                || !s.index_infos.is_empty()
                || !s.signatures.is_empty()
                || !s.members.is_empty()
        });
        if already_full {
            return Arc::clone(t);
        }
        let sym_key = Arc::as_ptr(&sym) as usize;
        if let Some(cached) = self.interface_shell_reify_cache.get(&sym_key) {
            return Arc::clone(cached);
        }
        let sym_ptr = Arc::as_ptr(&sym) as *const tsox_frontend::ast::Symbol;
        if self
            .type_resolution_stack
            .iter()
            .any(|e| e.target == sym_ptr)
        {
            return Arc::clone(t);
        }
        let full = self.resolve_interface_type_ex(&sym, None);
        let ptr = Arc::as_ptr(t) as *mut Type;
        unsafe {
            if let TypeData::Object(o) = &mut (*ptr).data
                && let Some(fs) = full.as_structured()
            {
                o.structured.members = fs.members.clone();
                o.structured.properties = fs.properties.clone();
                o.structured.signatures = fs.signatures.clone();
                o.structured.call_signature_count = fs.call_signature_count;
                o.structured.index_infos = fs.index_infos.clone();
            }
        }
        // 缓存回填后的壳本身：后续任意壳的 reify 都收敛到同一实例
        self.interface_shell_reify_cache
            .insert(sym_key, Arc::clone(t));
        Arc::clone(t)
    }

    /// 延迟 IndexedAccess 的驻留构造（同 (对象, 索引) 返回同一类型实例）
    pub(crate) fn deferred_indexed_access(
        &mut self,
        obj: &Arc<Type>,
        index: &Arc<Type>,
    ) -> Arc<Type> {
        let key = (obj.id, index.id);
        if let Some(cached) = self.deferred_indexed_access_cache.get(&key) {
            return Arc::clone(cached);
        }
        let mut ia = Type::new(
            TypeFlags::IndexedAccess,
            TypeData::IndexedAccess(IndexedAccessTypeData {
                constrained: Default::default(),
                object_type: Some(Arc::clone(obj)),
                index_type: Some(Arc::clone(index)),
                access_flags: Default::default(),
            }),
        );
        ia.symbol = obj.symbol.clone();
        let ia = Arc::new(ia);
        self.deferred_indexed_access_cache.insert(key, Arc::clone(&ia));
        ia
    }
}
