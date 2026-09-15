#![allow(unused_imports)]

use crate::checker::typenode_references::*;

impl Checker {
    pub(crate) fn get_type_from_this_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let constraint = self.container_instance_type_of(node);
        let result = self.create_this_type(node, constraint);
        self.cache_type(node, result.clone());
        result
    }

    pub(crate) fn container_instance_type_of(&mut self, node: &Arc<Node>) -> Arc<Type> {
        // 从 container 自身查起：调用方传入的即是类/接口容器（顶层类的
        // parent 是 SourceFile，跳过自身会一直走到 any）
        let mut cur: Option<Arc<Node>> = Some(Arc::clone(node));
        while let Some(n) = cur {
            match n.kind {
                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression => {
                    return self.build_class_instance_type_with_base(&n);
                }
                SyntaxKind::InterfaceDeclaration => {
                    return self.interface_declaration_type(&n);
                }
                _ => {}
            }
            cur = n.parent();
        }
        self.get_any_type()
    }

    #[allow(dead_code)]
    pub(crate) fn polymorphic_this_of(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> {
        let mut cur = node.parent();
        while let Some(n) = cur {
            match n.kind {
                SyntaxKind::ArrowFunction => {}
                SyntaxKind::MethodDeclaration
                | SyntaxKind::Constructor
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor
                | SyntaxKind::MethodSignature => {
                    let container = n.parent()?;
                    match container.kind {
                        SyntaxKind::ClassDeclaration
                        | SyntaxKind::ClassExpression
                        | SyntaxKind::InterfaceDeclaration => {
                            let instance = self.container_instance_type_of(&container);
                            return Some(self.create_this_type(&container, instance));
                        }
                        _ => return None,
                    }
                }
                SyntaxKind::FunctionExpression | SyntaxKind::FunctionDeclaration => {
                    return None;
                }
                _ => {}
            }
            cur = n.parent();
        }
        None
    }

    // 对象字面量方法的 this：取上下文签名 this 参数类型（Go getContextualThisParameterType 上下文敏感分支）；
    // 多态 this 只在接口声明内有意义，字面量方法中转为其约束
    #[allow(dead_code)]
    pub(crate) fn object_literal_method_contextual_this(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        let mut cur = node.parent();
        while let Some(n) = cur {
            match n.kind {
                SyntaxKind::ArrowFunction => {}
                SyntaxKind::FunctionExpression | SyntaxKind::FunctionDeclaration => return None,
                SyntaxKind::MethodDeclaration => {
                    let container = n.parent()?;
                    if container.kind != SyntaxKind::ObjectLiteralExpression {
                        return None;
                    }
                    let name = match &n.data {
                        NodeData::MethodDeclaration(d) => {
                            self.get_property_name_from_node(&d.name)
                        }
                        _ => return None,
                    };
                    // 上下文签名带 this 参数：取其约束（多态 this）
                    if let Some(ctx) = self.get_contextual_type(&container, ContextFlags::None) {
                        if let Some(prop_type) =
                            self.get_type_of_property_of_contextual_type(&ctx, &name)
                        {
                            if let Some(sig) = self
                                .get_signatures_of_type(
                                    &prop_type,
                                    crate::checker::SignatureKind::Call,
                                )
                                .into_iter()
                                .next()
                            {
                                if let Some(this_param) = sig.this_parameter.clone() {
                                    let t = self.get_type_of_symbol(&this_param);
                                    if crate::checker::utilities::is_this_type_parameter(&t)
                                        && let Some(constraint) =
                                            self.get_constraint_of_type_parameter(&t)
                                    {
                                        return Some(constraint);
                                    }
                                    return Some(t);
                                }
                            }
                        }
                    }
                    // Go getContextualThisParameterType：无上下文时回退到
                    // 对象字面量自身类型（widened），方法成员可经 this.test 解析
                    let literal_type = self.get_type_of_object_literal(&container);
                    return Some(self.get_widened_type(&literal_type));
                }
                _ => {}
            }
            cur = n.parent();
        }
        None
    }

    pub(crate) fn create_this_type(
        &mut self,
        container: &Arc<Node>,
        instance: Arc<Type>,
    ) -> Arc<Type> {
        let mut t = Type::new(
            TypeFlags::TypeParameter,
            TypeData::TypeParameter(TypeParameterData {
                constrained: ConstrainedTypeData::default(),
                constraint: Some(instance),
                target: None,
                mapper: None,
                is_this_type: true,
                resolved_default_type: OnceLock::new(),
            }),
        );
        t.symbol = self.program.symbol_map().symbol_of(container).cloned();
        Arc::new(t)
    }

    fn interface_declaration_type(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(sym) = self.program.symbol_map().symbol_of(node).cloned() {
            if let Some(t) = self.type_alias_links.get(&sym).and_then(|l| l.declared_type.clone())
            {
                return t;
            }
            // 重入时 _ex 返回 pending 壳，不能作为声明类型缓存（_ex 内部按 cache_result 自管）
            return self.resolve_interface_type_ex(&sym, None);
        }
        self.get_any_type()
    }

    #[allow(dead_code)]
    pub(crate) fn explicit_this_parameter_type(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> {
        let mut cur = node.parent();
        while let Some(n) = cur {
            let params = match &n.data {
                NodeData::MethodDeclaration(d) => Some(&d.parameters),
                NodeData::FunctionExpression(d) => Some(&d.parameters),
                NodeData::FunctionDeclaration(d) => Some(&d.parameters),
                NodeData::ArrowFunction(d) => Some(&d.parameters),
                _ => None,
            };
            if let Some(params) = params
                && let Some(first) = params.iter().next()
                && let NodeData::ParameterDeclaration(pd) = &first.data
                && (matches!(&pd.name.data, NodeData::Identifier(id) if id.text == "this")
                    || pd.name.kind == SyntaxKind::ThisKeyword)
                && let Some(tn) = &pd.type_node
            {
                return Some(self.get_type_from_type_node(tn));
            }
            cur = n.parent();
        }
        None
    }

    pub(crate) fn get_type_from_literal_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let literal = match &node.data {
            NodeData::LiteralTypeNode(data) => &data.literal,
            _ => return self.error_type(),
        };
        if literal.kind == SyntaxKind::NullKeyword {
            return self.null_type();
        }
        let result = self.literal_type_from_literal_node(literal);
        self.cache_type(node, result.clone());
        result
    }

    pub(crate) fn literal_type_from_literal_node(&mut self, literal: &Arc<Node>) -> Arc<Type> {
        match literal.kind {
            SyntaxKind::StringLiteral => self.get_string_literal_type(literal.text()),
            SyntaxKind::NumericLiteral => {
                if let Ok(n) = literal.text().parse::<f64>() {
                    self.get_number_literal_type(tsox_core::jsnum::Number::from(n))
                } else {
                    self.number_type()
                }
            }
            SyntaxKind::BigIntLiteral => {
                let text = literal.text();
                if let Some(t) = self.bigint_literal_types.get(text).cloned() {
                    return t;
                }
                let (neg, digits) = if let Some(rest) = text.strip_prefix('-') {
                    (true, rest.trim_end_matches('n'))
                } else {
                    (false, text.trim_end_matches('n'))
                };
                let t = Arc::new(Type::new(
                    TypeFlags::BigIntLiteral,
                    TypeData::Literal(LiteralTypeData {
                        value: LiteralValue::BigInt(tsox_core::jsnum::PseudoBigInt::new(
                            digits, neg,
                        )),
                        fresh_type: std::sync::OnceLock::new(),
                        regular_type: std::sync::OnceLock::new(),
                    }),
                ));
                self.bigint_literal_types
                    .insert(text.to_string(), Arc::clone(&t));
                t
            }
            SyntaxKind::TrueKeyword => self.true_type(),
            SyntaxKind::FalseKeyword => self.false_type(),
            _ => self.error_type(),
        }
    }

    pub(crate) fn get_type_from_type_reference(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }
        let result = self.resolve_type_reference(node);
        // 解析重入期返回的空壳（接口/类符号在册但成员空）不进节点缓存：
        // 缓存会把残缺型钉死，后续引用全部拿到幻影（tsc 重试拿完整版）
        let incomplete = result.as_structured().is_some_and(|s| {
            s.members.entries.is_empty()
        }) && result
            .symbol
            .as_ref()
            .is_some_and(|sym| {
                sym.flags.intersects(
                    tsox_frontend::ast::SymbolFlags::Interface
                        | tsox_frontend::ast::SymbolFlags::Class,
                ) && !sym.declarations.is_empty()
            });
        // 类成员填充窗口内的解析结果可能取到半成品 attach 快照，同样不缓存
        // 类成员填充窗口内的解析结果可能取到半成品 attach 快照，同样不缓存
        if !incomplete && self.filling_class_members.is_empty() {
            self.cache_type(node, result.clone());
        }
        result
    }
}
