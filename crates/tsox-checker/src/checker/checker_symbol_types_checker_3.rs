#![allow(unused_imports)]

use crate::checker::checker_symbol_types::*;

impl Checker {
    pub(crate) fn get_value_type_of_symbol(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_value_type_of_symbol"); 
        if let Some(links) = self.value_symbol_links.get(symbol) {
            if let Some(ref t) = links.resolved_type {
                return Arc::clone(t);
            }
        }

        if let Some(decl) = &symbol.value_declaration {
            if let Some(links) = self.type_node_links.get(decl) {
                if let Some(ref t) = links.resolved_type {
                    return Arc::clone(t);
                }
            }
        }

        for decl in &symbol.declarations {
            if let Some(links) = self.type_node_links.get(decl) {
                if let Some(ref t) = links.resolved_type {
                    return Arc::clone(t);
                }
            }
        }

        // Go getTypeOfFuncClassEnumModuleWorker（checker.go:17908-17914）：类
        // 符号值型是构造签名对象型（含隐式 new () => 实例），不依赖检查期
        // 缓存；类+namespace 合并符号的 valueDeclaration 是模块声明，缓存
        // 未命中时按类声明构建
        if symbol.flags.contains(tsox_frontend::ast::SymbolFlags::Class) {
            if let Some(decl) = symbol
                .declarations
                .iter()
                .find(|d| {
                    matches!(
                        d.kind,
                        tsox_frontend::ast::SyntaxKind::ClassDeclaration
                            | tsox_frontend::ast::SyntaxKind::ClassExpression
                    )
                })
                .cloned()
            {
                let t = self.get_type_of_class_declaration(&decl);
                self.value_symbol_links
                    .get_or_default(symbol)
                    .resolved_type = Some(Arc::clone(&t));
                return t;
            }
        }

        // 合并 function+namespace 的调用签名来源：未缓存时按需建
        // 函数型（过载走 build_overload_function_type）
        let fn_decl_count = symbol
            .declarations
            .iter()
            .filter(|d| d.kind == SyntaxKind::FunctionDeclaration)
            .count();
        if fn_decl_count > 1
            && let Some(t) = self.build_overload_function_type(symbol)
        {
            let t = self.attach_function_expando_type(symbol, t);
            self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
            return t;
        }
        if let Some(decl) = symbol
            .declarations
            .iter()
            .find(|d| d.kind == SyntaxKind::FunctionDeclaration)
        {
            let base = self.get_type_of_function_like(decl);
            let t = self.attach_function_expando_type(symbol, base);
            self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
            return t;
        }

        self.get_any_type()
    }

