#![allow(unused_imports)]

use crate::checker::typenode_constructors::*;

impl Checker {
    pub(crate) fn substituted_member_type_of(
        &mut self,
        owner: &Arc<Type>,
        prop: &Arc<Symbol>,
    ) -> Arc<Type> {
        let Some(obj) = owner.as_object() else {
            return self.get_type_of_symbol(prop);
        };
        if obj.type_arguments.is_empty() {
            return self.get_type_of_symbol(prop);
        }
        let Some(owner_sym) = owner.symbol.clone() else {
            return self.get_type_of_symbol(prop);
        };
        let key = (
            Arc::as_ptr(owner) as *const crate::checker::types::Type as usize,
            Arc::as_ptr(prop) as *const tsox_frontend::ast::Symbol as usize,
        );
        if let Some(cached) = self.instantiated_member_type_cache.get(&key) {
            return Arc::clone(&cached.2);
        }

        let result = if owner_sym.flags.contains(SymbolFlags::Interface) {
            let proper =
                self.resolve_interface_type_ex(&owner_sym, Some(obj.type_arguments.clone()));
            let prop_sym = proper
                .as_structured()
                .and_then(|s| s.members.get(&prop.name).cloned());
            match prop_sym {
                Some(ps) => self.get_type_of_symbol(&ps),
                None => self.get_type_of_symbol(prop),
            }
        } else {
            self.substitute_member_type_fallback(&owner_sym, prop, &obj.type_arguments)
        };

        if self.instantiated_member_type_cache.len() >= self.instantiated_member_type_cache_limit {
            self.instantiated_member_type_cache.clear();
        }

        self.instantiated_member_type_cache.insert(
            key,
            (Arc::clone(owner), Arc::clone(prop), Arc::clone(&result)),
        );
        result
    }

    pub(crate) fn substitute_member_type_fallback(
        &mut self,
        owner_sym: &Arc<Symbol>,
        prop: &Arc<Symbol>,
        args: &[Arc<Type>],
    ) -> Arc<Type> {
        let decl_tps = self.declared_type_parameter_types(owner_sym);
        if decl_tps.len() == args.len() && !decl_tps.is_empty() {
            let raw = self.member_decl_symbol_type(prop);
            let substitutions = args.to_vec();
            let r = self.substitute_infer_type_parameters(&raw, &decl_tps, &substitutions);
            r
        } else {
            self.get_type_of_symbol(prop)
        }
    }

    pub(crate) fn declared_type_parameter_types(&mut self, symbol: &Arc<Symbol>) -> Vec<Arc<Type>> {
        // 全部声明的类型参数（interface 增强文件的同名 T 是独立符号，须一并
        // 纳入代入表，否则增强成员的类型参数悬空）
        let tp_syms: Vec<Arc<Symbol>> = {
            let sym_map = self.program.symbol_map();
            symbol
                .declarations
                .iter()
                .filter_map(|decl| match &decl.data {
                    NodeData::InterfaceDeclaration(d) => d.type_parameters.as_ref(),
                    NodeData::ClassDeclaration(d) => d.type_parameters.as_ref(),
                    _ => None,
                })
                .flat_map(|tps| tps.iter())
                .filter_map(|tp| sym_map.symbol_of(tp).map(Arc::clone))
                .collect()
        };
        if tp_syms.is_empty() {
            return Vec::new();
        }

        self.push_ts2304_suppression();
        let types = tp_syms
            .iter()
            .map(|tp_sym| self.get_type_parameter_from_symbol(tp_sym))
            .collect();
        self.pop_ts2304_suppression();
        types
    }

