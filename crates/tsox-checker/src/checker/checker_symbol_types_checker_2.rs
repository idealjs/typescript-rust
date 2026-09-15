#![allow(unused_imports)]

use crate::checker::checker_symbol_types::*;

enum BindingPathSeg {
    Prop(String, bool),
    Index(usize),
}

impl Checker {
    pub(crate) fn resolve_symbol_declared_type_on_demand(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Type>> {
        use tsox_frontend::ast::NodeData;
        // 合并符号（UMD 全局 export as namespace + declare global 变量）声明
        // 列表混有非变量声明：优先取变量/属性/参数类声明
        let decl = symbol
            .declarations
            .iter()
            .find(|d| {
                matches!(
                    d.data,
                    NodeData::VariableDeclaration(_)
                        | NodeData::PropertyDeclaration(_)
                        | NodeData::PropertySignatureDeclaration(_)
                        | NodeData::ParameterDeclaration(_)
                        | NodeData::BindingElement(_)
                        | NodeData::EnumMember(_)
                        | NodeData::JsxAttribute(_)
                )
            })
            .cloned()
            .or_else(|| symbol.value_declaration.clone())
            .or_else(|| symbol.declarations.first().cloned())?;
        let type_node_and_init: (Option<Arc<Node>>, Option<Arc<Node>>) = match &decl.data {
            // 函数表达式/箭头函数变量（const getProps = () => {}）：按需建型并挂 expando
            NodeData::VariableDeclaration(d)
                if d.type_node.is_none()
                    && d.initializer.as_ref().is_some_and(|init| {
                        matches!(
                            init.kind,
                            SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction
                        )
                    }) =>
            {
                let init = d.initializer.clone().expect("checked above");
                let base = self.get_type_of_node(&init);
                let t = self.attach_function_expando_type(symbol, base);
                self.type_node_links.get_or_default(&decl).resolved_type = Some(Arc::clone(&t));
                return Some(t);
            }
            NodeData::VariableDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
            NodeData::PropertyDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
            NodeData::PropertySignatureDeclaration(d) => {
                (Some(Arc::clone(&d.type_node)), None)
            }
            NodeData::ParameterDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
            // 局部函数声明：按需建函数型（外层返回推断在体检查前消费标识符引用），
            // 与 check_function_declaration 同步挂 expando 属性（binder 期 exports 已就绪）
            NodeData::FunctionDeclaration(_) => {
                let base = self.get_type_of_function_like(&decl);
                let t = self.attach_function_expando_type(symbol, base);
                self.type_node_links.get_or_default(&decl).resolved_type = Some(Arc::clone(&t));
                return Some(t);
            }
            // 局部类声明：按需建实例型（return new C() 在体检查前消费）
            NodeData::ClassDeclaration(_) => {
                let t = self.get_type_of_class_declaration(&decl);
                self.type_node_links.get_or_default(&decl).resolved_type = Some(Arc::clone(&t));
                return Some(t);
            }
            // Go checkJsxAttribute：有初始化式按可变位置（fresh 字面量拓宽），
            // 无初始化式是 true 语法糖
            NodeData::JsxAttribute(d) => {
                return match d.initializer.as_ref() {
                    Some(init) => {
                        // {expr} 的初始化式是 JsxExpression 包装
                        let expr = match &init.data {
                            NodeData::JsxExpression(je) => je.expression.clone(),
                            _ => Some(Arc::clone(init)),
                        };
                        let t = match expr {
                            Some(e) => self.get_type_of_node(&e),
                            None => self.get_any_type(),
                        };
                        Some(self.get_widened_type(&t))
                    }
                    None => Some(self.true_type()),
                };
            }
            NodeData::BindingElement(_) => return self.binding_element_type(&decl),
            NodeData::EnumMember(_) => {
                let value = self.get_enum_member_value(&decl).value?;
                let t = match value {
                    tsox_frontend::evaluator::EvalValue::String(s) => {
                        self.get_string_literal_type(&s)
                    }
                    tsox_frontend::evaluator::EvalValue::Number(n) => {
                        self.get_number_literal_type(n)
                    }
                    _ => return None,
                };
                return Some(t);
            }
            _ => return None,
        };
        if type_node_and_init.0.is_none() && type_node_and_init.1.is_none() {
            // 无注解参数：上下文定型（IIFE 实参 / 调用上下文签名），rest 参数优先走上下文
            if decl.kind == SyntaxKind::Parameter {
                // JS 无注解形参：@param 标签充当注解
                //（Go getTypeForVariableLikeDeclaration 的 jsdoc 通道）
                if let Some(t) = self.jsdoc_type_annotation(&decl) {
                    self.value_symbol_links.get_or_default(symbol).resolved_type =
                        Some(Arc::clone(&t));
                    return Some(t);
                }
                let placeholder = self.error_type();
                let existing = self
                    .value_symbol_links
                    .get_or_default(symbol)
                    .resolved_type
                    .replace(placeholder);
                let t = self.contextual_type_of_parameter(&decl);
                match &t {
                    Some(t) => {
                        self.value_symbol_links.get_or_default(symbol).resolved_type =
                            Some(Arc::clone(t));
                    }
                    None => {
                        self.value_symbol_links.get_or_default(symbol).resolved_type = existing;
                    }
                }
                if t.is_some() {
                    return t;
                }
            }
            // rest 参数无注解：元素类型数组（无上下文时 any[]）
            if let NodeData::ParameterDeclaration(d) = &decl.data {
                if d.dot_dot_dot_token.is_some() {
                    let elem = self.get_any_type();
                    return Some(self.create_array_type(elem));
                }
            }
            // catch 子句变量无注解无初始化：unknown（useUnknownInCatchVariables 默认）
            if decl.kind == SyntaxKind::VariableDeclaration
                && decl
                    .parent()
                    .as_ref()
                    .is_some_and(|p| p.kind == SyntaxKind::CatchClause)
            {
                return Some(self.get_unknown_type());
            }
            if decl.kind == SyntaxKind::VariableDeclaration {
                let placeholder = self.error_type();
                let existing = self
                    .value_symbol_links
                    .get_or_default(symbol)
                    .resolved_type
                    .replace(placeholder);
                let t = self.initial_type_of_declaration(&decl);
                // expando：函数初始化式的变量携带函数体属性赋值（binder exports）
                let t = t.map(|t| {
                    let is_fn_init = matches!(
                        &decl.data,
                        NodeData::VariableDeclaration(d)
                            if d.initializer.as_ref().is_some_and(|i| matches!(
                                i.kind,
                                SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction
                            ))
                    );
                    if is_fn_init && !symbol.exports.is_empty() {
                        self.attach_function_expando_type(symbol, t)
                    } else {
                        t
                    }
                });
                // 环污染：结果成员含 error 标记（重入产物）→ 整体丢弃返 any
                let polluted = t.as_ref().is_some_and(|t| {
                    t.as_structured().is_some_and(|s| {
                        s.properties.iter().any(|p| {
                            self.value_symbol_links
                                .get(p)
                                .and_then(|l| l.resolved_type.as_ref())
                                .is_some_and(|t| crate::checker::utilities::is_type_error(t))
                        })
                    })
                });
                if polluted {
                    return Some(self.get_any_type());
                }
                match &t {
                    Some(t) => {
                        self.value_symbol_links.get_or_default(symbol).resolved_type =
                            Some(Arc::clone(t));
                    }
                    None => {
                        self.value_symbol_links.get_or_default(symbol).resolved_type = existing;
                    }
                }
                return t;
            }
            return None;
        }

        let placeholder = self.error_type();
        let existing = self
            .value_symbol_links
            .get_or_default(symbol)
            .resolved_type
            .replace(placeholder);
        let result = self.with_declaring_file_context(&decl, |checker| {
            let (type_node, initializer) = match &decl.data {
                NodeData::VariableDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
                NodeData::PropertyDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
                NodeData::PropertySignatureDeclaration(d) => {
                (Some(Arc::clone(&d.type_node)), None)
            }
                NodeData::ParameterDeclaration(d) => (d.type_node.clone(), d.initializer.clone()),
                _ => (None, None),
            };
            if let Some(tn) = type_node {
                Some(checker.get_type_from_type_node(&tn))
            } else if decl.kind == SyntaxKind::Parameter {
                // Go getTypeForVariableLikeDeclaration：参数先上下文定型，无果才用 initializer
                let param = Arc::clone(&decl);
                checker
                    .contextual_type_of_parameter(&param)
                    .or_else(|| checker.initial_type_of_declaration(&decl))
            } else {
                let owner_class = match &decl.data {
                    NodeData::PropertyDeclaration(_) => decl
                        .parent()
                        .as_ref()
                        .filter(|p| {
                            matches!(
                                p.kind,
                                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
                            )
                        })
                        .cloned(),
                    _ => None,
                };
                if let Some(class) = owner_class.as_ref() {
                    let this_type = checker.build_class_instance_type_with_base(class);
                    checker.this_type_stack.push(this_type);
                }
                let t = initializer.map(|init| {
                    if !checker
                        .get_combined_node_flags(&decl)
                        .intersects(NodeFlags::Constant)
                        && matches!(
                            init.kind,
                            SyntaxKind::NullKeyword | SyntaxKind::UndefinedKeyword
                        )
                    {
                        return checker.auto_type();
                    }
                    if checker.is_empty_array_literal(&init) {
                        return checker.auto_array_type();
                    }
                    let raw = checker.get_type_of_node(&init);
                    let widened_literal =
                        checker.get_widened_literal_type_for_initializer(&decl, &raw);
                    let regularized = checker.get_regular_type_of_literal_type(&widened_literal);
                    checker.widen_initializer_type(&regularized)
                });
                if owner_class.is_some() {
                    checker.this_type_stack.pop();
                }
                t
            }
        });

        let result = match (&result, &decl.data) {
            (Some(t), NodeData::ParameterDeclaration(pd))
                if pd.question_token.is_some() && pd.initializer.is_none() =>
            {
                Some(self.add_optional_undefined(Arc::clone(t)))
            }

            (Some(t), NodeData::PropertySignatureDeclaration(psd))
                if psd
                    .postfix_token
                    .as_ref()
                    .is_some_and(|tk| tk.kind == SyntaxKind::QuestionToken) =>
            {
                Some(self.get_optional_type(Arc::clone(t)))
            }
            _ => result,
        };
        match &result {
            Some(t) => {
                // 递归类型在构建窗口内经环断路器拿到 in-flight error：不驻留，
                // 恢复窗口前状态（占位 any 泄漏会把合并符号永久定格为 any，
                // 如 UMD 全局 + declare global const 的 typeof 链）
                if !crate::checker::utilities::is_type_error(t) {
                    self.value_symbol_links.get_or_default(symbol).resolved_type =
                        Some(Arc::clone(t));
                } else {
                    self.value_symbol_links.get_or_default(symbol).resolved_type = existing;
                }
            }
            None => {
                self.value_symbol_links.get_or_default(symbol).resolved_type = existing;
            }
        }
        result
    }

    /// 绑定元素解析属性时，把源类型的属性符号挂为 container（显示限定名用）。
    fn link_binding_element_container(&mut self, elem: &Arc<Node>, t: &Arc<Type>, name: &str) {
        let Some(sym) = t.symbol.clone() else { return };
        let member = self
            .resolve_interface_type_ex(&sym, None)
            .as_structured()
            .and_then(|s| s.members.get(name).cloned())
            .or_else(|| sym.members.entries.get(name).cloned());
        let Some(member) = member else { return };
        if let Some(s) = self.program.symbol_map().symbol_of(elem) {
            let links = self.value_symbol_links.get_or_default(s);
            if links.container_symbol.is_none() {
                links.container_symbol = Some(Arc::clone(&member));
            }
        }
        // 成员自身的限定容器 = 根类型符号（如 I），供 qualified_symbol_name 使用
        if member.parent().is_none() {
            let mlinks = self.value_symbol_links.get_or_default(&member);
            if mlinks.container_symbol.is_none() {
                mlinks.container_symbol = Some(sym);
            }
        }
    }

    /// 绑定元素类型：沿模式链上行到根声明取类型，再按属性/索引路径逐层查。
    fn binding_element_type(&mut self, elem: &Arc<Node>) -> Option<Arc<Type>> {
        use tsox_frontend::ast::NodeData;
        let mut path: Vec<BindingPathSeg> = Vec::new();
        let mut cur = Arc::clone(elem);
        loop {
            match &cur.data {
                NodeData::BindingElement(d) => {
                    let parent_kind = cur.parent().as_ref().map(|p| p.kind);
                    if parent_kind == Some(tsox_frontend::ast::SyntaxKind::ArrayBindingPattern) {
                        let pattern = cur.parent().expect("checked kind above");
                        let index = match &pattern.data {
                            NodeData::BindingPattern(bp) => bp
                                .elements
                                .iter()
                                .position(|e| Arc::ptr_eq(e, &cur)),
                            _ => None,
                        }?;
                        let renamed = d.property_name.as_ref().and_then(|n| match &n.data {
                            NodeData::NumericLiteral(num) => num.text.parse::<usize>().ok(),
                            _ => None,
                        });
                        path.push(BindingPathSeg::Index(renamed.unwrap_or(index)));
                    } else {
                        let renamed = d.property_name.as_ref().and_then(|n| match &n.data {
                            NodeData::Identifier(i) => Some(i.text.clone()),
                            NodeData::StringLiteral(s) => Some(s.text.clone()),
                            NodeData::NumericLiteral(num) => Some(num.text.clone()),
                            _ => None,
                        });
                        let seg = renamed.clone().or_else(|| {
                            d.name.as_ref().and_then(|n| match &n.data {
                                NodeData::Identifier(i) => Some(i.text.clone()),
                                _ => None,
                            })
                        })?;
                        path.push(BindingPathSeg::Prop(seg, renamed.is_some()));
                    }
                    cur = Arc::clone(cur.parent().as_ref()?);
                }
                NodeData::BindingPattern(_) => {
                    cur = Arc::clone(cur.parent().as_ref()?);
                }
                NodeData::ParameterDeclaration(d) => {
                    let mut t = match &d.type_node {
                        Some(tn) => self.get_type_from_type_node(tn),
                        None => {
                            let param = Arc::clone(&cur);
                            self.contextual_type_of_parameter(&param)?
                        }
                    };
                    for seg in path.iter().rev() {
                        t = self.binding_path_step(elem, t, seg)?;
                    }
                    return Some(t);
                }
                NodeData::VariableDeclaration(d) => {
                    let mut t = match (&d.type_node, &d.initializer) {
                        (Some(tn), _) => self.get_type_from_type_node(tn),
                        (None, Some(init)) => self.get_type_of_node(init),
                        _ => return None,
                    };
                    for seg in path.iter().rev() {
                        t = self.binding_path_step(elem, t, seg)?;
                    }
                    return Some(t);
                }
                _ => return None,
            }
        }
    }

    // 所在调用带显式类型实参时按实参实例化上下文签名（Go inferSignature：
    // 显式实参直接固定映射；错误实参落 error 型，参数位显示 any）
    fn substitute_explicit_call_type_args(
        &mut self,
        call: &Arc<Node>,
        sig: &Arc<Signature>,
    ) -> Arc<Signature> {
        if sig.type_parameters.is_empty() {
            return Arc::clone(sig);
        }
        let ta = match &call.data {
            NodeData::CallExpression(d) => d.type_arguments.as_ref(),
            NodeData::NewExpression(d) => d.type_arguments.as_ref(),
            _ => None,
        };
        let Some(ta) = ta else {
            return Arc::clone(sig);
        };
        if ta.len() != sig.type_parameters.len() {
            return Arc::clone(sig);
        }
        let args: Vec<Arc<Type>> = ta
            .iter()
            .map(|t| self.get_type_from_type_node(t))
            .collect();
        self.get_signature_instantiation(sig, &args)
    }

    fn contextual_type_of_parameter(
        &mut self,
        param: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        use tsox_frontend::ast::NodeData;
        let host = param.parent()?;
        let (host_kind_ok, host_type_params, host_params) = match &host.data {
            NodeData::FunctionExpression(d) => (true, d.type_parameters.clone(), Some(&d.parameters)),
            NodeData::ArrowFunction(d) => (true, d.type_parameters.clone(), Some(&d.parameters)),
            NodeData::FunctionDeclaration(d) => (true, d.type_parameters.clone(), Some(&d.parameters)),
            NodeData::MethodDeclaration(d) => (true, d.type_parameters.clone(), Some(&d.parameters)),
            _ => (false, None, None),
        };
        if !host_kind_ok || host_type_params.is_some() {
            return None;
        }
        let host_params = host_params?;
        let param_index = host_params.iter().position(|p| Arc::ptr_eq(p, param));
        let param_index = param_index?;
        // 对象字面量方法：上下文签名取字面量上下文类型的同名属性（Go getContextualSignatureForObjectLiteralMethod）
        if matches!(host.data, NodeData::MethodDeclaration(_)) {
            let obj_lit = host.parent()?;
            if obj_lit.kind != SyntaxKind::ObjectLiteralExpression {
                return None;
            }
            let ctx = self.get_contextual_type(&obj_lit, ContextFlags::None)?;
            let method_name = match &host.data {
                NodeData::MethodDeclaration(d) => self.get_property_name_from_node(&d.name),
                _ => String::new(),
            };
            let prop_type = self.get_type_of_property_of_contextual_type(&ctx, &method_name)?;
            let sigs =
                self.get_signatures_of_type(&prop_type, crate::checker::SignatureKind::Call);
            let sig = sigs.first()?.clone();
            let is_rest = matches!(&param.data, NodeData::ParameterDeclaration(pd) if pd.dot_dot_dot_token.is_some());
            let is_this_param = param_index == 0
                && matches!(&param.data, NodeData::ParameterDeclaration(pd)
                    if matches!(&pd.name.data, NodeData::Identifier(id) if id.text == "this")
                        || pd.name.kind == SyntaxKind::ThisKeyword);
            return self
                .contextual_param_type_at(&sig, &host_params, param_index, param, is_rest, is_this_param)
                .into();
        }
        let mut call = host.parent()?;
        let mut in_parens = false;
        while call.kind == tsox_frontend::ast::SyntaxKind::ParenthesizedExpression {
            in_parens = true;
            call = call.parent()?;
        }
        let call_ctx = match &call.data {
            NodeData::CallExpression(d) => {
                // IIFE：参数类型取对应实参的 widened 类型（Go GetImmediatelyInvokedFunctionExpression 分支）
                let callee_is_host = (!in_parens && Arc::ptr_eq(&d.expression, &host))
                    || (in_parens
                        && host
                            .parent()
                            .as_ref()
                            .is_some_and(|p| Arc::ptr_eq(p, &d.expression)));
                if callee_is_host {
                    let call_id = call.id();
                    if !self.resolving_contextual_calls.insert(call_id) {
                        return None;
                    }
                    let args: Vec<Arc<Node>> = d.arguments.iter().cloned().collect();
                    let result = (|| {
                        if param_index < args.len() {
                            let raw = self.get_type_of_node(&args[param_index]);
                            let widened = self.widen_argument_type_deep(&raw);
                            return Some(widened);
                        }
                        if host_params.iter().any(|p| {
                            matches!(&p.data, NodeData::ParameterDeclaration(pd) if pd.initializer.is_some())
                        }) {
                            return None;
                        }
                        Some(self.undefined_type())
                    })();
                    self.resolving_contextual_calls.remove(&call_id);
                    return result;
                }
                self.get_contextual_type_for_argument(&call, &host)
            }
            NodeData::NewExpression(_) => self.get_contextual_type_for_argument(&call, &host),
            _ => None,
        };
        let ctx = call_ctx.or_else(|| self.get_contextual_type(&host, ContextFlags::None))?;
        let sigs = self.get_signatures_of_type(&ctx, crate::checker::SignatureKind::Call);
        let sig = self.substitute_explicit_call_type_args(&call, &sigs.first()?.clone());
        let is_rest = matches!(&param.data, NodeData::ParameterDeclaration(pd) if pd.dot_dot_dot_token.is_some());
        let is_this_param = param_index == 0
            && matches!(&param.data, NodeData::ParameterDeclaration(pd)
                if matches!(&pd.name.data, NodeData::Identifier(id) if id.text == "this")
                    || pd.name.kind == SyntaxKind::ThisKeyword);
        self.contextual_param_type_at(&sig, &host_params, param_index, param, is_rest, is_this_param)
    }

    fn widen_argument_type_deep(&mut self, t: &Arc<Type>) -> Arc<Type> {
        let widened = self.get_widened_type(t);
        let TypeData::Object(o) = &widened.data else {
            return widened;
        };
        if widened.symbol.is_some() {
            return widened;
        }
        let mut new_props: Vec<Arc<Symbol>> = Vec::with_capacity(o.structured.properties.len());
        let mut new_members = SymbolTable::new();
        let mut changed = false;
        for p in &o.structured.properties {
            let pt = self.get_type_of_symbol(p);
            let wpt = self.widen_argument_type_deep(&pt);
            if Arc::ptr_eq(&pt, &wpt) {
                new_members.insert(p.name.clone(), Arc::clone(p));
                new_props.push(Arc::clone(p));
            } else {
                let np = Arc::new(Symbol::new(p.flags, p.name.clone()));
                self.value_symbol_links.insert(
                    &np,
                    ValueSymbolLinks {
                        resolved_type: Some(wpt),
                        ..Default::default()
                    },
                );
                new_members.insert(np.name.clone(), Arc::clone(&np));
                new_props.push(np);
                changed = true;
            }
        }
        if !changed {
            return widened;
        }
        let mut rebuilt = Type::new(
            widened.flags,
            TypeData::Object(ObjectTypeData {
                structured: StructuredTypeData {
                    members: new_members,
                    properties: new_props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        );
        rebuilt.object_flags = widened.object_flags;
        Arc::new(rebuilt)
    }

    fn binding_path_step(
        &mut self,
        elem: &Arc<Node>,
        t: Arc<Type>,
        seg: &BindingPathSeg,
    ) -> Option<Arc<Type>> {
        if let BindingPathSeg::Prop(name, renamed) = seg {
            if *renamed {
                self.link_binding_element_container(elem, &t, name);
            }
            return self.get_type_of_property_of_type(&t, name);
        }
        let BindingPathSeg::Index(index) = seg else {
            return None;
        };
        let index = *index;
        if let TypeData::Tuple(tuple) = &t.data {
            return tuple
                .element_infos
                .get(index)
                .and_then(|info| info.type_.clone());
        }
        if t.object_flags.contains(crate::checker::types::ObjectFlags::Tuple)
            && let Some(structured) = t.as_structured()
            && let Some(info) = structured.index_infos.first()
        {
            return info.value_type.clone();
        }
        if let Some(elem) = self.get_array_element_type_of(&t) {
            return Some(elem);
        }
        self.get_type_of_property_of_type(&t, &index.to_string())
    }

    pub(crate) fn with_declaring_file_context<T>(
        &mut self,
        decl: &Arc<Node>,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let saved_file = self.current_file.take();
        let saved_id = self.current_file_id;
        let saved_symbol = self.current_file_symbol.take();
        let mut pushed = 0usize;
        if let Some(file) = self.get_source_file_of_node(decl) {
            self.current_file = Some(Arc::clone(&file));
            self.current_file_id = file.node.id();
            self.current_file_symbol = self.program.symbol_map().symbol_of(&file.node).cloned();

            let mut chain: Vec<Arc<Node>> = Vec::new();
            let mut cur = decl.parent();
            while let Some(n) = cur {
                if matches!(
                    n.kind,
                    SyntaxKind::SourceFile
                        | SyntaxKind::ModuleDeclaration
                        | SyntaxKind::Block
                        | SyntaxKind::CatchClause
                        | SyntaxKind::ForStatement
                        | SyntaxKind::ForInStatement
                        | SyntaxKind::ForOfStatement
                        | SyntaxKind::FunctionDeclaration
                        | SyntaxKind::FunctionExpression
                        | SyntaxKind::ArrowFunction
                        | SyntaxKind::MethodDeclaration
                        | SyntaxKind::MethodSignature
                        | SyntaxKind::CallSignature
                        | SyntaxKind::ConstructSignature
                        | SyntaxKind::FunctionType
                        | SyntaxKind::ConstructorType
                        | SyntaxKind::Constructor
                        | SyntaxKind::GetAccessor
                        | SyntaxKind::SetAccessor
                        | SyntaxKind::InterfaceDeclaration
                        | SyntaxKind::ClassDeclaration
                        | SyntaxKind::ClassExpression
                        | SyntaxKind::TypeAliasDeclaration
                        | SyntaxKind::MappedType
                        | SyntaxKind::EnumDeclaration
                ) {
                    chain.push(Arc::clone(&n));
                    if n.kind == SyntaxKind::SourceFile {
                        break;
                    }
                }
                cur = n.parent();
            }
            for scope in chain.iter().rev() {
                self.push_scope(scope);
                pushed += 1;
            }
        }
        let result = f(self);
        for _ in 0..pushed {
            self.pop_scope();
        }
        self.current_file = saved_file;
        self.current_file_id = saved_id;
        self.current_file_symbol = saved_symbol;
        result
    }

    pub(crate) fn get_type_of_merged_namespace_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Arc<Type> {
        if let Some(cached) = self
            .declared_type_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())
        {
            return cached;
        }

        let value_type = self.get_value_type_of_symbol(symbol);

        let ns_type = self.resolve_namespace_type(symbol);

        let (call_sigs, construct_sigs) = match &value_type.data {
            TypeData::Object(obj) => {
                let cs = obj.structured.call_signatures().to_vec();
                let xs = obj.structured.construct_signatures().to_vec();
                (cs, xs)
            }
            _ => (Vec::new(), Vec::new()),
        };
        let merged = if call_sigs.is_empty() && construct_sigs.is_empty() {
            ns_type
        } else {
            let ns_obj = match &ns_type.data {
                TypeData::Object(obj) => obj,
                _ => {
                    self.declared_type_links
                        .get_or_default(symbol)
                        .declared_type = Some(Arc::clone(&value_type));
                    return value_type;
                }
            };
            let ns_structured = &ns_obj.structured;
            let mut structured = StructuredTypeData::default();
            structured.members = ns_structured.members.clone();
            structured.properties = ns_structured.properties.clone();
            structured.index_infos = ns_structured.index_infos.clone();

            let existing_sigs = ns_structured.signatures.clone();
            let existing_call_count = ns_structured.call_signature_count;
            structured.call_signature_count = call_sigs.len() + existing_call_count;
            structured.signatures = call_sigs;
            structured
                .signatures
                .extend(existing_sigs[..existing_call_count].to_vec());
            structured.signatures.extend(construct_sigs);
            structured
                .signatures
                .extend(existing_sigs[existing_call_count..].to_vec());
            Arc::new(Type {
                flags: TypeFlags::Object,
                object_flags: ObjectFlags::Anonymous,
                id: crate::checker::types::next_type_id(),
                symbol: Some(Arc::clone(symbol)),
                alias: None,
                data: TypeData::Object(ObjectTypeData {
                    structured,
                    target: None,
                    mapper: None,
                    type_arguments: Vec::new(),
                }),
            })
        };

        self.declared_type_links
            .get_or_default(symbol)
            .declared_type = Some(Arc::clone(&merged));
        merged
    }
}
