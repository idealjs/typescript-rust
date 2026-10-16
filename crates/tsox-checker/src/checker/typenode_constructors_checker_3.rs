#![allow(unused_imports)]

use crate::checker::mig::w9a::new_type_mapper;
use crate::checker::typenode_constructors::*;

impl Checker {
    pub(crate) fn resolve_instantiated_object_members(&mut self, object_type: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("resolve_instantiated_object_members");
        let Some(o) = object_type.as_object() else {
            return Arc::clone(object_type);
        };
        // Go getStructuredTypeMembers 的 MembersResolved 门（checker.go:20195）：
        // resolveAnonymousTypeMembers target 分支经 setStructuredTypeMembers 原位
        // 一次性填充（checker.go:21872-21883），解析后身份恒定——已解析直接返回，
        // 未解析产物按输入 id 驻存，重复索引访问复用同一对象
        if object_type
            .object_flags
            .contains(crate::checker::types::ObjectFlags::MembersResolved)
        {
            return Arc::clone(object_type);
        }
        let Some(target) = o.target.clone() else {
            return Arc::clone(object_type);
        };
        if let Some(resolved) = self.instantiated_members_resolved.get(&object_type.id) {
            return Arc::clone(resolved);
        }
        // Go resolveAnonymousTypeMembers 解析期先落空成员表
        //（checker.go:21874 setStructuredTypeMembers(t, nil, nil, nil, nil)，
        // MembersResolved 旗标随之置位）再实例化填充（checker.go:21875-21879）：
        // 构建期重入查询（属性型实例化回查同一惰性壳）经旗标门见到空表
        // 而非递归重建。驻存表先落空表产物，构建后原位填充共享 Arc
        let mapper = o.mapper.clone().or_else(|| {
            // Go resolveTypeReferenceMembers（checker.go:20228-20236）：引用成员表
            // 的 mapper 取 source.allTypeParameters × getTypeArguments(t)，不读
            // 引用自身 mapper 字段（getTypeFromTypeReference 急切引用无 mapper，
            // 依赖成员克隆挂 Instantiated+mapper 走 getTypeOfInstantiatedSymbol
            // 的 instantiateType 代入；checker.go:20246-20260
            // resolveObjectTypeMembers 声明型直引时恒等分支不建 mapper）
            let tps = target
                .as_interface_type()
                .map(|d| d.all_type_parameters.clone())
                .unwrap_or_default();
            if tps.is_empty() {
                return None;
            }
            let mut args = o.type_arguments.clone();
            if args.len() == tps.len().saturating_sub(1) {
                args.push(Arc::clone(object_type));
            }
            if args.len() != tps.len() {
                return None;
            }
            if tps.iter().zip(args.iter()).all(|(a, b)| Arc::ptr_eq(a, b)) {
                return None;
            }
            Some(Arc::new(new_type_mapper(tps, args)))
        });
        let result = Arc::new(crate::checker::types::Type {
            flags: object_type.flags,
            object_flags: object_type.object_flags | crate::checker::types::ObjectFlags::MembersResolved,
            id: crate::checker::types::next_type_id(),
            symbol: object_type.symbol.clone(),
            alias: object_type.alias.clone(),
            data: crate::checker::types::TypeData::Object(crate::checker::types::ObjectTypeData {
                structured: crate::checker::types::StructuredTypeData::default(),
                target: Some(Arc::clone(&target)),
                mapper: mapper.clone(),
                type_arguments: o.type_arguments.clone(),
                node: o.node.clone(),
            }),
        });
        self.instantiated_members_resolved
            .insert(object_type.id, Arc::clone(&result));
        let properties = self.get_properties_of_object_type(&target);
        let members = crate::checker::mig::wc3::r25k2_defs::create_instantiated_symbol_table_opt(
            self,
            &properties,
            mapper.as_ref(),
        );
        let call_signatures = self.instantiate_signatures(
            &self.get_signatures_of_type(&target, crate::checker::types::SignatureKind::Call),
            mapper.as_ref(),
        );
        let construct_signatures = self.instantiate_signatures(
            &self.get_signatures_of_type(&target, crate::checker::types::SignatureKind::Construct),
            mapper.as_ref(),
        );
        let source_index_infos = self.get_index_infos_of_type(&target);
        let index_infos = self.instantiate_index_infos(&source_index_infos, mapper.as_ref());
        let call_count = call_signatures.len();
        let mut signatures = call_signatures.clone();
        signatures.extend(construct_signatures.iter().cloned());
        let mut structured = crate::checker::types::StructuredTypeData::default();
        structured.members = members;
        structured.properties = structured
            .members
            .iter()
            .map(|(_, s)| Arc::clone(s))
            .filter(|s| !crate::checker::utilities_is_optional_symbol::is_reserved_member_name(&s.name))
            .collect();
        structured.signatures = signatures;
        structured.call_signature_count = call_count;
        structured.index_infos = index_infos;
        let ptr = Arc::as_ptr(&result) as *mut crate::checker::types::Type;
        unsafe {
            if let crate::checker::types::TypeData::Object(od) = &mut (*ptr).data {
                od.structured = structured;
            }
        }
        result
    }

