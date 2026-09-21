#![allow(unused_imports)]

use crate::checker::typenode_impl_chunk::*;

impl Checker {
    pub fn build_signature_from_function_like_type_node(
        &mut self,
        parameters: &Arc<NodeList>,
        return_type: Arc<Type>,
        is_construct: bool,
        contextual_signature: Option<&Arc<Signature>>,
        declaration: Option<Arc<Node>>,
    ) -> Arc<Signature> {
        let type_parameters = self.type_parameters_of_declaration(&declaration);
        let mut param_symbols: Vec<Arc<Symbol>> = Vec::with_capacity(parameters.len());
        let mut flags = SignatureFlags::None;
        if is_construct {
            flags |= SignatureFlags::Construct;
        }

        let mut min_argument_count: i32 = 0;
        let mut reached_optional_or_rest = false;

        let mut this_parameter: Option<Arc<Symbol>> = None;
        let mut instantiated_params: Vec<Arc<Type>> = Vec::new();
        for (i, param) in parameters.iter().enumerate() {
            let NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };
            let is_rest = pd.dot_dot_dot_token.is_some();
            let is_optional = pd.question_token.is_some();
            let is_this_param = i == 0
                && !is_rest
                && (matches!(&pd.name.data, NodeData::Identifier(id) if id.text == "this")
                    || pd.name.kind == SyntaxKind::ThisKeyword);

            let mapping_active = !self.type_argument_stack.is_empty();
            let (param_type, ctx_resolved) = match pd.type_node.as_ref() {
                Some(tn) => (self.get_type_from_type_node(tn), true),
                None => {
                    // ast.HasContextSensitiveParameters：带类型参数的函数不做上下文定型
                    let generic_source = declaration.as_ref().is_some_and(|d| {
                        match &d.data {
                            NodeData::FunctionExpression(fd) => fd.type_parameters.is_some(),
                            NodeData::ArrowFunction(ad) => ad.type_parameters.is_some(),
                            NodeData::FunctionDeclaration(fdd) => fdd.type_parameters.is_some(),
                            NodeData::MethodDeclaration(md) => md.type_parameters.is_some(),
                            _ => false,
                        }
                    });
                    match contextual_signature
                        .filter(|_| !generic_source)
                        .and_then(|ctx_sig| {
                            self.contextual_param_type_at(ctx_sig, parameters, i, param, is_rest, is_this_param)
                        }) {
                        Some(t) => {
                            // 推断期的泛型占位（类型含未解析类型参数）不落符号缓存：
                            // hover 按需经实例化后的上下文重定型（Go 推断期跳过
                            // context-sensitive 定型）
                            let mut free: Vec<Arc<Type>> = Vec::new();
                            self.collect_free_type_parameters_deep(&t, &mut free);
                            (t, free.is_empty())
                        }
                        None => (self.get_any_type(), false),
                    }
                }
            };

            let param_type = if pd.question_token.is_some() && pd.initializer.is_none() {
                self.add_optional_undefined(param_type)
            } else {
                param_type
            };

            let name = pd.name.text().to_string();
            let name = if name.is_empty() {
                format!("__arg{}", i)
            } else {
                name
            };

            let sym = match self.program.symbol_map().symbol_of(param) {
                Some(s) => Arc::clone(s),
                None => Arc::new(Symbol::new(SymbolFlags::Property, name)),
            };
            // 参数符号与声明共享：实例化上下文（映射活跃）不写缓存，
            // 防止实例化类型覆盖声明形式（Go 实例化签名用 cloneSymbol 隔离）
            if ctx_resolved && self.type_argument_stack.is_empty() {
                self.value_symbol_links.insert(
                    &sym,
                    ValueSymbolLinks {
                        resolved_type: Some(Arc::clone(&param_type)),
                        ..Default::default()
                    },
                );
            }
            param_symbols.push(sym);
            if mapping_active {
                instantiated_params.push(Arc::clone(&param_type));
            }
            if is_this_param && this_parameter.is_none() {
                this_parameter = param_symbols.pop();
                if mapping_active {
                    instantiated_params.pop();
                }
                continue;
            }
            if is_rest {
                flags |= SignatureFlags::HasRestParameter;
                reached_optional_or_rest = true;
            } else if is_optional
                || pd.initializer.is_some()
                || (pd.type_node.is_none()
                    && Self::iife_with_too_few_arguments(&declaration, parameters.len()))
            {
                reached_optional_or_rest = true;
            }
            if !reached_optional_or_rest {
                min_argument_count += 1;
            }
        }
        // 实例化上下文（映射活跃）中构建的签名：实例化参数类型挂在签名上
        // 而非共享参数符号缓存（Go 实例化签名用 cloneSymbol，语义等价）
        let instantiated_parameter_types = if instantiated_params.is_empty() {
            None
        } else {
            Some(instantiated_params)
        };
        let sig = Arc::new(Signature {
            id: 0,
            flags,
            min_argument_count,
            resolved_min_argument_count: -1,
            declaration,
            type_parameters,
            parameters: param_symbols,
            this_parameter,
            resolved_return_type: std::sync::OnceLock::new(),
            resolved_type_predicate: None,
            target: None,
            mapper: None,
            isolated_signature_type: std::sync::OnceLock::new(),
            instantiated_parameter_types,
        });

