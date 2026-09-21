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
            let elems: Vec<Arc<Type>> = s
                .as_structured()
                .map(|st| st.properties.iter().map(|p| self.get_type_of_symbol(p)).collect())
                .unwrap_or_default();
            let mut new_elems = Vec::with_capacity(elems.len());
            for (i, _e) in elems.iter().enumerate() {
                let key =
                    self.get_number_literal_type(tsox_core::jsnum::Number::from(i as f64));
                new_elems.push(self.mapped_template_at(m, &key, chain));
            }
            return self.create_tuple_type(new_elems);
        }
        // 对象：按 keyof S 的键展开
        let keys = self.get_index_type(s);
        self.expand_mapped_by_constraint(t, m, &keys, chain)
            .unwrap_or_else(|| Arc::clone(s))
    }

    /// 约束域（字面量并集/索引签名）展开为匿名对象
    pub(crate) fn expand_mapped_by_constraint(
        &mut self,
        _t: &Arc<Type>,
        m: &crate::checker::types::MappedTypeData,
        constraint: &Arc<Type>,
        chain: &[(Vec<Arc<Type>>, Vec<Arc<Type>>)],
    ) -> Option<Arc<Type>> {
        let all_literals = self.union_is_all_string_literals(constraint);
        let keys = self.string_literal_values(constraint);
        if !all_literals || keys.is_empty() {
            // 非字面量域：字符串/数字索引签名形态
            if constraint.flags.intersects(TypeFlags::String | TypeFlags::Number) {
                let key_type = if constraint.flags.contains(TypeFlags::Number) {
                    self.number_type()
                } else {
                    self.string_type()
                };
                let value = self.mapped_template_at(m, &key_type, chain);
                let info = crate::checker::types::IndexInfo {
                    key_type: Some(key_type),
                    value_type: Some(value),
                    is_readonly: false,
                    declaration: None,
                    index_symbol: None,
                    components: Vec::new(),
                };
                let obj = Type::new(
                    TypeFlags::Object,
                    TypeData::Object(crate::checker::types::ObjectTypeData {
                        structured: crate::checker::types::StructuredTypeData {
                            index_infos: vec![Arc::new(info)],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                );
                return Some(Arc::new(obj));
            }
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
        for key in keys {
            let key_type = self.get_string_literal_type(&key);
            let mut prop_type = self.mapped_template_at(m, &key_type, chain);
            if is_optional {
                prop_type = self.get_optional_type(prop_type);
            }
            // `as` 改名（Go getPropertyNameOfMismatchedAccessor 类似：name
            // 型实例化）：name_type 在 K→key 帧下解析，归约为字符串字面量
            let name = self
                .mapped_member_name(m, &key_type, chain)
                .unwrap_or_else(|| key.clone());
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
        let obj = Type::new(
            TypeFlags::Object,
            TypeData::Object(crate::checker::types::ObjectTypeData {
                structured: crate::checker::types::StructuredTypeData {
                    members,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        );
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

    /// 映射型声明节点词法帧下解析其内部类型节点（快路径无 MappedTypeData 时共用）
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
        for (ps, ss) in chain {
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
