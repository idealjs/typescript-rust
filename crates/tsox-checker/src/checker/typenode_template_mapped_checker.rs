#![allow(unused_imports)]
use std::cell::Cell;

use crate::checker::typenode_template_mapped::*;

impl Checker {
    pub(crate) fn build_template_literal_type(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("build_template_literal_type"); 
        let (head, spans) = match &node.data {
            NodeData::TemplateLiteralTypeNode(data) => {
                (Arc::clone(&data.head), Arc::clone(&data.template_spans))
            }
            _ => return self.error_type(),
        };
        let head_text = template_token_text(&head);

        let mut span_types: Vec<Arc<Type>> = Vec::new();
        let mut span_texts: Vec<String> = Vec::new();
        for span_node in spans.iter() {
            let (type_node, literal_node) = match &span_node.data {
                NodeData::TemplateLiteralTypeSpan(data) => {
                    (Arc::clone(&data.type_node), Arc::clone(&data.literal))
                }
                _ => return self.error_type(),
            };
            span_types.push(self.get_type_from_type_node(&type_node));
            span_texts.push(template_token_text(&literal_node));
        }

        if span_types
            .iter()
            .any(|t| t.flags.contains(TypeFlags::Never) || matches!(&t.data, TypeData::Union(_)))
            && !self.check_cross_product_union(node, &span_types)
        {
            return self.error_type();
        }

        let all_literal = span_types.iter().all(|t| {
            t.flags
                .intersects(TYPE_FLAGS_LITERAL | TypeFlags::Null | TypeFlags::Undefined)
        });
        if all_literal {
            let mut sb = String::new();
            sb.push_str(&head_text);
            for (t, text) in span_types.iter().zip(span_texts.iter()) {
                sb.push_str(&self.template_string_for_type(t));
                sb.push_str(text);
            }
            return self
                .get_string_literal_type(&crate::checker::string_surrogate::combine_surrogate_pairs(&sb));
        }

        let mut texts = Vec::with_capacity(span_types.len() + 1);
        texts.push(head_text);
        for t in span_texts {
            texts.push(t);
        }
        Arc::new(Type::new(
            TypeFlags::TemplateLiteral,
            TypeData::TemplateLiteral(TemplateLiteralTypeData {
                constrained: ConstrainedTypeData::default(),
                texts,
                types: span_types,
            }),
        ))
    }

    pub(crate) fn template_string_for_type(&self, t: &Arc<Type>) -> String { ::tsox_core::fntrace::enter("template_string_for_type"); 
        if t.flags.contains(TypeFlags::StringLiteral) {
            if let TypeData::Literal(lit) = &t.data {
                if let LiteralValue::String(s) = &lit.value {
                    return s.clone();
                }
            }
            return String::new();
        }
        if t.flags.contains(TypeFlags::NumberLiteral) {
            if let TypeData::Literal(lit) = &t.data {
                if let LiteralValue::Number(n) = &lit.value {
                    return n.to_string();
                }
            }
            return String::new();
        }
        if t.flags.contains(TypeFlags::BooleanLiteral) {
            if let TypeData::Literal(lit) = &t.data {
                if let LiteralValue::Boolean(b) = &lit.value {
                    return if *b { "true".into() } else { "false".into() };
                }
            }
            return String::new();
        }
        if t.flags.contains(TypeFlags::Null) {
            return "null".into();
        }
        if t.flags.contains(TypeFlags::Undefined) {
            return "undefined".into();
        }
        String::new()
    }

    pub(crate) fn build_mapped_type(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("build_mapped_type"); 
        self.push_scope(node);
        let result = self.build_mapped_type_inner(node);
        self.pop_scope();
        result
    }