    pub(crate) fn get_indexed_access_type(
        &mut self,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_indexed_access_type");
        let object_type = &self.resolve_instantiated_object_members(object_type);
        if object_type.flags.contains(TypeFlags::Any) {
            return self.any_type();
        }
        if object_type.flags.contains(TypeFlags::Unknown) {
            return self.unknown_type();
        }
        if index_type.flags.contains(TypeFlags::Any) {
            return self.any_type();
        }

        // Go getIndexedAccessTypeWorker：对象为联合时按成分分发取并集
        //（如 ({type:"FOO"}|{type:"BAR"})["type"] → "FOO"|"BAR"）
        if object_type.is_union() {
            if let Some(members) = object_type.types() {
                let parts: Vec<Arc<Type>> = members
                    .iter()
                    .map(|c| self.get_indexed_access_type(c, index_type))
                    .collect();
                return self.get_union_type(parts);
            }
        }

        if index_type.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &index_type.data {
                let prop_types: Vec<Arc<Type>> = u
                    .union_or_intersection
                    .types
                    .iter()
                    .map(|c| self.get_indexed_access_type(object_type, c))
                    .collect();
                if prop_types.is_empty() {
                    return self.any_type();
                }
                return self.get_union_type(prop_types);
            }
        }

        if object_type.flags.contains(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_constraint_of_type_parameter(object_type) {
                return self.get_indexed_access_type(&constraint, index_type);
            }
            // Go createIndexedAccessType：泛型对象的索引访问保持延迟（驻留保恒等）
            return self.deferred_indexed_access_ex(object_type, index_type, None);
        }

        // Go getPropertyTypeForIndexType → getPropertyOfType：交集对象逐成分
        // 解析属性，命中成分求交（{a: X} & {b: {}}["a"] → X，非 any）
        if object_type.flags.contains(TypeFlags::Intersection)
            && let Some(constituents) = object_type.types()
        {
            // Go getPropertyTypeForIndexType（checker.go:28612-28633）具名属性
            // 索引先 getPropertyOfType 全交型取属性：交集经
            // createUnionOrIntersectionProperty（checker.go:22704-22797）合成，
            // 索引签名成分仅联合（isUnion 臂 checker.go:22783）出属性，交集
            // 中声明属性存在时索引签名成分不出——
            // {a:"true"} & {[k:string]:"false"}["a"] 取 "true"，不按成分求交
            // 得 "true" & "false"（never）
            if index_type.flags.contains(TypeFlags::StringLiteral) {
                if let TypeData::Literal(lit) = &index_type.data {
                    if let LiteralValue::String(name) = &lit.value {
                        if let Some(prop) =
                            self.get_union_or_intersection_property(object_type, name)
                        {
                            return self.get_type_of_symbol(&prop);
                        }
                    }
                }
            }
            let resolved: Vec<Arc<Type>> = constituents
                .iter()
                .filter_map(|c| {
                    self.try_get_indexed_access_type(c, index_type, AccessFlags::None)
                })
                .collect();
            match resolved.len() {
                0 => {}
                1 => return resolved.into_iter().next().unwrap(),
                _ => return self.get_intersection_type(resolved),
            }
        }

