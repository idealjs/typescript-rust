#![allow(unused_imports)]

use crate::checker::checker_symbol_types::*;

enum BindingPathSeg {
    Prop(String, bool),
    Index(usize),
    Rest,
}

impl Checker {
    pub(crate) fn resolve_symbol_declared_type_on_demand(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Type>> {
        // Go getTypeOfVariableOrParameterOrPropertyWorker：变量类符号类型解析
        // 进 type resolution 栈，同符号重入即环（reportCircularityError）
        let uses_frame = symbol.declarations.iter().any(|d| {
            matches!(
                d.kind,
                SyntaxKind::Parameter
                    | SyntaxKind::PropertyDeclaration
                    | SyntaxKind::PropertySignature
                    | SyntaxKind::VariableDeclaration
                    | SyntaxKind::BindingElement
                    | SyntaxKind::ExportAssignment
                    | SyntaxKind::EnumMember
                    | SyntaxKind::JsxAttribute
            )
        });
        if !uses_frame {
            return self.resolve_symbol_declared_type_on_demand_inner(symbol);
        }
        if !self.push_type_resolution(
            Arc::as_ptr(symbol) as *const Symbol,
            crate::checker::TypeResolutionProperty::Type,
        ) {
            let plain_binding_element = symbol.declarations.iter().any(|d| {
                matches!(
                    &d.data,
                    NodeData::BindingElement(be) if be.initializer.is_none()
                )
            });
            if plain_binding_element {
                return Some(self.circular_constraint_type());
            }
            return Some(self.report_circularity_error(symbol));
        }
        let result = self.resolve_symbol_declared_type_on_demand_inner(symbol);
        if !self.pop_type_resolution() {
            return Some(self.report_circularity_error(symbol));
        }
        result
    }

    fn resolve_symbol_declared_type_on_demand_inner(
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
            // Go getTypeOfVariableOrParameterOrPropertyWorker：export= 符号
            // 类型 = 表达式（或注解型）拓宽
            NodeData::ExportAssignment(d) => {
                let t = if d.type_node.kind != SyntaxKind::MissingDeclaration {
                    self.get_type_from_type_node(&d.type_node)
                } else {
                    let expr_t = self.get_type_of_node(&d.expression);
                    self.get_widened_type(&expr_t)
                };
                self.value_symbol_links.get_or_default(symbol).resolved_type = Some(Arc::clone(&t));
                self.type_node_links.get_or_default(&decl).resolved_type = Some(Arc::clone(&t));
                return Some(t);
            }
            NodeData::EnumMember(_) => {
                // Go getTypeOfEnumMember → getDeclaredTypeOfEnumMember：成员值
                // 类型是枚举字面型（Foo.a），由枚举声明型解析填充成员链接
                if let Some(enum_decl) = decl.parent()
                    && let Some(enum_sym) = self
                        .program
                        .symbol_map()
                        .symbol_of(&enum_decl)
                        .map(Arc::clone)
                {
                    self.resolve_enum_type(&enum_sym);
                    if let Some(t) = self
                        .value_symbol_links
                        .get(symbol)
                        .and_then(|l| l.resolved_type.clone())
                    {
                        return Some(t);
                    }
                }
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
                    let owned =
                        checker.widen_unique_symbol_for_declaration(&decl, &raw);
                    let widened_literal =
                        checker.get_widened_literal_type_for_initializer(&decl, &owned);
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

    /// Go getBindingElementTypeFromParentType：strictNullChecks 下模式父声明
    /// 的初始化式不可能是 undefined 时，根类型剔除 undefined 成分
    pub(crate) fn filter_binding_parent_undefined(
        &mut self,
        decl: &Arc<Node>,
        t: Arc<Type>,
    ) -> Arc<Type> {
        if !self.strict_null_checks {
            return t;
        }
        let init = match &decl.data {
            NodeData::VariableDeclaration(d) => d.initializer.clone(),
            NodeData::ParameterDeclaration(d) => d.initializer.clone(),
            _ => None,
        };
        let Some(init) = init else {
            return t;
        };
        let init_t = self.get_type_of_node(&init);
        if Self::type_may_be_undefined(&init_t) {
            return t;
        }
        if t.is_union() {
            if let Some(members) = t.types() {
                let kept: Vec<Arc<Type>> = members
                    .iter()
                    .filter(|m| !Self::type_may_be_undefined(m))
                    .cloned()
                    .collect();
                return match kept.len() {
                    0 => self.never_type(),
                    1 => kept.into_iter().next().expect("nonempty"),
                    _ => self.get_union_type(kept),
                };
            }
            return t;
        }
        if Self::type_may_be_undefined(&t) {
            return self.never_type();
        }
        t
    }

    /// Go getTypeFactsWorker 的 EQUndefined 位：any/unknown/未定类型视为可能
    fn type_may_be_undefined(t: &Arc<Type>) -> bool {
        if t.flags.intersects(
            TypeFlags::Any
                | TypeFlags::Unknown
                | TypeFlags::Undefined
                | TypeFlags::TypeParameter
                | TypeFlags::Index
                | TypeFlags::Conditional,
        ) {
            return true;
        }
        t.is_union()
            && t.types()
                .is_some_and(|ts| ts.iter().any(Self::type_may_be_undefined))
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
    pub(crate) fn binding_element_type(&mut self, elem: &Arc<Node>) -> Option<Arc<Type>> {
        use tsox_frontend::ast::NodeData;
        if let Some(sym) = self.program.symbol_map().symbol_of(elem)
            && let Some(t) = self
                .value_symbol_links
                .get(sym)
                .and_then(|l| l.resolved_type.clone())
            && !crate::checker::utilities::is_type_error(&t)
        {
            return Some(t);
        }
        let mut path: Vec<(Arc<Node>, BindingPathSeg)> = Vec::new();
        let mut cur = Arc::clone(elem);
        loop {
            match &cur.data {
                NodeData::BindingElement(d) => {
                    let parent_kind = cur.parent().as_ref().map(|p| p.kind);
                    if d.dot_dot_dot_token.is_some() {
                        // rest 元素：类型是模式容器的“剩余部分”，不是属性查找
                        path.push((Arc::clone(&cur), BindingPathSeg::Rest));
                    } else if parent_kind == Some(tsox_frontend::ast::SyntaxKind::ArrayBindingPattern) {
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
                        path.push((Arc::clone(&cur), BindingPathSeg::Index(renamed.unwrap_or(index))));
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
                        path.push((Arc::clone(&cur), BindingPathSeg::Prop(seg, renamed.is_some())));
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
                            // Go getTypeForBindingElementParent：优先取上下文定型
                            // 已写入符号链接的参数型，未定型才落上下文重解析
                            let memoized = self
                                .program
                                .symbol_map()
                                .symbol_of(&cur)
                                .and_then(|s| {
                                    self.value_symbol_links
                                        .get(s)
                                        .and_then(|l| l.resolved_type.clone())
                                })
                                .filter(|t| !crate::checker::utilities::is_type_error(t));
                            match memoized {
                                Some(t) => t,
                                None => {
                                    let param = Arc::clone(&cur);
                                    match self.contextual_type_of_parameter(&param) {
                                        Some(t) => t,
                                        None => {
                                            // Go getTypeForVariableLikeDeclaration：参数无
                                            // 上下文类型时回退初始化式拓宽类型
                                            let Some(init) = &d.initializer else {
                                                return None;
                                            };
                                            let raw = self.get_type_of_node(init);
                                            let widened_literal =
                                                self.get_widened_literal_type_for_initializer(&cur, &raw);
                                            let regularized =
                                                self.get_regular_type_of_literal_type(&widened_literal);
                                            self.widen_initializer_type(&regularized)
                                        }
                                    }
                                }
                            }
                        }
                    };
                    t = self.filter_binding_parent_undefined(&cur, t);
                    for (seg_elem, seg) in path.iter().rev() {
                        t = self.binding_path_step(seg_elem, t, seg)?;
                    }
                    return Some(t);
                }
                NodeData::VariableDeclaration(d) => {
                    let mut t = match (&d.type_node, &d.initializer) {
                        (Some(tn), _) => self.get_type_from_type_node(tn),
                        (None, Some(init)) => {
                            let memoized = self
                                .program
                                .symbol_map()
                                .symbol_of(&cur)
                                .and_then(|s| {
                                    self.value_symbol_links
                                        .get(s)
                                        .and_then(|l| l.resolved_type.clone())
                                })
                                .filter(|t| !crate::checker::utilities::is_type_error(t));
                            match memoized {
                                Some(t) => t,
                                None => {
                                    let raw = self.get_type_of_node(init);
                                    // Go widenTypeInferredFromInitializer：解构根类型按
                                    // 初始化式拓宽（fresh 字面量 6 → number）
                                    let widened_literal =
                                        self.get_widened_literal_type_for_initializer(&cur, &raw);
                                    let regularized =
                                        self.get_regular_type_of_literal_type(&widened_literal);
                                    self.widen_initializer_type(&regularized)
                                }
                            }
                        }
                        // for-in/of 头声明无初始化式：迭代类型即根类型
                        //（Go getTypeForVariableLikeDeclaration 的 ForIn/ForOf 分支）
                        (None, None) => self.initial_type_of_declaration(&cur)?,
                    };
                    t = self.filter_binding_parent_undefined(&cur, t);
                    for (seg_elem, seg) in path.iter().rev() {
                        t = self.binding_path_step(seg_elem, t, seg)?;
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

    pub(crate) fn contextual_type_of_parameter(
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
            // Go getContextualSignatureForObjectLiteralMethod：getSingleSignature
            //（恰一个调用签名才用，多重载不给上下文参数型）
            if sigs.len() != 1 {
                return None;
            }
            let sig = sigs[0].clone();
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
                        if matches!(&param.data, NodeData::ParameterDeclaration(pd) if pd.initializer.is_some())
                        {
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
        // Go getContextuallyTypedParameterType：经 getContextualSignature 门控
        //（多重载且 noImplicitAny 关闭时无上下文签名，参数走隐式 any）
        let gated = self.get_contextual_call_signature(&ctx, &host)?;
        let sig = self.substitute_explicit_call_type_args(&call, &gated);
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

    /// 元素的 computed 属性名节点（仅显式 property_name 位）
    pub(crate) fn binding_element_computed_property_name(elem: &Arc<Node>) -> Option<Arc<Node>> {
        let tsox_frontend::ast::NodeData::BindingElement(be) = &elem.data else {
            return None;
        };
        let pn = be.property_name.as_ref()?;
        (pn.kind == tsox_frontend::ast::SyntaxKind::ComputedPropertyName)
            .then(|| Arc::clone(pn))
    }

    fn binding_path_step(
        &mut self,
        elem: &Arc<Node>,
        t: Arc<Type>,
        seg: &BindingPathSeg,
    ) -> Option<Arc<Type>> {
        // 带默认值的元素（a = expr）：类型由默认值提供，属性缺失不诊断
        //（Go getTypeForVariableLikeDeclaration 的 hasInitializer 分支）
        let elem_has_initializer = matches!(
            &elem.data,
            NodeData::BindingElement(d) if d.initializer.is_some()
        );
        let diagnostics_allowed = !self.in_ambient_declaration_context()
            && !elem_has_initializer
            && !t.flags.intersects(TypeFlags::Any | TypeFlags::Unknown | TypeFlags::Never)
            && !crate::checker::utilities::is_type_error(&t);
        if let BindingPathSeg::Rest = seg {
            return Some(self.rest_element_type(elem, &t));
        }
        if let BindingPathSeg::Prop(name, renamed) = seg {
            // Go isTypeUsableAsPropertyName：computed 名表达式解析失败（error 型）
            // 时属性查找不可用，错误已由名表达式自身报告
            if let Some(pn) = Self::binding_element_computed_property_name(elem)
                && let tsox_frontend::ast::NodeData::ComputedPropertyName(cd) = &pn.data
            {
                let name_expr_type = self.get_type_of_node(&cd.expression);
                if crate::checker::utilities::is_type_error(&name_expr_type) {
                    return None;
                }
                if crate::checker::utilities_token_is_identifier_or_keyword::is_type_usable_as_property_name(&name_expr_type)
                {
                    let derived = crate::checker::utilities_token_is_identifier_or_keyword::get_property_name_from_type(&name_expr_type);
                    if *renamed {
                        self.link_binding_element_container(elem, &t, &derived);
                    }
                    let result = self.get_type_of_property_of_type(&t, &derived);
                    if result.is_none() && diagnostics_allowed {
                        let display = self.boxed_declared_type_for_display(&t);
                        let type_str = self.type_to_string(&display);
                        let name_node = Self::binding_element_name_node(elem)
                            .unwrap_or_else(|| Arc::clone(elem));
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            name_node.loc,
                            tsox_core::diagnostics::messages_generated::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                            vec![derived, type_str],
                        ));
                    }
                    return result;
                }
                if name_expr_type.flags.intersects(TypeFlags::TypeParameter) {
                    let constraint = self
                        .get_constraint_of_type_parameter(&name_expr_type)
                        .or_else(|| self.get_base_constraint_of_type(&name_expr_type));
                    let index_like = constraint.as_ref().is_some_and(|c| {
                        c.flags.intersects(
                            TypeFlags::String | TypeFlags::Number | TypeFlags::ESSymbol,
                        )
                    });
                    if index_like {
                        return Some(self.get_indexed_access_type(&t, &name_expr_type));
                    }
                }
                // computed string 名走索引签名通道（Go getIndexedAccessTypeEx）：
                // 无匹配 string 索引签名报 TS2537，命中取值类型
                if name_expr_type.flags.intersects(TypeFlags::String) {
                    let structured = t.as_structured();
                    let match_info = structured.and_then(|s| {
                        s.index_infos.iter().find(|info| {
                            info.key_type
                                .as_ref()
                                .map(|k| k.flags.intersects(TypeFlags::String))
                                .unwrap_or(false)
                        })
                    });
                    if let Some(info) = match_info {
                        return info.value_type.clone();
                    }
                    if diagnostics_allowed {
                        let display = self.boxed_declared_type_for_display(&t);
                        let type_str = self.type_to_string(&display);
                        let key_str = self.type_to_string(&name_expr_type);
                        // Go 报错锚定计算名内的表达式节点
                        let anchor_loc = cd.expression.loc;
                        self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                            self.current_file.clone(),
                            anchor_loc,
                            tsox_core::diagnostics::messages_generated::
                                TYPE_0_HAS_NO_MATCHING_INDEX_SIGNATURE_FOR_TYPE_1,
                            vec![type_str, key_str],
                        ));
                    }
                    return None;
                }
                if name_expr_type.flags.intersects(TypeFlags::Number) {
                    let structured = t.as_structured();
                    let match_info = structured.and_then(|s| {
                        s.index_infos.iter().find(|info| {
                            info.key_type
                                .as_ref()
                                .map(|k| k.flags.intersects(TypeFlags::Number))
                                .unwrap_or(false)
                        })
                    });
                    if let Some(info) = match_info {
                        return info.value_type.clone();
                    }
                }
                if diagnostics_allowed {
                    let type_str = self.type_to_string(&name_expr_type);
                    let anchor_loc = cd.expression.loc;
                    self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                        self.current_file.clone(),
                        anchor_loc,
                        tsox_core::diagnostics::messages_generated::
                            TYPE_0_CANNOT_BE_USED_AS_AN_INDEX_TYPE,
                        vec![type_str],
                    ));
                }
                return Some(self.get_any_type());
            }
            if *renamed {
                self.link_binding_element_container(elem, &t, name);
            }
            if !Self::binding_element_computed_property_name(elem).is_some() {
                if let Some(prop) = self.get_property_of_type(&t, name) {
                    self.mark_property_as_referenced(&prop, None);
                }
            }
            let result = self.get_type_of_property_of_type(&t, name);
            if result.is_none() && diagnostics_allowed {
                let display = self.boxed_declared_type_for_display(&t);
                let type_str = self.type_to_string(&display);
                let name_node = Self::binding_element_name_node(elem)
                    .unwrap_or_else(|| Arc::clone(elem));
                self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                    self.current_file.clone(),
                    name_node.loc,
                    tsox_core::diagnostics::messages_generated::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                    vec![name.clone(), type_str],
                ));
            }
            return result;
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
        if let Some(prop) = self.get_type_of_property_of_type(&t, &index.to_string()) {
            return Some(prop);
        }
        if diagnostics_allowed {
            let pattern = elem
                .parent()
                .filter(|p| p.kind == tsox_frontend::ast::SyntaxKind::ArrayBindingPattern);
            return Some(self.check_iterated_type_or_element_type(
                crate::checker::checker_iteration::IterationUse::Destructuring,
                &t,
                pattern.as_ref(),
            ));
        }
        None
    }

    /// rest 绑定元素类型：对象模式取排除同模式其他绑定名后的剩余属性对象；
    /// 数组模式取剩余元素数组（Go getRestTypeAtObject / getRestTypeAtPosition）。
    pub(crate) fn rest_element_type(&mut self, elem: &Arc<Node>, t: &Arc<Type>) -> Arc<Type> {
        let is_array_pattern = elem
            .parent()
            .is_some_and(|p| p.kind == tsox_frontend::ast::SyntaxKind::ArrayBindingPattern);
        if is_array_pattern {
            if t.is_union()
                && let Some(members) = t.types()
            {
                let mapped: Vec<Arc<Type>> = members
                    .iter()
                    .map(|m| self.rest_element_type(elem, m))
                    .collect();
                return self.get_union_type(mapped);
            }
            if self.is_tuple_type(t) {
                let index = elem
                    .parent()
                    .and_then(|pattern| Self::binding_element_index(&pattern, elem))
                    .unwrap_or(0);
                let mut rest: Vec<Arc<Type>> = Vec::new();
                let mut i = index;
                while let Some(el) = self.get_tuple_element_type(t, i) {
                    rest.push(el);
                    i += 1;
                }
                let element_t = if rest.is_empty() {
                    self.never_type()
                } else {
                    self.get_union_type(rest)
                };
                return self.create_array_type(element_t);
            }
            if let Some(el) = self.get_array_element_type_of(t) {
                return self.create_array_type(el);
            }
            return Arc::clone(t);
        }
        let source = self.remove_nullable_constituents(t);
        if source.flags.contains(TypeFlags::Never) {
            return Arc::new(Type::new(
                TypeFlags::Object,
                TypeData::Object(ObjectTypeData::default()),
            ));
        }
        if source.is_union()
            && let Some(members) = source.types()
        {
            let mapped: Vec<Arc<Type>> = members
                .iter()
                .map(|m| self.rest_element_type(elem, m))
                .collect();
            return self.get_union_type(mapped);
        }
        let excluded = Self::pattern_excluded_property_names(elem);
        let source_props = self.get_properties_of_type(&source);
        let kept: Vec<Arc<Symbol>> = source_props
            .into_iter()
            .filter(|p| !excluded.contains(&p.name))
            .filter(|p| self.is_spreadable_property(p))
            .collect();
        let index_infos = self.get_index_infos_of_type(&source);
        let mut members = crate::checker::types::SymbolTable::default();
        for p in &kept {
            members.insert(p.name.clone(), Arc::clone(p));
        }
        let mut rebuilt = Type::new(
            TypeFlags::Object,
            TypeData::Object(ObjectTypeData {
                structured: crate::checker::types_impl_chunk::StructuredTypeData {
                    members,
                    properties: kept,
                    index_infos,
                    ..Default::default()
                },
                ..Default::default()
            }),
        );
        rebuilt.object_flags = source.object_flags | crate::checker::types::ObjectFlags::ObjectRestType;
        Arc::new(rebuilt)
    }

    fn remove_nullable_constituents(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if !t.is_union() {
            if t.flags.intersects(TYPE_FLAGS_NULLABLE) {
                return self.never_type();
            }
            return Arc::clone(t);
        }
        let Some(members) = t.types() else {
            return Arc::clone(t);
        };
        let kept: Vec<Arc<Type>> = members
            .iter()
            .filter(|m| !m.flags.intersects(TYPE_FLAGS_NULLABLE))
            .cloned()
            .collect();
        if kept.len() == members.len() {
            return Arc::clone(t);
        }
        match kept.len() {
            0 => self.never_type(),
            1 => Arc::clone(&kept[0]),
            _ => self.get_union_type(kept),
        }
    }

    pub(crate) fn is_spreadable_property(&self, prop: &Arc<Symbol>) -> bool {
        let no_private_ident = !prop.declarations.iter().any(|d| {
            matches!(
                &d.data,
                NodeData::PropertyDeclaration(pd) if pd.name.kind == tsox_frontend::ast::SyntaxKind::PrivateIdentifier
            ) || Self::is_method_or_accessor(d)
                && d.name().is_some_and(|n| n.kind == tsox_frontend::ast::SyntaxKind::PrivateIdentifier)
        });
        let not_accessor_like = !prop.flags.intersects(
            tsox_frontend::ast::SymbolFlags::Method
                | tsox_frontend::ast::SymbolFlags::GetAccessor
                | tsox_frontend::ast::SymbolFlags::SetAccessor,
        );
        if no_private_ident && not_accessor_like {
            return true;
        }
        !prop
            .declarations
            .iter()
            .any(|d| d.parent().is_some_and(|p| tsox_frontend::ast::is_class_like(&p)))
    }

    fn is_method_or_accessor(node: &Arc<Node>) -> bool {
        use tsox_frontend::ast::SyntaxKind as K;
        matches!(node.kind, K::MethodDeclaration | K::MethodSignature | K::GetAccessor | K::SetAccessor)
    }

    /// 同一对象模式中其他元素绑定的属性名（rest 类型需排除这些属性）
    fn pattern_excluded_property_names(elem: &Arc<Node>) -> Vec<String> {
        let mut out = Vec::new();
        let Some(pattern) = elem
            .parent()
            .filter(|p| p.kind == tsox_frontend::ast::SyntaxKind::ObjectBindingPattern)
        else {
            return out;
        };
        let NodeData::BindingPattern(bp) = &pattern.data else {
            return out;
        };
        for e in bp.elements.nodes.iter() {
            if Arc::ptr_eq(e, elem) {
                continue;
            }
            let NodeData::BindingElement(be) = &e.data else {
                continue;
            };
            if let Some(pn) = &be.property_name {
                match &pn.data {
                    NodeData::Identifier(i) => out.push(i.text.clone()),
                    NodeData::StringLiteral(s) => out.push(s.text.clone()),
                    NodeData::NumericLiteral(n) => out.push(n.text.clone()),
                    _ => {}
                }
            } else if let Some(n) = &be.name
                && n.kind == tsox_frontend::ast::SyntaxKind::Identifier
            {
                out.push(n.text().to_string());
            }
        }
        out
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
