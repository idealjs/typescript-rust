#![allow(unused_imports)]

use crate::checker::typenode_references::*;

fn type_node_contains_type_reference(node: &Arc<Node>) -> bool {
    match &node.data {
        NodeData::TypeReferenceNode(_) | NodeData::MappedTypeNode(_) => true,
        NodeData::UnionTypeNode(d) => d.types.iter().any(type_node_contains_type_reference),
        NodeData::IntersectionTypeNode(d) => {
            d.types.iter().any(type_node_contains_type_reference)
        }
        NodeData::ParenthesizedTypeNode(d) => type_node_contains_type_reference(&d.type_node),
        NodeData::TypeOperatorNode(d) => type_node_contains_type_reference(&d.type_node),
        _ => false,
    }
}

impl Checker {
    pub(crate) fn add_property_signature_member(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
        own_symbol: Option<&Arc<Symbol>>,
    ) { ::tsox_core::fntrace::enter("add_property_signature_member"); 
        let NodeData::PropertySignatureDeclaration(data) = &member.data else {
            unreachable!()
        };
        let name = self.member_declaration_name(&data.name);
        // Go declareSymbolEx：__missing 名成员不入容器符号表
        if name == crate::binder::symbols::INTERNAL_SYMBOL_NAME_MISSING {
            return;
        }
        // 空串字面量名 `"": any` 是合法属性；仅计算属性取名失败才跳过
        if name.is_empty() && matches!(&data.name.data, NodeData::ComputedPropertyName(_)) {
            return;
        }
        // Go declareSymbolEx（binder.go:294）：同名成员冲突报错后表保留首符号，
        // 后声明另建孤儿符号不入表；properties 从表投影（checker.go:23345
        // getNamedMembers）故同名属性只保留首声明一项，类型节点仍求值以暴露声明内错误
        if symbol_table.get(&name).is_some() {
            if !self.property_signature_type_deferred(&data.type_node, own_symbol) {
                self.get_type_from_type_node(&data.type_node);
            }
            return;
        }
        let is_optional = data
            .postfix_token
            .as_ref()
            .map(|t| t.kind == SyntaxKind::QuestionToken)
            .unwrap_or(false);

        let resolved_type = if self.property_signature_type_deferred(&data.type_node, own_symbol) {
            None
        } else {
            let mut prop_type = self.get_type_from_type_node(&data.type_node);
            // Go addOptionalityEx（checker.go:17671→m1a 同名）：strictNullChecks
            // 下 optional 声明型才并 undefined，getOptionalType 自身
            // debug.Assert(strictNullChecks)（checker.go:19740）；非 strict
            // 程序的接口成员（lib 的 IteratorYieldResult.done?: false 等）
            // 型为注解本体，多并的 undefined 经判别式组合计数/属性推断
            // （getNonMissingTypeOfSymbol 原样消费）污染判定
            if is_optional && self.strict_null_checks {
                prop_type = self.get_optional_type(prop_type, true);
            }
            if crate::checker::utilities::is_type_error(&prop_type) {
                None
            } else {
                Some(prop_type)
            }
        };

        let mut flags = SymbolFlags::Property;
        if is_optional {
            flags |= SymbolFlags::Optional;
        }
        let mut symbol = Symbol::new(flags, name.clone());

        symbol.declarations = vec![Arc::clone(member)];
        symbol.value_declaration = Some(Arc::clone(member));

        if let Some(m) = &data.modifiers {
            if m.modifier_flags.contains(ModifierFlags::Readonly) {
                symbol.check_flags |= CheckFlags::Readonly;
            }
        }
        let symbol = Arc::new(symbol);
        self.value_symbol_links.insert(
            &symbol,
            ValueSymbolLinks {
                resolved_type,
                ..Default::default()
            },
        );
        symbol_table.insert(name, Arc::clone(&symbol));
        props.push(symbol);
    }

    fn property_signature_type_deferred(
        &self,
        type_node: &Arc<Node>,
        own_symbol: Option<&Arc<Symbol>>,
    ) -> bool { ::tsox_core::fntrace::enter("property_signature_type_deferred");
        if self.variable_type_frame_depth > 0 {
            return true;
        }
        type_node_contains_type_reference(type_node)
            && own_symbol.is_some()
            && (self.type_argument_stack.is_empty() || {
                own_symbol.is_some_and(|sym| {
                    self.is_resolving(
                        Arc::as_ptr(sym) as *const tsox_frontend::ast::Symbol,
                        crate::checker::TypeResolutionProperty::DeclaredType,
                    )
                })
            })
    }

