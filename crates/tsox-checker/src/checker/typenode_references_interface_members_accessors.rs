#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    pub(crate) fn add_index_signature_member(
        &mut self,
        member: &Arc<Node>,
        index_infos: &mut Vec<Arc<crate::checker::IndexInfo>>,
    ) { ::tsox_core::fntrace::enter("add_index_signature_member"); 
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
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("resolve_accessor_pair_type"); 
        self.push_scope(accessor);
        let result = self.resolve_accessor_pair_type_worker(accessor);
        self.pop_scope();
        result
    }

    fn resolve_accessor_pair_type_worker(
        &mut self,
        accessor: &Arc<Node>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("resolve_accessor_pair_type_worker"); 
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
            let accessor = g.clone();
            // Go getTypeOfAccessors 的体推断同步执行（非 checkNodeDeferred）：
            // 体期 this.x 重入命中本符号帧计环（TS7023），不开 rt_infer 豁免界
            let saved_depth = self.call_return_query_depth;
            self.call_return_query_depth = self.call_return_query_depth.saturating_add(1);
            let inferred = self.infer_method_return_type(&accessor, &Some(Arc::clone(body)));
            self.call_return_query_depth = saved_depth;
            return self.get_widened_type(&inferred);
        }
        self.get_any_type()
    }

    /// Go 判定同一成员：标识符按文本、well-known 计算名按内部名
    fn member_names_match(a: &Arc<Node>, b: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("member_names_match"); 
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
        deferred: &mut Vec<Arc<Node>>,
    ) { ::tsox_core::fntrace::enter("add_get_accessor_member"); 
        let NodeData::GetAccessorDeclaration(data) = &member.data else {
            unreachable!()
        };
        if is_static_modifier(&data.modifiers) {
            return;
        }
        let name = self.member_declaration_name(&data.name);
        // Go declareSymbolEx：__missing 名成员不入容器符号表
        if name == crate::binder::symbols::INTERNAL_SYMBOL_NAME_MISSING {
            return;
        }
        if name.is_empty() {
            return;
        }

        match symbol_table.get(&name).cloned() {
            Some(existing) => {
                let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                unsafe {
                    (*existing_mut).flags |= SymbolFlags::GetAccessor;
                    (*existing_mut).declarations.push(Arc::clone(member));
                }
            }
            None => {
                let mut symbol = Symbol::new(
                    SymbolFlags::Property | SymbolFlags::GetAccessor,
                    name.clone(),
                );
                symbol.declarations.push(Arc::clone(member));
                let symbol = Arc::new(symbol);
                symbol_table.insert(name, Arc::clone(&symbol));
                props.push(symbol);
            }
        }
        deferred.push(Arc::clone(member));
    }

    pub(crate) fn add_set_accessor_member(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
        deferred: &mut Vec<Arc<Node>>,
    ) { ::tsox_core::fntrace::enter("add_set_accessor_member"); 
        let NodeData::SetAccessorDeclaration(data) = &member.data else {
            unreachable!()
        };
        if is_static_modifier(&data.modifiers) {
            return;
        }
        let name = self.member_declaration_name(&data.name);
        // Go declareSymbolEx：__missing 名成员不入容器符号表
        if name == crate::binder::symbols::INTERNAL_SYMBOL_NAME_MISSING {
            return;
        }
        if name.is_empty() {
            return;
        }

        match symbol_table.get(&name).cloned() {
            Some(existing) => {
                let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                unsafe {
                    (*existing_mut).flags |= SymbolFlags::SetAccessor;
                    (*existing_mut).declarations.push(Arc::clone(member));
                }
            }
            None => {
                let mut symbol = Symbol::new(
                    SymbolFlags::Property | SymbolFlags::SetAccessor,
                    name.clone(),
                );
                symbol.declarations.push(Arc::clone(member));
                let symbol = Arc::new(symbol);
                symbol_table.insert(name, Arc::clone(&symbol));
                props.push(symbol);
            }
        }
        deferred.push(Arc::clone(member));
    }

    // Go resolveObjectTypeMembers：accessor 成员型在全成员入表后解析，
    // 体推断期的 this.x 自引用（多态 this 约束即本壳）才可见
    pub(crate) fn resolve_deferred_accessor_member_types(
        &mut self,
        deferred: &[Arc<Node>],
        symbol_table: &SymbolTable,
    ) { ::tsox_core::fntrace::enter("resolve_deferred_accessor_member_types"); 
        for member in deferred {
            let name = match &member.data {
                NodeData::GetAccessorDeclaration(d) => self.member_declaration_name(&d.name),
                NodeData::SetAccessorDeclaration(d) => self.member_declaration_name(&d.name),
                _ => continue,
            };
            let Some(sym) = symbol_table.get(&name) else {
                continue;
            };
            if self
                .value_symbol_links
                .get(sym)
                .and_then(|l| l.resolved_type.clone())
                .is_some()
            {
                continue;
            }
            let in_type_literal = member
                .parent()
                .is_some_and(|p| p.kind == SyntaxKind::TypeLiteral);
            // 类/接口成员经符号定型（accessor 臂 push/pop 帧闭环，体推断重入
            // 计环报 TS7023）；TypeLiteral 成员维持字面量局部解析
            let prop_type = if in_type_literal {
                self.accessor_member_prop_type(member, &name, true)
            } else {
                self.get_type_of_symbol(sym)
            };
            self.value_symbol_links
                .get_or_default(sym)
                .resolved_type = Some(prop_type);
        }
    }

    fn accessor_member_prop_type(
        &mut self,
        member: &Arc<Node>,
        name: &str,
        in_type_literal: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("accessor_member_prop_type"); 
        if in_type_literal {
            self.type_literal_accessor_member_type(member, name)
        } else {
            self.resolve_accessor_pair_type(member)
        }
    }

    pub(crate) fn add_call_signature_member(
        &mut self,
        member: &Arc<Node>,
        call_signatures: &mut Vec<Arc<Signature>>,
    ) { ::tsox_core::fntrace::enter("add_call_signature_member"); 
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
    ) { ::tsox_core::fntrace::enter("add_construct_signature_member"); 
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
    ) { ::tsox_core::fntrace::enter("add_constructor_properties"); 
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