    pub(crate) fn build_mapped_type_inner(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("build_mapped_type_inner"); 
        let data = match &node.data {
            NodeData::MappedTypeNode(data) => data,
            _ => return self.error_type(),
        };

        let constraint_node = match &data.type_parameter.data {
            NodeData::TypeParameterDeclaration(tp) => match &tp.constraint {
                Some(c) => Arc::clone(c),
                None => return self.error_type(),
            },
            _ => return self.error_type(),
        };
        let constraint_type = self.get_type_from_type_node(&constraint_node);
        // Go checkMappedType（checker.go:3516-3533）：name type（as 子句）或
        // 约束型须可赋给 string | number | symbol，锚点分别取 NameType /
        // 类型参数约束节点（本节点经 type_node 缓存只解析一次）
        {
            let string_number_symbol = self.string_number_symbol_type();
            match data.name_type.clone() {
                Some(nt) => {
                    let name_type = self.get_type_from_type_node(&nt);
                    self.check_type_assignable_to(&name_type, &string_number_symbol, Some(&nt), None);
                }
                None => {
                    self.check_type_assignable_to(
                        &constraint_type,
                        &string_number_symbol,
                        Some(&constraint_node),
                        None,
                    );
                }
            }
        }
        let mapped_symbol = self.program.symbol_map().symbol_of(node).map(Arc::clone);

        if data.type_node.is_none()
            && self.no_implicit_any
            && self
                .current_file
                .as_ref()
                .is_some_and(|f| !f.file_name.starts_with("bundled://"))
        {
            let file = self.current_file.clone();
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                node.loc,
                tsox_core::diagnostics::messages_generated::
                    MAPPED_OBJECT_TYPE_IMPLICITLY_HAS_AN_ANY_TEMPLATE_TYPE,
                Vec::new(),
            ));
        }

        let mut keys: Vec<Arc<Type>> = Vec::new();
        let mut constraint_all_literals = true;
        match &constraint_type.data {
            TypeData::Union(u) => {
                for m in &u.union_or_intersection.types {
                    if m.flags
                        .intersects(TypeFlags::StringLiteral | TypeFlags::NumberLiteral)
                    {
                        keys.push(Arc::clone(m));
                    } else {
                        constraint_all_literals = false;
                    }
                }
            }
            _ => {
                if constraint_type
                    .flags
                    .intersects(TypeFlags::StringLiteral | TypeFlags::NumberLiteral)
                {
                    keys.push(Arc::clone(&constraint_type));
                } else {
                    constraint_all_literals = false;
                }
            }
        }
        if keys.is_empty() || !constraint_all_literals {
            // Go getTypeParameterFromMappedType：类型参数按声明符号取型
            // （get_type_from_type_node 对 TypeParameter 声明节点无臂，会落 error）
            let tp_type = self
                .program
                .symbol_map()
                .symbol_of(&data.type_parameter)
                .cloned()
                .map(|sym| self.get_type_parameter_from_symbol(&sym))
                .unwrap_or_else(|| self.error_type());
            // Go createMappedType：模板按需解析（getTemplateTypeFromMappedType），
            // 泛型约束下急切解析嵌套别名（Deep<T[K]>）会互递归
            let name_type = data
                .name_type
                .as_ref()
                .map(|n| self.get_type_from_type_node(n));
            let constraint_has_free_type_parameter = type_mentions_type_parameter(&constraint_type);
            let mut mapped_type = Arc::new(Type {
                flags: TypeFlags::Object,
                object_flags: crate::checker::types::ObjectFlags::Mapped,
                id: crate::checker::types::next_type_id(),
                symbol: mapped_symbol.clone(),
                alias: None,
                data: TypeData::Mapped(MappedTypeData {
                    object: ObjectTypeData { node: None,
                        structured: StructuredTypeData::default(),
                        ..Default::default()
                    },
                    declaration: Some(Arc::clone(node)),
                    type_parameter: Some(tp_type),
                    constraint_type: Some(constraint_type),
                    name_type,
                    template_type: None,
                    template_node: data.type_node.clone(),
                    template_subst: None,
                    modifiers_type: None,
                    resolved_apparent_type: OnceLock::new(),
                    contains_error: std::sync::atomic::AtomicBool::new(false),
                }),
            });
            // Go resolveStructuredTypeMembers（checker.go:20185）Mapped 臂在查询前
            // 经 resolveMappedTypeMembers 解析 keyof 约束的成员与索引信息；成员
            // 仅 Arc 独占时可写，等价时点是创建处，写回直接改本 Arc
            //（set_structured_type_members 先克隆再 get_mut 恒失败，不可用）。
            // 约束含游离类型参数的泛型映射维持惰性
            if !constraint_has_free_type_parameter
                && Checker::is_mapped_type_with_keyof_constraint_declaration(&mapped_type)
            {
                let (members, index_infos) = self.compute_mapped_type_members(&mapped_type);
                if let Some(mt) = Arc::get_mut(&mut mapped_type) {
                    let properties: Vec<Arc<Symbol>> = members
                        .iter()
                        .filter(|(_, s)| {
                            !crate::checker::utilities_is_optional_symbol::is_reserved_member_name(
                                &s.name,
                            )
                        })
                        .map(|(_, s)| Arc::clone(s))
                        .collect();
                    mt.object_flags |= crate::checker::types::ObjectFlags::MembersResolved;
                    if let TypeData::Mapped(m) = &mut mt.data {
                        m.object.structured.members = members;
                        m.object.structured.properties = properties;
                        m.object.structured.index_infos = index_infos;
                    }
                }
            }
            return mapped_type;
        }

        let tp_symbol = self
            .program
            .symbol_map()
            .symbol_of(&data.type_parameter)
            .map(Arc::clone);
        let tp_key = tp_symbol
            .as_ref()
            .map(|s| Arc::as_ptr(s) as *const tsox_frontend::ast::Symbol);

        let is_optional = data
            .question_token
            .as_ref()
            .map(|t| t.kind == SyntaxKind::QuestionToken)
            .unwrap_or(false);

        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        for key_type in &keys {
            let key_name =
                crate::checker::utilities_token_is_identifier_or_keyword::get_property_name_from_type(
                    key_type,
                );
            // Go addMemberForKeyType（checker.go:22158-22162）：无 as 子句时
            // propNameType = keyType，有 as 子句时 = 名节点按 key 实例化后的
            // 名型；属性符号 links.nameType = propNameType（checker.go:22172）
            // ——显示侧 getPropertyNameNodeForSymbolFromNameType 枚举臂
            //（nodebuilderimpl.go:2627-2642）据此呈计算属性名 [E.A]
            let (name, prop_name_type) = match &data.name_type {
                Some(name_node) => {
                    let t = self.resolve_mapped_decl_node(Some(node), name_node, key_type, &[]);
                    let name = match t.literal_value() {
                        Some(LiteralValue::String(s)) => s.clone(),
                        _ => key_name.clone(),
                    };
                    (name, Some(t))
                }
                None => (key_name.clone(), Some(Arc::clone(key_type))),
            };
            let mut prop_type = match &data.type_node {
                Some(tn) => {
                    if let Some(k) = tp_key {
                        let mut mapping = HashMap::new();
                        mapping.insert(k, Arc::clone(key_type));
                        self.type_argument_stack.push(mapping);
                    }
                    let t = self.get_type_from_type_node(tn);
                    if tp_key.is_some() {
                        self.type_argument_stack.pop();
                    }
                    t
                }
                None => self.get_any_type(),
            };
            if is_optional {
                prop_type = self.get_optional_type(prop_type, true);
            }
            let mut flags = SymbolFlags::Property;
            if is_optional {
                flags |= SymbolFlags::Optional;
            }
            let symbol = Arc::new(Symbol::new(flags, name.clone()));
            self.value_symbol_links.insert(
                &symbol,
                ValueSymbolLinks {
                    resolved_type: Some(prop_type),
                    name_type: prop_name_type,
                    ..Default::default()
                },
            );
            symbol_table.insert(name.clone(), Arc::clone(&symbol));
            props.push(symbol);
        }
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: mapped_symbol,
            alias: None,
            data: TypeData::Object(ObjectTypeData { node: None,
                structured: StructuredTypeData {
                    members: symbol_table,
                    properties: props,
                    index_infos: Vec::new(),
                    signatures: Vec::new(),
                    call_signature_count: 0,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

    pub(crate) fn string_literal_values(&self, t: &Arc<Type>) -> Vec<String> { ::tsox_core::fntrace::enter("string_literal_values"); 
        if t.flags.contains(TypeFlags::Never) {
            return Vec::new();
        }
        if t.flags.contains(TypeFlags::StringLiteral) {
            if let TypeData::Literal(lit) = &t.data {
                if let LiteralValue::String(s) = &lit.value {
                    return vec![s.clone()];
                }
            }
            return Vec::new();
        }
        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .flat_map(|c| self.string_literal_values(c))
                    .collect();
            }
        }
        Vec::new()
    }

    pub(crate) fn union_is_all_string_literals(&self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("union_is_all_string_literals"); 
        if t.flags.contains(TypeFlags::StringLiteral) {
            return true;
        }
        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                return u
                    .union_or_intersection
                    .types
                    .iter()
                    .all(|c| self.union_is_all_string_literals(c));
            }
        }
        false
    }

    pub(crate) fn add_optionality(&self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("add_optionality");
        if self.strict_null_checks {
            self.make_union_two(Arc::clone(t), self.undefined_type())
        } else {
            Arc::clone(t)
        }
    }
}