    pub(crate) fn collect_free_type_parameters_deep(
        &mut self,
        t: &Arc<Type>,
        out: &mut Vec<Arc<Type>>,
    ) {
        match &t.data {
            TypeData::TypeParameter(_) => {
                if !out.iter().any(|p| Arc::ptr_eq(p, t)) {
                    out.push(Arc::clone(t));
                }
            }
            TypeData::Union(u) => {
                for ty in &u.union_or_intersection.types {
                    self.collect_free_type_parameters_deep(ty, out);
                }
            }
            TypeData::Intersection(i) => {
                for ty in &i.union_or_intersection.types {
                    self.collect_free_type_parameters_deep(ty, out);
                }
            }
            TypeData::Object(o) => {
                for ty in &o.type_arguments {
                    self.collect_free_type_parameters_deep(ty, out);
                }

                for sig in o.structured.signatures.clone() {
                    for param in sig.parameters.iter() {
                        let pt = self.get_type_of_symbol(param);
                        self.collect_free_type_parameters_deep(&pt, out);
                    }
                    if let Some(rt) = sig.resolved_return_type.get() {
                        let rt = Arc::clone(rt);
                        self.collect_free_type_parameters_deep(&rt, out);
                    }
                }
            }
            TypeData::Tuple(tu) => {
                for ei in &tu.element_infos {
                    if let Some(ty) = &ei.type_ {
                        self.collect_free_type_parameters_deep(ty, out);
                    }
                }
            }
            TypeData::IndexedAccess(ia) => {
                if let Some(obj) = &ia.object_type {
                    self.collect_free_type_parameters_deep(obj, out);
                }
                if let Some(idx) = &ia.index_type {
                    self.collect_free_type_parameters_deep(idx, out);
                }
            }
            _ => {}
        }
    }

    #[allow(dead_code)]
    pub(crate) fn create_tuple_type_named(
        &mut self,
        element_types: Vec<Arc<Type>>,
        names: Vec<String>,
    ) -> Arc<Type> {
        let element_infos: Vec<TupleElementInfo> = element_types
            .iter()
            .enumerate()
            .map(|(i, t)| TupleElementInfo {
                label: names.get(i).cloned(),
                flags: ElementFlags::Required,
                labeled_declaration: None,
                type_: Some(Arc::clone(t)),
            })
            .collect();
        let fixed_length = element_infos.len();
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Tuple,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: crate::checker::types::TypeData::Tuple(crate::checker::types::TupleTypeData {
                interface_data: Default::default(),
                element_infos,
                min_length: fixed_length,
                fixed_length,
                combined_flags: ElementFlags::Required,
                readonly: false,
            }),
        })
    }

    pub(crate) fn create_tuple_type(&mut self, element_types: Vec<Arc<Type>>) -> Arc<Type> {
        let element_infos: Vec<TupleElementInfo> = element_types
            .iter()
            .map(|t| TupleElementInfo {
                label: None,
                flags: ElementFlags::Required,
                labeled_declaration: None,
                type_: Some(Arc::clone(t)),
            })
            .collect();
        let fixed_length = element_types.len();
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Tuple,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::Tuple(TupleTypeData {
                interface_data: InterfaceTypeData::default(),
                element_infos,
                min_length: fixed_length,
                fixed_length,
                combined_flags: ElementFlags::Required,
                readonly: false,
            }),
        })
    }

    pub(crate) fn get_index_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(TypeFlags::Never) {
            return self.never_type();
        }
        if t.flags.contains(TypeFlags::Any) {
            return self.string_type();
        }

        if t.flags.contains(TypeFlags::Union) {
            let types = match &t.data {
                TypeData::Union(u) => &u.union_or_intersection.types,
                _ => return self.never_type(),
            };
            let mut common: Option<Vec<String>> = None;
            for constituent in types {
                let k = self.get_index_type(constituent);
                let names = self.string_literal_values(&k);
                common = Some(match common.take() {
                    None => names,
                    Some(acc) => acc.into_iter().filter(|n| names.contains(n)).collect(),
                });
            }
            let names = common.unwrap_or_default();
            if names.is_empty() {
                return self.never_type();
            }
            let literals: Vec<Arc<Type>> = names
                .into_iter()
                .map(|n| self.get_string_literal_type(&n))
                .collect();
            return self.get_union_type(literals);
        }

        if t.flags.contains(TypeFlags::Intersection) {
            let types = match &t.data {
                TypeData::Intersection(i) => &i.union_or_intersection.types,
                _ => return self.never_type(),
            };
            let keys: Vec<Arc<Type>> = types.iter().map(|c| self.get_index_type(c)).collect();
            return self.get_union_type(keys);
        }

        if t.flags.contains(TypeFlags::TypeParameter) {
            if let Some(constraint) = self.get_constraint_of_type_parameter(t) {
                return self.get_index_type(&constraint);
            }
            // Go shouldDeferIndexType：类型参数的 keyof 保持延迟 Index 类型（驻留保恒等）
            return self.deferred_index_type(t);
        }
        // Go shouldDeferIndexType（InstantiableNonPrimitive）：泛型挂起的索引
        // 访问/条件的 keyof 同样保持延迟（keyof T["_type"] 不归约为 never）
        if t.flags.intersects(TypeFlags::IndexedAccess | TypeFlags::Conditional)
            || matches!(&t.data, TypeData::IndexedAccess(_))
        {
            return self.deferred_index_type(t);
        }

        if let TypeData::Mapped(m) = &t.data
            && let Some(constraint) = &m.constraint_type
        {
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
            if domain.flags.contains(TypeFlags::String) {
                let keys = vec![domain, self.number_type()];
                return self.get_union_type(keys);
            }
            return domain;
        }

        if let Some(structured) = t.as_structured() {
            let mut keys: Vec<Arc<Type>> = structured
                .properties
                .iter()
                .filter(|p| {
                    !p.name.starts_with('#')
                        && !crate::checker::exports::get_declaration_modifier_flags_from_symbol(p)
                            .intersects(
                                tsox_frontend::ast::ModifierFlags::NonPublicAccessibilityModifier,
                            )
                })
                .map(|p| self.get_string_literal_type(&p.name))
                .collect();
            for info in &structured.index_infos {
                if let Some(key) = &info.key_type {
                    keys.push(Arc::clone(key));

                    if key.flags.contains(TypeFlags::String) {
                        keys.push(self.number_type());
                    }
                }
            }
            if keys.is_empty() {
                return self.never_type();
            }
            return self.get_union_type(keys);
        }

        self.never_type()
    }

    pub(crate) fn type_node_references_name(node: &Arc<Node>, name: &str) -> bool {
        if node.kind == SyntaxKind::Identifier && node.text() == name {
            return true;
        }
        let mut found = false;
        tsox_frontend::ast::node_data_generated::for_each_child(node, |c| {
            found = found || Self::type_node_references_name(c, name);
            found
        });
        found
    }
}

