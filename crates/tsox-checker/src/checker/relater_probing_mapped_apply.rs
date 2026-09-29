#![allow(unused_imports)]

use crate::checker::relater_probing::*;

impl Checker {
    /// Go instantiateConstituent：按 S 的形态应用映射型
    pub(crate) fn apply_mapped_over_type(
        &mut self,
        t: &Arc<Type>,
        m: &crate::checker::types::MappedTypeData,
        s: &Arc<Type>,
        chain: &[(Vec<Arc<Type>>, Vec<Arc<Type>>)],
    ) -> Arc<Type> {
        // 原始类型/any/unknown/泛型载体：直接透传（Go: primitive no mapping）
        if !s.flags.intersects(TypeFlags::Object | TypeFlags::Intersection)
            && !self.type_is_generic(s)
        {
            return Arc::clone(s);
        }
        if let TypeData::Union(u) = &s.data {
            let parts: Vec<Arc<Type>> = u
                .union_or_intersection
                .types
                .iter()
                .map(|c| self.apply_mapped_over_type(t, m, c, chain))
                .collect();
            return self.get_union_type(parts);
        }
        if self.is_array_type(s) && !s.object_flags.contains(ObjectFlags::Tuple) {
            let mapped_elem = self.mapped_template_at(m, &self.number_type(), chain);
            return self.create_array_type(mapped_elem);
        }
        if s.object_flags.contains(ObjectFlags::Tuple) {
            let TypeData::Tuple(td) = &s.data else {
                let keys = self.get_index_type(s);
                return self
                    .expand_mapped_by_constraint(t, m, &keys, chain)
                    .unwrap_or_else(|| Arc::clone(s));
            };
            let (include_opt, exclude_opt) = m
                .declaration
                .as_ref()
                .and_then(|d| match &d.data {
                    NodeData::MappedTypeNode(md) => Some(match &md.question_token {
                        Some(tok) if tok.kind == tsox_frontend::ast::SyntaxKind::MinusToken => {
                            (false, true)
                        }
                        Some(_) => (true, false),
                        None => (false, false),
                    }),
                    _ => None,
                })
                .unwrap_or((false, false));
            let homomorphic_param = match &m.constraint_type.as_ref().map(|c| &c.data) {
                Some(TypeData::Index(idx)) => idx.target.clone(),
                _ => None,
            };
            let mut new_elems: Vec<Arc<Type>> = Vec::with_capacity(td.element_infos.len());
            let mut new_infos = td.element_infos.clone();
            for (i, ei) in td.element_infos.iter().enumerate() {
                let mapped = if i < td.fixed_length {
                    let key =
                        self.get_number_literal_type(tsox_core::jsnum::Number::from(i as f64));
                    self.mapped_template_at(m, &key, chain)
                } else if ei.flags.contains(ElementFlags::Variadic) {
                    let elem = ei.type_.clone().unwrap_or_else(|| self.error_type());
                    self.apply_mapped_over_type(t, m, &elem, chain)
                } else {
                    let elem = ei.type_.clone().unwrap_or_else(|| self.error_type());
                    let arr = self.create_array_type(elem);
                    match homomorphic_param.clone() {
                        Some(o) => {
                            let mut scoped: Vec<(Vec<Arc<Type>>, Vec<Arc<Type>>)> =
                                vec![(vec![o], vec![arr])];
                            scoped.extend(chain.iter().cloned());
                            self.mapped_template_at(m, &self.number_type(), &scoped)
                        }
                        None => self.mapped_template_at(m, &self.number_type(), chain),
                    }
                };
                match (include_opt, exclude_opt) {
                    (true, _) if ei.flags.contains(ElementFlags::Required) => {
                        new_infos[i].flags = ElementFlags::Optional;
                    }
                    (_, true) if ei.flags.contains(ElementFlags::Optional) => {
                        new_infos[i].flags = ElementFlags::Required;
                    }
                    _ => {}
                }
                new_elems.push(mapped);
            }
            return self.create_tuple_type_ex(new_elems, new_infos, td.readonly);
        }
        // 对象：按 keyof S 的键展开
        let keys = self.get_index_type(s);
        self.expand_mapped_by_constraint(t, m, &keys, chain)
            .unwrap_or_else(|| Arc::clone(s))
    }

