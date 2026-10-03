#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    pub(crate) fn add_property_signature_member(
        &mut self,
        member: &Arc<Node>,
        symbol_table: &mut SymbolTable,
        props: &mut Vec<Arc<Symbol>>,
        own_symbol: Option<&Arc<Symbol>>,
    ) {
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
        let is_optional = data
            .postfix_token
            .as_ref()
            .map(|t| t.kind == SyntaxKind::QuestionToken)
            .unwrap_or(false);

        let resolved_type = if self.property_signature_type_deferred(&data.type_node, own_symbol) {
            None
        } else {
            let mut prop_type = self.get_type_from_type_node(&data.type_node);
            if is_optional {
                prop_type = self.get_optional_type(prop_type);
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
    ) -> bool {
        if self.variable_type_frame_depth > 0 {
            return true;
        }
        matches!(
            &type_node.data,
            NodeData::TypeReferenceNode(_) | NodeData::MappedTypeNode(_)
        ) && own_symbol.is_some()
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
    ) {
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
            if let Some(decl_symbol) = decl_symbol.as_ref() {
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
        let symbol = Arc::new(symbol);
        if let Some(decl_symbol) = decl_symbol.as_ref() {
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
    ) {
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
        let mut prop_type = match data.type_node.as_ref() {
            Some(tn) => self.get_type_from_type_node(tn),
            None => match data.initializer.as_ref() {
                Some(init) => {
                    let raw = match &init.data {
                        NodeData::Identifier(_) => match self.resolve_identifier(init) {
                            Some(sym)
                                if sym.flags.intersects(
                                    SymbolFlags::BlockScopedVariable
                                        | SymbolFlags::FunctionScopedVariable,
                                ) =>
                            {
                                self.get_type_of_symbol(&sym)
                            }
                            _ => self.get_type_of_node(init),
                        },
                        _ => self.get_type_of_node(init),
                    };
                    let is_readonly = data
                        .modifiers
                        .as_ref()
                        .is_some_and(|m| m.modifier_flags.contains(ModifierFlags::Readonly));
                    let widened = if is_readonly {
                        raw
                    } else if self.is_empty_array_literal(init) {
                        if self.strict_null_checks {
                            self.get_widened_literal_type(&raw)
                        } else {
                            self.create_array_type(self.get_any_type())
                        }
                    } else {
                        self.get_widened_literal_type(&raw)
                    };
                    let regularized = self.get_regular_type_of_literal_type(&widened);
                    self.widen_initializer_type(&regularized)
                }
                None => self.get_any_type(),
            },
        };
        let is_optional = data
            .postfix_token
            .as_ref()
            .map(|t| t.kind == SyntaxKind::QuestionToken)
            .unwrap_or(false);
        if is_optional {
            prop_type = self.get_optional_type(prop_type);
        }
        let mut flags = SymbolFlags::Property;
        if is_optional {
            flags |= SymbolFlags::Optional;
        }
        let mut symbol = Symbol::new(flags, name.clone());

        symbol.declarations.push(Arc::clone(member));

        if let Some(m) = &data.modifiers {
            if m.modifier_flags.contains(ModifierFlags::Readonly) {
                symbol.check_flags |= CheckFlags::Readonly;
            }
        }
        let symbol = Arc::new(symbol);
        // 递归接口（text: (v) => SameInterface）在构建窗口内经 FunctionTypeNode
        // 环断路器拿到 in-flight error：不驻留，留 None 走 get_type_of_symbol
        // 的 on-demand 重解析（窗口关闭后节点缓存为完整结果）
        let resolved = if crate::checker::utilities::is_type_error(&prop_type) {
            None
        } else {
            Some(prop_type)
        };
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
    ) {
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