    pub(crate) fn resolve_enum_value_type(&mut self, symbol: &Arc<Symbol>) -> Arc<Type> { ::tsox_core::fntrace::enter("resolve_enum_value_type"); 
        if let Some(links) = self.value_symbol_links.get(symbol) {
            if let Some(ref t) = links.resolved_type {
                return Arc::clone(t);
            }
        }

        let _ = self.resolve_enum_type(symbol);

        let members: Vec<(String, Arc<Symbol>)> = symbol
            .members
            .iter()
            .map(|(k, v)| (k.clone(), Arc::clone(v)))
            .collect();
        let mut symbol_table = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::new();
        for (name, member_sym) in &members {
            if name.starts_with("\u{FE}") {
                continue;
            }

            let _ = self.get_type_of_symbol(member_sym);
            symbol_table.insert(name.clone(), Arc::clone(member_sym));
            props.push(Arc::clone(member_sym));
        }
        let mut index_infos: Vec<Arc<crate::checker::types::IndexInfo>> = Vec::new();
        let declared_type = self.get_declared_type_of_enum(&symbol);
        let props_number_like = props.iter().any(|prop| {
            self.get_type_of_symbol(prop)
                .flags
                .intersects(crate::checker::types_type_id::TYPE_FLAGS_NUMBER_LIKE)
        });
        if symbol.flags.intersects(SymbolFlags::ENUM)
            && (declared_type.flags.contains(TypeFlags::Enum) || props_number_like)
        {
            index_infos.push(self.enum_number_index_info());
        }
        let result = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::Object(ObjectTypeData { node: None,
                structured: StructuredTypeData {
                    constrained: ConstrainedTypeData::default(),
                    members: symbol_table,
                    properties: props,
                    signatures: Vec::new(),
                    call_signature_count: 0,
                    index_infos,
                    object_type_without_abstract_construct_signatures: std::sync::OnceLock::new(),
                },
                target: None,
                mapper: None,
                type_arguments: Vec::new(),
            }),
        });
        self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&result));
        result
    }

    pub(crate) fn get_type_of_function_like(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_function_like"); 
        // Go getReturnTypeOfSignature：有体签名的返回型推断进 type resolution 栈，
        // 同签名重入即环（重入静默 any），pop 失败报 TS2577/TS7023；
        // 无体声明只解析注解（Go 同位返回惰性签名型，无返回型查询）
        if Self::function_like_has_body(node)
            && !self.push_signature_return_resolution(Arc::as_ptr(node))
        {
            return self.partial_type_of_function_like(node);
        }
        let result = self.get_type_of_function_like_inner(node);
        if Self::function_like_has_body(node) && !self.pop_signature_return_resolution() {
            self.report_signature_return_circularity(node);
            return self.get_any_type();
        }
        result
    }

    fn get_type_of_function_like_inner(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_function_like_inner"); 
        let (parameters, body, type_node) = match &node.data {
            tsox_frontend::ast::NodeData::FunctionExpression(data) => {
                (&data.parameters, Some(&data.body), data.type_node.as_ref())
            }
            tsox_frontend::ast::NodeData::ArrowFunction(data) => {
                (&data.parameters, Some(&data.body), data.type_node.as_ref())
            }
            tsox_frontend::ast::NodeData::FunctionDeclaration(data) => (
                &data.parameters,
                data.body.as_ref(),
                data.type_node.as_ref(),
            ),
            tsox_frontend::ast::NodeData::MethodSignatureDeclaration(data) => (
                &data.parameters,
                None,
                data.type_node.as_ref(),
            ),
            _ => return self.get_any_type(),
        };

        let contextual_signature: Option<Arc<Signature>> = self
            .get_contextual_signature(node)
            .or_else(|| self.iife_contextual_signature(node));
        self.record_annotated_param_inferences(node, contextual_signature.as_ref());
        let contextual_signature = contextual_signature.as_ref();

        let is_arrow = matches!(node.data, tsox_frontend::ast::NodeData::ArrowFunction(_));
        if is_arrow {
            self.push_arrow_function_scope(node);
        } else {
            self.push_function_scope(node);
        }

        let placeholder = self.get_any_type();
        let _primed = self.build_signature_from_function_like_type_node(
            parameters,
            placeholder,
            false,
            contextual_signature,
            Some(Arc::clone(node)),
        );

        let is_generator = match &node.data {
            tsox_frontend::ast::NodeData::FunctionDeclaration(d) => d.asterisk_token.is_some(),
            tsox_frontend::ast::NodeData::FunctionExpression(d) => d.asterisk_token.is_some(),
            tsox_frontend::ast::NodeData::MethodDeclaration(d) => d.asterisk_token.is_some(),
            _ => false,
        };
        // 瞬态窗口覆盖整个推断段：重入臂产出的部分型在调用栈回溯至
        // get_type_of_node 的驻留点时仍须可识别——窗口若只包 partial 执行
        // 本身会在驻留发生前关闭，return <自身名> 的 identifier 节点缓存
        // 将驻留部分型，呈现期命中陈旧值呈 (p: I) => any
        let saved_partial_window = self.in_partial_fn_return.replace(Arc::as_ptr(node));
        self.set_signature_return_inference_phase(Arc::as_ptr(node), true);
        let body_inferred = type_node.is_none() && body.is_some();
        let is_decl = matches!(&node.data, tsox_frontend::ast::NodeData::FunctionDeclaration(_));
        let return_type = if is_generator && type_node.is_none() && body.is_some() {
            if is_decl {
                // Go getReturnTypeOfSignature（checker.go:21209-21211）无注解有体走
                // getReturnTypeFromBody：生成器返回型经 checkAndAggregateYieldOperandTypes
                // 聚合（next 取 yield 表达式上下文型，如动态 import 实参位为 string），
                // 非旧 infer_generator_return_type 近似
                self.get_return_type_from_body(node, CheckMode::Normal)
            } else {
                // 函数表达式/对象字面量方法：Go getTypeOfFuncClassEnumModuleWorker
                // （checker.go:17897-17920）对函数符返回惰性壳，函数体的首次返回型
                // 推断发生在 contextuallyCheck 的 assignContextualParameterTypes 之后，
                // 经 getReturnTypeFromBody(checkMode) 传播通道执行（checker.go:
                // 10768-10771）；本侧 compute_type_of_node→get_type_of_function_like
                // 急切求型，若在此走检查路聚合，会把生成器体内嵌套调用（effectFn(
                // function* (a){...})、yield* traverse(...)）的解析提前到上下文参数
                // 指派之前，推断期状态未固定即对峙泛型签名，呈 TS2345/TS2554
                //（contextualParamTypeVsNestedReturnTypeInference2/3/4 与
                // genericCallAtYieldExpressionInGenericCall3，波起点 PASS）。表达式/
                // 方法位保留 infer_generator_return_type 显示路收集，检查路体推断
                // 留给 m1c_5 contextually_check 的传播臂（mig/m1c_5.rs:168-174）
                let is_async_fn = node.has_syntactic_modifier(ModifierFlags::Async);
                self.infer_generator_return_type(body.unwrap(), is_async_fn)
            }
        } else {
            self.infer_function_return_type(Some(node), body, type_node)
        };
        self.in_partial_fn_return = saved_partial_window;
        // Go contextuallyCheckFunctionExpressionOrObjectLiteralMethod
        // （checker.go:10715-10722）以 CheckMode.Contextual|Inferential 解析体返回
        // 型：返回字面量成员/返回表达式经 instantiateTypeWithSingleGenericCallSignature
        // （checker.go:8064/8134）按上下文成员签名实例化；inferTypeArguments 对参数
        // 位含类型变量的全部实参（checker.go:10012，无论上下文敏感与否）都带推断
        // 语境检查。本侧成员检查无 CheckMode 通道且成员型节点缓存供 .types 显示
        // 复用，实例化按型施加于签名的返回型副本（原字面量型保持未实例化）。
        // 注解返回型 Go 走 getReturnTypeFromAnnotation（checker.go:21216）不做体
        // 推断，跳过
        let return_type = if body_inferred {
            match contextual_signature {
                Some(ctx) => self.instantiate_return_type_inferentially(&return_type, ctx),
                None => return_type,
            }
        } else {
            return_type
        };
        self.set_signature_return_inference_phase(Arc::as_ptr(node), false);
        if is_arrow {
            self.pop_arrow_function_scope();
        } else {
            self.pop_function_scope();
        }

        // 返回类型推断窗口结束即消费本函数的重入部分型：推断产物恰为部分型
        // 自身（return <自身名> 直传）时签名返回类型留惰性，函数型整体建成
        // 后回填自身成环——Go 符号型壳先挂接（getTypeOfFuncClassEnumModule
        // checker.go:17888）、getReturnTypeOfSignature 的 sig.resolvedReturnType
        // = 壳自身，呈现层 visitedTypes 命中后按名输出 typeof f
        let self_referential = self
            .fn_self_return_partials
            .remove(&Arc::as_ptr(node))
            .filter(|p| Arc::ptr_eq(p, &return_type))
            .is_some();

        let mut sig = self.build_signature_from_function_like_type_node_opt(
            parameters,
            if self_referential {
                None
            } else {
                Some(return_type)
            },
            false,
            contextual_signature,
            Some(Arc::clone(node)),
        );

        // Go assignContextualParameterTypes（checker.go:10921-10923）：上下文敏感
        // 函数表达式的签名在上下文签名带类型参数且自身无类型参数时吃进
        // context.typeParameters，getTypeOfNode 表达式臂（checker.go:33807 →
        // getTypeOfExpression → getTypeOfSymbol，checker.go:10712）显示带 <T, U>
        // 前缀。本侧赋值语句等场景检查期不取表达式型（check_expression 只做
        // 语法/副作用检查），显示经此现建签名（checker.go:10712 的符号型不可达），
        // 在此补同一 typeParameters 接收
        if sig.type_parameters.is_empty()
            && self.is_context_sensitive(node)
            && let Some(contextual) = contextual_signature
            && !contextual.type_parameters.is_empty()
        {
            Arc::make_mut(&mut sig).type_parameters = contextual.type_parameters.clone();
        }

        if !sig.type_parameters.is_empty()
            && let Some(contextual) = contextual_signature
        {
            if contextual.type_parameters.is_empty() {
                let inst = self.instantiate_signature_in_context_of(&sig, contextual);
                return self.create_function_or_constructor_type(vec![inst], false);
            }
        }
        // 自引用直传时函数型携带符号（Go newObjectType(ObjectFlagsAnonymous,
        // symbol) 壳语义）——呈现层按名输出 typeof 的取符依据；回填使
        // sig.resolvedReturnType 指向函数型自身（Arc 环，Go 类型图同构循环）
        let fn_symbol = if self_referential {
            self.get_symbol_of_declaration(node)
        } else {
            None
        };
        let fn_type = self.create_function_or_constructor_type_ex(
            vec![Arc::clone(&sig)],
            false,
            fn_symbol,
        );
        if self_referential {
            let _ = sig.resolved_return_type.set(Arc::clone(&fn_type));
        }
        fn_type
    }

    /// Go contextuallyCheckFunctionExpressionOrObjectLiteralMethod 的非上下文敏感
    /// 分支：注解参数（与注解返回型）与上下文签名对位做推断。Go 在检查现场直接
    /// 写入调用推断语境；推断语境在此不可达，落 pending 清单由推断实参循环回放
    fn record_annotated_param_inferences(
        &mut self,
        node: &Arc<Node>,
        contextual: Option<&Arc<Signature>>,
    ) { ::tsox_core::fntrace::enter("record_annotated_param_inferences"); 
        let Some(contextual) = contextual else {
            return;
        };
        if self.inference_loop_depth == 0 || self.is_context_sensitive(node) {
            return;
        }
        let (parameters, has_type_params, return_type_node) = match &node.data {
            tsox_frontend::ast::NodeData::FunctionExpression(d) => {
                (&d.parameters, d.type_parameters.is_some(), d.type_node.as_ref())
            }
            tsox_frontend::ast::NodeData::ArrowFunction(d) => {
                (&d.parameters, d.type_parameters.is_some(), d.type_node.as_ref())
            }
            tsox_frontend::ast::NodeData::MethodDeclaration(d) => (
                &d.parameters,
                d.type_parameters.is_some(),
                d.type_node.as_ref(),
            ),
            _ => return,
        };
        if has_type_params {
            return;
        }
        let ctx_param_count = contextual
            .parameters
            .len()
            .saturating_sub(if contextual.flags.contains(SignatureFlags::HasRestParameter) {
                1
            } else {
                0
            });
        if ctx_param_count <= parameters.len() {
            return;
        }
        for (i, param) in parameters.iter().enumerate() {
            let tsox_frontend::ast::NodeData::ParameterDeclaration(pd) = &param.data else {
                continue;
            };
            let Some(tn) = pd.type_node.as_ref() else {
                continue;
            };
            if i >= ctx_param_count {
                break;
            }
            let source = self.get_type_from_type_node(tn);
            let source = if pd.question_token.is_some() || pd.initializer.is_some() {
                self.add_optional_undefined(source)
            } else {
                source
            };
            let target = self
                .signature_instantiated_param_type(contextual, i)
                .unwrap_or_else(|| self.get_type_of_symbol(&contextual.parameters[i]));
            self.pending_annotated_param_inferences
                .push((source, target));
        }
        if let Some(rtn) = return_type_node {
            let source = self.get_type_from_type_node(rtn);
            if let Some(target) = self.get_return_type_of_signature(contextual) {
                self.pending_annotated_param_inferences
                    .push((source, target));
            }
        }
    }

    pub(crate) fn build_overload_function_type(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("build_overload_function_type"); 
        let fn_decls: Vec<Arc<Node>> = symbol
            .declarations
            .iter()
            .filter(|d| d.kind == SyntaxKind::FunctionDeclaration)
            .cloned()
            .collect();
        if fn_decls.len() <= 1 {
            return None;
        }

        let mut signatures: Vec<Arc<Signature>> = Vec::new();
        for (i, decl) in symbol.declarations.iter().enumerate() {
            if decl.kind != SyntaxKind::FunctionDeclaration {
                continue;
            }
            let has_body = match &decl.data {
                tsox_frontend::ast::NodeData::FunctionDeclaration(data) => data.body.is_some(),
                _ => false,
            };
            // Go getSignaturesOfSymbol（checker.go:20993-21002）：有体声明仅当
            // 上一声明同父同 Kind 且 pos==end（或 Reparsed）才视作实现体剔除；
            // 跨 namespace 块合并的实现不邻接，签名照常入集
            if i > 0 && has_body {
                let previous = &symbol.declarations[i - 1];
                let same_parent =
                    decl.parent().map(|p| p.id()) == previous.parent().map(|p| p.id());
                if same_parent
                    && decl.kind == previous.kind
                    && (decl.pos() == previous.end()
                        || previous.flags.intersects(NodeFlags::Reparsed))
                {
                    continue;
                }
            }
            let (parameters, type_node, body, is_generator) = match &decl.data {
                tsox_frontend::ast::NodeData::FunctionDeclaration(data) => (
                    &data.parameters,
                    data.type_node.as_ref(),
                    data.body.as_ref(),
                    data.asterisk_token.is_some(),
                ),
                _ => continue,
            };

            self.push_scope(decl);
            let is_async_fn = decl.has_syntactic_modifier(ModifierFlags::Async);
            let return_type = match (type_node, body) {
                (Some(tn), _) => self.get_type_from_type_node(tn),
                (None, Some(body)) => {
                    // Go getReturnTypeOfSignature（checker.go:21217-21225）：无注解
                    // 有体签名从体推断（空体 void），体缺失（ambient 重载）才 any；
                    // 环护栏同 get_type_of_function_like
                    if self.push_signature_return_resolution(Arc::as_ptr(decl)) {
                        self.set_signature_return_inference_phase(Arc::as_ptr(decl), true);
                        let t = if is_generator {
                            self.infer_generator_return_type(body, is_async_fn)
                        } else {
                            self.infer_function_return_type(Some(decl), Some(body), None)
                        };
                        self.set_signature_return_inference_phase(Arc::as_ptr(decl), false);
                        if self.pop_signature_return_resolution() {
                            t
                        } else {
                            self.report_signature_return_circularity(decl);
                            self.get_any_type()
                        }
                    } else {
                        self.partial_type_of_function_like(decl)
                    }
                }
                (None, None) => self.get_any_type(),
            };
            let sig = self.build_signature_from_function_like_type_node(
                parameters,
                return_type,
                false,
                None,
                Some(Arc::clone(decl)),
            );
            self.pop_scope();
            signatures.push(sig);
        }
        if signatures.is_empty() {
            return None;
        }
        Some(self.create_function_or_constructor_type(signatures, false))
    }

    pub(crate) fn type_of_function_symbol_for_type_query(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("type_of_function_symbol_for_type_query"); 
        let sym_key = Arc::as_ptr(symbol) as usize;
        if let Some(shell) = self.fn_typequery_shells.get(&sym_key) {
            return Arc::clone(shell);
        }
        let shell = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: crate::checker::types::ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: TypeData::Object(crate::checker::types::ObjectTypeData::default()),
        });
        self.fn_typequery_shells.insert(sym_key, Arc::clone(&shell));
        let base = match self.build_overload_function_type(symbol) {
            Some(t) => t,
            None => match symbol
                .declarations
                .iter()
                .find(|d| d.kind == SyntaxKind::FunctionDeclaration)
                .cloned()
            {
                Some(d) => self.get_type_of_function_like(&d),
                None => {
                    self.fn_typequery_shells.remove(&sym_key);
                    return shell;
                }
            },
        };
        self.fn_typequery_shells.remove(&sym_key);
        if let TypeData::Object(src) = &base.data {
            let mut structured = crate::checker::types::StructuredTypeData::default();
            structured.signatures = src.structured.signatures.clone();
            structured.call_signature_count = src.structured.call_signature_count;
            let object = crate::checker::types::ObjectTypeData { node: None,
                structured,
                target: src.target.clone(),
                mapper: src.mapper.clone(),
                type_arguments: src.type_arguments.clone(),
            };
            let shell_ptr = Arc::as_ptr(&shell) as *mut Type;
            unsafe {
                (*shell_ptr).data = TypeData::Object(object);
            }
        }
        let shell_key = Arc::as_ptr(&shell) as usize;
        self.fn_typequery_shell_ptrs.insert(shell_key);
        let links = self.value_symbol_links.get_or_default(symbol);
        if links.resolved_type.is_none() {
            links.resolved_type = Some(Arc::clone(&shell));
        }
        shell
    }

    /// Go getTypeOfFuncClassEnumModule（checker.go:17888-17894）：Function 符号
    /// 值型即时驻留匿名壳（links.resolvedType 先于成员解析），getReturnTypeFromBody
    /// 的体自引（return somefn）经 getTypeOfSymbol（checker.go:17482-17484 Function
    /// 位分派）命中已驻留壳，签名返回型即壳自身。本侧壳登记复用 fn_typequery_shells
    /// 窗口表；推断窗口内壳先装入占位签名（Go 建签在途时 getSignaturesOfType 命中
    /// 惰性签名），终态推断直走体推断（不经 get_type_of_function_like 重入门——
    /// 壳建立于 decl 自身体推断途中，重入门恒拦截产占位），回填覆写占位
    pub(crate) fn type_of_function_expression_symbol_shell(
        &mut self,
        symbol: &Arc<Symbol>,
        decl: &Arc<Node>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("type_of_function_expression_symbol_shell");
        if let Some(t) = self
            .value_symbol_links
            .get(symbol)
            .and_then(|l| l.resolved_type.clone())
        {
            return t;
        }
        let sym_key = Arc::as_ptr(symbol) as usize;
        if let Some(shell) = self.fn_typequery_shells.get(&sym_key) {
            return Arc::clone(shell);
        }
        let shell = Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: crate::checker::types::ObjectFlags::Anonymous,
            id: crate::checker::types::next_type_id(),
            symbol: Some(Arc::clone(symbol)),
            alias: None,
            data: crate::checker::types::TypeData::Object(
                crate::checker::types::ObjectTypeData::default(),
            ),
        });
        self.fn_typequery_shells.insert(sym_key, Arc::clone(&shell));
        let links = self.value_symbol_links.get_or_default(symbol);
        if links.resolved_type.is_none() {
            links.resolved_type = Some(Arc::clone(&shell));
        }
        let primed = self.partial_type_of_function_like(decl);
        if let crate::checker::types::TypeData::Object(src) = &primed.data {
            let mut structured = crate::checker::types::StructuredTypeData::default();
            structured.signatures = src.structured.signatures.clone();
            structured.call_signature_count = src.structured.call_signature_count;
            let object = crate::checker::types::ObjectTypeData { node: None,
                structured,
                target: src.target.clone(),
                mapper: src.mapper.clone(),
                type_arguments: src.type_arguments.clone(),
            };
            let shell_ptr = Arc::as_ptr(&shell) as *mut Type;
            unsafe {
                (*shell_ptr).data = crate::checker::types::TypeData::Object(object);
            }
        }
        let base = self.get_type_of_function_like_inner(decl);
        self.fn_typequery_shells.remove(&sym_key);
        if let crate::checker::types::TypeData::Object(src) = &base.data {
            let mut structured = crate::checker::types::StructuredTypeData::default();
            structured.signatures = src.structured.signatures.clone();
            structured.call_signature_count = src.structured.call_signature_count;
            let object = crate::checker::types::ObjectTypeData { node: None,
                structured,
                target: src.target.clone(),
                mapper: src.mapper.clone(),
                type_arguments: src.type_arguments.clone(),
            };
            let shell_ptr = Arc::as_ptr(&shell) as *mut Type;
            unsafe {
                (*shell_ptr).data = crate::checker::types::TypeData::Object(object);
            }
        }
        let shell_key = Arc::as_ptr(&shell) as usize;
        self.fn_typequery_shell_ptrs.insert(shell_key);
        shell
    }

    pub fn get_type_of_class_declaration(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_class_declaration");
        let members = match &node.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(data) => Arc::clone(&data.members),
            tsox_frontend::ast::NodeData::ClassExpression(data) => Arc::clone(&data.members),
            _ => return self.get_any_type(),
        };

        if let Some(links) = self.type_node_links.get(node) {
            if let Some(ref t) = links.resolved_type {
                // 类声明节点型恒为急建构造型（build_type_of_class_declaration
                // 产物；Go checker.go:17897-17919 类符号型恒非 any）——any 驻留
                // 是建型窗口占位，视同未解析触发重算并覆写
                if !t.flags.contains(TypeFlags::Any) {
                    return Arc::clone(t);
                }
            }
        }

        let node_id = node.id();
        if self.class_type_resolution_stack.contains(&node_id) {
            // Go getTypeOfFuncClassEnumModuleWorker：类符号型在建期重入先经
            // getBaseTypeVariableOfClass → getBaseConstructorTypeOfClass 重推
            // (symbol, ResolvedBaseConstructorType) 帧，pushTypeResolution 发现
            // 同帧在途即抓环标记（checker.go:19867），外层 popTypeResolution
            // 失败后发 TS2506（checker.go:17965）
            if let Some(sym) = self.program.symbol_map().symbol_of(node).cloned() {
                if sym.flags.contains(tsox_frontend::ast::SymbolFlags::Class) {
                    let _ = self.get_base_type_variable_of_class(&sym);
                }
            }
            return self.get_any_type();
        }
        self.class_type_resolution_stack.push(node_id);
        let result = self.build_type_of_class_declaration(node, &members);
        // Go getTypeOfFuncClassEnumModuleWorker（checker.go:17904-17911）：类符号
        // 值型基构造型变量存在（mixin 形态）时为匿名片构型与基构造型变量的
        // 交集型（{ new (...): ...; prototype: ...; } & TBase）。基构造型变量
        // 取节点直算版（get_base_type_variable_of_class_node）——符号→声明型
        // 间接链（get_base_type_variable_of_class）在本例三轮全量中恒 None，
        // 见 progress_notes.md
        let result = match self.program.symbol_map().symbol_of(node).cloned() {
            Some(class_sym)
                if class_sym.flags.contains(tsox_frontend::ast::SymbolFlags::Class) =>
            {
                match self.get_base_type_variable_of_class_node(node) {
                    Some(base_type_variable) => {
                        self.get_intersection_type(vec![result, base_type_variable])
                    }
                    None => result,
                }
            }
            _ => result,
        };
        self.class_type_resolution_stack.pop();
        self.type_node_links.get_or_default(node).resolved_type = Some(result.clone());
        result
    }
}