    /// Go TupleNormalizer.normalize（checker.go:23690 起）的 Variadic 分支：
    /// 具体元组元素逐个并入，泛型载体保持 Variadic，any/数组形态降为 Rest
    pub(crate) fn spread_variadic_tuple_element(
        &mut self,
        t: &Arc<Type>,
        info: &TupleElementInfo,
        element_types: &mut Vec<Arc<Type>>,
        element_infos: &mut Vec<TupleElementInfo>,
    ) {
        let rest_info = |labeled: &Option<Arc<Node>>| TupleElementInfo {
            label: None,
            flags: ElementFlags::Rest,
            labeled_declaration: labeled.clone(),
            type_: None,
        };
        if t.flags.contains(TypeFlags::Any) {
            element_types.push(Arc::clone(t));
            element_infos.push(rest_info(&info.labeled_declaration));
            return;
        }
        if self.type_is_generic(t) {
            element_types.push(Arc::clone(t));
            element_infos.push(TupleElementInfo {
                label: info.label.clone(),
                flags: ElementFlags::Variadic,
                labeled_declaration: info.labeled_declaration.clone(),
                type_: None,
            });
            return;
        }
        if let TypeData::Tuple(inner) = &t.data {
            for inner_ei in &inner.element_infos {
                element_types
                    .push(inner_ei.type_.clone().unwrap_or_else(|| self.error_type()));
                element_infos.push(TupleElementInfo {
                    label: inner_ei.label.clone(),
                    flags: inner_ei.flags,
                    labeled_declaration: inner_ei.labeled_declaration.clone(),
                    type_: None,
                });
            }
            return;
        }
        let elem = if self.is_array_type(t) {
            self.get_element_type_of_array_type(t)
                .unwrap_or_else(|| self.error_type())
        } else {
            self.error_type()
        };
        element_types.push(elem);
        element_infos.push(rest_info(&info.labeled_declaration));
    }

    /// Go createNormalizedTupleTypeEx 对含 Variadic 元素的元组做元素级归一
    pub(crate) fn normalize_variadic_tuple_elements(
        &mut self,
        element_types: Vec<Arc<Type>>,
        element_infos: &[TupleElementInfo],
    ) -> (Vec<Arc<Type>>, Vec<TupleElementInfo>) {
        let has_variadic = element_infos
            .iter()
            .any(|ei| ei.flags.contains(ElementFlags::Variadic));
        if !has_variadic {
            return (element_types, element_infos.to_vec());
        }
        let mut out_types: Vec<Arc<Type>> = Vec::new();
        let mut out_infos: Vec<TupleElementInfo> = Vec::new();
        for (ei, ty) in element_infos.iter().zip(element_types.iter()) {
            if ei.flags.contains(ElementFlags::Variadic) {
                self.spread_variadic_tuple_element(ty, ei, &mut out_types, &mut out_infos);
            } else {
                out_types.push(Arc::clone(ty));
                out_infos.push(TupleElementInfo {
                    label: ei.label.clone(),
                    flags: ei.flags,
                    labeled_declaration: ei.labeled_declaration.clone(),
                    type_: None,
                });
            }
        }
        (out_types, out_infos)
    }