    pub(crate) fn add_method_signature_member(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
    ) { ::tsox_core::fntrace::enter("add_method_signature_member");
        // 带实参接口 pass 的实参映射帧活跃（resolve_interface_type_ex(Some(args))）：
        // 本 pass 产出的成员符号是实例化副本，不得接管声明符号的合并指向——
        // 副本成员型经帧代入（如 Success 条件式 extends Micro<infer _A,_E,_R>
        // 的帧把 [Symbol.iterator] 返回型实参代成 infer 型参），声明位经
        // get_merged_symbol 取到副本即把代入结果驻留成声明成员型（Go
        // instantiateSymbol 副本只挂在实例类型的成员表，声明符号恒指声明成员）
        let in_instantiation_pass = !self.type_argument_stack.is_empty();
        let NodeData::MethodSignatureDeclaration(data) = &member.data else {
            unreachable!()
        };
        let name = self.member_declaration_name(&data.name);
        // Go declareSymbolEx：__missing 名成员不入容器符号表
        if name == crate::binder::symbols::INTERNAL_SYMBOL_NAME_MISSING {
            return;
        }
        // 空串字面量名 `"": any` 是合法属性；仅计算属性取名失败才跳过
        if name.is_empty() && matches!(&data.name.data, NodeData::ComputedPropertyName(_)) {
            return;
        }

        let decl_symbol = {
            let sym_map = self.program.symbol_map();
            sym_map.symbol_of(member).cloned()
        };

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

        if let Some(existing) = symbol_table.get(&name).cloned() {
            if !in_instantiation_pass
                && let Some(decl_symbol) = decl_symbol.as_ref()
            {
                self.record_merged_symbol_if_absent(&existing, decl_symbol);
            }
            let existing_type = self
                .value_symbol_links
                .get(&existing)
                .and_then(|l| l.resolved_type.clone());
            let merged_sigs = existing_type
                .as_ref()
                .and_then(|t| t.as_structured().map(|s| s.call_signatures().to_vec()))
                .unwrap_or_default();
            let mut all_sigs = merged_sigs;
            all_sigs.push(sig);
            let fn_type = self.create_function_or_constructor_type_ex(
                all_sigs,
                false,
                Some(Arc::clone(&existing)),
            );
            self.value_symbol_links.insert(
                &existing,
                ValueSymbolLinks {
                    resolved_type: Some(fn_type),
                    ..Default::default()
                },
            );
            return;
        }
        let mut flags = SymbolFlags::Property | SymbolFlags::Method;
        if data
            .postfix_token
            .as_ref()
            .is_some_and(|t| t.kind == SyntaxKind::QuestionToken)
        {
            flags |= SymbolFlags::Optional;
        }
        let mut symbol = Symbol::new(flags, name.clone());
        symbol.declarations.push(Arc::clone(&member));
        symbol.value_declaration = Some(Arc::clone(&member));
        let symbol = Arc::new(symbol);
        if !in_instantiation_pass
            && let Some(decl_symbol) = decl_symbol.as_ref()
        {
            self.record_merged_symbol_if_absent(&symbol, decl_symbol);
        }
        let fn_type =
            self.create_function_or_constructor_type_ex(vec![sig], false, Some(Arc::clone(&symbol)));
        self.value_symbol_links.insert(
            &symbol,
            ValueSymbolLinks {
                resolved_type: Some(fn_type),
                ..Default::default()
            },
        );
        symbol_table.insert(name, Arc::clone(&symbol));
        props.push(symbol);
    }