        if let TypeData::Mapped(m) = &object_type.data
            && let Some(constraint) = &m.constraint_type
        {
            // Go getIndexedAccessTypeOrUndefined 单键路径（checker.go:28605）走
            // getPropertyTypeForIndexType→getPropertyOfType（checker.go:28617-28628）：
            // literal 键按 as 名查 mapped 成员表（声明期急切构建的属性）直取
            // 属性型，成员表 miss 才落索引签名/回源。本侧成员表产物为 any 时
            // （急切求值期状态异常的兜底残留）不采信，继续走 domain 判定与
            // 回源重求双保险
            if index_type.flags.contains(TypeFlags::StringLiteral)
                && let TypeData::Literal(lit) = &index_type.data
                && let LiteralValue::String(name) = &lit.value
                && let Some(sym) = object_type
                    .as_structured()
                    .and_then(|s| s.members.get(name))
            {
                let t = self.get_type_of_symbol(sym);
                if !t.flags.contains(TypeFlags::Any) {
                    return t;
                }
            }
            let generic = constraint
                .flags
                .intersects(TypeFlags::TypeParameter | TypeFlags::IndexedAccess | TypeFlags::Index)
                || matches!(&constraint.data, TypeData::IndexedAccess(_));
            let domain = if generic {
                match self.constraint_of_indexed_access(constraint) {
                    Some(reduced) => reduced,
                    None => Arc::clone(constraint),
                }
            } else {
                Arc::clone(constraint)
            };
            if self.is_type_assignable_to(index_type, &domain) {
                if let Some(t) = self.instantiate_mapped_template_with_index(object_type, index_type)
                {
                    return t;
                }
                return self.get_any_type();
            }
        }

        if index_type.flags.contains(TypeFlags::StringLiteral) {
            if let TypeData::Literal(lit) = &index_type.data {
                if let LiteralValue::String(name) = &lit.value {
                    let name = name.clone();
                    // Go getPropertyTypeForIndexType（checker.go:28620-28659）hasPropName
                    // 臂：getPropertyOfType 解析成员（Object 臂经
                    // resolveStructuredTypeMembers 分派——映射壳
                    // resolveMappedTypeMembers、接口引用实例化成员），命中即
                    // getTypeOfSymbol 惰性求成员型；裸 structured 直读对别名
                    // 引用/未解析映射壳恒 miss（嵌套别名索引访问停半解析型）
                    if let Some(sym) = self.get_property_of_type(object_type, &name) {
                        return self.get_type_of_symbol(&sym);
                    }
                    if let Some(structured) = object_type.as_structured() {
                        if let Some(value_type) =
                            self.lookup_index_signature_value(structured, index_type)
                        {
                            return value_type;
                        }
                    }
                    if name == "length" && self.is_array_type(object_type) {
                        return self.number_type();
                    }
                    return self.any_type();
                }
            }
        }

        if index_type.flags.contains(TypeFlags::Number)
            || index_type.flags.contains(TypeFlags::NumberLiteral)
        {
            if self.is_array_type(object_type) {
                return self.get_array_element_type(object_type);
            }

            if self.is_tuple_type(object_type) {
                if let TypeData::Tuple(tup) = &object_type.data {
                    // 数字字面量按位取元素，number 取全体并集
                    if index_type.flags.contains(TypeFlags::NumberLiteral)
                        && let Some(n) = index_type.literal_value().and_then(|v| match v {
                            LiteralValue::Number(n) => Some(n),
                            _ => None,
                        })
                    {
                        let i = n.0 as usize;
                        if i < tup.element_infos.len()
                            && let Some(t) = &tup.element_infos[i].type_
                        {
                            return Arc::clone(t);
                        }
                    }
                    let elem_types: Vec<Arc<Type>> = tup
                        .element_infos
                        .iter()
                        .filter_map(|ei| ei.type_.clone())
                        .collect();
                    if !elem_types.is_empty() {
                        return self.get_union_type(elem_types);
                    }
                }
            }

            if index_type.flags.contains(TypeFlags::NumberLiteral)
                && let Some(n) = index_type.literal_value().and_then(|v| match v {
                    LiteralValue::Number(n) => Some(n),
                    _ => None,
                })
            {
                // 同 getStringLiteral 臂：数字名走 getPropertyOfType 成员解析
                //（Go getPropertyNameFromIndex 对数字名同径）
                let name = n.to_string();
                if let Some(sym) = self.get_property_of_type(object_type, &name) {
                    return self.get_type_of_symbol(&sym);
                }
                if let Some(structured) = object_type.as_structured()
                    && let Some(sym) = structured.members.get(&name)
                {
                    return self.get_type_of_symbol(sym);
                }
            }
        }

