#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    pub(crate) fn add_index_signature_member(
        &mut self,
        member: &Arc<Node>,
        index_infos: &mut Vec<Arc<crate::checker::IndexInfo>>,
    ) {
        let NodeData::IndexSignatureDeclaration(data) = &member.data else {
            unreachable!()
        };
        let mut key_type = None;
        let value_type;
        if let Some(param) = data.parameters.iter().next() {
            if let NodeData::ParameterDeclaration(pd) = &param.data {
                key_type = pd
                    .type_node
                    .as_ref()
                    .map(|t| self.get_type_from_type_node(t));
            }
        }
        value_type = Some(self.get_type_from_type_node(&data.type_node));
        let is_readonly = member
            .modifiers()
            .as_ref()
            .is_some_and(|m| m.flags().contains(ModifierFlags::Readonly));
        // Go resolveStructuredTypeMembers：同键索引签名去重（重复声明由
        // 2374 检查报错，类型只保留一个）
        if let Some(k) = &key_type
            && index_infos
                .iter()
                .any(|info| info.key_type.as_ref().is_some_and(|e| e.id == k.id))
        {
            return;
        }
        index_infos.push(Arc::new(crate::checker::IndexInfo {
            key_type,
            value_type,
            is_readonly,
            declaration: Some(Arc::clone(member)),
            index_symbol: None,
            components: Vec::new(),
        }));
    }

    /// Go getTypeOfAccessors 的成员级解析序：getter 注解 → setter 参数注解 →
    /// getter 体返回推断（加宽）；均无则 any
    pub(crate) fn resolve_accessor_pair_type(
        &mut self,
        accessor: &Arc<Node>,
    ) -> Arc<Type> {
        let class = accessor.parent();
        let getter = class.as_ref().and_then(|cls| {
            Self::class_members_of(cls).iter().find(|m| {
                m.kind == SyntaxKind::GetAccessor && Self::member_names_match(m, accessor)
            })
        });
        let setter = class.as_ref().and_then(|cls| {
            Self::class_members_of(cls).iter().find(|m| {
                m.kind == SyntaxKind::SetAccessor && Self::member_names_match(m, accessor)
            })
        });
        if let Some(g) = getter
            && let tsox_frontend::ast::NodeData::GetAccessorDeclaration(gd) = &g.data
            && let Some(tn) = &gd.type_node
        {
            return self.get_type_from_type_node(tn);
        }
        if let Some(s) = setter
            && let tsox_frontend::ast::NodeData::SetAccessorDeclaration(sd) = &s.data
            && let Some(param) = sd.parameters.iter().next()
            && let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &param.data
            && let Some(tn) = &pd.type_node
        {
            return self.get_type_from_type_node(tn);
        }
        if let Some(g) = getter
            && let tsox_frontend::ast::NodeData::GetAccessorDeclaration(gd) = &g.data
            && let Some(body) = &gd.body
        {
            let inferred = self.infer_method_return_type(&Some(Arc::clone(body)));
            return self.get_widened_type(&inferred);
        }
        self.get_any_type()
    }

    /// Go 判定同一成员：标识符按文本、well-known 计算名按内部名
    fn member_names_match(a: &Arc<Node>, b: &Arc<Node>) -> bool {
        let key = |n: &Arc<Node>| -> Option<String> {
            let name = n.name()?;
            match name.kind {
                SyntaxKind::Identifier | SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral => {
                    Some(name.text().to_string())
                }
                SyntaxKind::ComputedPropertyName => {
                    let tsox_frontend::ast::NodeData::ComputedPropertyName(cd) = &name.data
                    else {
                        return None;
                    };
                    crate::binder::symbols_binder_4::well_known_symbol_member_name(&cd.expression)
                }
                _ => None,
            }
        };
        match (key(a), key(b)) {
            (Some(x), Some(y)) => x == y,
            _ => false,
        }
    }

    pub(crate) fn add_get_accessor_member(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
    ) {
        let NodeData::GetAccessorDeclaration(data) = &member.data else {
            unreachable!()
        };
        if is_static_modifier(&data.modifiers) {
            return;
        }
        let name = self.member_declaration_name(&data.name);
        if name.is_empty() {
            return;
        }

        let prop_type = self.resolve_accessor_pair_type(member);
        match symbol_table.get(&name).cloned() {
            Some(existing) => {
                let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                unsafe {
                    (*existing_mut).flags |= SymbolFlags::GetAccessor;
                    (*existing_mut).declarations.push(Arc::clone(member));
                }
                self.value_symbol_links.insert(
                    &existing,
                    ValueSymbolLinks {
                        resolved_type: Some(prop_type),
                        ..Default::default()
                    },
                );
            }
            None => {
                let mut symbol = Symbol::new(
                    SymbolFlags::Property | SymbolFlags::GetAccessor,
                    name.clone(),
                );
                symbol.declarations.push(Arc::clone(member));
                let symbol = Arc::new(symbol);
                self.value_symbol_links.insert(
                    &symbol,
                    ValueSymbolLinks {
                        resolved_type: Some(prop_type),
                        ..Default::default()
                    },
                );
                symbol_table.insert(name, Arc::clone(&symbol));
                props.push(symbol);
            }
        }
    }

    pub(crate) fn add_set_accessor_member(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
    ) {
        let NodeData::SetAccessorDeclaration(data) = &member.data else {
            unreachable!()
        };
        if is_static_modifier(&data.modifiers) {
            return;
        }
        let name = self.member_declaration_name(&data.name);
        if name.is_empty() {
            return;
        }

        let prop_type = data
            .parameters
            .iter()
            .next()
            .and_then(|p| {
                if let NodeData::ParameterDeclaration(pd) = &p.data {
                    pd.type_node
                        .as_ref()
                        .map(|tn| self.get_type_from_type_node(tn))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| self.get_any_type());
        match symbol_table.get(&name).cloned() {
            Some(existing) => {
                let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                unsafe {
                    (*existing_mut).flags |= SymbolFlags::SetAccessor;
                    (*existing_mut).declarations.push(Arc::clone(member));
                }
                // getter 已建型则不覆盖（Go 解析序 getter 优先）；
                // setter 单独存在时用其参数注解
                let already_typed = self
                    .value_symbol_links
                    .get(&existing)
                    .and_then(|l| l.resolved_type.clone())
                    .is_some_and(|t| !t.flags.contains(TypeFlags::Any));
                if !already_typed {
                    let prop_type = self.resolve_accessor_pair_type(member);
                    self.value_symbol_links.insert(
                        &existing,
                        ValueSymbolLinks {
                            resolved_type: Some(prop_type),
                            ..Default::default()
                        },
                    );
                }
            }
            None => {
                let mut symbol = Symbol::new(
                    SymbolFlags::Property | SymbolFlags::SetAccessor,
                    name.clone(),
                );
                symbol.declarations.push(Arc::clone(member));
                let symbol = Arc::new(symbol);
                self.value_symbol_links.insert(
                    &symbol,
                    ValueSymbolLinks {
                        resolved_type: Some(prop_type),
                        ..Default::default()
                    },
                );
                symbol_table.insert(name, Arc::clone(&symbol));
                props.push(symbol);
            }
        }
    }

    pub(crate) fn add_call_signature_member(
        &mut self,
        member: &Arc<Node>,
        call_signatures: &mut Vec<Arc<Signature>>,
    ) {
        let NodeData::CallSignatureDeclaration(data) = &member.data else {
            unreachable!()
        };
        let suppress = self
            .current_file
            .as_ref()
            .is_some_and(|f| f.file_name.starts_with("bundled://"));
        if suppress {
            self.push_ts2304_suppression();
        }

        self.push_scope(member);
        let return_type = match data.type_node.as_ref() {
            Some(tn) => self.get_type_from_type_node(tn),
            None => self.get_any_type(),
        };
        let sig = self.build_signature_from_function_like_type_node(
            &data.parameters,
            return_type,
            false,
            None,
            Some(Arc::clone(member)),
        );
        self.pop_scope();
        if suppress {
            self.pop_ts2304_suppression();
        }
        call_signatures.push(sig);
    }

    pub(crate) fn add_construct_signature_member(
        &mut self,
        member: &Arc<Node>,
        construct_signatures: &mut Vec<Arc<Signature>>,
    ) {
        let NodeData::ConstructSignatureDeclaration(data) = &member.data else {
            unreachable!()
        };
        let suppress = self
            .current_file
            .as_ref()
            .is_some_and(|f| f.file_name.starts_with("bundled://"));
        if suppress {
            self.push_ts2304_suppression();
        }

        self.push_scope(member);
        let return_type = match data.type_node.as_ref() {
            Some(tn) => self.get_type_from_type_node(tn),
            None => self.get_any_type(),
        };
        let sig = self.build_signature_from_function_like_type_node(
            &data.parameters,
            return_type,
            true,
            None,
            Some(Arc::clone(member)),
        );
        self.pop_scope();
        if suppress {
            self.pop_ts2304_suppression();
        }
        construct_signatures.push(sig);
    }

    pub(crate) fn add_constructor_properties(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
    ) {
        let NodeData::ConstructorDeclaration(data) = &member.data else {
            unreachable!()
        };
        for param in data.parameters.iter() {
            let NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };
            if pd.name.kind != SyntaxKind::Identifier {
                continue;
            }
            let Some(modifiers) = &pd.modifiers else {
                continue;
            };
            if !modifiers.modifier_flags.intersects(
                ModifierFlags::Public
                    | ModifierFlags::Private
                    | ModifierFlags::Protected
                    | ModifierFlags::Readonly,
            ) {
                continue;
            }
            let name = pd.name.text().to_string();
            if name.is_empty() || symbol_table.get(&name).is_some() {
                continue;
            }
            let prop_type = match pd.type_node.as_ref() {
                Some(tn) => self.get_type_from_type_node(tn),
                None => match pd.initializer.as_ref() {
                    Some(init) => self.get_type_of_node(init),
                    None => self.get_any_type(),
                },
            };
            // Go bindParameter+getTypeForVariableLikeDeclaration：可选参数属性
            // 符号带 Optional 位，strictNullChecks 下属性类型补 | undefined
            let is_optional = pd.question_token.is_some();
            let prop_type = if is_optional && self.strict_null_checks {
                self.get_union_type(vec![prop_type, self.undefined_type()])
            } else {
                prop_type
            };
            let mut flags = SymbolFlags::Property;
            if is_optional {
                flags |= SymbolFlags::Optional;
            }
            let mut symbol = Symbol::new(flags, name.clone());

            symbol.declarations.push(Arc::clone(param));
            if modifiers.modifier_flags.contains(ModifierFlags::Readonly) {
                symbol.check_flags |= CheckFlags::Readonly;
            }
            let symbol = Arc::new(symbol);
            self.value_symbol_links.insert(
                &symbol,
                ValueSymbolLinks {
                    resolved_type: Some(prop_type),
                    ..Default::default()
                },
            );
            symbol_table.insert(name, Arc::clone(&symbol));
            props.push(symbol);
        }
    }
}
