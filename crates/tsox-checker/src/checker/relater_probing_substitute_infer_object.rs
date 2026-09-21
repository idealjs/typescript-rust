#![allow(unused_imports)]

use crate::checker::relater_probing::*;

impl Checker {

    fn substitute_type_parameter_constraints(
        &mut self,
        tps: &[Arc<Type>],
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Vec<Arc<Type>> {
        tps.iter()
            .map(|tp| {
                let crate::checker::types::TypeData::TypeParameter(tpd) = &tp.data else {
                    return Arc::clone(tp);
                };
                let new_constraint = tpd.constraint.as_ref().map(|c| {
                    self.substitute_infer_type_parameters(c, params, substitutions)
                });
                let unchanged = tpd
                    .constraint
                    .as_ref()
                    .zip(new_constraint.as_ref())
                    .is_some_and(|(o, n)| Arc::ptr_eq(o, n))
                    || tpd.constraint.is_none();
                if unchanged {
                    return Arc::clone(tp);
                }
                let mut rebuilt = Type::new(
                    tp.flags,
                    crate::checker::types::TypeData::TypeParameter(
                        crate::checker::types::TypeParameterData {
                            constrained: Default::default(),
                            constraint: new_constraint,
                            target: tpd.target.clone(),
                            mapper: tpd.mapper.clone(),
                            is_this_type: tpd.is_this_type,
                            resolved_default_type: std::sync::OnceLock::new(),
                        },
                    ),
                );
                rebuilt.object_flags = tp.object_flags;
                rebuilt.symbol = tp.symbol.clone();
                Arc::new(rebuilt)
            })
            .collect()
    }

    fn instantiate_this_parameter_symbol(
        &mut self,
        this_param: &Option<Arc<Symbol>>,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Option<Arc<Symbol>> {
        let original = this_param.as_ref()?;
        let old = match self
            .value_symbol_links
            .get(original)
            .and_then(|l| l.resolved_type.clone())
        {
            Some(t) => t,
            None => self.get_type_of_symbol(original),
        };
        let new = self.substitute_infer_type_parameters(&old, params, substitutions);
        if Arc::ptr_eq(&old, &new) {
            return Some(Arc::clone(original));
        }
        let mut cloned = Symbol::new(original.flags, original.name.clone());
        cloned.check_flags = original.check_flags;
        cloned.declarations = original.declarations.clone();
        cloned.value_declaration = original.value_declaration.clone();
        if let Some(parent) = original.parent() {
            cloned.set_parent(&parent);
        }
        let cloned = Arc::new(cloned);
        self.value_symbol_links.insert(
            &cloned,
            crate::checker::types::ValueSymbolLinks {
                resolved_type: Some(new),
                target: Some(Arc::clone(original)),
                ..Default::default()
            },
        );
        Some(cloned)
    }

    pub(crate) fn substitute_infer_object(
        &mut self,
        t: &Arc<Type>,
        o: &ObjectTypeData,
        params: &[Arc<Type>],
        substitutions: &[Arc<Type>],
    ) -> Arc<Type> {
        if t.object_flags.contains(ObjectFlags::Reference)
            && o.type_arguments.len() == 1
            && t.symbol.as_ref().is_some_and(|s| {
                self.globals
                    .get("Array")
                    .is_some_and(|arr| Arc::ptr_eq(arr, s))
            })
        {
            let new_elem =
                self.substitute_infer_type_parameters(&o.type_arguments[0], params, substitutions);

            if Arc::ptr_eq(&new_elem, &o.type_arguments[0]) {
                return Arc::clone(t);
            }
            // readonly 标志随实例重建保持（U | readonly U[] 代入 U 后仍是 readonly）
            let readonly = t.object_flags.contains(ObjectFlags::IsReadonlyArray);
            return self.create_array_type_ex(new_elem, readonly);
        }

        if !o.type_arguments.is_empty() {
            let new_args: Vec<Arc<Type>> = o
                .type_arguments
                .iter()
                .map(|arg| self.substitute_infer_type_parameters(arg, params, substitutions))
                .collect();
            let changed = o
                .type_arguments
                .iter()
                .zip(new_args.iter())
                .any(|(old, new)| !Arc::ptr_eq(old, new));
            if changed {
                let new_index_infos: Vec<Arc<crate::checker::IndexInfo>> = o
                    .structured
                    .index_infos
                    .iter()
                    .map(|info| {
                        let new_value = info.value_type.as_ref().map(|v| {
                            self.substitute_infer_type_parameters(v, params, substitutions)
                        });
                        if new_value.is_some()
                            && !new_value.as_ref().is_some_and(|nv| {
                                info.value_type
                                    .as_ref()
                                    .is_some_and(|ov| Arc::ptr_eq(nv, ov))
                            })
                        {
                            Arc::new(crate::checker::IndexInfo {
                                key_type: info.key_type.clone(),
                                value_type: new_value,
                                is_readonly: info.is_readonly,
                                declaration: info.declaration.clone(),
                                index_symbol: info.index_symbol.clone(),
                                components: info.components.clone(),
                            })
                        } else {
                            Arc::clone(info)
                        }
                    })
                    .collect();
                let new_signatures: Vec<Arc<Signature>> = o
                    .structured
                    .signatures
                    .iter()
                    .map(|sig| {
                        let owned_inst: Vec<Arc<Type>>;
                        let old_inst: &[Arc<Type>] = match sig.instantiated_parameter_types.as_ref()
                        {
                            Some(inst) => inst,
                            None => {
                                owned_inst = sig
                                    .parameters
                                    .iter()
                                    .map(|p| self.get_type_of_symbol(p))
                                    .collect();
                                &owned_inst
                            }
                        };
                        let new_inst: Vec<Arc<Type>> = old_inst
                            .iter()
                            .map(|pt| {
                                self.substitute_infer_type_parameters(pt, params, substitutions)
                            })
                            .collect();
                        let changed = old_inst
                            .iter()
                            .zip(new_inst.iter())
                            .any(|(old, new)| !Arc::ptr_eq(old, new));
                        let return_changed = self
                            .get_return_type_of_signature(sig)
                            .is_some_and(|rt| {
                                let sub = self.substitute_infer_type_parameters(
                                    &rt, params, substitutions,
                                );
                                !Arc::ptr_eq(&sub, &rt)
                            });
                        if !changed && !return_changed {
                            return Arc::clone(sig);
                        }
                        let mut inst = Signature::new();
                        inst.flags = sig.flags;
                        inst.min_argument_count = sig.min_argument_count;
                        inst.resolved_min_argument_count = sig.resolved_min_argument_count;
                        inst.declaration = sig.declaration.clone();
                        inst.target = sig.target.clone();
                        inst.parameters = sig.parameters.clone();
                        inst.this_parameter = self.instantiate_this_parameter_symbol(
                            &sig.this_parameter,
                            params,
                            substitutions,
                        );
                        inst.type_parameters = self.substitute_type_parameter_constraints(
                            &sig.type_parameters,
                            params,
                            substitutions,
                        );
                        inst.resolved_type_predicate = sig.resolved_type_predicate.clone();
                        inst.instantiated_parameter_types = Some(new_inst);
                        if let Some(rt) = self.get_return_type_of_signature(sig) {
                            let new_rt =
                                self.substitute_infer_type_parameters(&rt, params, substitutions);
                            let _ = inst.resolved_return_type.set(new_rt);
                        }
                        Arc::new(inst)
                    })
                    .collect();
                let mut rebuilt = Type::new(
                    t.flags,
                    TypeData::Object(ObjectTypeData {
                        structured: StructuredTypeData {
                            members: o.structured.members.clone(),
                            properties: o.structured.properties.clone(),
                            signatures: new_signatures,
                            call_signature_count: o.structured.call_signature_count,
                            index_infos: new_index_infos,
                            ..Default::default()
                        },
                        target: o.target.clone(),
                        mapper: o.mapper.clone(),
                        type_arguments: new_args,
                    }),
                );
                rebuilt.object_flags = t.object_flags;
                rebuilt.symbol = t.symbol.clone();
                return Arc::new(rebuilt);
            }
            return Arc::clone(t);
        }

        if t.object_flags.contains(ObjectFlags::Anonymous) && !o.structured.signatures.is_empty() {
            let signatures = o.structured.signatures.clone();
            let call_signature_count = o.structured.call_signature_count;
            let mut changed = false;
            let mut new_sigs: Vec<Arc<Signature>> = Vec::with_capacity(signatures.len());
            for sig in &signatures {
                let rest_offset = usize::from(sig.has_rest_parameter());
                let fixed = sig.parameters.len().saturating_sub(rest_offset);
                let mut new_params: Vec<Arc<Type>> = Vec::with_capacity(sig.parameters.len());
                let mut old_params: Vec<Arc<Type>> = Vec::with_capacity(sig.parameters.len());
                for i in 0..fixed {
                    let pt = self
                        .try_get_type_at_position(sig, i)
                        .unwrap_or_else(|| self.any_type());
                    old_params.push(Arc::clone(&pt));
                    new_params.push(self.substitute_infer_type_parameters(
                        &pt,
                        params,
                        substitutions,
                    ));
                }
                if rest_offset == 1 {
                    if let Some(rt) = self.get_effective_rest_type(sig) {
                        old_params.push(Arc::clone(&rt));
                        new_params.push(self.substitute_infer_type_parameters(
                            &rt,
                            params,
                            substitutions,
                        ));
                    }
                }
                let new_return = self
                    .get_return_type_of_signature(sig)
                    .map(|rt| self.substitute_infer_type_parameters(&rt, params, substitutions));
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
                inst.resolved_min_argument_count = sig.resolved_min_argument_count;
                inst.declaration = sig.declaration.clone();
                inst.target = Some(Arc::clone(sig));
                inst.parameters = sig.parameters.clone();
                inst.this_parameter = self.instantiate_this_parameter_symbol(
                    &sig.this_parameter,
                    params,
                    substitutions,
                );
                inst.type_parameters = self.substitute_type_parameter_constraints(
                    &sig.type_parameters,
                    params,
                    substitutions,
                );
                inst.resolved_type_predicate = sig.resolved_type_predicate.clone();
                inst.instantiated_parameter_types = Some(new_params);
                if let Some(nr) = new_return {
                    let _ = inst.resolved_return_type.set(nr);
                }
                new_sigs.push(Arc::new(inst));
            }
            if !changed {
                return Arc::clone(t);
            }
            // 匿名对象同时带属性/索引成员（如 { (...) => void; _out?: T }）时
            // 保持对象形态：签名代入 + 属性深代入合成（Go instantiateType
            // 对成员全量实例化，坍缩成纯函数型会丢属性推断通道）
            if !o.structured.properties.is_empty() || !o.structured.index_infos.is_empty() {
                let fresh = self.subst_object_in_progress.is_empty();
                let with_props = self.substitute_object_properties_deep(t, params, substitutions);
                if fresh {
                    self.subst_object_in_progress.clear();
                }
                let props_changed = !Arc::ptr_eq(&with_props, t);
                let shell = if props_changed {
                    with_props
                } else {
                    Arc::new(Type::new(
                        t.flags,
                        TypeData::Object(ObjectTypeData {
                            structured: StructuredTypeData {
                                members: o.structured.members.clone(),
                                properties: o.structured.properties.clone(),
                                call_signature_count: o.structured.call_signature_count,
                                index_infos: o.structured.index_infos.clone(),
                                ..Default::default()
                            },
                            target: o.target.clone(),
                            mapper: o.mapper.clone(),
                            type_arguments: o.type_arguments.clone(),
                        }),
                    ))
                };
                {
                    let shell_mut = Arc::as_ptr(&shell) as *mut Type;
                    unsafe {
                        if let TypeData::Object(so) = &mut (*shell_mut).data {
                            so.structured.signatures = new_sigs;
                            so.structured.call_signature_count = o.structured.call_signature_count;
                        }
                    }
                }
                return shell;
            }
            let is_construct = call_signature_count == 0;
            return self.create_function_or_constructor_type(new_sigs, is_construct);
        }

        // Go instantiateType→getObjectTypeInstantiation：匿名对象类型的成员/索引签名按需实例化
        // （签名实例化与泛型 owner 成员访问路径同样深入，不限 call-return）。
        // Go couldContainTypeVariables 的对象分支：仅匿名且符号为
        // Function/Method/Class/TypeLiteral/ObjectLiteral 的类型可含类型变量，
        // 接口等解析型对象直接原样返回（否则推断探测会整图遍历 lib 接口）。
        // 自带声明类型参数的符号型（类/接口）走下方 attach 分支
        let deep_ok = match t.symbol.as_ref() {
            None => true,
            Some(sym) => {
                t.object_flags.contains(ObjectFlags::Anonymous)
                    && sym.flags.intersects(
                        tsox_frontend::ast::SymbolFlags::Function
                            | tsox_frontend::ast::SymbolFlags::Method
                            | tsox_frontend::ast::SymbolFlags::Class
                            | tsox_frontend::ast::SymbolFlags::TypeLiteral
                            | tsox_frontend::ast::SymbolFlags::ObjectLiteral,
                    )
                    && self.declared_type_parameter_types(sym).is_empty()
            }
        };
        if deep_ok
            && (!o.structured.properties.is_empty() || !o.structured.index_infos.is_empty())
        {
            let fresh = self.subst_object_in_progress.is_empty();
            let result = self.substitute_object_properties_deep(t, params, substitutions);
            if fresh {
                self.subst_object_in_progress.clear();
            }
            return result;
        }

        // Go instantiateType→getObjectTypeInstantiation：带声明类型参数的符号型
        //（类/接口声明型）把类型参数过映射后挂为实参（C --{T→number}--> C<number>）
        if o.type_arguments.is_empty()
            && let Some(sym) = t.symbol.clone()
        {
            let tps = self.declared_type_parameter_types(&sym);
            if !tps.is_empty() {
                let mapped: Vec<Arc<Type>> = tps
                    .iter()
                    .map(|tp| self.substitute_infer_type_parameters(tp, params, substitutions))
                    .collect();
                let changed = tps
                    .iter()
                    .zip(mapped.iter())
                    .any(|(a, b)| !Arc::ptr_eq(a, b));
                if changed {
                    return self.attach_explicit_type_arguments_cached(t, mapped);
                }
            }
        }

        Arc::clone(t)
    }
}