pub(crate) fn type_mentions_type_parameter(t: &Arc<Type>) -> bool {
    if t.flags.contains(TypeFlags::TypeParameter) {
        return true;
    }
    let mut children: Vec<Arc<Type>> = Vec::new();
    match &t.data {
        TypeData::Union(u) => children.extend(u.union_or_intersection.types.iter().cloned()),
        TypeData::Intersection(i) => {
            children.extend(i.union_or_intersection.types.iter().cloned())
        }
        TypeData::IndexedAccess(ia) => {
            children.extend(ia.object_type.iter().cloned());
            children.extend(ia.index_type.iter().cloned());
        }
        TypeData::Index(i) => children.extend(i.target.iter().cloned()),
        TypeData::Conditional(c) => {
            children.extend(c.check_type.iter().cloned());
            children.extend(c.extends_type.iter().cloned());
            children.extend(c.resolved_true_type.get().cloned());
            children.extend(c.resolved_false_type.get().cloned());
        }
        TypeData::TemplateLiteral(tl) => children.extend(tl.types.iter().cloned()),
        TypeData::StringMapping(sm) => children.extend(sm.target.iter().cloned()),
        TypeData::Substitution(s) => children.extend(s.base_type.iter().cloned()),
        TypeData::Mapped(m) => {
            children.extend(m.type_parameter.iter().cloned());
            children.extend(m.constraint_type.iter().cloned());
            children.extend(m.name_type.iter().cloned());
            children.extend(m.template_type.iter().cloned());
        }
        // Go 实例化壳 mapper 无条件挂载（instantiateAnonymousType
        // checker.go:23794 d.mapper=m），别名/接口引用实参经 instantiateTypeWorker
        // 逐层实例化（checker.go:23529），自由类型参数可藏于实参任意深度；
        // 不透视为泛型即按具体键解析成员（Go resolveMappedTypeMembers
        // checker.go:22204 keyof 约束只产字面量键成员，泛型键无属性成员）。
        // target 是别名体自带形参位（已由实参代换），不构成游离性，不走
        TypeData::Object(o) => {
            children.extend(o.type_arguments.iter().cloned());
            if let Some(alias) = t.alias.as_ref() {
                children.extend(alias.type_arguments.iter().cloned());
            }
        }
        _ => {}
    }
    children
        .into_iter()
        .any(|c| type_mentions_type_parameter(&c))
}