    /// 约束域（字面量并集/索引签名）展开为匿名对象
    pub(crate) fn expand_mapped_by_constraint(
        &mut self,
        t: &Arc<Type>,
        m: &crate::checker::types::MappedTypeData,
        constraint: &Arc<Type>,
        chain: &[(Vec<Arc<Type>>, Vec<Arc<Type>>)],
    ) -> Option<Arc<Type>> {
        let constituents: Vec<Arc<Type>> = if constraint.flags.contains(TypeFlags::Union) {
            constraint.types().map(|ts| ts.to_vec()).unwrap_or_default()
        } else {
            vec![Arc::clone(constraint)]
        };
        let mut keys: Vec<Arc<Type>> = Vec::new();
        let mut index_keys: Vec<Arc<Type>> = Vec::new();
        for c in &constituents {
            if c.flags.contains(TypeFlags::Never) || c.flags.contains(TypeFlags::Union) {
                continue;
            }
            if c.flags
                .intersects(TypeFlags::StringLiteral | TypeFlags::NumberLiteral)
            {
                keys.push(Arc::clone(c));
                continue;
            }
            let key = if c.flags.intersects(TypeFlags::Any | TypeFlags::String) {
                self.string_type()
            } else if c.flags.contains(TypeFlags::Number) {
                self.number_type()
            } else if c.flags.intersects(TypeFlags::ESSymbol | TypeFlags::UniqueESSymbol) {
                self.es_symbol_type()
            } else {
                continue;
            };
            if !index_keys.iter().any(|k| Arc::ptr_eq(k, &key)) {
                index_keys.push(key);
            }
        }
        if keys.is_empty() && index_keys.is_empty() {
            return None;
        }
        let is_optional = m
            .declaration
            .as_ref()
            .and_then(|d| match &d.data {
                NodeData::MappedTypeNode(md) => md.question_token.as_ref(),
                _ => None,
            })
            .is_some_and(|tok| tok.kind == tsox_frontend::ast::SyntaxKind::QuestionToken);
        let mut members = tsox_frontend::ast::SymbolTable::new();
        let mut props = Vec::with_capacity(keys.len());
        for key_type in keys {
            let key_name =
                crate::checker::utilities_token_is_identifier_or_keyword::get_property_name_from_type(
                    &key_type,
                );
            let mut prop_type = self.mapped_template_at(m, &key_type, chain);
            if is_optional {
                prop_type = self.get_optional_type(prop_type);
            }
            // `as` 改名（Go getPropertyNameOfMismatchedAccessor 类似：name
            // 型实例化）：name_type 在 K→key 帧下解析，归约为字符串字面量
            let name = self
                .mapped_member_name(m, &key_type, chain)
                .unwrap_or_else(|| key_name.clone());
            let mut flags = tsox_frontend::ast::SymbolFlags::Property;
            if is_optional {
                flags |= tsox_frontend::ast::SymbolFlags::Optional;
            }
            let symbol = Arc::new(Symbol::new(flags, name.clone()));
            self.value_symbol_links.insert(
                &symbol,
                crate::checker::types::ValueSymbolLinks {
                    resolved_type: Some(prop_type),
                    ..Default::default()
                },
            );
            members.insert(name, Arc::clone(&symbol));
            props.push(symbol);
        }
        let mut index_infos = Vec::with_capacity(index_keys.len());
        for key_type in &index_keys {
            let value = self.mapped_template_at(m, key_type, chain);
            index_infos.push(Arc::new(crate::checker::types::IndexInfo {
                key_type: Some(Arc::clone(key_type)),
                value_type: Some(value),
                is_readonly: false,
                declaration: None,
                index_symbol: None,
                components: Vec::new(),
            }));
        }
        let mut obj = Type::new(
            TypeFlags::Object,
            TypeData::Object(crate::checker::types::ObjectTypeData { node: None,
                structured: crate::checker::types::StructuredTypeData {
                    members,
                    properties: props,
                    index_infos,
                    ..Default::default()
                },
                ..Default::default()
            }),
        );
        obj.symbol = t.symbol.clone();
        obj.object_flags |= crate::checker::types::ObjectFlags::Mapped;
        Some(Arc::new(obj))
    }

    /// Go instantiateMappedTypeTemplate：K→key 绑定 + 替换链，解析模板
    pub(crate) fn mapped_template_at(
        &mut self,
        m: &crate::checker::types::MappedTypeData,
        key: &Arc<Type>,
        chain: &[(Vec<Arc<Type>>, Vec<Arc<Type>>)],
    ) -> Arc<Type> {
        let Some(decl) = m.declaration.clone() else {
            return self.any_type();
        };
        let NodeData::MappedTypeNode(md) = &decl.data else {
            return self.any_type();
        };
        let Some(template_node) = md.type_node.clone() else {
            return self.any_type();
        };
        self.resolve_mapped_node(m, &template_node, key, chain)
    }