        let _ = sig.resolved_return_type.set(return_type);
        sig
    }

    pub(crate) fn type_parameters_of_declaration(
        &mut self,
        declaration: &Option<Arc<Node>>,
    ) -> Vec<Arc<Type>> {
        let Some(decl) = declaration else {
            return Vec::new();
        };
        let tp_list = match &decl.data {
            NodeData::FunctionDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::FunctionExpression(d) => d.type_parameters.as_ref(),
            NodeData::ArrowFunction(d) => d.type_parameters.as_ref(),
            NodeData::MethodDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::MethodSignatureDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::ConstructorDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::GetAccessorDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::SetAccessorDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::FunctionTypeNode(d) => d.type_parameters.as_ref(),
            NodeData::ConstructorTypeNode(d) => d.type_parameters.as_ref(),

            NodeData::CallSignatureDeclaration(d) => d.type_parameters.as_ref(),
            NodeData::ConstructSignatureDeclaration(d) => d.type_parameters.as_ref(),
            _ => None,
        };
        let Some(list) = tp_list else {
            return Vec::new();
        };

        let symbols: Vec<Arc<Symbol>> = list
            .iter()
            .filter_map(|tp| self.program.symbol_map().symbol_of(tp).map(Arc::clone))
            .collect();
        let tps: Vec<Arc<Type>> = symbols
            .iter()
            .map(|s| self.get_type_parameter_from_symbol(s))
            .collect();
        self.instantiate_declaration_type_parameters(tps)
    }

    pub fn create_function_or_constructor_type(
        &self,
        sigs: Vec<Arc<Signature>>,
        is_construct: bool,
    ) -> Arc<Type> {
        let call_signature_count = if is_construct { 0 } else { sigs.len() };
        let mut structured = StructuredTypeData::default();
        structured.signatures = sigs;
        structured.call_signature_count = call_signature_count;
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: None,
            alias: None,
            data: TypeData::Object(ObjectTypeData {
                structured,
                target: None,
                mapper: None,
                type_arguments: Vec::new(),
            }),
        })
    }

    pub(crate) fn check_cross_product_union(
        &mut self,
        node: &Arc<Node>,
        types: &[Arc<Type>],
    ) -> bool {
        if Self::cross_product_union_size(types) < 100_000 {
            return true;
        }
        let already = self
            .diagnostics
            .get_all()
            .iter()
            .any(|d| d.code == 2590 && d.loc.pos() == node.loc.pos());
        if !already {
            let file = self
                .get_source_file_of_node(node)
                .or_else(|| self.current_file.clone());
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                file,
                node.loc,
                tsox_core::diagnostics::messages_generated::
                    EXPRESSION_PRODUCES_A_UNION_TYPE_THAT_IS_TOO_COMPLEX_TO_REPRESENT,
                vec![],
            ));
        }
        false
    }

    pub fn get_nullable_type(&self, t: &Arc<Type>, flags: TypeFlags) -> Arc<Type> {
        let mut types = vec![Arc::clone(t)];
        if flags.contains(TypeFlags::Null) {
            types.push(self.null_type());
        }
        if flags.contains(TypeFlags::Undefined) {
            types.push(self.undefined_type());
        }
        if types.len() == 1 {
            return types.into_iter().next().expect("exactly one");
        }
        Arc::new(Type::new(
            TypeFlags::Union,
            TypeData::Union(UnionTypeData {
                union_or_intersection: UnionOrIntersectionTypeData {
                    structured: StructuredTypeData::default(),
                    types,
                },
                resolved_reduced_type: std::sync::OnceLock::new(),
                regular_type: std::sync::OnceLock::new(),
                origin: None,
                key_property_name: None,
                constituent_map: HashMap::new(),
            }),
        ))
    }

    pub fn collect_return_types_from_node(
        &mut self,
        fn_node: Option<&Arc<Node>>,
        node: &Arc<Node>,
        types: &mut Vec<Arc<Type>>,
        has_return_with_no_expression: &mut bool,
        has_return_of_type_never: &mut bool,
    ) {
        use tsox_frontend::ast::node_data_generated::for_each_child;
        match node.kind {
            SyntaxKind::ReturnStatement => {
                if let tsox_frontend::ast::NodeData::ReturnStatement(data) = &node.data {
                    match &data.expression {
                        None => {
                            *has_return_with_no_expression = true;
                        }
                        Some(expr) => {
                            let expr = Self::skip_parentheses(expr);
                            if self.is_bare_recursive_call(fn_node, &expr) {
                                *has_return_of_type_never = true;
                                return;
                            }
                            let t = self.get_type_of_node(&expr);
                            if t.flags.contains(TypeFlags::Never) {
                                *has_return_of_type_never = true;
                            }
                            types.push(t);
                        }
                    }
                }
                return;
            }

            SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor => return,
            _ => {}
        }
        for_each_child(node, |child| {
            self.collect_return_types_from_node(
                fn_node,
                child,
                types,
                has_return_with_no_expression,
                has_return_of_type_never,
            );
            false
        });
    }

    fn is_bare_recursive_call(&mut self, fn_node: Option<&Arc<Node>>, expr: &Arc<Node>) -> bool {
        if expr.kind != SyntaxKind::CallExpression {
            return false;
        }
        let NodeData::CallExpression(call) = &expr.data else {
            return false;
        };
        let callee = &call.expression;
        if callee.kind != SyntaxKind::Identifier {
            return false;
        }
        let Some(fn_node) = fn_node else {
            return false;
        };
        let mut owners = Vec::new();
        if let Some(s) = self.program.symbol_map().symbol_of(fn_node) {
            owners.push(Arc::clone(s));
        }
        let mut const_bound = false;
        if let Some(parent) = fn_node.parent()
            && parent.kind == SyntaxKind::VariableDeclaration
            && self
                .get_combined_node_flags(&parent)
                .contains(tsox_frontend::ast::NodeFlags::Const)
        {
            const_bound = true;
            if let Some(s) = self.program.symbol_map().symbol_of(&parent) {
                owners.push(Arc::clone(s));
            }
        }
        if owners.is_empty() {
            return false;
        }
        let Some(callee_sym) = self.resolve_identifier(callee) else {
            return false;
        };
        if !owners.iter().any(|o| Arc::ptr_eq(o, &callee_sym)) {
            return false;
        }
        if matches!(
            fn_node.kind,
            SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction
        ) {
            return const_bound;
        }
        true
    }

    pub fn may_return_never(fn_node: &Arc<Node>) -> bool {
        match fn_node.kind {
            SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction => true,
            SyntaxKind::MethodDeclaration => fn_node
                .parent()
                .is_some_and(|p| p.kind == SyntaxKind::ObjectLiteralExpression),
            _ => false,
        }
    }

    pub fn infer_function_return_type(
        &mut self,
        fn_node: Option<&Arc<Node>>,
        body: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
    ) -> Arc<Type> {
        if let Some(type_node) = type_node {
            return self.get_type_from_type_node(type_node);
        }
        let Some(body) = body else {
            // Go：无注解无体的签名（方法签名/重载声明）返回 any
            return self.get_any_type();
        };

        // Go checkNodeDeferred：函数值定型期体推断延后到外层符号帧出栈之后；
        // 调用位返回型查询（getResolvedSignature 内）是 Go 同步强制的，不开界
        let boundary = if self.call_return_query_depth == 0 {
            self.rt_infer_boundary_marks
                .push(self.type_resolution_stack.len());
            true
        } else {
            false
        };
        let result = self.infer_function_return_type_inner(body, fn_node);
        if boundary {
            self.rt_infer_boundary_marks.pop();
        }
        result
    }

    fn infer_function_return_type_inner(
        &mut self,
        body: &Arc<Node>,
        fn_node: Option<&Arc<Node>>,
    ) -> Arc<Type> {
        if body.kind != SyntaxKind::Block {
            let t = self.get_type_of_node(body);
            return self.get_widened_type(&t);
        }
        let mut types: Vec<Arc<Type>> = Vec::new();
        let mut has_return_with_no_expression = !self.function_body_definitely_returns(body);
        let mut has_return_of_type_never = false;
        self.collect_return_types_from_node(
            fn_node,
            body,
            &mut types,
            &mut has_return_with_no_expression,
            &mut has_return_of_type_never,
        );
        if types.is_empty() {
            let never_returning = !has_return_with_no_expression
                && (has_return_of_type_never || fn_node.is_some_and(Self::may_return_never));
            if never_returning {
                return self.never_type();
            }
            return self.void_type();
        }
        // Go checkAndAggregateReturnExpressionTypes：strictNullChecks 下体尾
        // 可达（隐式 return undefined）时并入 undefined
        if self.strict_null_checks && has_return_with_no_expression {
            let undef = self.undefined_type();
            if !types.iter().any(|t| t.flags.contains(TypeFlags::Undefined)) {
                types.push(undef);
            }
        }
        let inferred = if types.len() == 1 {
            types.into_iter().next().expect("exactly one")
        } else {
            self.get_union_type(types)
        };

        self.get_widened_type(&inferred)
    }

    /// Go getWidenedTypeForVariableLikeDeclaration 的绑定模式分支：
    /// 数组模式 → 元组（默认值取加宽型，rest 元素成数组尾）；对象模式 → 匿名对象
    pub fn implied_type_for_binding_pattern(&mut self, pattern: &Arc<Node>) -> Arc<Type> {
        match &pattern.data {
            NodeData::BindingPattern(bp) => match pattern.kind {
                SyntaxKind::ArrayBindingPattern => {
                    let mut element_types: Vec<Arc<Type>> = Vec::new();
                    let mut infos: Vec<TupleElementInfo> = Vec::new();
                    for el in bp.elements.iter() {
                        if el.kind == SyntaxKind::OmittedExpression {
                            element_types.push(self.any_type());
                            infos.push(TupleElementInfo {
                                label: None,
                                flags: ElementFlags::Optional,
                                labeled_declaration: None,
                                type_: None,
                            });
                            continue;
                        }
                        let NodeData::BindingElement(bd) = &el.data else {
                            element_types.push(self.any_type());
                            infos.push(TupleElementInfo {
                                label: None,
                                flags: ElementFlags::Required,
                                labeled_declaration: None,
                                type_: None,
                            });
                            continue;
                        };
                        let (t, optional) = match &bd.initializer {
                            Some(init) => {
                                let it = self.get_type_of_node(init);
                                (self.get_widened_type(&it), true)
                            }
                            None => (self.any_type(), false),
                        };
                        let elem = if bd.dot_dot_dot_token.is_some() {
                            let rest_arr = self.create_array_type(Arc::clone(&t));
                            infos.push(TupleElementInfo {
                                label: None,
                                flags: ElementFlags::Rest,
                                labeled_declaration: None,
                                type_: Some(Arc::clone(&rest_arr)),
                            });
                            rest_arr
                        } else {
                            infos.push(TupleElementInfo {
                                label: None,
                                flags: if optional {
                                    ElementFlags::Optional
                                } else {
                                    ElementFlags::Required
                                },
                                labeled_declaration: None,
                                type_: Some(Arc::clone(&t)),
                            });
                            t
                        };
                        element_types.push(elem);
                    }
                    self.create_tuple_type_ex(element_types, infos, false)
                }
                SyntaxKind::ObjectBindingPattern => {
                    let mut symbol_table = SymbolTable::new();
                    let mut props: Vec<Arc<Symbol>> = Vec::new();
                    for el in bp.elements.iter() {
                        let NodeData::BindingElement(bd) = &el.data else {
                            continue;
                        };
                        let key = bd
                            .property_name
                            .as_ref()
                            .map(|p| p.text().to_string())
                            .or_else(|| bd.name.as_ref().map(|n| n.text().to_string()))
                            .unwrap_or_default();
                        if key.is_empty() {
                            continue;
                        }
                        let (t, optional) = match &bd.initializer {
                            Some(init) => {
                                let it = self.get_type_of_node(init);
                                (self.get_widened_type(&it), true)
                            }
                            None => (self.any_type(), false),
                        };
                        let mut flags = SymbolFlags::Property;
                        if optional {
                            flags |= SymbolFlags::Optional;
                        }
                        let mut sym = Symbol::new(flags, key.clone());
                        sym.declarations.push(Arc::clone(el));
                        let sym = Arc::new(sym);
                        self.value_symbol_links.insert(
                            &sym,
                            ValueSymbolLinks {
                                resolved_type: Some(t),
                                ..Default::default()
                            },
                        );
                        symbol_table.insert(key, Arc::clone(&sym));
                        props.push(sym);
                    }
                    Arc::new(Type {
                        flags: TypeFlags::Object,
                        object_flags: ObjectFlags::Anonymous,
                        id: crate::checker::types::next_type_id(),
                        symbol: None,
                        alias: None,
                        data: TypeData::Object(ObjectTypeData {
                            structured: StructuredTypeData {
                                members: symbol_table,
                                properties: props,
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    })
                }
                _ => self.get_any_type(),
            },
            _ => self.get_any_type(),
        }
    }
}
