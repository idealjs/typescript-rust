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
                    index_infos: Vec::new(),
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
        let is_async_fn = node.has_syntactic_modifier(ModifierFlags::Async);
        self.set_signature_return_inference_phase(Arc::as_ptr(node), true);
        let return_type = if is_generator && type_node.is_none() && body.is_some() {
            self.infer_generator_return_type(body.unwrap(), is_async_fn)
        } else {
            self.infer_function_return_type(Some(node), body, type_node)
        };
        self.set_signature_return_inference_phase(Arc::as_ptr(node), false);
        if is_arrow {
            self.pop_arrow_function_scope();
        } else {
            self.pop_function_scope();
        }

        let sig = self.build_signature_from_function_like_type_node(
            parameters,
            return_type,
            false,
            contextual_signature,
            Some(Arc::clone(node)),
        );

        if !sig.type_parameters.is_empty()
            && let Some(contextual) = contextual_signature
        {
            if contextual.type_parameters.is_empty() {
                let inst = self.instantiate_signature_in_context_of(&sig, contextual);
                return self.create_function_or_constructor_type(vec![inst], false);
            }
        }
        self.create_function_or_constructor_type(vec![sig], false)
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
        for decl in &fn_decls {
            let has_body = match &decl.data {
                tsox_frontend::ast::NodeData::FunctionDeclaration(data) => data.body.is_some(),
                _ => false,
            };
            if has_body {
                continue;
            }
            let (parameters, type_node) = match &decl.data {
                tsox_frontend::ast::NodeData::FunctionDeclaration(data) => {
                    (&data.parameters, data.type_node.as_ref())
                }
                _ => continue,
            };

            self.push_scope(decl);
            let return_type = match type_node {
                Some(tn) => self.get_type_from_type_node(tn),
                None => self.get_any_type(),
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

    pub fn get_type_of_class_declaration(&mut self, node: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_type_of_class_declaration"); 
        let members = match &node.data {
            tsox_frontend::ast::NodeData::ClassDeclaration(data) => Arc::clone(&data.members),
            tsox_frontend::ast::NodeData::ClassExpression(data) => Arc::clone(&data.members),
            _ => return self.get_any_type(),
        };

        if let Some(links) = self.type_node_links.get(node) {
            if let Some(ref t) = links.resolved_type {
                return Arc::clone(t);
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
        self.class_type_resolution_stack.pop();
        self.type_node_links.get_or_default(node).resolved_type = Some(result.clone());
        result
    }
}