    /// `as` 改名后的成员名：name_type 节点在 K→key 帧下解析，字符串
    /// 字面量取其值；非字面量（仍泛型）回落原键名
    fn mapped_member_name(
        &mut self,
        m: &crate::checker::types::MappedTypeData,
        key: &Arc<Type>,
        chain: &[(Vec<Arc<Type>>, Vec<Arc<Type>>)],
    ) -> Option<String> {
        let decl = m.declaration.clone()?;
        let NodeData::MappedTypeNode(md) = &decl.data else {
            return None;
        };
        let name_node = md.name_type.clone()?;
        let t = self.resolve_mapped_node(m, &name_node, key, chain);
        match t.literal_value() {
            Some(crate::checker::types::LiteralValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    /// K→key + 替换链帧下解析映射型声明内的类型节点
    fn resolve_mapped_node(
        &mut self,
        m: &crate::checker::types::MappedTypeData,
        node: &Arc<tsox_frontend::ast::Node>,
        key: &Arc<Type>,
        chain: &[(Vec<Arc<Type>>, Vec<Arc<Type>>)],
    ) -> Arc<Type> {
        let decl = m.declaration.clone();
        self.resolve_mapped_decl_node(decl.as_ref(), node, key, chain)
    }

    pub(crate) fn resolve_mapped_decl_node(
        &mut self,
        decl: Option<&Arc<tsox_frontend::ast::Node>>,
        node: &Arc<tsox_frontend::ast::Node>,
        key: &Arc<Type>,
        chain: &[(Vec<Arc<Type>>, Vec<Arc<Type>>)],
    ) -> Arc<Type> {
        let tp_node = decl.and_then(|d| match &d.data {
            NodeData::MappedTypeNode(md) => Some(Arc::clone(&md.type_parameter)),
            _ => None,
        });
        let tp_sym = tp_node.and_then(|tp| self.program.symbol_map().symbol_of(&tp).cloned());
        let mut pushed = 0usize;
        if let Some(sym) = tp_sym.as_ref() {
            let mut mapping = HashMap::new();
            mapping.insert(Arc::as_ptr(sym) as *const Symbol, Arc::clone(key));
            self.type_argument_stack.push(mapping);
            pushed += 1;
        }
        for (ps, ss) in chain.iter().rev() {
            let mut mapping = HashMap::new();
            for (i, p) in ps.iter().enumerate() {
                if let Some(sym) = p.symbol.as_ref() {
                    mapping.insert(
                        Arc::as_ptr(sym) as *const Symbol,
                        Arc::clone(&ss[i.min(ss.len() - 1)]),
                    );
                }
            }
            if !mapping.is_empty() {
                self.type_argument_stack.push(mapping);
                pushed += 1;
            }
        }
        // 模板按词法祖先链解析（Record 的 K/T 在其 TypeAliasDeclaration），
        // 调用现场栈（如 objWrapper 约束解析）含无关同名类型参数，不得泄漏
        let saved_scopes = std::mem::take(&mut self.scope_stack);
        let mut scope_chain: Vec<u64> = Vec::new();
        let mut cur = node.parent();
        while let Some(c) = cur {
            scope_chain.push(c.id());
            cur = c.parent();
        }
        scope_chain.reverse();
        self.scope_stack = scope_chain;
        let resolved = self.get_type_from_type_node(node);
        self.scope_stack = saved_scopes;
        for _ in 0..pushed {
            self.type_argument_stack.pop();
        }
        resolved
    }

    /// 惰性模板解析后的替换链应用（get_template_type_from_mapped_type 调用）
    pub(crate) fn apply_template_subst_chain(
        &mut self,
        t: &Arc<Type>,
        chain: &[(Vec<Arc<Type>>, Vec<Arc<Type>>)],
    ) -> Arc<Type> {
        let mut result = Arc::clone(t);
        for (ps, ss) in chain {
            result = self.substitute_infer_type_parameters(&result, ps, ss);
        }
        result
    }
}
