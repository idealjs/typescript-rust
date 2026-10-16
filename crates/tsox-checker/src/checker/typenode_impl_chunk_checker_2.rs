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
    ) -> Arc<Signature> { ::tsox_core::fntrace::enter("build_signature_from_function_like_type_node");
        self.build_signature_from_function_like_type_node_opt(
            parameters,
            Some(return_type),
            is_construct,
            contextual_signature,
            declaration,
        )
    }

    pub fn build_signature_from_function_like_type_node_opt(
        &mut self,
        parameters: &Arc<NodeList>,
        return_type: Option<Arc<Type>>,
        is_construct: bool,
        contextual_signature: Option<&Arc<Signature>>,
        declaration: Option<Arc<Node>>,
    ) -> Arc<Signature> { ::tsox_core::fntrace::enter("build_signature_from_function_like_type_node_opt"); 
        let type_parameters = self.type_parameters_of_declaration(&declaration);
        // Go instantiateSignatureEx（erase=false）：实例化窗口内构建的签名，自身
        // 类型参数克隆为新实例并经组合 mapper（[旧参→克隆]⊕[容器实参映射]）流入
        // 参数/返回型。克隆使 getErasedSignature 按实例匹配时只擦自身克隆，
        // 容器实参代入的外层类型参数（推断变量）不被误擦
        let mut member_tp_frame: std::collections::HashMap<
            *const tsox_frontend::ast::Symbol,
            Arc<Type>,
        > = std::collections::HashMap::new();
        let signature_type_parameters = if !self.type_argument_stack.is_empty() {
            let mut frame_entries: Vec<(*const tsox_frontend::ast::Symbol, usize)> = Vec::new();
            let mut clones: Vec<Arc<Type>> = type_parameters
                .iter()
                .enumerate()
                .map(|(i, tp)| {
                    let inherited_constraint = match &tp.data {
                        crate::checker::types::TypeData::TypeParameter(tpd) => {
                            tpd.constraint.clone()
                        }
                        _ => None,
                    };
                    let mut clone = Type::new(
                        crate::checker::types::TypeFlags::TypeParameter,
                        crate::checker::types::TypeData::TypeParameter(
                            crate::checker::types::TypeParameterData {
                                constrained: Default::default(),
                                constraint: inherited_constraint,
                                target: Some(Arc::clone(tp)),
                                mapper: None,
                                is_this_type: false,
                                resolved_default_type: std::sync::RwLock::new(None),
                            },
                        ),
                    );
                    clone.symbol = tp.symbol.clone();
                    clone.alias = tp.alias.clone();
                    let clone = Arc::new(clone);
                    if let Some(sym) = tp.symbol.as_ref() {
                        frame_entries.push((Arc::as_ptr(sym), i));
                    }
                    clone
                })
                .collect::<Vec<_>>();
            // Go instantiateSignatureEx（checker.go:21852-21856）克隆挂组合
            // mapper，getResolvedTypeParameterDefault（checker.go:23314-23337）
            // 显示期沿 target 取声明默认并经 mapper 再实例化（Promise<unknown>
            // 实例成员 then 呈 TResult1 = unknown 而非声明默认 T）。本仓引用
            // 计数下 mapper 强持克隆成自持环、Arc::get_mut 永久失败，改建型期
            // 预钉：此刻容器帧（type_argument_stack）仍活跃，按声明节点解析
            // 默认——与参数解析同一帧替换机制（resolve_type_parameter_reference，
            // 含符号漂移回退），帧内命中即容器实参代入。仅声明确有默认才钉
            // （type_parameter_has_default 以 is_some 为先手判据）；原型不落
            // 缓存，惰性解析留给原型自身显示语境
            for (clone, tp) in clones.iter_mut().zip(type_parameters.iter()) {
                let default_node = match tp
                    .symbol
                    .as_ref()
                    .and_then(|sym| sym.declarations.first())
                {
                    Some(d) => match &d.data {
                        tsox_frontend::ast::NodeData::TypeParameterDeclaration(td) => {
                            td.default_type.clone()
                        }
                        _ => None,
                    },
                    None => None,
                };
                let Some(default_node) = default_node else {
                    continue;
                };
                let resolved = self.get_type_from_type_node(&default_node);
                if let Some(clone_mut) = Arc::get_mut(clone) {
                    if let crate::checker::types::TypeData::TypeParameter(d) = &mut clone_mut.data
                    {
                        *d.resolved_default_type.write().unwrap() = Some(resolved);
                    }
                }
            }
            // Go instantiateSignatureEx（checker.go:21843-21852）：fresh 型参约束
            // 沿组合 mapper（{旧参→fresh}⊕基映射）惰性代入
            //（getConstraintFromTypeParameter checker.go:18074-18099
            // instantiateType(getConstraintOfTypeParameter(target), mapper)），
            // 兄弟约束内层引用落 fresh 实例——与签名声明位同一对象，显示按
            // tp 实例缓存命名不发明 _1 后缀（typeParameterToName
            // nodebuilderimpl.go:1532-1534）。本仓克隆携带继承约束（约束字段
            // 急解析缓存，内层引用指克隆前的型参实例），仅映射返回不回写约束
            // 会把 V extends U[] 留在克隆前旧实例上，显示位签名声明表（克隆）
            // 外呈阴影 U_1（typeParameterConstraintInstantiation 的 >m.map 应呈
            // V extends U[]）——默认预钉后（Arc::get_mut 先手）按
            // [克隆前列表→克隆] 映射改指（与 substituted_return 同一映射），
            // 链式引用（U←V←W）由克隆前列表已收敛的不动点保证一趟代入到位
            let mut repointed_constraints: Vec<Option<Arc<Type>>> =
                Vec::with_capacity(clones.len());
            for clone in &clones {
                let inherited = match &clone.data {
                    crate::checker::types::TypeData::TypeParameter(tpd) => tpd.constraint.clone(),
                    _ => None,
                };
                let repointed = match inherited {
                    Some(c) => {
                        let s =
                            self.substitute_infer_type_parameters(&c, &type_parameters, &clones);
                        if Arc::ptr_eq(&s, &c) { None } else { Some(s) }
                    }
                    None => None,
                };
                repointed_constraints.push(repointed);
            }
            for (i, sc) in repointed_constraints.into_iter().enumerate() {
                let Some(sc) = sc else { continue };
                if let Some(clone_mut) = Arc::get_mut(&mut clones[i])
                    && let crate::checker::types::TypeData::TypeParameter(d) = &mut clone_mut.data
                {
                    d.constraint = Some(sc);
                }
            }
            for (k, i) in frame_entries {
                if let Some(clone) = clones.get(i) {
                    member_tp_frame.insert(k, Arc::clone(clone));
                }
            }
            clones
        } else {
            type_parameters.clone()
        };
        let pushed_member_tp_frame = !member_tp_frame.is_empty();
        if pushed_member_tp_frame {
            self.type_argument_stack.push(member_tp_frame);
        }
        // Go instantiateSignatureEx（checker.go:21857-21866）：参数与返回同经
        // 组合 mapper。返回型由调用方在成员帧推入前解析，返回里的声明型参
        // 引用未经 [声明型参→克隆] 映射（参数在帧内解析已映射），此处补映射，
        // 未引用声明型参的返回恒等不变
        let substituted_return = match return_type.as_ref() {
            Some(rt) if pushed_member_tp_frame => Some(self.substitute_infer_type_parameters(
                rt,
                &type_parameters,
                &signature_type_parameters,
            )),
            Some(rt) => Some(Arc::clone(rt)),
            None => None,
        };
        let mut param_symbols: Vec<Arc<Symbol>> = Vec::with_capacity(parameters.len());
        let mut flags = SignatureFlags::None;
        if is_construct {
            flags |= SignatureFlags::Construct;
        }

        let mut min_argument_count: i32 = 0;

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
                Some(tn) => {
                    // Go getTypeForVariableLikeDeclaration（checker.go:17662）：
                    // 声明型存在即 addOptionalityEx(declaredType, isProperty=false,
                    // isOptional=HasQuestionToken)，strictNullChecks 下 optional
                    // 参数声明型并 undefined（depth?: D → D | undefined）
                    let declared = self.get_type_from_type_node(tn);
                    (self.add_optionality_ex(&declared, false, is_optional), true)
                }
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

            let param_type = if pd.question_token.is_some() {
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
            }
            let param_is_optional = is_rest
                || is_optional
                || pd.initializer.is_some()
                || (pd.type_node.is_none()
                    && Self::iife_with_too_few_arguments(&declaration, parameters.len()));
            if !param_is_optional {
                min_argument_count = param_symbols.len() as i32;
            }
        }
        // 实例化上下文（映射活跃）中构建的签名：实例化参数类型挂在签名上
        // 而非共享参数符号缓存（Go 实例化签名用 cloneSymbol，语义等价）
        let instantiated_parameter_types = if instantiated_params.is_empty() {
            None
        } else {
            Some(instantiated_params)
        };
        if pushed_member_tp_frame {
            self.type_argument_stack.pop();
        }
        let sig = Arc::new(Signature {
            id: 0,
            flags,
            min_argument_count,
            resolved_min_argument_count: -1,
            declaration,
            type_parameters: signature_type_parameters,
            parameters: param_symbols,
            this_parameter,
            resolved_return_type: std::sync::OnceLock::new(),
            resolved_type_predicate: std::sync::OnceLock::new(),
            target: None,
            mapper: None,
            isolated_signature_type: std::sync::OnceLock::new(),
            instantiated_parameter_types,
        });

        // 返回类型 None = OnceLock 留空（惰性）：自引用直传的函数建型由调用方
        // 在整体建成（get_type_of_function_like_inner）后回填函数型自身
        if let Some(rt) = substituted_return {
            let _ = sig.resolved_return_type.set(rt);
        }
        sig
    }

    pub(crate) fn type_parameters_of_declaration(
        &mut self,
        declaration: &Option<Arc<Node>>,
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("type_parameters_of_declaration"); 
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
        // Go getTypeParametersFromDeclaration（checker.go:21096-21100）：
        // AppendIfUnique 按声明类型身份去重——重复类型参数并入同一符号后
        // 两声明解析同一 TP 类型，签名只收一个（duplicateTypeParameters1
        // 渲染 <X> 而非 <X_1, X>）
        let mut tps: Vec<Arc<Type>> = Vec::with_capacity(symbols.len());
        for s in &symbols {
            let declared = self.get_type_parameter_from_symbol(s);
            if !tps.iter().any(|t| Arc::ptr_eq(t, &declared)) {
                tps.push(declared);
            }
        }
        self.instantiate_declaration_type_parameters(tps)
    }

    pub fn create_function_or_constructor_type(
        &self,
        sigs: Vec<Arc<Signature>>,
        is_construct: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("create_function_or_constructor_type"); 
        self.create_function_or_constructor_type_ex(sigs, is_construct, None)
    }

    pub fn create_function_or_constructor_type_ex(
        &self,
        sigs: Vec<Arc<Signature>>,
        is_construct: bool,
        symbol: Option<Arc<Symbol>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("create_function_or_constructor_type_ex"); 
        let call_signature_count = if is_construct { 0 } else { sigs.len() };
        let mut structured = StructuredTypeData::default();
        structured.signatures = sigs;
        structured.call_signature_count = call_signature_count;
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol,
            alias: None,
            data: TypeData::Object(ObjectTypeData { node: None,
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
    ) -> bool { ::tsox_core::fntrace::enter("check_cross_product_union"); 
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

    pub fn get_nullable_type(&mut self, t: &Arc<Type>, flags: TypeFlags) -> Arc<Type> { ::tsox_core::fntrace::enter("get_nullable_type"); 
        let missing = (flags & !t.flags) & (TypeFlags::Undefined | TypeFlags::Null);
        if missing.is_empty() {
            return Arc::clone(t);
        }
        if missing == TypeFlags::Undefined {
            return self.get_union_type(vec![Arc::clone(t), self.undefined_type()]);
        }
        if missing == TypeFlags::Null {
            return self.get_union_type(vec![Arc::clone(t), self.null_type()]);
        }
        self.get_union_type(vec![Arc::clone(t), self.undefined_type(), self.null_type()])
    }

    pub fn collect_return_types_from_node(
        &mut self,
        fn_node: Option<&Arc<Node>>,
        node: &Arc<Node>,
        types: &mut Vec<Arc<Type>>,
        has_return_with_no_expression: &mut bool,
        has_return_of_type_never: &mut bool,
    ) { ::tsox_core::fntrace::enter("collect_return_types_from_node"); 
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
                            if Self::is_async_function(fn_node)
                                && let NodeData::AwaitExpression(awaited) = &expr.data
                            {
                                let inner = Self::skip_parentheses(&awaited.expression);
                                if self.is_bare_recursive_call(fn_node, &inner) {
                                    *has_return_of_type_never = true;
                                    return;
                                }
                            }
                            let t = self.return_expression_type(fn_node, &expr);
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

    // Go checkAndAggregateReturnExpressionTypes（checker.go:21502）经
    // checkExpressionCachedEx 读返回表达式：流窄化新鲜字面量（const a = 0 的
    // case 窄化臂）保留 fresh 位，交由 finalize_inferred_return_type 的
    // is_fresh_literal_type 门加宽；get_type_of_node 惰性读取声明型常规字面量
    // 丢 fresh 位致 0 泄入签名显示。裸标识符（基线）与数组/对象字面量形态
    // （枚举聚合产生点：checkArrayLiteral 元素经
    // checkExpressionForMutableLocation 逐个拓宽，新鲜枚举字面量 widen 到
    // 枚举型）走检查路，其余形态维持惰性读，全量检查波及面压到字面量 return
    fn return_expression_type(
        &mut self,
        fn_node: Option<&Arc<Node>>,
        expr: &Arc<Node>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("return_expression_type"); 
        let bare = Self::skip_parentheses(expr);
        if bare.kind == SyntaxKind::Identifier
            && !self.is_own_function_reference(fn_node, &bare)
        {
            return self.check_expression_cached_ex(
                expr,
                crate::checker::types::CheckMode::Normal,
            );
        }
        match bare.kind {
            SyntaxKind::ArrayLiteralExpression | SyntaxKind::ObjectLiteralExpression => {
                self.check_expression_cached_ex(
                    expr,
                    crate::checker::types::CheckMode::Normal,
                )
            }
            _ => self.get_type_of_node(expr),
        }
    }

    // is_bare_recursive_call（本文件下方）仅护调用形自引用；裸标识符自引用
    //（function f() { return f } / const g = () => g）改道检查路会绕开既有
    // 部分型护栏（fn_self_return_partials/push_signature_return_resolution），
    // 按同款 owner 语义判自引用后回退基线读
    fn is_own_function_reference(
        &mut self,
        fn_node: Option<&Arc<Node>>,
        ident: &Arc<Node>,
    ) -> bool { ::tsox_core::fntrace::enter("is_own_function_reference"); 
        let Some(fn_node) = fn_node else {
            return false;
        };
        let Some(ident_sym) = self.resolve_identifier(ident) else {
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
        if matches!(
            fn_node.kind,
            SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction
        ) {
            return const_bound && owners.iter().any(|o| Arc::ptr_eq(o, &ident_sym));
        }
        owners.iter().any(|o| Arc::ptr_eq(o, &ident_sym))
    }

    fn is_bare_recursive_call(&mut self, fn_node: Option<&Arc<Node>>, expr: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_bare_recursive_call"); 
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

    pub fn may_return_never(fn_node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("may_return_never"); 
        match fn_node.kind {
            SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction => true,
            SyntaxKind::MethodDeclaration => fn_node
                .parent()
                .is_some_and(|p| p.kind == SyntaxKind::ObjectLiteralExpression),
            _ => false,
        }
    }

    fn is_async_function(fn_node: Option<&Arc<Node>>) -> bool { ::tsox_core::fntrace::enter("is_async_function"); 
        fn_node
            .is_some_and(|n| n.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Async))
    }

    pub fn infer_function_return_type(
        &mut self,
        fn_node: Option<&Arc<Node>>,
        body: Option<&Arc<Node>>,
        type_node: Option<&Arc<Node>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("infer_function_return_type"); 
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
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("infer_function_return_type_inner"); 
        let is_async = fn_node
            .map(|n| n.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Async))
            .unwrap_or(false);
        if body.kind != SyntaxKind::Block {
            let mut t = self.return_expression_type(fn_node, body);
            if self.is_const_context(body) {
                t = self.get_regular_type_of_literal_type(&t);
            }
            if is_async {
                t = self.awaited_type_of_body_return(fn_node, t);
            }
            return self.finalize_inferred_return_type(fn_node, t, is_async);
        }
        let mut types: Vec<Arc<Type>> = Vec::new();
        // Go checkAndAggregateReturnExpressionTypes（checker.go:21462）：
        // hasReturnWithNoExpression 初值取 functionHasImplicitReturn（checker.go:21507）
        // 的 EndFlowNode 可达性（binder.go:1648-1657 currentFlow 不可达不设）；
        // try{..}finally{return} 等体尾不可达形态不并入 undefined
        let mut has_return_with_no_expression = match fn_node {
            Some(f) => self.function_has_implicit_return(f),
            None => !self.function_body_definitely_returns(body),
        };
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
            let base = if never_returning {
                self.never_type()
            } else {
                self.void_type()
            };
            if is_async {
                return self.create_promise_return_type_for(&base);
            }
            return base;
        }
        // Go checkAndAggregateReturnExpressionTypes：strictNullChecks 下体尾
        // 可达（隐式 return undefined）时并入 undefined
        if self.strict_null_checks && has_return_with_no_expression {
            let undef = self.undefined_type();
            if !types.iter().any(|t| t.flags.contains(TypeFlags::Undefined)) {
                types.push(undef);
            }
        }
        if is_async {
            for t in types.iter_mut() {
                let owned = std::mem::replace(t, self.never_type());
                *t = self.awaited_type_of_body_return(fn_node, owned);
            }
        }
        let inferred = if types.len() == 1 {
            types.into_iter().next().expect("exactly one")
        } else {
            self.get_union_type(types)
        };

        self.finalize_inferred_return_type(fn_node, inferred, is_async)
    }

    // Go checkAndAggregateReturnExpressionTypes 的 async 分支：体返回表达式的
    // 类型先取 awaited 型（坏 thenable 报 TS1058 后按 errorType 继续参与）
    fn awaited_type_of_body_return(
        &mut self,
        fn_node: Option<&Arc<Node>>,
        t: Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("awaited_type_of_body_return"); 
        self.check_awaited_type_no_alias(
            &t,
            fn_node,
            &tsox_core::diagnostics::messages_generated::
                THE_RETURN_TYPE_OF_AN_ASYNC_FUNCTION_MUST_EITHER_BE_A_VALID_PROMISE_OR_MUST_NOT_CONTAIN_A_CALLABLE_THEN_MEMBER,
        )
        .map(|a| self.unwrap_awaited_type(&a))
        .unwrap_or_else(|| self.get_error_type())
    }

    // Go getReturnTypeFromBody 公共尾段：单元型按上下文签名返回型决定字面量
    // 保留或加宽，再整体加宽，async 容器包 Promise
    fn finalize_inferred_return_type(
        &mut self,
        fn_node: Option<&Arc<Node>>,
        mut t: Arc<Type>,
        is_async: bool,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("finalize_inferred_return_type"); 
        if crate::checker::is_unit_type(&t) {
            let contextual = match fn_node.and_then(|f| self.get_contextual_signature(f)) {
                Some(sig) => {
                    if sig
                        .declaration
                        .as_ref()
                        .is_some_and(|d| fn_node.is_some_and(|f| Arc::ptr_eq(d, f)))
                    {
                        Some(Arc::clone(&t))
                    } else {
                        self.get_return_type_of_signature(&sig).map(|rt| {
                            if is_async {
                                self.get_promised_type_of_promise(&rt).unwrap_or(rt)
                            } else {
                                rt
                            }
                        })
                    }
                }
                None => None,
            };
            let keep = contextual
                .as_ref()
                .is_some_and(|c| self.is_literal_of_contextual_type(&t, c));
            if !keep && crate::checker::is_fresh_literal_type(&t) {
                t = self.get_base_type_of_literal_type(&t);
            }
            t = self.get_regular_type_of_literal_type(&t);
        }
        let widened = self.widen_inferred_return_type(&t);
        if is_async {
            return self.create_promise_return_type_for(&widened);
        }
        widened
    }

    // Go getWidenedType：仅 RequiresWidening（widening null/undefined、对象/
    // 数组字面量）参与加宽；裸 fresh literal 与非数组/元组引用的类型实参不 widen
    fn widen_inferred_return_type(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("widen_inferred_return_type"); 
        if t.flags.intersects(crate::checker::types::TYPE_FLAGS_NULLABLE)
            && t
                .object_flags
                .intersects(crate::checker::types::OBJECT_FLAGS_REQUIRES_WIDENING)
        {
            return self.get_any_type();
        }
        if t.flags.contains(TypeFlags::Object)
            && t.object_flags.contains(ObjectFlags::ObjectLiteral)
        {
            if let Some(widened) = self.widen_object_literal_properties(t) {
                return widened;
            }
            if t.object_flags.contains(ObjectFlags::FreshLiteral) {
                if let Some(regular) = self.regular_object_literal_type(t) {
                    return regular;
                }
            }
            return Arc::clone(t);
        }
        if let TypeData::Union(union_data) = &t.data {
            let widened: Vec<Arc<Type>> = union_data
                .union_or_intersection
                .types
                .iter()
                .map(|member| self.widen_inferred_return_type(member))
                .collect();
            if widened
                .iter()
                .zip(union_data.union_or_intersection.types.iter())
                .all(|(w, o)| Arc::ptr_eq(w, o))
            {
                return Arc::clone(t);
            }
            // Go getWidenedTypeWithContext 联合分支（checker.go:19468-19481）：成员
            // widen 后经 getUnionTypeEx 重建，含空对象字面量时 Subtype 归约
            let reduction = if widened.iter().any(|w| self.is_empty_object_type(w)) {
                crate::checker::types::UnionReduction::Subtype
            } else {
                crate::checker::types::UnionReduction::Literal
            };
            return self.get_union_type_ex(widened, reduction, None, None);
        }
        if crate::checker::is_array_or_tuple_type(t)
            && t.object_flags.contains(ObjectFlags::Reference)
            && let Some(obj) = t.as_object()
            && !obj.type_arguments.is_empty()
        {
            let widened: Vec<Arc<Type>> = obj
                .type_arguments
                .iter()
                .map(|a| self.widen_inferred_return_type(a))
                .collect();
            let unchanged = widened
                .iter()
                .zip(obj.type_arguments.iter())
                .all(|(w, o)| Arc::ptr_eq(w, o));
            if !unchanged {
                return crate::checker::checker_attach_explicit_type_arguments::attach_explicit_type_arguments(
                    t, widened,
                );
            }
        }
        Arc::clone(t)
    }

    // Go createPromiseType：全局 Promise 泛型壳 + awaited 型实参
    pub(crate) fn create_promise_return_type_for(&mut self, promised: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("create_promise_return_type_for"); 
        let Some(promise_sym) = self.globals.get("Promise").cloned() else {
            return self.unknown_type();
        };
        let declared = self.get_declared_type_of_symbol(&promise_sym);
        let arg = self
            .get_awaited_type(promised)
            .unwrap_or_else(|| self.unknown_type());
        self.rebuild_with_type_arguments(&declared, vec![arg])
    }

    /// Go getWidenedTypeForVariableLikeDeclaration 的绑定模式分支：
    /// 数组模式 → 元组（默认值取加宽型，rest 元素成数组尾）；对象模式 → 匿名对象
    pub fn implied_type_for_binding_pattern(&mut self, pattern: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("implied_type_for_binding_pattern"); 
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
                        data: TypeData::Object(ObjectTypeData { node: None,
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
