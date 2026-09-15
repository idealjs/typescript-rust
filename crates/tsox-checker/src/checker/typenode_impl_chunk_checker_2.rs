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
        symbols
            .iter()
            .map(|s| self.get_type_parameter_from_symbol(s))
            .collect()
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

    pub fn collect_return_types_from_node(&mut self, node: &Arc<Node>, types: &mut Vec<Arc<Type>>) {
        use tsox_frontend::ast::node_data_generated::for_each_child;
        match node.kind {
            SyntaxKind::ReturnStatement => {
                if let tsox_frontend::ast::NodeData::ReturnStatement(data) = &node.data {
                    if let Some(expr) = &data.expression {
                        let t = self.get_type_of_node(expr);
                        types.push(t);
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
            self.collect_return_types_from_node(child, types);
            false
        });
    }

    pub fn infer_function_return_type(
        &mut self,
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

        if body.kind != SyntaxKind::Block {
            let t = self.get_type_of_node(body);
            return self.get_widened_type(&t);
        }
        let mut types: Vec<Arc<Type>> = Vec::new();
        self.collect_return_types_from_node(body, &mut types);
        if types.is_empty() {
            return self.void_type();
        }
        let inferred = if types.len() == 1 {
            types.into_iter().next().expect("exactly one")
        } else {
            self.get_union_type(types)
        };

        self.get_widened_type(&inferred)
    }
}