    pub(crate) fn add_property_declaration_member(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
    ) { ::tsox_core::fntrace::enter("add_property_declaration_member"); 
        let NodeData::PropertyDeclaration(data) = &member.data else {
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
        // 空串字面量名 `"": any` 是合法属性；仅计算属性取名失败才跳过
        if name.is_empty() && matches!(&data.name.data, NodeData::ComputedPropertyName(_)) {
            return;
        }
        // Go declareSymbolEx（binder.go:294）：同名成员冲突报错后表保留首符号，
        // 后声明符号不入表/properties（checker.go:23345 getNamedMembers 表投影），
        // 类型注解节点仍求值以暴露声明内错误
        if symbol_table.get(&name).is_some() {
            if let Some(tn) = &data.type_node {
                self.get_type_from_type_node(tn);
            }
            return;
        }
        // Go resolveDeclaredMembers（checker.go:20777-20790）：成员解析只建符号
        // 表，不计算属性类型——属性型经 getTypeOfSymbol 惰性求值
        //（checkDeclarationInitializer 时点在成员表完成后，前向 this.X 方可
        // 解析、隐式 any 只报一次）。留 None 走 get_type_of_symbol 的
        // on-demand 求值（resolve_symbol_declared_type_on_demand_inner
        // 初始化式臂：getQuickTypeOfExpression/checkExpressionCachedEx +
        // widenTypeInferredFromInitializer，checker.go:17775-17882）
        let mut prop_type: Option<Arc<Type>> = match data.type_node.as_ref() {
            Some(tn) => Some(self.get_type_from_type_node(tn)),
            None => None,
        };
        let is_optional = data
            .postfix_token
            .as_ref()
            .map(|t| t.kind == SyntaxKind::QuestionToken)
            .unwrap_or(false);
        // 同 deferred 臂口径：非 strict 下 optional 不并 undefined（Go
        // addOptionalityEx 的 strictNullChecks 门，checker.go:17671）
        if is_optional && self.strict_null_checks {
            prop_type = prop_type.map(|pt| self.get_optional_type(pt, true));
        }
        let mut flags = SymbolFlags::Property;
        if is_optional {
            flags |= SymbolFlags::Optional;
        }
        let mut symbol = Symbol::new(flags, name.clone());

        symbol.declarations.push(Arc::clone(member));
        symbol.value_declaration = Some(Arc::clone(member));

        if let Some(m) = &data.modifiers {
            if m.modifier_flags.contains(ModifierFlags::Readonly) {
                symbol.check_flags |= CheckFlags::Readonly;
            }
        }
        let symbol = Arc::new(symbol);
        // error 型（环断路器 in-flight 产物）与 None（初始化式型走 on-demand）
        // 均不驻留，get_type_of_symbol 的 on-demand 重解析兜底
        let resolved = prop_type.filter(|pt| !crate::checker::utilities::is_type_error(pt));
        self.value_symbol_links.insert(
            &symbol,
            ValueSymbolLinks {
                resolved_type: resolved,
                ..Default::default()
            },
        );
        symbol_table.insert(name, Arc::clone(&symbol));
        props.push(symbol);
    }

    pub(crate) fn add_method_declaration_member(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
    ) { ::tsox_core::fntrace::enter("add_method_declaration_member"); 
        let NodeData::MethodDeclaration(data) = &member.data else {
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
        // 空串字面量名 `"": any` 是合法属性；仅计算属性取名失败才跳过
        if name.is_empty() && matches!(&data.name.data, NodeData::ComputedPropertyName(_)) {
            return;
        }

        self.push_scope(member);
        let return_type = match data.type_node.as_ref() {
            Some(tn) => self.get_type_from_type_node(tn),
            None => {
                if data.body.as_ref().is_some_and(|b| {
                    !Self::function_body_has_explicit_return(b)
                }) {
                    self.void_type()
                } else if data.body.is_some() {
                    self.infer_method_return_type(member, &data.body)
                } else {
                    self.get_any_type()
                }
            }
        };
        let sig = self.build_signature_from_function_like_type_node(
            &data.parameters,
            return_type,
            false,
            None,
            Some(Arc::clone(member)),
        );
        self.pop_scope();

        if let Some(existing) = symbol_table.get(&name).cloned() {
            if data.body.is_some() {
                let existing_mut = Arc::as_ptr(&existing) as *mut Symbol;
                unsafe {
                    (*existing_mut).declarations.push(Arc::clone(member));
                }
                return;
            }
            let existing_type = self
                .value_symbol_links
                .get(&existing)
                .and_then(|l| l.resolved_type.clone());
            let merged_sigs = existing_type
                .as_ref()
                .and_then(|t| t.as_structured().map(|s| s.call_signatures().to_vec()))
                .unwrap_or_default();
            let mut all_sigs = merged_sigs;
            all_sigs.push(sig);
            let fn_type = self.create_function_or_constructor_type_ex(
                all_sigs,
                false,
                Some(Arc::clone(&existing)),
            );
            self.value_symbol_links.insert(
                &existing,
                ValueSymbolLinks {
                    resolved_type: Some(fn_type),
                    ..Default::default()
                },
            );
            return;
        }
        let mut flags = SymbolFlags::Property | SymbolFlags::Method;
        if data
            .postfix_token
            .as_ref()
            .is_some_and(|t| t.kind == SyntaxKind::QuestionToken)
        {
            flags |= SymbolFlags::Optional;
        }
        let mut symbol = Symbol::new(flags, name.clone());

        symbol.declarations.push(Arc::clone(member));
        symbol.value_declaration = Some(Arc::clone(member));
        let symbol = Arc::new(symbol);
        let fn_type =
            self.create_function_or_constructor_type_ex(vec![sig], false, Some(Arc::clone(&symbol)));
        self.value_symbol_links.insert(
            &symbol,
            ValueSymbolLinks {
                resolved_type: Some(fn_type),
                ..Default::default()
            },
        );
        symbol_table.insert(name, Arc::clone(&symbol));
        props.push(symbol);
    }
}