impl Checker {
    fn deferred_index_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if let Some(cached) = self.index_type_cache.get(&t.id) {
            return Arc::clone(cached);
        }
        let index = Arc::new(Type::new(
            TypeFlags::Index,
            TypeData::Index(crate::checker::types::IndexTypeData {
                constrained: Default::default(),
                target: Some(Arc::clone(t)),
                index_flags: Default::default(),
            }),
        ));
        self.index_type_cache.insert(t.id, Arc::clone(&index));
        index
    }

    // 类实例型成员是急建合成符号（无注解方法返回 any 驻缓存）：回源 binder
    // 声明符号走惰性体推断（Go 成员即 binder 符号、返回型惰性解析）
    pub(crate) fn member_decl_symbol_type(&mut self, prop: &Arc<Symbol>) -> Arc<Type> {
        if let Some(decl) = prop
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::MethodDeclaration)
        {
            let binder_sym = self.program.symbol_map().symbol_of(decl).map(Arc::clone);
            if let Some(bs) = binder_sym
                && !Arc::ptr_eq(&bs, prop)
            {
                let t = self.get_type_of_symbol(&bs);
                if !t.flags.contains(TypeFlags::Any) {
                    return self.substitute_member_this_type(decl, t);
                }
            }
        }
        self.get_type_of_symbol(prop)
    }

    // Go getTypeWithThisArgument：非 this 接收者上的成员签名把多态 this
    // 按声明容器实例化（A.foo(): this -> A）
    fn substitute_member_this_type(
        &mut self,
        method_decl: &Arc<Node>,
        t: Arc<Type>,
    ) -> Arc<Type> {
        let owner = match method_decl.parent() {
            Some(p) => p,
            None => return t,
        };
        if !matches!(
            owner.kind,
            SyntaxKind::ClassDeclaration
                | SyntaxKind::ClassExpression
                | SyntaxKind::InterfaceDeclaration
        ) {
            return t;
        }
        let instance = self.container_instance_type_of(&owner);
        let this_t = self.create_this_type(&owner, Arc::clone(&instance));
        self.substitute_infer_type_parameters(&t, &[this_t], &[instance])
    }
}