        if let Some(structured) = object_type.as_structured() {
            if let Some(value_type) = self.lookup_index_signature_value(structured, index_type) {
                return value_type;
            }
        }
        self.any_type()
    }

    pub(crate) fn instantiate_mapped_template_with_index(
        &mut self,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("instantiate_mapped_template_with_index"); 
        let TypeData::Mapped(m) = &object_type.data else {
            return None;
        };
        let substituted = m
            .declaration
            .as_ref()
            .and_then(|decl| match &decl.data {
                tsox_frontend::ast::NodeData::MappedTypeNode(d) => {
                    d.type_node.as_ref().map(|tn| {
                        (
                            Arc::clone(&d.type_parameter),
                            Arc::clone(tn),
                            Arc::clone(decl),
                        )
                    })
                }
                _ => None,
            })
            .and_then(|(tp_node, template_node, decl)| {
                let tp_sym = self.program.symbol_map().symbol_of(&tp_node).cloned()?;
                let chain = m
                    .template_subst
                    .as_ref()
                    .map(|c| c.as_ref().clone())
                    .unwrap_or_default();

                if !Self::type_node_references_name(&template_node, &tp_sym.name) {
                    return None;
                }
                let mut mapping = std::collections::HashMap::new();
                mapping.insert(
                    Arc::as_ptr(&tp_sym) as *const tsox_frontend::ast::Symbol,
                    Arc::clone(index_type),
                );
                self.push_scope(&decl);
                self.type_argument_stack.push(mapping);
                let t = self.get_type_from_type_node(&template_node);
                self.type_argument_stack.pop();
                self.pop_scope();
                let t = if chain.is_empty() {
                    t
                } else {
                    self.apply_template_subst_chain(&t, &chain)
                };
                Some(t)
            });
        if let Some(t) = substituted {
            return Some(t);
        }
        if let Some(t) = &m.template_type {
            return Some(Arc::clone(t));
        }
        let template_node = m.template_node.clone()?;
        let chain = m
            .template_subst
            .as_ref()
            .map(|c| c.as_ref().clone())
            .unwrap_or_default();
        let saved_stack = std::mem::take(&mut self.type_argument_stack);
        let t = self.get_type_from_type_node(&template_node);
        self.type_argument_stack = saved_stack;
        let t = if chain.is_empty() {
            t
        } else {
            self.apply_template_subst_chain(&t, &chain)
        };
        let ptr = Arc::as_ptr(object_type) as *mut crate::checker::types::Type;
        unsafe {
            if let TypeData::Mapped(m) = &mut (*ptr).data {
                m.template_type = Some(Arc::clone(&t));
            }
        }
        Some(t)
    }

    pub(crate) fn lookup_index_signature_value(
        &mut self,
        structured: &StructuredTypeData,
        index_type: &Arc<Type>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("lookup_index_signature_value"); 
        let is_string_like = index_type
            .flags
            .intersects(TypeFlags::String | TypeFlags::StringLiteral);
        let is_number_like = index_type
            .flags
            .intersects(TypeFlags::Number | TypeFlags::NumberLiteral);
        let mut string_index: Option<Arc<Type>> = None;
        for info in &structured.index_infos {
            let Some(key) = info.key_type.as_ref() else {
                continue;
            };
            if key.flags.contains(TypeFlags::String) {
                string_index = info.value_type.clone();
            } else if key.flags.contains(TypeFlags::Number) && is_number_like {
                return info.value_type.clone();
            }
        }
        if (is_string_like || is_number_like) && string_index.is_some() {
            return string_index;
        }
        None
    }
}
